use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, interface_events::Events,
    script_runtime::CacheSource, Result,
};
#[path = "../tests/support/widget_pointer_suppression.rs"]
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
        "tests/fixtures/widget-event-order-vm-cache",
        1 << 20,
    )?);
    client.canvas = [200, 200];
    client.preferences.mouse_camera = false;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-pointer-suppression-jvm.bin"
    ));
    assert_eq!(r.u32()?, 96);
    for count in 1..=4 {
        for pattern in 0..3 {
            client.interfaces = support::order::install(count, 7)?;
            client.interface_events = Events::default();
            for tick in 0..8 {
                assert_eq!([r.u32()?, r.u32()?, r.u32()?], [count, pattern, tick]);
                let expected = support::order::expected(&mut r)?;
                client.input = support::order::input(tick as usize);
                client.scripts.variables.set_player(0, 0);
                client.scripts.variables.set_player(1, 0);
                client.tick_interfaces(support::suppressed(pattern, tick))?;
                assert!(
                    client.script_failures.is_empty(),
                    "widgets{count} pattern{pattern} tick{tick}: {:?}",
                    client.script_failures
                );
                assert_eq!(
                    client.scripts.variables.player(0),
                    expected.len() as i32,
                    "widgets{count} pattern{pattern} tick{tick} count"
                );
                let hash = expected
                    .iter()
                    .fold(0i32, |h, &id| h.wrapping_mul(31).wrapping_add(id));
                assert_eq!(
                    client.scripts.variables.player(1),
                    hash,
                    "widgets{count} pattern{pattern} tick{tick} dispatch order"
                );
            }
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("96 original menu-suppressed pointer/wheel traces match exact collected order and cache-loaded ClientState tick script count/order fingerprints");
    Ok(())
}
