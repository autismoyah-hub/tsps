use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    input::Input,
    interface_events::Events,
    script_runtime::CacheSource,
    vm::{Host, Value, Variable},
    Result,
};
#[path = "../tests/support/widget_wheel_context.rs"]
mod support;
fn main() -> Result<()> {
    support::verify()?;
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
    client.preferences.mouse_camera = false;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-wheel-context-jvm.bin"
    ));
    assert_eq!(r.u32()?, 72);
    for case in 0..72 {
        let current = r.u32()? as i32;
        let press = r.u32()? as i32;
        let button = r.u32()?;
        let menu = r.u32()? != 0;
        let wheel = r.u32()? as i32;
        let expected = if r.u8()? != 0 {
            Some([r.u32()? as i32, r.u32()? as i32])
        } else {
            None
        };
        client.interfaces = support::order::install(1, 0)?;
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
        client.interface_events = Events::default();
        client.input = Input {
            pointer: [current, 50],
            press_position: [press, 50],
            pressed_buttons: button as u8,
            wheel,
            ..Default::default()
        };
        for id in 0..3 {
            client
                .scripts
                .write(Variable::ClientInt, id, Value::Int(-999))?;
        }
        client.tick_interfaces(menu)?;
        assert!(
            client.script_failures.is_empty(),
            "case{case}: {:?}",
            client.script_failures
        );
        let expected = expected.map(|p| [1, p[0], p[1]]).unwrap_or([-999; 3]);
        for (id, value) in expected.into_iter().enumerate() {
            let actual = client.scripts.read(Variable::ClientInt, id as i32)?;
            assert!(
                matches!(actual,Value::Int(v) if v==value),
                "case{case} varc{id}: {actual:?}, expected {value}"
            );
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("72 original wheel context cases match cache-loaded script argument substitution through ClientState ticks");
    Ok(())
}
