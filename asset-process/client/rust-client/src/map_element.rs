//! Cache 2:35 map element definitions (GameMapElementConfig).
use crate::{buffer::Reader, disk_cache::DiskCache, Error, Result};
use std::collections::HashMap;
#[derive(Clone, Debug)]
pub struct Definition {
    pub id: u32,
    pub sprites: [Option<u32>; 2],
    pub name: Option<String>,
    pub color: u32,
    pub text_size: u8,
    pub world_map: bool,
    pub minimap: bool,
    pub actions: [Option<String>; 5],
    pub target: Option<String>,
    pub category: i32,
    pub alignment: [Option<u8>; 2],
    pub polygon: Option<Vec<[i16; 2]>>,
    pub polygon_colors: Vec<i32>,
    pub polygon_flags: Vec<i8>,
}
fn sprite(r: &mut Reader<'_>) -> Result<Option<u32>> {
    let head = r.u8()?;
    if head & 128 != 0 {
        Ok(Some(((head as u32 & 127) << 24) | r.u24()?))
    } else {
        let id = ((head as u32) << 8) | r.u8()? as u32;
        Ok((id != 32767).then_some(id))
    }
}
impl Definition {
    pub fn decode(id: u32, bytes: &[u8]) -> Result<Self> {
        let mut d = Self {
            id,
            sprites: [None; 2],
            name: None,
            color: 0,
            text_size: 0,
            world_map: true,
            minimap: false,
            actions: Default::default(),
            target: None,
            category: -1,
            alignment: [Some(1), Some(1)],
            polygon: None,
            polygon_colors: vec![],
            polygon_flags: vec![],
        };
        let mut r = Reader::new(bytes);
        loop {
            match r.u8()? {
                0 => break,
                op @ 1..=2 => d.sprites[op as usize - 1] = sprite(&mut r)?,
                3 => d.name = Some(r.string()?),
                4 => d.color = r.u24()?,
                5 => {
                    r.u24()?;
                }
                6 => d.text_size = r.u8()?,
                7 => {
                    let flags = r.u8()?;
                    if flags & 1 == 0 {
                        d.world_map = false;
                    }
                    if flags & 2 != 0 {
                        d.minimap = true;
                    }
                }
                8 | 28 => {
                    r.u8()?;
                }
                op @ 10..=14 => d.actions[op as usize - 10] = Some(r.string()?),
                15 => {
                    let n = r.u8()? as usize;
                    d.polygon = Some(
                        (0..n)
                            .map(|_| Ok([r.u16()? as i16, r.u16()? as i16]))
                            .collect::<Result<_>>()?,
                    );
                    r.u32()?;
                    let count = r.u8()?;
                    d.polygon_colors = (0..count)
                        .map(|_| Ok(r.u32()? as i32))
                        .collect::<Result<_>>()?;
                    d.polygon_flags = (0..n).map(|_| Ok(r.u8()? as i8)).collect::<Result<_>>()?;
                }
                16 => {}
                17 => d.target = Some(r.string()?),
                18 | 25 => {
                    sprite(&mut r)?;
                }
                19 => d.category = r.u16()? as i32,
                21 | 22 => {
                    r.u32()?;
                }
                23 => {
                    r.bytes(3)?;
                }
                24 => {
                    r.bytes(4)?;
                }
                op @ 29..=30 => {
                    let value = r.u8()?;
                    d.alignment[op as usize - 29] = (value <= 2).then_some(value);
                }
                _ => {}
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("map element trailing bytes"));
        }
        Ok(d)
    }
    pub fn load_all(cache: &mut DiskCache) -> Result<HashMap<u32, Self>> {
        cache
            .group(2, 35, [0; 4])?
            .into_iter()
            .map(|(id, bytes)| Ok((id, Self::decode(id, &bytes)?)))
            .collect()
    }
}
