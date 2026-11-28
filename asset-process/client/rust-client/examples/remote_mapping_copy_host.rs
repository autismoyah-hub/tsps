use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_state::{Interfaces, RemoteMappings},
    vm::Host,
    vm::{Object, Stacks},
    widget::Widget,
    Result,
};
fn main() -> Result<()> {
    let mut r = Reader::new(include_bytes!("../tests/fixtures/remote-map-copy-jvm.bin"));
    assert_eq!(r.u32()?, 8);
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    for case in 0..8 {
        let seeded = r.u8()? != 0;
        let key = r.u32()? as i32;
        let endpoint = if r.u8()? != 0 {
            let n = r.u16()? as usize;
            Some(String::from_utf8(r.bytes(n)?.to_vec()).unwrap())
        } else {
            None
        };
        let mut expected = Vec::new();
        for _ in 0..2 {
            expected.push((r.u8()? != 0, r.u32()? as i32, r.u32()? as i32));
        }
        let fonts_shared = r.u8()? != 0;
        let endpoints_shared = r.u8()? != 0;
        client.interfaces = Interfaces::default();
        client
            .interfaces
            .install_group(0, vec![Widget::empty(0, 0)])?;
        let source = client.interfaces.create_child(0, 0, 11, true)?;
        if seeded {
            let mut maps = RemoteMappings::default();
            maps.fonts.borrow_mut().insert(key, 29);
            maps.endpoints.insert(endpoint.clone(), 31);
            client
                .interfaces
                .runtime
                .get_mut(&source)
                .unwrap()
                .remote_mappings = Some(Box::new(maps));
        }
        let mut stacks = Stacks::default();
        for value in [0, 0, 1] {
            stacks.push_int(value)?;
        }
        client.active_widgets = [None; 2];
        client.extended(105, false, &mut stacks)?;
        let copy = client.active_widgets[0].unwrap();
        stacks.push_int(key)?;
        stacks.push_int(101)?;
        client.extended(1131, false, &mut stacks)?;
        stacks.push_object(endpoint.clone().map_or(Object::Null, Object::String))?;
        stacks.push_int(202)?;
        client.extended(1132, false, &mut stacks)?;
        for (id, (present, font, value)) in [source, copy].into_iter().zip(expected) {
            let maps = client.interfaces.runtime[&id].remote_mappings.as_ref();
            assert_eq!(maps.is_some(), present, "case{case}");
            assert_eq!(
                maps.and_then(|m| m.fonts.borrow().get(&key).copied())
                    .unwrap_or(-999),
                font
            );
            assert_eq!(
                maps.and_then(|m| m.endpoints.get(&endpoint))
                    .copied()
                    .unwrap_or(-999),
                value
            );
        }
        let original = client.interfaces.runtime[&source].remote_mappings.as_ref();
        let copied = client.interfaces.runtime[&copy]
            .remote_mappings
            .as_ref()
            .unwrap();
        assert_eq!(
            original.is_some_and(|original| std::rc::Rc::ptr_eq(&original.fonts, &copied.fonts)),
            fonts_shared
        );
        assert_eq!(
            original.is_some_and(|original| std::ptr::eq(&original.endpoints, &copied.endpoints)),
            endpoints_shared
        );
    }
    assert_eq!(r.remaining(), 0);
    println!(
        "8 original widget-copy mapping cases pass through actual ClientState opcode105/1131/1132"
    );
    Ok(())
}
