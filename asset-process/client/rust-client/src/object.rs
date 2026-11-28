//! Revision 239 object definitions (GameObjectComposition constructor).
use crate::{buffer::Reader, disk_cache::DiskCache, Error, Result};
use std::collections::HashMap;
#[derive(Clone, Debug, PartialEq)]
pub enum Param {
    Int(i32),
    Long(i64),
    String(String),
}
#[derive(Clone, Debug)]
pub struct Operation {
    pub index: u8,
    pub sub: Option<u8>,
    pub condition: Option<(u16, u16, Option<u16>, i32, i32)>,
    pub text: String,
}
#[derive(Clone, Debug, Default)]
pub struct Sound {
    pub ambient: Option<u16>,
    pub range: u8,
    pub retain: u8,
    pub delay: [u16; 2],
    pub effects: Vec<u16>,
    pub easing: Option<u8>,
    pub fade_in: Option<(u8, u32)>,
    pub fade_out: Option<(u8, u32)>,
    pub category: Option<u8>,
}
#[derive(Clone, Debug)]
pub struct Definition {
    pub id: u32,
    pub name: String,
    pub models: Vec<u32>,
    pub model_shapes: Option<Vec<u8>>,
    pub size: [u8; 2],
    pub interact_type: u8,
    pub blocks_projectiles: bool,
    pub interactive: u8,
    pub contour: i32,
    pub non_flat_shading: bool,
    pub model_clipped: bool,
    pub animation: Option<u16>,
    pub decor_displacement: u8,
    pub ambient: i8,
    pub contrast: i32,
    pub operations: Vec<Operation>,
    pub recolor: Vec<(u16, u16)>,
    pub retexture: Vec<(u16, u16)>,
    pub map_icon: Option<u16>,
    pub map_scene: Option<u16>,
    pub rotated: bool,
    pub clipped: bool,
    pub scale: [u16; 3],
    pub offset: [i16; 3],
    pub obstructs_ground: bool,
    pub solid: bool,
    pub support_items: u8,
    pub extra: u8,
    pub transforms: Option<Vec<Option<u16>>>,
    pub transform_varbit: Option<u16>,
    pub transform_varp: Option<u16>,
    pub sound: Sound,
    pub randomize_animation: bool,
    pub flag90: bool,
    pub params: HashMap<u32, Param>,
    pub description: Option<String>,
    pub content_group: Option<u16>,
}
fn optional(r: &mut Reader<'_>) -> Result<Option<u16>> {
    let n = r.u16()?;
    Ok((n != 65535).then_some(n))
}
impl Definition {
    pub fn decode(id: u32, data: &[u8], low_detail: bool, apply_solid: bool) -> Result<Self> {
        Self::decode_mode(id, data, low_detail, apply_solid, false)
    }
    pub fn decode_local(id: u32, data: &[u8]) -> Result<Self> {
        Self::decode_mode(id, data, false, true, true)
    }
    fn decode_mode(
        id: u32,
        data: &[u8],
        low_detail: bool,
        apply_solid: bool,
        local: bool,
    ) -> Result<Self> {
        let mut d = Self {
            id,
            name: "null".into(),
            models: Vec::new(),
            model_shapes: None,
            size: [1, 1],
            interact_type: 2,
            blocks_projectiles: true,
            interactive: 255,
            contour: -1,
            non_flat_shading: false,
            model_clipped: false,
            animation: None,
            decor_displacement: 16,
            ambient: 0,
            contrast: 0,
            operations: Vec::new(),
            recolor: Vec::new(),
            retexture: Vec::new(),
            map_icon: None,
            map_scene: None,
            rotated: false,
            clipped: true,
            scale: [128; 3],
            offset: [0; 3],
            obstructs_ground: false,
            solid: false,
            support_items: 255,
            extra: 0,
            transforms: None,
            transform_varbit: None,
            transform_varp: None,
            sound: Sound::default(),
            randomize_animation: true,
            flag90: false,
            params: HashMap::new(),
            description: None,
            content_group: None,
        };
        let mut r = Reader::new(data);
        loop {
            let op = r.u8()?;
            match op {
                3 if local => d.description = Some(r.string()?),
                200 if local => d.content_group = Some(r.u16()?),
                1 | 5 if local => {
                    let n = r.u8()? as usize;
                    if n > 0 {
                        if !d.models.is_empty() && !low_detail {
                            r.bytes(n * if op == 1 { 3 } else { 2 })?;
                        } else {
                            d.models.clear();
                            d.model_shapes = (op == 1).then(Vec::new);
                            for _ in 0..n {
                                d.models.push(r.u16()? as u32);
                                if let Some(s) = d.model_shapes.as_mut() {
                                    s.push(r.u8()?);
                                }
                            }
                        }
                    }
                }
                0 => break,
                2 => d.name = r.string()?,
                6 | 7 => {
                    let n = r.u8()? as usize;
                    if n > 0 {
                        if !d.models.is_empty() && !low_detail {
                            r.bytes(n * if op == 6 { 5 } else { 4 })?;
                        } else {
                            d.models.clear();
                            d.model_shapes = (op == 6).then(Vec::new);
                            for _ in 0..n {
                                d.models.push(r.u32()?);
                                if let Some(s) = d.model_shapes.as_mut() {
                                    s.push(r.u8()?);
                                }
                            }
                        }
                    }
                }
                14 => d.size[0] = r.u8()?,
                15 => d.size[1] = r.u8()?,
                17 => {
                    d.interact_type = 0;
                    d.blocks_projectiles = false;
                }
                18 => d.blocks_projectiles = false,
                19 => d.interactive = r.u8()?,
                21 => d.contour = 0,
                22 => d.non_flat_shading = true,
                23 => d.model_clipped = true,
                24 => d.animation = optional(&mut r)?,
                27 => d.interact_type = 1,
                28 => d.decor_displacement = r.u8()?,
                29 => d.ambient = r.u8()? as i8,
                39 => d.contrast = r.u8()? as i8 as i32 * 5,
                30..=34 => {
                    let text = r.string()?;
                    if !text.eq_ignore_ascii_case("hidden") {
                        d.operations.push(Operation {
                            index: op - 30,
                            sub: None,
                            condition: None,
                            text,
                        });
                    }
                }
                100 => {
                    let index = r.u8()?;
                    let sub = Some(r.u8()?);
                    let text = r.string()?;
                    d.operations.push(Operation {
                        index,
                        sub,
                        condition: None,
                        text,
                    });
                }
                101 | 102 => {
                    let index = r.u8()?;
                    let a = r.u16()?;
                    let b = r.u16()?;
                    let c = if op == 102 { Some(r.u16()?) } else { None };
                    let min = r.u32()? as i32;
                    let max = r.u32()? as i32;
                    let text = r.string()?;
                    d.operations.push(Operation {
                        index,
                        sub: None,
                        condition: Some((a, b, c, min, max)),
                        text,
                    });
                }
                40 | 41 => {
                    let n = r.u8()?;
                    let values = if op == 40 {
                        &mut d.recolor
                    } else {
                        &mut d.retexture
                    };
                    values.clear();
                    for _ in 0..n {
                        values.push((r.u16()?, r.u16()?));
                    }
                }
                61 => {
                    r.u16()?;
                }
                62 => d.rotated = true,
                64 => d.clipped = false,
                65..=67 => d.scale[(op - 65) as usize] = r.u16()?,
                68 => d.map_scene = Some(r.u16()?),
                69 => {
                    r.u8()?;
                }
                70..=72 => d.offset[(op - 70) as usize] = r.u16()? as i16,
                73 => d.obstructs_ground = true,
                74 => d.solid = true,
                75 => d.support_items = r.u8()?,
                77 | 92 => {
                    d.transform_varbit = optional(&mut r)?;
                    d.transform_varp = optional(&mut r)?;
                    let fallback = if op == 92 { optional(&mut r)? } else { None };
                    let n = r.u8()?;
                    let mut ids = Vec::new();
                    for _ in 0..=n {
                        ids.push(optional(&mut r)?);
                    }
                    ids.push(fallback);
                    d.transforms = Some(ids);
                }
                78 => {
                    d.sound.ambient = Some(r.u16()?);
                    d.sound.range = r.u8()?;
                    d.sound.retain = r.u8()?;
                }
                79 => {
                    d.sound.delay = [r.u16()?, r.u16()?];
                    d.sound.range = r.u8()?;
                    d.sound.retain = r.u8()?;
                    let n = r.u8()?;
                    d.sound.effects.clear();
                    for _ in 0..n {
                        d.sound.effects.push(r.u16()?);
                    }
                }
                81 => d.contour = r.u8()? as i32 * 256,
                82 => d.map_icon = Some(r.u16()?),
                89 => d.randomize_animation = false,
                90 => d.flag90 = true,
                91 => d.sound.easing = Some(r.u8()?),
                93 => {
                    d.sound.fade_in = Some((r.u8()?, r.u16()? as u32 * 20));
                    d.sound.fade_out = Some((r.u8()?, r.u16()? as u32 * 20));
                }
                95 => d.sound.category = Some(r.u8()?),
                96 => d.extra = r.u8()?,
                249 => {
                    for _ in 0..r.u8()? {
                        let kind = r.u8()?;
                        let key = r.u24()?;
                        let val = match kind {
                            1 => Param::String(r.string()?),
                            2 => Param::Long(r.u64()? as i64),
                            _ => Param::Int(r.u32()? as i32),
                        };
                        d.params.entry(key).or_insert(val);
                    }
                }
                // The original switch explicitly treats all other bytes as no-ops.
                _ => {}
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("object definition trailing bytes"));
        }
        if d.interactive == 255 {
            d.interactive = (!d.models.is_empty()
                && (d
                    .model_shapes
                    .as_ref()
                    .map_or(true, |s| s.first() == Some(&10)))
                || !d.operations.is_empty()) as u8;
        }
        if d.support_items == 255 {
            d.support_items = (d.interact_type != 0) as u8;
        }
        if apply_solid && d.solid {
            d.interact_type = 0;
            d.blocks_projectiles = false;
        }
        Ok(d)
    }
    pub fn load_all(cache: &mut DiskCache) -> Result<HashMap<u32, Self>> {
        cache
            .group(2, 6, [0; 4])?
            .into_iter()
            .map(|(id, data)| {
                Ok((
                    id,
                    Self::decode_local(id, &data)
                        .map_err(|e| Error::Io(format!("object {id}: {e}")))?,
                ))
            })
            .collect()
    }
    pub fn dimensions(&self, rotation: u8) -> [u8; 2] {
        if rotation & 1 != 0 {
            [self.size[1], self.size[0]]
        } else {
            self.size
        }
    }
    pub fn transformed_id(&self, varbit: i32, varp: i32) -> Option<u32> {
        let Some(ids) = &self.transforms else {
            return Some(self.id);
        };
        let value = if self.transform_varbit.is_some() {
            varbit
        } else if self.transform_varp.is_some() {
            varp
        } else {
            -1
        };
        let i = if value >= 0 && (value as usize) < ids.len() - 1 {
            value as usize
        } else {
            ids.len() - 1
        };
        ids[i].map(u32::from)
    }
    pub fn model_ids(&self, shape: u8) -> Vec<u32> {
        match &self.model_shapes {
            Some(shapes) => shapes
                .iter()
                .position(|&s| s == shape)
                .map(|i| vec![self.models[i]])
                .unwrap_or_default(),
            None => {
                if shape == 10 {
                    self.models.clone()
                } else {
                    Vec::new()
                }
            }
        }
    }
}
