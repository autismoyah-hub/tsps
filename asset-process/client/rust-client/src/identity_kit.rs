//! Body-part kits from archive 2/group 3 (ob).
use crate::{
    buffer::Reader, disk_cache::DiskCache, item_model::Models, model::Model, Error, Result,
};
use std::collections::HashMap;
#[derive(Clone, Debug)]
pub struct Kit {
    pub id: u32,
    pub body_part: i32,
    pub disabled: bool,
    pub models: Vec<u32>,
    pub heads: [i32; 5],
    pub recolor: Vec<(u16, u16)>,
    pub retexture: Vec<(u16, u16)>,
}
impl Kit {
    pub fn decode(id: u32, data: &[u8], local: bool) -> Result<Self> {
        let mut k = Self {
            id,
            body_part: -1,
            disabled: false,
            models: vec![],
            heads: [-1; 5],
            recolor: vec![],
            retexture: vec![],
        };
        let mut r = Reader::new(data);
        loop {
            let op = r.u8()?;
            match op {
                0 => break,
                1 => k.body_part = r.u8()? as i32,
                3 => k.disabled = true,
                2 if local => {
                    k.models.clear();
                    for _ in 0..r.u8()? {
                        k.models.push(r.u16()? as u32);
                    }
                }
                5 => {
                    k.models.clear();
                    for _ in 0..r.u8()? {
                        k.models.push(r.u32()?);
                    }
                }
                40 | 41 => {
                    let pairs = if op == 40 {
                        &mut k.recolor
                    } else {
                        &mut k.retexture
                    };
                    pairs.clear();
                    for _ in 0..r.u8()? {
                        pairs.push((r.u16()?, r.u16()?));
                    }
                }
                60..=64 if local => k.heads[(op - 60) as usize] = r.u16()? as i32,
                70..=74 => k.heads[(op - 70) as usize] = r.u32()? as i32,
                _ => return Err(Error::Io(format!("identity kit {id}: opcode {op}"))),
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("identity kit trailing bytes"));
        }
        Ok(k)
    }
    pub fn load_all(cache: &mut DiskCache) -> Result<HashMap<u32, Self>> {
        cache
            .group(2, 3, [0; 4])?
            .into_iter()
            .map(|(id, bytes)| Ok((id, Self::decode(id, &bytes, true)?)))
            .collect()
    }
    pub fn mesh(
        &self,
        cache: &mut DiskCache,
        models: &mut Models,
        head: bool,
    ) -> Result<Option<Model>> {
        let ids = if head {
            self.heads
                .iter()
                .filter(|&&v| v != -1)
                .map(|&v| v as u32)
                .collect::<Vec<_>>()
        } else {
            self.models.clone()
        };
        if ids.is_empty() && !head {
            return Ok(None);
        }
        let available = cache.reference(7)?;
        if ids.iter().any(|&id| {
            available
                .groups
                .binary_search_by_key(&id, |g| g.id)
                .is_err()
        }) {
            return Err(Error::Invalid("identity kit model unavailable"));
        }
        let mut parts = vec![];
        for id in ids {
            parts.push((*models.raw(cache, id)?).clone());
        }
        let mut model = if !head && parts.len() == 1 {
            parts.remove(0)
        } else {
            Model::combine(&parts)?
        };
        for &(from, to) in &self.recolor {
            model.recolor(from, to);
        }
        for &(from, to) in &self.retexture {
            model.retexture(from, to);
        }
        Ok(Some(model))
    }
    pub fn matches(&self, part: usize, gender: u8) -> bool {
        self.body_part == part as i32 + if gender == 1 { 7 } else { 0 }
    }
}
