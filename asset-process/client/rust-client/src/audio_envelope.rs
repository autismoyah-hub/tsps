//! Piecewise sound envelopes from ac.ag/af/ab.
use crate::{buffer::Reader, Error, Result};
#[derive(Clone, Debug)]
pub struct Envelope {
    pub form: u8,
    pub start: i32,
    pub end: i32,
    pub segments: Vec<(u16, u16)>,
}
impl Default for Envelope {
    fn default() -> Self {
        Self {
            form: 0,
            start: 0,
            end: 0,
            segments: vec![(0, 0), (65535, 65535)],
        }
    }
}
impl Envelope {
    pub fn decode(r: &mut Reader<'_>) -> Result<Self> {
        let mut e = Self {
            form: r.u8()?,
            start: r.u32()? as i32,
            end: r.u32()? as i32,
            ..Self::default()
        };
        e.decode_segments(r)?;
        Ok(e)
    }
    pub fn decode_segments(&mut self, r: &mut Reader<'_>) -> Result<()> {
        let n = r.u8()?;
        if n == 0 {
            return Err(Error::Invalid("empty sound envelope"));
        }
        self.segments = (0..n)
            .map(|_| Ok((r.u16()?, r.u16()?)))
            .collect::<Result<_>>()?;
        Ok(())
    }
}
#[derive(Default)]
pub struct Cursor {
    end: i32,
    index: usize,
    step: i32,
    amplitude: i32,
    time: i32,
}
impl Cursor {
    pub fn next(&mut self, e: &Envelope, samples: i32) -> i32 {
        if self.time >= self.end {
            self.amplitude = (e.segments[self.index].1 as i32) << 15;
            self.index = (self.index + 1).min(e.segments.len() - 1);
            self.end = (e.segments[self.index].0 as f64 / 65536.0 * samples as f64) as i32;
            if self.end > self.time {
                self.step = ((e.segments[self.index].1 as i32) << 15).wrapping_sub(self.amplitude)
                    / (self.end - self.time);
            }
        }
        self.amplitude = self.amplitude.wrapping_add(self.step);
        self.time = self.time.wrapping_add(1);
        self.amplitude.wrapping_sub(self.step) >> 15
    }
}
