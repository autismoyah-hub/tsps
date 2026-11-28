use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    game_packet,
    protocol::Packet,
    vm::{Host, Object, Stacks},
    Result,
};
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let mut r = Reader::new(include_bytes!("../tests/fixtures/map-area-script-jvm.bin"));
    for _ in 0..r.u32()? {
        r.u32()?;
        let len = r.u32()? as usize;
        r.bytes(len)?;
    }
    let count = r.u32()?;
    for case in 0..count {
        let op = r.u32()? as u16;
        let mut stacks = Stacks::default();
        stacks.push_int(123456)?;
        for _ in 0..r.u32()? {
            stacks.push_int(r.u32()? as i32)?;
        }
        client.extended(op, false, &mut stacks)?;
        let expected = (0..r.u32()?)
            .map(|_| r.u32().map(|v| v as i32))
            .collect::<Result<Vec<_>>>()?;
        assert_eq!(stacks.ints(), expected, "ints {case}");
        assert_eq!(stacks.objects().len(), r.u32()? as usize);
        for actual in stacks.objects() {
            let len = r.u32()? as usize;
            let expected = std::str::from_utf8(r.bytes(len)?).unwrap();
            let Object::String(actual) = actual else {
                panic!("string")
            };
            assert_eq!(actual, expected);
        }
    }
    assert_eq!(r.remaining(), 0);
    assert_eq!(client.world_map.areas.as_ref().unwrap().len(), 52);
    let mut r = Reader::new(include_bytes!("../tests/fixtures/heatmap-state-jvm.bin"));
    for case in 0..r.u32()? {
        let raw = r.u8()?;
        let before = client.heatmap.map.as_ref().map(|m| m.points.clone());
        client.apply(
            game_packet::decode(&Packet {
                opcode: 26,
                payload: vec![raw],
            })?
            .unwrap(),
        )?;
        assert_eq!(client.heatmap.map.is_some(), r.u8()? != 0);
        let retained = r.u8()? != 0;
        assert_eq!(
            client.heatmap.map.as_ref().map_or(0, |m| m.points.len()),
            r.u32()? as usize
        );
        if retained {
            assert_eq!(
                client.heatmap.map.as_ref().map(|m| m.points.clone()),
                before
            );
        }
        if let Some(map) = &mut client.heatmap.map {
            map.add([case as i32, -(case as i32)]);
        }
    }
    assert!(game_packet::decode(&Packet {
        opcode: 26,
        payload: vec![]
    })
    .is_err());
    assert!(game_packet::decode(&Packet {
        opcode: 26,
        payload: vec![1, 2]
    })
    .is_err());
    client.heatmap.enabled(true);
    client.heatmap.refresh([3200, 6400], [[128, 256]], 99);
    client.world_map.center = [123, 456];
    client.world_map.target_zoom = 3.0;
    let current = client.world_map.current;
    client.prepare_login()?;
    assert!(client.heatmap.map.is_none());
    assert_eq!(client.world_map.areas.as_ref().unwrap().len(), 52);
    assert_eq!(client.world_map.current, current);
    assert_eq!(client.world_map.center, [123, 456]);
    assert_eq!(client.world_map.target_zoom, 3.0);
    println!("4096 original map metadata traces through ClientState,1024 original packet26 transitions, malformed payload rejection and login reset/preservation passed");
    Ok(())
}
