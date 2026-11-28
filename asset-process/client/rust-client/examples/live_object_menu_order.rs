//! Exercise widget queries on real login interfaces, with no synthetic server updates.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
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
    if let Some((target, plane, height)) = live.camera_follow_target(&client) {
        client.camera.tick_follow(
            &live.world.as_ref().unwrap().0,
            target,
            plane,
            client.preferences.camera_follow_height,
        );
        client
            .camera
            .set_target_height(height, client.preferences.camera_follow_height);
    }
    client.camera.pitch = 2048;
    client.camera.yaw = 0;
    live.paint(&mut client, 765, 503)?;
    // Scene/menu mutations below are local; polling temporarily restores the real interface root.
    let original_root = client.interfaces.root;
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
    use gielinor_client::{
        buffer::Reader,
        context_menu::Action,
        location_update::{Change, Event, Object},
    };
    let definitions =
        gielinor_client::object::Definition::load_all(&mut client.scripts.source.cache)?;
    let mut chosen = None;
    'scan: for y in (32..480).step_by(16) {
        for x in (32..740).step_by(16) {
            let point = [x, y];
            client.input.pointer = point;
            for entry in live.menu_entries(&mut client, point)? {
                if let Action::Object { id, tile, .. } = entry.action {
                    let scene = &live.world.as_ref().unwrap().0;
                    if let Some(placed) = scene.objects.iter().find(|o| {
                        !o.rejected
                            && o.id == id as u32
                            && o.plane as i32 == client.data.position[0]
                            && o.x as i32 + scene.base_x == tile[0] as i32
                            && o.y as i32 + scene.base_y == tile[1] as i32
                    }) {
                        if definitions
                            .get(&(id as u32))
                            .is_some_and(|d| d.transforms.is_none() && d.animation.is_none())
                        {
                            chosen = Some((point, id, tile, placed.clone()));
                            break 'scan;
                        }
                    }
                }
            }
        }
    }
    let (point, id, tile, placed) = chosen.expect("visible static root object");
    let key = [
        placed.plane as i32,
        tile[0] as i32,
        tile[1] as i32,
        gielinor_client::location_update::category(placed.shape)? as i32,
    ];
    // Local removal makes the pending add retain operation labels instead of retiring as unchanged.
    live.world.as_mut().unwrap().0.replace_location(
        key,
        &Object {
            id: -1,
            shape: placed.shape,
            rotation: placed.rotation,
        },
        &definitions,
    )?;
    let mut r = Reader::new(include_bytes!("../tests/fixtures/object-menu-order-jvm.bin"));
    for case in 0..r.u32()? {
        let mask = r.u32()?;
        for _ in 0..r.u32()? {
            r.u32()?;
        }
        let expected = (0..r.u32()?)
            .map(|_| r.u32())
            .collect::<gielinor_client::Result<Vec<_>>>()?
            .into_iter()
            .filter(|code| *code != 23)
            .collect::<Vec<_>>();
        client.zone.locations.push_back(Event::Change(
            [key[0], key[1], key[2]],
            Change {
                at: 0,
                object: Object {
                    id: id as i32,
                    shape: placed.shape,
                    rotation: placed.rotation,
                },
                operations: mask as u8,
                labels: Some(std::array::from_fn(|i| Some(format!("Op{i}")))),
            },
        ));
        client.interfaces.root = original_root;
        live.poll_elapsed(&mut client, 0)?;
        client.interfaces.root = Some(group);
        client.input.pointer = point;
        live.paint(&mut client, 765, 503)?;
        let actual = live
            .context_menu(&mut client, point)?
            .into_iter()
            .filter_map(|e| match e.action {
                Action::Object {
                    id: got,
                    tile: at,
                    operation,
                    ..
                } if got == id && at == tile => Some(if operation == 5 {
                    1001
                } else {
                    operation as u32 + 2
                }),
                Action::Examine {
                    kind: 1,
                    id: got,
                    tile: at,
                } if got == id && at == tile => Some(1002),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "case {case} object {id} at {tile:?}");
    }
    assert_eq!(r.remaining(), 0);
    println!("32 live picked-object operation masks match original sorted menus at {point:?}");
    Ok(())
}
