//! GameClient.aft wheel and focus reports (packets16 and29).
use crate::protocol::Packet;
#[derive(Debug)]
pub struct State {
    pub reported_focus: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            reported_focus: true,
        }
    }
}
impl State {
    pub fn wheel(delta: i32) -> Option<Packet> {
        (delta != 0).then(|| Packet {
            opcode: 16,
            payload: (delta as u16).to_be_bytes().to_vec(),
        })
    }
    pub fn focus(&mut self, focused: bool) -> Option<Packet> {
        if focused == self.reported_focus {
            return None;
        }
        self.reported_focus = focused;
        Some(Packet {
            opcode: 29,
            payload: vec![u8::from(focused)],
        })
    }
}
