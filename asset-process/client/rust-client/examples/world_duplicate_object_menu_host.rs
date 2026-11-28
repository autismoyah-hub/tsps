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
    let definitions = gielinor_client::object::Definition::load_all(&mut c.scripts.source.cache)?;
    let object = live
        .world
        .as_ref()
        .unwrap()
        .0
        .objects
        .iter()
        .find(|object| {
            !object.rejected
                && object.shape == 10
                && definitions.get(&object.id).is_some_and(|d| {
                    d.transforms.is_none()
                        && d.animation.is_none()
                        && !d.models.is_empty()
                        && d.interactive != 0
                        && d.size[0] <= 2
                        && d.size[1] <= 2
                })
        })
        .expect("small cache scene object")
        .clone();
    println!("Native fixture object {} shape {}", object.id, object.shape);
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
        "../tests/fixtures/world-duplicate-object-menu-jvm.bin"
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
        let direct = r.u32()?;
        let _direct = direct;
        let tile = 2;
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
            e.definition = Some(gielinor_client::world_entity::Definition::decode(
                0,
                &[14, 23, 1, 0],
            )?);
            e.pose.x = 6400 + e.dimensions[0] * 64 - tile * 128;
            e.pose.z = e.pose.x;
            e.steps[0].pose = e.pose;
            let label = if slot == 0 { "Direct" } else { "Nested" };
            e.locations.apply(
                [0, tile, tile],
                gielinor_client::location_update::Change {
                    at: 0,
                    object: gielinor_client::location_update::Object {
                        id: object.id as i32,
                        shape: object.shape,
                        rotation: object.rotation,
                    },
                    operations: if slot == 0 { mask } else { 31 - mask } as u8,
                    labels: Some(std::array::from_fn(|i| Some(format!("{label}{i}")))),
                },
                |_| gielinor_client::location_update::Object {
                    id: -1,
                    shape: object.shape,
                    rotation: object.rotation,
                },
            )?;
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
                let _ = (index, target);
                expected.push((opcode, option));
            }
        }
        c.camera
            .tick_follow(&live.world.as_ref().unwrap().0, [6400, 6400], 0, 50);
        live.paint(&mut c, 765, 503)?;
        let direct_generation = c.world_entities[&0].entities[&id].spawn_generation;
        let nested_generation = c.world_entities[&7].entities[&id].spawn_generation;
        assert!(
            live.instance_scene(id, direct_generation)
                .unwrap()
                .objects
                .iter()
                .any(|o| o.id == object.id),
            "case{case} direct object scene"
        );
        assert!(
            live.instance_scene(id, nested_generation)
                .unwrap()
                .objects
                .is_empty(),
            "case{case} nested object scene remains empty"
        );
        let (point, _) = live
            .pick_where_view(|view, tag| view == id && tag >> 32 == object.id as u64 + 1)
            .expect("controlled root child object pick");
        let entries = live.menu_entries(&mut c, point)?;
        let mut actual = entries
            .into_iter()
            .filter_map(|e| match e.action.scene().1 {
                gielinor_client::context_menu::Action::Object { operation, .. } => Some((
                    if operation == 5 {
                        1001
                    } else {
                        2 + operation as u32
                    },
                    e.option,
                )),
                gielinor_client::context_menu::Action::Examine { kind: 1, .. } => {
                    Some((1002, e.option))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        actual.sort();
        expected.sort();
        assert_eq!(
            actual, expected,
            "case{case} root-owned object pending-menu membership"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original duplicate-world object menus match native rendered child picking");
    Ok(())
}
