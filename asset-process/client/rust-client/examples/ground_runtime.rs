//! Original JVM packet replay through a cache-backed client, with fixture item valuations.
use gielinor_client::{
    buffer::Reader,
    item::{Definition, Definitions},
    Result,
};
fn main() -> Result<()> {
    let mut definitions = Definitions {
        items: Default::default(),
    };
    for id in 1..=4 {
        let mut d = Definition::decode(id, &[0]).unwrap();
        d.price = id as i32 * 10;
        d.stackable = id as i32 % 2;
        d.recolor = vec![(0, 100), (1, 101)];
        d.retexture = vec![(0, 200)];
        definitions.items.insert(id, d);
    }
    let cache = std::env::args().nth(1).expect("cache path");
    let mut client = gielinor_client::client_state::ClientState::load(
        gielinor_client::disk_cache::DiskCache::open(cache, 64 << 20)?,
    )?;
    client.items = definitions;
    client.server_tick = 1000;

    for (data, base) in [
        (
            include_bytes!("../tests/fixtures/ground-items-jvm.bin").as_slice(),
            [0, 0],
        ),
        (
            include_bytes!("../tests/fixtures/ground-specific-jvm.bin").as_slice(),
            [3000, 3200],
        ),
    ] {
        client.zone = Default::default();
        client.zone.origin = [0, 4, 4];
        client.world_base = base;
        let mut r = Reader::new(data);
        for case in 0..r.u32().unwrap() {
            let op = r.u16().unwrap();
            let len = r.u32().unwrap() as usize;
            let packet = gielinor_client::protocol::Packet {
                opcode: op,
                payload: r.bytes(len).unwrap().to_vec(),
            };
            client.apply(gielinor_client::game_packet::decode(&packet)?.expect("zone handler"))?;
            let state = &client.zone;
            assert_eq!(
                state.origin,
                std::array::from_fn(|_| r.u32().unwrap() as i32)
            );
            for plane in 0..4 {
                let count = r.u32().unwrap() as usize;
                let pile = state
                    .piles
                    .get(&[plane, base[0] + 4, base[1] + 4])
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                assert_eq!(pile.len(), count, "pile size {case} plane {plane}");
                for item in pile {
                    let expected: [i32; 6] = std::array::from_fn(|_| r.u32().unwrap() as i32);
                    assert_eq!(
                        [
                            item.id as i32,
                            item.quantity,
                            item.operations as i32,
                            item.visible,
                            item.despawn,
                            item.ownership as i32
                        ],
                        expected,
                        "pile state {case}"
                    );
                    assert_eq!(item.private, r.u8().unwrap() != 0);
                    assert_eq!(item.custom.is_some(), r.u8().unwrap() != 0);
                    if let Some(custom) = &item.custom {
                        assert_eq!(custom.model, r.u32().unwrap());
                        for values in [&custom.recolor, &custom.retexture] {
                            let len = r.u32().unwrap() as usize;
                            let expected = (0..len).map(|_| r.u16().unwrap()).collect::<Vec<_>>();
                            assert_eq!(*values, expected, "custom {case}");
                        }
                    }
                }
            }
        }
        assert_eq!(r.remaining(), 0);
    }
    let packet = gielinor_client::protocol::Packet {
        opcode: 83,
        payload: vec![],
    };
    client.apply(gielinor_client::game_packet::decode(&packet)?.unwrap())?;
    assert_eq!(client.server_tick, 1001);
    assert!(client.zone.pending.is_empty());
    println!("Verified 1024 enclosed/absolute zone packet/pile/customisation transitions through ClientState and server tick progression");
    Ok(())
}
