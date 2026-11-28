use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    game_packet,
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
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/rebuild-selection-jvm.bin"
    ));
    assert_eq!(r.u32()?, 32);
    for case in 0..32 {
        let size = r.u32()? as i32;
        let plane = r.u32()? as u8;
        let length = r.u32()? as usize;
        let payload = r.bytes(length)?.to_vec();
        let mut expected = [[0; 2]; 3];
        for base in &mut expected {
            *base = [r.u32()? as i32, r.u32()? as i32];
        }
        let chunk_count = r.u32()? as usize;
        assert_eq!(r.u8()?, 1); // Original marks the rebuild pending.
        assert_eq!(r.u8()?, 1); // Original pending target is the selected child.
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
        for (parent, size) in [(0, size), (duplicate, 40)] {
            let mut entity = Entity::default();
            entity.dimensions = [size; 2];
            client
                .world_entities
                .get_mut(&parent)
                .unwrap()
                .entities
                .insert(7, entity);
        }
        client.world_base = [123, 456];
        client.apply(
            game_packet::decode(&Packet {
                opcode: 47,
                payload: vec![0, 7, plane],
            })?
            .unwrap(),
        )?;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 109,
                payload,
            })?
            .unwrap(),
        )?;
        assert_eq!(client.world_base, expected[0]);
        let entity = &client.world_entities[&0].entities[&7];
        let rebuilt = entity.rebuild.as_ref().expect("selected child rebuilt");
        assert_eq!(rebuilt.base, expected[1]);
        assert_eq!(rebuilt.dimensions, [size as usize; 2]);
        assert_eq!(rebuilt.chunks.len(), chunk_count);
        assert!(rebuilt.chunks.iter().all(Option::is_none));
        assert!(rebuilt.regions.is_empty());
        assert_eq!(entity.rebuild_generation, case + 1);
        assert!(entity.rebuild_generation > 0);
        assert!(
            client.world_entities[&duplicate].entities[&7]
                .rebuild
                .is_none(),
            "case{case} unrelated view"
        );
        assert_eq!(expected[2], [123, 456]);
    }
    assert_eq!(r.remaining(), 0);
    println!("32 original packet47-to109 selected-child base and empty chunk rebuilds match cache-loaded ClientState; asynchronous scene construction is separate acceptance");
    Ok(())
}
