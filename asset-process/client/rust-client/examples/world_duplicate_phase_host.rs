use gielinor_client::{
    actor_sequence, animation::Sequence, animation_readiness::Readiness, buffer::Reader,
    client_state::ClientState, disk_cache::DiskCache, game_packet, npc_info::Npc, protocol::Packet,
};
use std::collections::HashSet;
fn action(frame: u16, sequence: &Sequence) -> actor_sequence::State {
    actor_sequence::State {
        id: sequence.id as i32,
        frame: frame as i32 - 1,
        elapsed: sequence.lengths[frame as usize - 1] as u32,
        ..Default::default()
    }
}
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
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let sequences = Sequence::load_all(&mut cache)?;
    let mut client = ClientState::load(cache)?;
    client
        .npcs
        .insert(1, gielinor_client::npc::Definition::decode(1, &[0])?);
    let mut candidates = sequences
        .values()
        .flat_map(|s| {
            s.sounds.iter().filter_map(move |(&f, sounds)| {
                (f > 0
                    && s.cached_animation.is_none()
                    && f as usize + 1 < s.frames.len()
                    && sounds.len() == 1
                    && sounds[0].range > 0
                    && sounds[0].loops > 0)
                    .then_some((s.id, f, sounds[0].id))
            })
        })
        .collect::<Vec<_>>();
    candidates.sort();
    let mut ids = HashSet::new();
    let choices = candidates
        .into_iter()
        .filter(|v| ids.insert(v.2))
        .take(10)
        .collect::<Vec<_>>();
    assert_eq!(choices.len(), 10);
    let mut readiness = Readiness::default();
    for &(id, _, _) in &choices {
        for _ in 0..128 {
            if readiness.ready(&mut client.scripts.source.cache, &sequences[&id]) {
                break;
            }
            client.scripts.source.cache.wait_for_recovery(true);
        }
        assert!(readiness.ready(&mut client.scripts.source.cache, &sequences[&id]));
    }
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/world-duplicate-phase-jvm.bin"
    ));
    let count = r.u32()?;
    assert_eq!(count, 16);
    for case in 0..count {
        client.world_entities.clear();
        client.detached_world_entities.clear();
        client.active_world = 0;
        client.view_registry = Default::default();
        client.npc_state = Default::default();
        client.player_state = None;
        client.audio = Default::default();
        client.world_base = [0; 2];
        client.actor_cycle = 10;
        let id = r.u32()? as u16;
        let _order = r.u32()?;
        for _ in 0..r.u32()? {
            let opcode = r.u32()? as u16;
            let n = r.u32()? as usize;
            let payload = r.bytes(n)?.to_vec();
            client.apply(game_packet::decode(&Packet { opcode, payload })?.unwrap())?;
        }
        let _dimensions: [i32; 3] = std::array::from_fn(|_| r.u32().unwrap() as i32);
        let npc_order = r.u32()?;
        let mut expected = Vec::new();
        for _ in 0..r.u32()? {
            let slot = r.u32()? as usize;
            let index = r.u32()?;
            let view = match slot {
                0 => 0,
                3 => id + 16,
                4 => 7,
                _ => id,
            };
            expected.push((
                choices[slot * 2 + usize::from(index == 9)].2 as u32,
                Some(view),
            ));
        }
        for world in client.world_entities.values_mut() {
            for e in world.entities.values_mut() {
                e.animation.reset();
                e.default_animation.reset();
            }
        }
        for slot in 0..5 {
            let mut state = gielinor_client::npc_info::State::default();
            for index in if npc_order == 0 { [1, 9] } else { [9, 1] } {
                let mut actor = npc();
                actor.path.position = [1600; 2];
                actor.path.fine = [1600.; 2];
                actor.path.tiles[0] = [12; 2];
                let (sequence, frame, _) = choices[slot * 2 + usize::from(index == 9)];
                actor.sequence = action(frame, &sequences[&sequence]);
                state.indices.push(index);
                state.npcs.insert(index, actor);
            }
            if slot == 0 {
                client.npc_state = state;
            } else {
                let (parent, target) = match slot {
                    2 => (7, id),
                    3 => (0, id + 16),
                    4 => (0, 7),
                    _ => (0, id),
                };
                client
                    .world_entities
                    .get_mut(&parent)
                    .unwrap()
                    .entities
                    .get_mut(&target)
                    .unwrap()
                    .npcs = state;
            }
        }
        client.tick_actor_sequences()?;
        assert_eq!(
            client.audio.pending_effects().collect::<Vec<_>>(),
            expected,
            "case {case} actual duplicate-instance NPC sound enqueue order"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original ht.bt duplicate-instance phase traces passed actual-cache ClientState sound enqueue order with ten distinct cache sounds; choices {choices:?}; no playback");
    Ok(())
}
