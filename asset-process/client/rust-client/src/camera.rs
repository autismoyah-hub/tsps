//! Revision 239 14-bit camera angles, input inertia, and fixed-point orbit placement.
use crate::{interface_script::ints, vm::Stacks, Result};
pub struct State {
    pub pitch: i32,
    pub yaw: i32,
    pub locked: bool,
    pub unlocked_pitch: bool,
    pub velocity: [i32; 2],
    pub mouse: [i32; 2],
    pub mode: u8,
    pub sync_id: i32,
    pub recenter_pending: bool,
    pub shake: [Shake; 5],
    pub focus: [i32; 3],
    /// Last resolved following target, retained when the free camera takes over.
    pub tracked: [i32; 2],
    pub tracked_height: i32,
    pub terrain_pitch: i32,
    pub eye: [i32; 3],
    pub rendered_angles: [i32; 2],
    pub rotation: [Option<crate::camera_tween::Scalar>; 2],
    pub movement: Option<Target>,
    pub look: Option<Target>,
    pub target_heights: [i32; 2],
    pub path: Option<crate::camera_tween::Path>,
    pub path_absolute: bool,
    pending_path: Option<PathUpdate>,
    pub look_record: Option<Target>,
    pub look_path: Option<crate::camera_tween::Path>,
    pending_look: Option<LookUpdate>,
    pub resume_tracking: bool,
    pub smooth_reset: bool,
    pub return_eye: [i32; 3],
    pub return_angles: [i32; 2],
    pub movement_record: Option<Target>,
    pub follow_target: Option<(u8, i32)>,
    pub free_speed: i32,
    pub free_direction: i32,
    pub free_vertical: i32,
    report_delay: i32,
    report_pending: bool,
    base: Option<[i32; 2]>,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Target {
    pub tile: [i32; 2],
    pub height: i32,
    pub speed: i32,
    pub proportional: i32,
    pub snap: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathUpdate {
    pub tile: [i32; 2],
    pub height: i32,
    pub middle: Option<[i32; 2]>,
    pub duration: u16,
    pub easing: u8,
    pub absolute: bool,
    pub relative: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LookUpdate {
    pub tile: [i32; 2],
    pub height: i32,
    pub relative: bool,
    pub duration: u16,
    pub easing: u8,
    pub rotate: bool,
    pub resume: bool,
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Shake {
    pub requested: bool,
    pub active: bool,
    pub random_amplitude: i32,
    pub sine_amplitude: i32,
    pub frequency: i32,
    pub elapsed: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Update {
    Follow {
        kind: u8,
        id: i32,
    },
    SmoothReset,
    LookPath(LookUpdate),
    Path(PathUpdate),
    Reset,
    Mode(u8),
    Sync(i32),
    Unlock(bool),
    Shake([u8; 4]),
    Target {
        look: bool,
        relative: bool,
        target: Target,
    },
    Rotate {
        relative: bool,
        yaw: i16,
        pitch: i16,
        duration: u16,
        easing: u8,
    },
}
pub fn decode(packet: &crate::protocol::Packet) -> Result<Option<Update>> {
    use crate::buffer::Transform::{Add, Neg, Sub};
    let mut r = crate::buffer::Reader::new(&packet.payload);
    let update = match packet.opcode {
        1 => {
            let kind = r.u8()?;
            r.u16()?;
            Update::Follow {
                kind,
                id: r.u16()? as i32,
            }
        }
        87 => {
            let id = r.u32()? as i32;
            Update::Follow {
                kind: r.transformed(Neg)?,
                id,
            }
        }
        29 => {
            r.bytes(4)?;
            Update::SmoothReset
        }
        137 => {
            let duration = r.u16()?;
            let relative = r.transformed(Sub)? != 0;
            let x = r.u16()? as i32;
            let easing = r.transformed(Neg)?;
            let y = r.u16()? as i32;
            let height = r.u16_add(false)? as i16 as i32;
            Update::LookPath(LookUpdate {
                tile: [x, y],
                height,
                relative,
                duration,
                easing,
                rotate: false,
                resume: false,
            })
        }
        141 => {
            let duration = r.u16()?;
            let y = r.u16()? as i32;
            let height = r.u16()? as i32;
            let easing = r.transformed(Add)?;
            let x = r.u16_add(true)? as i32;
            Update::LookPath(LookUpdate {
                tile: [x, y], height, relative: false, duration, easing,
                rotate: true, resume: false,
            })
        }
        148 => {
            let resume = r.transformed(Add)? != 0;
            let y = r.u16_le()? as i32;
            let relative = r.u8()? != 0;
            let x = r.u16_add(true)? as i32;
            let duration = r.u16_le()?;
            let height = r.u16_add(true)? as i16 as i32;
            let easing = r.u8()?;
            Update::LookPath(LookUpdate {
                tile: [x, y],
                height,
                relative,
                duration,
                easing,
                rotate: true,
                resume,
            })
        }
        134 => {
            let duration = r.u16_add(true)?;
            let x = r.u16()? as i32;
            let easing = r.transformed(Add)?;
            let height = r.u16_add(false)? as i32;
            let y = r.u16_add(true)? as i32;
            let absolute = r.transformed(Sub)? & 1 != 0;
            Update::Path(PathUpdate {
                tile: [x, y],
                height,
                middle: None,
                duration,
                easing,
                absolute,
                relative: false,
            })
        }
        130 => {
            let absolute = r.u8()? & 1 != 0;
            let x = r.u16_add(false)? as i32;
            let duration = r.u16_le()?;
            let easing = r.transformed(Neg)?;
            let relative = r.transformed(Add)? != 0;
            let y = r.u16()? as i32;
            let height = r.u16_add(false)? as i16 as i32;
            Update::Path(PathUpdate {
                tile: [x, y],
                height,
                middle: None,
                duration,
                easing,
                absolute,
                relative,
            })
        }
        129 => {
            let duration = r.u16_add(true)?;
            let y = r.u16_add(false)? as i32;
            let my = r.u16_add(false)? as i32;
            let height = r.u16()? as i32;
            let easing = r.transformed(Neg)?;
            let x = r.u16_add(false)? as i32;
            let absolute = r.transformed(Sub)? & 1 != 0;
            let mx = r.u16_le()? as i32;
            Update::Path(PathUpdate {
                tile: [x, y],
                height,
                middle: Some([mx, my]),
                duration,
                easing,
                absolute,
                relative: false,
            })
        }
        128 => {
            let mx = r.u16_le()? as i32;
            let y = r.u16()? as i32;
            let my = r.u16_add(false)? as i32;
            let height = r.u16_le()? as i16 as i32;
            let absolute = r.transformed(Add)? & 1 != 0;
            let easing = r.transformed(Neg)?;
            let x = r.u16_le()? as i32;
            let relative = r.transformed(Add)? != 0;
            let duration = r.u16_add(false)?;
            Update::Path(PathUpdate {
                tile: [x, y],
                height,
                middle: Some([mx, my]),
                duration,
                easing,
                absolute,
                relative,
            })
        }
        3 => Update::Reset,
        38 => Update::Mode(r.u8()?),
        65 => Update::Sync(r.u32()? as i32),
        131 => Update::Unlock(r.u8()? == 1),
        103 => Update::Shake([r.u8()?, r.u8()?, r.u8()?, r.u8()?]),
        146 => {
            let x = r.u16_le()? as i32;
            let speed = r.transformed(Neg)? as i32;
            let height = r.u16_le()? as i32;
            let proportional = r.transformed(Neg)? as i32;
            let y = r.u16()? as i32;
            Update::Target {
                look: false,
                relative: false,
                target: Target {
                    tile: [x, y],
                    height,
                    speed,
                    proportional,
                    snap: proportional >= 100,
                },
            }
        }
        147 => {
            let y = r.u16()? as i32;
            let relative = r.u8()? != 0;
            let speed = r.u8()? as i32;
            let height = r.u16_add(false)? as i16 as i32;
            let x = r.u16_le()? as i32;
            let proportional = r.transformed(Add)? as i32;
            Update::Target {
                look: false,
                relative,
                target: Target {
                    tile: [x, y],
                    height,
                    speed,
                    proportional,
                    snap: proportional >= 100,
                },
            }
        }
        139 => {
            let y = r.u16()? as i32;
            let height = r.u16_add(false)? as i32;
            let proportional = r.u8()? as i32;
            let speed = r.transformed(Sub)? as i32 * 8;
            let x = r.u16_le()? as i32;
            Update::Target {
                look: true,
                relative: false,
                target: Target {
                    tile: [x, y],
                    height,
                    speed,
                    proportional,
                    snap: proportional >= 100,
                },
            }
        }
        144 => {
            let relative = r.transformed(Sub)? != 0;
            let speed = r.transformed(Neg)? as i32 * 8;
            let height = r.u16()? as i16 as i32;
            let y = r.u16_add(true)? as i32;
            let x = r.u16()? as i32;
            let proportional = r.transformed(Sub)? as i32;
            Update::Target {
                look: true,
                relative,
                target: Target {
                    tile: [x, y],
                    height,
                    speed,
                    proportional,
                    snap: proportional >= 100,
                },
            }
        }
        40 | 51 => Update::Rotate {
            relative: packet.opcode == 51,
            yaw: r.u16()? as i16,
            pitch: r.u16()? as i16,
            duration: r.u16()?,
            easing: r.u8()?,
        },
        _ => return Ok(None),
    };
    if r.remaining() != 0 {
        return Err(crate::Error::Invalid("camera packet trailing bytes"));
    }
    Ok(Some(update))
}
impl Default for State {
    fn default() -> Self {
        Self {
            pitch: 1024,
            yaw: 0,
            locked: false,
            unlocked_pitch: false,
            velocity: [0; 2],
            mouse: [0; 2],
            mode: 0,
            sync_id: 0,
            recenter_pending: false,
            shake: [Shake::default(); 5],
            focus: [0; 3],
            tracked: [0; 2],
            tracked_height: 0,
            terrain_pitch: 0,
            eye: [0; 3],
            rendered_angles: [0; 2],
            rotation: [None, None],
            movement: None,
            look: None,
            target_heights: [0; 2],
            path: None,
            path_absolute: false,
            pending_path: None,
            look_record: None,
            look_path: None,
            pending_look: None,
            resume_tracking: false,
            smooth_reset: false,
            return_eye: [0; 3],
            return_angles: [0; 2],
            movement_record: None,
            follow_target: None,
            free_speed: 0,
            free_direction: 0,
            free_vertical: 0,
            report_delay: 0,
            report_pending: false,
            base: None,
        }
    }
}
impl State {
    pub fn input_report(
        &mut self,
        held: &std::collections::HashSet<i32>,
    ) -> Option<crate::protocol::Packet> {
        if self.report_delay > 0 {
            self.report_delay -= 1;
        }
        self.report_pending |= (96..=99).any(|key| held.contains(&key));
        if !self.report_pending || self.report_delay > 0 {
            return None;
        }
        self.report_pending = false;
        self.report_delay = 20;
        let yaw = self.yaw / 8;
        let pitch = self.pitch / 8;
        Some(crate::protocol::Packet {
            opcode: 8,
            payload: vec![
                yaw as u8,
                (yaw >> 8) as u8,
                (pitch as u8).wrapping_add(128),
                (pitch >> 8) as u8,
            ],
        })
    }
    pub fn resolve_snaps(&mut self, scene: &crate::scene::Scene, plane: usize) {
        if let Some(update) = self.pending_look.take() {
            let old = self.look_record.unwrap_or(Target {
                tile: [scene.base_x, scene.base_y],
                height: self.target_heights[1],
                ..Target::default()
            });
            let height = if update.relative {
                old.height.wrapping_add(update.height)
            } else {
                update.height
            };
            let target = Target {
                tile: update.tile,
                height,
                snap: false,
                ..old
            };
            self.look_record = Some(target);
            self.target_heights[1] = height;
            if update.rotate {
                let point = target.point(scene, plane);
                // Coordinate rotations use the full double horizontal distance, unlike fb.bn's integer distance.
                let [x, y, z] = std::array::from_fn(|i| point[i].wrapping_sub(self.eye[i]));
                let horizontal = (x.wrapping_mul(x).wrapping_add(z.wrapping_mul(z)) as f64).sqrt();
                let factor = 2607.5945f32 as f64;
                let pitch = (((y as f64).atan2(horizontal) * factor) as i32).clamp(
                    if self.unlocked_pitch { 1 } else { 1024 },
                    if self.unlocked_pitch { 4160 } else { 3064 },
                );
                let mut yaw = (-((x as f64).atan2(z as f64) * factor) as i32) & 16383;
                if (yaw - self.rendered_angles[1]).abs() > 8192 {
                    yaw += if yaw < self.rendered_angles[1] {
                        16384
                    } else {
                        -16384
                    };
                }
                for (axis, end) in [pitch, yaw].into_iter().enumerate() {
                    self.rotation[axis] = Some(crate::camera_tween::Scalar::new(
                        self.rendered_angles[axis],
                        end,
                        update.duration as i32,
                        update.easing as i32,
                    ));
                }
                self.look = None;
                self.look_path = None;
                self.resume_tracking = update.resume;
            } else {
                let point = |t: Target| {
                    [
                        (t.tile[0] - scene.base_x) * 128 + 64,
                        t.height,
                        (t.tile[1] - scene.base_y) * 128 + 64,
                    ]
                };
                self.look_path = Some(crate::camera_tween::Path::new(
                    point(old),
                    point(target),
                    None,
                    update.duration as i32,
                    update.easing as i32,
                ));
                self.look = Some(target);
            }
        }
        if let Some(update) = self.pending_path.take() {
            let tile = |tile: [i32; 2]| {
                [
                    (tile[0] - scene.base_x) * 128 + 64,
                    (tile[1] - scene.base_y) * 128 + 64,
                ]
            };
            let [x, z] = tile(update.tile);
            let mut start = self.eye;
            let height = if update.absolute {
                if update.relative {
                    self.eye[1].wrapping_sub(update.height)
                } else {
                    scene
                        .elevation(plane, x, z)
                        .unwrap_or(0)
                        .wrapping_sub(update.height)
                }
            } else {
                start[1] = scene
                    .elevation(plane, start[0], start[2])
                    .unwrap_or(0)
                    .wrapping_sub(start[1]);
                if update.relative {
                    self.target_heights[0].wrapping_add(update.height)
                } else {
                    update.height
                }
            };
            self.path = Some(crate::camera_tween::Path::new(
                start,
                [x, height, z],
                update.middle.map(tile),
                update.duration as i32,
                update.easing as i32,
            ));
            self.path_absolute = update.absolute;
        }
        if let Some(target) = &mut self.movement {
            if target.snap {
                self.eye = target.point(scene, plane);
                target.snap = false;
            }
        }
        if let Some(target) = &mut self.look {
            if target.snap {
                self.rendered_angles =
                    look_angles(self.eye, target.point(scene, plane), self.unlocked_pitch);
                target.snap = false;
            }
        }
    }
    pub fn tick_locked(&mut self, scene: &crate::scene::Scene, plane: usize) {
        if !self.locked {
            return;
        }
        if self.smooth_reset {
            let movement = self.movement_record.unwrap_or_default();
            let look = self.look_record.unwrap_or_default();
            for axis in 0..3 {
                self.eye[axis] = approach(
                    self.eye[axis],
                    self.return_eye[axis],
                    movement.speed,
                    movement.proportional,
                );
            }
            self.rendered_angles = approach_angles(
                self.rendered_angles,
                self.return_angles,
                look.speed,
                look.proportional,
                self.unlocked_pitch,
            );
            if self.eye == self.return_eye && self.rendered_angles == self.return_angles {
                self.apply(Update::Reset);
            }
            return;
        }
        self.resolve_snaps(scene, plane);
        if let Some(target) = self.movement {
            let point = target.point(scene, plane);
            for axis in 0..3 {
                self.eye[axis] = approach(
                    self.eye[axis],
                    point[axis],
                    target.speed,
                    target.proportional,
                );
            }
        } else if let Some(path) = &mut self.path {
            self.eye = path.value();
            if !self.path_absolute {
                self.eye[1] = scene
                    .elevation(plane, self.eye[0], self.eye[2])
                    .unwrap_or(0)
                    .wrapping_sub(self.eye[1]);
            }
            path.clock.advance();
        }
        if let Some(mut target) = self.look {
            let point = if let Some(path) = &mut self.look_path {
                let mut point = path.value();
                point[1] = scene
                    .elevation(plane, point[0], point[2])
                    .unwrap_or(0)
                    .wrapping_sub(point[1]);
                path.clock.advance();
                let complete = path.clock.complete();
                target.speed = 100;
                self.look.as_mut().unwrap().speed = 100;
                self.look_record.as_mut().unwrap().speed = 100;
                if complete {
                    self.look_path = None;
                }
                point
            } else {
                target.point(scene, plane)
            };
            let angles = look_angles(self.eye, point, self.unlocked_pitch);
            self.rendered_angles = approach_angles(
                self.rendered_angles,
                angles,
                target.speed,
                target.proportional,
                self.unlocked_pitch,
            );
        } else {
            self.tick_rotation();
        }
    }
    pub fn view_pose(&mut self, height: i32, zoom: [i32; 2]) -> ([i32; 3], [i32; 2]) {
        let mut pitch = self.pitch.max(self.terrain_pitch / 256);
        if self.shake[4].requested {
            pitch = pitch.max(1024 + self.shake[4].sine_amplitude);
        }
        self.return_angles = [pitch, self.yaw];
        self.return_eye = position(
            self.focus,
            pitch,
            self.yaw,
            600 + 3 * (pitch / 8),
            height,
            zoom,
        );
        if !self.locked {
            self.rendered_angles = self.return_angles;
            self.eye = self.return_eye;
        }
        (self.eye, self.rendered_angles)
    }
    /// Temporary render pose: GameClient.im consumes only requested channels.
    /// Keeping canonical state intact also keeps roof selection independent of shake.
    pub fn shaken_pose(&self, mut random: impl FnMut() -> f64) -> ([i32; 3], [i32; 2]) {
        let mut eye = self.eye;
        let mut angles = self.rendered_angles;
        for (axis, shake) in self.shake.iter().enumerate() {
            if !shake.requested {
                continue;
            }
            let delta = (random() * (shake.random_amplitude * 2 + 1) as f64
                - shake.random_amplitude as f64
                + libm::sin(shake.elapsed as f64 * (shake.frequency as f64 / 100.0))
                    * shake.sine_amplitude as f64) as i32;
            match axis {
                0..=2 => eye[axis] = eye[axis].wrapping_add(delta),
                3 => angles[1] = angles[1].wrapping_add(delta) & 16383,
                4 => {
                    angles[0] = angles[0].wrapping_add(delta).clamp(
                        if self.unlocked_pitch { 1 } else { 1024 },
                        if self.unlocked_pitch { 4160 } else { 3064 },
                    )
                }
                _ => unreachable!(),
            }
        }
        (eye, angles)
    }
    pub fn tick_rotation(&mut self) {
        if !self.locked {
            return;
        }
        for axis in 0..2 {
            if let Some(tween) = &mut self.rotation[axis] {
                let mut value = tween.value() & 16383;
                if axis == 0 {
                    value = value.clamp(
                        if self.unlocked_pitch { 1 } else { 1024 },
                        if self.unlocked_pitch { 4160 } else { 3064 },
                    );
                }
                self.rendered_angles[axis] = value;
                tween.advance();
            }
        }
        if self.resume_tracking
            && self
                .rotation
                .iter()
                .all(|t| t.as_ref().is_none_or(|t| t.complete()))
        {
            self.rotation = [None, None];
            if let Some(mut target) = self.look_record {
                target.speed = 100;
                self.look_record = Some(target);
                self.look = Some(target);
            }
            self.resume_tracking = false;
        }
    }
    /// GameClient.method_if shifts root-local camera coordinates and releases locks.
    pub fn rebase(&mut self, old: [i32; 2], new: [i32; 2]) {
        for (axis, position_axis) in [(0, 0), (1, 2)] {
            let delta = old[axis].wrapping_sub(new[axis]).wrapping_mul(128);
            self.focus[position_axis] = self.focus[position_axis].wrapping_add(delta);
            self.eye[position_axis] = self.eye[position_axis].wrapping_add(delta);
            self.return_eye[position_axis] = self.return_eye[position_axis].wrapping_add(delta);
            self.tracked[axis] = self.tracked[axis].wrapping_add(delta);
        }
        self.locked = false;
        self.base = Some(new);
    }
    pub fn tick_follow(
        &mut self,
        scene: &crate::scene::Scene,
        target: [i32; 2],
        plane: usize,
        follow_height: i32,
    ) {
        let base = [scene.base_x, scene.base_y];
        if let Some(old) = self.base {
            if old != base {
                self.rebase(old, base);
            }
        }
        self.base = Some(base);
        let target = [target[0] - base[0] * 128, target[1] - base[1] * 128];
        self.tracked = target;
        let delta = [target[0] - self.focus[0], target[1] - self.focus[2]];
        if delta.iter().any(|&v| !(-500..=500).contains(&v)) {
            self.focus[0] = target[0];
            self.focus[2] = target[1];
        } else {
            self.focus[0] += delta[0] / 16;
            self.focus[2] += delta[1] / 16;
        }
        let tile = [self.focus[0] >> 7, self.focus[2] >> 7];
        let height = scene
            .elevation(plane, self.focus[0], self.focus[2])
            .unwrap_or(0);
        let mut drop = 0;
        if tile.iter().all(|&v| v > 3 && v < 100) {
            for x in tile[0] - 4..=tile[0] + 4 {
                for y in tile[1] - 4..=tile[1] + 4 {
                    let mut p = plane;
                    if p < 3
                        && scene
                            .tile(1, x as usize, y as usize)
                            .is_some_and(|tile| tile.flags & 2 != 0)
                    {
                        p += 1;
                    }
                    drop = drop.max(height - scene.height(p, x as usize, y as usize).unwrap_or(0));
                }
            }
        }
        let bounds = if self.unlocked_pitch {
            [1, 4160]
        } else {
            [1024, 3064]
        };
        let wanted = drop
            .wrapping_mul(192)
            .wrapping_shl(3)
            .clamp(bounds[0] * 256, bounds[1] * 256);
        self.terrain_pitch +=
            (wanted - self.terrain_pitch) / if wanted > self.terrain_pitch { 24 } else { 80 };
        self.set_target_height(scene.elevation(plane, target[0], target[1]).unwrap_or(0), follow_height);
    }
    pub fn set_target_height(&mut self, height: i32, follow_height: i32) {
        self.tracked_height = height;
        self.focus[1] = height.wrapping_sub(follow_height);
    }
    pub fn roof_plane(&self, scene: &crate::scene::Scene, plane: u8, hidden: bool) -> u8 {
        let view = crate::roof::View {
            plane,
            size: [scene.width as i32, scene.height as i32],
            hidden,
            eye: self.eye,
            pitch: self.rendered_angles[0],
            tracked: self.tracked,
            orb: (self.mode == 1).then_some([self.focus[0], self.focus[2]]),
        };
        let roof = |tile: [i32; 2]| {
            scene
                .tile(plane as usize, tile[0] as usize, tile[1] as usize)
                .is_some_and(|t| t.flags & 4 != 0)
        };
        if self.locked {
            view.locked(
                roof,
                scene
                    .elevation(plane as usize, self.eye[0], self.eye[2])
                    .unwrap_or(0),
            )
        } else {
            view.following(roof)
        }
    }
    pub fn apply(&mut self, update: Update) {
        match update {
            Update::Follow { kind, id } => self.follow_target = Some((kind, id)),
            Update::SmoothReset => {
                if self.locked {
                    self.smooth_reset = true;
                    for shake in &mut self.shake {
                        shake.requested = false;
                        shake.active = false;
                    }
                }
            }
            Update::LookPath(look) => {
                self.locked = true;
                self.smooth_reset = false;
                self.pending_look = Some(look);
            }
            Update::Path(path) => {
                self.locked = true;
                self.smooth_reset = false;
                self.movement = None;
                self.pending_path = Some(path);
            }
            Update::Reset => {
                self.locked = false;
                self.rotation = [None, None];
                self.movement = None;
                self.look = None;
                self.target_heights = [0; 2];
                self.path = None;
                self.pending_path = None;
                self.look_record = None;
                self.look_path = None;
                self.pending_look = None;
                self.resume_tracking = false;
                self.smooth_reset = false;
                self.movement_record = None;
                for shake in &mut self.shake {
                    shake.requested = false;
                    shake.active = false;
                }
            }
            Update::Mode(mode) => self.mode = mode,
            Update::Sync(id) => {
                if id != self.sync_id {
                    self.sync_id = id;
                    if self.mode == 1 { self.recenter_pending = true; }
                }
            }
            Update::Target {
                look,
                relative,
                mut target,
            } => {
                self.locked = true;
                self.smooth_reset = false;
                let index = usize::from(look);
                if relative {
                    target.height = target.height.wrapping_add(self.target_heights[index]);
                }
                self.target_heights[index] = target.height;
                if look {
                    self.look_record = Some(target);
                    self.look_path = None;
                    self.pending_look = None;
                } else {
                    self.pending_path = None;
                    self.movement_record = Some(target);
                }
                let slot = if look {
                    &mut self.look
                } else {
                    &mut self.movement
                };
                *slot = Some(target);
            }
            Update::Unlock(unlocked) => self.unlocked_pitch = unlocked,
            Update::Shake([index, random, sine, frequency]) => {
                if let Some(shake) = self.shake.get_mut(index as usize) {
                    *shake = Shake {
                        requested: true,
                        active: false,
                        random_amplitude: random as i32 * if index >= 3 { 8 } else { 1 },
                        sine_amplitude: sine as i32 * if index >= 3 { 8 } else { 1 },
                        frequency: frequency as i32,
                        elapsed: 0,
                    };
                }
            }
            Update::Rotate {
                relative,
                yaw,
                pitch,
                duration,
                easing,
            } => {
                self.locked = true;
                self.smooth_reset = false;
                self.look = None;
                self.look_path = None;
                let bounds = if self.unlocked_pitch {
                    [1, 4160]
                } else {
                    [1024, 3064]
                };
                for (axis, value) in [pitch as i32 * 8, yaw as i32 * 8].into_iter().enumerate() {
                    if if relative { value == 0 } else { value < 0 } {
                        continue;
                    }
                    let start = self.rendered_angles[axis];
                    let mut end = if relative {
                        start.wrapping_add(value)
                    } else {
                        value
                    };
                    if axis == 0 {
                        end = end.clamp(bounds[0], bounds[1]);
                    } else if !relative {
                        end &= 16383;
                        if (end - start).abs() > 8192 {
                            end += if end < start { 16384 } else { -16384 };
                        }
                    }
                    self.rotation[axis] = Some(crate::camera_tween::Scalar::new(
                        start,
                        end,
                        duration as i32,
                        easing as i32,
                    ));
                }
            }
        }
    }
    pub fn execute(&mut self, opcode: u16, stacks: &mut Stacks) -> Result<bool> {
        match opcode {
            5504 => {
                let [pitch, yaw] = ints::<2>(stacks)?;
                if !self.locked {
                    self.pitch = legacy_angle(pitch);
                    self.yaw = legacy_angle(yaw);
                }
            }
            5505 => stacks.push_int(self.pitch / 8)?,
            5506 => stacks.push_int(self.yaw / 8)?,
            _ => return Ok(false),
        }
        Ok(true)
    }
    pub fn tick_input(
        &mut self,
        held: &std::collections::HashSet<i32>,
        pointer: [i32; 2],
        mouse_drag: bool,
    ) {
        if mouse_drag {
            let dy = pointer[1].wrapping_sub(self.mouse[1]);
            self.velocity[0] = (dy.wrapping_mul(2) as f32 * 8.0) as i32;
            self.mouse[1] = if dy == -1 || dy == 1 {
                pointer[1]
            } else {
                pointer[1].wrapping_add(self.mouse[1]) / 2
            };
            let dx = self.mouse[0].wrapping_sub(pointer[0]);
            self.velocity[1] = (dx.wrapping_mul(2) as f32 * 8.0) as i32;
            self.mouse[0] = if dx == -1 || dx == 1 {
                pointer[0]
            } else {
                pointer[0].wrapping_add(self.mouse[0]) / 2
            };
        } else {
            for (axis, first, second, speed) in [(1, 96, 97, -192i32), (0, 98, 99, 96i32)] {
                let target = if held.contains(&first) {
                    speed
                } else if held.contains(&second) {
                    -speed
                } else {
                    0
                };
                self.velocity[axis] = if target == 0 {
                    self.velocity[axis] / 2
                } else {
                    self.velocity[axis].wrapping_add(target.wrapping_sub(self.velocity[axis]) / 2)
                };
            }
            self.mouse = pointer;
        }
        self.yaw = self.yaw.wrapping_add(self.velocity[1] / 2) & 16383;
        let limits = if self.unlocked_pitch {
            [1, 4160]
        } else {
            [1024, 3064]
        };
        self.pitch = self
            .pitch
            .wrapping_add(self.velocity[0] / 2)
            .clamp(limits[0], limits[1]);
    }
    /// GameClient's pending mode-1 recenter waits for a valid resolved focus target.
    pub fn recenter_free(&mut self, scene: &crate::scene::Scene, target: [i32; 2], plane: usize, follow_height: i32) {
        if self.mode != 1 || !self.recenter_pending { return; }
        let local = [target[0].wrapping_sub(scene.base_x.wrapping_mul(128)),
            target[1].wrapping_sub(scene.base_y.wrapping_mul(128))];
        if local[0] < 0 || local[1] < 0 || (local[0] >> 7) as usize >= scene.width
            || (local[1] >> 7) as usize >= scene.height { return; }
        self.focus[0] = local[0];
        self.focus[2] = local[1];
        let height = scene.elevation(plane, local[0], local[1]).unwrap_or(0);
        self.focus[1] = self.focus[1].min(height.wrapping_sub(follow_height));
        self.recenter_pending = false;
    }
    /// Original mode-1 camera translation. Returns true when Escape requests EXIT_FREECAM.
    pub fn tick_free(&mut self, held: &std::collections::HashSet<i32>, speeds: [i32; 2]) -> bool {
        let mut direction = if held.contains(&33) {
            0
        } else if held.contains(&49) {
            8192
        } else {
            -1
        };
        if held.contains(&48) {
            direction = match direction {
                0 => 14336,
                8192 => 10240,
                _ => 12288,
            };
        } else if held.contains(&50) {
            direction = match direction {
                0 => 2048,
                8192 => 6144,
                _ => 4096,
            };
        }
        let vertical = if held.contains(&35) {
            -1
        } else if held.contains(&51) {
            1
        } else {
            0
        };
        let wanted = if direction >= 0 || vertical != 0 {
            self.free_direction = direction;
            self.free_vertical = vertical;
            speeds[usize::from(held.contains(&81))].wrapping_mul(16)
        } else {
            0
        };
        if self.free_speed < wanted {
            self.free_speed = self.free_speed.wrapping_add(wanted / 8).min(wanted);
        } else if self.free_speed > wanted {
            self.free_speed = self.free_speed.wrapping_mul(9) / 10;
        }
        if self.free_speed > 0 {
            let step = self.free_speed / 16;
            if self.free_direction >= 0 {
                let angle = (self.free_direction - self.rendered_angles[1]) & 16383;
                let radians = angle as f64 * 3.834951969714103E-4;
                let sin = (radians.sin() * 65536.0) as i32;
                let cos = (radians.cos() * 65536.0) as i32;
                self.focus[0] = self.focus[0].wrapping_add(sin.wrapping_mul(step) / 65536);
                self.focus[2] = self.focus[2].wrapping_add(cos.wrapping_mul(step) / 65536);
            }
            if self.free_vertical != 0 {
                self.focus[1] = self.focus[1]
                    .wrapping_add(step.wrapping_mul(self.free_vertical))
                    .min(0);
            }
        } else {
            self.free_direction = -1;
            self.free_vertical = -1;
        }
        let wanted = if self.unlocked_pitch { 256 } else { 1024 * 256 };
        self.terrain_pitch +=
            (wanted - self.terrain_pitch) / if wanted > self.terrain_pitch { 24 } else { 80 };
        if held.contains(&13) {
            self.mode = 0;
            true
        } else {
            false
        }
    }
}
impl Target {
    fn point(self, scene: &crate::scene::Scene, plane: usize) -> [i32; 3] {
        let x = (self.tile[0] - scene.base_x)
            .wrapping_mul(128)
            .wrapping_add(64);
        let z = (self.tile[1] - scene.base_y)
            .wrapping_mul(128)
            .wrapping_add(64);
        [
            x,
            scene
                .elevation(plane, x, z)
                .unwrap_or(0)
                .wrapping_sub(self.height),
            z,
        ]
    }
}
/// rr.bq: approach each coordinate with integer speed and proportional acceleration.
pub fn approach(mut value: i32, target: i32, speed: i32, proportional: i32) -> i32 {
    if value < target {
        value = value.wrapping_add(
            speed.wrapping_add(target.wrapping_sub(value).wrapping_mul(proportional) / 1000),
        );
        if value > target {
            value = target;
        }
    }
    if value > target {
        value = value.wrapping_sub(
            speed.wrapping_add(value.wrapping_sub(target).wrapping_mul(proportional) / 1000),
        );
        if value < target {
            value = target;
        }
    }
    value
}
pub fn look_angles(eye: [i32; 3], target: [i32; 3], unlocked: bool) -> [i32; 2] {
    let [x, y, z] = std::array::from_fn(|i| target[i].wrapping_sub(eye[i]));
    let horizontal = (x.wrapping_mul(x).wrapping_add(z.wrapping_mul(z)) as f64).sqrt() as i32;
    let factor = 2607.5945f32 as f64;
    let pitch = ((y as f64).atan2(horizontal as f64) * factor) as i32;
    let yaw = ((x as f64).atan2(z as f64) * -factor) as i32;
    [
        pitch.clamp(
            if unlocked { 1 } else { 1024 },
            if unlocked { 4160 } else { 3064 },
        ),
        yaw & 16383,
    ]
}
/// fb.bn: yaw uses the shortest signed turn and clamps when it crosses its target.
pub fn approach_angles(
    current: [i32; 2],
    target: [i32; 2],
    speed: i32,
    proportional: i32,
    unlocked: bool,
) -> [i32; 2] {
    let pitch = target[0].clamp(
        if unlocked { 1 } else { 1024 },
        if unlocked { 4160 } else { 3064 },
    );
    let pitch = if current[0] < pitch {
        current[0]
            .wrapping_add(
                speed
                    .wrapping_add(pitch.wrapping_sub(current[0]).wrapping_mul(proportional) / 1000),
            )
            .min(pitch)
    } else if current[0] > pitch {
        current[0]
            .wrapping_sub(
                speed
                    .wrapping_add(current[0].wrapping_sub(pitch).wrapping_mul(proportional) / 1000),
            )
            .max(pitch)
    } else {
        pitch
    };
    let delta = |v: i32| {
        if v > 8192 {
            v - 16384
        } else if v < -8192 {
            v + 16384
        } else {
            v
        }
    };
    let target = target[1] & 16383;
    let before = delta(target - current[1]);
    let mut yaw = current[1];
    if before > 0 {
        yaw = yaw.wrapping_add(before.wrapping_mul(proportional) / 1000 + speed);
    } else if before < 0 {
        yaw = yaw.wrapping_sub(before.wrapping_neg().wrapping_mul(proportional) / 1000 + speed);
    }
    let after = delta(target.wrapping_sub(yaw));
    if (after < 0 && before > 0) || (after > 0 && before < 0) {
        yaw = target;
    }
    [pitch & 16383, yaw & 16383]
}
pub fn legacy_angle(value: i32) -> i32 {
    ((value as f32 * 8.0) as i32) & 16383
}

/// yb.dk: eye position relative to an orbit focus. Arithmetic intentionally wraps like Java int.
pub fn position(
    focus: [i32; 3],
    pitch: i32,
    yaw: i32,
    distance: i32,
    height: i32,
    zoom: [i32; 2],
) -> [i32; 3] {
    let delta = height.wrapping_sub(334).clamp(0, 100);
    let zoom = zoom[0].wrapping_add(delta.wrapping_mul(zoom[1].wrapping_sub(zoom[0])) / 100);
    let mut z = zoom.wrapping_mul(distance) / 256;
    let mut x = 0i32;
    let mut y = 0i32;
    let pitch = 16384i32.wrapping_sub(pitch) & 16383;
    let yaw = 16384i32.wrapping_sub(yaw) & 16383;
    let trig = |angle: i32| {
        let radians = angle as f64 * 3.834951969714103E-4;
        [
            (radians.sin() * 65536.0) as i32,
            (radians.cos() * 65536.0) as i32,
        ]
    };
    if pitch != 0 {
        let [sin, cos] = trig(pitch);
        let next = cos.wrapping_mul(y).wrapping_sub(z.wrapping_mul(sin)) >> 16;
        z = z.wrapping_mul(cos).wrapping_add(sin.wrapping_mul(y)) >> 16;
        y = next;
    }
    if yaw != 0 {
        let [sin, cos] = trig(yaw);
        let next = sin.wrapping_mul(z).wrapping_add(cos.wrapping_mul(x)) >> 16;
        z = cos.wrapping_mul(z).wrapping_sub(x.wrapping_mul(sin)) >> 16;
        x = next;
    }
    [
        focus[0].wrapping_sub(x),
        focus[1].wrapping_sub(y),
        focus[2].wrapping_sub(z),
    ]
}
