//! Original GameClient mouse recorder packet110 and 500-sample queue.
use crate::{protocol::Packet, writer::Writer};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    pub point: [i32; 2],
    pub millis: i64,
}
#[derive(Debug)]
pub struct Recorder {
    pub samples: Vec<Sample>,
    pub last_point: [i32; 2],
    pub last_millis: i64,
}
impl Default for Recorder {
    fn default() -> Self {
        Self {
            samples: Vec::new(),
            last_point: [-1, -1],
            last_millis: -1,
        }
    }
}
impl Recorder {
    pub fn record(&mut self, sample: Sample) {
        if self.samples.len() < 500 {
            self.samples.push(sample);
        }
    }
    pub fn send(&mut self, enabled: bool, clicked: bool) -> Option<Packet> {
        if !enabled {
            self.samples.clear();
            return None;
        }
        if !clicked && self.samples.len() < 40 {
            return None;
        }
        let mut writer: Option<Writer> = None;
        let mut remainder = 0i32;
        let mut moved = 0;
        let mut retained_from = 0;
        for (index, sample) in self.samples.iter().enumerate() {
            if writer.as_ref().is_some_and(|w| w.len() >= 246) {
                break;
            }
            retained_from = index;
            let point = sample.point.map(|v| v.clamp(-1, 65534));
            if point == self.last_point {
                continue;
            }
            let w = writer.get_or_insert_with(|| {
                let mut w = Writer::new();
                w.bytes(&[0, 0]);
                w
            });
            let (delta, time) = if self.last_millis != -1 {
                let elapsed = sample.millis.wrapping_sub(self.last_millis);
                remainder = remainder.wrapping_add((elapsed % 20) as i32);
                (
                    [
                        point[0].wrapping_sub(self.last_point[0]),
                        point[1].wrapping_sub(self.last_point[1]),
                    ],
                    (elapsed / 20) as i32,
                )
            } else {
                (point, i32::MAX)
            };
            self.last_point = point;
            if time < 8 && delta.iter().all(|v| (-32..=31).contains(v)) {
                w.u16(
                    ((time << 12)
                        .wrapping_add((delta[0] + 32) << 6)
                        .wrapping_add(delta[1] + 32)) as u16,
                );
            } else if time < 32 && delta.iter().all(|v| (-128..=127).contains(v)) {
                w.u8((128i32.wrapping_add(time)) as u8);
                w.u16((((delta[0] + 128) << 8) + (delta[1] + 128)) as u16);
            } else {
                if time < 32 {
                    w.u8((192i32.wrapping_add(time)) as u8);
                } else {
                    w.u16(((time & 8191) + 57344) as u16);
                }
                w.u32(if point.contains(&-1) {
                    0x80000000
                } else {
                    (point[0] | point[1] << 16) as u32
                });
            }
            moved += 1;
            self.last_millis = sample.millis;
        }
        // The original retains the last visited sample, including when no point moved.
        self.samples.drain(..retained_from);
        writer.map(|w| {
            let mut payload = w.into_vec();
            payload[0] = (remainder / moved) as u8;
            payload[1] = (remainder % moved) as u8;
            Packet {
                opcode: 110,
                payload,
            }
        })
    }
}
