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
    live.paint(&mut client, 765, 503)?;
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let mut owners = std::collections::HashMap::<u32, usize>::new();
    for &(owner, _) in client.interfaces.dynamic.keys() {
        *owners.entry(owner).or_default() += 1;
    }
    let (owner, count) = owners
        .into_iter()
        .max_by_key(|&(id, count)| (count, id))
        .expect("real dynamic interfaces");
    let expected = client
        .interfaces
        .dynamic
        .iter()
        .filter(|((p, _), _)| *p == owner)
        .map(|((_, index), _)| *index)
        .collect::<std::collections::BTreeSet<_>>();
    let mut active = [Some(owner), None];
    let mut stack = Stacks::default();
    for value in [0, owner as i32, -1] {
        stack.push_int(value)?;
    }
    execute(
        211,
        false,
        &mut stack,
        &mut cache,
        &mut client.interfaces,
        &mut active,
        client.canvas,
        &client.items,
    )?;
    assert_eq!(stack.pop_int()?, count as i32);
    execute(
        215,
        false,
        &mut stack,
        &mut cache,
        &mut client.interfaces,
        &mut active,
        client.canvas,
        &client.items,
    )?;
    let Object::Array(indices) = stack.pop_object()? else {
        panic!("query array")
    };
    let gielinor_client::vm::Array::Ints(values) = indices.borrow().data().clone() else {
        panic!("int query array")
    };
    assert_eq!(values, expected.iter().copied().collect::<Vec<_>>());
    let mut visited = Vec::new();
    loop {
        execute(
            213,
            false,
            &mut stack,
            &mut cache,
            &mut client.interfaces,
            &mut active,
            client.canvas,
            &client.items,
        )?;
        if stack.pop_int()? == 0 {
            break;
        }
        visited.push(client.interfaces.runtime[&active[0].unwrap()].child_index);
    }
    assert_eq!(visited, values);
    let mounts = client
        .interfaces
        .children
        .values()
        .cloned()
        .collect::<Vec<_>>();
    let mut checked = 0;
    for mount in mounts {
        let roots = client
            .interfaces
            .widgets
            .values()
            .filter(|w| {
                client.interfaces.runtime[&w.id].logical_id >> 16 == mount.group as u32
                    && w.parent.is_none()
            })
            .map(|w| w.id)
            .collect::<Vec<_>>();
        for id in roots {
            stack.push_int(id as i32)?;
            execute(
                2506,
                false,
                &mut stack,
                &mut cache,
                &mut client.interfaces,
                &mut active,
                client.canvas,
                &client.items,
            )?;
            assert_eq!(stack.pop_int()?, mount.parent as i32);
            checked += 1;
        }
    }
    assert!(checked > 0, "mounted root parent checks");
    live.paint(&mut client, 765, 503)?;
    assert!(client.script_failures.is_empty());
    println!("Queried {count} real dynamic widgets from owner {owner}; verified {checked} mounted root parents and rendered frame");
    live.session.close();
    Ok(())
}
