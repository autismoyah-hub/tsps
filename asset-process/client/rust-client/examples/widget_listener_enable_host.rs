use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_events::Events,
    script_runtime::CacheSource,
    vm::{Host, Value, Variable},
    Result,
};
#[path = "../tests/support/widget_listener_enable.rs"]
mod support;
fn main() -> Result<()> {
    support::verify()?;
    support::verify_editor_setters()?;
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    client.scripts.source = CacheSource::new(DiskCache::open(
        "tests/fixtures/widget-listener-enable-vm-cache",
        1 << 20,
    )?);
    client.canvas = [200, 200];
    client.preferences.mouse_camera = false;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-listener-enable-jvm.bin"
    ));
    assert_eq!(r.u32()?, 192);
    for count in 1..=4 {
        for enabled in 0..2 {
            for attach_tick in [0, 3, 6] {
                client.interfaces = support::install(count, enabled != 0)?;
                client.interface_events = Events::default();
                client.interface_events.changed(1407, 100);
                client.interface_events.changed(1418, 0);
                for tick in 0..8 {
                    assert_eq!(
                        [r.u32()?, r.u32()?, r.u32()?, r.u32()?],
                        [count, enabled, attach_tick, tick]
                    );
                    let expected = support::order::expected(&mut r)?;
                    if tick == attach_tick {
                        support::attach(&mut client.interfaces, count);
                    }
                    client.input = support::order::input(tick as usize);
                    for id in [0, 1] {
                        client
                            .scripts
                            .write(Variable::ClientInt, id, Value::Int(0))?;
                    }
                    client.tick_interfaces(false)?;
                    assert!(
                        client.script_failures.is_empty(),
                        "count{count} enabled{enabled} attach{attach_tick} tick{tick}: {:?}",
                        client.script_failures
                    );
                    let actual = |value: Value| match value {
                        Value::Int(v) => v,
                        _ => panic!("integer marker"),
                    };
                    assert_eq!(
                        actual(client.scripts.read(Variable::ClientInt, 0)?),
                        expected.len() as i32,
                        "count{count} enabled{enabled} attach{attach_tick} tick{tick} count"
                    );
                    let hash = expected
                        .iter()
                        .fold(0i32, |h, &id| h.wrapping_mul(31).wrapping_add(id));
                    assert_eq!(
                        actual(client.scripts.read(Variable::ClientInt, 1)?),
                        hash,
                        "count{count} enabled{enabled} attach{attach_tick} tick{tick} order"
                    );
                }
            }
        }
    }
    assert_eq!(r.remaining(), 0);
    println!(
        "192 original listener-enable history traces match actual cache-loaded ClientState ticks"
    );
    Ok(())
}
