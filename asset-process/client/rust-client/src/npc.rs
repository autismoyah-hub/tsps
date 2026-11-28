//! Revision 239 NPC definitions, including the local world's legacy extensions.
use crate::{
    buffer::Reader,
    disk_cache::DiskCache,
    object::{Operation, Param, Sound},
    Error, Result,
};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Definition {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<u16>,
    pub size: u8,
    pub models: Vec<u32>,
    pub head_models: Vec<u32>,
    /// Idle, left turn, right turn, walk, backwards, left, right, then run and crawl quartets.
    pub animations: [i32; 15],
    pub restart_idle: bool,
    pub recolor: Vec<(u16, u16)>,
    pub retexture: Vec<(u16, u16)>,
    pub operations: Vec<Operation>,
    pub minimap: bool,
    pub combat: i32,
    pub scale: [u16; 2],
    pub ambient: i32,
    pub contrast: i32,
    pub turn_speed: u16,
    pub transforms: Option<Vec<Option<u16>>>,
    pub transform_varbit: Option<u16>,
    pub transform_varp: Option<u16>,
    pub interactable: bool,
    pub rotation: bool,
    pub follower: bool,
    pub low_priority: bool,
    pub overlay_height: i32,
    pub stats: [u16; 6],
    pub head_icons: Vec<(i32, i16)>,
    pub footprint: i32,
    pub hide_overlap: bool,
    pub overlap: u16,
    pub z_buffer: bool,
    pub priority: u8,
    pub sound: Sound,
    pub params: HashMap<u32, Param>,
    /// Server-only extension bytes, retained without assigning client semantics.
    pub extensions: HashMap<u8, Vec<u32>>,
}
fn optional(r: &mut Reader<'_>) -> Result<Option<u16>> {
    let n = r.u16()?;
    Ok((n != 65535).then_some(n))
}
impl Definition {
    pub fn decode(id: u32, data: &[u8]) -> Result<Self> {
        Self::decode_mode(id, data, false)
    }
    pub fn decode_local(id: u32, data: &[u8]) -> Result<Self> {
        Self::decode_mode(id, data, true)
    }
    fn decode_mode(id: u32, data: &[u8], local: bool) -> Result<Self> {
        let mut d = Self {
            id,
            name: "null".into(),
            description: None,
            category: None,
            size: 1,
            models: vec![],
            head_models: vec![],
            animations: [-1; 15],
            restart_idle: false,
            recolor: vec![],
            retexture: vec![],
            operations: vec![],
            minimap: true,
            combat: -1,
            scale: [128; 2],
            ambient: 0,
            contrast: 0,
            turn_speed: 32,
            transforms: None,
            transform_varbit: None,
            transform_varp: None,
            interactable: true,
            rotation: true,
            follower: false,
            low_priority: false,
            overlay_height: -1,
            stats: [1; 6],
            head_icons: vec![],
            footprint: -1,
            hide_overlap: false,
            overlap: 39188,
            z_buffer: true,
            priority: 0,
            sound: Sound::default(),
            params: HashMap::new(),
            extensions: HashMap::new(),
        };
        let mut r = Reader::new(data);
        loop {
            let op = r.u8()?;
            match op {
                0 => break,
                2 => d.name = r.string()?,
                3 if local => d.description = Some(r.string()?),
                12 => d.size = r.u8()?,
                13 => d.animations[0] = r.u16()? as i32,
                14 => d.animations[3] = r.u16()? as i32,
                15 => d.animations[1] = r.u16()? as i32,
                16 => d.animations[2] = r.u16()? as i32,
                17 => {
                    for i in 3..7 {
                        d.animations[i] = r.u16()? as i32;
                    }
                }
                18 => d.category = Some(r.u16()?),
                30..=34 => {
                    let text = r.string()?;
                    if !text.eq_ignore_ascii_case("hidden") {
                        d.operations.retain(|v| v.index != op - 30);
                        d.operations.push(Operation {
                            index: op - 30,
                            sub: None,
                            condition: None,
                            text,
                        });
                    }
                }
                40 | 41 => {
                    let values = if op == 40 {
                        &mut d.recolor
                    } else {
                        &mut d.retexture
                    };
                    values.clear();
                    for _ in 0..r.u8()? {
                        values.push((r.u16()?, r.u16()?));
                    }
                }
                1 | 60 if local => {
                    let values = if op == 1 {
                        &mut d.models
                    } else {
                        &mut d.head_models
                    };
                    values.clear();
                    for _ in 0..r.u8()? {
                        values.push(r.u16()? as u32);
                    }
                }
                61 | 62 => {
                    let values = if op == 61 {
                        &mut d.models
                    } else {
                        &mut d.head_models
                    };
                    values.clear();
                    for _ in 0..r.u8()? {
                        values.push(r.u32()?);
                    }
                }
                74..=79 => d.stats[(op - 74) as usize] = r.u16()?,
                93 => d.minimap = false,
                95 => d.combat = r.u16()? as i32,
                97 | 98 => d.scale[(op - 97) as usize] = r.u16()?,
                99 => d.priority = 1,
                100 => d.ambient = r.u8()? as i8 as i32,
                101 => d.contrast = r.u8()? as i8 as i32 * 5,
                102 => {
                    let mask = r.u8()?;
                    d.head_icons.clear();
                    for i in 0..(8 - mask.leading_zeros()) {
                        if mask & (1 << i) == 0 {
                            d.head_icons.push((-1, -1));
                        } else {
                            let wide = r.clone().u8()? & 128 != 0;
                            let group = r.large_smart()?;
                            let group = if !wide && group == 32767 {
                                -1
                            } else {
                                group as i32
                            };
                            d.head_icons
                                .push((group, (r.unsigned_smart()? as i32 - 1) as i16));
                        }
                    }
                }
                103 => d.turn_speed = r.u16()?,
                106 | 118 => {
                    d.transform_varbit = optional(&mut r)?;
                    d.transform_varp = optional(&mut r)?;
                    let fallback = if op == 118 { optional(&mut r)? } else { None };
                    let n = r.u8()?;
                    let mut ids = Vec::new();
                    for _ in 0..=n {
                        ids.push(optional(&mut r)?);
                    }
                    ids.push(fallback);
                    d.transforms = Some(ids);
                }
                107 => d.interactable = false,
                109 => d.rotation = false,
                111 => d.priority = 2,
                114 => d.animations[7] = r.u16()? as i32,
                115 => {
                    for i in 7..11 {
                        d.animations[i] = r.u16()? as i32;
                    }
                }
                116 => d.animations[11] = r.u16()? as i32,
                117 => {
                    for i in 11..15 {
                        d.animations[i] = r.u16()? as i32;
                    }
                }
                122 => d.follower = true,
                123 => d.low_priority = true,
                124 => d.overlay_height = r.u16()? as i32,
                126 => d.footprint = r.u16()? as i32,
                130 => d.restart_idle = true,
                145 => d.hide_overlap = true,
                146 => d.overlap = r.u16()?,
                147 => d.z_buffer = false,
                148 => {
                    d.sound.ambient = Some(r.u16()?);
                    d.sound.range = r.u8()?;
                    d.sound.retain = r.u8()?;
                }
                149 => d.sound.easing = Some(r.u8()?),
                150 => {
                    d.sound.fade_in = Some((r.u8()?, r.u16()? as u32 * 20));
                    d.sound.fade_out = Some((r.u8()?, r.u16()? as u32 * 20));
                }
                151 => d.sound.category = Some(r.u8()?),
                152 => {
                    d.sound.delay = [r.u16()?, r.u16()?];
                    d.sound.range = r.u8()?;
                    d.sound.retain = r.u8()?;
                    d.sound.effects.clear();
                    for _ in 0..r.u8()? {
                        d.sound.effects.push(r.u16()?);
                    }
                }
                249 => {
                    for _ in 0..r.u8()? {
                        let kind = r.u8()?;
                        let key = r.u24()?;
                        let value = match kind {
                            1 => Param::String(r.string()?),
                            2 => Param::Long(r.u64()? as i64),
                            _ => Param::Int(r.u32()? as i32),
                        };
                        d.params.entry(key).or_insert(value);
                    }
                }
                251 => {
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
                252 | 253 => {
                    let index = r.u8()?;
                    let a = r.u16()?;
                    let b = r.u16()?;
                    let c = if op == 253 { Some(r.u16()?) } else { None };
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
                129 | 210 if local => {
                    d.extensions.insert(op, vec![]);
                }
                200..=202 | 205..=209 | 212 if local => {
                    d.extensions.insert(op, vec![r.u8()? as u32]);
                }
                204 | 211 | 213..=216 if local => {
                    d.extensions.insert(op, vec![r.u16()? as u32]);
                }
                203 if local => {
                    let n = r.u8()?;
                    let mut values = vec![];
                    for _ in 0..=n {
                        values.push(r.u32()?);
                        values.push(r.u8()? as u32);
                    }
                    d.extensions.insert(op, values);
                }
                _ => return Err(Error::Io(format!("NPC {id}: unknown opcode {op}"))),
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("NPC trailing bytes"));
        }
        if d.footprint == -1 {
            d.footprint = (d.size as f32 * 128.0 * 0.4) as i32;
        }
        Ok(d)
    }
    pub fn load_all(cache: &mut DiskCache) -> Result<HashMap<u32, Self>> {
        cache
            .group(2, 9, [0; 4])?
            .into_iter()
            .map(|(id, data)| Ok((id, Self::decode_local(id, &data)?)))
            .collect()
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
}
