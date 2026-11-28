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
    let mut r = Reader::new(include_bytes!("../tests/fixtures/location-world-jvm.bin"));
    assert_eq!(r.u32()?, 256);
    for case in 0..256 {
        let selected = r.u32()? as u16;
        let tile = r.u32()? as u8;
        let plane = r.u32()? as u8;
        let shape = r.u32()? as u8;
        let rotation = r.u32()? as u8;
        let mut expected = Vec::new();
        for _ in 0..3 {
            let mut parts = Vec::new();
            for _ in 0..2 {
                parts.push(if r.u8()? != 0 {
                    Some([
                        r.u32()? as i32,
                        r.u32()? as i32,
                        r.u32()? as i32,
                        r.u32()? as i32,
                        r.u32()? as i32,
                    ])
                } else {
                    None
                });
            }
            expected.push(parts);
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
        client.zone.locations.clear();
        client.world_base = [0; 2];
        client.actor_cycle = 100;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 47,
                payload: vec![0, selected as u8, plane],
            })?
            .unwrap(),
        )?;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 30,
                payload: vec![
                    (shape * 4 + rotation).wrapping_add(128),
                    0,
                    0,
                    0,
                    tile,
                    tile,
                ],
            })?
            .unwrap(),
        )?;
        let groups = [
            &client.zone.locations,
            &client.world_entities[&0].entities[&7].zone.locations,
            &client.world_entities[&duplicate].entities[&7]
                .zone
                .locations,
        ];
        for (i, group) in groups.into_iter().enumerate() {
            assert_eq!(
                group.len(),
                usize::from(expected[i][0].is_some()),
                "case{case} view{i}"
            );
            if let Some(event) = group.front() {
                let gielinor_client::location_update::Event::Animate(at, request) = event else {
                    panic!("animation event expected")
                };
                assert_eq!(request.sequence, 0);
                let parts = gielinor_client::location_animation::parts(
                    request.shape,
                    request.rotation,
                    Some(0),
                    0,
                )?;
                for (part, want) in parts.into_iter().zip(&expected[i]) {
                    assert_eq!(
                        part.map(|p| [p.shape as i32, p.rotation as i32, at[0], at[1], at[2]]),
                        *want,
                        "case{case} view{i}"
                    );
                }
            }
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("256 original packet47-to30 selected-view wall mutations match cache-loaded event routing and renderable part selection; terrain rendering is separate acceptance");
    Ok(())
}
