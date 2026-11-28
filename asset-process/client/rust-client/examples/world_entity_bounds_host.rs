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
    let mut r = Reader::new(include_bytes!("../tests/fixtures/world-bounds-jvm.bin"));
    assert_eq!(r.u32()?, 48);
    for case in 0..48 {
        let selected = r.u32()? as u16;
        let tile = r.u32()? as i32;
        let plane = r.u32()? as u8;
        let mut expected = Vec::new();
        for _ in 0..3 {
            let mut values = [0; 9];
            for value in &mut values {
                *value = r.u32()? as i32;
            }
            expected.push(values);
        }
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
        for (parent, size) in [(0, 16), (duplicate, 32)] {
            let mut entity = Entity::default();
            entity.dimensions = [size; 2];
            client
                .world_entities
                .get_mut(&parent)
                .unwrap()
                .entities
                .insert(7, entity);
        }
        for parent in [0, 7, 65] {
            let state = client.world_entities.entry(parent).or_default();
            let mut e = Entity::default();
            e.definition = Some(gielinor_client::world_entity::Definition::decode(-1, &[0])?);
            state.entities.insert(5, e);
            state.visible = vec![5];
        }
        // Avoid lazy definition loads for the pre-existing selection fixtures.
        for state in client.world_entities.values_mut() {
            for e in state.entities.values_mut() {
                e.definition = Some(gielinor_client::world_entity::Definition::decode(-1, &[0])?);
            }
        }
        client.actor_cycle = 100;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 47,
                payload: vec![0, selected as u8, plane],
            })?
            .unwrap(),
        )?;
        let [hi, lo] = (tile.wrapping_mul(128) as i16).to_be_bytes();
        client.apply(
            game_packet::decode(&Packet {
                opcode: 122,
                payload: vec![1, 2, 34, hi, lo, hi, lo, 0],
            })?
            .unwrap(),
        )?;
        for (i, parent) in [0, 7, 65].into_iter().enumerate() {
            let e = &client.world_entities[&parent].entities[&5];
            let p = e.pose;
            let q = e.target();
            assert_eq!(
                [
                    e.queued as i32,
                    p.x,
                    p.y,
                    p.z,
                    p.angle,
                    q.x,
                    q.y,
                    q.z,
                    q.angle
                ],
                expected[i],
                "case{case} view{i}"
            );
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("48 original packet47-to122 movement transitions match cache-loaded selected parent bounds and unrelated view preservation");
    Ok(())
}
