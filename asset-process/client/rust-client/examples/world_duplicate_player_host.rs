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
        "../tests/fixtures/world-duplicate-player-jvm.bin"
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
        client.apply(
            game_packet::decode(&Packet {
                opcode: 28,
                payload: update,
            })?
            .unwrap(),
        )?;
        let players = client.player_state.as_ref().unwrap();
        assert_eq!(players.local_view, local_view, "case {case} local view");
        assert_eq!(players.high.len(), high);
        assert_eq!(players.root_actor(42).is_some(), root_player);
        // This bounded check covers the selected global view; independent duplicate
        // player copies still require identity-aware player storage.
        assert!(direct_player && nested_player);
        assert!(players.view_actor(id, 42).is_some());
        assert_eq!(
            players.views[&id].dimensions, [expected; 2],
            "case {case} global descriptor"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original duplicate-world player packet28 traces passed cache-backed application, global descriptors and local-view selection");
    Ok(())
}
