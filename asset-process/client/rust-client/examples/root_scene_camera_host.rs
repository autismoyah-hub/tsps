use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, scene::Scene};
#[path = "../tests/support/root_scene_camera.rs"]
mod support;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut c = ClientState::load(cache)?;
    let mut live = gielinor_client::live_runtime::Connection::connect(
        &mut c,
        "127.0.0.1:43617",
        "rustclient239",
        "work/kotlin-world/test-account.password",
        "work/kotlin-world/.data/client.key",
    )?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(90);
    while live.world.is_none() || !live.session.is_established() || live.minimap_view(&c).is_none()
    {
        live.poll(&mut c)?;
        assert!(
            std::time::Instant::now() < deadline,
            "scene login deadline: title={:?}, established={}",
            live.title.as_ref().map(|t| t.screen),
            live.session.is_established()
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let source_tile = [3200, 3200]; // Explicit cached terrain source, independent of instance destination coordinates.
    let chunk = gielinor_client::rebuild::Chunk {
        plane: 0,
        zone_x: (source_tile[0] / 8) as u16,
        zone_y: (source_tile[1] / 8) as u16,
        rotation: 0,
    };
    println!(
        "Native terrain fixture region {} chunk {},{}",
        chunk.region(),
        chunk.zone_x,
        chunk.zone_y
    );
    // Freeze polling and replay the controlled local scenario without sending fixture packets.
    let mut chunks = vec![None; 676];
    for index in [0, 12, 156, 168] {
        chunks[index] = Some(chunk);
    }
    let scene = Scene::load_world_entity(
        &mut c.scripts.source.cache,
        &gielinor_client::rebuild::WorldEntity {
            base: [0, 0],
            dimensions: [104, 104],
            chunks,
            regions: vec![chunk.region()],
        },
    )?;
    let mesh =
        gielinor_client::world_mesh::WorldMesh::build(&scene, &mut c.scripts.source.cache, 0)?;
    live.world = Some((scene, mesh));
    c.world_base = [0, 0];
    c.canvas = [765, 503];
    c.interfaces = Default::default();
    let group = 65533;
    let mut viewport = gielinor_client::widget::Widget::empty((group as u32) << 16, 4);
    viewport.content = 1337;
    viewport.raw_size = [765, 503];
    c.interfaces.install_group(group, vec![viewport])?;
    c.interfaces.root = Some(group);
    c.preferences.device.roofs_hidden = true;
    c.camera.pitch = 2048;
    c.camera.yaw = 0;

    c.player_state = None;
    c.npc_state = Default::default();
    c.world_entities = Default::default();
    c.detached_world_entities.clear();
    c.view_registry = Default::default();
    c.active_world = 0;
    c.zone = Default::default();
    c.data.position = [0, 50, 50];
    c.camera.locked = true;
    c.outbound.clear();
    let mut checked = 0;
    let mut model_checks = 0;
    for (index, case) in support::cases()?.iter().take(32).enumerate() {
        assert_eq!(case.size, [104, 104]);
        c.camera.eye = case.eye;
        c.camera.rendered_angles = case.angles;
        let actual = live.paint(&mut c, 765, 503)?;
        let actual_pick = live.pick_where(|tag| tag >> 63 == 1);
        let model_point = live.pick_where(|tag| tag >> 63 == 0 && tag >> 32 != 0);
        let actual_actions = if let Some((point, _)) = model_point {
            c.input.pointer = point;
            Some(
                live.menu_entries(&mut c, point)?
                    .into_iter()
                    .map(|entry| entry.action)
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        };
        assert_eq!(
            c.camera.eye, case.eye,
            "global camera changed at edge{index}"
        );
        c.camera.eye = case.expected;
        let expected = live.paint(&mut c, 765, 503)?;
        let expected_pick = live.pick_where(|tag| tag >> 63 == 1);
        assert!(
            expected_pick.is_some(),
            "original edge{index} has no cached terrain"
        );
        let differences = actual
            .pixels()
            .iter()
            .zip(expected.pixels())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            differences, 0,
            "root edge{index} raw{:?} original{:?}",
            case.eye, case.expected
        );
        assert_eq!(actual_pick, expected_pick, "root edge{index} terrain pick");
        if let (Some((point, _)), Some(actual_actions)) = (model_point, actual_actions) {
            let expected_actions = live
                .menu_entries(&mut c, point)?
                .into_iter()
                .map(|entry| entry.action)
                .collect::<Vec<_>>();
            assert_eq!(
                actual_actions, expected_actions,
                "root edge{index} model selection"
            );
            assert!(
                expected_actions.iter().any(|action| matches!(
                    action,
                    gielinor_client::context_menu::Action::Object { .. }
                        | gielinor_client::context_menu::Action::Examine { kind: 1, .. }
                )),
                "root edge{index} has no selectable cached object"
            );
            model_checks += 1;
        }
        checked += 1;
    }
    assert_eq!(checked, 32);
    assert!(c.outbound.is_empty());
    assert!(model_checks > 0, "no cached model selection exercised");
    println!("{model_checks} cached object menu comparisons use the original root camera");
    println!("32 original root scene edge cameras match actual-cache live rendering pixels and terrain picks; global camera retained");
    Ok(())
}
