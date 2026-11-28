use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, interface_state::Interfaces,
    remote_card::Card, remote_json, remote_url::Context, remote_widget::Loader, vm::Host,
    vm::Stacks, widget::Widget, Result,
};
fn text(r: &mut Reader<'_>) -> Result<String> {
    let n = r.u16()? as usize;
    Ok(String::from_utf8(r.bytes(n)?.to_vec()).unwrap())
}
fn optional(r: &mut Reader<'_>) -> Result<Option<String>> {
    if r.u8()? != 0 {
        Ok(Some(text(r)?))
    } else {
        Ok(None)
    }
}

fn main() -> Result<()> {
    let mut r = Reader::new(include_bytes!("../tests/fixtures/remote-dismiss-jvm.bin"));
    assert_eq!(r.u32()?, 144);
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    for case in 0..144 {
        let op = r.u32()? as u16;
        let dot = r.u8()? != 0;
        let kind = r.u32()? as u8;
        let loader_kind = r.u32()?;
        let id = optional(&mut r)?;
        let event = r.u32()? as i32;
        let failed = r.u8()? != 0;
        let result = r.u32()? as i32;
        let count = r.u32()?;
        let mut expected = Vec::new();
        for _ in 0..count {
            expected.push(r.u32()? as i32);
        }
        let expected_kind = r.u32()? as i32;
        let expected_id = optional(&mut r)?;
        let mut state = Interfaces::default();
        state.install_group(0, vec![Widget::empty(0, kind)])?;
        if loader_kind > 0 {
            let mut loader = Loader::default();
            if loader_kind == 2 {
                let mut card = Card::parse(&remote_json::content("{}").unwrap(), None)?;
                card.id = id;
                let card = std::sync::Arc::new(card);
                loader.content = Some(card.raw.clone());
                loader.card = Some(card);
            }
            state.runtime.get_mut(&0).unwrap().remote_loader = Some(Box::new(loader));
        }
        let mut context = Context::default();
        context.crm.event(event, Some("previous".into()));
        let mut stack = Stacks::default();
        stack.push_int(77)?;
        if op >= 2000 {
            stack.push_int(0)?;
        }
        client.interfaces = state;
        client.active_widgets = [Some(0); 2];
        client.remote = context;
        let actual = client.extended(op, dot, &mut stack);
        let context = &client.remote;
        assert_eq!(actual.is_err(), failed, "case{case}");
        if !failed {
            assert_eq!(result, 1);
            assert!(matches!(actual?, gielinor_client::vm::Control::Continue));
        }
        assert_eq!(stack.ints(), expected.as_slice(), "case{case}");
        assert_eq!(context.crm.event_kind, expected_kind);
        assert_eq!(context.crm.event_id, expected_id);
        assert!(stack.objects().is_empty());
    }
    assert_eq!(r.remaining(), 0);
    println!("144 original dismissal cases pass through actual ClientState dispatch");
    Ok(())
}
