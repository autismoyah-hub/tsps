//! Pinned ay.aq's independent PCM pump policy (512 sample frames per block).
use crate::{Error, Result};
pub const BLOCK: i32 = 512;
/// Device methods use sample frames, including when output is stereo.
pub trait Device {
    fn queued(&mut self) -> Result<i32>;
    fn open(&mut self, capacity: i32) -> Result<()>;
    fn close(&mut self);
    /// Mix and submit one BLOCK. Mixing precedes submission failure.
    fn write_block(&mut self) -> Result<()>;
    fn skip(&mut self, frames: i32);
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pump {
    pub capacity: i32,
    pub target: i32,
    pub adjustment: i32,
    pub previous_queued: i32,
    pub drained: i32,
    pub previous_drained: i32,
    pub retry_at: i64,
    pub check_at: i64,
    pub mixed_at: i64,
    pub fresh: bool,
}
impl Pump {
    /// Sample rate must be positive; native output uses 22050 Hz.
    pub fn step(&mut self, now: i64, rate: i32, device: &mut impl Device) -> Option<Error> {
        if self.retry_at != 0 && now < self.retry_at { return None; }
        let result = self.fill(now, device);
        let error = match result {
            Ok(false) => return None,
            Ok(true) => None,
            Err(error) => {
                device.close();
                self.retry_at = now.wrapping_add(2000);
                Some(error)
            }
        };
        let time = if now > self.mixed_at.wrapping_add(500000) { self.mixed_at } else { now };
        // Original uses integer milliseconds for each skipped 512-frame block.
        let interval = if rate > 0 { 512000 / rate } else { 0 };
        if interval > 0 {
            while time > self.mixed_at.wrapping_add(5000) {
                device.skip(BLOCK);
                self.mixed_at = self.mixed_at.wrapping_add(interval as i64);
            }
        } else {
            self.mixed_at = time;
        }
        error
    }
    fn fill(&mut self, now: i64, device: &mut impl Device) -> Result<bool> {
        if self.retry_at != 0 {
            device.open(self.capacity)?;
            self.retry_at = 0;
            self.fresh = true;
        }
        let mut queued = device.queued()?;
        self.drained = self.drained.max(self.previous_queued.wrapping_sub(queued));
        let mut target = self.target.wrapping_add(self.adjustment);
        if target.wrapping_add(BLOCK) > 32768 { target = 32768 - BLOCK; }
        if target.wrapping_add(BLOCK) > self.capacity {
            self.capacity = self.capacity.wrapping_add(1024).min(32768);
            device.close();
            device.open(self.capacity)?;
            queued = 0;
            self.fresh = true;
            if target.wrapping_add(BLOCK) > self.capacity {
                target = self.capacity.wrapping_sub(BLOCK);
                self.adjustment = target.wrapping_sub(self.target);
            }
        }
        while queued < target {
            self.mixed_at = now;
            device.write_block()?;
            queued = queued.wrapping_add(BLOCK);
        }
        if now > self.check_at {
            if !self.fresh {
                if self.drained == 0 && self.previous_drained == 0 {
                    device.close();
                    self.retry_at = now.wrapping_add(2000);
                    return Ok(false);
                }
                self.adjustment = self.previous_drained.min(self.drained);
                self.previous_drained = self.drained;
            } else { self.fresh = false; }
            self.drained = 0;
            self.check_at = now.wrapping_add(2000);
        }
        self.previous_queued = queued;
        Ok(true)
    }
}
