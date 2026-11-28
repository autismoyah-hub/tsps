//! Actual-cache packet application for duplicate child-world identities.
use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, game_packet, protocol::Packet,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/world-duplicate-player-packet-jvm.bin"
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
        let n = r.u32()? as usize;
        let initial = r.bytes(n)?;
        assert_eq!(client.initialize_players(initial, 42)?, initial.len());
        let n = r.u32()? as usize;
        let update = r.bytes(n)?.to_vec();
        let local_view = r.u32()? as u16;
        let high = r.u32()? as usize;
        let root_player = r.u8()? != 0;
        let direct_player = r.u8()? != 0;
        let nested_player = r.u8()? != 0;
        let retained = [r.u32()? as i32, r.u32()? as i32];
        assert_ne!(r.u8()?, 0);
        assert_ne!(r.u8()?, 0);
        let tick_positions = [
            [r.u32()? as i32, r.u32()? as i32],
            [r.u32()? as i32, r.u32()? as i32],
        ];
        client.apply(
            game_packet::decode(&Packet {
                opcode: 28,
                payload: update.clone(),
            })?
            .unwrap(),
        )?;
        let players = client.player_state.as_ref().unwrap();
        assert_eq!(players.local_view, local_view, "case {case} local view");
        assert_eq!(players.high.len(), high);
        assert_eq!(players.root_actor(42).is_some(), root_player);
        assert!(direct_player && nested_player);
        assert_eq!(players.views[&id].dimensions, [expected; 2]);
        let generations = [
            client.world_entities[&0].entities[&id].spawn_generation,
            client.world_entities[&7].entities[&id].spawn_generation,
        ];
        for (generation, x) in generations.into_iter().zip([1234, 2345]) {
            client
                .player_state
                .as_mut()
                .unwrap()
                .instance_view_mut(id, generation)
                .unwrap()
                .players
                .get_mut(&42)
                .unwrap()
                .path
                .position[0] = x;
        }
        client.apply(
            game_packet::decode(&Packet {
                opcode: 28,
                payload: update,
            })?
            .unwrap(),
        )?;
        for (generation, x) in generations.into_iter().zip(retained) {
            assert_eq!(
                client
                    .player_state
                    .as_ref()
                    .unwrap()
                    .instance_actor(id, generation, 42)
                    .unwrap()
                    .path
                    .position[0],
                x,
                "case {case} retained instance {generation}"
            );
        }
        client.actor_cycle = 10;
        client.tick_actor_sequences()?;
        for (generation, point) in generations.into_iter().zip(tick_positions) {
            assert_eq!(
                client
                    .player_state
                    .as_ref()
                    .unwrap()
                    .instance_actor(id, generation, 42)
                    .unwrap()
                    .path
                    .position,
                point,
                "case {case} tick instance {generation}"
            );
        }
        let expected_reset = [r.u32()? as i32, r.u32()? as i32];
        for (generation, sequence) in generations.into_iter().zip([3, 4]) {
            client
                .player_state
                .as_mut()
                .unwrap()
                .instance_view_mut(id, generation)
                .unwrap()
                .players
                .get_mut(&42)
                .unwrap()
                .sequence
                .id = sequence;
        }
        client.apply(
            game_packet::decode(&Packet {
                opcode: 47,
                payload: vec![(id >> 8) as u8, id as u8, 0],
            })?
            .unwrap(),
        )?;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 92,
                payload: vec![],
            })?
            .unwrap(),
        )?;
        for (generation, expected) in generations.into_iter().zip(expected_reset) {
            assert_eq!(
                client
                    .player_state
                    .as_ref()
                    .unwrap()
                    .instance_actor(id, generation, 42)
                    .unwrap()
                    .sequence
                    .id,
                expected,
                "case{case} instance{generation} selected reset"
            );
        }
        client.actor_cycle = 10;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 72,
                payload: vec![0, 251, 0, 2, 0, 3, 127, 0, 42],
            })?
            .unwrap(),
        )?;
        for generation in generations {
            let expected = [r.u32()? as i32, r.u32()? as i32, r.u32()? as i32];
            let effect = client
                .player_state
                .as_ref()
                .unwrap()
                .instance_actor(id, generation, 42)
                .unwrap()
                .spots
                .slots
                .get(&1);
            let actual = effect.map_or([-1; 3], |e| [e.id as i32, e.height, e.start]);
            assert_eq!(
                actual, expected,
                "case{case} instance{generation} selected spot"
            );
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original duplicate-player instances passed actual-cache packet47 selection, packet92 reset and packet72 spot");
    Ok(())
}
