use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, input::Input, interface_events::Events,
    script_runtime::CacheSource, Result,
};
#[path = "../tests/support/widget_event_order.rs"]
mod order;
#[path = "../tests/support/mouse_button_cycle.rs"]
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
        "tests/fixtures/widget-event-order-vm-cache",
        1 << 20,
    )?);
    client.canvas = [200, 200];
    let mut pending = Input::default();
    for case in support::cases()? {
        if case.tick == 0 {
            client.interfaces = order::install(2, 7)?;
            client.interface_events = Events::default();
        }
        client.preferences.mouse_camera = case.camera;
        support::apply(&case, &mut client.input, &mut pending)?;
        support::check(&case, &client.input);
        support::check(&case, &pending);
        client.scripts.variables.set_player(0, 0);
        client.scripts.variables.set_player(1, 0);
        client.tick_interfaces(false)?;
        assert!(
            client.script_failures.is_empty(),
            "camera{} tick{}: {:?}",
            case.camera,
            case.tick,
            client.script_failures
        );
        assert_eq!(
            client.scripts.variables.player(0),
            case.callbacks.len() as i32,
            "camera{} tick{} callback count",
            case.camera,
            case.tick
        );
        let hash = case
            .callbacks
            .iter()
            .fold(0i32, |h, &v| h.wrapping_mul(31).wrapping_add(v));
        assert_eq!(
            client.scripts.variables.player(1),
            hash,
            "camera{} tick{} callback order",
            case.camera,
            case.tick
        );
    }
    println!("1024 original multi-button cycles match native Input consumers and cache-loaded ClientState callback count/order fingerprints");
    Ok(())
}
