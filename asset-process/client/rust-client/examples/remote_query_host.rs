use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_state::Interfaces,
    remote_card::Card,
    remote_json,
    remote_url::Context,
    remote_widget::Loader,
    vm::Host,
    vm::{Object, Stacks},
    widget::Widget,
    Result,
};
fn text(r: &mut Reader<'_>) -> Result<String> {
    assert_eq!(r.u8()?, 1);
    let n = r.u16()? as usize;
    Ok(String::from_utf8(r.bytes(n)?.to_vec()).unwrap())
}
fn wrong(v: i32) -> Object {
    Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
        gielinor_client::vm::Array::Ints(vec![v]),
    )))
}

fn main() -> Result<()> {
    let mut r = Reader::new(include_bytes!("../tests/fixtures/remote-query-jvm.bin"));
    assert_eq!(r.u32()?, 864);
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    for case in 0..864 {
        let op = r.u32()? as u16;
        let dot = r.u8()? != 0;
        let kind = r.u32()? as u8;
        let status = r.u32()? as i32;
        let loader_kind = r.u32()?;
        let key = r.u32()?;
        let failed = r.u8()? != 0;
        let result = r.u32()? as i32;
        let count = r.u32()?;
        let mut ints = Vec::new();
        for _ in 0..count {
            ints.push(r.u32()? as i32);
        }
        let count = r.u32()?;
        let mut objects = Vec::new();
        for _ in 0..count {
            objects.push(match r.u8()? {
                0 => Object::Null,
                1 => Object::String(text(&mut r)?),
                2 => wrong(r.u32()? as i32),
                _ => panic!("fixture object tag"),
            });
        }
        let mut state = Interfaces::default();
        state.install_group(0, vec![Widget::empty(0, kind)])?;
        let runtime = state.runtime.get_mut(&0).unwrap();
        runtime.remote_status = status;
        if loader_kind > 0 {
            let mut loader = Loader::default();
            if loader_kind == 2 {
                let mut card = Card::parse(&remote_json::content("{}").unwrap(), None)?;
                card.meta.insert("known".encode_utf16().collect(), 42);
                card.meta_strings
                    .insert("known".encode_utf16().collect(), "value".into());
                loader.card = Some(std::sync::Arc::new(card));
            }
            runtime.remote_loader = Some(Box::new(loader));
        }
        let context = Context::default();
        let mut stack = Stacks::default();
        stack.push_int(77)?;
        stack.push_int(93)?;
        stack.pop_int()?;
        if op >= 2000 {
            stack.push_int(0)?;
        }
        stack.push_object(Object::String("sentinel".into()))?;
        stack.push_object(match key {
            0 => Object::String("known".into()),
            1 => Object::String("missing".into()),
            2 => Object::Null,
            _ => wrong(9),
        })?;
        client.interfaces = state;
        client.remote = context;
        client.active_widgets = [Some(0); 2];
        let actual = client.extended(op, dot, &mut stack);
        assert_eq!(actual.is_err(), failed, "case{case}");
        if !failed {
            assert_eq!(result, 1);
            assert!(matches!(actual?, gielinor_client::vm::Control::Continue));
        }
        assert_eq!(stack.ints(), ints.as_slice(), "case{case}");
        assert_eq!(stack.objects().len(), objects.len(), "case{case}");
        for (actual, expected) in stack.objects().iter().zip(&objects) {
            match (actual, expected) {
                (Object::Null, Object::Null) => (),
                (Object::String(a), Object::String(b)) => assert_eq!(a, b, "case{case}"),
                (Object::Array(a), Object::Array(b)) => {
                    assert_eq!(*a.borrow(), *b.borrow(), "case{case}")
                }
                _ => panic!("case{case}: object type mismatch"),
            }
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("864 original remote query cases pass actual ClientState dispatch");
    Ok(())
}
