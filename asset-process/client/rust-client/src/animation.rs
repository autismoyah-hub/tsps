//! Revision 239 sequence, skeleton and frame decoders (GameAnimation/es/ed).
use crate::{buffer::Reader, disk_cache::DiskCache, Error, Result};
use std::collections::HashMap;
#[derive(Clone, Debug)]
pub struct Sound {
    pub id: u16,
    pub location: u8,
    pub loops: u8,
    pub range: u8,
    pub retain: u8,
}
#[derive(Clone, Debug)]
pub struct Sequence {
    pub id: u32,
    pub frames: Vec<u32>,
    pub lengths: Vec<u16>,
    pub chat_frames: Vec<u32>,
    pub cached_animation: Option<u32>,
    pub cached_range: [u16; 2],
    pub total_length: u32,
    pub loop_frames: i32,
    pub interleave: Option<Vec<u32>>,
    pub bone_mask: Option<[bool; 256]>,
    pub stretch: bool,
    pub priority: u8,
    pub shield: i32,
    pub weapon: i32,
    pub max_loops: u8,
    pub precedence: u8,
    pub priority_moving: u8,
    pub restart: u8,
    pub extra: i8,
    pub flag19: bool,
    pub sounds: HashMap<u16, Vec<Sound>>,
}
impl Sequence {
    pub fn decode(id: u32, data: &[u8]) -> Result<Self> {
        let mut s = Self {
            id,
            frames: Vec::new(),
            lengths: Vec::new(),
            chat_frames: Vec::new(),
            cached_animation: None,
            cached_range: [0; 2],
            total_length: 0,
            loop_frames: -1,
            interleave: None,
            bone_mask: None,
            stretch: false,
            priority: 5,
            shield: -1,
            weapon: -1,
            max_loops: 99,
            precedence: 255,
            priority_moving: 255,
            restart: 2,
            extra: 0,
            flag19: false,
            sounds: HashMap::new(),
        };
        let mut explicit_precedence = false;
        let mut explicit_moving_priority = false;
        let mut r = Reader::new(data);
        loop {
            match r.u8()? {
                0 => break,
                1 => {
                    let n = r.u16()? as usize;
                    s.lengths = (0..n).map(|_| r.u16()).collect::<Result<_>>()?;
                    s.frames = (0..n).map(|_| Ok(r.u16()? as u32)).collect::<Result<_>>()?;
                    for f in &mut s.frames {
                        *f |= (r.u16()? as u32) << 16;
                    }
                }
                2 => s.loop_frames = r.u16()? as i32,
                3 => {
                    let n = r.u8()?;
                    let mut values = (0..n)
                        .map(|_| Ok(r.u8()? as u32))
                        .collect::<Result<Vec<_>>>()?;
                    values.push(9999999);
                    s.interleave = Some(values);
                }
                4 => s.stretch = true,
                5 => s.priority = r.u8()?,
                6 => s.shield = r.u16()? as i32,
                7 => s.weapon = r.u16()? as i32,
                8 => s.max_loops = r.u8()?,
                9 => {
                    s.precedence = r.u8()?;
                    explicit_precedence = true;
                }
                10 => {
                    s.priority_moving = r.u8()?;
                    explicit_moving_priority = true;
                }
                11 => s.restart = r.u8()?,
                12 => {
                    let n = r.u8()?;
                    s.chat_frames = (0..n).map(|_| Ok(r.u16()? as u32)).collect::<Result<_>>()?;
                    for f in &mut s.chat_frames {
                        *f |= (r.u16()? as u32) << 16;
                    }
                }
                13 => {
                    let id = r.u32()?;
                    s.cached_animation = ((id as i32) >= 0).then_some(id);
                }
                14 => {
                    for _ in 0..r.u16()? {
                        let frame = r.u16()?;
                        let sound = Sound {
                            id: r.u16()?,
                            location: r.u8()?,
                            loops: r.u8()?,
                            range: r.u8()?,
                            retain: r.u8()?,
                        };
                        if sound.id >= 1 && sound.loops >= 1 {
                            s.sounds.entry(frame).or_default().push(sound);
                        }
                    }
                }
                15 => s.cached_range = [r.u16()?, r.u16()?],
                16 => s.extra = r.u8()? as i8,
                17 => {
                    let mut mask = [false; 256];
                    for _ in 0..r.u8()? {
                        mask[r.u8()? as usize] = true;
                    }
                    s.bone_mask = Some(mask);
                }
                19 => s.flag19 = true,
                _ => {}
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("sequence trailing bytes"));
        }
        s.total_length = s.lengths.iter().map(|&v| v as u32).sum();
        let default = if s.interleave.is_some() || s.bone_mask.is_some() {
            2
        } else {
            0
        };
        if !explicit_precedence {
            s.precedence = default;
        }
        if !explicit_moving_priority {
            s.priority_moving = default;
        }
        Ok(s)
    }
    pub fn load_all(cache: &mut DiskCache) -> Result<HashMap<u32, Self>> {
        cache
            .group(2, 12, [0; 4])?
            .into_iter()
            .map(|(id, data)| {
                Ok((
                    id,
                    Self::decode(id, &data)
                        .map_err(|e| Error::Io(format!("sequence {id}: {e}")))?,
                ))
            })
            .collect()
    }
}
#[derive(Clone, Debug)]
pub struct Bone {
    pub parent: i16,
    pub matrices: Vec<[f32; 16]>,
    pub extra: Vec<[f32; 3]>,
}
#[derive(Clone, Debug)]
pub struct Skeleton {
    pub id: u16,
    pub transform_types: Vec<u8>,
    pub labels: Vec<Vec<u8>>,
    pub bones: Vec<Bone>,
    pub poses: u8,
}
impl Skeleton {
    pub fn decode(id: u16, data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let n = r.u8()? as usize;
        let transform_types = r.bytes(n)?.to_vec();
        let sizes = r.bytes(n)?.to_vec();
        let labels = sizes
            .into_iter()
            .map(|n| Ok(r.bytes(n as usize)?.to_vec()))
            .collect::<Result<_>>()?;
        let mut bones = Vec::new();
        let mut poses = 0;
        if r.remaining() > 0 {
            let count = r.u16()? as usize;
            if count > 0 {
                poses = r.u8()?;
                for _ in 0..count {
                    let parent = r.u16()? as i16;
                    if parent >= count as i16 && parent >= 0 {
                        return Err(Error::Invalid("skeleton bone parent"));
                    }
                    let mut matrices = Vec::new();
                    let mut extra = Vec::new();
                    for _ in 0..poses {
                        let mut matrix = [0.0; 16];
                        for v in &mut matrix {
                            *v = f32::from_bits(r.u32()?);
                        }
                        matrices.push(matrix);
                        extra.push([
                            f32::from_bits(r.u32()?),
                            f32::from_bits(r.u32()?),
                            f32::from_bits(r.u32()?),
                        ]);
                    }
                    bones.push(Bone {
                        parent,
                        matrices,
                        extra,
                    });
                }
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("skeleton trailing bytes"));
        }
        Ok(Self {
            id,
            transform_types,
            labels,
            bones,
            poses,
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transform {
    pub label: usize,
    pub values: [i16; 3],
}
#[derive(Clone, Debug)]
pub struct Frame {
    pub skeleton: u16,
    pub transforms: Vec<Transform>,
    pub has_alpha: bool,
}
impl Frame {
    pub fn decode(data: &[u8], skeleton: &Skeleton) -> Result<Self> {
        let mut flags = Reader::new(data);
        // ed skips the header: fg supplies the group's first skeleton to every frame.
        flags.u16()?;
        let count = flags.u8()? as usize;
        let masks = flags.bytes(count)?;
        let mut values = Reader::new(&data[3 + count..]);
        let mut transforms = Vec::new();
        let mut last = -1;
        let mut has_alpha = false;
        for (i, &mask) in masks.iter().enumerate() {
            if mask == 0 {
                continue;
            }
            let kind = *skeleton.transform_types.get(i)
                .ok_or(Error::Invalid("frame skeleton label count"))?;
            if kind != 0 {
                for j in ((last + 1) as usize..i).rev() {
                    if skeleton.transform_types[j] == 0 {
                        transforms.push(Transform {
                            label: j,
                            values: [0; 3],
                        });
                        break;
                    }
                }
            }
            let mut v = [if kind == 3 { 128 } else { 0 }; 3];
            for axis in 0..3 {
                if mask & (1 << axis) != 0 {
                    v[axis] = values.signed_smart()?;
                }
            }
            transforms.push(Transform {
                label: i,
                values: v,
            });
            last = i as i32;
            has_alpha |= kind == 5;
        }
        if values.remaining() != 0 {
            return Err(Error::Invalid("animation frame trailing bytes"));
        }
        Ok(Self {
            skeleton: skeleton.id,
            transforms,
            has_alpha,
        })
    }
}
