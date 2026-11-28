//! nl.al endpoint selection: actor membership, tile fallback and root coordinates.
use crate::world_entity::State;

#[derive(Clone, Copy, Debug)]
pub struct View {
    pub id: u16,
    pub base: [i32; 2],
    pub dimensions: [i32; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub view: u16,
    /// Root-local fine units. Height is sampled in the selected view at this point.
    pub fine: [i32; 2],
}

/// `views` follows cc iteration order; actor lookup returns view-local fine units.
fn resolve_impl(
    views: &[View],
    entities: Option<&State>,
    tile: [i32; 2],
    target: i32,
    mut actor: impl FnMut(usize, i32) -> Option<[i32; 2]>,
    zero_fallback: bool,
) -> FloatPoint {
    let root_slot = views
        .iter()
        .position(|v| v.id == 0)
        .expect("root world view");
    let root = &views[root_slot];
    let mut selected = None;
    if target != 0 {
        // NPCs search every view. Players prefer child copies, then root fallback.
        let slot = views
            .iter()
            .enumerate()
            .find(|&(slot, v)| (target > 0 || v.id != 0) && actor(slot, target).is_some())
            .map_or(root_slot, |(slot, _)| slot);
        let view = &views[slot];
        if let Some(fine) = actor(slot, target) {
            selected = Some((view, fine));
        }
    }
    if let Some((view, fine)) = selected {
        let fine = entities
            .and_then(|s| s.entities.get(&view.id))
            .map_or(fine, |e| {
                let offsets = e.definition.as_ref().map_or([0; 2], |d| d.scene_offset);
                let x = fine[0].wrapping_sub(e.dimensions[0] * 64 + offsets[0]);
                let z = fine[1].wrapping_sub(e.dimensions[1] * 64 + offsets[1]);
                // GameActor.eq uses double trigonometry, unlike tile endpoints' float matrix.
                let angle = -(e.pose.angle as f64) * std::f64::consts::PI / 1024.0;
                let cos = libm::cos(angle);
                let sin = libm::sin(angle);
                [
                    ((cos * x as f64 - sin * z as f64) as i32).wrapping_add(e.pose.x),
                    ((z as f64 * cos + sin * x as f64) as i32).wrapping_add(e.pose.z),
                ]
            });
        if !zero_fallback || fine != [0, 0] {
            return FloatPoint {
                view: view.id,
                fine: fine.map(|n| n as f32),
            };
        }
    }
    let view = views
        .iter()
        .find(|v| {
            v.id != 0
                && (0..2).all(|i| {
                    tile[i] >= v.base[i] && tile[i] < v.base[i].wrapping_add(v.dimensions[i])
                })
        })
        .unwrap_or(root);
    let fine = std::array::from_fn(|i| {
        tile[i]
            .wrapping_sub(view.base[i])
            .wrapping_mul(128)
            .wrapping_add(64)
    });
    let fine =
        entities
            .and_then(|s| s.entities.get(&view.id))
            .map_or(fine.map(|n| n as f32), |e| {
                let p = e.transform_horizontal(fine);
                [p[0], p[2]]
            });
    FloatPoint {
        view: view.id,
        fine,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatPoint {
    pub view: u16,
    pub fine: [f32; 2],
}
pub fn resolve(
    views: &[View],
    entities: Option<&State>,
    tile: [i32; 2],
    target: i32,
    mut actor: impl FnMut(u16, i32) -> Option<[i32; 2]>,
) -> Point {
    let p = resolve_impl(
        views,
        entities,
        tile,
        target,
        |slot, target| actor(views[slot].id, target),
        false,
    );
    Point {
        view: p.view,
        fine: p.fine.map(|n| n as i32),
    }
}
/// ca.hp retains tile transform fractions and falls back for a zero actor point.
pub fn resolve_scripted(
    views: &[View],
    entities: Option<&State>,
    tile: [i32; 2],
    target: i32,
    mut actor: impl FnMut(u16, i32) -> Option<[i32; 2]>,
) -> FloatPoint {
    resolve_impl(
        views,
        entities,
        tile,
        target,
        |slot, target| actor(views[slot].id, target),
        true,
    )
}

/// Preserve the original registry node while resolving actor and tile endpoints.
pub fn resolve_registered(
    root: View,
    registry: &crate::world_view::Registry,
    players: Option<&crate::player_info::State>,
    root_npcs: &crate::npc_info::State,
    worlds: &std::collections::HashMap<u16, State>,
    detached: &[State],
    tile: [i32; 2],
    target: i32,
    scripted: bool,
) -> FloatPoint {
    let ids = registry.ordered();
    let descriptors =
        crate::player_info::instance_descriptors(&ids, ids.len(), root.base, worlds, detached);
    let views = descriptors
        .iter()
        .map(|&(id, _, base, dimensions)| {
            if id == 0 {
                root
            } else {
                View {
                    id,
                    base,
                    dimensions,
                }
            }
        })
        .collect::<Vec<_>>();
    let actor = |slot: usize, target: i32| {
        let (id, generation, base, _) = descriptors[slot];
        let position = if target > 0 {
            let index = u16::try_from(target - 1).ok()?;
            let npcs = if id == 0 {
                root_npcs
            } else {
                &worlds
                    .values()
                    .chain(detached.iter())
                    .filter_map(|world| world.entities.get(&id))
                    .find(|e| e.registered && e.spawn_generation == generation)?
                    .npcs
            };
            npcs.npcs.get(&index)?.path.position
        } else {
            let index = u16::try_from(target.wrapping_neg().wrapping_sub(1)).ok()?;
            players?
                .instance_actor(id, generation, index)?
                .path
                .position
        };
        Some(std::array::from_fn(|i| {
            position[i].wrapping_sub(base[i].wrapping_mul(128))
        }))
    };
    let mut resolved = resolve_impl(&views, worlds.get(&0), tile, target, actor, scripted);
    if !scripted {
        resolved.fine = resolved.fine.map(|n| n as i32 as f32);
    }
    resolved
}
