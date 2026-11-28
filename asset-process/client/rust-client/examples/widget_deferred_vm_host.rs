use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_state::{Interfaces, ScriptEvent},
    protocol::Packet,
    script_runtime::CacheSource,
    vm::{Status, Value},
    widget::Widget,
    Result,
};

fn install(client: &mut ClientState) -> Result<()> {
    client.interfaces = Interfaces::default();
    let mut widget = Widget::empty(0, 0);
    widget.flags = 2046;
    client.interfaces.install_group(0, vec![widget])?;
    client.interfaces.runtime.get_mut(&0).unwrap().item = 777;
    client.active_widgets = [Some(0), None];
    client.outbound.clear();
    client.script_failures.clear();
    Ok(())
}
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5", 64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    client.scripts.source = CacheSource::new(DiskCache::open(
        "tests/fixtures/widget-deferred-vm-cache",
        1 << 20,
    )?);
    for (variant, bytes) in [
        (
            0,
            include_bytes!("../tests/fixtures/widget-deferred-completion-jvm.bin").as_slice(),
        ),
        (
            1,
            include_bytes!("../tests/fixtures/widget-deferred-callback-jvm.bin").as_slice(),
        ),
        (
            2,
            include_bytes!("../tests/fixtures/widget-deferred-mutation-jvm.bin").as_slice(),
        ),
    ] {
        let callbacks = variant == 1;
        let mut input = Reader::new(bytes);
        for case in 0..input.u32()? {
            let requests = input.u32()?;
            let ending = input.u32()?;
            assert_eq!(input.u8()?, 0);
            let mut expected = Vec::new();
            for _ in 0..input.u32()? {
                let opcode = input.u32()? as u16;
                let n = input.u32()? as usize;
                expected.push(Packet {
                    opcode,
                    payload: input.bytes(n)?.to_vec(),
                });
            }
            assert_eq!(input.u32()?, 0);
            assert_eq!(input.u8()?, 0);
            assert_eq!(input.u8()?, 0);
            install(&mut client)?;
            let target = if variant == 2 {
                let id = client.interfaces.create_child(0, 5, 4, false)?;
                client.interfaces.widgets.get_mut(&id).unwrap().flags = 2046;
                client.interfaces.runtime.get_mut(&id).unwrap().item = 777;
                client.active_widgets = [Some(id), None];
                id
            } else {
                0
            };
            if variant != 0 {
                client
                    .interfaces
                    .runtime
                    .get_mut(&target)
                    .unwrap()
                    .listeners
                    .insert(
                        1409,
                        (vec![Value::Int(if callbacks { 43 } else { 45 })], None),
                    );
            }
            let result = client.run_listener(&[Value::Int(case as i32 + 1)], &Default::default());
            assert_eq!(
                result.is_err(),
                ending != 0,
                "variant {variant} case {case}"
            );
            assert_eq!(
                client.outbound.drain(..).collect::<Vec<_>>(),
                expected,
                "variant {variant} case {case}"
            );
            if callbacks {
                assert_eq!(client.script_failures.len(), requests as usize);
            }
        }
        assert_eq!(input.remaining(), 0);
    }
    // The descriptor retains its item, but the target is looked up when dispatched.
    for dynamic in [false, true] {
        install(&mut client)?;
        let id = if dynamic {
            client.interfaces.create_child(0, 5, 4, false)?
        } else {
            0
        };
        client.interfaces.runtime.get_mut(&id).unwrap().item = 777;
        client.interfaces.widgets.get_mut(&id).unwrap().flags = 2046;
        client.active_widgets = [Some(id), None];
        let mut execution = client.begin_event(ScriptEvent {
            widget: id,
            arguments: vec![Value::Int(44)],
            context: Default::default(),
        })?;
        assert_eq!(execution.step(&mut client, 2)?, Status::Yielded);
        assert!(client.outbound.is_empty());
        let current = if dynamic {
            client.interfaces.create_child(0, 5, 4, false)?
        } else {
            id
        };
        client.interfaces.runtime.get_mut(&current).unwrap().item = 999;
        client.interfaces.widgets.get_mut(&current).unwrap().flags = 2046;
        while execution.step(&mut client, 1)? != Status::Finished {}
        assert_eq!(
            client.outbound.pop_front(),
            Some(gielinor_client::interface_action::operation(
                0,
                if dynamic { 5 } else { -1 },
                777,
                1,
                None
            )?)
        );
        assert!(client.outbound.is_empty());
    }
    install(&mut client)?;
    let child = client.interfaces.create_child(0, 5, 4, false)?;
    client.interfaces.widgets.get_mut(&child).unwrap().flags = 2046;
    let runtime = client.interfaces.runtime.get_mut(&child).unwrap();
    runtime.item = 777;
    runtime.listeners.insert(1409, (vec![Value::Int(45)], None));
    client.run_listener(&[Value::Int(46)], &Default::default())?;
    assert_eq!(
        client.outbound.pop_front(),
        Some(gielinor_client::interface_action::operation(
            0, 5, 777, 1, None
        )?)
    );
    assert!(client.outbound.is_empty());
    assert!(client.interfaces.script_child(0, 5).is_none());
    assert!(client.script_failures.is_empty());
    println!("27 original deferred completion/callback/mutation packet traces plus captured-item/current-target and explicit retired-target VM probes passed");
    Ok(())
}
