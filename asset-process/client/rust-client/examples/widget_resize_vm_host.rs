use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, interface_state::Interfaces,
    script_event::Context, script_runtime::CacheSource, vm::Value, widget::Widget, Result,
};

fn install(client: &mut ClientState) -> Result<()> {
    client.interfaces = Interfaces::default();
    client
        .interfaces
        .install_group(0, vec![Widget::empty(0, 0)])?;
    client.active_widgets = [Some(0), None];
    client
        .interfaces
        .runtime
        .get_mut(&0)
        .unwrap()
        .listeners
        .insert(1427, (vec![Value::Int(42)], None));
    Ok(())
}
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    client.scripts.source = CacheSource::new(DiskCache::open(
        "tests/fixtures/widget-resize-vm-cache",
        1 << 20,
    )?);
    let mut input = Reader::new(include_bytes!(
        "../tests/fixtures/widget-resize-queue-order-jvm.bin"
    ));
    let cases = input.u32()?;
    for case in 0..cases {
        let initial = input.u32()?;
        let depth = input.u32()? as i32;
        install(&mut client)?;
        for _ in 0..initial {
            client.interfaces.queue_resize(0, depth)?;
        }
        for round in 0..2 {
            client.drain_resize_events();
            let expected_count = input.u32()? as usize;
            let expected_listener = input.u8()? != 0;
            let queue = client.interfaces.resize_events.borrow();
            assert_eq!(queue.len(), expected_count, "case {case} round {round}");
            assert_eq!(
                client.interfaces.listener(0, 1427).is_some(),
                expected_listener
            );
            for queued in queue.iter() {
                assert_eq!(queued.event.context.resize_depth, input.u32()? as i32);
            }
            assert!(client.script_failures.is_empty());
        }
    }
    assert_eq!(input.remaining(), 0);
    // A callback failure must not strand the next event or contaminate its depth.
    install(&mut client)?;
    client.interfaces.queue_resize(0, 10)?;
    client
        .interfaces
        .runtime
        .get_mut(&0)
        .unwrap()
        .listeners
        .insert(1427, (vec![Value::Int(43)], None));
    client.interfaces.queue_resize(0, 0)?;
    client.drain_resize_events();
    assert_eq!(client.script_failures.len(), 1);
    assert_eq!(client.scripts.variables.player(0), 42);
    assert!(client.interfaces.resize_events.borrow().is_empty());
    // Independent listeners reset depth after an error.
    install(&mut client)?;
    assert!(client
        .run_listener(
            &[Value::Int(42)],
            &Context {
                resize_depth: 10,
                ..Default::default()
            }
        )
        .is_err());
    client.run_listener(&[Value::Int(42)], &Context::default())?;
    assert_eq!(
        client
            .interfaces
            .resize_events
            .borrow()
            .front()
            .unwrap()
            .event
            .context
            .resize_depth,
        1
    );
    install(&mut client)?;
    let mut execution = client.begin_event(gielinor_client::interface_state::ScriptEvent {
        widget: 0,
        arguments: vec![Value::Int(42)],
        context: Context {
            resize_depth: 8,
            ..Default::default()
        },
    })?;
    while execution.step(&mut client, 1)? != gielinor_client::vm::Status::Finished {}
    assert_eq!(
        client
            .interfaces
            .resize_events
            .borrow()
            .front()
            .unwrap()
            .event
            .context
            .resize_depth,
        9
    );
    install(&mut client)?;
    client.interfaces.root = Some(0);
    client
        .interfaces
        .widgets
        .get_mut(&0)
        .unwrap()
        .size_alignment = [1, 1];
    client
        .interfaces
        .runtime
        .get_mut(&0)
        .unwrap()
        .listeners
        .insert(1427, (vec![Value::Int(43)], None));
    client.resize_canvas([800, 600])?;
    assert_eq!(client.canvas, [800, 600]);
    {
        let queue = client.interfaces.resize_events.borrow();
        assert_eq!(queue.len(), 1);
        assert_eq!(queue[0].event.context.resize_depth, 0);
    }
    client.resize_canvas([800, 600])?;
    assert_eq!(client.interfaces.resize_events.borrow().len(), 1);
    assert!(client.drain_resize_events() > 0);
    assert_eq!(client.scripts.variables.player(0), 42);
    assert!(client.interfaces.resize_events.borrow().is_empty());
    println!(
        "Automatic canvas-change callback executed at depth zero through the ClientState VM host"
    );
    println!("{cases} original queue-order cases (two drains each), failure continuation and event-depth isolation passed");
    Ok(())
}
