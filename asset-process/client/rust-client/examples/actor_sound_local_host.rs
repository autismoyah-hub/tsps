use gielinor_client::{
    animation::Sequence,
    client_state::ClientState,
    disk_cache::DiskCache,
    npc_info::Npc,
    player_info,
    world_entity::{Entity, State},
    Result,
};
fn npc() -> Npc {
    Npc {
        visual: Default::default(),
        path: Default::default(),
        sequence: gielinor_client::actor_sequence::State {
            id: -1,
            frame: 11,
            ..Default::default()
        },
        pose: Default::default(),
        idle_fallback: Default::default(),
        spots: Default::default(),
        overlays: Default::default(),
        turn: Default::default(),
        facing: None,
        freezes: Default::default(),
        exact: None,
        id: 1,
        position: [0; 2],
        angle: 0,
        spawn_cycle: None,
        steps: vec![],
        jump: false,
        blocks: vec![],
        appearance: Default::default(),
        animations: [-1; 15],
    }
}
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let sequences = Sequence::load_all(&mut cache)?;
    let mut client = ClientState::load(cache)?;
    let mut candidates = sequences
        .values()
        .flat_map(|s| {
            s.sounds.iter().filter_map(move |(&frame, sounds)| {
                (frame > 0
                    && sounds.len() == 1
                    && sounds[0].range == 0
                    && sounds[0].loops > 0
                    && s.cached_animation.is_none()
                    && frame as usize + 1 < s.frames.len())
                .then_some((s.id, frame))
            })
        })
        .collect::<Vec<_>>();
    candidates.sort();
    let (sequence, frame) = *candidates
        .first()
        .expect("cache sequence with single zero-range frame sound");
    for world in [0u16, 7, 2] {
        for player in [false, true] {
            for index in [42u16, 41] {
                client.audio = Default::default();
                client.world_entities.clear();
                client.detached_world_entities.clear();
                client.npc_state = Default::default();
                client.world_base = [3200, 3200];
                client.player_state = Some(player_info::State::initialize(&[0; 4608], 42)?.0);
                let base = if world == 0 {
                    client.world_base
                } else {
                    [100, 200]
                };
                let mut actor = npc();
                actor.sequence.id = sequence as i32;
                actor.sequence.frame = frame as i32 - 1;
                actor.sequence.elapsed = sequences[&sequence].lengths[frame as usize - 1] as u32;
                actor.path.tiles[0] = [base[0] + 12, base[1] + 20];
                actor.path.position = actor.path.tiles[0].map(|p| p * 128 + 64);
                actor.path.fine = actor.path.position.map(|p| p as f32);
                if world == 0 {
                    client.npc_state.indices.push(index);
                    client.npc_state.npcs.insert(index, actor);
                } else {
                    let mut e = Entity::default();
                    e.dimensions = [24, 40];
                    e.rebuild = Some(gielinor_client::rebuild::WorldEntity {
                        base,
                        dimensions: [24, 40],
                        chunks: vec![],
                        regions: vec![],
                    });
                    e.npcs.indices.push(index);
                    e.npcs.npcs.insert(index, actor);
                    let mut root = State::default();
                    root.visible.push(world);
                    root.entities.insert(world, e);
                    client.world_entities.insert(0, root);
                }
                if player {
                    let npc = client.view_npc(world, index).unwrap().clone();
                    let state = client.player_state.as_mut().unwrap();
                    if !state.high.contains(&index) { state.high.push(index); }
                    let mut actor = state.players[42].as_ref().unwrap().clone();
                    actor.path = npc.path;
                    actor.sequence = npc.sequence;
                    actor.position = [0, base[0] + 12, base[1] + 20];
                    if world == 0 {
                        state.players[index as usize] = Some(actor);
                        if !state.high.contains(&index) {
                            state.high.push(index);
                        }
                        client.npc_state = Default::default();
                    } else {
                        state.views.insert(
                            world,
                            player_info::View {
                                base,
                                dimensions: [24, 40],
                                players: std::collections::BTreeMap::from([(index, actor)]),
                            },
                        );
                        client
                            .world_entities
                            .get_mut(&0)
                            .unwrap()
                            .entities
                            .get_mut(&world)
                            .unwrap()
                            .npcs = Default::default();
                    }
                }
                let mut advanced = false;
                for _ in 0..128 {
                    client.tick_actor_sequences()?;
                    let current = if player {
                        client
                            .player_state
                            .as_ref()
                            .unwrap()
                            .view_actor(world, index)
                            .unwrap()
                            .sequence
                            .frame
                    } else {
                        client.view_npc(world, index).unwrap().sequence.frame
                    };
                    if current >= frame as i32 {
                        advanced = true;
                        break;
                    }
                }
                assert!(advanced, "world{world} index{index} sound frame reached");
                assert_eq!(
                    client.audio.pending_count(),
                    usize::from(index == 42),
                    "world{world} player{player} index{index} zero-range local admission"
                );
            }
        }
    }
    println!("12 actual-cache NPC/player local-index admissions across root and child worlds match GameClient.nc, sequence{sequence} frame{frame}");
    Ok(())
}
