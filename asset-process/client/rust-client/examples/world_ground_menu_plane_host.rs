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
        assert!(std::time::Instant::now() < deadline, "scene login deadline");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
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
        "../tests/fixtures/world-ground-menu-planes-jvm.bin"
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
        let root_plane = r.u32()? as i32;
        let local_plane = r.u32()? as u8;
        let config_plane = r.u32()? as u8;
        let target = r.u32()? as u16;
        let local = r.u32()? as u16;
        let expected = (0..r.u32()?)
            .map(|_| r.u32())
            .collect::<Result<Vec<_>, _>>()?;
        c.data.position = [root_plane, 50, 50];
        for (retained, id) in [(0u8, 7u16), (1, 8)] {
            let mut payload = vec![retained];
            if retained != 0 {
                payload.extend_from_slice(&[1, 0]);
            }
            payload.extend_from_slice(&[0, id as u8, 0, 0xbc, 128, 0, 0, 0]);
            c.apply(
                game_packet::decode(&Packet {
                    opcode: 122,
                    payload,
                })?
                .unwrap(),
            )?;
            let e = c
                .world_entities
                .get_mut(&0)
                .unwrap()
                .entities
                .get_mut(&id)
                .unwrap();
            e.plane = if id == 7 {
                local_plane
            } else {
                3 - root_plane as u8
            };
            let config = if id == 7 {
                config_plane
            } else {
                (config_plane + 1) % 4
            };
            e.definition = Some(gielinor_client::world_entity::Definition::decode(
                0,
                &[2, config, 14, 23, 1, 0],
            )?);
            e.pose.x = if id == 7 { 6016 } else { 6784 };
            e.pose.z = 6400;
            e.steps[0].pose = e.pose;
        }
        let mut state = gielinor_client::player_info::State::initialize(&vec![0; 4608], 1)?.0;
        state.local_view = local;
        c.player_state = Some(state);
        for plane in 0..4 {
            let mut item = c.items.items[&995].clone();
            item.id = 100 + plane as u32;
            item.name = format!("Plane{plane}");
            item.ground_operations.clear();
            c.items.items.insert(item.id, item);
            let item = gielinor_client::zone::Item {
                id: 100 + plane as u16,
                quantity: 1,
                operations: 0,
                visible: 0,
                despawn: i32::MAX,
                ownership: 0,
                private: false,
                custom: None,
            };
            c.zone.piles.insert([plane, 50, 50], vec![item.clone()]);
            for id in [7, 8] {
                c.world_entities
                    .get_mut(&0)
                    .unwrap()
                    .entities
                    .get_mut(&id)
                    .unwrap()
                    .zone
                    .piles
                    .insert([plane, 16, 16], vec![item.clone()]);
            }
        }
        c.camera.tick_follow(
            &live.world.as_ref().unwrap().0,
            [6400, 6400],
            root_plane as usize,
            50,
        );
        live.paint(&mut c, 765, 503)?;
        let (point, _) = live
            .pick_where_view(|view, tag| view == target && tag & 0x1000000000000000 != 0)
            .expect("controlled target ground pick");
        let entries = live.menu_entries(&mut c, point)?;
        let actual = entries
            .into_iter()
            .filter_map(|entry| {
                let (view, action) = entry.action.scene();
                if view != target {
                    return None;
                }
                match action {
                    gielinor_client::context_menu::Action::Examine { kind: 2, id, .. } => {
                        Some(id as u32 - 100)
                    }
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(actual,expected,"case{case}: root plane {root_plane}, local plane {local_plane}, config {config_plane}, target {target}, local {local}");
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original ground-menu plane selections match native rendered picking");
    Ok(())
}
