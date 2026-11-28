//! Real login, then a local IF_RESYNC replay; synthetic updates never reach the server.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_packet::{Events, Update},
    live_runtime::Connection,
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
    live.paint(&mut client, 765, 503)?;
    let mounts = client.interfaces.children.clone();
    assert!(!mounts.is_empty());
    let model = client
        .interfaces
        .runtime
        .keys()
        .copied()
        .find(|id| client.interfaces.widgets[id].kind == 0)
        .unwrap();
    client.interfaces.runtime.get_mut(&model).unwrap().frame = 7;
    let snapshot = client
        .interfaces
        .runtime
        .iter()
        .map(|(&id, r)| (id, (r.instance, r.frame, r.frame_cycle)))
        .collect::<std::collections::HashMap<_, _>>();
    let layout = client.interfaces.layout(client.canvas)?;
    let geometry = layout
        .order
        .iter()
        .map(|&id| {
            let p = &layout.widgets[&id];
            (id, p.absolute, p.geometry.size, p.clip, p.hidden)
        })
        .collect::<Vec<_>>();
    let mut children = mounts.values().cloned().collect::<Vec<_>>();
    children.sort_by_key(|s| s.parent);
    for child in &mut children {
        child.kind = child.kind.wrapping_add(1);
    }
    let events = client
        .interfaces
        .events
        .iter()
        .map(|(&(widget, child), &masks)| Events {
            widget,
            first: child,
            last: child,
            masks,
        })
        .collect();
    client.interface(Update::Resync {
        root: client.interfaces.root.unwrap(),
        children,
        events,
    })?;
    assert_eq!(client.interfaces.children, mounts, "modal kinds changed");
    assert!(
        client.interfaces.scripts.is_empty(),
        "resync queued redundant load/close callbacks"
    );
    let after = client
        .interfaces
        .runtime
        .iter()
        .map(|(&id, r)| (id, (r.instance, r.frame, r.frame_cycle)))
        .collect::<std::collections::HashMap<_, _>>();
    assert_eq!(
        after, snapshot,
        "widget identity or animation clocks changed"
    );
    let layout = client.interfaces.layout(client.canvas)?;
    let after = layout
        .order
        .iter()
        .map(|&id| {
            let p = &layout.widgets[&id];
            (id, p.absolute, p.geometry.size, p.clip, p.hidden)
        })
        .collect::<Vec<_>>();
    assert_eq!(after, geometry);
    live.paint(&mut client, 765, 503)?;
    assert!(client.script_failures.is_empty());
    println!("Retained {} real interface mounts and {} widget identities/clocks; unchanged layout and rendered frame",mounts.len(),snapshot.len());
    live.session.close();
    Ok(())
}
