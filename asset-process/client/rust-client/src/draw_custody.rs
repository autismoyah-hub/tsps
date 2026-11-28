//! Passive draw/raster custody for the private test adapter (issue #175,
//! schema `draw-custody/1`). Records which world draw, if any, supplied the
//! raster a capture encodes, and the scene epoch it belongs to. Counters and
//! records only: nothing here forces a draw or changes rendering, input,
//! packets or game state. See rust-client/docs/harness/draw-custody.md.
use std::sync::atomic::{AtomicU64, Ordering};

pub const SCHEMA: &str = "draw-custody/1";

/// Process-unique identity of one `Custody` (one live `Connection`), so an
/// adapter never reads another connection's paint as its own frame.
static NEXT_IDENTITY: AtomicU64 = AtomicU64::new(1);

/// A scene boundary that invalidates every earlier draw for readiness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boundary { Reset, Reconnect, Rebuild, Installed }
impl Boundary {
    pub fn name(self) -> &'static str {
        match self { Self::Reset => "reset", Self::Reconnect => "reconnect", Self::Rebuild => "rebuild", Self::Installed => "installed" }
    }
}

/// What one `Connection::paint` call returned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Entered but not finished (an error left paint early).
    Incomplete,
    /// No loaded root world: a blank raster.
    NoWorld,
    /// No visible, non-empty viewport widget: a blank raster.
    NoViewport,
    /// The root world was drawn into the returned raster.
    Drawn,
}
impl Outcome {
    pub fn name(self) -> &'static str {
        match self { Self::Incomplete => "incomplete", Self::NoWorld => "no_world", Self::NoViewport => "no_viewport", Self::Drawn => "drawn" }
    }
}

/// One completed root world draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draw { pub generation: u64, pub epoch: u64, pub location_revision: u64, pub actor_cycle: i32 }
impl Draw {
    fn json(&self) -> String {
        format!("{{\"generation\":{},\"epoch\":{},\"location_revision\":{},\"actor_cycle\":{}}}",
            self.generation, self.epoch, self.location_revision, self.actor_cycle)
    }
}

/// The record of the most recent `paint` call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paint {
    pub serial: u64,
    pub outcome: Outcome,
    /// Only for `Drawn`: the draw this call put into its raster.
    pub draw: Option<Draw>,
    /// The title screen cleared this paint's raster afterwards (same frame).
    pub title_overwrote: bool,
    /// Child views admitted and projected during this paint. Admission and
    /// projection only: NOT proof that any child pixel is in the raster.
    pub child_views_admitted: Vec<[u64; 3]>,
}

#[derive(Debug)]
pub struct Custody {
    pub identity: u64,
    pub scene_epoch: u64,
    pub epoch_reason: Option<Boundary>,
    pub location_revision: u64,
    pub paint_serial: u64,
    pub draw_generation: u64,
    pub last_draw: Option<Draw>,
    pub last_paint: Option<Paint>,
    /// The draw generation whose pick/camera is current; None after any pick reset.
    pub camera_generation: Option<u64>,
    pub world_loaded: bool,
}
impl Default for Custody {
    fn default() -> Self { Self::new() }
}
impl Custody {
    pub fn new() -> Self {
        Self { identity: NEXT_IDENTITY.fetch_add(1, Ordering::Relaxed), scene_epoch: 0, epoch_reason: None,
            location_revision: 0, paint_serial: 0, draw_generation: 0, last_draw: None, last_paint: None,
            camera_generation: None, world_loaded: false }
    }
    /// A boundary: the pick is gone and every earlier draw belongs to an old epoch.
    pub fn boundary(&mut self, reason: Boundary, world_loaded: bool) {
        self.scene_epoch += 1;
        self.epoch_reason = Some(reason);
        self.camera_generation = None;
        self.world_loaded = world_loaded;
    }
    /// Dynamic location changes were applied to the loaded scene (pick reset).
    pub fn locations_applied(&mut self) {
        self.location_revision += 1;
        self.camera_generation = None;
    }
    /// Entry of every `paint` call; the record starts incomplete and untitled.
    pub fn paint_begin(&mut self) {
        self.paint_serial += 1;
        self.last_paint = Some(Paint { serial: self.paint_serial, outcome: Outcome::Incomplete, draw: None,
            title_overwrote: false, child_views_admitted: Vec::new() });
    }
    /// A blank-raster return (`NoWorld` or `NoViewport`). Never carries a draw.
    pub fn paint_blank(&mut self, outcome: Outcome) {
        debug_assert!(matches!(outcome, Outcome::NoWorld | Outcome::NoViewport));
        if let Some(paint) = self.last_paint.as_mut().filter(|p| p.serial == self.paint_serial) {
            paint.outcome = outcome;
            paint.draw = None;
        }
    }
    pub fn child_view_admitted(&mut self, id: u16, spawn_generation: u64, rebuild_generation: u64) {
        if let Some(paint) = self.last_paint.as_mut().filter(|p| p.serial == self.paint_serial) {
            paint.child_views_admitted.push([id as u64, spawn_generation, rebuild_generation]);
        }
    }
    /// At the site that publishes the pick and returns the drawn raster.
    pub fn paint_drawn(&mut self, actor_cycle: i32) {
        self.draw_generation += 1;
        let draw = Draw { generation: self.draw_generation, epoch: self.scene_epoch,
            location_revision: self.location_revision, actor_cycle };
        self.last_draw = Some(draw);
        self.camera_generation = Some(draw.generation);
        if let Some(paint) = self.last_paint.as_mut().filter(|p| p.serial == self.paint_serial) {
            paint.outcome = Outcome::Drawn;
            paint.draw = Some(draw);
        }
    }
    /// The title screen cleared the raster of the most recent paint.
    pub fn title_overwrote(&mut self) {
        if let Some(paint) = self.last_paint.as_mut() { paint.title_overwrote = true; }
    }
    /// The state.json `draw_custody` object: counters at receipt time. It is a
    /// pre-paint/completion observation and never describes a captured raster.
    pub fn state_json(&self) -> String {
        format!("{{\"schema\":\"{SCHEMA}\",\"scope\":\"connection counters when this receipt was written; not a captured raster\",\"connection\":{},\"scene_epoch\":{},\"epoch_reason\":{},\"world_loaded\":{},\"location_revision\":{},\"paint_serial\":{},\"draw_generation\":{},\"camera_generation\":{},\"last_draw\":{}}}",
            self.identity, self.scene_epoch, self.epoch_reason.map_or("null".into(), |r| format!("\"{}\"", r.name())),
            self.world_loaded, self.location_revision, self.paint_serial, self.draw_generation,
            opt(self.camera_generation), self.last_draw.map_or("null".into(), |d| d.json()))
    }
}

fn opt(value: Option<u64>) -> String { value.map_or("null".into(), |v| v.to_string()) }

/// Why a frame's raster has, or has no, paint of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    /// Exactly one paint of the same connection ran since the previous frame.
    Painted,
    /// No live connection this frame.
    NotLive,
    /// No paint since the previous frame (the raster is not a paint result).
    Unpainted,
    /// A counter gap, a reset or an unseen connection: fail closed.
    Ambiguous,
}
impl Link {
    pub fn name(self) -> &'static str {
        match self { Self::Painted => "painted", Self::NotLive => "not_live", Self::Unpainted => "unpainted", Self::Ambiguous => "ambiguous" }
    }
}

/// Adapter-side bookkeeping, advanced on EVERY `Adapter::frame` call. It keeps
/// the last serial seen for every connection identity, so a connection that
/// reappears can never have an old paint claimed again.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FrameTracker {
    pub frames: u64,
    /// The connection seen by the most recent live frame.
    last_live: Option<u64>,
    /// Last paint serial seen per connection identity.
    seen: std::collections::BTreeMap<u64, u64>,
    /// Highest identity ever seen (identities are allocated monotonically).
    max_identity: u64,
}
impl FrameTracker {
    /// Links this frame to the custody's latest paint and records what was seen.
    /// - The connection of the previous live frame: `painted` only for exactly
    ///   one new paint, `unpainted` for none, otherwise `ambiguous`.
    /// - A connection seen earlier but not on the previous live frame:
    ///   `unpainted` if its serial is unchanged, otherwise `ambiguous` (a paint
    ///   while another connection was observed cannot be placed in a frame).
    /// - A connection never seen: `painted` only if it is newer than every
    ///   identity seen and its whole life is exactly one paint; `unpainted` with
    ///   no paint; otherwise `ambiguous`.
    /// A frame without a connection changes none of this.
    pub fn observe(&mut self, custody: Option<&Custody>) -> Link {
        self.frames += 1;
        let Some(c) = custody else { return Link::NotLive; };
        let (identity, serial) = (c.identity, c.paint_serial);
        let link = match self.seen.get(&identity).copied() {
            Some(last) if self.last_live == Some(identity) => {
                if serial == last { Link::Unpainted }
                else if last.checked_add(1) == Some(serial) { Link::Painted }
                else { Link::Ambiguous }
            }
            Some(last) => if serial == last { Link::Unpainted } else { Link::Ambiguous },
            None if serial == 0 => Link::Unpainted,
            None if serial == 1 && identity > self.max_identity => Link::Painted,
            None => Link::Ambiguous,
        };
        self.seen.insert(identity, serial);
        self.last_live = Some(identity);
        self.max_identity = self.max_identity.max(identity);
        link
    }
}

/// `frame-<seq>-draw.json`: custody of the raster this frame encodes.
pub fn frame_json(sequence: u64, frame: u64, link: Link, custody: Option<&Custody>) -> String {
    let paint = custody.and_then(|c| c.last_paint.as_ref()).filter(|_| link == Link::Painted);
    let outcome = match link {
        Link::Painted => paint.map_or("incomplete", |p| p.outcome.name()),
        other => other.name(),
    };
    let draw = paint.and_then(|p| p.draw);
    let title = paint.is_some_and(|p| p.title_overwrote);
    let current = custody.zip(draw).is_some_and(|(c, d)| !title && d.epoch == c.scene_epoch
        && d.location_revision == c.location_revision && c.camera_generation == Some(d.generation));
    let children = paint.map_or(String::new(), |p| p.child_views_admitted.iter()
        .map(|[id, spawn, rebuild]| format!("[{id},{spawn},{rebuild}]")).collect::<Vec<_>>().join(","));
    let field = |value: Option<String>| value.unwrap_or_else(|| "null".into());
    format!("{{\"schema\":\"{SCHEMA}\",\"sequence\":{sequence},\"adapter_frame\":{frame},\"link\":\"{}\",\"connection\":{},\"paint_serial\":{},\"outcome\":\"{outcome}\",\"title_overwrote\":{title},\"raster_draw_generation\":{},\"raster_draw_epoch\":{},\"raster_location_revision\":{},\"raster_actor_cycle\":{},\"current_epoch\":{},\"current_location_revision\":{},\"camera_generation\":{},\"child_views_admitted\":[{children}],\"world_raster_current\":{current},\"limits\":\"world pixels lie inside the viewport clip beneath UI; Enhanced may replace display pixels of the same draw; child_views_admitted is admission/projection only, not child pixel proof; no pixel-content check\"}}\n",
        link.name(),
        field(custody.map(|c| c.identity.to_string())),
        field(paint.map(|p| p.serial.to_string())),
        field(draw.map(|d| d.generation.to_string())),
        field(draw.map(|d| d.epoch.to_string())),
        field(draw.map(|d| d.location_revision.to_string())),
        field(draw.map(|d| d.actor_cycle.to_string())),
        field(custody.map(|c| c.scene_epoch.to_string())),
        field(custody.map(|c| c.location_revision.to_string())),
        field(custody.and_then(|c| c.camera_generation).map(|g| g.to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn current(json: &str) -> bool { json.contains("\"world_raster_current\":true") }

    #[test] fn boundaries_advance_the_epoch_and_drop_the_camera() {
        let mut c = Custody::new();
        for (n, reason) in [Boundary::Reset, Boundary::Reconnect, Boundary::Rebuild, Boundary::Installed].into_iter().enumerate() {
            c.paint_begin(); c.paint_drawn(5);
            c.boundary(reason, reason == Boundary::Installed);
            assert_eq!((c.scene_epoch, c.epoch_reason, c.camera_generation), (n as u64 + 1, Some(reason), None));
        }
        assert!(c.world_loaded);
        assert!(c.state_json().contains("\"epoch_reason\":\"installed\""));
    }
    #[test] fn blank_paints_advance_the_serial_but_never_the_generation() {
        let mut c = Custody::new();
        let mut t = FrameTracker::default();
        for outcome in [Outcome::NoWorld, Outcome::NoViewport] {
            c.paint_begin(); c.paint_blank(outcome);
            let json = frame_json(1, 0, t.observe(Some(&c)), Some(&c));
            assert!(json.contains(&format!("\"outcome\":\"{}\"", outcome.name())));
            assert!(json.contains("\"raster_draw_generation\":null"));
            assert!(!current(&json));
        }
        assert_eq!((c.paint_serial, c.draw_generation), (2, 0));
    }
    #[test] fn a_drawn_paint_links_its_generation_epoch_and_cycle() {
        let mut c = Custody::new();
        c.boundary(Boundary::Installed, true);
        let mut t = FrameTracker::default();
        c.paint_begin(); c.paint_drawn(77);
        let json = frame_json(9, 0, t.observe(Some(&c)), Some(&c));
        assert!(json.contains("\"link\":\"painted\""), "{json}");
        assert!(json.contains("\"outcome\":\"drawn\""));
        assert!(json.contains("\"raster_draw_generation\":1,\"raster_draw_epoch\":1,\"raster_location_revision\":0,\"raster_actor_cycle\":77"));
        assert!(current(&json));
    }
    #[test] fn a_boundary_after_the_draw_is_not_current() {
        let mut c = Custody::new();
        let mut t = FrameTracker::default();
        c.paint_begin(); c.paint_drawn(1);
        c.boundary(Boundary::Rebuild, false);
        let json = frame_json(1, 0, t.observe(Some(&c)), Some(&c));
        assert!(json.contains("\"raster_draw_epoch\":0") && json.contains("\"current_epoch\":1"));
        assert!(!current(&json));
    }
    #[test] fn a_location_revision_after_the_draw_is_not_current() {
        let mut c = Custody::new();
        let mut t = FrameTracker::default();
        c.paint_begin(); c.paint_drawn(1); c.locations_applied();
        assert!(!current(&frame_json(1, 0, t.observe(Some(&c)), Some(&c))));
    }
    #[test] fn a_title_overwrite_is_not_current_and_resets_on_the_next_paint() {
        let mut c = Custody::new();
        let mut t = FrameTracker::default();
        c.paint_begin(); c.paint_drawn(1); c.title_overwrote();
        let json = frame_json(1, 0, t.observe(Some(&c)), Some(&c));
        assert!(json.contains("\"title_overwrote\":true") && !current(&json));
        c.paint_begin(); c.paint_drawn(2);
        let json = frame_json(2, 0, t.observe(Some(&c)), Some(&c));
        assert!(json.contains("\"title_overwrote\":false") && current(&json));
    }
    #[test] fn a_frame_without_its_own_paint_never_inherits_the_earlier_draw() {
        let mut c = Custody::new();
        let mut t = FrameTracker::default();
        c.paint_begin(); c.paint_drawn(1);
        assert_eq!(t.observe(Some(&c)), Link::Painted);
        let link = t.observe(Some(&c));
        assert_eq!(link, Link::Unpainted);
        let json = frame_json(2, 1, link, Some(&c));
        assert!(json.contains("\"outcome\":\"unpainted\"") && json.contains("\"paint_serial\":null")
            && json.contains("\"raster_draw_generation\":null") && !current(&json));
    }
    #[test] fn a_stale_camera_over_a_blank_viewport_is_not_current() {
        let mut c = Custody::new();
        let mut t = FrameTracker::default();
        c.paint_begin(); c.paint_drawn(1); t.observe(Some(&c));
        c.paint_begin(); c.paint_blank(Outcome::NoViewport);
        let json = frame_json(2, 1, t.observe(Some(&c)), Some(&c));
        assert!(json.contains("\"camera_generation\":1") && json.contains("\"outcome\":\"no_viewport\""));
        assert!(json.contains("\"raster_draw_generation\":null") && !current(&json));
    }
    #[test] fn child_views_are_admissions_listed_per_paint_only() {
        let mut c = Custody::new();
        c.paint_begin(); c.child_view_admitted(9, 42, 3); c.paint_drawn(1);
        let mut t = FrameTracker::default();
        let json = frame_json(1, 0, t.observe(Some(&c)), Some(&c));
        assert!(json.contains("\"child_views_admitted\":[[9,42,3]]"));
        c.paint_begin(); c.paint_drawn(2);
        assert!(frame_json(2, 1, t.observe(Some(&c)), Some(&c)).contains("\"child_views_admitted\":[]"));
    }
    #[test] fn frames_fail_closed_on_live_loss_new_connection_and_counter_gaps() {
        let mut t = FrameTracker::default();
        assert_eq!(t.observe(None), Link::NotLive);
        let mut a = Custody::new();
        a.paint_begin(); a.paint_drawn(1);
        assert_eq!(t.observe(Some(&a)), Link::Painted, "a new connection's one and only paint");
        a.paint_begin(); a.paint_begin();
        assert_eq!(t.observe(Some(&a)), Link::Ambiguous, "two paints between frames");
        let mut b = Custody::new();
        b.paint_begin(); b.paint_begin();
        assert_eq!(t.observe(Some(&b)), Link::Ambiguous, "an unseen connection with history");
        assert_eq!(t.observe(None), Link::NotLive);
        assert_eq!(t.observe(Some(&b)), Link::Unpainted, "a frame without a connection never frees an old paint");
        b.paint_begin();
        assert_eq!(t.observe(Some(&b)), Link::Painted, "exactly one new paint of the seen connection");
        assert_eq!(t.observe(Some(&a)), Link::Unpainted, "a reappearing connection without a new paint");
        let json = frame_json(1, t.frames, Link::Ambiguous, Some(&a));
        assert!(json.contains("\"outcome\":\"ambiguous\"") && !current(&json));
        assert_eq!(t.frames, 8);
        assert!(frame_json(1, 0, Link::NotLive, None).contains("\"connection\":null"));
    }
    #[test] fn an_incomplete_paint_is_not_current() {
        let mut c = Custody::new();
        let mut t = FrameTracker::default();
        c.paint_begin();
        let json = frame_json(1, 0, t.observe(Some(&c)), Some(&c));
        assert!(json.contains("\"outcome\":\"incomplete\"") && !current(&json));
    }
    #[test] fn a_reappearing_connection_never_reclaims_its_old_paint() {
        let mut t = FrameTracker::default();
        let mut a = Custody::new();
        let mut b = Custody::new();
        a.paint_begin(); a.paint_drawn(1);
        assert_eq!(t.observe(Some(&a)), Link::Painted);
        b.paint_begin(); b.paint_drawn(1);
        assert_eq!(t.observe(Some(&b)), Link::Painted);
        let link = t.observe(Some(&a));
        assert_eq!(link, Link::Unpainted, "A1 -> B1 -> A1 without a new paint");
        let json = frame_json(3, t.frames, link, Some(&a));
        assert!(json.contains("\"raster_draw_generation\":null") && !current(&json), "{json}");
        b.paint_begin();
        assert_eq!(t.observe(Some(&b)), Link::Ambiguous, "B painted while A was observed");
        a.paint_begin();
        assert_eq!(t.observe(Some(&a)), Link::Ambiguous, "A painted while B was observed");
    }
    #[test] fn a_reappearing_connection_after_live_loss_never_reclaims_its_old_paint() {
        let mut t = FrameTracker::default();
        let mut a = Custody::new();
        let mut b = Custody::new();
        a.paint_begin(); a.paint_drawn(1);
        assert_eq!(t.observe(Some(&a)), Link::Painted);
        assert_eq!(t.observe(None), Link::NotLive);
        b.paint_begin();
        assert_eq!(t.observe(Some(&b)), Link::Painted);
        assert_eq!(t.observe(Some(&a)), Link::Unpainted, "A1 -> None -> B1 -> A1");
    }
    #[test] fn an_unseen_older_connection_is_never_painted() {
        let mut older = Custody::new();
        let mut newer = Custody::new();
        let mut t = FrameTracker::default();
        newer.paint_begin();
        assert_eq!(t.observe(Some(&newer)), Link::Painted);
        older.paint_begin();
        assert_eq!(t.observe(Some(&older)), Link::Ambiguous, "identity below one already seen");
    }
}
