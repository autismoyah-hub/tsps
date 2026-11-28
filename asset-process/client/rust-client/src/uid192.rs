//! Packet113 CRC validation and the original24-byte cached/persistent login identifier.
use crate::{crypto::crc32, Error, Result};
use std::{
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};
/// GameClient.ib checks the entire24-byte value before calling jg.hf.
pub fn decode(bytes: &[u8]) -> Result<Option<[u8; 24]>> {
    if bytes.len() != 28 {
        return Err(Error::Invalid("UID192 packet length"));
    }
    let check = u32::from_be_bytes(bytes[24..].try_into().unwrap());
    Ok((crc32(&bytes[..24]) == check).then(|| bytes[..24].try_into().unwrap()))
}
#[derive(Default)]
pub struct State {
    cached: Option<[u8; 24]>,
    file: Option<std::cell::RefCell<std::fs::File>>,
    storage_initialized: bool,
}
impl State {
    pub fn with_path(path: PathBuf) -> Self {
        let mut state = Self::default();
        state.configure_storage(path);
        state
    }
    pub fn configure_storage(&mut self, path: PathBuf) {
        if self.storage_initialized {
            return;
        }
        self.storage_initialized = true;
        let open = || -> std::io::Result<std::fs::File> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(path)?;
            // mn's25-byte random-file bound discards oversized contents at open.
            if file.metadata()?.len() > 25 {
                file.set_len(0)?;
            }
            Ok(file)
        };
        self.file = open().ok().map(std::cell::RefCell::new);
    }
    pub fn cached(&self) -> Option<&[u8; 24]> {
        self.cached.as_ref()
    }
    /// gc.he prefers the server-updated memory value, including an all-zero value.
    /// ee.ax rereads24 bytes otherwise; missing/short/all-zero files yield allFF.
    pub fn login_bytes(&self) -> [u8; 24] {
        if let Some(value) = self.cached {
            return value;
        }
        let read = || -> std::io::Result<[u8; 24]> {
            let storage = self.file.as_ref().ok_or(std::io::ErrorKind::NotFound)?;
            let mut file = storage.borrow_mut();
            file.seek(SeekFrom::Start(0))?;
            let mut value = [0; 24];
            file.read_exact(&mut value)?;
            if value.iter().all(|&b| b == 0) {
                return Err(std::io::ErrorKind::InvalidData.into());
            }
            Ok(value)
        };
        read().unwrap_or([255; 24])
    }
    /// jg.hf updates memory first. File failures do not abort packet processing.
    pub fn receive(&mut self, value: Option<[u8; 24]>) {
        let Some(value) = value else {
            return;
        };
        self.cached = Some(value);
        if let Some(storage) = &self.file {
            let persist = || -> std::io::Result<()> {
                let mut file = storage.borrow_mut();
                file.seek(SeekFrom::Start(0))?;
                file.write_all(&value)
            };
            let _ = persist();
        }
    }
}
