//! Editable-widget event policies (original ss, initialized by GameWidget.fe).
#[derive(Clone, Debug)]
pub struct KeyModes {
    keys: [u8; 112],
    chars: [u8; 192],
}
impl Default for KeyModes {
    fn default() -> Self {
        let mut result = Self {
            keys: [3; 112],
            chars: [3; 192],
        };
        result.keys[1..=12].fill(0);
        result.chars[..32].fill(0);
        result.chars[128] = 0;
        for key in [81, 82, 86] {
            result.keys[key] = 2;
        }
        result
    }
}
impl KeyModes {
    pub fn set_key(&mut self, key: i32, mode: i32) {
        if (0..=3).contains(&mode) {
            if let Some(slot) = usize::try_from(key).ok().and_then(|i| self.keys.get_mut(i)) {
                *slot = mode as u8;
            }
        }
    }
    pub fn set_char(&mut self, key: i32, mode: i32) {
        if (0..=3).contains(&mode) {
            if let Some(slot) = self.chars.get_mut(key as u16 as usize) {
                *slot = mode as u8;
            }
        }
    }
    pub fn key(&self, key: i32) -> u8 {
        usize::try_from(key)
            .ok()
            .and_then(|i| self.keys.get(i))
            .copied()
            .unwrap_or(0)
    }
    pub fn character(&self, key: i32) -> u8 {
        self.chars.get(key as u16 as usize).copied().unwrap_or(0)
    }
    /// Bit zero consumes the event; bit one permits editor handling.
    pub fn consumes(mode: u8) -> bool {
        mode & 1 != 0
    }
    pub fn handles(mode: u8) -> bool {
        mode & 2 != 0
    }
}
