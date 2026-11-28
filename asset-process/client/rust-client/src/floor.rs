//! Floor definitions (pf/of), HSL conversion and the fk rasterizer palette.
use crate::{buffer::Reader, disk_cache::DiskCache, Error, Result};
use std::collections::HashMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hsl {
    pub hue: i32,
    pub saturation: i32,
    pub lightness: i32,
}
fn components(rgb: u32) -> (f64, f64, f64) {
    let r = ((rgb >> 16) & 255) as f64 / 256.0;
    let g = ((rgb >> 8) & 255) as f64 / 256.0;
    let b = (rgb & 255) as f64 / 256.0;
    let min = r.min(g).min(b);
    let max = r.max(g).max(b);
    let light = (max + min) / 2.0;
    let (mut hue, mut saturation) = (0.0, 0.0);
    if min != max {
        saturation = if light < 0.5 {
            (max - min) / (max + min)
        } else {
            (max - min) / (2.0 - max - min)
        };
        hue = if max == r {
            (g - b) / (max - min)
        } else if max == g {
            2.0 + (b - r) / (max - min)
        } else {
            (r - g) / (max - min) + 4.0
        };
    }
    (hue / 6.0, saturation, light)
}
pub fn hsl(rgb: u32) -> Hsl {
    let (h, s, l) = components(rgb);
    Hsl {
        hue: (h * 256.0) as i32,
        saturation: ((s * 256.0) as i32).clamp(0, 255),
        lightness: ((l * 256.0) as i32).clamp(0, 255),
    }
}
pub fn pack_hsl(hue: i32, mut saturation: i32, lightness: i32) -> i32 {
    for threshold in [179, 192, 217, 243] {
        if lightness > threshold {
            saturation /= 2;
        }
    }
    ((hue / 4) << 10) + ((saturation / 32) << 7) + lightness / 2
}
#[derive(Debug)]
pub struct Underlay {
    pub rgb: u32,
    pub hue: i32,
    pub saturation: i32,
    pub lightness: i32,
    pub hue_multiplier: i32,
}
impl Underlay {
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let mut rgb = 0;
        loop {
            match r.u8()? {
                0 => break,
                1 => rgb = r.u24()?,
                _ => return Err(Error::Unsupported("underlay definition opcode")),
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("underlay trailing bytes"));
        }
        let (h, s, l) = components(rgb);
        let hue_multiplier = ((512.0 * (s * if l > 0.5 { 1.0 - l } else { l })) as i32).max(1);
        Ok(Self {
            rgb,
            hue: (h * hue_multiplier as f64) as i32,
            saturation: ((s * 256.0) as i32).clamp(0, 255),
            lightness: ((l * 256.0) as i32).clamp(0, 255),
            hue_multiplier,
        })
    }
}
#[derive(Debug)]
pub struct Overlay {
    pub rgb: u32,
    pub texture: Option<u8>,
    pub hide_underlay: bool,
    pub secondary_rgb: Option<u32>,
    pub primary: Hsl,
    pub secondary: Option<Hsl>,
}
impl Overlay {
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let (mut rgb, mut texture, mut hide_underlay, mut secondary_rgb) = (0, None, true, None);
        loop {
            match r.u8()? {
                0 => break,
                1 => rgb = r.u24()?,
                2 => texture = Some(r.u8()?),
                5 => hide_underlay = false,
                7 => secondary_rgb = Some(r.u24()?),
                3 | 4 | 6 | 8 => {}
                9 => {
                    r.u8()?;
                }
                _ => return Err(Error::Unsupported("overlay definition opcode")),
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("overlay trailing bytes"));
        }
        Ok(Self {
            rgb,
            texture,
            hide_underlay,
            secondary_rgb,
            primary: hsl(rgb),
            secondary: secondary_rgb.map(hsl),
        })
    }
}
pub struct Floors {
    pub underlays: HashMap<u16, Underlay>,
    pub overlays: HashMap<u16, Overlay>,
}
impl Floors {
    pub fn load(cache: &mut DiskCache) -> Result<Self> {
        let underlays = cache
            .group(2, 1, [0; 4])?
            .into_iter()
            .map(|(id, data)| {
                Ok((
                    u16::try_from(id).map_err(|_| Error::Invalid("underlay id"))?,
                    Underlay::decode(&data)?,
                ))
            })
            .collect::<Result<_>>()?;
        let overlays = cache
            .group(2, 4, [0; 4])?
            .into_iter()
            .map(|(id, data)| {
                Ok((
                    u16::try_from(id).map_err(|_| Error::Invalid("overlay id"))?,
                    Overlay::decode(&data)?,
                ))
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            underlays,
            overlays,
        })
    }
}
pub fn palette(brightness: f64) -> Result<Vec<u32>> {
    if !brightness.is_finite() || brightness <= 0.0 || brightness > 10.0 {
        return Err(Error::Invalid("palette brightness"));
    }
    let gamma: [u32; 256] =
        std::array::from_fn(|n| (libm::pow(n as f64 / 256.0, brightness) * 256.0) as u32);
    let mut result = Vec::with_capacity(65536);
    for i in 0..512 {
        let hue = (i >> 3) as f64 / 64.0 + 0.0078125;
        let saturation = (i & 7) as f64 / 8.0 + 0.0625;
        for j in 0..128 {
            let light = j as f64 / 128.0;
            let high = if light < 0.5 {
                light * (1.0 + saturation)
            } else {
                light + saturation - light * saturation
            };
            let low = 2.0 * light - high;
            let channel = |mut h: f64| {
                if h > 1.0 {
                    h -= 1.0;
                }
                if h < 0.0 {
                    h += 1.0;
                }
                let value = if 6.0 * h < 1.0 {
                    low + (high - low) * 6.0 * h
                } else if 2.0 * h < 1.0 {
                    high
                } else if 3.0 * h < 2.0 {
                    low + (high - low) * (0.6666666666666666 - h) * 6.0
                } else {
                    low
                };
                gamma[(value * 256.0) as usize]
            };
            result.push(
                ((channel(hue + 0.3333333333333333) << 16)
                    + (channel(hue) << 8)
                    + channel(hue - 0.3333333333333333))
                .max(1),
            );
        }
    }
    Ok(result)
}
