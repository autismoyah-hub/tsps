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

    let moved_pointer = std::env::args().nth(1).as_deref() == Some("pointer");
    let outside_pointer = std::env::args().nth(1).as_deref() == Some("outside");
    let mut r = Reader::new(include_bytes!("../tests/fixtures/world-walk-range-jvm.bin"));
    let mut checked = 0;
    for case in 0..r.u32()? {
        let view = r.u32()?;
        let owner = r.u32()? as i32;
        let player = [r.u32()? as i32, r.u32()? as i32];
        let target = [r.u32()? as i32, r.u32()? as i32];
        let expected = [r.u32()? as i32, r.u32()? as i32];
        if view != 0
            || ![-2, 0].contains(&owner)
            || player[0] < 0
            || ![[99, 99], [100, 100]].contains(&target)
        {
            continue;
        }
        c.world_entities = Default::default();
        c.detached_world_entities.clear();
        c.view_registry = Default::default();
        c.active_world = 0;
        c.npc_state = Default::default();
        c.zone = Default::default();
        c.outbound.clear();
        c.scene_walk_commands.clear();
        c.actor_cycle += 1;
        if owner == -2 {
            c.player_state = None;
            c.data.position = [0, 100, 100];
        } else {
            let mut init = vec![0; 4608];
            let packed = ((player[0] as u32) << 14) | player[1] as u32;
            for bit in 0..30 {
                if packed & (1 << (29 - bit)) != 0 {
                    init[bit / 8] |= 1 << (7 - bit % 8);
                }
            }
            c.initialize_players(&init, 1)?;
        }
        c.camera.tick_follow(
            &live.world.as_ref().unwrap().0,
            [100 * 128 + 64, 100 * 128 + 64],
            0,
            50,
        );
        live.paint(&mut c, 765, 503)?;
        let pick = |tile: [i32; 2], live: &gielinor_client::live_runtime::Connection| {
            live.pick_where(|tag| {
                tag >> 63 == 1
                    && ((tag >> 14) & 16383) as i32 == tile[0]
                    && (tag & 16383) as i32 == tile[1]
            })
            .map(|(point, _)| point)
            .expect("target terrain tile visible")
        };
        let point = pick(target, &live);
        let pressed = if moved_pointer || outside_pointer {
            pick(
                if target == [99, 99] {
                    [100, 100]
                } else {
                    [99, 99]
                },
                &live,
            )
        } else {
            point
        };
        use gielinor_client::context_menu::Action;
        let action = live
            .menu_entries(&mut c, pressed)?
            .into_iter()
            .map(|entry| entry.action)
            .find(|action| matches!(action.scene(), (0, Action::Walk(_))))
            .expect("root walk action");
        c.input.pointer = point;
        if outside_pointer {
            live.paint(&mut c, 765, 503)?;
            c.input.pointer = [-1, -1];
        }
        c.menu_action_at(action, pressed)?;
        assert!(c.outbound.is_empty());
        assert_eq!(
            live.dispatch_scene_walks(&mut c)?,
            0,
            "selection awaits drawing"
        );
        live.paint(&mut c, 765, 503)?;
        assert_eq!(live.dispatch_scene_walks(&mut c)?, 1);
        let packet = c.outbound.pop_front().unwrap();
        let wanted =
            gielinor_client::world_action::walk(expected[0] as u16, expected[1] as u16, 0)?;
        assert_eq!((packet.opcode,packet.payload),(wanted.opcode,wanted.payload),
            "case{case} pointer_moved{moved_pointer} player{player:?} target{target:?}: original terrain selection");
        assert!(c.outbound.is_empty());
        assert_eq!(live.dispatch_scene_walks(&mut c)?, 0);
        checked += 1;
    }
    assert_eq!(r.remaining(), 0);
    assert_eq!(checked, 12);
    println!("12 original terrain target selections match native rendered picking; moved_pointer={moved_pointer}; outside_pointer={outside_pointer}");
    Ok(())
}
