//! Sequence clocks and cached native animation assets.
use crate::{
    animation::{Frame, Sequence, Skeleton},
    cached_animation::CachedAnimation,
    disk_cache::DiskCache,
    model::Model,
    skeletal::{Matrix, Rig},
    Error, Result,
};
use std::{collections::HashMap, sync::Arc};
/// What [`Animations::apply_observed`] did to the model. The two unavailable
/// outcomes and `NoFrame` leave the model in its unanimated pose.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Applied {
    Stopped,
    Classic { frame_index: usize, frame_id: u32, skeleton_id: u16 },
    Cached { animation_id: u32, skeleton_id: u16, frame_index: usize },
    /// The clock's frame index is outside the sequence's frame list.
    NoFrame { frame_index: usize, frames: usize },
    FrameUnavailable { frame_index: usize, frame_id: u32, error: Error },
    CachedUnavailable { animation_id: u32, error: Error },
}
#[derive(Clone, Default, Debug)]
pub struct Clock {
    pub frame: usize,
    pub elapsed: u32,
    pub loops: u32,
    pub stopped: bool,
}
impl Clock {
    /// qp.ad chooses a frame uniformly, then an elapsed tick within that frame.
    pub fn randomize(&mut self, sequence: &Sequence, draws: [f64; 2]) -> Result<()> {
        if draws
            .iter()
            .any(|n| !n.is_finite() || !(0.0..1.0).contains(n))
        {
            return Err(Error::Invalid("animation random draw"));
        }
        if sequence.cached_animation.is_some() {
            let count = sequence.cached_range[1]
                .checked_sub(sequence.cached_range[0])
                .ok_or(Error::Invalid("cached sequence range"))?;
            self.frame = (draws[0] * count as f64) as usize;
        } else {
            self.frame = (draws[0] * sequence.frames.len() as f64) as usize;
            let length = *sequence
                .lengths
                .get(self.frame)
                .ok_or(Error::Invalid("empty randomized animation"))?;
            self.elapsed = (draws[1] * length as f64) as u32;
        }
        Ok(())
    }
    pub fn advance(&mut self, sequence: &Sequence, mut ticks: u32) -> Result<(u8, Vec<usize>)> {
        if self.stopped {
            return Ok((0, Vec::new()));
        }
        let mut flags = if sequence.loop_frames == -1 { 8 } else { 0 };
        let mut events = Vec::new();
        if sequence.cached_animation.is_some() {
            let count = sequence.cached_range[1]
                .checked_sub(sequence.cached_range[0])
                .ok_or(Error::Invalid("cached sequence range"))? as usize;
            if count == 0 {
                self.stopped = true;
                return Ok((flags | 2, events));
            }
            if sequence.loop_frames > 0 && ticks > 0 {
                ticks -= (ticks - 1) / sequence.loop_frames as u32 * sequence.loop_frames as u32;
            }
            if ticks > 1_000_000 {
                return Err(Error::Invalid("animation tick limit"));
            }
            for _ in 0..ticks {
                self.frame += 1;
                flags |= 4;
                if flags & 2 == 0 {
                    events.push(self.frame);
                }
                if self.frame >= count {
                    self.loops += 1;
                    flags |= 1;
                    let next = self.frame as i64 - sequence.loop_frames as i64;
                    if self.loops >= sequence.max_loops as u32 {
                        flags |= 2;
                    }
                    if next < 0 || next >= count as i64 {
                        flags |= 2;
                        self.frame = 0;
                    } else {
                        self.frame = next as usize;
                    }
                    if flags & 2 == 0 {
                        events.push(self.frame);
                    }
                }
            }
        } else {
            if sequence.frames.is_empty() {
                self.stopped = true;
                return Ok((flags | 2, events));
            }
            if sequence.loop_frames > 0 && sequence.total_length > 0 {
                ticks %= sequence.total_length;
            }
            if self.frame >= sequence.frames.len() {
                self.frame = 0;
                self.elapsed = 0;
            }
            self.elapsed = self
                .elapsed
                .checked_add(ticks)
                .ok_or(Error::Invalid("animation elapsed overflow"))?;
            let mut transitions = 0;
            while self.elapsed > sequence.lengths[self.frame] as u32 {
                self.elapsed -= sequence.lengths[self.frame] as u32;
                self.frame += 1;
                flags |= 4;
                if flags & 2 == 0 {
                    events.push(self.frame);
                }
                if self.frame >= sequence.frames.len() {
                    self.loops += 1;
                    flags |= 1;
                    let next = self.frame as i64 - sequence.loop_frames as i64;
                    if self.loops >= sequence.max_loops as u32 {
                        flags |= 2;
                    }
                    if next < 0 || next >= sequence.frames.len() as i64 {
                        flags |= 2;
                        self.frame = 0;
                    } else {
                        self.frame = next as usize;
                    }
                    if flags & 2 == 0 {
                        events.push(self.frame);
                    }
                }
                transitions += 1;
                if transitions > 1_000_000 {
                    return Err(Error::Invalid("nonprogressing animation"));
                }
            }
        }
        Ok((flags, events))
    }
}
pub struct Animations {
    pub sequences: HashMap<u32, Sequence>,
    skeletons: HashMap<u16, Arc<Skeleton>>,
    frames: HashMap<u32, Arc<Frame>>,
    cached: HashMap<u32, (Arc<CachedAnimation>, Arc<Rig>)>,
    local_pose: Vec<Option<Matrix>>,
}
impl Animations {
    pub fn apply_interface(
        &mut self,
        cache: &mut DiskCache,
        model: &mut Model,
        sequence: u32,
        clock: &Clock,
    ) -> Result<()> {
        let seq = self
            .sequences
            .get(&sequence)
            .ok_or(Error::Invalid("interface animation sequence"))?;
        let primary=seq.frames.get(clock.frame).copied();
        let legacy=seq.cached_animation.is_none();
        let chat=legacy.then(||seq.chat_frames.get(clock.frame).copied()).flatten().filter(|id|id&65535!=65535);
        // Interface animation requires its primary frame before applying an optional chat frame.
        if legacy && primary.map_or(true,|id|self.frame(cache,id).is_err()){return Ok(());}
        self.apply(cache,model,sequence,clock)?;
        if let Some(id)=chat {
            if let Ok(frame)=self.frame(cache,id) {
                let skeleton=self.skeleton(cache,frame.skeleton)?;
                model.animate_classic(&skeleton,&frame,false)?;
            }
        }
        Ok(())
    }
    pub fn load(cache: &mut DiskCache) -> Result<Self> {
        Ok(Self {
            sequences: Sequence::load_all(cache)?,
            skeletons: HashMap::new(),
            frames: HashMap::new(),
            cached: HashMap::new(),
            local_pose: Vec::new(),
        })
    }
    fn skeleton(&mut self, cache: &mut DiskCache, id: u16) -> Result<Arc<Skeleton>> {
        if let Some(s) = self.skeletons.get(&id) {
            return Ok(s.clone());
        }
        let s = Arc::new(Skeleton::decode(id, &cache.file(1, id as u32, 0, [0; 4])?)?);
        self.skeletons.insert(id, s.clone());
        Ok(s)
    }
    pub fn apply_actor(
        &mut self,
        cache: &mut DiskCache,
        model: &mut Model,
        action: Option<&crate::actor_sequence::State>,
        pose: Option<&crate::actor_sequence::State>,
        idle: i32,
    ) -> Result<()> {
        let pose = pose.filter(|p| action.is_none() || p.id != idle);
        match (action, pose) {
            (Some(a), Some(p)) => self.apply_blended(
                cache,
                model,
                a.id as u32,
                &a.clock(),
                p.id as u32,
                &p.clock(),
            ),
            (Some(a), None) | (None, Some(a)) => self.apply(cache, model, a.id as u32, &a.clock()),
            _ => Ok(()),
        }
    }
    fn frame(&mut self, cache: &mut DiskCache, id: u32) -> Result<Arc<Frame>> {
        if let Some(frame) = self.frames.get(&id) {
            return Ok(frame.clone());
        }
        let files=cache.group(0,id>>16,[0;4])?;
        let data=&files.first().ok_or(Error::Invalid("empty frame group"))?.1;
        let sid=u16::from_be_bytes(data.get(..2).ok_or(Error::Truncated)?.try_into().unwrap());
        let skeleton=self.skeleton(cache,sid)?;
        // cs.ae / fg validates and caches the entire frame group atomically.
        let decoded=files.into_iter().map(|(file,data)| Ok(((id&0xffff0000)|file,Arc::new(Frame::decode(&data,&skeleton)?))))
            .collect::<Result<Vec<_>>>()?;
        let frame=decoded.iter().find(|(key,_)|*key==id).map(|(_,frame)|frame.clone()).ok_or(Error::Invalid("animation frame missing"))?;
        if self.frames.len().saturating_add(decoded.len())>4096 {self.frames.clear();}
        self.frames.extend(decoded);
        Ok(frame)
    }
    fn cached_asset(
        &mut self,
        cache: &mut DiskCache,
        id: u32,
    ) -> Result<(Arc<CachedAnimation>, Arc<Rig>)> {
        if let Some(pair) = self.cached.get(&id) {
            return Ok(pair.clone());
        }
        let data = cache.file(22, id >> 16, id & 65535, [0; 4])?;
        let sid = u16::from_be_bytes(data.get(1..3).ok_or(Error::Truncated)?.try_into().unwrap());
        let skeleton = self.skeleton(cache, sid)?;
        let animation = Arc::new(CachedAnimation::decode(&data, &skeleton)?);
        let rig = Arc::new(Rig::new(&skeleton, animation.pose)?);
        if self.cached.len() >= 16 {
            self.cached.clear();
        }
        self.cached.insert(id, (animation.clone(), rig.clone()));
        Ok((animation, rig))
    }
    fn sequence_ready(&mut self,cache:&mut DiskCache,id:i32)->bool {
        let Some(sequence)=self.sequences.get(&(id as u32)) else{return false;};
        if let Some(asset)=sequence.cached_animation {self.cached_asset(cache,asset).is_ok()}
        else if let Some(&frame)=sequence.frames.first(){self.frame(cache,frame).is_ok()}
        else{false}
    }
    pub fn world_entity_matrix(
        &mut self,
        cache: &mut DiskCache,
        entity: &crate::world_entity::Entity,
    ) -> Result<Option<Matrix>> {
        let Some(state)=entity.scene_animation(|id|self.sequence_ready(cache,id)) else{return Ok(None);};
        if state.id < 0 {
            return Ok(None);
        }
        let seq = self
            .sequences
            .get(&(state.id as u32))
            .ok_or(Error::Invalid("world entity render sequence"))?;
        let Some(id) = seq.cached_animation else {
            return Ok(None);
        };
        let (animation, rig) = self.cached_asset(cache, id)?;
        let skeleton = self.skeleton(cache, animation.skeleton)?;
        rig.scene_matrix(
            &skeleton,
            &animation,
            state.frame as f32 + entity.animation_fraction,
        )
    }
    fn cached_pass(
        &mut self,
        model: &mut Model,
        skeleton: &Skeleton,
        animation: &CachedAnimation,
        rig: &Rig,
        time: f32,
        mask: Option<&[bool; 256]>,
        selected: bool,
        skin: bool,
    ) -> Result<()> {
        let local = rig.local_matrices(skeleton, animation, time)?;
        self.local_pose
            .resize(self.local_pose.len().max(local.len()), None);
        for (i, m) in local.into_iter().enumerate() {
            if mask.map_or(true, |mask| mask.get(i).copied() == Some(selected)) {
                self.local_pose[i] = Some(m);
            }
        }
        if skin {
            let local = self.local_pose[..skeleton.bones.len()]
                .iter()
                .map(|m| m.ok_or(Error::Invalid("uninitialized masked bone")))
                .collect::<Result<Vec<_>>>()?;
            model.skin(&rig.from_local(skeleton, &local)?);
        }
        if !selected {
            model.animate_cached_alpha(skeleton, animation, time)?;
        }
        Ok(())
    }
    pub fn apply(
        &mut self,
        cache: &mut DiskCache,
        model: &mut Model,
        sequence: u32,
        clock: &Clock,
    ) -> Result<()> {
        self.apply_observed(cache, model, sequence, clock).map(drop)
    }
    /// As [`Self::apply`], also reporting what was applied. The unavailable
    /// outcomes leave `model` untouched, exactly as `apply` always has.
    pub fn apply_observed(
        &mut self,
        cache: &mut DiskCache,
        model: &mut Model,
        sequence: u32,
        clock: &Clock,
    ) -> Result<Applied> {
        if clock.stopped {
            return Ok(Applied::Stopped);
        }
        let seq = self
            .sequences
            .get(&sequence)
            .ok_or(Error::Invalid("animation sequence missing"))?;
        if let Some(id) = seq.cached_animation {
            let (animation, rig) = match self.cached_asset(cache, id) {
                Ok(pair) => pair,
                Err(error) => return Ok(Applied::CachedUnavailable { animation_id: id, error }),
            };
            let skeleton = self.skeleton(cache, animation.skeleton)?;
            self.cached_pass(
                model,
                &skeleton,
                &animation,
                &rig,
                clock.frame as f32,
                None,
                false,
                true,
            )?;
            Ok(Applied::Cached { animation_id: id, skeleton_id: skeleton.id, frame_index: clock.frame })
        } else if let Some(&id) = seq.frames.get(clock.frame) {
            let frame = match self.frame(cache, id) {
                Ok(frame) => frame,
                Err(error) => return Ok(Applied::FrameUnavailable { frame_index: clock.frame, frame_id: id, error }),
            };
            let skeleton = self.skeleton(cache, frame.skeleton)?;
            model.animate_classic(&skeleton, &frame, false)?;
            Ok(Applied::Classic { frame_index: clock.frame, frame_id: id, skeleton_id: skeleton.id })
        } else {
            Ok(Applied::NoFrame { frame_index: clock.frame, frames: seq.frames.len() })
        }
    }
    #[cfg(test)]
    pub(crate) fn preload_classic(&mut self, skeleton: Skeleton, frames: Vec<(u32, Frame)>) {
        self.skeletons.insert(skeleton.id, Arc::new(skeleton));
        self.frames.extend(frames.into_iter().map(|(id, frame)| (id, Arc::new(frame))));
    }
    /// GameAnimation.ao: masked action/locomotion composition, including mixed formats.
    pub fn apply_blended(
        &mut self,
        cache: &mut DiskCache,
        model: &mut Model,
        primary: u32,
        first: &Clock,
        secondary: u32,
        second: &Clock,
    ) -> Result<()> {
        if first.stopped {
            return self.apply(cache, model, secondary, second);
        }
        if second.stopped {
            return self.apply(cache, model, primary, first);
        }
        let a = self
            .sequences
            .get(&primary)
            .ok_or(Error::Invalid("primary blend sequence"))?
            .clone();
        let b = self
            .sequences
            .get(&secondary)
            .ok_or(Error::Invalid("secondary blend sequence"))?
            .clone();
        // A missing cached primary returns the base copy; a missing legacy primary uses the secondary.
        if let Some(id)=a.cached_animation {
            if self.cached_asset(cache,id).is_err(){return Ok(());}
        }else if let Some(&id)=a.frames.get(first.frame) {
            if self.frame(cache,id).is_err(){return self.apply(cache,model,secondary,second);}
        }else{return self.apply(cache,model,secondary,second);}
        if (a.cached_animation.is_some() && b.cached_animation.is_some() && a.bone_mask.is_none())
            || (a.cached_animation.is_none() && a.interleave.is_none())
        {
            return self.apply(cache, model, primary, first);
        }
        // Check legacy-secondary fallback before any masked mutation of the model.
        if b.cached_animation.is_none() {
            if let Some(&id)=b.frames.get(second.frame) {
                if self.frame(cache,id).is_err(){return self.apply(cache,model,primary,first);}
            }
        }
        let mut hierarchy = None;
        let mut deferred = None;
        if let Some(id) = a.cached_animation {
            let (animation, rig) = self.cached_asset(cache, id)?;
            let skeleton = self.skeleton(cache, animation.skeleton)?;
            self.cached_pass(
                model,
                &skeleton,
                &animation,
                &rig,
                first.frame as f32,
                a.bone_mask.as_ref(),
                false,
                b.cached_animation.is_none(),
            )?;
            hierarchy = Some(skeleton);
        } else {
            let Some(&id) = a.frames.get(first.frame) else {
                return self.apply(cache, model, secondary, second);
            };
            let frame = self.frame(cache, id)?;
            let skeleton = self.skeleton(cache, frame.skeleton)?;
            if b.cached_animation.is_some() {
                deferred = Some((skeleton, frame));
            } else {
                model.animate_classic_mask(
                    &skeleton,
                    &frame,
                    a.interleave.as_deref(),
                    false,
                    false,
                )?;
            }
        }
        if let Some(id) = b.cached_animation {
            let Ok((animation, rig)) = self.cached_asset(cache, id) else {return Ok(());};
            let skeleton = match hierarchy {
                Some(s) => s,
                None => self.skeleton(cache, animation.skeleton)?,
            };
            let rig = if skeleton.id == animation.skeleton {
                rig
            } else {
                Arc::new(Rig::new(&skeleton, animation.pose)?)
            };
            self.cached_pass(
                model,
                &skeleton,
                &animation,
                &rig,
                second.frame as f32,
                a.bone_mask.as_ref(),
                true,
                true,
            )?;
        } else if let Some(&id) = b.frames.get(second.frame) {
            let frame = self.frame(cache, id)?;
            let skeleton = self.skeleton(cache, frame.skeleton)?;
            model.animate_classic_mask(&skeleton, &frame, a.interleave.as_deref(), true, false)?;
        }
        if let Some((skeleton, frame)) = deferred {
            model.animate_classic_mask(&skeleton, &frame, a.interleave.as_deref(), false, false)?;
        }
        Ok(())
    }
}
#[cfg(test)]
pub(crate) mod test_assets {
    //! Hand-declared classic assets. Expected poses are literals, never derived
    //! from the code under test.
    use super::*;
    use crate::animation::Transform;
    pub const SEQUENCE: u32 = 4000;
    pub const SKELETON: u16 = 77;
    pub const FRAMES: [u32; 3] = [0x0005_0000, 0x0005_0001, 0x0005_0002];
    pub const LENGTHS: [u16; 3] = [50, 75, 100];
    pub const PIVOT: [i16; 3] = [16, -64, 8];
    /// Vertices 0..3 carry skin label 1 and move; 4..7 carry label 2 and do not.
    pub const BASE: [[i32; 3]; 8] = [
        [0, 0, 0], [32, 0, 0], [0, -48, 0], [0, 0, 24],
        [64, 0, 64], [96, -16, 64], [64, -80, 96], [80, -32, 128],
    ];
    /// Frame 0 is identity, frame 1 doubles label 1 about PIVOT, frame 2 also
    /// translates it by (8,0,-16): v' = 2v - PIVOT (+ translation).
    pub const POSES: [[[i32; 3]; 8]; 3] = [
        BASE,
        [[-16, 64, -8], [48, 64, -8], [-16, -32, -8], [-16, 64, 40],
         [64, 0, 64], [96, -16, 64], [64, -80, 96], [80, -32, 128]],
        [[-8, 64, -24], [56, 64, -24], [-8, -32, -24], [-8, 64, 24],
         [64, 0, 64], [96, -16, 64], [64, -80, 96], [80, -32, 128]],
    ];
    pub fn model() -> Model {
        let faces = vec![
            [0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2], [4, 5, 6], [4, 6, 7],
            [4, 7, 5], [5, 7, 6], [0, 4, 1], [1, 4, 5], [2, 6, 3], [3, 6, 7],
        ];
        Model {
            format: crate::model::Format::Legacy,
            vertices: BASE.iter().map(|v| v.map(|c| c as f32)).collect(),
            colors: vec![43051; faces.len()],
            faces,
            render_types: None,
            priorities: None,
            priority: 0,
            alpha: None,
            vertex_skins: Some(vec![1, 1, 1, 1, 2, 2, 2, 2]),
            face_skins: None,
            bone_weights: None,
            face_textures: None,
            texture_coords: None,
            texture_types: Vec::new(),
            texture_triangles: Vec::new(),
            face_flags: None,
            metadata: None,
        }
    }
    /// `labels` are the skin labels of the scale and translation transforms.
    pub fn skeleton(labels: [u8; 2]) -> Skeleton {
        Skeleton {
            id: SKELETON,
            transform_types: vec![0, 3, 1],
            labels: vec![vec![], vec![labels[0]], vec![labels[1]]],
            bones: Vec::new(),
            poses: 0,
        }
    }
    pub fn frame(pivot: [i16; 3], scale: i16, translation: [i16; 3]) -> Frame {
        let t = |label, values| Transform { label, values };
        Frame {
            skeleton: SKELETON,
            transforms: vec![t(0, pivot), t(1, [scale; 3]), t(2, translation)],
            has_alpha: false,
        }
    }
    pub fn frames(pivot: [i16; 3]) -> Vec<(u32, Frame)> {
        vec![
            (FRAMES[0], frame(pivot, 128, [0; 3])),
            (FRAMES[1], frame(pivot, 256, [0; 3])),
            (FRAMES[2], frame(pivot, 256, [8, 0, -16])),
        ]
    }
    pub fn sequence(lengths: [u16; 3]) -> Sequence {
        let mut s = Sequence::decode(SEQUENCE, &[0]).unwrap();
        s.frames = FRAMES.to_vec();
        s.lengths = lengths.to_vec();
        s
    }
    /// Animations holding `sequence` and no decoded skeletons or frames.
    pub fn unloaded(sequence: Sequence) -> Animations {
        Animations {
            sequences: HashMap::from([(sequence.id, sequence)]),
            skeletons: HashMap::new(),
            frames: HashMap::new(),
            cached: HashMap::new(),
            local_pose: Vec::new(),
        }
    }
    /// `unloaded` with the declared skeleton and frames already decoded.
    pub fn animations(sequence: Sequence, labels: [u8; 2], pivot: [i16; 3]) -> Animations {
        let mut a = unloaded(sequence);
        a.preload_classic(skeleton(labels), frames(pivot));
        a
    }
    /// A cache with an empty data file: every archive read fails.
    pub struct EmptyCache(pub std::path::PathBuf, pub DiskCache);
    impl EmptyCache {
        pub fn new(name: &str) -> Self {
            let unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!("rust239-{name}-{}-{unique}", std::process::id()));
            std::fs::create_dir_all(&root).unwrap();
            std::fs::write(root.join("main_file_cache.dat2"), []).unwrap();
            let cache = DiskCache::open(&root, 1 << 20).unwrap();
            Self(root, cache)
        }
    }
    impl Drop for EmptyCache {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    pub fn vertices(model: &Model) -> Vec<[i32; 3]> {
        model.vertices.iter().map(|v| v.map(|c| c as i32)).collect()
    }
}
#[cfg(test)]
mod tests {
    use super::{test_assets::*, *};
    fn clock(frame: usize) -> Clock {
        Clock { frame, ..Default::default() }
    }
    /// `apply` and `apply_observed` mutate identically and agree on errors.
    fn same(animations: &mut Animations, cache: &mut DiskCache, sequence: u32, clock: &Clock) -> Result<Applied> {
        let (mut a, mut b) = (model(), model());
        let plain = animations.apply(cache, &mut a, sequence, clock);
        let observed = animations.apply_observed(cache, &mut b, sequence, clock);
        match (&plain, &observed) {
            (Ok(()), Ok(_)) => {}
            (Err(p), Err(o)) => assert_eq!(p, o),
            _ => panic!("apply {plain:?} disagrees with apply_observed {observed:?}"),
        }
        assert_eq!(vertices(&a), vertices(&b));
        observed
    }
    #[test]
    fn observed_classic_frames_match_declared_poses_and_plain_apply() {
        let mut cache = EmptyCache::new("apply-classic");
        let mut animations = animations(sequence(LENGTHS), [1, 1], PIVOT);
        for (index, pose) in POSES.iter().enumerate() {
            let mut m = model();
            let applied = animations.apply_observed(&mut cache.1, &mut m, SEQUENCE, &clock(index)).unwrap();
            assert_eq!(applied, Applied::Classic { frame_index: index, frame_id: FRAMES[index], skeleton_id: SKELETON });
            assert_eq!(vertices(&m), pose.to_vec(), "frame {index}");
            same(&mut animations, &mut cache.1, SEQUENCE, &clock(index)).unwrap();
        }
    }
    #[test]
    fn unavailable_and_inactive_outcomes_keep_the_unanimated_model() {
        let mut cache = EmptyCache::new("apply-unavailable");
        // Nothing decoded: the empty cache cannot supply frame 0.
        let mut animations = unloaded(sequence(LENGTHS));
        let applied = same(&mut animations, &mut cache.1, SEQUENCE, &clock(0)).unwrap();
        assert!(
            matches!(applied, Applied::FrameUnavailable { frame_index: 0, frame_id, .. } if frame_id == FRAMES[0]),
            "{applied:?}"
        );
        let mut cached = sequence(LENGTHS);
        cached.cached_animation = Some(0x0009_0003);
        animations.sequences.insert(SEQUENCE, cached);
        let applied = same(&mut animations, &mut cache.1, SEQUENCE, &clock(0)).unwrap();
        assert!(matches!(applied, Applied::CachedUnavailable { animation_id: 0x0009_0003, .. }), "{applied:?}");
        animations.sequences.insert(SEQUENCE, sequence(LENGTHS));
        let applied = same(&mut animations, &mut cache.1, SEQUENCE, &clock(3)).unwrap();
        assert_eq!(applied, Applied::NoFrame { frame_index: 3, frames: 3 });
        let stopped = Clock { stopped: true, ..clock(1) };
        assert_eq!(same(&mut animations, &mut cache.1, SEQUENCE, &stopped).unwrap(), Applied::Stopped);
        for outcome in [clock(0), clock(3), stopped] {
            let mut m = model();
            animations.apply(&mut cache.1, &mut m, SEQUENCE, &outcome).unwrap();
            assert_eq!(vertices(&m), BASE.to_vec());
        }
    }
    #[test]
    fn missing_sequence_and_skeleton_mismatch_still_fail_both_ways() {
        let mut cache = EmptyCache::new("apply-errors");
        let mut animations = animations(sequence(LENGTHS), [1, 1], PIVOT);
        let error = same(&mut animations, &mut cache.1, SEQUENCE + 1, &clock(0)).unwrap_err();
        assert_eq!(error, Error::Invalid("animation sequence missing"));
        let mut wrong = frame(PIVOT, 128, [0; 3]);
        wrong.skeleton = SKELETON + 1;
        animations.preload_classic(skeleton([1, 1]), vec![(FRAMES[0], wrong)]);
        // The frame names a skeleton the empty cache cannot load: an error, not an outcome.
        assert!(same(&mut animations, &mut cache.1, SEQUENCE, &clock(0)).is_err());
    }
}
