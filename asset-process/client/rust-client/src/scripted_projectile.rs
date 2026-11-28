//! Script-curve projectile trajectories (ca.cq), with Java float operation order.
use crate::Result;
use std::collections::BTreeMap;
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Endpoint {
    pub tile: [i32; 3],
    pub offset: [i32; 3],
    pub entity: i32,
}
#[derive(Clone, Debug, Default)]
pub struct Definition {
    pub spot: u16,
    pub source: Endpoint,
    pub target: Endpoint,
    pub start: i32,
    pub end: i32,
    pub curves: [i32; 3],
}
#[derive(Clone, Debug)]
pub enum Event {
    Create {
        key: u16,
        definition: Definition,
    },
    Update {
        key: u16,
        target: Endpoint,
        pause: Option<u16>,
        stop: bool,
    },
}
impl Event {
    pub fn decode(kind: u8, origin: [i32; 3], plane: i32, bytes: &[u8]) -> Result<Self> {
        use crate::{
            buffer::{Reader, Transform},
            Error,
        };
        let mut r = Reader::new(bytes);
        let event = match kind {
            12 => {
                let source_x = r.u16_le()? as i16 as i32;
                let spot = r.u16()?;
                let target_z = r.u16()? as i16 as i32;
                let tile = coordinate(r.u32_middle()?);
                let start = r.u16_add(false)? as i32;
                let source_height = r.u16()? as i16 as i32;
                let y_curve = r.u16_le()? as i32;
                let entity = middle_entity(&mut r)?;
                let key = r.u16()?;
                let end = r.u16_le()? as i32;
                let at = r.u8()?;
                let target_x = r.u16()? as i16 as i32;
                let x_curve = r.u16_le()? as i32;
                let source_z = r.u16_add(true)? as i16 as i32;
                let target_height = r.u16_le()? as i16 as i32;
                let b = r.bytes(3)?;
                let source_entity =
                    ((((b[0] as i32) << 16) | ((b[2] as i32) << 8) | b[1] as i32) << 8) >> 8;
                let z_curve = r.u16_le()? as i32;
                Event::Create {
                    key,
                    definition: Definition {
                        spot,
                        source: Endpoint {
                            tile: [
                                plane,
                                origin[1] + ((at >> 4) & 7) as i32,
                                origin[2] + (at & 7) as i32,
                            ],
                            offset: [source_x, source_height, source_z],
                            entity: source_entity,
                        },
                        target: Endpoint {
                            tile,
                            offset: [target_x, target_height, target_z],
                            entity,
                        },
                        start,
                        end,
                        curves: [x_curve, y_curve, z_curve],
                    },
                }
            }
            1 => {
                let z = r.u16_le()? as i16 as i32;
                let height = r.u16_add(false)? as i16 as i32;
                let packed = r.u32_inverse_middle()?;
                let tile = if packed == u32::MAX {
                    [-1; 3]
                } else {
                    coordinate(packed)
                };
                let x = r.u16()? as i16 as i32;
                let entity = middle_entity(&mut r)?;
                let pause = r.u16_le()?;
                let key = r.u16()?;
                let stop = r.transformed(Transform::Add)? == 1;
                Event::Update {
                    key,
                    target: Endpoint {
                        tile,
                        offset: [x, height, z],
                        entity,
                    },
                    pause: (pause != 65535).then_some(pause),
                    stop,
                }
            }
            _ => return Err(Error::Invalid("scripted projectile record")),
        };
        if r.remaining() != 0 {
            return Err(Error::Invalid("scripted projectile trailing bytes"));
        }
        Ok(event)
    }
}
fn coordinate(p: u32) -> [i32; 3] {
    [
        (p as i32) >> 28,
        ((p >> 14) & 16383) as i32,
        (p & 16383) as i32,
    ]
}
fn middle_entity(r: &mut crate::buffer::Reader) -> Result<i32> {
    let b = r.bytes(3)?;
    Ok(((((b[1] as i32) << 16) | ((b[0] as i32) << 8) | b[2] as i32) << 8) >> 8)
}
#[derive(Clone, Debug)]
pub struct Effect {
    pub serial: u64,
    pub visible: bool,
    pub motion: Motion,
    pub clock: crate::animation_runtime::Clock,
    pub sequence: i32,
    pub pending_shared: bool,
}
/// The target uses shared rd/rc packet objects. Even duplicate create keys mutate
/// the definition seen by existing effects; pending redirects share the last rc.
#[derive(Default)]
pub struct State {
    pub definition: Definition,
    pub redirect: Endpoint,
    pub effects: BTreeMap<u16, Effect>,
    serial: u64,
}
impl State {
    pub fn ordered_keys(&self) -> Vec<u16> {
        let mut keys = self.effects.keys().copied().collect::<Vec<_>>();
        keys.sort_by_key(|key| (*key & 63, self.effects[key].serial));
        keys
    }
    pub fn apply(
        &mut self,
        event: Event,
        cycle: i32,
        mut point: impl FnMut(&Endpoint) -> Result<[f32; 3]>,
        sequence: impl FnOnce(u16) -> Result<i32>,
    ) -> Result<()> {
        match event {
            Event::Create {
                key,
                mut definition,
            } => {
                definition.start = definition.start.wrapping_add(cycle);
                definition.end = definition.end.wrapping_add(cycle);
                self.definition = definition;
                if !self.effects.contains_key(&key) {
                    let position = point(&self.definition.source)?;
                    self.serial = self.serial.wrapping_add(1);
                    self.effects.insert(
                        key,
                        Effect {
                            serial: self.serial,
                            visible: false,
                            motion: Motion::new(position),
                            clock: Default::default(),
                            sequence: sequence(self.definition.spot)?,
                            pending_shared: false,
                        },
                    );
                }
            }
            Event::Update {
                key,
                target,
                pause,
                stop,
            } => {
                self.redirect = target;
                if let Some(effect) = self.effects.get_mut(&key) {
                    if self.redirect.tile[1] != -1 {
                        effect.pending_shared = true;
                    }
                    if let Some(delay) = pause {
                        effect.motion.pause_until = cycle.wrapping_add(delay as i32);
                        effect.motion.stop_after_pause = stop;
                    }
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct Motion {
    pub position: [f32; 3],
    pub origin: [f32; 3],
    pub redirected: bool,
    pub redirect_progress: f32,
    pub pause_until: i32,
    pub stop_after_pause: bool,
    pub yaw: u16,
    pub pitch: u16,
    pub pending: Option<Endpoint>,
}
impl Motion {
    pub fn new(origin: [f32; 3]) -> Self {
        Self {
            position: origin,
            origin,
            redirected: false,
            redirect_progress: -1.0,
            pause_until: 0,
            stop_after_pause: false,
            yaw: 0,
            pitch: 0,
            pending: None,
        }
    }
    pub fn redirect(&mut self, target: Endpoint, pause: i32, stop: bool) {
        if target.tile[1] != -1 {
            self.pending = Some(target);
        }
        if pause != -1 {
            self.pause_until = pause;
            self.stop_after_pause = stop;
        }
    }
    pub fn advance(
        &mut self,
        definition: &mut Definition,
        target: [f32; 3],
        cycle: i32,
        ticks: i32,
        mut curve: impl FnMut(i32, i32) -> Result<i32>,
    ) -> Result<bool> {
        if self.pause_until > 0 {
            if self.pause_until.wrapping_sub(cycle) > 0 {
                definition.start = definition.start.wrapping_add(ticks);
                definition.end = definition.end.wrapping_add(ticks);
            } else {
                self.pause_until = 0;
                if self.stop_after_pause {
                    definition.end = 0;
                }
            }
            return Ok(false);
        }
        let progress = cycle.wrapping_sub(definition.start) as f32
            / definition.end.wrapping_sub(definition.start) as f32;
        let adjusted = if !self.redirected {
            progress
        } else {
            let remaining = 1.0 - self.redirect_progress;
            if remaining <= 1.0e-6 {
                1.0
            } else {
                (progress - self.redirect_progress) / remaining
            }
            .clamp(0.0, 1.0)
        };
        let input = (((adjusted * 65536.0) as f64 + 0.5).floor()) as i32;
        let components = [
            curve(definition.curves[0], input)? as f32 / 65536.0,
            curve(definition.curves[1], input)? as f32 / 65536.0,
            curve(definition.curves[2], input)? as f32 / 65536.0,
        ];
        let displacement = std::array::from_fn(|i| target[i] - self.origin[i]);
        let distance = length(displacement);
        let direction = normalize(displacement);
        let mut side = cross([0.0, 1.0, 0.0], direction);
        if length(side) < 1.0e-6 {
            side = cross([1.0, 0.0, 0.0], direction);
        }
        let side = normalize(side);
        let up = normalize(cross(direction, side));
        let scales = [
            components[0] * 128.0,
            128.0 * components[1],
            distance * components[2],
        ];
        let mut next = [0.0; 3];
        for (axis, scale) in [side, up, direction].into_iter().zip(scales) {
            for i in 0..3 {
                next[i] += axis[i] * scale;
            }
        }
        for i in 0..3 {
            next[i] += self.origin[i];
        }
        let delta: [f32; 3] = std::array::from_fn(|i| next[i] - self.position[i]);
        self.position = next;
        let horizontal = ((delta[0] * delta[0] + delta[2] * delta[2]) as f64).sqrt() as f32;
        if horizontal > 0.0 {
            self.yaw = (((libm::atan2(delta[0] as f64, delta[2] as f64) * 325.9493f32 as f64)
                as i32)
                .wrapping_add(1024)
                & 2047) as u16;
            self.pitch = ((libm::atan2(delta[1] as f64, horizontal as f64) * 325.9493f32 as f64)
                as i32
                & 2047) as u16;
        }
        if let Some(target) = self.pending.take() {
            self.origin = self.position;
            self.redirect_progress = progress;
            self.redirected = true;
            definition.target = target;
        }
        Ok(true)
    }
}
fn length(v: [f32; 3]) -> f32 {
    ((v[1] * v[1] + v[0] * v[0] + v[2] * v[2]) as f64).sqrt() as f32
}
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let reciprocal = 1.0 / length(v);
    v.map(|n| n * reciprocal)
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        b[1] * a[0] - b[0] * a[1],
    ]
}
