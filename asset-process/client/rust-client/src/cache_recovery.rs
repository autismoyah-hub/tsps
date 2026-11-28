//! Shared read-only JS5 recovery with bounded request waves.
//! Requests are deduplicated; disk files are never changed by this worker.
use crate::{cache::GroupInfo, crypto, js5, Error, Result};
use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
    sync::{
        mpsc::{self, SyncSender},
        Arc, Condvar, Mutex,
    },
    thread,
    time::{Duration, Instant},
};
// Only the master index has no externally supplied CRC/version pair.
type Key = (u8, u16, Option<(u32, u32)>);

#[derive(Clone)]
pub struct MasterIndex {
    entries: Vec<(u32, u32)>,
}
impl MasterIndex {
    pub fn parse(bytes: &[u8], max_size: usize) -> Result<Self> {
        let container = crate::cache::Container::parse(bytes, max_size)?;
        let payload = container.uncompressed()?;
        let entries = payload
            .chunks_exact(8)
            .map(|entry| {
                (
                    u32::from_be_bytes(entry[..4].try_into().unwrap()),
                    u32::from_be_bytes(entry[4..].try_into().unwrap()),
                )
            })
            .collect();
        Ok(Self { entries })
    }
    /// uy.hu treats absent entries and the zero/zero pair as an absent archive.
    pub fn entry(&self, archive: u8) -> Option<(u32, u32)> {
        self.entries
            .get(archive as usize)
            .copied()
            .filter(|&v| v != (0, 0))
    }
}

/// ui.fa checks the complete reference container CRC, then format and revision.
/// Group/file table parsing is a separate step after this validation.
pub fn validate_reference(bytes: &[u8], crc: u32, version: u32, max_size: usize) -> Result<()> {
    if crypto::crc32(bytes) != crc {
        return Err(Error::Invalid("reference CRC mismatch"));
    }
    let data = crate::cache::Container::parse(bytes, max_size)?.decompress()?;
    let mut reader = crate::buffer::Reader::new(&data);
    let format = reader.u8()?;
    if !(5..=7).contains(&format) {
        return Err(Error::Unsupported("archive index format"));
    }
    let actual = if format >= 6 { reader.u32()? } else { 0 };
    if actual != version {
        return Err(Error::Invalid("reference version mismatch"));
    }
    Ok(())
}
enum Entry {
    Pending,
    Ready(Arc<Vec<u8>>),
    Failed(Instant, String),
}
#[derive(Default)]
struct State {
    entries: HashMap<Key, Entry>,
    order: VecDeque<Key>,
    bytes: usize,
    requests: u64,
    completed: u64,
    failures: u64,
}
#[derive(Clone)]
pub struct Recovery {
    shared: Arc<(Mutex<State>, Condvar)>,
    sender: SyncSender<Vec<Key>>,
}
impl Recovery {
    pub fn connect(
        address: SocketAddr,
        revision: u32,
        key: [u32; 4],
        max_size: usize,
    ) -> Result<Self> {
        let shared = Arc::new((Mutex::new(State::default()), Condvar::new()));
        // At most 200 groups queued, even when every message is a full wave.
        let (sender, receiver) = mpsc::sync_channel::<Vec<Key>>(200 / js5::MAX_BATCH);
        let worker = shared.clone();
        thread::Builder::new()
            .name("js5-recovery".into())
            .spawn(move || {
                let mut client: Option<js5::Client> = None;
                while let Ok(requests) = receiver.recv() {
                    let fetched = (|| -> std::result::Result<Vec<Vec<u8>>, String> {
                        if client.is_none() {
                            client = Some(
                                js5::Client::connect(
                                    address,
                                    revision,
                                    key,
                                    Duration::from_secs(3),
                                    max_size,
                                )
                                .map_err(|e| e.to_string())?,
                            );
                        }
                        client
                            .as_mut()
                            .unwrap()
                            .fetch_batch(&requests.iter().map(|r| (r.0, r.1)).collect::<Vec<_>>())
                            .map_err(|e| e.to_string())
                    })();
                    let results = match fetched {
                        Ok(containers) => containers.into_iter().map(Ok).collect::<Vec<_>>(),
                        Err(error) => {
                            client = None;
                            requests.iter().map(|_| Err(error.clone())).collect()
                        }
                    };
                    for (request, fetched) in requests.into_iter().zip(results) {
                        let result = (|| -> std::result::Result<Vec<u8>, String> {
                            let mut bytes = fetched?;
                            if let Some((crc, version)) = request.2 {
                                if request.0 == 255 {
                                    validate_reference(&bytes, crc, version, max_size)
                                        .map_err(|e| e.to_string())?;
                                } else {
                                    if crypto::crc32(&bytes) != crc {
                                        return Err("JS5 group CRC mismatch".into());
                                    }
                                    bytes.extend_from_slice(&version.to_be_bytes());
                                }
                            } else {
                                MasterIndex::parse(&bytes, max_size).map_err(|e| e.to_string())?;
                            }
                            Ok(bytes)
                        })();
                        let (lock, wake) = &*worker;
                        let Ok(mut state) = lock.lock() else { break };
                        match result {
                            Ok(bytes) => {
                                // Bound retained compressed containers independently of model caches.
                                // Include the largest container header and version suffix;
                                // otherwise a full wave of maximum-size groups evicts itself.
                                let budget =
                                    max_size.saturating_add(13).saturating_mul(js5::MAX_BATCH);
                                while state.bytes.saturating_add(bytes.len()) > budget {
                                    let Some(old) = state.order.pop_front() else {
                                        break;
                                    };
                                    if let Some(Entry::Ready(data)) = state.entries.remove(&old) {
                                        state.bytes -= data.len();
                                    }
                                }
                                state.bytes += bytes.len();
                                state.completed += 1;
                                state.order.push_back(request);
                                state.entries.insert(request, Entry::Ready(Arc::new(bytes)));
                            }
                            Err(error) => {
                                client = None;
                                state.failures += 1;
                                state.entries.insert(
                                    request,
                                    Entry::Failed(Instant::now() + Duration::from_secs(1), error),
                                );
                            }
                        }
                        wake.notify_all();
                    }
                }
            })?;
        Ok(Self { shared, sender })
    }
    /// Most gameplay consumers use zero wait. Bootstrap, world loading and the
    /// current synchronous VM compatibility path may use a bounded wait.
    pub fn group(
        &self,
        archive: u8,
        info: &GroupInfo,
        wait: Duration,
    ) -> Result<Option<Arc<Vec<u8>>>> {
        let group =
            u16::try_from(info.id).map_err(|_| Error::Unsupported("JS5 group identifier"))?;
        self.request((archive, group, Some((info.crc, info.version))), wait)
    }

    /// Atomically queue a bounded wave; ready/pending groups are deduplicated.
    /// A full queue leaves all new groups unqueued so a later scan can retry.
    pub fn prefetch_groups(&self, archive: u8, groups: &[GroupInfo]) -> Result<()> {
        if groups.len() > js5::MAX_BATCH {
            return Err(Error::Invalid("JS5 recovery batch size"));
        }
        let keys = groups
            .iter()
            .map(|info| {
                let group = u16::try_from(info.id)
                    .map_err(|_| Error::Unsupported("JS5 group identifier"))?;
                Ok((archive, group, Some((info.crc, info.version))))
            })
            .collect::<Result<Vec<Key>>>()?;
        for (i, key) in keys.iter().enumerate() {
            if keys[..i].iter().any(|old| old.0 == key.0 && old.1 == key.1) {
                return Err(Error::Invalid("duplicate JS5 recovery batch group"));
            }
        }
        let mut state = self
            .shared
            .0
            .lock()
            .map_err(|_| Error::Invalid("JS5 recovery lock"))?;
        let now = Instant::now();
        let pending = keys
            .into_iter()
            .filter(|key| match state.entries.get(key) {
                Some(Entry::Ready(_) | Entry::Pending) => false,
                Some(Entry::Failed(retry, _)) if now < *retry => false,
                _ => true,
            })
            .collect::<Vec<_>>();
        if pending.is_empty() {
            return Ok(());
        }
        match self.sender.try_send(pending.clone()) {
            Ok(()) => {
                state.requests += pending.len() as u64;
                for key in pending {
                    state.entries.insert(key, Entry::Pending);
                }
                Ok(())
            }
            Err(mpsc::TrySendError::Full(_)) => Ok(()),
            Err(mpsc::TrySendError::Disconnected(_)) => Err(Error::Invalid("JS5 recovery stopped")),
        }
    }
    pub fn master(&self, wait: Duration) -> Result<Option<Arc<Vec<u8>>>> {
        self.request((255, 255, None), wait)
    }
    pub fn reference(
        &self,
        archive: u8,
        crc: u32,
        version: u32,
        wait: Duration,
    ) -> Result<Option<Arc<Vec<u8>>>> {
        self.request((255, u16::from(archive), Some((crc, version))), wait)
    }
    fn request(&self, key: Key, wait: Duration) -> Result<Option<Arc<Vec<u8>>>> {
        let deadline = Instant::now() + wait;
        let (lock, wake) = &*self.shared;
        let mut state = lock
            .lock()
            .map_err(|_| Error::Invalid("JS5 recovery lock"))?;
        loop {
            let now = Instant::now();
            match state.entries.get(&key) {
                Some(Entry::Ready(bytes)) => return Ok(Some(bytes.clone())),
                Some(Entry::Pending) => {}
                Some(Entry::Failed(retry, _)) if now < *retry => {}
                _ => match self.sender.try_send(vec![key]) {
                    Ok(()) => {
                        state.requests += 1;
                        state.entries.insert(key, Entry::Pending);
                    }
                    Err(mpsc::TrySendError::Full(_)) => {}
                    Err(mpsc::TrySendError::Disconnected(_)) => {
                        return Err(Error::Invalid("JS5 recovery stopped"))
                    }
                },
            }
            if now >= deadline {
                return match state.entries.get(&key) {
                    Some(Entry::Failed(_, error)) if !wait.is_zero() => {
                        Err(Error::Io(error.clone()))
                    }
                    _ => Ok(None),
                };
            }
            let interval = deadline
                .saturating_duration_since(now)
                .min(Duration::from_millis(100));
            state = wake
                .wait_timeout(state, interval)
                .map_err(|_| Error::Invalid("JS5 recovery lock"))?
                .0;
        }
    }
    pub fn stats(&self) -> (u64, u64, u64, usize) {
        self.shared
            .0
            .lock()
            .map(|s| (s.requests, s.completed, s.failures, s.bytes))
            .unwrap_or_default()
    }
}
