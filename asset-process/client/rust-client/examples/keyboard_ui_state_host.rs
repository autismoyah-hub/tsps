use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, input::Input, script_runtime::CacheSource,
    Result,
};
#[path = "../tests/support/keyboard_ui_state.rs"]
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
        "tests/fixtures/keyboard-ui-state-vm-cache",
        1 << 20,
    )?);
    client.interfaces = support::install()?;
    client.canvas = [200, 200];
    let mut pending = Input::default();
    for (cycle, case) in support::cases()?.iter().enumerate() {
        support::apply(case, &mut client.input, &mut pending)?;
        support::check(case, &client.input, cycle);
        support::check(case, &pending, cycle);
        client.scripts.variables.set_player(0, 0);
        client.scripts.variables.set_player(1, 0);
        client.tick_interfaces(false)?;
        assert!(
            client.script_failures.is_empty(),
            "cycle{cycle}: {:?}",
            client.script_failures
        );
        assert_eq!(
            client.scripts.variables.player(0),
            case.callbacks.len() as i32,
            "cycle{cycle} count"
        );
        let hash = case.callbacks.iter().fold(0i32, |h, &(op, key)| {
            h.wrapping_mul(31)
                .wrapping_add(op)
                .wrapping_mul(31)
                .wrapping_add(key)
        });
        assert_eq!(
            client.scripts.variables.player(1),
            hash,
            "cycle{cycle} order/key fingerprint"
        );
    }
    println!("512 original UI keyboard cycles match states, bounded lists and cache-script callback key/count/order");
    Ok(())
}
