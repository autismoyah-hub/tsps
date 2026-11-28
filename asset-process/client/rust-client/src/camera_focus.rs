//! Original ny.hz target ownership and oe.bj root-space camera projection.
use crate::{projectile_world::View, world_entity::State};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Focus {
    pub view: u16,
    pub fine: [i32; 2],
    pub plane: u8,
}

/// Actor positions are local to their view. Views follow the cc registry order.
pub fn resolve_local(
    views: &[View],
    entities: Option<&State>,
    root_plane: u8,
    kind: u8,
    id: i32,
    mut actor: impl FnMut(u16, u8, u16) -> Option<([i32; 2], u8)>,
) -> Option<Focus> {
    let root = views.iter().find(|v| v.id == 0)?;
    let (view, fine, plane) = match kind {
        0 | 1 => {
            let index = u16::try_from(id).ok()?;
            // Players prefer any child copy; NPCs use the first registered copy.
            let view = views
                .iter()
                .find(|v| (kind == 1 || v.id != 0) && actor(v.id, kind, index).is_some())
                .unwrap_or(root);
            let (fine, plane) = actor(view.id, kind, index)?;
            (view, fine, plane)
        }
        2 => {
            let entity = entities?.entities.get(&u16::try_from(id).ok()?)?;
            (root, [entity.pose.x, entity.pose.z], root_plane)
        }
        3 => {
            let tile = [(id >> 14) & 16383, id & 16383];
            let view = views
                .iter()
                .find(|v| {
                    v.id != 0
                        && (0..2).all(|i| {
                            tile[i] >= v.base[i]
                                && tile[i] < v.base[i].wrapping_add(v.dimensions[i])
                        })
                })
                .unwrap_or(root);
            let fine = std::array::from_fn(|i| {
                tile[i]
                    .wrapping_sub(view.base[i])
                    .wrapping_mul(128)
                    .wrapping_add(64)
            });
            // xf.af does not update its plane field (initialized to zero).
            (view, fine, 0)
        }
        _ => return None,
    };
    Some(Focus {
        view: view.id,
        fine,
        plane,
    })
}

impl Focus {
    pub fn projected(self, entities: Option<&State>) -> Self {
        let mut projected = self;
        if self.view != 0 {
            if let Some(entity) = entities.and_then(|s| s.entities.get(&self.view)) {
                let point = entity.transform_horizontal(self.fine);
                projected.fine = [point[0] as i32, point[2] as i32];
            }
        }
        projected
    }
    /// tt.cd samples the target in its own scene, then adds the root-ground
    /// height under its owning entity. Horizontal projection must happen later.
    pub fn ground_height(
        self,
        entities: Option<&State>,
        root_plane: u8,
        mut height: impl FnMut(u16, u8, [i32; 2]) -> i32,
    ) -> i32 {
        let local = height(self.view, self.plane, self.fine);
        if let Some(entity) = entities.and_then(|s| s.entities.get(&self.view)) {
            local.wrapping_add(height(0, root_plane, [entity.pose.x, entity.pose.z]))
        } else {
            local
        }
    }
}
pub fn resolve(
    views: &[View],
    entities: Option<&State>,
    root_plane: u8,
    kind: u8,
    id: i32,
    actor: impl FnMut(u16, u8, u16) -> Option<([i32; 2], u8)>,
) -> Option<Focus> {
    Some(resolve_local(views, entities, root_plane, kind, id, actor)?.projected(entities))
}

/// ny.hz -> ir.ae preserves registry instance identity for NPC position;
/// GameActor.ag separately resolves the plane through the oldest global owner.
pub fn resolve_registered_npc(
    root_base: [i32; 2],
    root_plane: u8,
    registry: &crate::world_view::Registry,
    root_npcs: &crate::npc_info::State,
    worlds: &std::collections::HashMap<u16, State>,
    detached: &[State],
    index: u16,
) -> Option<Focus> {
    let mut instances = Vec::new();
    for (rank, id) in registry.ordered().into_iter().enumerate() {
        if id == 0 {
            instances.push((0, 0, rank, id, root_base, root_npcs));
        } else {
            for e in worlds
                .values()
                .chain(detached.iter())
                .filter_map(|w| w.entities.get(&id))
                .filter(|e| e.registered)
            {
                let base = e.rebuild.as_ref().map_or([0; 2], |r| r.base);
                instances.push((id & 15, e.spawn_generation, rank, id, base, &e.npcs));
            }
        }
    }
    instances.sort_by_key(|&(bucket, generation, rank, ..)| (bucket, generation, rank));
    let (_, _, _, view, base, npcs) = instances
        .into_iter()
        .find(|v| v.5.npcs.contains_key(&index))?;
    let position = npcs.npcs.get(&index)?.path.position;
    let plane = if view == 0 {
        root_plane
    } else {
        crate::world_entity::registered_entity(view, worlds, detached)
            .map_or(root_plane, |e| e.plane)
    };
    Some(Focus {
        view,
        fine: std::array::from_fn(|i| position[i].wrapping_sub(base[i].wrapping_mul(128))),
        plane,
    })
}

/// cc.ap searches every registered child view before falling back to root.
pub fn resolve_registered_tile(
    root_base: [i32; 2],
    registry: &crate::world_view::Registry,
    worlds: &std::collections::HashMap<u16, State>,
    detached: &[State],
    packed: i32,
) -> Focus {
    let tile = [(packed >> 14) & 16383, packed & 16383];
    let mut instances = Vec::new();
    for (rank, id) in registry
        .ordered()
        .into_iter()
        .enumerate()
        .filter(|(_, id)| *id != 0)
    {
        for e in worlds
            .values()
            .chain(detached.iter())
            .filter_map(|w| w.entities.get(&id))
            .filter(|e| e.registered)
        {
            let base = e.rebuild.as_ref().map_or([0; 2], |r| r.base);
            instances.push((
                id & 15,
                e.spawn_generation,
                rank,
                View {
                    id,
                    base,
                    dimensions: e.dimensions,
                },
            ));
        }
    }
    instances.sort_by_key(|&(bucket, generation, rank, _)| (bucket, generation, rank));
    let view = instances
        .into_iter()
        .map(|(_, _, _, view)| view)
        .find(|view| {
            (0..2).all(|i| {
                tile[i] >= view.base[i] && tile[i] < view.base[i].wrapping_add(view.dimensions[i])
            })
        })
        .unwrap_or(View {
            id: 0,
            base: root_base,
            dimensions: [104; 2],
        });
    Focus {
        view: view.id,
        fine: std::array::from_fn(|i| {
            tile[i]
                .wrapping_sub(view.base[i])
                .wrapping_mul(128)
                .wrapping_add(64)
        }),
        plane: 0,
    }
}

/// ny.hz -> mt.af selects the first non-root player copy in registry order.
/// Player.ag reads that copy's plane, unlike the NPC global-owner lookup.
pub fn resolve_registered_player(
    root_base: [i32; 2],
    registry: &crate::world_view::Registry,
    players: &crate::player_info::State,
    worlds: &std::collections::HashMap<u16, State>,
    detached: &[State],
    index: u16,
) -> Option<Focus> {
    let ids = registry.ordered();
    let descriptors =
        crate::player_info::instance_descriptors(&ids, ids.len(), root_base, worlds, detached);
    let mut candidates = Vec::new();
    for (rank, &id) in ids.iter().enumerate().filter(|&(_, id)| *id != 0) {
        let mut registered = false;
        for &(view, generation, _, _) in descriptors.iter().filter(|v| v.0 == id) {
            registered = true;
            if let Some(state) = players.instance_view(view, generation) {
                candidates.push((view & 15, generation, rank, view, state));
            }
        }
        // Legacy callers may construct an unqualified view without entity metadata.
        if !registered {
            if let Some(state) = players.views.get(&id) {
                candidates.push((id & 15, 0, rank, id, state));
            }
        }
    }
    candidates.sort_by_key(|&(bucket, generation, rank, ..)| (bucket, generation, rank));
    for (_, _, _, view, state) in candidates {
        if let Some(player) = state.players.get(&index) {
            return Some(Focus {
                view,
                fine: std::array::from_fn(|i| {
                    player.path.position[i].wrapping_sub(state.base[i].wrapping_mul(128))
                }),
                plane: player.position[0] as u8,
            });
        }
    }
    let player = players.root_actor(index)?;
    Some(Focus {
        view: 0,
        fine: std::array::from_fn(|i| {
            player.path.position[i].wrapping_sub(root_base[i].wrapping_mul(128))
        }),
        plane: player.position[0] as u8,
    })
}
