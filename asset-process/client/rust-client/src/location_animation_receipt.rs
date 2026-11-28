//! Location animation receipts for headless captures (#111).
//!
//! Not yet wired into the live client's paint, so it observes no real live
//! draw. `WorldMesh::advance` records each observation from the exact model,
//! clock and vertices it writes (prepared), and `location_animation_draw`
//! records what one draw received (submitted) and the pixels whose final
//! depth-written tag is the location's (tag pixels). Tags are per location,
//! not per part or face, and none of this replaces inspecting the image.
//! A paint/draw ledger numbers paints and draws in one serial domain, and
//! validation is strict against hand-declared expectations. A missing, pending or
//! inactive record never validates, nor does a clock that disagrees with the
//! applied frame or lies outside the declared bounds. Digests do not cover the
//! clock; it is checked separately. Declared targets are kept first when a
//! receipt is truncated, so a retained declared target can still validate; a
//! declared target absent from a truncated receipt is indeterminate.
use crate::{
    animation::Sequence,
    animation_runtime::{Applied, Clock},
};
use sha2::{Digest, Sha256};

/// Most observations one receipt keeps. Declared targets are kept first.
pub const MAX_OBSERVATIONS: usize = 256;

/// One animated location part: `key` is `[plane, x, z, category]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Target {
    pub key: [i32; 4],
    pub location_id: u32,
    pub part: usize,
}

/// [`Applied`] without the load error, which is diagnostic text only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Stopped,
    Classic { frame_index: usize, frame_id: u32, skeleton_id: u16 },
    Cached { animation_id: u32, skeleton_id: u16, frame_index: usize },
    NoFrame { frame_index: usize, frames: usize },
    FrameUnavailable { frame_index: usize, frame_id: u32 },
    CachedUnavailable { animation_id: u32 },
}
impl Outcome {
    pub fn of(applied: &Applied) -> Self {
        match *applied {
            Applied::Stopped => Self::Stopped,
            Applied::Classic { frame_index, frame_id, skeleton_id } => {
                Self::Classic { frame_index, frame_id, skeleton_id }
            }
            Applied::Cached { animation_id, skeleton_id, frame_index } => {
                Self::Cached { animation_id, skeleton_id, frame_index }
            }
            Applied::NoFrame { frame_index, frames } => Self::NoFrame { frame_index, frames },
            Applied::FrameUnavailable { frame_index, frame_id, .. } => {
                Self::FrameUnavailable { frame_index, frame_id }
            }
            Applied::CachedUnavailable { animation_id, .. } => Self::CachedUnavailable { animation_id },
        }
    }
    /// The applied frame, for the only outcomes that animated the model.
    pub fn animated_frame(&self) -> Option<usize> {
        match *self {
            Self::Classic { frame_index, .. } | Self::Cached { frame_index, .. } => Some(frame_index),
            _ => None,
        }
    }
    pub fn animated(&self) -> bool {
        self.animated_frame().is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClockState {
    pub frame: usize,
    pub elapsed: u32,
    pub loops: u32,
    pub stopped: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub target: Target,
    pub model_ids: Vec<u32>,
    pub sequence_id: u32,
    /// The sequence was absent and an empty stand-in was decoded in its place.
    pub sequence_placeholder: bool,
    pub timing_digest: [u8; 32],
    pub random_start: bool,
    pub clock: ClockState,
    /// Length of the clock's frame in a classic sequence; `None` for cached
    /// sequences or a frame index outside the length list.
    pub frame_length: Option<u32>,
    pub outcome: Outcome,
    /// Exact digest of the model-order vertices the pose produced.
    pub pose_digest: [u8; 32],
    /// Exact digest of the vertices written into each triangle slot, in slot order.
    pub slots_digest: [u8; 32],
    /// The slots' scene tag, shared by every face of this location.
    pub tag: u64,
    pub triangles: usize,
}
impl Observation {
    /// Record what was used: `applied` is the result of applying `sequence` at
    /// `clock`; `vertices` are the posed model vertices and `slots` the
    /// per-face vertices written for drawing, carrying `tag`.
    #[allow(clippy::too_many_arguments)]
    pub fn record(
        target: Target,
        model_ids: Vec<u32>,
        sequence: &Sequence,
        sequence_placeholder: bool,
        random_start: bool,
        clock: &Clock,
        applied: &Applied,
        vertices: &[[f32; 3]],
        slots: &[[[f32; 3]; 3]],
        tag: u64,
    ) -> Self {
        Self {
            target,
            model_ids,
            sequence_id: sequence.id,
            sequence_placeholder,
            timing_digest: timing_digest(sequence),
            random_start,
            clock: ClockState {
                frame: clock.frame,
                elapsed: clock.elapsed,
                loops: clock.loops,
                stopped: clock.stopped,
            },
            frame_length: sequence
                .cached_animation
                .is_none()
                .then(|| sequence.lengths.get(clock.frame).map(|&n| n as u32))
                .flatten(),
            outcome: Outcome::of(applied),
            pose_digest: pose_digest(vertices),
            slots_digest: slots_digest(slots),
            tag,
            triangles: slots.len(),
        }
    }
}

/// SHA-256 of `b"pose-f32\0"`, the vertex count (u32 LE), then each
/// coordinate's exact `f32` bit pattern (LE). Nothing is rounded, so a
/// sub-unit offset or `-0.0` in place of `0.0` changes the digest.
pub fn pose_digest(vertices: &[[f32; 3]]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"pose-f32\0");
    hash.update((vertices.len() as u32).to_le_bytes());
    for vertex in vertices {
        for coordinate in vertex {
            hash.update(coordinate.to_bits().to_le_bytes());
        }
    }
    hash.finalize().into()
}

/// As [`pose_digest`] over triangle slots (three vertices each), with its own
/// prefix `b"slots-f32\0"` and the slot count.
pub fn slots_digest(slots: &[[[f32; 3]; 3]]) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"slots-f32\0");
    hash.update((slots.len() as u32).to_le_bytes());
    for vertex in slots.iter().flatten() {
        for coordinate in vertex {
            hash.update(coordinate.to_bits().to_le_bytes());
        }
    }
    hash.finalize().into()
}

/// SHA-256 of the sequence fields that select frames over time: cached
/// animation and range, frame ids and lengths, loop offset and loop limit.
pub fn timing_digest(sequence: &Sequence) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"timing\0");
    hash.update(sequence.cached_animation.map_or(u64::MAX, u64::from).to_le_bytes());
    for value in sequence.cached_range {
        hash.update(value.to_le_bytes());
    }
    hash.update((sequence.frames.len() as u32).to_le_bytes());
    for frame in &sequence.frames {
        hash.update(frame.to_le_bytes());
    }
    hash.update((sequence.lengths.len() as u32).to_le_bytes());
    for length in &sequence.lengths {
        hash.update(length.to_le_bytes());
    }
    hash.update(sequence.loop_frames.to_le_bytes());
    hash.update([sequence.max_loops]);
    hash.finalize().into()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub observations: Vec<Observation>,
    /// Observations before truncation.
    pub total: usize,
    pub truncated: bool,
    /// Some location model was still loading when the mesh was built.
    pub pending_assets: bool,
}
impl Receipt {
    /// Declared targets first, in declaration order, then the rest by
    /// (key, part, location); at most [`MAX_OBSERVATIONS`] are kept.
    pub fn collect(mut observations: Vec<Observation>, declared: &[Target], pending_assets: bool) -> Self {
        let total = observations.len();
        observations.sort_by_key(|o| {
            let rank = declared.iter().position(|t| *t == o.target).unwrap_or(declared.len());
            (rank, o.target.key, o.target.part, o.target.location_id)
        });
        observations.truncate(MAX_OBSERVATIONS);
        Self { truncated: total > observations.len(), observations, total, pending_assets }
    }
}

/// A completed world draw: `paint_id` is the paint that completed it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawIdentity {
    pub draw_id: u64,
    pub paint_id: u64,
}
/// What one draw submitted for one observation's triangle slots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Submission {
    pub target: Target,
    pub tag: u64,
    /// Distinct slots of this observation yielded to the rasteriser.
    pub yielded_slots: usize,
    /// All yields of those slots; above `yielded_slots` when one repeats.
    pub yields: usize,
    /// [`slots_digest`] over the yielded distinct slots, in slot order.
    pub slots_digest: [u8; 32],
    /// Root-mesh triangles with this tag that are not this observation's slots.
    pub same_tag_other_slots: usize,
    /// Viewport pixels whose final depth-written tag is this one.
    pub tag_pixels: u32,
}
/// Everything one completed world draw recorded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawEvidence {
    pub receipt: Receipt,
    pub submissions: Vec<Submission>,
    pub roof_plane: u8,
    /// Presented pixels came from a renderer other than the tagged CPU pass.
    pub enhanced: bool,
    /// Two observations claimed the same triangle slot; no submission is trusted.
    pub overlapping: Option<(Target, Target)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Completed {
    pub identity: DrawIdentity,
    pub evidence: DrawEvidence,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stale {
    NoPaint,
    /// The current paint has not completed a world draw, and none is retained.
    NoCompletedDraw { paint_id: u64 },
    /// A later paint began without completing a world draw.
    PaintAdvanced { draw: DrawIdentity, paint_id: u64 },
    /// `complete` was called twice in one paint.
    AlreadyCompleted { draw: DrawIdentity },
}

/// Paint entries and completed world draws, numbered in one domain. A capture
/// is valid only when the current paint is the one that completed the draw.
#[derive(Debug, Default)]
pub struct DrawLedger {
    paint_id: u64,
    draws: u64,
    completed: Option<Completed>,
}
impl DrawLedger {
    /// Call on every paint entry, before any early return.
    pub fn begin_paint(&mut self) -> u64 {
        self.paint_id += 1;
        self.paint_id
    }
    /// Call where the world draw completes, with what that draw used.
    pub fn complete(&mut self, evidence: DrawEvidence) -> Result<DrawIdentity, Stale> {
        if self.paint_id == 0 {
            return Err(Stale::NoPaint);
        }
        if let Some(done) = self.completed.as_ref().filter(|c| c.identity.paint_id == self.paint_id) {
            return Err(Stale::AlreadyCompleted { draw: done.identity });
        }
        self.draws += 1;
        let identity = DrawIdentity { draw_id: self.draws, paint_id: self.paint_id };
        self.completed = Some(Completed { identity, evidence });
        Ok(identity)
    }
    /// Call wherever the last draw's camera/pick state is discarded.
    pub fn invalidate(&mut self) {
        self.completed = None;
    }
    pub fn capture(&self) -> Result<&Completed, Stale> {
        if self.paint_id == 0 {
            return Err(Stale::NoPaint);
        }
        match &self.completed {
            None => Err(Stale::NoCompletedDraw { paint_id: self.paint_id }),
            Some(done) if done.identity.paint_id != self.paint_id => {
                Err(Stale::PaintAdvanced { draw: done.identity, paint_id: self.paint_id })
            }
            Some(done) => Ok(done),
        }
    }
}

/// What a harness declares for one target, from hand-written literals.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expected {
    pub target: Target,
    pub model_ids: Vec<u32>,
    pub sequence_id: u32,
    pub timing_digest: [u8; 32],
    pub random_start: bool,
    pub outcome: Outcome,
    pub pose_digest: [u8; 32],
    pub triangles: usize,
    pub frame_length: Option<u32>,
    /// Capture sampling is not exact, so elapsed and loop counts are intervals.
    /// The frame comes from `outcome`, and the clock must not be stopped.
    pub elapsed: std::ops::RangeInclusive<u32>,
    pub loops: std::ops::RangeInclusive<u32>,
    /// Fewest depth-won tag pixels accepted; at least 1. Tags are per
    /// location, so this is evidence for the animated part only when the
    /// location has no other same-tag geometry (checked as `TagShared`).
    pub min_tag_pixels: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    Stale(Stale),
    /// The camera diagnostic names a different draw than the receipt.
    CameraMismatch { camera: DrawIdentity, receipt: DrawIdentity },
    NoTargets,
    /// The expectation itself declares a non-animated outcome.
    UnanimatedExpectation(Target),
    DuplicateExpectation(Target),
    Malformed { total: usize, recorded: usize, truncated: bool },
    PendingAssets,
    /// Not recorded although the receipt is complete: the part was not built.
    Absent(Target),
    /// Not recorded and the receipt was truncated: presence is unknown.
    Indeterminate(Target),
    Duplicate(Target),
    Placeholder(Target),
    NotAnimated(Target, Outcome),
    /// The recorded clock contradicts the recorded outcome or frame length.
    ClockInconsistent(Target, &'static str),
    Mismatch(Target, &'static str),
    InvalidExpectation(Target, &'static str),
    EnhancedPresentation,
    OverlappingSlots(Target, Target),
    SubmissionOrphan(Target),
    DuplicateSubmissionRecord(Target),
    SubmissionMissing(Target),
    NotSubmitted(Target),
    PartiallySubmitted { target: Target, yielded: usize, expected: usize },
    DuplicateYield { target: Target, yields: usize, slots: usize },
    /// The submitted slots differ from those prepared by the pose.
    SubmittedChanged(Target),
    /// Other root geometry carries this location's tag, so tag pixels cannot
    /// be attributed to the animated part.
    TagShared { target: Target, others: usize },
    NotVisible { target: Target, tag_pixels: u32, min: u32 },
}

/// Pass only if the capture is current, names the camera's draw, and every
/// declared target was recorded exactly once with every declared value.
pub fn validate(
    capture: Result<&Completed, Stale>,
    camera: DrawIdentity,
    expected: &[Expected],
) -> Result<DrawIdentity, Failure> {
    let completed = capture.map_err(Failure::Stale)?;
    if completed.identity != camera {
        return Err(Failure::CameraMismatch { camera, receipt: completed.identity });
    }
    if expected.is_empty() {
        return Err(Failure::NoTargets);
    }
    for (i, e) in expected.iter().enumerate() {
        if !e.outcome.animated() {
            return Err(Failure::UnanimatedExpectation(e.target));
        }
        if expected[..i].iter().any(|other| other.target == e.target) {
            return Err(Failure::DuplicateExpectation(e.target));
        }
        if e.min_tag_pixels == 0 {
            return Err(Failure::InvalidExpectation(e.target, "min_tag_pixels"));
        }
    }
    let evidence = &completed.evidence;
    let receipt = &evidence.receipt;
    let recorded = receipt.observations.len();
    let consistent = if receipt.truncated {
        recorded == MAX_OBSERVATIONS && receipt.total > recorded
    } else {
        receipt.total == recorded && recorded <= MAX_OBSERVATIONS
    };
    if !consistent {
        return Err(Failure::Malformed { total: receipt.total, recorded, truncated: receipt.truncated });
    }
    if receipt.pending_assets {
        return Err(Failure::PendingAssets);
    }
    if evidence.enhanced {
        return Err(Failure::EnhancedPresentation);
    }
    if let Some((a, b)) = evidence.overlapping {
        return Err(Failure::OverlappingSlots(a, b));
    }
    // Submissions and recorded observations must correspond one to one.
    for (i, s) in evidence.submissions.iter().enumerate() {
        if !receipt.observations.iter().any(|o| o.target == s.target) {
            return Err(Failure::SubmissionOrphan(s.target));
        }
        if evidence.submissions[..i].iter().any(|other| other.target == s.target) {
            return Err(Failure::DuplicateSubmissionRecord(s.target));
        }
    }
    if let Some(o) = receipt.observations.iter().find(|o| !evidence.submissions.iter().any(|s| s.target == o.target)) {
        return Err(Failure::SubmissionMissing(o.target));
    }
    for e in expected {
        let mut matches = receipt.observations.iter().filter(|o| o.target == e.target);
        let Some(o) = matches.next() else {
            return Err(if receipt.truncated { Failure::Indeterminate(e.target) } else { Failure::Absent(e.target) });
        };
        if matches.next().is_some() {
            return Err(Failure::Duplicate(e.target));
        }
        if o.sequence_placeholder {
            return Err(Failure::Placeholder(e.target));
        }
        let Some(applied_frame) = o.outcome.animated_frame() else {
            return Err(Failure::NotAnimated(e.target, o.outcome.clone()));
        };
        // An animated outcome comes from a running clock at that frame. A
        // classic clock never rests beyond its frame's length (Clock::advance
        // steps while elapsed > length).
        let classic = matches!(o.outcome, Outcome::Classic { .. });
        let clock_checks: [(&'static str, bool); 4] = [
            ("stopped", !o.clock.stopped),
            ("frame", o.clock.frame == applied_frame),
            ("frame_length", o.frame_length.is_some() == classic),
            ("elapsed", o.frame_length.map_or(true, |length| o.clock.elapsed <= length)),
        ];
        if let Some((field, _)) = clock_checks.into_iter().find(|(_, ok)| !ok) {
            return Err(Failure::ClockInconsistent(e.target, field));
        }
        let fields: [(&'static str, bool); 10] = [
            ("model_ids", o.model_ids == e.model_ids),
            ("sequence_id", o.sequence_id == e.sequence_id),
            ("timing_digest", o.timing_digest == e.timing_digest),
            ("random_start", o.random_start == e.random_start),
            ("outcome", o.outcome == e.outcome),
            ("pose_digest", o.pose_digest == e.pose_digest),
            ("triangles", o.triangles == e.triangles),
            ("frame_length", o.frame_length == e.frame_length),
            ("clock.elapsed", e.elapsed.contains(&o.clock.elapsed)),
            ("clock.loops", e.loops.contains(&o.clock.loops)),
        ];
        if let Some((field, _)) = fields.into_iter().find(|(_, equal)| !equal) {
            return Err(Failure::Mismatch(e.target, field));
        }
        // What the draw received, after what the pose prepared.
        let Some(s) = evidence.submissions.iter().find(|s| s.target == e.target) else {
            return Err(Failure::SubmissionMissing(e.target));
        };
        if s.tag != o.tag {
            return Err(Failure::Mismatch(e.target, "submission.tag"));
        }
        if s.yielded_slots == 0 {
            return Err(Failure::NotSubmitted(e.target));
        }
        if s.yielded_slots != o.triangles {
            return Err(Failure::PartiallySubmitted { target: e.target, yielded: s.yielded_slots, expected: o.triangles });
        }
        if s.yields != s.yielded_slots {
            return Err(Failure::DuplicateYield { target: e.target, yields: s.yields, slots: s.yielded_slots });
        }
        if s.slots_digest != o.slots_digest {
            return Err(Failure::SubmittedChanged(e.target));
        }
        if s.same_tag_other_slots != 0 {
            return Err(Failure::TagShared { target: e.target, others: s.same_tag_other_slots });
        }
        if s.tag_pixels < e.min_tag_pixels {
            return Err(Failure::NotVisible { target: e.target, tag_pixels: s.tag_pixels, min: e.min_tag_pixels });
        }
    }
    Ok(completed.identity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation_runtime::test_assets::*;
    const TARGET: Target = Target { key: [0, 3200, 3201, 2], location_id: 51000, part: 0 };
    const MODELS: [u32; 1] = [60000];
    const TAG: u64 = (51000 + 1) << 32 | 25 << 14 | 1;

    /// Integer literals as the exact `f32` values the classic path produces.
    fn f32s(vertices: &[[i32; 3]]) -> Vec<[f32; 3]> {
        vertices.iter().map(|v| v.map(|c| c as f32)).collect()
    }
    fn slots_of(m: &crate::model::Model) -> Vec<[[f32; 3]; 3]> {
        m.faces.iter().map(|f| f.map(|v| m.vertices[v as usize])).collect()
    }
    /// Declared values, from literals only.
    fn expected(frame: usize) -> Expected {
        Expected {
            target: TARGET,
            model_ids: MODELS.to_vec(),
            sequence_id: SEQUENCE,
            timing_digest: timing_digest(&sequence(LENGTHS)),
            random_start: false,
            outcome: Outcome::Classic { frame_index: frame, frame_id: FRAMES[frame], skeleton_id: SKELETON },
            pose_digest: pose_digest(&f32s(&POSES[frame])),
            triangles: 12,
            frame_length: Some(LENGTHS[frame] as u32),
            // `observe` samples elapsed 7 in loop 0.
            elapsed: 5..=10,
            loops: 0..=0,
            min_tag_pixels: 1,
        }
    }
    /// Apply through the real runtime and record what it used.
    fn observe(animations: &mut Animations, cache: &mut EmptyCache, sequence: &Sequence, frame: usize) -> Observation {
        let clock = Clock { frame, elapsed: 7, ..Default::default() };
        let mut m = model();
        let applied = animations.apply_observed(&mut cache.1, &mut m, sequence.id, &clock).unwrap();
        Observation::record(TARGET, MODELS.to_vec(), sequence, false, false, &clock, &applied, &m.vertices, &slots_of(&m), TAG)
    }
    /// A complete, visible submission of every slot exactly as prepared.
    fn drawn(o: &Observation) -> Submission {
        Submission {
            target: o.target,
            tag: o.tag,
            yielded_slots: o.triangles,
            yields: o.triangles,
            slots_digest: o.slots_digest,
            same_tag_other_slots: 0,
            tag_pixels: 10,
        }
    }
    /// One full submission per recorded target.
    fn evidence(receipt: Receipt) -> DrawEvidence {
        let mut submissions: Vec<Submission> = Vec::new();
        for o in &receipt.observations {
            if !submissions.iter().any(|s| s.target == o.target) {
                submissions.push(drawn(o));
            }
        }
        DrawEvidence { receipt, submissions, roof_plane: 0, enhanced: false, overlapping: None }
    }
    fn completed(evidence: DrawEvidence) -> DrawLedger {
        let mut ledger = DrawLedger::default();
        ledger.begin_paint();
        ledger.complete(evidence).unwrap();
        ledger
    }
    fn captured(observations: Vec<Observation>) -> DrawLedger {
        completed(evidence(Receipt::collect(observations, &[TARGET], false)))
    }
    fn check(ledger: &DrawLedger, frame: usize) -> Result<DrawIdentity, Failure> {
        let camera = ledger.capture().map(|c| c.identity).unwrap_or(DrawIdentity { draw_id: 0, paint_id: 0 });
        validate(ledger.capture(), camera, &[expected(frame)])
    }
    use crate::animation_runtime::Animations;

    #[test]
    fn declared_poses_validate_from_the_runtime_observation() {
        let mut cache = EmptyCache::new("receipt-pass");
        let s = sequence(LENGTHS);
        let mut animations = animations(s.clone(), [1, 1], PIVOT);
        for frame in 0..3 {
            let ledger = captured(vec![observe(&mut animations, &mut cache, &s, frame)]);
            assert_eq!(check(&ledger, frame), Ok(DrawIdentity { draw_id: 1, paint_id: 1 }));
            // A capture of one frame never satisfies another frame's declaration.
            let other = (frame + 1) % 3;
            assert_eq!(check(&ledger, other), Err(Failure::Mismatch(TARGET, "outcome")));
        }
    }

    #[test]
    fn tampered_pivot_label_and_duration_are_rejected() {
        let mut cache = EmptyCache::new("receipt-tamper");
        let s = sequence(LENGTHS);
        let mut pivot = animations(s.clone(), [1, 1], [16, -64, 9]);
        let ledger = captured(vec![observe(&mut pivot, &mut cache, &s, 1)]);
        assert_eq!(check(&ledger, 1), Err(Failure::Mismatch(TARGET, "pose_digest")));
        // Translation applied to the static label instead of the moving one.
        let mut label = animations(s.clone(), [1, 2], PIVOT);
        let ledger = captured(vec![observe(&mut label, &mut cache, &s, 2)]);
        assert_eq!(check(&ledger, 2), Err(Failure::Mismatch(TARGET, "pose_digest")));
        let long = sequence([50, 76, 100]);
        let mut duration = animations(long.clone(), [1, 1], PIVOT);
        let ledger = captured(vec![observe(&mut duration, &mut cache, &long, 1)]);
        assert_eq!(check(&ledger, 1), Err(Failure::Mismatch(TARGET, "timing_digest")));
        let mut swapped = sequence(LENGTHS);
        swapped.frames.swap(1, 2);
        let mut order = animations(swapped.clone(), [1, 1], PIVOT);
        let ledger = captured(vec![observe(&mut order, &mut cache, &swapped, 1)]);
        assert_eq!(check(&ledger, 1), Err(Failure::Mismatch(TARGET, "timing_digest")));
        // Every declared identity field is compared.
        let good = observe(&mut animations(s.clone(), [1, 1], PIVOT), &mut cache, &s, 0);
        for (field, tamper) in [
            ("model_ids", (|o: &mut Observation| o.model_ids = vec![60001]) as fn(&mut Observation)),
            ("sequence_id", |o| o.sequence_id += 1),
            ("random_start", |o| o.random_start = true),
            ("triangles", |o| o.triangles = 11),
        ] {
            let mut o = good.clone();
            tamper(&mut o);
            assert_eq!(check(&captured(vec![o]), 0), Err(Failure::Mismatch(TARGET, field)));
        }
    }

    #[test]
    fn missing_inactive_and_placeholder_animation_never_validates() {
        let mut cache = EmptyCache::new("receipt-missing");
        let s = sequence(LENGTHS);
        // Nothing decoded and an empty cache: the frame is unavailable.
        let mut missing = unloaded(s.clone());
        let o = observe(&mut missing, &mut cache, &s, 0);
        // The unanimated pose equals frame 0's declared pose; it still fails.
        assert_eq!(o.pose_digest, pose_digest(&f32s(&POSES[0])));
        assert_eq!(
            check(&captured(vec![o]), 0),
            Err(Failure::NotAnimated(TARGET, Outcome::FrameUnavailable { frame_index: 0, frame_id: FRAMES[0] }))
        );
        let mut animations = animations(s.clone(), [1, 1], PIVOT);
        let o = observe(&mut animations, &mut cache, &s, 3);
        assert_eq!(check(&captured(vec![o]), 0), Err(Failure::NotAnimated(TARGET, Outcome::NoFrame { frame_index: 3, frames: 3 })));
        let mut o = observe(&mut animations, &mut cache, &s, 0);
        o.outcome = Outcome::of(&Applied::Stopped);
        assert_eq!(check(&captured(vec![o.clone()]), 0), Err(Failure::NotAnimated(TARGET, Outcome::Stopped)));
        o.outcome = Outcome::CachedUnavailable { animation_id: 9 };
        assert!(matches!(check(&captured(vec![o]), 0), Err(Failure::NotAnimated(..))));
        let mut o = observe(&mut animations, &mut cache, &s, 0);
        o.sequence_placeholder = true;
        assert_eq!(check(&captured(vec![o.clone()]), 0), Err(Failure::Placeholder(TARGET)));
        o.sequence_placeholder = false;
        let ledger = completed(evidence(Receipt::collect(vec![o], &[TARGET], true)));
        assert_eq!(check(&ledger, 0), Err(Failure::PendingAssets));
        // An expectation of an unanimated pose is refused, not satisfied.
        let ledger = captured(vec![observe(&mut animations, &mut cache, &s, 0)]);
        let mut e = expected(0);
        e.outcome = Outcome::Stopped;
        assert_eq!(validate(ledger.capture(), ledger.capture().unwrap().identity, &[e]), Err(Failure::UnanimatedExpectation(TARGET)));
        assert_eq!(validate(ledger.capture(), ledger.capture().unwrap().identity, &[]), Err(Failure::NoTargets));
        assert_eq!(
            validate(ledger.capture(), ledger.capture().unwrap().identity, &[expected(0), expected(0)]),
            Err(Failure::DuplicateExpectation(TARGET))
        );
    }

    #[test]
    fn absence_duplication_and_truncation_are_exact() {
        let mut cache = EmptyCache::new("receipt-bounds");
        let s = sequence(LENGTHS);
        let mut animations = animations(s.clone(), [1, 1], PIVOT);
        let good = observe(&mut animations, &mut cache, &s, 0);
        let other = |i: usize| Observation {
            target: Target { key: [0, 3000 + i as i32, 3000, 2], location_id: 1, part: 0 },
            ..good.clone()
        };
        assert_eq!(check(&captured(vec![other(0)]), 0), Err(Failure::Absent(TARGET)));
        assert_eq!(check(&captured(vec![good.clone(), good.clone()]), 0), Err(Failure::Duplicate(TARGET)));
        // 300 other parts and the target: the declared target is kept despite truncation.
        let mut many: Vec<_> = (0..300).map(other).collect();
        many.push(good.clone());
        let receipt = Receipt::collect(many.clone(), &[TARGET], false);
        assert_eq!((receipt.total, receipt.observations.len(), receipt.truncated), (301, MAX_OBSERVATIONS, true));
        assert_eq!(receipt.observations[0].target, TARGET);
        assert!(check(&captured(many), 0).is_ok());
        // Truncated without the target: presence is unknown.
        let ledger = completed(evidence(Receipt::collect((0..300).map(other).collect(), &[], false)));
        assert_eq!(check(&ledger, 0), Err(Failure::Indeterminate(TARGET)));
        // Totals inconsistent with the recorded list are malformed.
        for (total, truncated) in [(2, false), (1, true)] {
            let ledger = completed(evidence(Receipt { observations: vec![good.clone()], total, truncated, pending_assets: false }));
            assert_eq!(check(&ledger, 0), Err(Failure::Malformed { total, recorded: 1, truncated }));
        }
        // Declared targets keep declaration order; the rest are sorted.
        let receipt = Receipt::collect(vec![other(2), other(1), good.clone()], &[TARGET], false);
        let order: Vec<_> = receipt.observations.iter().map(|o| o.target.key[1]).collect();
        assert_eq!(order, vec![TARGET.key[1], 3001, 3002]);
    }

    #[test]
    fn ledger_requires_the_current_paint_to_complete_the_draw() {
        let mut cache = EmptyCache::new("receipt-ledger");
        let s = sequence(LENGTHS);
        let mut animations = animations(s.clone(), [1, 1], PIVOT);
        let receipt = evidence(Receipt::collect(vec![observe(&mut animations, &mut cache, &s, 0)], &[TARGET], false));
        let mut ledger = DrawLedger::default();
        assert_eq!(ledger.capture().unwrap_err(), Stale::NoPaint);
        assert_eq!(ledger.complete(receipt.clone()).unwrap_err(), Stale::NoPaint);
        // Paint 1 completes draw 1.
        ledger.begin_paint();
        assert_eq!(ledger.capture().unwrap_err(), Stale::NoCompletedDraw { paint_id: 1 });
        let first = ledger.complete(receipt.clone()).unwrap();
        assert_eq!(first, DrawIdentity { draw_id: 1, paint_id: 1 });
        assert_eq!(check(&ledger, 0), Ok(first));
        assert_eq!(ledger.complete(receipt.clone()).unwrap_err(), Stale::AlreadyCompleted { draw: first });
        // Paint 2 returns early: a capture now, or a delayed retry, is stale.
        ledger.begin_paint();
        assert_eq!(ledger.capture().unwrap_err(), Stale::PaintAdvanced { draw: first, paint_id: 2 });
        assert_eq!(check(&ledger, 0), Err(Failure::Stale(Stale::PaintAdvanced { draw: first, paint_id: 2 })));
        // Paint 3 resumes: the new draw is current and numbered after the first.
        ledger.begin_paint();
        let resumed = ledger.complete(receipt.clone()).unwrap();
        assert_eq!(resumed, DrawIdentity { draw_id: 2, paint_id: 3 });
        assert_eq!(check(&ledger, 0), Ok(resumed));
        // The camera must name the same draw.
        assert_eq!(
            validate(ledger.capture(), first, &[expected(0)]),
            Err(Failure::CameraMismatch { camera: first, receipt: resumed })
        );
        // Discarding the draw state invalidates the receipt until a new draw.
        ledger.invalidate();
        assert_eq!(ledger.capture().unwrap_err(), Stale::NoCompletedDraw { paint_id: 3 });
        ledger.begin_paint();
        ledger.begin_paint();
        let later = ledger.complete(receipt).unwrap();
        assert_eq!(later, DrawIdentity { draw_id: 3, paint_id: 5 });
        assert_eq!(check(&ledger, 0), Ok(later));
    }

    #[test]
    fn clock_must_agree_with_the_applied_frame_and_declared_bounds() {
        let mut cache = EmptyCache::new("receipt-clock");
        let s = sequence(LENGTHS);
        let mut animations = animations(s.clone(), [1, 1], PIVOT);
        let good = observe(&mut animations, &mut cache, &s, 1);
        assert!(check(&captured(vec![good.clone()]), 1).is_ok());
        // The digests are unchanged by every tamper below; only the clock checks catch them.
        let inconsistent: [(&str, fn(&mut Observation)); 5] = [
            ("frame", |o| o.clock.frame = 0),
            ("frame", |o| o.clock.frame = 2),
            ("stopped", |o| o.clock.stopped = true),
            ("frame_length", |o| o.frame_length = None),
            ("elapsed", |o| o.clock.elapsed = 76),
        ];
        for (field, tamper) in inconsistent {
            let mut o = good.clone();
            tamper(&mut o);
            assert_eq!((o.pose_digest, o.timing_digest), (good.pose_digest, good.timing_digest));
            let ledger = captured(vec![o]);
            let mut wide = expected(1);
            (wide.elapsed, wide.loops) = (0..=u32::MAX, 0..=u32::MAX);
            assert_eq!(
                validate(ledger.capture(), ledger.capture().unwrap().identity, &[wide]),
                Err(Failure::ClockInconsistent(TARGET, field)),
                "{field}"
            );
        }
        let declared: [(&str, fn(&mut Observation)); 5] = [
            ("clock.elapsed", |o| o.clock.elapsed = 4),
            ("clock.elapsed", |o| o.clock.elapsed = 11),
            ("clock.elapsed", |o| o.clock.elapsed = 75),
            ("clock.loops", |o| o.clock.loops = 1),
            ("frame_length", |o| o.frame_length = Some(80)),
        ];
        for (field, tamper) in declared {
            let mut o = good.clone();
            tamper(&mut o);
            assert_eq!(check(&captured(vec![o]), 1), Err(Failure::Mismatch(TARGET, field)), "{field}");
        }
        // A cached outcome has no classic frame length.
        let mut o = good.clone();
        o.outcome = Outcome::Cached { animation_id: 9, skeleton_id: SKELETON, frame_index: 1 };
        assert_eq!(check(&captured(vec![o]), 1), Err(Failure::ClockInconsistent(TARGET, "frame_length")));
    }

    #[test]
    fn real_clock_boundaries_validate_only_their_declared_frame() {
        let mut cache = EmptyCache::new("receipt-boundaries");
        let s = sequence(LENGTHS);
        let mut animations = animations(s.clone(), [1, 1], PIVOT);
        // Lengths 50/75/100; Clock::advance steps while elapsed > length.
        for (ticks, frame, elapsed, loops) in
            [(0, 0, 0, 0), (50, 0, 50, 0), (51, 1, 1, 0), (125, 1, 75, 0), (126, 2, 1, 0), (225, 2, 100, 0), (226, 0, 1, 1)]
        {
            let mut clock = Clock::default();
            clock.advance(&s, ticks).unwrap();
            assert_eq!((clock.frame, clock.elapsed, clock.loops), (frame, elapsed, loops), "ticks {ticks}");
            let mut m = model();
            let applied = animations.apply_observed(&mut cache.1, &mut m, SEQUENCE, &clock).unwrap();
            let o = Observation::record(TARGET, MODELS.to_vec(), &s, false, false, &clock, &applied, &m.vertices, &slots_of(&m), TAG);
            let ledger = captured(vec![o]);
            let mut e = expected(frame);
            e.elapsed = elapsed..=elapsed;
            let result = validate(ledger.capture(), ledger.capture().unwrap().identity, &[e]);
            // After wrapping, loop 1 is outside the declared single pass.
            let want = if loops == 0 { Ok(DrawIdentity { draw_id: 1, paint_id: 1 }) } else { Err(Failure::Mismatch(TARGET, "clock.loops")) };
            assert_eq!(result, want, "ticks {ticks}");
        }
    }

    #[test]
    fn digests_are_exact_stable_and_sensitive() {
        let pose = f32s(&POSES[0]);
        assert_ne!(pose_digest(&pose), pose_digest(&f32s(&POSES[1])));
        assert_ne!(pose_digest(&pose[..7]), pose_digest(&pose));
        assert_ne!(pose_digest(&[]), pose_digest(&[[0.0, 0.0, 0.0]]));
        // Exact bit patterns: no truncation hides sub-unit or signed-zero changes.
        for (a, b) in [([16.0, 0.0, 8.0], [16.25, 0.0, 8.0]), ([0.0, 0.0, 0.0], [-0.0, 0.0, 0.0]), ([1.0, -1.0, 2.5], [1.0, -1.0, 2.500001])] {
            assert_ne!(pose_digest(&[a]), pose_digest(&[b]), "{a:?} {b:?}");
            assert_ne!(slots_digest(&[[a; 3]]), slots_digest(&[[b; 3]]), "{a:?} {b:?}");
        }
        // Slot and pose digests use distinct prefixes over the same coordinates.
        assert_ne!(slots_digest(&[[[1.0, 2.0, 3.0]; 3]]), pose_digest(&[[1.0, 2.0, 3.0]; 3]));
        let mut looped = sequence(LENGTHS);
        looped.loop_frames = 1;
        assert_ne!(timing_digest(&looped), timing_digest(&sequence(LENGTHS)));
        let mut cached = sequence(LENGTHS);
        cached.cached_animation = Some(0);
        assert_ne!(timing_digest(&cached), timing_digest(&sequence(LENGTHS)));
        // The encodings are fixed: SHA-256(prefix ++ 0u32 LE) for empty input.
        for (prefix, digest) in [(&b"pose-f32\0"[..], pose_digest(&[])), (&b"slots-f32\0"[..], slots_digest(&[]))] {
            let mut hash = Sha256::new();
            hash.update(prefix);
            hash.update(0u32.to_le_bytes());
            let empty: [u8; 32] = hash.finalize().into();
            assert_eq!(digest, empty);
        }
    }

    #[test]
    fn sub_unit_prepared_pose_change_is_rejected() {
        let mut cache = EmptyCache::new("receipt-subunit");
        let s = sequence(LENGTHS);
        let mut animations = animations(s.clone(), [1, 1], PIVOT);
        let clock = Clock { frame: 1, elapsed: 7, ..Default::default() };
        let mut m = model();
        let applied = animations.apply_observed(&mut cache.1, &mut m, SEQUENCE, &clock).unwrap();
        m.vertices[0][0] += 0.25;
        let o = Observation::record(TARGET, MODELS.to_vec(), &s, false, false, &clock, &applied, &m.vertices, &slots_of(&m), TAG);
        assert_eq!(check(&captured(vec![o]), 1), Err(Failure::Mismatch(TARGET, "pose_digest")));
    }

    #[test]
    fn submission_evidence_fails_closed() {
        let mut cache = EmptyCache::new("receipt-submission");
        let s = sequence(LENGTHS);
        let mut animations = animations(s.clone(), [1, 1], PIVOT);
        let o = observe(&mut animations, &mut cache, &s, 0);
        let base = evidence(Receipt::collect(vec![o.clone()], &[TARGET], false));
        let run = |evidence: DrawEvidence, e: Expected| {
            let ledger = completed(evidence);
            validate(ledger.capture(), ledger.capture().unwrap().identity, &[e])
        };
        assert!(run(base.clone(), expected(0)).is_ok());
        let other = Target { key: [0, 1, 1, 2], location_id: 7, part: 0 };
        let mut orphan = base.clone();
        orphan.submissions.push(Submission { target: other, ..drawn(&o) });
        assert_eq!(run(orphan, expected(0)), Err(Failure::SubmissionOrphan(other)));
        let mut twice = base.clone();
        twice.submissions.push(drawn(&o));
        assert_eq!(run(twice, expected(0)), Err(Failure::DuplicateSubmissionRecord(TARGET)));
        let mut missing = base.clone();
        missing.submissions.clear();
        assert_eq!(run(missing, expected(0)), Err(Failure::SubmissionMissing(TARGET)));
        let mut overlapping = base.clone();
        overlapping.overlapping = Some((TARGET, other));
        assert_eq!(run(overlapping, expected(0)), Err(Failure::OverlappingSlots(TARGET, other)));
        let mut enhanced = base.clone();
        enhanced.enhanced = true;
        assert_eq!(run(enhanced, expected(0)), Err(Failure::EnhancedPresentation));
        let cases: [(fn(&mut Submission), Failure); 8] = [
            (|s| s.tag ^= 1, Failure::Mismatch(TARGET, "submission.tag")),
            (|s| (s.yielded_slots, s.yields) = (0, 0), Failure::NotSubmitted(TARGET)),
            (|s| (s.yielded_slots, s.yields) = (11, 11), Failure::PartiallySubmitted { target: TARGET, yielded: 11, expected: 12 }),
            (|s| s.yields = 13, Failure::DuplicateYield { target: TARGET, yields: 13, slots: 12 }),
            (|s| s.slots_digest[0] ^= 1, Failure::SubmittedChanged(TARGET)),
            (|s| s.same_tag_other_slots = 1, Failure::TagShared { target: TARGET, others: 1 }),
            (|s| s.tag_pixels = 0, Failure::NotVisible { target: TARGET, tag_pixels: 0, min: 1 }),
            (|s| s.tag_pixels = 1, Failure::NotVisible { target: TARGET, tag_pixels: 1, min: 2 }),
        ];
        for (i, (tamper, failure)) in cases.into_iter().enumerate() {
            let mut evidence = base.clone();
            tamper(&mut evidence.submissions[0]);
            let mut e = expected(0);
            if i == 7 {
                e.min_tag_pixels = 2;
            }
            assert_eq!(run(evidence, e), Err(failure), "case {i}");
        }
        let mut e = expected(0);
        e.min_tag_pixels = 0;
        assert_eq!(run(base, e), Err(Failure::InvalidExpectation(TARGET, "min_tag_pixels")));
    }
}
