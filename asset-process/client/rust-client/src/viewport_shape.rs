//! Viewport aspect limits and script-controlled zoom (GameAABB.co / ig.bw).
use crate::{interface_script::ints, interface_state::Interfaces, vm::Stacks, Result};
#[derive(Clone, Debug)]
pub struct Settings {
    pub fov: [i16; 2],
    pub zoom: [i16; 2],
    pub limits: [i16; 4],
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            fov: [256, 205],
            zoom: [256, 320],
            limits: [1, 32767, 1, 32767],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    pub offset: [i32; 2],
    pub size: [i32; 2],
    pub focal: i32,
}
impl Settings {
    pub fn shape(&self, [mut width, mut height]: [i32; 2]) -> Shape {
        width = width.max(1);
        height = height.max(1);
        let delta = height - 334;
        let mut zoom = if delta < 0 {
            self.fov[0] as f64
        } else if delta >= 100 {
            self.fov[1] as f64
        } else {
            ((self.fov[1] as i32 - self.fov[0] as i32) * delta / 100 + self.fov[0] as i32) as f64
        };
        let ratio = height as f64 * zoom * 512.0 / (width.wrapping_mul(334) as f64);
        let mut offset = [0; 2];
        if ratio < (self.limits[2] as f64) {
            let ratio = self.limits[2] as f64;
            zoom = ratio * width as f64 * 334.0 / (height.wrapping_mul(512) as f64);
            if zoom > self.limits[1] as f64 {
                zoom = self.limits[1] as f64;
                let adjusted = height as f64 * zoom * 512.0 / (ratio * 334.0);
                offset[0] = ((width as f64 - adjusted) / 2.0) as i32;
                width -= offset[0] * 2;
            }
        } else if ratio > self.limits[3] as f64 {
            let ratio = self.limits[3] as f64;
            zoom = ratio * width as f64 * 334.0 / (height.wrapping_mul(512) as f64);
            if zoom < self.limits[0] as f64 {
                zoom = self.limits[0] as f64;
                let adjusted = ratio * width as f64 * 334.0 / (zoom * 512.0);
                offset[1] = ((height as f64 - adjusted) / 2.0) as i32;
                height -= offset[1] * 2;
            }
        }
        Shape {
            offset,
            size: [width, height],
            focal: (height as f64 * zoom / 334.0) as i32,
        }
    }
    pub fn execute(
        &mut self,
        op: u16,
        s: &mut Stacks,
        state: &Interfaces,
        canvas: [i32; 2],
    ) -> Result<bool> {
        match op {
            6200 => {
                let values = ints::<2>(s)?;
                for i in 0..2 {
                    self.fov[i] =
                        libm::pow(2.0, (7.0f32 + values[i] as f32 / 256.0) as f64) as i32 as i16;
                    if self.fov[i] <= 0 {
                        self.fov[i] = 256;
                    }
                }
            }
            6201 => {
                self.zoom = ints::<2>(s)?.map(|v| v as i16);
                if self.zoom[0] <= 0 {
                    self.zoom[0] = 256;
                }
                if self.zoom[1] <= 0 {
                    self.zoom[1] = 320;
                }
            }
            6202 => {
                self.limits = ints::<4>(s)?.map(|v| v as i16);
                for start in [0, 2] {
                    if self.limits[start] <= 0 {
                        self.limits[start] = 1;
                    }
                    if self.limits[start + 1] <= 0 {
                        self.limits[start + 1] = 32767;
                    } else {
                        self.limits[start + 1] = self.limits[start + 1].max(self.limits[start]);
                    }
                }
            }
            6203 => {
                let layout = state.layout(canvas)?;
                let size = layout
                    .order
                    .iter()
                    .find(|id| state.widgets[id].content == 1337)
                    .map(|id| self.shape(layout.widgets[id].geometry.size).size)
                    .unwrap_or([-1; 2]);
                s.push_int(size[0])?;
                s.push_int(size[1])?;
            }
            6204 => {
                s.push_int(self.zoom[0] as i32)?;
                s.push_int(self.zoom[1] as i32)?;
            }
            6205 => {
                for v in self.fov {
                    s.push_int(((libm::log(v as f64) / libm::log(2.0) - 7.0) * 256.0) as i32)?;
                }
            }
            6220 | 6221 => s.push_int(0)?,
            6222 => s.push_int(canvas[0])?,
            6223 => s.push_int(canvas[1])?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}
