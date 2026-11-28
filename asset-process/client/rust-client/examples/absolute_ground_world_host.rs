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
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/absolute-ground-world-jvm.bin"
    ));
    assert_eq!(r.u32()?, 36);
    let ids = [0, 1, 17, 2];
    let sizes = [104, 16, 16, 8];
    let bases = [[3200, 3200], [100, 200], [108, 208], [90, 190]];
    for case in 0..36 {
        let reverse = r.u32()?;
        let selected = r.u32()? as u16;
        let point = [r.u32()? as i32, r.u32()? as i32];
        let expected = [r.u8()? != 0, r.u8()? != 0, r.u8()? != 0, r.u8()? != 0];
        client.world_entities.clear();
        client.world_entities.insert(0, State::default());
        client.world_base = bases[0];
        client.zone = Default::default();
        client.view_registry = Registry::default();
        for id in if reverse == 0 { [1, 17, 2] } else { [17, 1, 2] } {
            client.view_registry.insert(id);
        }
        for i in 1..4 {
            let mut entity = Entity::default();
            entity.dimensions = [sizes[i]; 2];
            entity.rebuild = Some(gielinor_client::rebuild::WorldEntity {
                base: bases[i],
                dimensions: [sizes[i] as usize; 2],
                chunks: vec![],
                regions: vec![],
            });
            client
                .world_entities
                .get_mut(&0)
                .unwrap()
                .entities
                .insert(ids[i], entity);
        }
        for i in 0..4 {
            if (bases[i][0]..bases[i][0] + sizes[i]).contains(&point[0])
                && (bases[i][1]..bases[i][1] + sizes[i]).contains(&point[1])
            {
                let zone = if i == 0 {
                    &mut client.zone
                } else {
                    &mut client
                        .world_entities
                        .get_mut(&0)
                        .unwrap()
                        .entities
                        .get_mut(&ids[i])
                        .unwrap()
                        .zone
                };
                zone.piles.insert(
                    [1, point[0], point[1]],
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
        }
        client.apply(
            game_packet::decode(&Packet {
                opcode: 47,
                payload: vec![0, selected as u8, 3],
            })?
            .unwrap(),
        )?;
        let coord = (1u32 << 28) | ((point[0] as u32) << 14) | point[1] as u32;
        let mut payload = vec![0, 129, 0, 0, 0, 1];
        payload.extend(coord.to_be_bytes());
        client.apply(
            game_packet::decode(&Packet {
                opcode: 41,
                payload,
            })?
            .unwrap(),
        )?;
        for i in 0..4 {
            let zone = if i == 0 {
                &client.zone
            } else {
                &client.world_entities[&0].entities[&ids[i]].zone
            };
            assert_eq!(
                zone.piles.contains_key(&[1, point[0], point[1]]),
                expected[i],
                "case{case} view{}",
                ids[i]
            );
        }
        assert_eq!((client.active_world, client.active_plane), (selected, 3));
    }
    assert_eq!(r.remaining(), 0);
    println!("36 original packet47-to41 absolute ground removals match cache-loaded ClientState independently of selected view");
    Ok(())
}
