//! Timed actor color overrides and render-sampled transparency (fm / za).
use crate::actor_update::Block;
use std::cell::Cell;
#[derive(Clone, Debug)]
pub struct Transparency {
    pub start: i32,
    pub end: i32,
    pub from: u8,
    pub to: u8,
    current: Cell<u8>,
}
impl Default for Transparency {
    fn default() -> Self {
        Self {
            start: -1,
            end: -1,
            from: 0,
            to: 0,
            current: Cell::new(0),
        }
    }
}
impl Transparency {
    pub fn set(&mut self, now: i32, start: i32, end: i32, from: u8, to: u8, use_start: bool) {
        self.start = start;
        self.end = end;
        self.to = to;
        self.from = if !use_start && start >= now {
            self.current.get()
        } else {
            from
        };
    }
    /// A fresh actor inherits the schedule, but not another actor's sampled value.
    pub fn view_copy(&self) -> Self {
        Self {
            current: Cell::new(0),
            ..self.clone()
        }
    }
    pub fn active(&self, cycle: i32) -> bool {
        if cycle < self.start {
            self.current.get() != 0
        } else if cycle >= self.end {
            self.to != 0
        } else {
            self.from != 0 || self.to != 0
        }
    }
    pub fn sample(&self, cycle: i32) -> u8 {
        if cycle < self.start {
            return self.current.get();
        }
        let value = if cycle >= self.end {
            self.to
        } else {
            let t =
                cycle.wrapping_sub(self.start) as f32 / self.end.wrapping_sub(self.start) as f32;
            ((self.to as i32 - self.from as i32) as f32 * t + self.from as f32) as i32 as u8
        };
        self.current.set(value);
        value
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Tint {
    pub start: i32,
    pub end: i32,
    pub hsl: [u8; 3],
    pub weight: u8,
}
impl Default for Tint {
    fn default() -> Self {
        Self {
            start: -1,
            end: -1,
            hsl: [255; 3],
            weight: 0,
        }
    }
}
impl Tint {
    pub fn active(&self, cycle: i32) -> bool {
        self.weight as i8 > 0 && cycle >= self.start && cycle < self.end
    }
    pub fn color(&self, color: i32) -> i32 {
        let mut parts = [color >> 10 & 63, color >> 7 & 7, color & 127];
        for (part, target) in parts.iter_mut().zip(self.hsl) {
            if target != 255 {
                *part += ((target as i8 as i32 - *part) * self.weight as i32) >> 7;
            }
        }
        (parts[0] << 10 | parts[1] << 7 | parts[2]) & 65535
    }
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub tint: Tint,
    pub transparency: Option<Transparency>,
}
impl State {
    pub fn view_copy(&self) -> Self {
        Self {
            tint: self.tint,
            transparency: self.transparency.as_ref().map(Transparency::view_copy),
        }
    }
    pub fn apply(&mut self, block: &Block, cycle: i32) {
        match block {
            Block::Tint {
                delays,
                hsl,
                weight,
            } => {
                self.tint = Tint {
                    start: cycle.wrapping_add(delays[0] as i32),
                    end: cycle.wrapping_add(delays[1] as i32),
                    hsl: *hsl,
                    weight: *weight,
                }
            }
            Block::Transparency {
                delays,
                alpha,
                use_start,
            } => self.transparency.get_or_insert_with(Default::default).set(
                cycle,
                cycle.wrapping_add(delays[0] as i32),
                cycle.wrapping_add(delays[1] as i32),
                alpha[0] as u8,
                alpha[1] as u8,
                *use_start,
            ),
            _ => {}
        }
    }
    pub fn alpha(&self, cycle: i32) -> u8 {
        self.transparency.as_ref().map_or(0, |t| t.sample(cycle))
    }
}
/// Model.gj; override 255 suppresses model pixels, while picking remains active.
pub fn face_alpha(base: u8, override_alpha: u8) -> u8 {
    let base = crate::model_alpha::normalize(base);
    if override_alpha == 255 {
        253
    } else if override_alpha > 0 && base < 253 {
        base + (((253 - base) as u32 * override_alpha as u32) >> 8) as u8
    } else {
        base
    }
}
