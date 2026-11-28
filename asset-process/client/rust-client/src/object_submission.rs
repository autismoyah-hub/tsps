//! Multi-tile GameGameObject admission in the two software scene passes.
use crate::{
    scene::{PlacedObject, Scene},
    scene_visibility::Visibility,
    terrain_submission::View,
};
pub struct BoundarySubmission {
    pub identity: usize,
    pub batch: usize,
}
pub struct GroundSubmission {
    pub identity: usize,
    pub batch: usize,
}
pub struct DecorationSubmission {
    pub identity: usize,
    pub batch: usize,
}
pub struct Submission {
    pub batch: usize,
    pub traversal_id: Option<usize>,
    pub object: PlacedObject,
    pub size: [usize; 2],
}
pub fn admitted(
    view: &View,
    visibility: &mut Visibility,
    origin: [i32; 2],
    tiles: impl IntoIterator<Item = ([i32; 2], usize, i32)>,
) -> [bool; 2] {
    let mut result = [false; 2];
    for (tile, minimum, height) in tiles {
        let visible = view.slot_admitted(visibility, tile, minimum, height);
        if tile == origin {
            result[0] = visible;
        }
        result[1] |= visible;
    }
    result
}
pub fn scene_object(
    view: &View,
    visibility: &mut Visibility,
    scene: &Scene,
    object: &PlacedObject,
    size: [usize; 2],
) -> [bool; 2] {
    let mut result = [false; 2];
    if object.rejected || object.plane >= 4 {
        return result;
    }
    let origin = [object.x as usize, object.y as usize];
    for x in origin[0]..origin[0] + size[0] {
        for z in origin[1]..origin[1] + size[1] {
            if x >= scene.width || z >= scene.height {
                continue;
            }
            let tile = [x as i32, z as i32];
            let visible = if object.runtime {
                let occupant = PlacedObject {
                    x: x as u8,
                    y: z as u8,
                    ..object.clone()
                };
                scene.object_render_plane(&occupant).is_some_and(|minimum| {
                    view.slot_admitted(
                        visibility,
                        tile,
                        minimum,
                        scene.height(object.plane as usize, x, z).unwrap(),
                    )
                })
            } else {
                let flags = std::array::from_fn(|plane| scene.tile(plane, x, z).unwrap().flags);
                let heights = std::array::from_fn(|plane| scene.height(plane, x, z).unwrap());
                view.admitted(
                    visibility,
                    object.plane as usize,
                    tile,
                    flags,
                    heights,
                    flags[1] & 2 != 0,
                )
            };
            if [x, z] == origin {
                result[0] = visible;
            }
            result[1] |= visible;
        }
    }
    result
}

/// Map persistent scene identities to final per-view model batches.
pub fn register_calls(
    frame: &mut crate::render3d::Frame,
    view: u16,
    models: impl IntoIterator<Item = (usize, u64, usize)>,
    calls: &[Vec<usize>; 2],
    batch_base: usize,
) {
    use std::collections::{HashMap, HashSet};
    let members = models
        .into_iter()
        .map(|(id, tag, batch)| (id, (tag, batch + batch_base)))
        .collect::<HashMap<_, _>>();
    let admitted = calls
        .each_ref()
        .map(|pass| pass.iter().copied().collect::<HashSet<_>>());
    for (&id, &(tag, batch)) in &members {
        frame.model_admission(
            view,
            tag,
            batch,
            [admitted[0].contains(&id), admitted[1].contains(&id)],
        );
    }
    let ordered = calls.each_ref().map(|pass| {
        pass.iter()
            .filter_map(|id| members.get(id).copied())
            .collect::<Vec<_>>()
    });
    frame.model_first_order(view, members.values().copied(), ordered[0].clone());
    frame.model_later_order(view, members.values().copied(), ordered[1].clone());
}

/// Execute the relative ordinary-object and actor-body invocation stream.
/// Attached spot effects have a separate Java zone list and retain their own scheduling.
pub fn register_render_calls(
    frame: &mut crate::render3d::Frame,
    view: u16,
    ordinary: impl IntoIterator<Item = (usize, u64, usize)>,
    bodies: impl IntoIterator<Item = (u64, usize)>,
    actor_tags: &[Option<u64>],
    calls: &[Vec<crate::scene_object_layout::Call>; 2],
    batch_base: usize,
) {
    register_calls_with_spots(
        frame,
        view,
        ordinary,
        bodies,
        std::iter::empty(),
        actor_tags,
        &[],
        calls,
        batch_base,
    );
}
pub fn register_calls_with_spots(
    frame: &mut crate::render3d::Frame,
    view: u16,
    ordinary: impl IntoIterator<Item = (usize, u64, usize)>,
    bodies: impl IntoIterator<Item = (u64, usize)>,
    spots: impl IntoIterator<Item = ((u64, u8), (u64, usize))>,
    actor_tags: &[Option<u64>],
    spot_keys: &[(u64, u8)],
    calls: &[Vec<crate::scene_object_layout::Call>; 2],
    batch_base: usize,
) {
    register_calls_with_effects(
        frame,
        view,
        ordinary,
        bodies,
        spots,
        std::iter::empty(),
        actor_tags,
        spot_keys,
        calls,
        batch_base,
    );
}
pub fn register_calls_with_effects(
    frame: &mut crate::render3d::Frame,
    view: u16,
    ordinary: impl IntoIterator<Item = (usize, u64, usize)>,
    bodies: impl IntoIterator<Item = (u64, usize)>,
    spots: impl IntoIterator<Item = ((u64, u8), (u64, usize))>,
    effects: impl IntoIterator<Item = (usize, (u64, usize))>,
    actor_tags: &[Option<u64>],
    spot_keys: &[(u64, u8)],
    calls: &[Vec<crate::scene_object_layout::Call>; 2],
    batch_base: usize,
) {
    register_calls_with_children(
        frame,
        view,
        ordinary,
        bodies,
        spots,
        effects,
        actor_tags,
        spot_keys,
        &[],
        calls,
        batch_base,
    );
}
pub fn register_calls_with_children(
    frame: &mut crate::render3d::Frame,
    view: u16,
    ordinary: impl IntoIterator<Item = (usize, u64, usize)>,
    bodies: impl IntoIterator<Item = (u64, usize)>,
    spots: impl IntoIterator<Item = ((u64, u8), (u64, usize))>,
    effects: impl IntoIterator<Item = (usize, (u64, usize))>,
    actor_tags: &[Option<u64>],
    spot_keys: &[(u64, u8)],
    child_views: &[(usize, u16)],
    calls: &[Vec<crate::scene_object_layout::Call>; 2],
    batch_base: usize,
) {
    register_calls_with_boundaries(
        frame,
        view,
        ordinary,
        std::iter::empty(),
        bodies,
        spots,
        effects,
        actor_tags,
        spot_keys,
        child_views,
        calls,
        batch_base,
    );
}
pub fn register_calls_with_boundaries(
    frame: &mut crate::render3d::Frame,
    view: u16,
    ordinary: impl IntoIterator<Item = (usize, u64, usize)>,
    boundaries: impl IntoIterator<Item = (usize, u64, usize)>,
    bodies: impl IntoIterator<Item = (u64, usize)>,
    spots: impl IntoIterator<Item = ((u64, u8), (u64, usize))>,
    effects: impl IntoIterator<Item = (usize, (u64, usize))>,
    actor_tags: &[Option<u64>],
    spot_keys: &[(u64, u8)],
    child_views: &[(usize, u16)],
    calls: &[Vec<crate::scene_object_layout::Call>; 2],
    batch_base: usize,
) {
    register_calls_with_decorations(
        frame,
        view,
        ordinary,
        boundaries,
        std::iter::empty(),
        bodies,
        spots,
        effects,
        actor_tags,
        spot_keys,
        child_views,
        calls,
        batch_base,
    );
}
pub fn register_calls_with_decorations(
    frame: &mut crate::render3d::Frame,
    view: u16,
    ordinary: impl IntoIterator<Item = (usize, u64, usize)>,
    boundaries: impl IntoIterator<Item = (usize, u64, usize)>,
    decorations: impl IntoIterator<Item = (usize, u64, usize)>,
    bodies: impl IntoIterator<Item = (u64, usize)>,
    spots: impl IntoIterator<Item = ((u64, u8), (u64, usize))>,
    effects: impl IntoIterator<Item = (usize, (u64, usize))>,
    actor_tags: &[Option<u64>],
    spot_keys: &[(u64, u8)],
    child_views: &[(usize, u16)],
    calls: &[Vec<crate::scene_object_layout::Call>; 2],
    batch_base: usize,
) {
    register_calls_with_ground(
        frame,
        view,
        ordinary,
        boundaries,
        decorations,
        std::iter::empty(),
        bodies,
        spots,
        effects,
        actor_tags,
        spot_keys,
        child_views,
        calls,
        batch_base,
    );
}
pub fn register_calls_with_ground(
    frame: &mut crate::render3d::Frame,
    view: u16,
    ordinary: impl IntoIterator<Item = (usize, u64, usize)>,
    boundaries: impl IntoIterator<Item = (usize, u64, usize)>,
    decorations: impl IntoIterator<Item = (usize, u64, usize)>,
    ground: impl IntoIterator<Item = (usize, u64, usize)>,
    bodies: impl IntoIterator<Item = (u64, usize)>,
    spots: impl IntoIterator<Item = ((u64, u8), (u64, usize))>,
    effects: impl IntoIterator<Item = (usize, (u64, usize))>,
    actor_tags: &[Option<u64>],
    spot_keys: &[(u64, u8)],
    child_views: &[(usize, u16)],
    calls: &[Vec<crate::scene_object_layout::Call>; 2],
    batch_base: usize,
) {
    register_calls_with_items(
        frame,
        view,
        ordinary,
        boundaries,
        decorations,
        ground,
        std::iter::empty(),
        bodies,
        spots,
        effects,
        actor_tags,
        spot_keys,
        child_views,
        calls,
        batch_base,
    );
}
pub fn register_calls_with_items(
    frame: &mut crate::render3d::Frame,
    view: u16,
    ordinary: impl IntoIterator<Item = (usize, u64, usize)>,
    boundaries: impl IntoIterator<Item = (usize, u64, usize)>,
    decorations: impl IntoIterator<Item = (usize, u64, usize)>,
    ground: impl IntoIterator<Item = (usize, u64, usize)>,
    items: impl IntoIterator<Item = (usize, u64, usize)>,
    bodies: impl IntoIterator<Item = (u64, usize)>,
    spots: impl IntoIterator<Item = ((u64, u8), (u64, usize))>,
    effects: impl IntoIterator<Item = (usize, (u64, usize))>,
    actor_tags: &[Option<u64>],
    spot_keys: &[(u64, u8)],
    child_views: &[(usize, u16)],
    calls: &[Vec<crate::scene_object_layout::Call>; 2],
    batch_base: usize,
) {
    register_calls_with_terrain(
        frame,
        view,
        ordinary,
        boundaries,
        decorations,
        ground,
        items,
        None,
        bodies,
        spots,
        effects,
        actor_tags,
        spot_keys,
        child_views,
        calls,
        batch_base,
    );
}
pub fn register_calls_with_terrain(
    frame: &mut crate::render3d::Frame,
    view: u16,
    ordinary: impl IntoIterator<Item = (usize, u64, usize)>,
    boundaries: impl IntoIterator<Item = (usize, u64, usize)>,
    decorations: impl IntoIterator<Item = (usize, u64, usize)>,
    ground: impl IntoIterator<Item = (usize, u64, usize)>,
    items: impl IntoIterator<Item = (usize, u64, usize)>,
    terrain: Option<Vec<(usize, u64)>>,
    bodies: impl IntoIterator<Item = (u64, usize)>,
    spots: impl IntoIterator<Item = ((u64, u8), (u64, usize))>,
    effects: impl IntoIterator<Item = (usize, (u64, usize))>,
    actor_tags: &[Option<u64>],
    spot_keys: &[(u64, u8)],
    child_views: &[(usize, u16)],
    calls: &[Vec<crate::scene_object_layout::Call>; 2],
    batch_base: usize,
) {
    use crate::scene_object_layout::Call;
    use std::collections::{HashMap, HashSet};
    let mut groups = HashMap::<Call, Vec<(u64, usize)>>::new();
    for (id, tag, batch) in ordinary {
        groups
            .entry(Call::Ordinary(id))
            .or_default()
            .push((tag, batch + batch_base));
    }
    for (id, tag, batch) in boundaries {
        groups
            .entry(Call::Boundary(id))
            .or_default()
            .push((tag, batch + batch_base));
    }
    for (id, tag, batch) in decorations {
        groups
            .entry(Call::Decoration(id))
            .or_default()
            .push((tag, batch + batch_base));
    }
    for (id, tag, batch) in ground {
        groups
            .entry(Call::Ground(id))
            .or_default()
            .push((tag, batch + batch_base));
    }
    // Item geometry is generated with actors and already has global batch IDs.
    for (id, tag, batch) in items {
        groups.entry(Call::Item(id)).or_default().push((tag, batch));
    }
    let mut by_tag = HashMap::<u64, Vec<(u64, usize)>>::new();
    for (tag, batch) in bodies {
        by_tag.entry(tag).or_default().push((tag, batch));
    }
    for models in by_tag.values_mut() {
        models.sort_unstable();
        models.dedup();
    }
    for (index, tag) in actor_tags.iter().enumerate() {
        if let Some(models) = tag.and_then(|tag| by_tag.get(&tag)) {
            groups.insert(Call::Temporary(index), models.clone());
        }
    }
    for (index, model) in effects {
        groups
            .entry(Call::Temporary(index))
            .or_default()
            .push(model);
    }
    let mut by_spot = HashMap::<(u64, u8), Vec<(u64, usize)>>::new();
    for (key, model) in spots {
        by_spot.entry(key).or_default().push(model);
    }
    for models in by_spot.values_mut() {
        models.sort_unstable();
        models.dedup();
    }
    for (index, key) in spot_keys.iter().enumerate() {
        if let Some(models) = by_spot.get(key) {
            groups.insert(Call::ActorSpot(index), models.clone());
        }
    }
    let admitted = calls
        .each_ref()
        .map(|pass| pass.iter().copied().collect::<HashSet<_>>());
    for (call, models) in &groups {
        for &(tag, batch) in models {
            frame.model_admission(
                view,
                tag,
                batch,
                [admitted[0].contains(call), admitted[1].contains(call)],
            );
        }
    }
    if let Some(terrain) = terrain {
        use crate::scene_raster_order::{Call as RasterCall, Primitive};
        let terrain = terrain.into_iter().collect::<HashMap<_, _>>();
        let mut members = groups
            .values()
            .flatten()
            .map(|&(tag, batch)| Primitive::Model(view, tag, batch))
            .collect::<Vec<_>>();
        members.extend(terrain.values().map(|&tag| Primitive::Terrain(view, tag)));
        let children = child_views.iter().copied().collect::<HashMap<_, _>>();
        let program = calls[0]
            .iter()
            .flat_map(|call| {
                if let Call::Terrain(id) = call {
                    return terrain
                        .get(id)
                        .map(|&tag| RasterCall::Primitive(Primitive::Terrain(view, tag)))
                        .into_iter()
                        .collect::<Vec<_>>();
                }
                if let Call::Temporary(id) = call {
                    if let Some(&child) = children.get(id) {
                        return vec![RasterCall::Child(child)];
                    }
                }
                groups
                    .get(call)
                    .into_iter()
                    .flatten()
                    .map(|&(tag, batch)| RasterCall::Primitive(Primitive::Model(view, tag, batch)))
                    .collect()
            })
            .collect();
        frame.scene_view_raster_order(view, members, program, children.values().copied());
    }
    let members = groups.values().flatten().copied().collect::<Vec<_>>();
    let ordered = calls.each_ref().map(|pass| {
        pass.iter()
            .filter_map(|call| groups.get(call))
            .flatten()
            .copied()
            .collect::<Vec<_>>()
    });
    if view == 0 {
        use crate::render3d::ModelSceneCall;
        let children = child_views.iter().copied().collect::<HashMap<_, _>>();
        let program = calls.each_ref().map(|pass| {
            pass.iter()
                .flat_map(|call| {
                    if let Call::Temporary(index) = call {
                        if let Some(&child) = children.get(index) {
                            return vec![ModelSceneCall::Child(child)];
                        }
                    }
                    groups
                        .get(call)
                        .into_iter()
                        .flatten()
                        .map(|&(tag, batch)| ModelSceneCall::Model(tag, batch))
                        .collect::<Vec<_>>()
                })
                .collect()
        });
        frame.root_model_call_order(children.values().copied(), program);
    } else {
        assert!(
            child_views.is_empty(),
            "only root-owned child scenes are prepared"
        );
    }
    frame.model_first_order(view, members.iter().copied(), ordered[0].clone());
    frame.model_later_order(view, members, ordered[1].clone());
}
pub fn actor_spot_batches<'a>(
    triangles: impl IntoIterator<Item = &'a crate::render3d::Triangle>,
    view: u16,
) -> Vec<((u64, u8), (u64, usize))> {
    let mut batches = triangles
        .into_iter()
        .filter_map(|triangle| {
            let order = triangle.model_order?;
            let owner = order.actor_spot?;
            let slot = u8::try_from(order.source_ordinal?).ok()?;
            (triangle.view == view).then_some(((owner, slot), (triangle.tag, order.batch)))
        })
        .collect::<Vec<_>>();
    batches.sort_unstable();
    batches.dedup();
    batches
}
/// Body batches include attachments sharing their body identity, excluding actor spots.
pub fn actor_body_batches<'a>(
    triangles: impl IntoIterator<Item = &'a crate::render3d::Triangle>,
    view: u16,
) -> Vec<(u64, usize)> {
    let mut batches = triangles
        .into_iter()
        .filter_map(|triangle| {
            let order = triangle.model_order?;
            (triangle.view == view
                && matches!(triangle.tag >> 60, 2 | 4)
                && order.actor_spot.is_none())
            .then_some((triangle.tag, order.batch))
        })
        .collect::<Vec<_>>();
    batches.sort_unstable();
    batches.dedup();
    batches
}

pub fn temporary_effect_batches<'a>(
    triangles: impl IntoIterator<Item = &'a crate::render3d::Triangle>,
    view: u16,
) -> Vec<(usize, (u64, usize))> {
    let mut batches = triangles
        .into_iter()
        .filter_map(|triangle| {
            let order = triangle.model_order?;
            let id = order.temporary_id?;
            (triangle.view == view).then_some((id, (triangle.tag, order.batch)))
        })
        .collect::<Vec<_>>();
    batches.sort_unstable();
    batches.dedup();
    batches
}
