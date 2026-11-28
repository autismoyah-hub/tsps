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
    let mut r = Reader::new(include_bytes!("../tests/fixtures/relative-zone-jvm.bin"));
    assert_eq!(r.u32()?, 32);
    for case in 0..32 {
        let selected = r.u32()? as u16;
        let tile = r.u32()? as u8;
        let plane = r.u32()? as u8;
        let mut expected = Vec::new();
        for _ in 0..3 {
            expected.push(if r.u8()? != 0 {
                Some([
                    r.u32()? as i32,
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
        for (parent, size) in [(0, 16), (duplicate, 8)] {
            let mut entity = Entity::default();
            entity.dimensions = [size; 2];
            client
                .world_entities
                .get_mut(&parent)
                .unwrap()
                .entities
                .insert(7, entity);
        }
        client.world_spots.clear();
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
                opcode: 9,
                payload: vec![tile.wrapping_add(128), tile, 255],
            })?
            .unwrap(),
        )?;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 86,
                payload: vec![12, 0, 0, 7, 0, 0],
            })?
            .unwrap(),
        )?;
        let groups = [
            &client.world_spots,
            &client.world_entities[&0].entities[&7].world_spots,
            &client.world_entities[&duplicate].entities[&7].world_spots,
        ];
        for (i, group) in groups.into_iter().enumerate() {
            assert_eq!(
                group.len(),
                usize::from(expected[i].is_some()),
                "case{case} view{i}"
            );
            if let Some(want) = expected[i] {
                let e = &group[0];
                assert_eq!(
                    [
                        e.animation.id as i32,
                        e.position[0],
                        e.animation.start,
                        e.position[1] * 128 + 64,
                        e.position[2] * 128 + 64,
                        -e.animation.height
                    ],
                    want,
                    "case{case} view{i}"
                );
            }
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("32 original packet47-to9-to86 relative graphics admission and placement cases match cache-loaded ClientState; flat-terrain height input compared before renderer resolution");
    Ok(())
}
