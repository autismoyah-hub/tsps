//! Exercise widget queries on real login interfaces, with no synthetic server updates.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_script::execute,
    live_runtime::Connection,
    vm::{Object, Stacks},
};
use std::time::{Duration, Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let mut live = Connection::connect(
        &mut client,
        "127.0.0.1:43617",
        "rustclient239",
        "work/kotlin-world/test-account.password",
        "work/kotlin-world/.data/client.key",
    )?;
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        live.poll(&mut client)?;
        if live.world.is_some()
            && live.session.is_established()
            && !live.scripts_busy()
            && client.interfaces.scripts.is_empty()
            && !client.interfaces.children.is_empty()
            && client
                .interfaces
                .widgets
                .values()
                .any(|w| w.actions.iter().any(|action| action == "Inventory"))
        {
            break;
        }
        assert!(Instant::now() < deadline, "login/interface timeout");
        std::thread::sleep(Duration::from_millis(20));
    }
    client.canvas = [765, 503];
    if let Some((target,plane,height))=live.camera_follow_target(&client) {
        client.camera.tick_follow(&live.world.as_ref().unwrap().0,target,plane,client.preferences.camera_follow_height);
        client.camera.set_target_height(height,client.preferences.camera_follow_height);
    }
    client.camera.pitch=2048;
    client.camera.yaw=0;
    live.paint(&mut client, 765, 503)?;
    // Freeze network polling; every following mutation is local to this probe.
    let group = 65533;
    let base = (group as u32) << 16;
    let mut viewport = gielinor_client::widget::Widget::empty(base, 4);
    viewport.content = 1337;
    viewport.raw_size = [765, 503];
    let mut overlay = gielinor_client::widget::Widget::empty(base + 1, 4);
    overlay.raw_size = [24, 24];
    overlay.hidden = true;
    overlay.no_click_through = true;
    client
        .interfaces
        .install_group(group, vec![viewport, overlay])?;
    client.interfaces.root = Some(group);
    live.paint(&mut client, 765, 503)?;
    let mut chosen = None;
    'scan: for y in (32..480).step_by(32) {
        for x in (32..740).step_by(32) {
            let point = [x, y];
            client.input.pointer = point;
            let entries = live.menu_entries(&mut client, point)?;
            if !entries.is_empty() {
                chosen = Some((
                    point,
                    entries.into_iter().map(|e| e.action).collect::<Vec<_>>(),
                ));
                break 'scan;
            }
        }
    }
    let (point, expected) = chosen.expect("visible live world menu");
    let overlay = client.interfaces.widgets.get_mut(&(base + 1)).unwrap();
    overlay.hidden = false;
    overlay.raw_position = [point[0] - 12, point[1] - 12];
    assert!(
        live.menu_entries(&mut client, point)?.is_empty(),
        "IF3 overlay must clear world menu"
    );
    client
        .interfaces
        .widgets
        .get_mut(&(base + 1))
        .unwrap()
        .no_click_through = false;
    assert_eq!(
        live.menu_entries(&mut client, point)?
            .into_iter()
            .map(|e| e.action)
            .collect::<Vec<_>>(),
        expected
    );
    client
        .interfaces
        .widgets
        .get_mut(&(base + 1))
        .unwrap()
        .no_click_through = true;
    client.input.pointer = [0, 0];
    assert_eq!(
        live.menu_entries(&mut client, point)?
            .into_iter()
            .map(|e| e.action)
            .collect::<Vec<_>>(),
        expected,
        "press point is distinct from barrier pointer"
    );
    client.input.pointer = point;
    client
        .interfaces
        .widgets
        .get_mut(&(base + 1))
        .unwrap()
        .hidden = true;
    assert_eq!(
        live.menu_entries(&mut client, point)?
            .into_iter()
            .map(|e| e.action)
            .collect::<Vec<_>>(),
        expected
    );
    println!("Live world menu barrier at {point:?}: {} actions suppressed, restored for transparent/hidden overlay and distinct pointer",expected.len());
    Ok(())
}
