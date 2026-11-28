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
#[path = "../tests/support/widget_remote_queue.rs"]
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
        "tests/fixtures/widget-remote-queue-vm-cache",
        1 << 20,
    )?);
    client.canvas = [200, 200];
    client.preferences.mouse_camera = false;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-remote-queue-jvm.bin"
    ));
    assert_eq!(r.u32()?, 20);
    for case in 0..20 {
        let count = r.u32()?;
        let remote = r.u32()?;
        let mounted = r.u32()? != 0;
        let expected = (0..r.u32()?)
            .map(|_| r.u32().map(|v| v as i32))
            .collect::<Result<Vec<_>>>()?;
        client.interfaces = support::install(count, remote, mounted)?;
        client.interface_events = Events::default();
        client.interface_events.changed(1407, 100);
        client.input = Input {
            pointer: [-1, -1],
            ..Default::default()
        };
        for id in [0, 1] {
            client
                .scripts
                .write(Variable::ClientInt, id, Value::Int(0))?;
        }
        client.tick_interfaces(false)?;
        assert!(
            client.script_failures.is_empty(),
            "case{case}: {:?}",
            client.script_failures
        );
        let actual = |value: Value| match value {
            Value::Int(v) => v,
            _ => panic!("integer marker"),
        };
        assert_eq!(
            actual(client.scripts.read(Variable::ClientInt, 0)?),
            expected.len() as i32,
            "case{case} count"
        );
        let hash = expected
            .iter()
            .fold(0i32, |h, &id| h.wrapping_mul(31).wrapping_add(id));
        assert_eq!(
            actual(client.scripts.read(Variable::ClientInt, 1)?),
            hash,
            "case{case} order"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("20 original remote-ready resize/transmit queue traces match actual cache-loaded ClientState ticks");
    Ok(())
}
