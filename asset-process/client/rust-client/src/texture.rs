//! Revision 239 textures: one indexed sprite per definition, with scrolling.
use crate::{buffer::Reader, disk_cache::DiskCache, sprite::SpriteSheet, Error, Result};
use std::collections::HashMap;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition {
    pub sprite: u16,
    pub average_hsl: u16,
    pub opaque: bool,
    pub direction: u8,
    pub speed: u8,
}
impl Definition {
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let result = Self {
            sprite: r.u16()?,
            average_hsl: r.u16()?,
            opaque: r.u8()? == 1,
            direction: r.u8()?,
            speed: r.u8()?,
        };
        if r.remaining() != 0 {
            return Err(Error::Invalid("texture definition length"));
        }
        Ok(result)
    }
    pub fn load_all(cache: &mut DiskCache) -> Result<HashMap<u16, Self>> {
        cache
            .group(9, 0, [0; 4])?
            .into_iter()
            .map(|(id, bytes)| {
                Ok((
                    u16::try_from(id).map_err(|_| Error::Invalid("texture id"))?,
                    Self::decode(&bytes)?,
                ))
            })
            .collect()
    }
}
pub struct Texture {
    pub definition: Definition,
    pub size: usize,
    pub pixels: Vec<u32>,
}
impl Texture {
    pub fn load(
        cache: &mut DiskCache,
        definition: Definition,
        size: usize,
        brightness: f64,
    ) -> Result<Self> {
        let sheet =
            SpriteSheet::decode(&cache.file(8, definition.sprite as u32, 0, [0; 4])?, 16384)?;
        Self::from_sprite(definition, &sheet, size, brightness)
    }
    pub fn from_sprite(
        definition: Definition,
        sheet: &SpriteSheet,
        size: usize,
        brightness: f64,
    ) -> Result<Self> {
        if ![64, 128].contains(&size)
            || ![64, 128].contains(&sheet.width)
            || sheet.height != sheet.width
        {
            return Err(Error::Invalid("texture dimensions"));
        }
        if !brightness.is_finite() || brightness <= 0.0 || brightness > 10.0 {
            return Err(Error::Invalid("texture brightness"));
        }
        let sprite = sheet
            .sprites
            .first()
            .ok_or(Error::Invalid("empty texture sprite"))?;
        if sprite.x + sprite.width > sheet.width
            || sprite.y + sprite.height > sheet.height
            || sprite.indices.len() != sprite.width * sprite.height
        {
            return Err(Error::Invalid("texture sprite bounds"));
        }
        let mut normalized = vec![0; sheet.width * sheet.height];
        for y in 0..sprite.height {
            normalized[(y + sprite.y) * sheet.width + sprite.x
                ..(y + sprite.y) * sheet.width + sprite.x + sprite.width]
                .copy_from_slice(&sprite.indices[y * sprite.width..(y + 1) * sprite.width]);
        }
        let gamma: [u32; 256] =
            std::array::from_fn(|n| (libm::pow(n as f64 / 256.0, brightness) * 256.0) as u32);
        let colors = sheet
            .palette
            .iter()
            .map(|&rgb| {
                (gamma[((rgb >> 16) & 255) as usize] << 16)
                    | (gamma[((rgb >> 8) & 255) as usize] << 8)
                    | gamma[(rgb & 255) as usize]
            })
            .collect::<Vec<_>>();
        let mut pixels = Vec::with_capacity(size * size);
        for y in 0..size {
            for x in 0..size {
                let index = normalized
                    [(y * sheet.width / size) * sheet.width + x * sheet.width / size]
                    as usize;
                pixels.push(
                    *colors
                        .get(index)
                        .ok_or(Error::Invalid("texture palette index"))?,
                );
            }
        }
        Ok(Self {
            definition,
            size,
            pixels,
        })
    }
    pub fn animate(&mut self, ticks: i32) {
        let direction = self.definition.direction;
        if !(1..=4).contains(&direction) {
            return;
        }
        let step = ticks.wrapping_mul(self.definition.speed as i32);
        let mut next = vec![0; self.pixels.len()];
        if direction <= 2 {
            let offset = step.wrapping_mul(self.size as i32);
            let offset = if direction == 1 {
                offset.wrapping_neg()
            } else {
                offset
            };
            for (i, pixel) in next.iter_mut().enumerate() {
                *pixel =
                    self.pixels[(i as i32).wrapping_add(offset) as usize & (self.pixels.len() - 1)];
            }
        } else {
            let offset = if direction == 3 {
                step.wrapping_neg()
            } else {
                step
            };
            for y in 0..self.size {
                for x in 0..self.size {
                    next[y * self.size + x] = self.pixels[y * self.size
                        + ((x as i32).wrapping_add(offset) as usize & (self.size - 1))];
                }
            }
        }
        self.pixels = next;
    }
}
