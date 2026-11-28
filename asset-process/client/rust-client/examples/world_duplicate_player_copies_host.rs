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
        "../tests/fixtures/world-duplicate-player-copies-selected-jvm.bin"
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
                payload: update.clone(),
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
        let local_before = client
            .player_state
            .as_ref()
            .unwrap()
            .local_actor()
            .unwrap()
            .path
            .position[0];
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
        let remove_parent = r.u32()? as u16;
        let after_removal = [r.u32()? as i32, r.u32()? as i32];
        assert_ne!(r.u8()?, 0);
        let after_update = [r.u32()? as i32, r.u32()? as i32];
        assert_eq!(local_before, r.u32()? as i32, "case {case} local lookup");
        let local_after = r.u32()? as i32;
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
        let survivor = generations[0].max(generations[1]);
        let removed = generations[0].min(generations[1]);
        let players = client.player_state.as_ref().unwrap();
        assert!(players.instance_actor(id, removed, 42).is_none());
        assert_eq!(
            players.local_actor().unwrap().path.position[0],
            local_after,
            "case {case} local lookup after removal"
        );
        assert_eq!(
            players
                .instance_actor(id, survivor, 42)
                .unwrap()
                .path
                .position,
            after_removal,
            "case {case} after removal"
        );
        client.apply(
            game_packet::decode(&Packet {
                opcode: 28,
                payload: update,
            })?
            .unwrap(),
        )?;
        assert_eq!(
            client
                .player_state
                .as_ref()
                .unwrap()
                .instance_actor(id, survivor, 42)
                .unwrap()
                .path
                .position,
            after_update,
            "case {case} after next player packet"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original duplicate-player instances passed actual-cache packet28 retention, actor tick bounds and packet122 removal");
    Ok(())
}
