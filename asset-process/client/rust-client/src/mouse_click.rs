//! Original GameClient.ip click reporting, packet35 EVENT_MOUSE_CLICK_V2.
use crate::{buffer::Transform, protocol::Packet, writer::Writer};
#[derive(Clone, Copy, Debug)]
pub struct Press {
    pub button: u8,
    pub point: [i32; 2],
    pub millis: i64,
}
#[derive(Debug)]
pub struct State {
    pub last_millis: i64,
    pub extra: u8,
}
impl Default for State {
    fn default() -> Self {
        Self {
            last_millis: -1,
            extra: 0,
        }
    }
}
impl State {
    pub fn send(
        &mut self,
        button: u8,
        mouse_camera: bool,
        point: [i32; 2],
        canvas: [i32; 2],
        millis: i64,
    ) -> Option<Packet> {
        if button != 1 && button != 2 && !(button == 4 && !mouse_camera) {
            return None;
        }
        let elapsed = millis.wrapping_sub(self.last_millis).min(32767) as i32;
        self.last_millis = millis;
        let point = std::array::from_fn::<_, 2, _>(|i| {
            if point[i] < 0 {
                0
            } else if point[i] > canvas[i] {
                canvas[i]
            } else {
                point[i]
            }
        });
        let mut w = Writer::new();
        w.u16_add(point[0] as u16, false);
        w.u16_add(((elapsed << 1) + i32::from(button == 2)) as u16, true);
        w.u16_add(point[1] as u16, false);
        w.transformed(self.extra, Transform::Sub);
        self.extra = 0;
        Some(Packet {
            opcode: 35,
            payload: w.into_vec(),
        })
    }
}
