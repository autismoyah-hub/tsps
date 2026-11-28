//! GamePreferences versions 0..12 and insertion-ordered remembered login tokens.
use crate::{buffer::Reader, device_options::DeviceOptions, writer::Writer, Result};

pub struct SavedPreferences {
    pub device: DeviceOptions,
    pub window_mode: i32,
    pub remembered_username: Option<String>,
    pub accepted_eula: i32,
    tokens: Vec<(u32, u32)>,
}
impl Default for SavedPreferences {
    fn default() -> Self {
        Self {
            device: DeviceOptions::default(),
            window_mode: 1,
            remembered_username: None,
            accepted_eula: -1,
            tokens: Vec::new(),
        }
    }
}
impl SavedPreferences {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut p = Self::default();
        if bytes.is_empty() {
            return Ok(p);
        }
        let mut r = Reader::new(bytes);
        let version = r.u8()?;
        if version > 12 {
            return Ok(p);
        }
        p.device.roofs_hidden = r.u8()? == 1;
        if version > 1 {
            p.device.title_music_disabled = r.u8()? == 1;
        }
        if version > 3 {
            p.window_mode = r.u8()? as i32;
        }
        if version > 2 {
            for _ in 0..r.u8()? {
                let key = r.u32()?;
                let value = r.u32()?;
                p.put(key, value);
            }
        }
        if version > 4 {
            let name = r.string()?;
            p.remembered_username = (!name.is_empty()).then_some(name);
        }
        if version > 5 {
            p.device.hide_username = r.u8()? & 1 != 0;
        }
        if version > 6 {
            p.device.brightness = r.u8()? as f64 / 100.0;
            for volume in &mut p.device.volumes {
                *volume = r.u8()? as i32;
            }
        }
        if version > 7 {
            p.accepted_eula = r.u8()? as i32;
        }
        if version > 8 {
            p.device.display_fps = r.u8()? == 1;
        }
        if version > 9 {
            p.device.extra = r.u32()? as i32;
        }
        if version > 10 {
            p.device.draw_distance = r.u8()? as i32;
        }
        if version > 11 {
            r.u8()?;
        } // Injected client deliberately disables this option.
        Ok(p)
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut w = Writer::new();
        w.u8(12);
        w.u8(self.device.roofs_hidden as u8);
        w.u8(self.device.title_music_disabled as u8);
        w.u8(self.window_mode as u8);
        w.u8(self.tokens.len() as u8);
        for &(key, value) in &self.tokens {
            w.u32(key);
            w.u32(value);
        }
        w.string(self.remembered_username.as_deref().unwrap_or(""))?;
        w.u8(self.device.hide_username as u8);
        w.u8((self.device.brightness * 100.0) as i32 as u8);
        for volume in self.device.volumes {
            w.u8(volume as u8);
        }
        w.u8(self.accepted_eula as u8);
        w.u8(self.device.display_fps as u8);
        w.u32(self.device.extra as u32);
        w.u8(self.device.draw_distance as u8);
        w.u8(self.device.option22 as u8);
        Ok(w.into_vec())
    }
    fn put(&mut self, key: u32, value: u32) {
        if let Some(entry) = self.tokens.iter_mut().find(|entry| entry.0 == key) {
            entry.1 = value;
        } else {
            self.tokens.push((key, value));
        }
    }
    pub fn token(&self, username: &str) -> Option<u32> {
        let key = username_hash(username);
        self.tokens
            .iter()
            .find(|entry| entry.0 == key)
            .map(|entry| entry.1)
    }
    pub fn remember_token(&mut self, username: &str, token: u32) {
        let key = username_hash(username);
        if self.tokens.len() >= 10 && !self.tokens.iter().any(|entry| entry.0 == key) {
            self.tokens.remove(0);
        }
        self.put(key, token);
    }
}
/// Matches the English-locale Java hash for native title input. Locale-specific
/// Unicode lowercasing outside the title's character whitelist remains separate.
pub fn username_hash(username: &str) -> u32 {
    username
        .to_lowercase()
        .encode_utf16()
        .fold(0u32, |h, c| h.wrapping_mul(31).wrapping_add(c as u32))
}

/// A native-client profile, isolated by world address and public RSA key.
/// Passwords and one-time authenticator codes are never stored here.
pub struct Profile {
    path: std::path::PathBuf,
    previous: Vec<u8>,
}
impl Profile {
    pub fn open(
        root: &std::path::Path,
        address: &str,
        public_key: &[u8],
    ) -> Result<(Self, Option<Vec<u8>>)> {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        hash.update(address.as_bytes());
        hash.update([0]);
        hash.update(public_key);
        let name: String = hash.finalize().iter().map(|b| format!("{b:02x}")).collect();
        let path = root.join(format!("{name}.dat"));
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => Some(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        Ok((
            Self {
                path,
                previous: bytes.clone().unwrap_or_default(),
            },
            bytes,
        ))
    }
    pub fn save(&mut self, bytes: &[u8]) -> Result<()> {
        if bytes == self.previous {
            return Ok(());
        }
        use std::io::Write;
        std::fs::create_dir_all(self.path.parent().unwrap())?;
        let temporary = self
            .path
            .with_extension(format!("{}.tmp", std::process::id()));
        let mut file = std::fs::File::create(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(temporary, &self.path)?;
        self.previous = bytes.to_vec();
        Ok(())
    }
}
