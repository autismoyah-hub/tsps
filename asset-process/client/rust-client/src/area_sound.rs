//! Area effect admission and attenuation from GameSceneTilePaint.dc / rr / nf.ag.
use crate::{
    buffer::{Reader, Transform},
    Error, Result,
};
#[derive(Clone, Debug)]
pub struct Request {
    pub id: u16,
    pub loops: u8,
    pub delay: u8,
    pub radius: u8,
    pub inner: u8,
    pub tile: [i32; 2],
    pub packed: i32,
}
impl Request {
    pub fn decode(origin: [i32; 3], base: [i32; 2], raw: &[u8]) -> Result<Self> {
        let mut r = Reader::new(raw);
        let at = r.transformed(Transform::Sub)?;
        let inner = r.transformed(Transform::Neg)?;
        let radius = r.u8()? & 31;
        let delay = r.transformed(Transform::Neg)?;
        let loops = r.transformed(Transform::Add)?;
        let id = r.u16()?;
        if r.remaining() != 0 {
            return Err(Error::Invalid("area sound trailing bytes"));
        }
        let local = [
            origin[1] + ((at >> 4) & 7) as i32,
            origin[2] + (at & 7) as i32,
        ];
        Ok(Self {
            id,
            loops,
            delay,
            radius,
            inner,
            tile: [base[0] + local[0], base[1] + local[1]],
            packed: (local[0] << 16) + (local[1] << 8) + radius as i32,
        })
    }
    pub fn admitted(&self, effects: i32, area: i32, count: usize) -> bool {
        area != 0 && self.loops > 0 && count < 50 && (self.radius != 0 || effects != 0)
    }
    pub fn volume(&self, listener: [i32; 2], effects: i32, area: i32) -> i32 {
        if self.packed == 0 {
            return effects;
        }
        let distance = ((self.tile[0] * 128 + 64 - listener[0]).abs()
            + (self.tile[1] * 128 + 64 - listener[1]).abs()
            - 128)
            .max(0);
        let radius = (self.radius as i32) << 7;
        if distance >= radius {
            return 0;
        }
        let inner = (((self.inner as i32 & 31) - 1) << 7).max(0);
        let scale = if inner < radius {
            ((radius - distance) as f32 / (radius - inner) as f32).clamp(0.0, 1.0)
        } else {
            1.0
        };
        (area as f32 * scale).ceil() as i32
    }
}
