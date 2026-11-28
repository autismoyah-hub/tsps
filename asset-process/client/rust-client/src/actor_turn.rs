//! Discrete actor rotation and idle-turn pose selection (aa.by).
#[derive(Clone, Copy, Debug, Default)]
pub struct State {
    pub angle: u16,
    pub target: u16,
    pub elapsed: i32,
}
impl State {
    /// Poses are idle, left turn, right turn and forward walk.
    pub fn tick(&mut self, speed: u16, frozen: bool, mut pose: i32, poses: [i32; 4]) -> i32 {
        if frozen {
            return pose;
        }
        let difference = (self.target as i32 - self.angle as i32) & 2047;
        if difference == 0 {
            self.elapsed = 0;
            return pose;
        }
        self.elapsed = self.elapsed.wrapping_add(1);
        let direction = if difference > 1024 { -1 } else { 1 };
        let mut angle = self.angle as i32 + speed as i32 * direction;
        let rotating = difference >= speed as i32 && difference <= 2048 - speed as i32;
        if !rotating {
            angle = self.target as i32;
        }
        if speed > 0 && pose == poses[0] && (self.elapsed > 25 || rotating) {
            pose = if direction == -1 && poses[1] != -1 {
                poses[1]
            } else if direction == 1 && poses[2] != -1 {
                poses[2]
            } else {
                poses[3]
            };
        }
        self.angle = (angle & 2047) as u16;
        pose
    }
}
pub fn path_angle(position: [i32; 2], target: [i32; 2]) -> Option<u16> {
    use std::cmp::Ordering::*;
    match (position[0].cmp(&target[0]), position[1].cmp(&target[1])) {
        (Less, Less) => Some(1280),
        (Less, Greater) => Some(1792),
        (Less, Equal) => Some(1536),
        (Greater, Less) => Some(768),
        (Greater, Greater) => Some(256),
        (Greater, Equal) => Some(512),
        (Equal, Less) => Some(1024),
        (Equal, Greater) => Some(0),
        (Equal, Equal) => None,
    }
}

/// Root-world facing selection (kh.bu), before path advancement.
pub fn select(
    turn: &mut State,
    face: &mut Option<crate::actor_update::Face>,
    path: &crate::actor_path::Path,
    size: i32,
    blocked: bool,
    mut entity: impl FnMut(u16, i32) -> Option<[i32; 2]>,
) {
    select_resolved(turn,face,path,size,blocked,|kind,index| {
        entity(kind,index).map(|target| toward(path.position,target))
    });
}
pub fn select_resolved(
    turn: &mut State,
    face: &mut Option<crate::actor_update::Face>,
    path: &crate::actor_path::Path,
    size: i32,
    blocked: bool,
    mut entity: impl FnMut(u16, i32) -> Option<Option<u16>>,
) {
    use crate::actor_update::FaceTarget;
    let mut resolved = None;
    let mut instant = false;
    let mut walking = false;
    if let Some(face) = face {
        walking = face.turn_on_walk;
        match face.target {
            FaceTarget::Angle(angle) if walking || path.len == 0 || path.stalled > 0 => {
                resolved = Some(angle);
                instant = face.instant;
                face.instant = false;
                face.target = FaceTarget::Reset;
            }
            FaceTarget::Tile { x, z, size } if walking || path.len == 0 => {
                let target = [
                    x as i32 * 128 + size[0] as i32 * 64,
                    z as i32 * 128 + size[1] as i32 * 64,
                ];
                resolved = Some(toward(path.position, target).unwrap_or(turn.target));
                instant = face.instant;
                face.instant = false;
            }
            FaceTarget::Entity {
                kind,
                index,
                fallback,
            } if kind != 0 => {
                resolved = Some(entity(kind, index).map_or(fallback, |angle| angle.unwrap_or(turn.target)));
            }
            _ => {}
        }
    }
    if resolved.is_none() && path.len > 0 && !walking && !blocked {
        resolved = path_angle(
            path.position,
            path.tiles[path.len - 1].map(|v| v * 128 + size * 64),
        );
    }
    if let Some(angle) = resolved {
        turn.target = angle;
    }
    if instant {
        turn.angle = turn.target;
    }
}
pub fn toward(position: [i32; 2], target: [i32; 2]) -> Option<u16> {
    let x = position[0].wrapping_sub(target[0]);
    let z = position[1].wrapping_sub(target[1]);
    if x == 0 && z == 0 {
        None
    } else {
        Some(((libm::atan2(x as f64, z as f64) * 325.94932345220167) as i32 & 2047) as u16)
    }
}
