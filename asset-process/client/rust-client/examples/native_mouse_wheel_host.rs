use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    input::{self, Input, Update},
    interface_events::Events,
    script_runtime::CacheSource,
    vm::{Host, Value, Variable},
    Result,
};
#[path = "../tests/support/widget_event_order.rs"]
mod order;
#[path = "../tests/support/native_mouse_wheel.rs"]
mod support;

fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    client.scripts.source = CacheSource::new(DiskCache::open(
        "tests/fixtures/widget-wheel-context-vm-cache",
        1 << 20,
    )?);
    client.canvas = [200, 200];
    client.interfaces = order::install(1, 0)?;
    client.interface_events = Events::default();
    client
        .interfaces
        .runtime
        .get_mut(&0)
        .unwrap()
        .listeners
        .insert(
            1417,
            (
                vec![
                    Value::Int(1417),
                    Value::Int(-2147483647),
                    Value::Int(-2147483646),
                ],
                None,
            ),
        );
    let mut pending = Input::default();
    input::dispatch(
        Update::Pointer([30, 50]),
        [&mut client.input, &mut pending],
        None,
        |_| None,
        |_| false,
    )?;
    let mut wheel = support::NativeWheel::default();
    for (step, case) in support::cases()?.into_iter().enumerate() {
        client.input.begin_cycle();
        pending.begin_cycle();
        let delta = support::normalize(&mut wheel, &case);
        input::dispatch(
            Update::Wheel(delta),
            [&mut client.input, &mut pending],
            None,
            |_| None,
            |_| false,
        )?;
        for id in 0..3 {
            client
                .scripts
                .write(Variable::ClientInt, id, Value::Int(-999))?;
        }
        client.tick_interfaces(false)?;
        assert!(
            client.script_failures.is_empty(),
            "step{step}: {:?}",
            client.script_failures
        );
        let expected = if case.expected == 0 {
            [-999; 3]
        } else {
            [1, 0, case.expected]
        };
        for (id, want) in expected.into_iter().enumerate() {
            let got = client.scripts.read(Variable::ClientInt, id as i32)?;
            assert!(
                matches!(got, Value::Int(v) if v==want),
                "step{step} varc{id}: {got:?}, expected {want}"
            );
        }
        assert_eq!(client.input.wheel, case.expected, "step{step} frame wheel");
        assert_eq!(pending.wheel, case.expected, "step{step} pending wheel");
    }
    println!("1024 original native wheel messages match cache-loaded widget callback arguments and both Input consumers");
    Ok(())
}
