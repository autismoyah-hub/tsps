use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, scene::Scene,
};
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
    chunks[168] = Some(chunk);
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

    c.world_entities = Default::default();
    c.detached_world_entities.clear();
    c.view_registry = Default::default();
    c.active_world = 0;
    c.npc_state = Default::default();
    c.zone = Default::default();
    c.player_state = None;
    let mut r = Reader::new(include_bytes!("../tests/fixtures/world-walk-plane-jvm.bin"));
    let mut checked = 0;
    let mut rejected = 0;
    for case in 0..r.u32()? {
        let view = r.u32()?;
        let level = r.u32()?;
        let far_plane = r.u32()?;
        let near_plane = r.u32()?;
        let order = r.u32()?;
        let _selected = [r.u32()? as i32, r.u32()? as i32];
        let ready = r.u8()? != 0;
        if view != 0 || far_plane != near_plane || order != 0 {
            continue;
        }
        c.data.position = [level as i32, 100, 100];
        c.outbound.clear();
        c.scene_walk_commands.clear();
        c.actor_cycle += 1;
        // Reuse cached surface geometry while controlling its physical-plane tag.
        // The fixture isolates picker eligibility; it is not a four-floor cache reconstruction.
        for triangle in &mut live.world.as_mut().unwrap().1.triangles {
            if triangle.tag >> 63 == 1 {
                triangle.tag = (triangle.tag & !(3_u64 << 28)) | (near_plane as u64) << 28;
            }
        }
        c.camera.tick_follow(
            &live.world.as_ref().unwrap().0,
            [100 * 128 + 64, 100 * 128 + 64],
            0,
            50,
        );
        live.paint(&mut c, 765, 503)?;
        let point = live
            .pick_where(|tag| {
                tag >> 63 == 1 && ((tag >> 14) & 16383) == 100 && (tag & 16383) == 100
            })
            .map(|(point, _)| point)
            .expect("controlled terrain remains drawn");
        c.input.pointer = point;
        use gielinor_client::context_menu::Action;
        let has_walk = live
            .menu_entries(&mut c, point)?
            .iter()
            .any(|entry| matches!(entry.action.scene(), (0, Action::Walk(_))));
        assert_eq!(
            has_walk, ready,
            "case{case} level{level} surface{near_plane}: original plane menu eligibility"
        );
        // Exercise the armed draw even when no movement menu is available.
        c.menu_action_at(Action::Walk([100, 100]), point)?;
        assert!(c.outbound.is_empty());
        assert_eq!(live.dispatch_scene_walks(&mut c)?, 0);
        live.paint(&mut c, 765, 503)?;
        assert_eq!(
            live.dispatch_scene_walks(&mut c)?,
            usize::from(ready),
            "case{case}: original plane readiness"
        );
        if ready {
            let packet = c.outbound.pop_front().unwrap();
            let expected = gielinor_client::world_action::walk(100, 100, 0)?;
            assert_eq!(
                (packet.opcode, packet.payload),
                (expected.opcode, expected.payload)
            );
        } else {
            rejected += 1;
        }
        assert!(c.outbound.is_empty());
        assert_eq!(live.dispatch_scene_walks(&mut c)?, 0);
        checked += 1;
    }
    assert_eq!(r.remaining(), 0);
    assert_eq!((checked, rejected), (16, 6));
    println!("16 original plane eligibility cases match native movement menus and deferred picking; 6 upper-plane targets rejected");
    Ok(())
}
