//! Config group15 ambient definitions and millisecond fade semantics (pn/wd).
use crate::{buffer::Reader, disk_cache::DiskCache, Error, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct Fade {
    pub duration: i32,
    pub mode: u8,
}
impl Default for Fade {
    fn default() -> Self {
        Self {
            duration: 300,
            mode: 0,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct RandomGroup {
    pub min_ms: i32,
    pub max_ms: i32,
    pub sounds: Vec<u16>,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Definition {
    pub id: u32,
    pub loops: Vec<u16>,
    pub random: Vec<RandomGroup>,
    pub fade_in: Fade,
    pub fade_out: Fade,
}
impl Definition {
    pub fn decode(id: u32, bytes: Option<&[u8]>) -> Result<Self> {
        let mut d = Self {
            id,
            ..Self::default()
        };
        let Some(bytes) = bytes else { return Ok(d) };
        let mut r = Reader::new(bytes);
        loop {
            match r.u8()? {
                0 => return Ok(d),
                1 => {
                    d.loops.clear();
                    for _ in 0..r.u8()? {
                        d.loops.push(r.u16()?);
                    }
                }
                2 => {
                    let min_ms = r.u16()? as i32 * 20;
                    let max_ms = r.u16()? as i32 * 20;
                    let count = r.u8()?;
                    let mut sounds = Vec::with_capacity(count as usize);
                    for _ in 0..count {
                        sounds.push(r.u16()?);
                    }
                    // Oversized groups are consumed before being discarded.
                    if count <= 48 && d.random.len() < 8 {
                        d.random.push(RandomGroup {
                            min_ms,
                            max_ms,
                            sounds,
                        });
                    }
                }
                tag @ (3 | 4) => {
                    let mode = r.u8()?;
                    if mode >= 28 {
                        return Err(Error::Invalid("ambient easing"));
                    }
                    let fade = Fade {
                        mode,
                        duration: r.u16()? as i32 * 20,
                    };
                    if tag == 3 {
                        d.fade_in = fade;
                    } else {
                        d.fade_out = fade;
                    }
                }
                _ => {}
            }
        }
    }
    pub fn load(cache: &mut DiskCache, id: u32) -> Result<Self> {
        let exists = cache
            .reference(2)?
            .groups
            .iter()
            .find(|g| g.id == 15)
            .is_some_and(|g| g.files.iter().any(|f| f.id == id));
        if exists {
            Self::decode(id, Some(&cache.file(2, 15, id, [0; 4])?))
        } else {
            Self::decode(id, None)
        }
    }
}

/// The original uses a signed delta and Java float-to-int saturation, including NaN.
pub fn scaled_duration(duration: i32, delta: i32, baseline: i32) -> i32 {
    if delta >= baseline {
        duration
    } else {
        (duration as f32 * (delta as f32 / baseline as f32)) as i32
    }
}

pub fn fade_volume(from: i32, target: i32, current: i32, elapsed: i64, fade: &Fade) -> i32 {
    if current == target {
        return current;
    }
    let ease = crate::camera_tween::ease_millis(elapsed, fade.duration as i64, fade.mode);
    if current > target {
        target.wrapping_add((from.wrapping_sub(target) as f64 * (1.0 - ease)) as i32)
    } else {
        ((target.wrapping_sub(from) as f64 * ease) as i32).wrapping_add(from)
    }
}
