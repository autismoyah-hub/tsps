//! Original GameModel.fj/bu stable bucket ordering and priority interleave.
#[derive(Clone, Copy, Debug)]
pub struct Face {
    pub index: usize,
    pub depth: i32,
    pub priority: u8,
}
pub fn sorted(mut faces: Vec<Face>, priorities: bool) -> Vec<usize> {
    faces.sort_by_key(|f| std::cmp::Reverse(f.depth));
    if !priorities {
        return faces.into_iter().map(|f| f.index).collect();
    }
    if faces.iter().any(|f| f.priority >= 12) {
        return Vec::new();
    }
    let mut groups: [Vec<Face>; 12] = std::array::from_fn(|_| Vec::new());
    for face in faces {
        groups[usize::from(face.priority)].push(face);
    }
    let average = |a: usize, b: usize| {
        let count = groups[a].len() + groups[b].len();
        if count == 0 {
            0
        } else {
            groups[a]
                .iter()
                .chain(&groups[b])
                .map(|f| f.depth)
                .fold(0i32, |sum, value| sum.wrapping_add(value))
                / count as i32
        }
    };
    let thresholds = [average(1, 2), average(3, 4), average(6, 8)];
    let mut special = groups[10].iter().chain(&groups[11]).peekable();
    let mut result = Vec::new();
    for priority in 0..10 {
        let threshold = match priority {
            0 => Some(thresholds[0]),
            3 => Some(thresholds[1]),
            5 => Some(thresholds[2]),
            _ => None,
        };
        if let Some(threshold) = threshold {
            while special.peek().is_some_and(|f| f.depth > threshold) {
                result.push(special.next().unwrap().index);
            }
        }
        result.extend(groups[priority].iter().map(|f| f.index));
    }
    result.extend(special.map(|f| f.index));
    result
}

/// GameModel.fr raster policies (fq.ag, fq.aa, fq.af, fq.ae, fq.as).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Mode {
    #[default]
    Auto,
    Source,
    Depth,
    /// Priority color pass without depth writes, then a depth-only pass.
    Priority,
    Background,
}

impl Mode {
    pub fn actor(player: bool, npc_z_buffer: bool, player_override: bool) -> Self {
        if player || !npc_z_buffer || player_override {
            Self::Priority
        } else {
            Self::Auto
        }
    }
}

/// Face metadata retained through placement and child-view transforms.
#[derive(Clone, Copy, Debug)]
pub struct Submission {
    /// Actor identity for attached effects, independent of the picking tag.
    pub actor_spot: Option<u64>,
    /// Explicit insertion order for non-pickable temporary renderables.
    pub source_ordinal: Option<usize>,
    /// Accepted temporary scene input identity for non-pickable standalone effects.
    pub temporary_id: Option<usize>,
    /// Scene sorting uses the insertion height before animation displacement.
    pub scene_height_offset: i32,
    pub mode: Mode,
    pub priority: u8,
    pub has_priorities: bool,
    pub has_alpha: bool,
    /// Index of the renderable's first submitted face; attachments reuse their body's index.
    pub batch: usize,
}
impl Submission {
    pub fn new(model: &crate::model::Model, face: usize, batch: usize) -> Self {
        Self {
            actor_spot: None,
            source_ordinal: None,
            temporary_id: None,
            scene_height_offset: 0,
            mode: Mode::Auto,
            priority: model
                .priorities
                .as_ref()
                .map_or(model.priority, |p| p[face]),
            has_priorities: model.priorities.is_some(),
            has_alpha: model.alpha.is_some(),
            batch,
        }
    }
}

/// Join independently built face lists without merging their renderable identities.
/// Tags alone are insufficient: non-interactive spot effects all use tag zero.
pub fn append(
    out: &mut Vec<crate::render3d::Triangle>,
    mut incoming: Vec<crate::render3d::Triangle>,
) {
    rebase(&mut incoming, out.len());
    out.extend(incoming);
}
/// Rebase one independently built list before chaining it with prior submissions.
pub fn rebase(triangles: &mut [crate::render3d::Triangle], base: usize) {
    for triangle in triangles {
        if let Some(order) = &mut triangle.model_order {
            order.batch += base;
        }
    }
}
