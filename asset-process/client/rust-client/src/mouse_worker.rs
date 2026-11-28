//! Independent 50ms native mouse sampler (original dv.run).
use crate::{
    mouse_record::{Recorder, Sample},
    protocol::Packet,
    Result,
};
use std::{
    sync::{Arc, Condvar, Mutex},
    thread::JoinHandle,
    time::{Duration, Instant},
};
struct Shared {
    recorder: Recorder,
    current: Sample,
    pending: Sample,
    stop: bool,
    packets: u64,
}
#[derive(Clone, Copy, Debug)]
pub struct Stats {
    pub queued: usize,
    pub current: Sample,
    pub last_point: [i32; 2],
    pub packets: u64,
}
pub struct Worker {
    shared: Arc<(Mutex<Shared>, Condvar)>,
    join: Option<JoinHandle<()>>,
    epoch: Instant,
    start_millis: i64,
}
impl Worker {
    pub fn start() -> Result<Self> {
        let shared = Arc::new((
            Mutex::new(Shared {
                recorder: Recorder::default(),
                current: Sample {
                    point: [-1, -1],
                    millis: -1,
                },
                pending: Sample {
                    point: [-1, -1],
                    millis: -1,
                },
                stop: false,
                packets: 0,
            }),
            Condvar::new(),
        ));
        let thread_shared = shared.clone();
        let join = std::thread::Builder::new()
            .name("mouse-recorder".into())
            .spawn(move || {
                let (lock, wake) = &*thread_shared;
                let mut state = lock.lock().unwrap_or_else(|e| e.into_inner());
                while !state.stop {
                    let sample = state.current;
                    state.recorder.record(sample);
                    let (next, _) = wake
                        .wait_timeout_while(state, Duration::from_millis(50), |s| !s.stop)
                        .unwrap_or_else(|e| e.into_inner());
                    state = next;
                }
            })?;
        Ok(Self {
            shared,
            join: Some(join),
            epoch: Instant::now(),
            start_millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
                .min(i64::MAX as u128) as i64,
        })
    }
    pub fn pointer(&self, point: [i32; 2]) {
        let mut state = self.shared.0.lock().unwrap_or_else(|e| e.into_inner());
        state.pending = Sample {
            point,
            millis: self.millis(),
        };
    }
    pub fn latch_pointer(&self) {
        let mut state = self.shared.0.lock().unwrap_or_else(|e| e.into_inner());
        state.current = state.pending;
    }
    pub fn millis(&self) -> i64 {
        self.start_millis
            .saturating_add(self.epoch.elapsed().as_millis().min(i64::MAX as u128) as i64)
    }
    pub fn reset_queue(&self) {
        let mut state = self.shared.0.lock().unwrap_or_else(|e| e.into_inner());
        state.recorder.samples.clear();
    }
    pub fn send(&self, clicked: bool) -> (Option<Packet>, [i32; 2]) {
        let mut state = self.shared.0.lock().unwrap_or_else(|e| e.into_inner());
        let packet = state.recorder.send(true, clicked);
        if packet.is_some() {
            state.packets += 1;
        }
        (packet, state.recorder.last_point)
    }
    pub fn stats(&self) -> Stats {
        let state = self.shared.0.lock().unwrap_or_else(|e| e.into_inner());
        Stats {
            queued: state.recorder.samples.len(),
            current: state.current,
            last_point: state.recorder.last_point,
            packets: state.packets,
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        {
            let mut state = self.shared.0.lock().unwrap_or_else(|e| e.into_inner());
            state.stop = true;
        }
        self.shared.1.notify_all();
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}
