//! Actual-cache packet application for duplicate child-world identities.
use gielinor_client::npc_info::Npc;
use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, game_packet, protocol::Packet,
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
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/world-duplicate-removal-jvm.bin"
    ));
    let count = r.u32()?;
    for case in 0..count {
        client.world_entities.clear();
        client.detached_world_entities.clear();
        client.active_world = 0;
        client.view_registry = Default::default();
        let id = r.u32()? as u16;
        let _order = r.u32()?;
        for _ in 0..r.u32()? {
            let opcode = r.u32()? as u16;
            let n = r.u32()? as usize;
            let payload = r.bytes(n)?.to_vec();
            client.apply(game_packet::decode(&Packet { opcode, payload })?.unwrap())?;
        }
        assert!(
            client.world_entities[&0].entities[&7].spawn_generation
                < client.world_entities[&0].entities[&id].spawn_generation,
            "case {case} packet spawn order"
        );
        let expected = r.u32()? as i32;
        let direct = r.u32()? as i32;
        let nested = r.u32()? as i32;
        assert_eq!(
            client.view_entity(id).unwrap().dimensions,
            [expected; 2],
            "case {case}"
        );
        assert_eq!(
            client.world_entities[&0].entities[&id].dimensions,
            [direct; 2]
        );
        assert_eq!(
            client.world_entities[&7].entities[&id].dimensions,
            [nested; 2]
        );
        for parent in [0, 7] {
            let mut actor = npc();
            actor.overlays.speech.text = Some("retained".into());
            actor.overlays.speech.remaining = 5;
            let entity = client
                .world_entities
                .get_mut(&parent)
                .unwrap()
                .entities
                .get_mut(&id)
                .unwrap();
            entity.npcs.indices.push(1);
            entity.npcs.npcs.insert(1, actor);
        }
        client.world_base = [3200; 2];
        client
            .world_entities
            .get_mut(&0)
            .unwrap()
            .entities
            .get_mut(&7)
            .unwrap()
            .rebuild = Some(gielinor_client::rebuild::WorldEntity {
            base: [1000; 2],
            dimensions: [32; 2],
            chunks: vec![],
            regions: vec![],
        });
        for parent in [0, 7] {
            client
                .world_entities
                .get_mut(&parent)
                .unwrap()
                .entities
                .get_mut(&id)
                .unwrap()
                .zone
                .piles
                .insert(
                    [0, 1, 1],
                    vec![gielinor_client::zone::Item {
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
        let remove_parent = r.u32()? as u16;
        let after = r.u32()? as i32;
        let root_link = r.u8()? != 0;
        let nested_link = r.u8()? != 0;
        let direct_speech = r.u32()? as i32;
        let nested_speech = r.u32()? as i32;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 47,
                payload: vec![0, remove_parent as u8, 0],
            })?
            .unwrap(),
        )?;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 122,
                payload: if remove_parent == 0 {
                    vec![1, 1, 0]
                } else {
                    vec![0]
                },
            })?
            .unwrap(),
        )?;
        assert_eq!(
            client.view_entity(id).unwrap().dimensions,
            [after; 2],
            "case {case} retained global"
        );
        assert!(client.view_registry.ordered().contains(&id));
        assert_eq!(
            client.world_entities[&0].entities.contains_key(&id),
            root_link
        );
        assert_eq!(
            client.world_entities[&7].entities.contains_key(&id),
            nested_link
        );
        client.tick_actor_sequences()?;
        client.tick_actor_sequences()?;
        let expected_speech = if after == direct {
            direct_speech
        } else {
            nested_speech
        };
        assert_eq!(
            client.view_npc(id, 1).unwrap().overlays.speech.remaining,
            expected_speech,
            "case {case} registered countdown"
        );
        for (parent, expected) in [(0, direct_speech), (7, nested_speech)] {
            if let Some(entity) = client.world_entities[&parent].entities.get(&id) {
                assert_eq!(
                    entity.npcs.npcs[&1].overlays.speech.remaining, expected,
                    "case {case} retained parent countdown"
                );
            }
        }
        let direct_pile = r.u8()? != 0;
        let nested_pile = r.u8()? != 0;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 41,
                payload: vec![0, 129, 0, 0, 0, 1, 0, 0, 64, 1],
            })?
            .unwrap(),
        )?;
        assert!(
            !client
                .view_entity(id)
                .unwrap()
                .zone
                .piles
                .contains_key(&[0, 1, 1]),
            "case {case} global pile removed"
        );
        for (parent, expected) in [(0, direct_pile), (7, nested_pile)] {
            if let Some(entity) = client.world_entities[&parent].entities.get(&id) {
                assert_eq!(
                    entity.zone.piles.contains_key(&[0, 1, 1]),
                    expected,
                    "case {case} parent pile identity"
                );
            }
        }
        let remaining = r.u8()? != 0;
        let other_parent = if remove_parent == 0 { 7 } else { 0 };
        client.apply(
            game_packet::decode(&Packet {
                opcode: 47,
                payload: vec![0, other_parent, 0],
            })?
            .unwrap(),
        )?;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 122,
                payload: if other_parent == 0 {
                    vec![1, 1, 0]
                } else {
                    vec![0]
                },
            })?
            .unwrap(),
        )?;
        assert_eq!(
            client.view_entity(id).is_some(),
            remaining,
            "case {case} final unregister"
        );
        assert!(!client.view_registry.ordered().contains(&id));
        assert!(
            client
                .detached_world_entities
                .iter()
                .all(|state| !state.entities.contains_key(&id)),
            "case {case} orphan released"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original removal traces passed actual-cache global/parent identity and NPC countdown checks");
    Ok(())
}
