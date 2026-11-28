use gielinor_client::npc_info::Npc;
use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    game_packet,
    protocol::Packet,
    world_entity::{Entity, State},
    world_view::Registry,
    zone::Item,
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
    let mut client = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/nested-world-removal-jvm.bin"
    ));
    assert_eq!(r.u32()?, 48);
    let bases = [[3200, 3200], [100, 200], [300, 400]];
    let sizes = [104, 16, 8];
    for case in 0..48 {
        let selected = r.u32()? as u16;
        let mode = r.u32()?;
        let plane = r.u32()? as u8;
        let point = r.u32()? as usize;
        let active = r.u32()? as u16;
        let registered = [r.u8()? != 0, r.u8()? != 0, r.u8()? != 0];
        let links = [r.u8()? != 0, r.u8()? != 0];
        let piles = [r.u8()? != 0, r.u8()? != 0, r.u8()? != 0];
        let speech = (0..3)
            .map(|_| Ok((r.u32()? as i32, r.u8()? != 0)))
            .collect::<Result<Vec<_>>>()?;
        client.world_entities.clear();
        client.detached_world_entities.clear();
        client.world_entities.insert(0, State::default());
        client.world_entities.insert(1, State::default());
        client.world_base = bases[0];
        client.zone = Default::default();
        client.view_registry = Registry::default();
        client.view_registry.insert(1);
        client.view_registry.insert(2);
        for id in 1..=2 {
            let mut entity = Entity::default();
            entity.dimensions = [sizes[id]; 2];
            entity.rebuild = Some(gielinor_client::rebuild::WorldEntity {
                base: bases[id],
                dimensions: [sizes[id] as usize; 2],
                chunks: vec![],
                regions: vec![],
            });
            let parent = client.world_entities.get_mut(&((id - 1) as u16)).unwrap();
            parent.visible.push(id as u16);
            parent.entities.insert(id as u16, entity);
        }
        client.npc_state = Default::default();
        for id in 0..3 {
            let mut actor = npc();
            actor.overlays.speech.text = Some("retained".into());
            actor.overlays.speech.remaining = 2;
            let state = if id == 0 {
                &mut client.npc_state
            } else {
                &mut client
                    .world_entities
                    .get_mut(&((id - 1) as u16))
                    .unwrap()
                    .entities
                    .get_mut(&(id as u16))
                    .unwrap()
                    .npcs
            };
            state.indices.push(1);
            state.npcs.insert(1, actor);
            let zone = if id == 0 {
                &mut client.zone
            } else {
                &mut client
                    .world_entities
                    .get_mut(&((id - 1) as u16))
                    .unwrap()
                    .entities
                    .get_mut(&(id as u16))
                    .unwrap()
                    .zone
            };
            zone.piles.insert(
                [1, bases[id][0] + 1, bases[id][1] + 1],
                vec![Item {
                    id: 1,
                    quantity: 1,
                    operations: 31,
                    visible: 0,
                    despawn: 100,
                    ownership: 0,
                    private: false,
                    custom: None,
                }],
            );
        }
        for packet in [
            Packet {
                opcode: 47,
                payload: vec![0, selected as u8, plane],
            },
            Packet {
                opcode: 122,
                payload: if mode == 0 { vec![0] } else { vec![1, 0] },
            },
        ] {
            client.apply(game_packet::decode(&packet)?.unwrap())?;
        }
        let coord =
            (1u32 << 28) | (((bases[point][0] + 1) as u32) << 14) | (bases[point][1] + 1) as u32;
        let mut payload = vec![0, 129, 0, 0, 0, 1];
        payload.extend(coord.to_be_bytes());
        client.apply(
            game_packet::decode(&Packet {
                opcode: 41,
                payload,
            })?
            .unwrap(),
        )?;
        client.tick_actor_sequences()?;
        client.tick_actor_sequences()?;
        assert_eq!(
            (client.active_world, client.active_plane),
            (active, plane),
            "case{case} active parent"
        );
        for id in 0..3 {
            assert_eq!(
                client.view_registry.ordered().contains(&(id as u16)),
                registered[id],
                "case{case} registry{id}"
            );
            if !registered[id] {
                continue;
            }
            let actor = client
                .view_npc(id as u16, 1)
                .expect("registered actor retained");
            assert_eq!(
                (
                    actor.overlays.speech.remaining,
                    actor.overlays.speech.text.is_some()
                ),
                speech[id],
                "case{case} view{id} speech"
            );
            let zone = if id == 0 {
                &client.zone
            } else {
                &client
                    .view_entity(id as u16)
                    .expect("registered nested world retained")
                    .zone
            };
            assert_eq!(
                zone.piles
                    .contains_key(&[1, bases[id][0] + 1, bases[id][1] + 1]),
                piles[id],
                "case{case} view{id} absolute pile"
            );
        }
        assert_eq!(
            client.world_entities[&0].entities.contains_key(&1),
            links[0]
        );
        if registered[1] {
            assert_eq!(
                client.world_entities[&1].entities.contains_key(&2),
                links[1]
            );
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("48 original47-to122-to41 nested removal lifetimes and absolute ground routing match actual cache-loaded ClientState");
    Ok(())
}
