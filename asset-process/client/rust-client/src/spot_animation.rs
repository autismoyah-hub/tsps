//! Spot animation definitions, archive 2/group 13 (oj).
use crate::{buffer::Reader, disk_cache::DiskCache, Error, Result};
use std::collections::HashMap;
#[derive(Clone, Debug)]
pub struct Definition {
    pub id: u32,
    pub model: u32,
    pub sequence: i32,
    pub scale: [i32; 2],
    pub rotation: u16,
    pub ambient: i32,
    pub contrast: i32,
    pub flag: bool,
    pub recolor: Vec<(u16, u16)>,
    pub retexture: Vec<(u16, u16)>,
}
impl Definition {
    pub fn decode(id: u32, data: &[u8], local: bool) -> Result<Self> {
        let mut d = Self {
            id,
            model: 0,
            sequence: -1,
            scale: [128; 2],
            rotation: 0,
            ambient: 0,
            contrast: 0,
            flag: true,
            recolor: vec![],
            retexture: vec![],
        };
        let mut r = Reader::new(data);
        loop {
            match r.u8()? {
                0 => break,
                1 if local => d.model = r.u16()? as u32,
                3 => d.model = r.u32()?,
                2 => d.sequence = r.u16()? as i32,
                4 => d.scale[0] = r.u16()? as i32,
                5 => d.scale[1] = r.u16()? as i32,
                6 => d.rotation = r.u16()?,
                7 => d.ambient = r.u8()? as i32,
                8 => d.contrast = r.u8()? as i32,
                10 => d.flag = false,
                op @ (40 | 41) => {
                    let pairs = if op == 40 {
                        &mut d.recolor
                    } else {
                        &mut d.retexture
                    };
                    pairs.clear();
                    for _ in 0..r.u8()? {
                        pairs.push((r.u16()?, r.u16()?));
                    }
                }
                _ => return Err(Error::Invalid("spot animation opcode")),
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("spot animation trailing bytes"));
        }
        Ok(d)
    }
    /// oj.ae returns no base model while GameModelData.ag awaits archive7.
    /// Effect clocks continue independently; retry rendering on a later frame.
    pub fn base_model(
        &self,
        cache: &mut DiskCache,
        models: &mut crate::item_model::Models,
    ) -> Result<Option<crate::model::Model>> {
        let mut model = match models.raw(cache, self.model) {
            Ok(model) => (*model).clone(),
            Err(Error::PendingAsset { .. }) => return Ok(None),
            Err(error) => return Err(error),
        };
        for &(from, to) in &self.recolor { model.recolor(from, to); }
        for &(from, to) in &self.retexture { model.retexture(from, to); }
        Ok(Some(model))
    }
    pub fn load_all(cache: &mut DiskCache) -> Result<HashMap<u32, Self>> {
        cache
            .group(2, 13, [0; 4])?
            .into_iter()
            .map(|(id, bytes)| Ok((id, Self::decode(id, &bytes, true)?)))
            .collect()
    }
}
