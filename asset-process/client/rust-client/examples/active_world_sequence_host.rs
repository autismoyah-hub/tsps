use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    game_packet::{self},
    npc_info::Npc,
    protocol::Packet,
    world_entity::{Entity, State},
    Result,
};
fn npc(marker: i32) -> Npc {
    Npc {
        visual: Default::default(),
        path: Default::default(),
        sequence: gielinor_client::actor_sequence::State {
            id: marker,
            frame: 11,
            delay: 5,
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
    let mut client = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/active-world-sequence-jvm.bin"
    ));
    assert_eq!(r.u32()?, 32);
    for case in 0..32 {
        let selected = r.u32()? as u16;
        let marker = r.u32()? as i32;
        let plane = r.u32()? as u8;
        let expected: Vec<_> = (0..3)
            .map(|_| Ok((r.u32()? as i32, r.u32()? as i32, r.u32()? as i32)))
            .collect::<Result<_>>()?;
        // Place a duplicate in a world that precedes the root in this map's actual iteration order.
        let mut duplicate = None;
        for _ in 0..1024 {
            client.world_entities = (0..64).map(|id| (id, State::default())).collect();
            let first = *client.world_entities.keys().next().unwrap();
            if first != 0 {
                duplicate = Some(first);
                break;
            }
        }
        let duplicate = duplicate.expect("could not arrange duplicate-first map");
        client.npc_state.npcs.clear();
        client.npc_state.npcs.insert(1, npc(marker));
        for parent in [0, duplicate] {
            let mut entity = Entity::default();
            entity.npcs.npcs.insert(1, npc(marker));
            client
                .world_entities
                .get_mut(&parent)
                .unwrap()
                .entities
                .insert(7, entity);
        }
        client.apply(
            game_packet::decode(&Packet {
                opcode: 47,
                payload: vec![0, selected as u8, plane],
            })?
            .unwrap(),
        )?;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 20,
                payload: vec![plane.wrapping_neg(), 0, 129, 255, 255],
            })?
            .unwrap(),
        )?;
        let states = [
            &client.npc_state.npcs[&1].sequence,
            &client.world_entities[&0].entities[&7].npcs.npcs[&1].sequence,
            &client.world_entities[&duplicate].entities[&7].npcs.npcs[&1].sequence,
        ];
        for (i, state) in states.into_iter().enumerate() {
            assert_eq!(
                (state.id, state.frame, state.delay),
                expected[i],
                "case{case} view{i}"
            );
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("32 original packet47-to20 sequences match cache-loaded ClientState with forced duplicate-first lookup order");
    Ok(())
}
