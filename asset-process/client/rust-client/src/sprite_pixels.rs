//! True-color sprite transforms from GameSpritePixels (am/au/ai/ad/ar/di).
use crate::{disk_cache::DiskCache, raster::Raster, sprite::SpriteSheet, Error, Result};
#[derive(Clone, Debug)]
pub struct Pixels {
    pub canvas: [usize; 2],
    pub offset: [usize; 2],
    pub size: [usize; 2],
    pub pixels: Vec<u32>,
}
impl Pixels {
    pub fn load(cache: &mut DiskCache, id: u32) -> Result<Self> {
        Self::from_sheet(
            &SpriteSheet::decode(&cache.file(8, id, 0, [0; 4])?, 16 << 20)?,
            0,
        )
    }
    pub fn from_sheet(sheet: &SpriteSheet, index: usize) -> Result<Self> {
        let s = sheet
            .sprites
            .get(index)
            .ok_or(Error::Invalid("sprite index"))?;
        Ok(Self {
            canvas: [sheet.width, sheet.height],
            offset: [s.x, s.y],
            size: [s.width, s.height],
            pixels: s
                .indices
                .iter()
                .map(|&i| sheet.palette[i as usize])
                .collect(),
        })
    }
    pub fn flip(&mut self, horizontal: bool, vertical: bool) {
        let [w, h] = self.size;
        if horizontal {
            for row in self.pixels.chunks_mut(w.max(1)) {
                row.reverse();
            }
            self.offset[0] = self.canvas[0] - w - self.offset[0];
        }
        if vertical {
            for y in 0..h / 2 {
                for x in 0..w {
                    self.pixels.swap(y * w + x, (h - y - 1) * w + x);
                }
            }
            self.offset[1] = self.canvas[1] - h - self.offset[1];
        }
    }
    pub fn pad(&mut self, amount: usize) {
        let [w, h] = self.size;
        let left = amount.min(self.offset[0]);
        let top = amount.min(self.offset[1]);
        let right = amount.min(self.canvas[0] - w - self.offset[0]);
        let bottom = amount.min(self.canvas[1] - h - self.offset[1]);
        let nw = w + left + right;
        let nh = h + top + bottom;
        let mut pixels = vec![0; nw * nh];
        for y in 0..h {
            pixels[(y + top) * nw + left..(y + top) * nw + left + w]
                .copy_from_slice(&self.pixels[y * w..(y + 1) * w]);
        }
        self.pixels = pixels;
        self.size = [nw, nh];
        self.offset[0] -= left;
        self.offset[1] -= top;
    }
    pub fn outline(&mut self, color: u32) {
        let [w, h] = self.size;
        let mut out = self.pixels.clone();
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if self.pixels[i] == 0
                    && ((x > 0 && self.pixels[i - 1] != 0)
                        || (y > 0 && self.pixels[i - w] != 0)
                        || (x + 1 < w && self.pixels[i + 1] != 0)
                        || (y + 1 < h && self.pixels[i + w] != 0))
                {
                    out[i] = color;
                }
            }
        }
        self.pixels = out;
    }
    pub fn shadow(&mut self, color: u32) {
        let [w, h] = self.size;
        for y in (1..h).rev() {
            for x in (1..w).rev() {
                let i = y * w + x;
                if self.pixels[i] == 0 && self.pixels[i - w - 1] != 0 {
                    self.pixels[i] = color;
                }
            }
        }
    }
    pub fn widget_effects(&mut self, horizontal: bool, vertical: bool, outline: u8, shadow: u32) {
        self.flip(horizontal, vertical);
        if outline > 0 {
            self.pad(outline as usize);
            self.outline(1);
        }
        if outline >= 2 {
            self.outline(0xffffff);
        }
        if shadow != 0 {
            self.shadow(shadow);
        }
    }
    pub fn draw(&self, raster: &mut Raster, x: i32, y: i32, alpha: u16) -> Result<()> {
        self.draw_scaled(
            raster,
            [x, y, self.canvas[0] as i32, self.canvas[1] as i32],
            alpha,
        )
    }
    /// ga.ae sprite submission. The caller supplies the parent clip.
    pub fn draw_widget(
        &self,
        raster: &mut Raster,
        rect: [i32; 4],
        angle: u16,
        tiled: bool,
        if3: bool,
        alpha: u16,
    ) -> Result<()> {
        let [x, y, width, height] = rect;
        if !if3 {
            return self.draw(raster, x, y, 256);
        }
        let saved = raster.clip().map(|v| v as i32);
        if x >= saved[2]
            || y >= saved[3]
            || x.wrapping_add(width) <= saved[0]
            || y.wrapping_add(height) <= saved[1]
        {
            return Ok(());
        }
        let [sw, sh] = self.canvas;
        if sw == 0 || sh == 0 {
            return Ok(());
        }
        if !tiled {
            if angle != 0 {
                self.draw_rotated(
                    raster,
                    [x + width / 2, y + height / 2],
                    angle as i32,
                    width.wrapping_mul(4096) / sw as i32,
                );
                return Ok(());
            }
            return self.draw_scaled(raster, rect, alpha);
        }
        raster.set_clip(
            x.max(saved[0]),
            y.max(saved[1]),
            x.wrapping_add(width).min(saved[2]),
            y.wrapping_add(height).min(saved[3]),
        );
        let result = (|| {
            for column in 0..(width + sw as i32 - 1) / sw as i32 {
                for row in 0..(height + sh as i32 - 1) / sh as i32 {
                    let left = x + column * sw as i32;
                    let top = y + row * sh as i32;
                    if angle != 0 {
                        self.draw_rotated(
                            raster,
                            [left + sw as i32 / 2, top + sh as i32 / 2],
                            angle as i32,
                            4096,
                        );
                    } else {
                        self.draw(raster, left, top, alpha)?;
                    }
                }
            }
            Ok(())
        })();
        raster.set_clip(saved[0], saved[1], saved[2], saved[3]);
        result
    }
    /// GameSpritePixels.bs/dq: canvas-centred rotation, 16-bit angle and 12-bit scale.
    pub fn draw_rotated(&self, raster: &mut Raster, center: [i32; 2], angle: i32, scale: i32) {
        if scale == 0 || self.size.contains(&0) {
            return;
        }
        let pivot: [i32; 2] = std::array::from_fn(|axis| {
            (self.canvas[axis] as i32)
                .wrapping_shl(3)
                .wrapping_sub((self.offset[axis] as i32).wrapping_shl(4))
        });
        let center = center.map(|v| v.wrapping_shl(4));
        let radians = (angle & 65535) as f64 * 9.587379924285257E-5;
        let sin = radians.sin();
        let cos = radians.cos();
        let forward_sin = (sin * scale as f64 + 0.5).floor() as i32;
        let forward_cos = (cos * scale as f64 + 0.5).floor() as i32;
        let mut low = [i32::MAX; 2];
        let mut high = [i32::MIN; 2];
        for x in [0, (self.size[0] as i32).wrapping_shl(4)] {
            for y in [0, (self.size[1] as i32).wrapping_shl(4)] {
                let x = x.wrapping_sub(pivot[0]);
                let y = y.wrapping_sub(pivot[1]);
                let point = [
                    x.wrapping_mul(forward_cos)
                        .wrapping_add(y.wrapping_mul(forward_sin)),
                    x.wrapping_neg()
                        .wrapping_mul(forward_sin)
                        .wrapping_add(y.wrapping_mul(forward_cos)),
                ];
                for axis in 0..2 {
                    low[axis] = low[axis].min(point[axis]);
                    high[axis] = high[axis].max(point[axis]);
                }
            }
        }
        let clip = raster.clip();
        for axis in 0..2 {
            low[axis] =
                (((low[axis] >> 12).wrapping_add(center[axis])) >> 4).max(clip[axis] as i32);
            high[axis] = (((high[axis].wrapping_add(4095) >> 12)
                .wrapping_add(center[axis])
                .wrapping_add(15))
                >> 4)
                .min(clip[axis + 2] as i32);
        }
        if low[0] >= high[0] || low[1] >= high[1] {
            return;
        }
        let reciprocal = 16777216.0 / scale as f64;
        let step_sin = (sin * reciprocal + 0.5).floor() as i32;
        let step_cos = (cos * reciprocal + 0.5).floor() as i32;
        let dx = low[0]
            .wrapping_shl(4)
            .wrapping_add(8)
            .wrapping_sub(center[0]);
        let dy = low[1]
            .wrapping_shl(4)
            .wrapping_add(8)
            .wrapping_sub(center[1]);
        let mut row_x = pivot[0]
            .wrapping_shl(8)
            .wrapping_sub(dy.wrapping_mul(step_sin) >> 4);
        let mut row_y = pivot[1]
            .wrapping_shl(8)
            .wrapping_add(dy.wrapping_mul(step_cos) >> 4);
        for y in low[1]..high[1] {
            let mut sx = row_x.wrapping_add(dx.wrapping_mul(step_cos) >> 4);
            let mut sy = row_y.wrapping_add(dx.wrapping_mul(step_sin) >> 4);
            for x in low[0]..high[0] {
                if sx >= 0
                    && sy >= 0
                    && (sx >> 12) < self.size[0] as i32
                    && (sy >> 12) < self.size[1] as i32
                {
                    let pixel =
                        self.pixels[(sy >> 12) as usize * self.size[0] + (sx >> 12) as usize];
                    if pixel != 0 {
                        raster.blend(x as usize, y as usize, pixel, 256);
                    }
                }
                sx = sx.wrapping_add(step_cos);
                sy = sy.wrapping_add(step_sin);
            }
            row_x = row_x.wrapping_sub(step_sin);
            row_y = row_y.wrapping_add(step_cos);
        }
    }
    pub fn draw_scaled(
        &self,
        raster: &mut Raster,
        [x, y, width, height]: [i32; 4],
        alpha: u16,
    ) -> Result<()> {
        if width <= 0 || height <= 0 {
            return Ok(());
        }
        if self.canvas.contains(&0) {
            return Ok(());
        }
        let steps = [
            ((self.canvas[0] as i64) << 16) / width as i64,
            ((self.canvas[1] as i64) << 16) / height as i64,
        ];
        if steps.contains(&0) {
            return Err(Error::Invalid("sprite scale precision"));
        }
        let clip = raster.clip();
        let origins = [x as i64, y as i64];
        let dimensions = [width as i64, height as i64];
        let starts: [i64; 2] = std::array::from_fn(|axis| {
            origins[axis] + (((self.offset[axis] as i64) << 16) + steps[axis] - 1) / steps[axis]
        });
        let ends: [i64; 2] = std::array::from_fn(|axis| {
            if self.size[axis] < self.canvas[axis] {
                origins[axis]
                    + ((((self.offset[axis] + self.size[axis]) as i64) << 16) + steps[axis] - 1)
                        / steps[axis]
            } else {
                origins[axis] + dimensions[axis]
            }
        });
        let left = starts[0].max(clip[0] as i64);
        let top = starts[1].max(clip[1] as i64);
        let right = ends[0].min(clip[2] as i64);
        let bottom = ends[1].min(clip[3] as i64);
        for dy in top..bottom {
            let sy = ((dy - y as i64) * steps[1] >> 16) - self.offset[1] as i64;
            if sy < 0 || sy >= self.size[1] as i64 {
                continue;
            }
            for dx in left..right {
                let sx = ((dx - x as i64) * steps[0] >> 16) - self.offset[0] as i64;
                if sx >= 0 && sx < self.size[0] as i64 {
                    let pixel = self.pixels[sy as usize * self.size[0] + sx as usize];
                    if pixel != 0 {
                        raster.blend(dx as usize, dy as usize, pixel, alpha);
                    }
                }
            }
        }
        Ok(())
    }
}
