use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, input::Input, interface_state::Interfaces,
    script_runtime::CacheSource, vm::Value, widget::Widget, Result,
};
#[path = "../tests/support/native_delete.rs"]
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
        "tests/fixtures/keyboard-ring-cycle-vm-cache",
        1 << 20,
    )?);
    client.interfaces = Interfaces::default();
    let mut w = Widget::empty(0, 4);
    w.raw_size = [100, 100];
    client.interfaces.install_group(0, vec![w])?;
    client.interfaces.root = Some(0);
    client.canvas = [200, 200];
    client
        .interfaces
        .runtime
        .get_mut(&0)
        .unwrap()
        .listeners
        .insert(
            1419,
            (
                vec![
                    Value::Int(1419),
                    Value::Int(-2147483640),
                    Value::Int(-2147483639),
                ],
                None,
            ),
        );
    let mut pending = Input::buffered();
    for (step, case) in support::cases()?.iter().enumerate() {
        support::dispatch(case, &mut client.input, &mut pending)?;
        client.scripts.variables.set_player(0, 0);
        client.scripts.variables.set_player(1, 0);
        client.cycle += 1;
        client.tick_interfaces_elapsed(1, &mut pending, false)?;
        assert_eq!(
            client.scripts.variables.player(0),
            0,
            "early Delete callback"
        );
        client.cycle += 1;
        client.tick_interfaces_elapsed(1, &mut pending, false)?;
        assert!(
            client.script_failures.is_empty(),
            "{:?}",
            client.script_failures
        );
        assert_eq!(
            client.scripts.variables.player(0),
            case.keys.len() as i32,
            "Delete step{step} repeat{} count",
            case.repeat
        );
        let hash = case.keys.iter().fold(0i32, |h, &(key, ch)| {
            [1419, key, ch]
                .into_iter()
                .fold(h, |a, v| a.wrapping_mul(31).wrapping_add(v))
        });
        assert_eq!(
            client.scripts.variables.player(1),
            hash,
            "Delete step{step} callback order"
        );
    }
    println!(
        "128 native AWT Delete transitions match actual-cache buffered key and typed callbacks"
    );
    Ok(())
}
