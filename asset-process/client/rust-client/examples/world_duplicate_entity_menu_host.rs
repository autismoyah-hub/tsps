use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, game_packet, npc_info::Npc,
    protocol::Packet, scene::Scene,
};
fn npc(point: i32) -> Npc {
    Npc {
        visual: Default::default(),
        path: {
            let mut p = gielinor_client::actor_path::Path::default();
            p.position = [point; 2];
            p
        },
        sequence: Default::default(),
        pose: Default::default(),
        idle_fallback: Default::default(),
        spots: Default::default(),
        overlays: Default::default(),
        turn: Default::default(),
        facing: None,
        freezes: Default::default(),
        exact: None,
        id: 16299,
        position: [point >> 7; 2],
        angle: 0,
        spawn_cycle: None,
        steps: Vec::new(),
        jump: false,
        blocks: Vec::new(),
        appearance: Default::default(),
        animations: [-1; 15],
    }
}
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
        "../tests/fixtures/world-duplicate-entity-menu-jvm.bin"
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
        let mask = r.u32()?;
        for (slot, parent) in [0, 7].into_iter().enumerate() {
            let e = c
                .world_entities
                .get_mut(&parent)
                .unwrap()
                .entities
                .get_mut(&id)
                .unwrap();
            let mut def = gielinor_client::world_entity::Definition::decode(
                321 + slot as i32,
                &[14, 23, 0, 0],
            )?;
            def.name = if slot == 0 { "Direct" } else { "Nested" }.into();
            def.operations = std::array::from_fn(|i| Some(format!("Op{i}")));
            def.bounds = [0, 0, 512, 512];
            e.definition = Some(def);
            e.options = if slot == 0 { mask } else { 31 - mask } as i32;
            e.pose.x = 6400;
            e.pose.z = 6400;
            e.steps[0].pose = e.pose;
            if slot == 0 {
                e.npcs.indices = vec![41];
                e.npcs.insertion_order = vec![41];
                e.npcs.npcs.insert(41, npc(e.dimensions[0] * 64));
            }
        }
        let mut expected = Vec::new();
        for _ in 0..r.u32()? {
            let opcode = r.u32()?;
            let index = r.u32()?;
            let n = r.u16()? as usize;
            let option = String::from_utf8(r.bytes(n)?.to_vec())?;
            let n = r.u16()? as usize;
            let target = String::from_utf8(r.bytes(n)?.to_vec())?;
            if opcode != 23 {
                expected.push((opcode, index, option, target));
            }
        }
        c.camera
            .tick_follow(&live.world.as_ref().unwrap().0, [6400, 6400], 0, 50);
        live.paint(&mut c, 765, 503)?;
        let (point, _) = live
            .pick_where_view(|view, tag| view == id && tag == 0x4000000000000029)
            .expect("controlled root child NPC pick");
        let entries = live.menu_entries(&mut c, point)?;
        let mut actual = entries
            .into_iter()
            .filter_map(|e| match e.action.scene().1 {
                gielinor_client::context_menu::Action::WorldEntity { id, operation } => Some((
                    if operation == 6 {
                        1013
                    } else {
                        62 + operation as u32
                    },
                    id as u32,
                    e.option,
                    e.target,
                )),
                _ => None,
            })
            .collect::<Vec<_>>();
        actual.sort();
        expected.sort();
        assert_eq!(
            actual, expected,
            "case{case} root-owned entity menu membership"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original duplicate-world entity menus match native rendered child picking");
    Ok(())
}
