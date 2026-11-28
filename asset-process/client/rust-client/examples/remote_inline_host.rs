use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_state::{Interfaces, RemoteMappings},
    remote_widget::Loader,
    vm::{Host, Object, Stacks},
    widget::Widget,
    Result,
};
fn main() -> Result<()> {
    let mut r = Reader::new(include_bytes!("../tests/fixtures/remote-inline-jvm.bin"));
    assert_eq!(r.u32()?, 336);
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    for case in 0..336 {
        let op = r.u32()? as u16;
        let dot = r.u8()? != 0;
        let kind = r.u32()? as u8;
        let seeded = r.u8()? != 0;
        let input = if r.u8()? != 0 {
            let len = r.u16()? as usize;
            Some(String::from_utf8(r.bytes(len)?.to_vec()).unwrap())
        } else {
            None
        };
        let failed = r.u8()? != 0;
        let present = r.u8()? != 0;
        let status = r.u32()? as i32;
        let version = r.u32()? as i32;
        let content = r.u8()? != 0;
        let maps = r.u8()? != 0;
        let ints = r.u32()? as usize;
        let objects = r.u32()? as usize;
        let mut interfaces = Interfaces::default();
        interfaces.install_group(0, vec![Widget::empty(0, kind)])?;
        if seeded {
            let runtime = interfaces.runtime.get_mut(&0).unwrap();
            runtime.remote_loader = Some(Box::new(Loader {
                status: 2,
                version: 7,
                content: Some(Default::default()),
                ..Default::default()
            }));
            let maps = RemoteMappings::default();
            maps.fonts.borrow_mut().insert(17, 29);
            runtime.remote_mappings = Some(Box::new(maps));
        }
        let active = [Some(0); 2];
        let mut stacks = Stacks::default();
        stacks.push_object(input.map_or(Object::Null, Object::String))?;
        if op >= 2000 {
            stacks.push_int(0)?;
        }
        client.interfaces = interfaces;
        client.active_widgets = active;
        let result = client.extended(op, dot, &mut stacks);
        assert_eq!(result.is_err(), failed, "case {case}");
        if !failed {
            result?;
        }
        let runtime = &client.interfaces.runtime[&0];
        let loader = runtime.remote_loader.as_ref();
        assert_eq!(loader.is_some(), present, "case {case}");
        assert_eq!(loader.map_or(-99, |v| v.status), status, "case {case}");
        assert_eq!(loader.map_or(-99, |v| v.version), version, "case {case}");
        assert_eq!(
            loader.is_some_and(|v| v.content.is_some()),
            content,
            "case {case}"
        );
        assert_eq!(runtime.remote_mappings.is_some(), maps);
        if seeded {
            assert_eq!(
                runtime.remote_mappings.as_ref().unwrap().fonts.borrow().get(&17),
                Some(&29)
            );
        }
        assert_eq!(stacks.ints().len(), ints);
        assert_eq!(stacks.objects().len(), objects);
    }
    assert_eq!(r.remaining(), 0);
    println!("Actual-cache ClientState:336 inline remote opcode errors, partial loader states, null no-ops and aliases passed");
    Ok(())
}
