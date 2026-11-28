//! Original zg player-density heatmap, used by the world map.
use crate::{Error, Result};
#[derive(Default)]
pub struct Heatmap {
    pub points: Vec<[i32; 2]>,
    kernels: std::collections::BTreeMap<i32, Vec<i32>>,
}
impl Heatmap {
    pub fn clear(&mut self) {
        self.points.clear();
    }
    pub fn add(&mut self, point: [i32; 2]) {
        if self.points.len() < 2048 {
            self.points.push(point);
        }
    }
    pub fn render(&mut self, origin: [i32; 2], size: [usize; 2], scale: f32) -> Result<Vec<u32>> {
        if !scale.is_finite() || !(0.0..=8.0).contains(&scale) || size.iter().any(|&n| n > 4096) {
            return Err(Error::Invalid("heatmap dimensions/scale"));
        }
        let radius = (scale * 18.0) as i32;
        let side = (radius * 2 + 1) as usize;
        let kernel = self.kernels.entry(radius).or_insert_with(|| {
            let sigma = (radius as f32 / 3.0) as f64;
            let row = (-radius..=radius)
                .map(|n| {
                    let x = n as f64 / sigma;
                    libm::exp(x * -x / 2.0) / libm::sqrt(std::f64::consts::PI * 2.0) / sigma
                })
                .collect::<Vec<_>>();
            let center = row[radius as usize] * row[radius as usize];
            (0..side * side)
                .map(|n| (256.0 * (row[n / side] * row[n % side] / center)) as i32)
                .collect()
        });
        let [width, height] = size;
        let mut values = vec![0i32; width * height];
        for &[px, py] in &self.points {
            let x = (scale * px.wrapping_sub(origin[0]) as f32) as i32 - radius;
            let y = (height as f32 - py.wrapping_sub(origin[1]) as f32 * scale) as i32 - radius;
            let left = x.max(0);
            let top = y.max(0);
            let w = (x + side as i32).min(width as i32) - left;
            let h = (y + side as i32).min(height as i32) - top;
            if w <= 0 || h <= 0 {
                continue;
            }
            // zg.ap keys the source offset off a zero clipped origin, including
            // kernels clipped on both sides of a target smaller than the kernel.
            let sx = if left == 0 { side - w as usize } else { 0 };
            let sy = if top == 0 { side - h as usize } else { 0 };
            for row in 0..h as usize {
                for col in 0..w as usize {
                    values[(top as usize + row) * width + left as usize + col] +=
                        kernel[(sy + row) * side + sx + col];
                }
            }
        }
        let palette = palette();
        Ok(values
            .into_iter()
            .map(|n| {
                let level = (n + 63) / 256;
                0xff000000
                    | if level <= 0 {
                        0
                    } else {
                        palette[(level as usize).min(2000) - 1]
                    }
            })
            .collect())
    }
}
fn palette() -> [u32; 2000] {
    let mut out = [0; 2000];
    for i in 0..16 {
        out[i] = hsl(
            ((240 - i as i32 * 12) as f32 / 360.0) as f64,
            0.9999f32 as f64,
            (0.075f32 + 0.425f32 * i as f32 / 16.0) as f64,
        );
    }
    let mut i = 16;
    let mut hue = 48;
    while i < 2000 {
        let end = (i * 2).min(2000);
        let rgb = hsl((hue as f32 / 360.0) as f64, 0.9999f32 as f64, 0.5);
        out[i..end].fill(rgb);
        i = end;
        hue -= 8;
    }
    out
}
fn hsl(h: f64, s: f64, l: f64) -> u32 {
    let high = if l < 0.5 {
        l * (s + 1.0)
    } else {
        l + s - l * s
    };
    let low = l * 2.0 - high;
    let mut rh = h + 1.0 / 3.0;
    if rh > 1.0 {
        rh -= 1.0;
    }
    let mut bh = h - 1.0 / 3.0;
    if bh < 0.0 {
        bh += 1.0;
    }
    let r = if rh * 6.0 < 1.0 {
        low + (high - low) * 6.0 * rh
    } else if rh * 2.0 < 1.0 {
        high
    } else if rh * 3.0 < 2.0 {
        6.0 * ((high - low) * (2.0 / 3.0 - rh)) + low
    } else {
        low
    };
    let g = if h * 6.0 < 1.0 {
        low + h * (6.0 * (high - low))
    } else if h * 2.0 < 1.0 {
        high
    } else if h * 3.0 < 2.0 {
        (2.0 / 3.0 - h) * (high - low) * 6.0 + low
    } else {
        low
    };
    let b = if bh * 6.0 < 1.0 {
        low + bh * (6.0 * (high - low))
    } else if bh * 2.0 < 1.0 {
        high
    } else if bh * 3.0 < 2.0 {
        (high - low) * (2.0 / 3.0 - bh) * 6.0 + low
    } else {
        low
    };
    ((r * 256.0) as u32) << 16 | ((g * 256.0) as u32) << 8 | (b * 256.0) as u32
}

#[derive(Default)]
pub struct State {
    pub map: Option<Heatmap>,
    pub cycle: i32,
}
impl State {
    pub fn enabled(&mut self, value: bool) {
        if value {
            self.map.get_or_insert_with(Heatmap::default);
        } else {
            self.map = None;
        }
    }
    pub fn refresh(
        &mut self,
        base: [i32; 2],
        positions: impl IntoIterator<Item = [i32; 2]>,
        cycle: i32,
    ) {
        if let Some(map) = &mut self.map {
            self.cycle = cycle;
            map.clear();
            for [x, y] in positions {
                map.add([base[0].wrapping_add(x >> 7), base[1].wrapping_add(y >> 7)]);
            }
        }
    }
}
