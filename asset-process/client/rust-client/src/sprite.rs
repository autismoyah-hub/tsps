//! Indexed cache sprites, recovered from ht.ak([BI)V in gamepack 239.1.
//! The client skips the optional alpha plane; palette index zero is transparent.
use crate::{buffer::Reader, Error, Result};

#[derive(Debug, PartialEq, Eq)]
pub struct Sprite {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub indices: Vec<u8>,
}
#[derive(Debug)]
pub struct SpriteSheet {
    pub width: usize,
    pub height: usize,
    pub palette: Vec<u32>,
    pub sprites: Vec<Sprite>,
}
impl SpriteSheet {
    pub fn decode(data: &[u8], max_pixels: usize) -> Result<Self> {
        let count_pos = data.len().checked_sub(2).ok_or(Error::Truncated)?;
        let count = Reader::new(&data[count_pos..]).u16()? as usize;
        let metadata = data
            .len()
            .checked_sub(7 + count * 8)
            .ok_or(Error::Truncated)?;
        let mut r = Reader::new(&data[metadata..count_pos]);
        let width = r.u16()? as usize;
        let height = r.u16()? as usize;
        if width
            .checked_mul(height)
            .ok_or(Error::Invalid("sprite dimensions"))?
            > max_pixels
        {
            return Err(Error::Invalid("sprite canvas limit"));
        }
        let palette_len = r.u8()? as usize + 1;
        let mut sprites: Vec<Sprite> = (0..count)
            .map(|_| Sprite {
                x: 0,
                y: 0,
                width: 0,
                height: 0,
                indices: Vec::new(),
            })
            .collect();
        for sprite in &mut sprites {
            sprite.x = r.u16()? as usize;
        }
        for sprite in &mut sprites {
            sprite.y = r.u16()? as usize;
        }
        for sprite in &mut sprites {
            sprite.width = r.u16()? as usize;
        }
        for sprite in &mut sprites {
            sprite.height = r.u16()? as usize;
        }
        let palette_pos = metadata
            .checked_sub((palette_len - 1) * 3)
            .ok_or(Error::Truncated)?;
        let mut p = Reader::new(&data[palette_pos..metadata]);
        let mut palette = vec![0; palette_len];
        for color in &mut palette[1..] {
            *color = p.u24()?.max(1);
        }
        let mut pixels = Reader::new(&data[..palette_pos]);
        let mut total = 0usize;
        for s in &mut sprites {
            if s.x + s.width > width || s.y + s.height > height {
                return Err(Error::Invalid("sprite outside canvas"));
            }
            let size = s
                .width
                .checked_mul(s.height)
                .ok_or(Error::Invalid("sprite dimensions"))?;
            total = total
                .checked_add(size)
                .ok_or(Error::Invalid("sprite pixel count"))?;
            if total > max_pixels {
                return Err(Error::Invalid("sprite pixel limit"));
            }
            let flags = pixels.u8()?;
            if flags & !3 != 0 {
                return Err(Error::Unsupported("sprite flags"));
            }
            let raw = pixels.bytes(size)?;
            if raw.iter().any(|&i| i as usize >= palette_len) {
                return Err(Error::Invalid("sprite palette index"));
            }
            s.indices = vec![0; size];
            if flags & 1 == 0 {
                s.indices.copy_from_slice(raw);
            } else {
                for x in 0..s.width {
                    for y in 0..s.height {
                        s.indices[x + y * s.width] = raw[y + x * s.height];
                    }
                }
            }
            if flags & 2 != 0 {
                pixels.bytes(size)?;
            }
        }
        if pixels.remaining() != 0 {
            return Err(Error::Invalid("sprite pixel trailing bytes"));
        }
        Ok(Self {
            width,
            height,
            palette,
            sprites,
        })
    }
    /// Unscaled ARGB pixels. Transparent palette entries remain zero.
    pub fn argb(&self, sprite: usize) -> Result<Vec<u32>> {
        let s = self
            .sprites
            .get(sprite)
            .ok_or(Error::Invalid("sprite index"))?;
        s.indices
            .iter()
            .map(|&i| {
                if i == 0 {
                    Ok(0)
                } else {
                    self.palette
                        .get(i as usize)
                        .map(|&color| 0xff000000 | color)
                        .ok_or(Error::Invalid("sprite palette index"))
                }
            })
            .collect()
    }
}
