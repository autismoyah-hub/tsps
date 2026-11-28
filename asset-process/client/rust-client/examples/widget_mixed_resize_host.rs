use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    input::Input,
    interface_events::Events,
    interface_state::Interfaces,
    script_runtime::CacheSource,
    vm::{Host, Value, Variable},
    widget::Widget,
    Result,
};
fn trace(r: &mut Reader<'_>) -> Result<Vec<i32>> {
    (0..r.u32()?).map(|_| r.u32().map(|v| v as i32)).collect()
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
        "tests/fixtures/widget-mixed-resize-vm-cache",
        1 << 20,
    )?);
    client.canvas = [200, 200];
    client.preferences.mouse_camera = false;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-mixed-resize-jvm.bin"
    ));
    assert_eq!(r.u32()?, 36);
    for case in 0..36 {
        let phase = r.u32()?;
        let count = r.u32()?;
        let position = r.u32()?;
        let initial = r.u32()?;
        client.interfaces = Interfaces::default();
        client.interface_events = Events::default();
        client.interfaces.install_group(
            0,
            (0..count)
                .map(|id| {
                    let mut w = Widget::empty(id, 4);
                    w.raw_size = [100, 100];
                    w
                })
                .collect(),
        )?;
        client.interfaces.root = Some(0);
        // Seed release transitions with listener processing enabled, as GameWidget.ec.
        for runtime in client.interfaces.runtime.values_mut() {
            runtime.listeners.insert(1400, (Vec::new(), None));
        }
        let layout = client.interfaces.layout(client.canvas)?;
        let held = Input {
            pointer: [10, 10],
            press_position: [10, 10],
            buttons: 1,
            pressed_buttons: 1,
            ..Default::default()
        };
        assert!(client
            .interface_events
            .collect(&client.interfaces, &layout, &held, false, false)
            .is_empty());
        for id in 0..count {
            for (p, op) in [1408, 1402, 1419].into_iter().enumerate() {
                client
                    .interfaces
                    .runtime
                    .get_mut(&id)
                    .unwrap()
                    .listeners
                    .insert(
                        op,
                        (
                            vec![Value::Int(if p as u32 == phase && id == position {
                                42
                            } else {
                                43
                            })],
                            None,
                        ),
                    );
            }
        }
        client
            .interfaces
            .runtime
            .get_mut(&0)
            .unwrap()
            .listeners
            .insert(1427, (vec![Value::Int(99)], None));
        if initial != 0 {
            assert!(client.interfaces.queue_resize(0, 0)?);
        }
        client.input = Input {
            pointer: [10, 10],
            key_events: vec![(1, 0)],
            ..Default::default()
        };
        for round in 0..2 {
            let expected = trace(&mut r)?;
            client
                .scripts
                .write(Variable::ClientInt, 0, Value::Int(0))?;
            client
                .scripts
                .write(Variable::ClientInt, 1, Value::Int(0))?;
            client.tick_interfaces(false)?;
            assert!(
                client.script_failures.is_empty(),
                "case{case} round{round}: {:?}",
                client.script_failures
            );
            assert_eq!(
                match client.scripts.read(Variable::ClientInt, 0)? {
                    Value::Int(v) => v,
                    _ => panic!("integer trace"),
                },
                expected.len() as i32,
                "case{case} round{round} count"
            );
            let hash = expected
                .iter()
                .fold(0i32, |h, &id| h.wrapping_mul(31).wrapping_add(id));
            assert_eq!(
                match client.scripts.read(Variable::ClientInt, 1)? {
                    Value::Int(v) => v,
                    _ => panic!("integer trace"),
                },
                hash,
                "case{case} round{round} order"
            );
            for runtime in client.interfaces.runtime.values_mut() {
                runtime.listeners.clear();
            }
            client.input = Input::default();
        }
    }
    assert_eq!(r.remaining(), 0);
    println!(
        "36 original mixed-phase resize traces match two actual cache-loaded ClientState ticks"
    );
    Ok(())
}
