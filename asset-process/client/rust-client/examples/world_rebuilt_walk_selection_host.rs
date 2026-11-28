use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, game_packet,
    protocol::Packet, scene::Scene,
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
    let scene = Scene::empty(0, 0)?;
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
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/world-rebuilt-walk-selection-jvm.bin"
    ));
    let count = r.u32()?;
    for case in 0..count {
        c.world_entities = Default::default();
        c.detached_world_entities.clear();
        c.active_world = 0;
        c.view_registry = Default::default();
        c.player_state = None;
        c.npc_state = Default::default();
        c.zone = Default::default();
        c.data.position = [0, 50, 50];
        c.actor_cycle += 1;
        let id = r.u32()? as u16;
        let _order = r.u32()?;
        for _ in 0..r.u32()? {
            let opcode = r.u32()? as u16;
            let n = r.u32()? as usize;
            let payload = r.bytes(n)?.to_vec();
            c.apply(game_packet::decode(&Packet { opcode, payload })?.unwrap())?;
        }
        let _oldest = r.u32()?;
        let _direct = r.u32()?;
        let _nested = r.u32()?;
        let mut rebuild_packets = Vec::new();
        for _ in 0..r.u32()? {
            let opcode = r.u32()? as u16;
            let n = r.u32()? as usize;
            rebuild_packets.push(Packet {
                opcode,
                payload: r.bytes(n)?.to_vec(),
            });
        }
        let ready = r.u8()? != 0;
        let _java_selected = [r.u32()? as i32, r.u32()? as i32];
        let expected_base = [r.u32()? as i32, r.u32()? as i32];
        for parent in [0, 7] {
            let e = c
                .world_entities
                .get_mut(&parent)
                .unwrap()
                .entities
                .get_mut(&id)
                .unwrap();
            e.definition = Some(gielinor_client::world_entity::Definition::decode(
                0,
                &[14, 23, 1, 0],
            )?);
            e.pose.x = 6400 + e.dimensions[0] * 64 - 4 * 128;
            e.pose.z = e.pose.x;
            e.steps[0].pose = e.pose;
            if parent == 0 {
                let dimensions = e.dimensions.map(|v| v as usize);
                let mut chunks = vec![None; 4 * (dimensions[0] / 8) * (dimensions[1] / 8)];
                chunks[0] = Some(chunk);
                e.rebuild = Some(gielinor_client::rebuild::WorldEntity {
                    base: [0, 0],
                    dimensions,
                    chunks,
                    regions: vec![chunk.region()],
                });
                e.rebuild_generation += 1;
            }
        }
        c.camera
            .tick_follow(&live.world.as_ref().unwrap().0, [6400, 6400], 0, 50);
        live.paint(&mut c, 765, 503)?;
        let generation = c.world_entities[&0].entities[&id].spawn_generation;
        assert!(
            live.instance_scene(id, generation)
                .unwrap()
                .missing_regions
                .is_empty(),
            "actual terrain available"
        );
        let (point, _) = live
            .pick_where_view(|view, tag| view == id && tag & 0x8000000000000000 != 0)
            .expect("root child terrain pick");
        use gielinor_client::context_menu::Action;
        let action = live
            .menu_entries(&mut c, point)?
            .into_iter()
            .map(|e| e.action)
            .find(|a| a.scene().0 == id && matches!(a.scene().1, Action::Walk(_)))
            .expect("child walk menu entry");
        c.input.pointer = point;
        c.prepare_scene_menu(point);
        live.paint(&mut c, 765, 503)?;
        let captured = live.instance_scene(id, generation).unwrap().walk.selected;
        assert!(captured[0] >= 0, "direct scene captured a terrain tile");
        for packet in rebuild_packets {
            c.apply(game_packet::decode(&packet)?.unwrap())?;
        }
        // Same registered instance, new geometry and possibly a shifted base.
        live.paint(&mut c, 765, 503)?;
        let survivor_generation = c.view_entity(id).unwrap().spawn_generation;
        let rebuilt = live.instance_scene(id, survivor_generation).unwrap();
        assert_eq!([rebuilt.base_x, rebuilt.base_y], expected_base);
        c.outbound.clear();
        c.menu_action_from_open_menu(action)?;
        assert!(
            c.outbound.is_empty(),
            "menu accept does not send immediately"
        );
        assert_eq!(
            live.dispatch_scene_walks(&mut c)?,
            usize::from(ready),
            "case{case}: retained walk selection after same-instance geometry rebuild"
        );
        assert_eq!(c.outbound.len(), usize::from(ready));
        if ready {
            let expected = gielinor_client::world_action::walk(
                captured[0].wrapping_add(expected_base[0]) as u16,
                captured[1].wrapping_add(expected_base[1]) as u16,
                0,
            )?;
            let actual = c.outbound.pop_front().unwrap();
            assert_eq!(
                (actual.opcode, actual.payload),
                (expected.opcode, expected.payload)
            );
        }
        assert_eq!(
            live.dispatch_scene_walks(&mut c)?,
            0,
            "selection emits once after rebuild"
        );
        c.outbound.clear();
    }
    assert_eq!(r.remaining(), 0);
    println!(
        "64 original packet109 geometry rebuilds preserve native walk selection and view base"
    );
    Ok(())
}
