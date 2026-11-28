//! GameClient.aft packet54 keyboard timing report.
use crate::protocol::Packet;
#[derive(Debug)]
pub struct State {
    pub last_millis: i64,
}
impl Default for State {
    fn default() -> Self {
        Self { last_millis: -1 }
    }
}
impl State {
    pub fn send(&mut self, idle_age: i32, keys: &[i32], millis: i64) -> Option<Packet> {
        if idle_age > 1 {
            return None;
        }
        let mut payload = Vec::with_capacity(keys.len() * 4);
        for &key in keys {
            let elapsed = millis.wrapping_sub(self.last_millis).min(0xff_ffff);
            self.last_millis = millis;
            payload.push((key as u8).wrapping_neg());
            payload.extend_from_slice(&(elapsed as u32).to_le_bytes()[..3]);
        }
        Some(Packet {
            opcode: 54,
            payload,
        })
    }
}
