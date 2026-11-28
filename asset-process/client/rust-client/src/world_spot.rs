//! Location graphics effects from enclosed zone record 5.
use crate::{buffer::Reader, Error, Result};
#[derive(Clone, Debug)]
pub struct Request {
    pub id: u16,
    pub height: u8,
    pub delay: u16,
    pub position: [i32; 3],
}
impl Request {
    pub fn decode(origin: [i32; 3], bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let height = r.u8()?;
        let id = r.u16_le()?;
        let delay = r.u16_le()?;
        let at = r.u8()?;
        if r.remaining() != 0 {
            return Err(Error::Invalid("world spot trailing bytes"));
        }
        Ok(Self {
            id,
            height,
            delay,
            position: [
                origin[0],
                origin[1] + ((at >> 4) & 7) as i32,
                origin[2] + (at & 7) as i32,
            ],
        })
    }
}

#[derive(Clone, Debug)]
pub struct Effect {
    pub position: [i32; 3],
    pub animation: crate::actor_spot::Effect,
    pub elevation: Option<i32>,
}
impl Request {
    pub fn start(self, cycle: i32) -> Effect {
        Effect {
            position: self.position,
            elevation: None,
            animation: crate::actor_spot::Effect {
                id: self.id,
                height: self.height as i32,
                start: cycle.wrapping_add(self.delay as i32),
                clock: Default::default(),
            },
        }
    }
}

impl Request {
    pub fn specific(r: &mut crate::buffer::Reader<'_>) -> crate::Result<Self> {
        let height = r.transformed(crate::buffer::Transform::Neg)?;
        let id = r.u16_add(false)?;
        let b = r.bytes(3)?;
        let packed = ((b[0] as i32) << 16) | ((b[2] as i32) << 8) | b[1] as i32;
        let delay = r.u16_add(false)?;
        Ok(Self {
            id,
            height,
            delay,
            position: [
                -1,
                (packed >> 16) + ((packed >> 4) & 7),
                ((packed >> 8) & 255) + (packed & 7),
            ],
        })
    }
}

/// Advance one game tick, retaining delayed effects and removing completed ones.
pub fn tick(
    effects: &mut Vec<Effect>,
    plane: i32,
    cycle: i32,
    definitions: &std::collections::HashMap<u32, crate::spot_animation::Definition>,
    sequences: &std::collections::HashMap<u32, crate::animation::Sequence>,
) -> Result<Vec<([i32; 2], u32, usize)>> {
    draw(effects, plane, cycle, 1, true, false, definitions, sequences)
}

/// cf.cu advances child graphics only when the parent admitted the visible scene.
/// The delta belongs to this draw, so skipped draws never accumulate per effect.
pub fn draw(
    effects: &mut Vec<Effect>,
    plane: i32,
    cycle: i32,
    ticks: u32,
    admitted: bool,
    hidden: bool,
    definitions: &std::collections::HashMap<u32, crate::spot_animation::Definition>,
    sequences: &std::collections::HashMap<u32, crate::animation::Sequence>,
) -> Result<Vec<([i32; 2], u32, usize)>> {
    if !admitted || hidden { return Ok(Vec::new()); }
    let mut events = Vec::new();
    let mut retained = Vec::new();
    for mut effect in effects.drain(..) {
        if effect.position[0] != plane {
            continue;
        }
        let definition = definitions
            .get(&(effect.animation.id as u32))
            .ok_or(Error::Invalid("world spot definition"))?;
        if definition.sequence < 0 {
            continue;
        }
        let id = definition.sequence as u32;
        if cycle >= effect.animation.start {
            let sequence = sequences
                .get(&id)
                .ok_or(Error::Invalid("world spot sequence"))?;
            let (flags, frames) = effect.animation.clock.advance(sequence, ticks)?;
            for frame in frames {
                if sequence.sounds.contains_key(&(frame as u16)) {
                    events.push((
                        [effect.position[1] * 128 + 64, effect.position[2] * 128 + 64],
                        id,
                        frame,
                    ));
                }
            }
            if flags & 1 != 0 {
                continue;
            }
        }
        retained.push(effect);
    }
    *effects = retained;
    Ok(events)
}
