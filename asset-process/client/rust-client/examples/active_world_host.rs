use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    game_packet::{self, Update},
    protocol::Packet,
    world_entity::{Entity, State},
    Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!("../tests/fixtures/active-world-jvm.bin"));
    assert_eq!(r.u32()?, 2048);
    for case in 0..2048 {
        let id = r.u32()? as u16;
        let plane = r.u32()? as u8;
        let previous = r.u32()? as u16;
        let failed = r.u8()? != 0;
        let reported = r.u32()?;
        let selected = r.u32()? as u16;
        let root_plane = r.u32()? as u8;
        let child_plane = r.u32()? as u8;
        let nested_plane = r.u32()? as u8;
        let closed = r.u8()? != 0;
        assert_eq!(failed, closed);
        assert_eq!(reported, u32::from(failed));
        client.world_entities.clear();
        let mut root = State::default();
        root.entities.insert(
            1,
            Entity {
                plane: 3,
                ..Default::default()
            },
        );
        let mut child = State::default();
        child.entities.insert(
            2,
            Entity {
                plane: 1,
                ..Default::default()
            },
        );
        client.world_entities.insert(0, root);
        client.world_entities.insert(1, child);
        client.active_world = previous;
        client.active_plane = if previous == 0 { 2 } else { 3 };
        let mut payload = id.to_be_bytes().to_vec();
        payload.push(plane);
        let update = game_packet::decode(&Packet {
            opcode: 47,
            payload,
        })?
        .unwrap();
        assert!(matches!(update, Update::ActiveWorld { .. }));
        let result = client.apply(update);
        assert_eq!(result.is_err(), failed, "case{case}");
        assert_eq!(client.active_world, selected, "case{case}");
        assert_eq!(
            client.active_plane,
            if selected == 0 {
                root_plane
            } else {
                child_plane
            },
            "case{case}"
        );
        assert_eq!(
            client.world_entities[&0].entities[&1].plane, child_plane,
            "case{case}"
        );
        assert_eq!(
            client.world_entities[&1].entities[&2].plane, nested_plane,
            "case{case}"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("2048 original packet47 root/child selection cases match actual ClientState, including all unsigned plane bytes and rejection of nested-only targets");
    Ok(())
}
