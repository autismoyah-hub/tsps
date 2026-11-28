//! Persistent locomotion/idle animation clocks (GameActor.dc and GameClient.nc).
use crate::{actor_freeze::Freezes, actor_sequence::State, animation::Sequence, Error, Result};
use std::collections::HashMap;
#[derive(Clone, Debug, Default)]
pub struct Pose {
    pub state: State,
}
impl Pose {
    /// Separate idle clock maintained after the locomotion clock in GameClient.nc.
    pub fn tick_fallback(
        &mut self,
        idle: i32,
        pose: &State,
        sequences: &HashMap<u32, Sequence>,
        frozen: bool,
        assets_ready: bool,
    ) -> Result<()> {
        if frozen {
            return Ok(());
        }
        if idle < 0 {
            self.state.reset();
        } else if self.state.id != idle {
            self.state.id = idle;
            self.state.loading = 0;
            if self.state.frame == -1 {
                self.state.frame = 0;
            }
        }
        if pose.loading > 0 && self.state.loading < 30 {
            if pose.id == self.state.id {
                self.state = pose.clone();
            } else {
                self.tick(sequences, false, assets_ready)?;
            }
        }
        Ok(())
    }
    pub fn set(&mut self, id: i32, idle: i32, restart_idle: bool) {
        let s = &mut self.state;
        if s.id == id {
            return;
        }
        if restart_idle && (s.id == idle || id == idle) {
            s.frame = 0;
            s.elapsed = 0;
            s.loops = 0;
        }
        if id >= 0 {
            s.loading = 0;
            if s.frame == -1 {
                s.frame = 0;
            }
        } else {
            s.frame = -1;
        }
        s.id = id;
    }
    pub fn tick(
        &mut self,
        sequences: &HashMap<u32, Sequence>,
        frozen: bool,
        assets_ready: bool,
    ) -> Result<Vec<usize>> {
        let s = &mut self.state;
        if frozen {
            return Ok(vec![]);
        }
        if s.id < 0 {
            s.loading = 0;
            return Ok(vec![]);
        }
        let sequence = sequences
            .get(&(s.id as u32))
            .ok_or(Error::Invalid("actor pose sequence"))?;
        s.loading = if assets_ready {
            0
        } else {
            s.loading.wrapping_add(1)
        };
        let mut clock = s.clock();
        let (flags, events) = clock.advance(sequence, 1)?;
        s.frame = clock.frame as i32;
        s.elapsed = clock.elapsed;
        s.loops = clock.loops;
        if flags & 2 != 0 {
            s.frame = 0;
            s.elapsed = 0;
            s.loops = 0;
        }
        Ok(events)
    }
}

/// Select the locomotion sequence before discrete path bookkeeping.
pub fn movement(
    poses: [i32; 15],
    path: &crate::actor_path::Path,
    m: crate::actor_path::Movement,
    angle: u16,
    target: u16,
) -> i32 {
    if path.len == 0 || m.blocked {
        return poses[0];
    }
    let tile = path.tiles[path.len - 1].map(|v| v * 128 + m.size * 64);
    if (tile[0] as f32 - path.fine[0])
        .abs()
        .max((tile[1] as f32 - path.fine[1]).abs())
        > 288.0
    {
        return poses[0];
    }
    let mut difference = (target as i32 - angle as i32) & 2047;
    if difference > 1024 {
        difference -= 2048;
    }
    let mut pose = if (-256..=256).contains(&difference) {
        poses[3]
    } else if (256..768).contains(&difference) {
        poses[6]
    } else if (-768..=-256).contains(&difference) {
        poses[5]
    } else {
        poses[4]
    };
    if pose == -1 {
        pose = poses[3];
    }
    let speed = path.speed(m);
    let offset = if speed >= 8 {
        7
    } else if speed <= 2 {
        11
    } else {
        return pose;
    };
    for slot in 0..4 {
        if pose == poses[slot + 3] && poses[offset + slot] != -1 {
            return poses[offset + slot];
        }
    }
    pose
}

/// GamePlayer.gi / GameNPC.hx skips a model while its fallback is still within the loading grace period.
pub fn select<'a>(
    freezes: &'a Freezes,
    action: &'a State,
    pose: &'a State,
    fallback: &'a State,
    idle: i32,
    low_detail: bool,
    mut ready: impl FnMut(i32) -> bool,
) -> Option<(Option<&'a State>, Option<&'a State>)> {
    let action = freezes
        .displayed(action)
        .filter(|s| s.id >= 0 && ready(s.id));
    let pose = if low_detail {
        None
    } else {
        freezes
            .displayed_pose(pose)
            .filter(|s| s.id >= 0 && ready(s.id) && (action.is_none() || s.id != idle))
    };
    if action.is_none() && pose.is_none() {
        if fallback.id >= 0 && (1..30).contains(&fallback.loading) {
            return None;
        }
        return Some((None, (fallback.id >= 0).then_some(fallback)));
    }
    Some((action, pose))
}
