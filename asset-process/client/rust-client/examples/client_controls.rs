//! Cache-backed server metadata, widget notifications and action policy integration.
use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    context_menu::Action,
    disk_cache::DiskCache,
    game_packet,
    protocol::Packet,
    vm::{Host, Object, Stacks, Value},
    widget::Widget,
    Result,
};
fn apply(c: &mut ClientState, op: u16, payload: Vec<u8>) -> Result<()> {
    c.apply(
        game_packet::decode(&Packet {
            opcode: op,
            payload,
        })?
        .unwrap(),
    )
}
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        std::env::args().nth(1).expect("cache path"),
        64 << 20,
    )?)?;
    let mut widget = Widget::empty(0, 0);
    widget.raw_size = [100, 100];
    c.interfaces.install_group(0, vec![widget])?;
    c.interfaces.root = Some(0);
    c.interfaces
        .runtime
        .get_mut(&0)
        .unwrap()
        .listeners
        .insert(1422, (vec![Value::Int(1422)], None));
    let layout = c.interfaces.layout([765, 503])?;
    let mut r = Reader::new(include_bytes!("../tests/fixtures/client-metadata-jvm.bin"));
    for _ in 0..r.u32()? {
        let op = r.u32()? as u16;
        let len = r.u32()? as usize;
        apply(&mut c, op, r.bytes(len)?.to_vec())?;
        let events = c
            .interface_events
            .collect(&c.interfaces, &layout, &c.input, false, false);
        assert_eq!(events.len(), usize::from(op == 37));
        for _ in 0..3 {
            let op = r.u32()? as u16;
            r.u64()?;
            assert_eq!(r.u32()?, 0);
            let mut s = Stacks::default();
            c.extended(op, false, &mut s)?;
            let ints = (0..r.u32()?)
                .map(|_| r.u32().map(|v| v as i32))
                .collect::<Result<Vec<_>>>()?;
            assert_eq!(s.ints(), ints);
            assert_eq!(s.objects().len(), r.u32()? as usize);
            for actual in s.objects() {
                let len = r.u32()? as usize;
                let expected = String::from_utf8(r.bytes(len)?.to_vec()).unwrap();
                let Object::String(actual) = actual else {
                    panic!("string")
                };
                assert_eq!(actual, &expected);
            }
        }
    }
    assert_eq!(r.remaining(), 0);
    apply(&mut c, 79, vec![255, 254, 1, 0])?;
    assert_eq!(c.interaction_policy.get(0), [1, 0]);
    let action = Action::Object {
        id: 325,
        tile: [3200, 3200],
        operation: 1,
        sub: 0,
    };
    c.menu_action(action)?;
    assert!(c.outbound.is_empty());
    apply(&mut c, 79, vec![0, 0, 1, 2])?;
    assert!(!c.interaction_policy.root_action(action)?);
    let examine = Action::Examine {
        kind: 1,
        id: 325,
        tile: [3200, 3200],
    };
    assert!(c.interaction_policy.root_action(examine)?);
    c.menu_action(examine)?;
    assert_eq!(c.outbound.pop_front().unwrap().opcode, 78);
    apply(&mut c, 66, vec![0, 0])?;
    assert!(!c.interaction_policy.root_action(examine)?);
    apply(&mut c, 79, vec![255, 254, 1, 1])?;
    c.menu_action(action)?;
    assert_eq!(c.outbound.len(), 1);
    let assets = std::sync::Arc::new(gielinor_client::minimap::Assets::load(
        &mut c.scripts.source.cache,
    )?);
    assert!(assets.flags.len() >= 4, "cached steering sprites");
    let terrain = std::sync::Arc::new(gielinor_client::sprite_pixels::Pixels {
        canvas: [512, 512],
        size: [512, 512],
        offset: [0, 0],
        pixels: vec![0x203040; 512 * 512],
    });
    let mask = gielinor_client::minimap::Mask {
        width: 150,
        starts: vec![0; 150],
        widths: vec![150; 150],
    };
    let mut raster = gielinor_client::raster::Raster::new(340, 340, 340 * 340)?;
    let direction =
        gielinor_client::steering::minimap_click(&mask, [75, 75], [120, 35], 3000).unwrap();
    let packet = c.steering.request(direction)?;
    assert_eq!(packet.opcode, 44);
    assert_eq!(packet.payload, [direction]);
    for i in 0..4 {
        let origin = [10 + (i % 2) * 170, 10 + (i / 2) * 170];
        let view = gielinor_client::minimap::View {
            terrain: terrain.clone(),
            assets: assets.clone(),
            markers: vec![],
            players: vec![],
            entities: vec![],
            icons: vec![],
            destination: None,
            hint: None,
            compass_disabled: false,
            source: [256, 256],
            angle: (i * 4096) as u16,
            disabled: false,
            local_visible: true,
            steering: Some(([origin[0] + 120, origin[1] + 35], c.steering)),
        };
        view.draw(&mut raster, origin, &mask)?;
    }
    assert!(
        raster
            .pixels()
            .iter()
            .filter(|&&v| v != 0xff203040 && v != 0 && v != 0xff000000)
            .count()
            > 100
    );
    std::fs::write("work/steering-cache.bmp", raster.bmp()?)?;
    for _ in 0..30 {
        c.steering.tick();
    }
    assert_eq!(c.steering.remaining, 0);
    c.outbound.clear();
    apply(&mut c, 79, vec![0, 0, 2, 1])?;
    for direction in 0..16 {
        c.menu_action(Action::Heading(direction))?;
        let packet = c.outbound.pop_front().unwrap();
        assert_eq!((packet.opcode, packet.payload), (44, vec![direction]));
        assert_eq!(
            (c.steering.direction, c.steering.remaining),
            (direction, 30)
        );
    }
    assert!(c.menu_action(Action::Heading(16)).is_err());
    apply(&mut c, 79, vec![0, 0, 1, 1])?;
    c.menu_action(Action::Heading(3))?;
    assert!(c.outbound.is_empty());
    let retained = c.data.server_string.clone();
    c.prepare_login()?;
    assert_eq!(c.data.server_string, retained);
    assert!(c.data.reboot_message.is_none());
    assert_eq!(c.interaction_policy.get(0), [1, 1]);
    println!("2048 metadata transitions, 6144 production script reads, reboot notifications, per-world action restrictions/reset and login state passed");
    Ok(())
}
