//! Synchronous equivalent of GameAnimation.ad asset readiness.
use crate::{
    animation::{Frame, Sequence, Skeleton},
    cached_animation::CachedAnimation,
    disk_cache::DiskCache,
    skeletal::Rig,
    Error, Result,
};
use std::collections::{HashMap, HashSet};
#[derive(Default)]
pub struct Readiness {
    legacy: HashSet<u32>,
    cached: HashSet<u32>,
    known: HashMap<(u8, u32), bool>,
}
impl Readiness {
    pub fn known(&self, sequence: &Sequence) -> Option<bool> {
        let key = if let Some(id) = sequence.cached_animation {
            (22, id)
        } else if let Some(&frame) = sequence.frames.first() {
            (0, frame >> 16)
        } else {
            return Some(false);
        };
        self.known.get(&key).copied()
    }
    pub fn ready(&mut self, cache: &mut DiskCache, sequence: &Sequence) -> bool {
        if let Some(id) = sequence.cached_animation {
            if self.cached.contains(&id) {
                return true;
            }
            let ready = self.cached_ready(cache, id).is_ok();
            self.known.insert((22, id), ready);
            if !ready {
                return false;
            }
            self.cached.insert(id);
            true
        } else if let Some(&frame) = sequence.frames.first() {
            let group = frame >> 16;
            if self.legacy.contains(&group) {
                return true;
            }
            let ready = self.legacy_ready(cache, group).is_ok();
            self.known.insert((0, group), ready);
            if !ready {
                return false;
            }
            self.legacy.insert(group);
            true
        } else {
            false
        }
    }
    fn legacy_ready(&self, cache: &mut DiskCache, group: u32) -> Result<()> {
        let frames = cache.group(0, group, [0; 4])?;
        let data = &frames
            .first()
            .ok_or(Error::Invalid("empty animation frame group"))?
            .1;
        let id = u16::from_be_bytes(data.get(..2).ok_or(Error::Truncated)?.try_into().unwrap());
        let skeleton = Skeleton::decode(id, &cache.file(1, id as u32, 0, [0; 4])?)?;
        for (_, data) in frames {
            Frame::decode(&data, &skeleton)?;
        }
        Ok(())
    }
    fn cached_ready(&self, cache: &mut DiskCache, id: u32) -> Result<()> {
        let data = cache.file(22, id >> 16, id & 65535, [0; 4])?;
        let skeleton_id =
            u16::from_be_bytes(data.get(1..3).ok_or(Error::Truncated)?.try_into().unwrap());
        let skeleton =
            Skeleton::decode(skeleton_id, &cache.file(1, skeleton_id as u32, 0, [0; 4])?)?;
        let animation = CachedAnimation::decode(&data, &skeleton)?;
        Rig::new(&skeleton, animation.pose)?;
        Ok(())
    }
}
