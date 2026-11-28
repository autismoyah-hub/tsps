//! Original GameSpritePixels.ok area downsampling for world-map region images.
use crate::{raster::Raster, sprite_pixels::Pixels, Error, Result};
pub fn draw(image: &Pixels, raster: &mut Raster, [x, y, width, height]: [i32; 4]) -> Result<()> {
    if width <= 0 || height <= 0 || image.canvas.contains(&0) {
        return Err(Error::Invalid("map downsample dimensions"));
    }
    let [cw, ch] = image.canvas.map(|v| v as i64);
    let [ox, oy] = image.offset.map(|v| v as i64);
    let [sw, sh] = image.size.map(|v| v as i64);
    let (w, h) = (width as i64, height as i64);
    if w > cw
        || h > ch
        || ox + sw > cw
        || oy + sh > ch
        || image.size[0].checked_mul(image.size[1]) != Some(image.pixels.len())
    {
        return Err(Error::Invalid("map downsample source"));
    }
    let clip = raster.clip().map(|v| v as i64);
    let x = x as i64;
    let y = y as i64;
    let left = (x + ox * w / cw).max(clip[0]);
    let right = (x + ((ox + sw) * w + cw - 1) / cw).min(clip[2]);
    let top = (y + oy * h / ch).max(clip[1]);
    let bottom = (y + ((oy + sh) * h + ch - 1) / ch).min(clip[3]);
    for py in top..bottom {
        for px in left..right {
            let dx = (px - x) * 16;
            let dy = (py - y) * 16;
            let mut sx0 = dx * cw / w - ox * 16;
            let mut sx1 = (dx + 16) * cw / w - ox * 16;
            let mut sy0 = dy * ch / h - oy * 16;
            let mut sy1 = (dy + 16) * ch / h - oy * 16;
            let threshold = (sx1 - sx0) * (sy1 - sy0) / 2;
            if threshold == 0 {
                continue;
            }
            sx0 = sx0.max(0);
            sy0 = sy0.max(0);
            sx1 = sx1.min(sw * 16) - 1;
            sy1 = sy1.min(sh * 16) - 1;
            let first_x = 16 - (sx0 & 15);
            let last_x = (sx1 & 15) + 1;
            let first_y = 16 - (sy0 & 15);
            let last_y = (sy1 & 15) + 1;
            let ax = sx0 >> 4;
            let bx = sx1 >> 4;
            let ay = sy0 >> 4;
            let by = sy1 >> 4;
            let (mut total, mut red, mut green, mut blue) = (0i32, 0i32, 0i32, 0i32);
            for sy in ay..=by {
                let mut wy = 16;
                if sy == ay {
                    wy = first_y;
                }
                if sy == by {
                    wy = last_y;
                }
                for sx in ax..=bx {
                    let color = image.pixels[(sy * sw + sx) as usize];
                    if color != 0 {
                        let weight = (wy
                            * if sx == ax {
                                first_x
                            } else if sx == bx {
                                last_x
                            } else {
                                16
                            }) as i32;
                        total = total.wrapping_add(weight);
                        red = red.wrapping_add(((color >> 16 & 255) as i32).wrapping_mul(weight));
                        green =
                            green.wrapping_add(((color >> 8 & 255) as i32).wrapping_mul(weight));
                        blue = blue.wrapping_add(((color & 255) as i32).wrapping_mul(weight));
                    }
                }
            }
            if total as i64 >= threshold {
                let color = ((red / total) << 16)
                    .wrapping_add((green / total) << 8)
                    .wrapping_add(blue / total) as u32;
                let color = if color == 0 { 1 } else { color };
                raster.fill_rect(px as i32, py as i32, 1, 1, color, 256);
            }
        }
    }
    Ok(())
}
