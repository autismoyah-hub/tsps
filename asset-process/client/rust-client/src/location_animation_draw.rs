//! What one root world draw submitted for each animated location (#111).
//!
//! [`Tally`] wraps exactly the root-mesh iterator handed to `Frame::draw_iter`
//! and attributes each yielded triangle to an animated location's slot by
//! address, so triangles from other slices or copies are never counted.
//! [`Probe`] then counts the location's depth-won tag pixels from the final
//! frame and records the draw in a [`DrawLedger`]. Submitted faces entered the
//! rasteriser's input; they may still have been clipped or culled. Tag pixels
//! are per location, not per part or face.
use crate::{
    location_animation_receipt::{
        slots_digest, DrawEvidence, DrawIdentity, DrawLedger, Receipt, Stale, Submission, Target,
    },
    render3d::{Frame, Triangle},
    world_mesh::WorldMesh,
};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

/// Index of the element at `addr` in a slice of `len` elements of `size`
/// bytes starting at `base`, or `None` outside it or off an element boundary.
pub fn slot_index(base: usize, len: usize, size: usize, addr: usize) -> Option<usize> {
    let offset = addr.checked_sub(base)?;
    if size == 0 || offset % size != 0 {
        return None;
    }
    let index = offset / size;
    (index < len).then_some(index)
}

struct Owned {
    target: Target,
    tag: u64,
    slots: Vec<usize>,
    same_tag_other_slots: usize,
}

/// Counts, during one draw, which animated slots were yielded and how often.
pub struct Tally<'m> {
    triangles: &'m [Triangle],
    owned: Vec<Owned>,
    owner: HashMap<usize, usize>,
    overlapping: Option<(Target, Target)>,
    yields: RefCell<HashMap<usize, usize>>,
    foreign: RefCell<usize>,
}
impl<'m> Tally<'m> {
    pub fn new(mesh: &'m WorldMesh) -> Self {
        let triangles = mesh.triangles.as_slice();
        let mut owned: Vec<Owned> = mesh
            .animated_slots()
            .map(|(target, tag, slots)| Owned { target, tag, slots, same_tag_other_slots: 0 })
            .collect();
        let mut owner = HashMap::new();
        let mut overlapping = None;
        for (i, o) in owned.iter().enumerate() {
            for &slot in &o.slots {
                if let Some(previous) = owner.insert(slot, i) {
                    overlapping.get_or_insert((owned[previous].target, o.target));
                }
            }
        }
        // One pass over the mesh, then per part: same-tag triangles that are not its own slots.
        let mut per_tag: HashMap<u64, usize> = HashMap::new();
        for t in triangles {
            *per_tag.entry(t.tag).or_default() += 1;
        }
        for o in &mut owned {
            let own = o.slots.iter().filter(|&&s| triangles[s].tag == o.tag).count();
            o.same_tag_other_slots = per_tag.get(&o.tag).copied().unwrap_or(0).saturating_sub(own);
        }
        Self {
            triangles,
            owned,
            owner,
            overlapping,
            yields: RefCell::new(HashMap::new()),
            foreign: RefCell::new(0),
        }
    }
    fn see(&self, t: &Triangle) {
        let base = self.triangles.as_ptr() as usize;
        let index = slot_index(base, self.triangles.len(), std::mem::size_of::<Triangle>(), t as *const Triangle as usize)
            .filter(|&i| std::ptr::eq(&self.triangles[i], t));
        match index {
            Some(i) if self.owner.contains_key(&i) => *self.yields.borrow_mut().entry(i).or_default() += 1,
            Some(_) => {}
            None => *self.foreign.borrow_mut() += 1,
        }
    }
    /// Wrap the root-mesh iterator given to `Frame::draw_iter`. Triangles
    /// from anywhere else pass through uncounted as foreign.
    pub fn wrap<'t, 'a: 't, I>(&'t self, triangles: I) -> impl Iterator<Item = &'a Triangle> + 't
    where
        I: Iterator<Item = &'a Triangle> + 't,
    {
        triangles.inspect(move |t| self.see(t))
    }
    /// End the draw's mesh borrow. Call right after `draw_iter` returns.
    pub fn finish(self) -> Seen {
        let yields = self.yields.into_inner();
        let submissions = self
            .owned
            .iter()
            .map(|o| {
                let yielded: Vec<usize> = o.slots.iter().copied().filter(|s| yields.contains_key(s)).collect();
                let written: Option<Vec<_>> = yielded.iter().map(|&s| self.triangles[s].local_vertices).collect();
                Submission {
                    target: o.target,
                    tag: o.tag,
                    yielded_slots: yielded.len(),
                    yields: yielded.iter().map(|s| yields[s]).sum(),
                    // A slot without written vertices can never match the prepared digest.
                    slots_digest: written.map_or([0; 32], |v| slots_digest(&v)),
                    same_tag_other_slots: o.same_tag_other_slots,
                    tag_pixels: 0,
                }
            })
            .collect();
        Seen { submissions, overlapping: self.overlapping, foreign: self.foreign.into_inner() }
    }
}

/// A finished tally, owning no borrow of the mesh.
pub struct Seen {
    submissions: Vec<Submission>,
    overlapping: Option<(Target, Target)>,
    /// Yielded triangles outside the mesh's slice (other chained sources).
    pub foreign: usize,
}

/// Holds the ledger across paints; the live client owns one.
#[derive(Debug, Default)]
pub struct Probe {
    ledger: DrawLedger,
}
impl Probe {
    pub fn begin_paint(&mut self) -> u64 {
        self.ledger.begin_paint()
    }
    /// Record a completed root draw. Call before the frame moves, with the
    /// receipt of the mesh that was drawn. A submission without a recorded
    /// observation is dropped only when the receipt was truncated, where the
    /// omission is legitimate; otherwise it is kept, and validation refuses
    /// it as an orphan.
    pub fn complete(
        &mut self,
        receipt: Receipt,
        seen: Seen,
        frame: &Frame,
        roof_plane: u8,
        enhanced: bool,
    ) -> Result<DrawIdentity, Stale> {
        let wanted: HashSet<u64> = seen.submissions.iter().map(|s| s.tag).collect();
        let mut pixels: HashMap<u64, u32> = HashMap::new();
        for (&tag, &view) in frame.tags.iter().zip(&frame.views) {
            if view == 0 && wanted.contains(&tag) {
                *pixels.entry(tag).or_default() += 1;
            }
        }
        let submissions = seen
            .submissions
            .into_iter()
            .filter(|s| !receipt.truncated || receipt.observations.iter().any(|o| o.target == s.target))
            .map(|s| Submission { tag_pixels: pixels.get(&s.tag).copied().unwrap_or(0), ..s })
            .collect();
        self.ledger.complete(DrawEvidence { receipt, submissions, roof_plane, enhanced, overlapping: seen.overlapping })
    }
    pub fn invalidate(&mut self) {
        self.ledger.invalidate();
    }
    pub fn ledger(&self) -> &DrawLedger {
        &self.ledger
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation_runtime::test_assets::*;
    use crate::location_animation_receipt::{pose_digest, timing_digest, validate, Expected, Failure, Outcome};
    use crate::render3d::Camera;
    use crate::world_mesh::{test_support::{self, CENTRE}, WorldMesh};

    #[test]
    fn slot_index_rejects_every_address_outside_an_element() {
        let (base, len, size) = (4096, 3, 16);
        for (addr, want) in [
            (0, None),
            (base - size, None),
            (base - 1, None),
            (base, Some(0)),
            (base + size, Some(1)),
            (base + 2 * size, Some(2)),
            (base + 3 * size, None),
            (usize::MAX, None),
            (base + 1, None),
            (base + size - 1, None),
        ] {
            assert_eq!(slot_index(base, len, size, addr), want, "{addr}");
        }
        assert_eq!(slot_index(base, len, 0, base), None);
        assert_eq!(slot_index(base, 0, size, base), None);
    }

    /// The mesh advanced by `ticks` from a fresh clock (126: frame 2, elapsed 1).
    fn advanced(mut mesh: WorldMesh, ticks: u32) -> WorldMesh {
        let mut cache = EmptyCache::new("draw-advance");
        let mut animations = animations(sequence(LENGTHS), [1, 1], PIVOT);
        let scene = crate::scene::Scene::empty_sized(3200, 3200, 16, 16).unwrap();
        mesh.advance(&scene, &mut cache.1, &mut animations, ticks).unwrap();
        mesh
    }
    fn expected(frame: usize, elapsed: u32) -> Expected {
        Expected {
            target: test_support::target(0),
            model_ids: vec![60000],
            sequence_id: SEQUENCE,
            timing_digest: timing_digest(&sequence(LENGTHS)),
            random_start: false,
            outcome: Outcome::Classic { frame_index: frame, frame_id: FRAMES[frame], skeleton_id: SKELETON },
            pose_digest: pose_digest(&POSES[frame].map(|v| v.map(|c| c as f32))),
            triangles: 12,
            frame_length: Some(LENGTHS[frame] as u32),
            elapsed: elapsed..=elapsed,
            loops: 0..=0,
            min_tag_pixels: 1,
        }
    }
    /// Looks along +z at the location from 600 units in front of it.
    fn facing() -> Camera {
        Camera::angles([CENTRE[0] + 40, -20, CENTRE[1] - 600], 0, 0, 128.0).unwrap()
    }
    /// A tag no test location uses.
    const WALL: u64 = 99;
    /// Draw once through `draw`, then record the draw as the live client will.
    fn capture<'m>(mesh: &'m WorldMesh, enhanced: bool, draw: impl FnOnce(&mut Frame, &Tally<'m>)) -> (Probe, usize) {
        let (probe, foreign, _) = capture_all(mesh, enhanced, draw);
        (probe, foreign)
    }
    /// As [`capture`], keeping the drawn frame for independent pixel checks.
    fn capture_frame<'m>(mesh: &'m WorldMesh, enhanced: bool, draw: impl FnOnce(&mut Frame, &Tally<'m>)) -> (Probe, Frame) {
        let (probe, _, frame) = capture_all(mesh, enhanced, draw);
        (probe, frame)
    }
    fn capture_all<'m>(mesh: &'m WorldMesh, enhanced: bool, draw: impl FnOnce(&mut Frame, &Tally<'m>)) -> (Probe, usize, Frame) {
        let mut probe = Probe::default();
        probe.begin_paint();
        let mut frame = Frame::new(64, 64).unwrap();
        let tally = Tally::new(mesh);
        draw(&mut frame, &tally);
        let seen = tally.finish();
        let foreign = seen.foreign;
        probe.complete(mesh.location_receipt(&[test_support::target(0)]), seen, &frame, 3, enhanced).unwrap();
        (probe, foreign, frame)
    }
    fn result(probe: &Probe, e: Expected) -> Result<DrawIdentity, Failure> {
        let camera = probe.ledger().capture().map(|c| c.identity).unwrap_or(DrawIdentity { draw_id: 0, paint_id: 0 });
        validate(probe.ledger().capture(), camera, &[e])
    }
    fn palette() -> Vec<u32> {
        (0..65536u32).map(|v| v.wrapping_mul(123457) & 0xffffff).collect()
    }
    fn submission(probe: &Probe) -> Submission {
        probe.ledger().capture().unwrap().evidence.submissions[0].clone()
    }
    fn plain(mesh: &WorldMesh, frame: &mut Frame, tally: &Tally, camera: &Camera) {
        frame.draw_iter(camera, tally.wrap(mesh.visible_triangles(3)), &palette(), &HashMap::new());
    }

    #[test]
    fn a_visible_animated_location_validates_from_the_real_draw() {
        let mesh = advanced(test_support::standard(), 126);
        let (probe, foreign) = capture(&mesh, false, |frame, tally| plain(&mesh, frame, tally, &facing()));
        assert_eq!(foreign, 0);
        let s = submission(&probe);
        assert_eq!((s.yielded_slots, s.yields, s.same_tag_other_slots), (12, 12, 0));
        assert!(s.tag_pixels > 0, "{s:?}");
        assert_eq!(result(&probe, expected(2, 1)), Ok(DrawIdentity { draw_id: 1, paint_id: 1 }));
        // Another frame's declaration never matches this draw.
        assert_eq!(result(&probe, expected(1, 1)), Err(Failure::Mismatch(test_support::target(0), "outcome")));
    }

    #[test]
    fn submitted_but_unseen_locations_are_not_visible() {
        let mesh = advanced(test_support::standard(), 126);
        let target = test_support::target(0);
        // Turned away, and placed beyond the location looking further on.
        let away = Camera::angles([CENTRE[0] + 40, -20, CENTRE[1] - 600], 0, 8192, 128.0).unwrap();
        let beyond = Camera::angles([CENTRE[0] + 40, -20, CENTRE[1] + 400], 0, 0, 128.0).unwrap();
        for camera in [away, beyond] {
            let (probe, _) = capture(&mesh, false, |frame, tally| plain(&mesh, frame, tally, &camera));
            assert_eq!(submission(&probe).yielded_slots, 12);
            assert_eq!(result(&probe, expected(2, 1)), Err(Failure::NotVisible { target, tag_pixels: 0, min: 1 }));
        }
        // A nearer, opaque, depth-writing wall with another tag, centred on the
        // camera 300 units in front of it, covering the whole view.
        // `selection_only` makes picking-only faces (alpha 255, never rasterised),
        // so the wall must be made opaque explicitly.
        let mut wall = Triangle::selection_only(crate::model_bounds::Bounds::new(&[[0.0; 3]], [0; 3]), WALL);
        wall.model_bounds = None;
        wall.alpha = 0;
        let (x, z) = ((CENTRE[0] + 40) as f32, (CENTRE[1] - 300) as f32);
        wall.vertices = [[x - 20000.0, 20000.0, z], [x, -20000.0, z], [x + 20000.0, 20000.0, z]];
        wall.colors = [100, 100, 100];
        // Both windings, so face culling cannot let the location show through.
        let mut reversed = wall;
        reversed.vertices.swap(1, 2);
        let walls = [wall, reversed];
        let pixels = |frame: &Frame, tag: u64| frame.tags.iter().zip(&frame.views).filter(|&(&t, &v)| t == tag && v == 0).count();
        // Control: without the wall, the same view shows the location.
        let (_, open) = capture_frame(&mesh, false, |frame, tally| plain(&mesh, frame, tally, &facing()));
        assert!(pixels(&open, test_support::tag()) > 0, "the location is visible without the wall");
        assert_eq!(pixels(&open, WALL), 0);
        let (probe, frame) = capture_frame(&mesh, false, |frame, tally| {
            frame.draw_iter(&facing(), tally.wrap(mesh.visible_triangles(3)).chain(walls.iter()), &palette(), &HashMap::new())
        });
        // Independently of the probe: the wall was drawn and won every location pixel.
        assert!(pixels(&frame, WALL) > 0, "the wall is rasterised and depth-written");
        assert_eq!(pixels(&frame, test_support::tag()), 0, "no location pixel survives the wall");
        assert_eq!(submission(&probe).yielded_slots, 12, "the location was still submitted");
        assert_eq!(result(&probe, expected(2, 1)), Err(Failure::NotVisible { target, tag_pixels: 0, min: 1 }));
    }

    #[test]
    fn roof_filtered_partial_and_repeated_submissions_fail() {
        let target = test_support::target(0);
        let roofed = advanced(test_support::one(1, SEQUENCE), 126);
        let (probe, _) = capture(&roofed, false, |frame, tally| {
            frame.draw_iter(&facing(), tally.wrap(roofed.visible_triangles(0)), &palette(), &HashMap::new())
        });
        assert_eq!(result(&probe, expected(2, 1)), Err(Failure::NotSubmitted(target)));
        let mesh = advanced(test_support::standard(), 126);
        let (probe, _) = capture(&mesh, false, |frame, tally| {
            frame.draw_iter(&facing(), tally.wrap(mesh.visible_triangles(3).skip(1)), &palette(), &HashMap::new())
        });
        assert_eq!(result(&probe, expected(2, 1)), Err(Failure::PartiallySubmitted { target, yielded: 11, expected: 12 }));
        let (probe, _) = capture(&mesh, false, |frame, tally| {
            let again = std::iter::once(&mesh.triangles[0]);
            frame.draw_iter(&facing(), tally.wrap(mesh.visible_triangles(3).chain(again)), &palette(), &HashMap::new())
        });
        assert_eq!(result(&probe, expected(2, 1)), Err(Failure::DuplicateYield { target, yields: 13, slots: 12 }));
    }

    #[test]
    fn copies_and_other_slices_are_never_attributed() {
        let mesh = advanced(test_support::standard(), 126);
        // Equal contents at other addresses.
        let copies: Vec<Triangle> = mesh.triangles[..12].to_vec();
        let side = if (copies.as_ptr() as usize) < (mesh.triangles.as_ptr() as usize) { "below" } else { "above" };
        eprintln!("copies are allocated {side} the mesh");
        let (probe, foreign) = capture(&mesh, false, |frame, tally| {
            frame.draw_iter(&facing(), tally.wrap(copies.iter()), &palette(), &HashMap::new())
        });
        assert_eq!(foreign, 12);
        assert_eq!(result(&probe, expected(2, 1)), Err(Failure::NotSubmitted(test_support::target(0))));
        let (probe, foreign) = capture(&mesh, false, |frame, tally| {
            frame.draw_iter(&facing(), tally.wrap(mesh.visible_triangles(3).chain(copies.iter())), &palette(), &HashMap::new())
        });
        assert_eq!(foreign, 12);
        assert!(result(&probe, expected(2, 1)).is_ok());
    }

    #[test]
    fn slots_changed_after_the_pose_are_rejected() {
        let target = test_support::target(0);
        let mut mesh = advanced(test_support::standard(), 126);
        mesh.triangles[0].local_vertices.as_mut().unwrap()[0][0] += 0.5;
        let (probe, _) = capture(&mesh, false, |frame, tally| plain(&mesh, frame, tally, &facing()));
        assert_eq!(result(&probe, expected(2, 1)), Err(Failure::SubmittedChanged(target)));
        // Frame 0's first slot starts at the origin: +0.0 replaced by -0.0.
        let mut mesh = advanced(test_support::standard(), 0);
        let slot = mesh.triangles[0].local_vertices.as_mut().unwrap();
        assert_eq!(slot[0][0].to_bits(), 0.0f32.to_bits());
        slot[0][0] = -0.0;
        let (probe, _) = capture(&mesh, false, |frame, tally| plain(&mesh, frame, tally, &facing()));
        assert_eq!(result(&probe, expected(0, 0)), Err(Failure::SubmittedChanged(target)));
    }

    #[test]
    fn shared_tags_overlapping_slots_enhanced_and_stale_draws_fail() {
        let target = test_support::target(0);
        let mut shared = test_support::standard();
        test_support::add_same_tag_static(&mut shared);
        let shared = advanced(shared, 126);
        let (probe, _) = capture(&shared, false, |frame, tally| plain(&shared, frame, tally, &facing()));
        assert_eq!(result(&probe, expected(2, 1)), Err(Failure::TagShared { target, others: 1 }));
        let mut overlapping = test_support::standard();
        test_support::add_overlapping_part(&mut overlapping);
        let overlapping = advanced(overlapping, 126);
        let (probe, _) = capture(&overlapping, false, |frame, tally| plain(&overlapping, frame, tally, &facing()));
        assert_eq!(result(&probe, expected(2, 1)), Err(Failure::OverlappingSlots(target, test_support::target(1))));
        let mesh = advanced(test_support::standard(), 126);
        let (probe, _) = capture(&mesh, true, |frame, tally| plain(&mesh, frame, tally, &facing()));
        assert_eq!(result(&probe, expected(2, 1)), Err(Failure::EnhancedPresentation));
        let (mut probe, _) = capture(&mesh, false, |frame, tally| plain(&mesh, frame, tally, &facing()));
        probe.begin_paint();
        assert!(matches!(result(&probe, expected(2, 1)), Err(Failure::Stale(Stale::PaintAdvanced { .. }))));
        probe.invalidate();
        assert!(matches!(result(&probe, expected(2, 1)), Err(Failure::Stale(Stale::NoCompletedDraw { .. }))));
    }

    #[test]
    fn a_drawn_part_missing_from_a_complete_receipt_is_an_orphan() {
        // The part gains slots after the advance that built the receipt.
        let mut mesh = advanced(test_support::standard(), 126);
        test_support::add_unadvanced_part(&mut mesh);
        let (probe, _) = capture(&mesh, false, |frame, tally| plain(&mesh, frame, tally, &facing()));
        let completed = probe.ledger().capture().unwrap();
        assert!(!completed.evidence.receipt.truncated);
        assert_eq!(completed.evidence.submissions.len(), 2, "the unrecorded submission is kept");
        assert_eq!(result(&probe, expected(2, 1)), Err(Failure::SubmissionOrphan(test_support::target(2))));
    }

    #[test]
    fn only_a_truncated_receipt_drops_submissions_it_omitted() {
        let mut mesh = test_support::standard();
        test_support::add_other_locations(&mut mesh, 300);
        let mesh = advanced(mesh, 126);
        let (probe, _) = capture(&mesh, false, |frame, tally| plain(&mesh, frame, tally, &facing()));
        let completed = probe.ledger().capture().unwrap();
        let receipt = &completed.evidence.receipt;
        assert!(receipt.truncated);
        assert_eq!((receipt.total, receipt.observations.len()), (301, crate::location_animation_receipt::MAX_OBSERVATIONS));
        // One submission per kept observation; the declared target was kept first.
        assert_eq!(completed.evidence.submissions.len(), receipt.observations.len());
        assert_eq!(receipt.observations[0].target, test_support::target(0));
        assert!(result(&probe, expected(2, 1)).is_ok());
    }
}
