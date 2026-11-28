//! Registered target identity and coordinates for one actor-view simulation phase.
use crate::{actor_turn, npc_info, player_info, world_entity, world_view};
use std::collections::{HashMap, HashSet};
type Key = (u16, i32);
#[derive(Clone)]
enum Position {
    Same([i32; 2]),
    Transformed { point: [f32; 2], base: [i32; 2] },
}
#[derive(Clone)]
pub struct Targets {
    positions: HashMap<Key, Position>,
    live: Option<HashSet<Key>>,
}
impl Targets {
    pub fn local(positions: &HashMap<Key, [i32; 2]>) -> Self {
        Self {
            positions: positions
                .iter()
                .map(|(&k, &v)| (k, Position::Same(v)))
                .collect(),
            live: None,
        }
    }
    pub fn update(&mut self, kind: u16, index: u16, position: [i32; 2]) {
        let key = (kind, index as i32);
        if self.live.as_ref().is_none_or(|live| live.contains(&key)) {
            self.positions.insert(key, Position::Same(position));
        }
    }
    pub fn angle(&self, kind: u16, index: i32, observer: [i32; 2]) -> Option<Option<u16>> {
        self.positions
            .get(&(kind, index))
            .map(|target| match target {
                Position::Same(point) => actor_turn::toward(observer, *point),
                Position::Transformed { point, base } => {
                    // hz.br truncates the float difference, not the target coordinate.
                    let delta = std::array::from_fn(|i| {
                        (observer[i].wrapping_sub(base[i].wrapping_mul(128)) as f32 - point[i])
                            as i32
                    });
                    actor_turn::toward(delta, [0; 2])
                }
            })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Observer {
    pub view: u16,
    pub base: [i32; 2],
    pub generation: u64,
}
pub fn collect(
    observer: u16,
    root_base: [i32; 2],
    registry: &world_view::Registry,
    root_npcs: &npc_info::State,
    players: Option<&player_info::State>,
    worlds: &HashMap<u16, world_entity::State>,
    detached: &[world_entity::State],
) -> Targets {
    collect_inner(
        observer, None, root_base, registry, root_npcs, players, worlds, detached,
    )
}
pub fn collect_instance(
    observer: Observer,
    root_base: [i32; 2],
    registry: &world_view::Registry,
    root_npcs: &npc_info::State,
    players: Option<&player_info::State>,
    worlds: &HashMap<u16, world_entity::State>,
    detached: &[world_entity::State],
) -> Targets {
    collect_inner(
        observer.view,
        Some(observer),
        root_base,
        registry,
        root_npcs,
        players,
        worlds,
        detached,
    )
}
fn collect_inner(
    observer: u16,
    instance: Option<Observer>,
    root_base: [i32; 2],
    registry: &world_view::Registry,
    root_npcs: &npc_info::State,
    players: Option<&player_info::State>,
    worlds: &HashMap<u16, world_entity::State>,
    detached: &[world_entity::State],
) -> Targets {
    let entity = |id| world_entity::registered_entity(id, worlds, detached);
    let base = |id| {
        if id == 0 {
            root_base
        } else {
            entity(id)
                .and_then(|e| e.rebuild.as_ref())
                .map_or([0; 2], |r| r.base)
        }
    };
    let mut chosen: HashMap<Key, (u16, [i32; 2], [i32; 2], Option<u64>)> = HashMap::new();
    let mut views = registry.ordered();
    if !views.contains(&observer) {
        views.push(observer);
    }
    // ir.ae visits every registered instance, including equal IDs. Different
    // IDs in the same bucket can interleave with duplicate registrations.
    let registered_count = registry.ordered().len();
    let mut npc_views = Vec::new();
    for (rank, &view) in views.iter().enumerate() {
        let bucket = if rank < registered_count {
            (view & 15) as usize
        } else {
            16
        };
        if view == 0 {
            npc_views.push((bucket, 0, rank, view, root_base, root_npcs));
        } else {
            for candidate in worlds
                .values()
                .chain(detached.iter())
                .filter_map(|world| world.entities.get(&view))
                .filter(|e| e.registered)
            {
                let target_base = candidate.rebuild.as_ref().map_or([0; 2], |r| r.base);
                npc_views.push((
                    bucket,
                    candidate.spawn_generation,
                    rank,
                    view,
                    target_base,
                    &candidate.npcs,
                ));
            }
        }
    }
    npc_views.sort_by_key(|&(bucket, generation, rank, ..)| (bucket, generation, rank));
    for &(_, generation, _, view, target_base, npcs) in &npc_views {
        for (&id, npc) in &npcs.npcs {
            chosen.entry((1, id as i32)).or_insert((
                view,
                target_base,
                npc.path.position,
                Some(generation),
            ));
        }
    }
    // mt.af prefers non-root instances in registry traversal order.
    if let Some(players) = players {
        let mut player_views=Vec::new();
        for &(bucket,generation,rank,view,_,_) in &npc_views {
            if view==0 {continue;}
            if let Some(state)=players.instance_view(view,generation) {
                player_views.push((bucket,generation,rank,view,state));
            }
        }
        // Legacy hand-assembled callers may supply player views without entity metadata.
        for (rank,&view) in views.iter().enumerate().filter(|&(_,id)|*id!=0) {
            if npc_views.iter().any(|entry|entry.3==view) {continue;}
            if let Some(state)=players.views.get(&view) {
                let bucket=if rank<registered_count {(view&15) as usize}else{16};
                player_views.push((bucket,0,rank,view,state));
            }
        }
        player_views.sort_by_key(|&(bucket,generation,rank,..)|(bucket,generation,rank));
        for (_,generation,_,view,state) in player_views {
            for (&id,p) in &state.players {
                chosen.entry((2,id as i32)).or_insert((view,state.base,p.path.position,(generation!=0).then_some(generation)));
            }
        }
        for (id,p) in players.root_iter() {
            chosen.entry((2,id as i32)).or_insert((0,root_base,p.path.position,None));
        }
    }
    if let Some(root) = worlds.get(&0) {
        for (&id, entity) in &root.entities {
            let position = [
                entity.pose.x.wrapping_add(root_base[0].wrapping_mul(128)),
                entity.pose.z.wrapping_add(root_base[1].wrapping_mul(128)),
            ];
            chosen.insert((3, id as i32), (0, root_base, position, None));
        }
    }
    let observer_base = instance.map_or_else(
        || {
            players
                .and_then(|p| p.views.get(&observer))
                .map_or_else(|| base(observer), |v| v.base)
        },
        |o| o.base,
    );
    let observer_generation = instance.map_or_else(
        || {
            if observer == 0 {
                0
            } else {
                entity(observer).map_or(0, |e| e.spawn_generation)
            }
        },
        |o| o.generation,
    );
    let root_entities = worlds.get(&0);
    let mut result = Targets {
        positions: HashMap::new(),
        live: Some(HashSet::new()),
    };
    for (key, (view, target_base, position, generation)) in chosen {
        if view == observer {
            if generation.is_some_and(|g| g != observer_generation) {
                // Equal numeric IDs bypass world projection in hz.br, but each
                // actor still uses its own view-local coordinates. Updating a
                // different instance must not replace the selected target.
                let point = std::array::from_fn(|i| {
                    position[i].wrapping_sub(target_base[i].wrapping_mul(128)) as f32
                });
                result.positions.insert(
                    key,
                    Position::Transformed {
                        point,
                        base: observer_base,
                    },
                );
            } else {
                result.positions.insert(key, Position::Same(position));
                result.live.as_mut().unwrap().insert(key);
            }
            continue;
        }
        let fine =
            std::array::from_fn(|i| position[i].wrapping_sub(target_base[i].wrapping_mul(128)));
        let point = if view == 0 {
            [fine[0] as f32, fine[1] as f32]
        } else {
            let Some(e) = root_entities.and_then(|w| w.entities.get(&view)) else {
                continue;
            };
            let p = e.transform_horizontal(fine);
            [p[0], p[2]]
        };
        let point = if observer == 0 {
            point
        } else {
            let Some(e) = root_entities.and_then(|w| w.entities.get(&observer)) else {
                continue;
            };
            let p = e.inverse_horizontal(point.map(|v| v as i32));
            [p[0], p[2]]
        };
        result.positions.insert(
            key,
            Position::Transformed {
                point,
                base: observer_base,
            },
        );
    }
    result
}
