use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_state::{Interfaces, RemoteMappings},
    remote_request::Requester,
    remote_url::Context,
    remote_widget::Loader,
    vm::Host,
    vm::{Object, Stacks},
    widget::Widget,
    Result,
};
fn main() -> Result<()> {
    let mut r = Reader::new(include_bytes!("../tests/fixtures/remote-url-jvm.bin"));
    assert_eq!(r.u32()?, 1792);
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    for case in 0..1792 {
        let opcode = r.u32()? as u16;
        let dot = r.u8()? != 0;
        let kind = r.u32()? as u8;
        let seeded = r.u8()? != 0;
        let available = r.u8()? != 0;
        let user_id = r.u64()? as i64;
        let input = if r.u8()? != 0 {
            let n = r.u16()? as usize;
            Some(String::from_utf8(r.bytes(n)?.to_vec()).unwrap())
        } else {
            None
        };
        let failed = r.u8()? != 0;
        let present = r.u8()? != 0;
        let status = r.u32()? as i32;
        let version = r.u32()? as i32;
        let maps = r.u8()? != 0;
        let fonts = r.u32()? as usize;
        let queued = r.u32()? as usize;
        let url = if queued > 0 {
            let n = r.u16()? as usize;
            Some(String::from_utf8(r.bytes(n)?.to_vec()).unwrap())
        } else {
            None
        };
        let ints = r.u32()? as usize;
        let objects = r.u32()? as usize;
        let mut state = Interfaces::default();
        state.install_group(0, vec![Widget::empty(0, kind)])?;
        if seeded {
            let runtime = state.runtime.get_mut(&0).unwrap();
            runtime.remote_loader = Some(Box::new(Loader {
                status: 2,
                version: 7,
                ..Default::default()
            }));
            let maps = RemoteMappings::default();
            maps.fonts.borrow_mut().insert(17, 29);
            runtime.remote_mappings = Some(Box::new(maps));
        }
        let context = Context {
            user_id,
            requester: available.then(Requester::manual),
            ..Default::default()
        };
        let mut stacks = Stacks::default();
        stacks.push_object(input.map_or(Object::Null, Object::String))?;
        if opcode >= 2000 {
            stacks.push_int(0)?;
        }
        client.interfaces = state;
        client.remote = context;
        client.active_widgets = [Some(0); 2];
        let result = client.extended(opcode, dot, &mut stacks);
        let state = &client.interfaces;
        let context = &client.remote;
        assert_eq!(result.is_err(), failed, "case {case}");
        if !failed {
            result?;
        }
        let runtime = &state.runtime[&0];
        let loader = runtime.remote_loader.as_ref();
        assert_eq!(loader.is_some(), present, "case {case}");
        assert_eq!(loader.map_or(-99, |v| v.status), status, "case {case}");
        assert_eq!(loader.map_or(-99, |v| v.version), version, "case {case}");
        assert_eq!(runtime.remote_mappings.is_some(), maps, "case {case}");
        assert_eq!(
            runtime
                .remote_mappings
                .as_ref()
                .map_or(0, |v| v.fonts.borrow().len()),
            fonts,
            "case {case}"
        );
        let requests = context
            .requester
            .as_ref()
            .map_or(&[][..], Requester::queued);
        assert_eq!(requests.len(), queued, "case {case}");
        assert_eq!(
            requests.first().map(|v| v.url.as_str()),
            url.as_deref(),
            "case {case}"
        );
        assert_eq!(stacks.ints().len(), ints);
        assert_eq!(stacks.objects().len(), objects);
    }
    assert_eq!(r.remaining(), 0);
    for (body, expected) in [("{}", 6), ("{\"version\":2}", 7)] {
        let mut state = Interfaces::default();
        let mut widget = Widget::empty(0, 11);
        widget.raw_size = [100, 100];
        state.install_group(0, vec![widget])?;
        state.root = Some(0);
        client.interfaces = state;
        client.active_widgets = [Some(0); 2];
        client.remote = Context {
            user_id: 17,
            requester: Some(Requester::manual()),
            ..Default::default()
        };
        let mut stacks = Stacks::default();
        stacks.push_object(Object::String("http://example.invalid/%userid%".into()))?;
        client.extended(1130, false, &mut stacks)?;
        client.remote.requester.as_ref().unwrap().queued()[0]
            .complete(Some(body.as_bytes().to_vec()));
        client.tick_interfaces(false)?;
        assert_eq!(
            client.interfaces.runtime[&0]
                .remote_loader
                .as_ref()
                .unwrap()
                .status,
            expected
        );
        assert_eq!(client.interfaces.runtime[&0].remote_status, expected);
    }
    client
        .remote
        .registry
        .cards
        .insert("retained".into(), Default::default());
    let pending = client
        .remote
        .requester
        .as_mut()
        .unwrap()
        .request("http://example.invalid/preserved".into())?;
    client.prepare_login()?;
    assert_eq!(client.remote.user_id, 17);
    assert!(client.remote.registry.cards.contains_key("retained"));
    pending.complete(Some(b"held".to_vec()));
    assert_eq!(
        client
            .remote
            .requester
            .as_ref()
            .unwrap()
            .queued()
            .last()
            .unwrap()
            .result()
            .flatten()
            .as_deref(),
        Some(&b"held".to_vec())
    );
    println!("Actual-cache ClientState:1792 URL opcode cases,2 interface-tick response transitions and retained HTTP/user context across full login reset passed");
    Ok(())
}
