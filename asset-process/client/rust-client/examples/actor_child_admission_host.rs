use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    npc_info::{Appearance, Npc},
    scene::Scene,
    world_actors::Actors,
    Result,
};
fn main() -> Result<()> {
    let cache = std::env::args().nth(1).expect("cache path");
    let mut client = ClientState::load(DiskCache::open(cache, 64 << 20)?)?;
    client.data.position = [0, 50, 50];
    let actors = Actors::load(&mut client.scripts.source.cache)?;
    let mut d = client.npcs[&16299].clone();
    d.id = 60000;
    d.transforms = Some(vec![Some(16299), None]);
    d.transform_varbit = None;
    d.transform_varp = Some(42);
    client.npcs.insert(60000, d);
    client.npc_state.npcs.insert(
        1,
        Npc {
            visual: Default::default(),
            path: {
                let mut p = gielinor_client::actor_path::Path::default();
                p.teleport([50, 50], 1);
                p.position[0] += 1;
                p.fine[0] += 1.;
                p
            },
            sequence: Default::default(),
            pose: Default::default(),
            idle_fallback: Default::default(),
            spots: Default::default(),
            overlays: Default::default(),
            turn: Default::default(),
            facing: None,
            animations: client.npcs[&60000].animations,
            freezes: Default::default(),
            exact: None,
            id: 60000,
            position: [50, 50],
            angle: 0,
            spawn_cycle: None,
            steps: vec![],
            jump: false,
            blocks: vec![],
            appearance: Appearance::default(),
        },
    );
    client.npc_state.indices.push(1);
    let template = client.npc_state.npcs.remove(&1).unwrap();
    let base_definition = client.npcs[&16299].clone();
    let target_varp =
        gielinor_client::overhead::overlay_varps(&mut client.scripts.source.cache)?[1].unwrap();
    let scene = Scene::empty_sized(0, 0, 16, 16)?;
    let appearance = gielinor_client::player_appearance::Update {
        appearance: Default::default(),
        skull: -1,
        overhead: -1,
        animations: [-1; 7],
        name: "Admission".into(),
        combat: 3,
        skill: 0,
        hidden: false,
        force_refresh: false,
        actions: Default::default(),
        team: 0,
    };
    let mut r = gielinor_client::buffer::Reader::new(include_bytes!(
        "../tests/fixtures/actor-child-admission-jvm.bin"
    ));
    for case in 0..r.u32()? {
        let local = r.u32()? as u16;
        let target = r.u32()? as u16;
        client.preferences.render_self = r.u8()? != 0;
        let (mut state, _) = gielinor_client::player_info::State::initialize(&[0; 4608], local)?;
        state.high.clear();
        client.npc_state.indices.clear();
        client.npc_state.npcs.clear();
        for kind in 0..2 {
            for _ in 0..r.u32()? {
                let id = r.u32()? as u16;
                let priority = r.u32()? as u8;
                let size = r.u32()? as u8;
                let fine = [r.u32()? as i32, r.u32()? as i32];
                let visible = r.u8()? != 0;
                if kind == 0 {
                    let mut a = appearance.clone();
                    a.hidden = !visible;
                    let mut p = gielinor_client::player_info::Player::at(
                        [0, fine[0] >> 7, fine[1] >> 7],
                        Some(a),
                        1,
                        None,
                    );
                    p.path.position = fine;
                    state.players[id as usize] = Some(p);
                    state.high.push(id);
                } else {
                    client.npc_state.indices.push(id);
                    if visible {
                        let mut definition = base_definition.clone();
                        definition.id = 50000 + id as u32;
                        definition.priority = priority;
                        definition.size = size;
                        definition.transforms = None;
                        let mut npc = template.clone();
                        npc.id = definition.id as u16;
                        npc.path.position = fine;
                        client.npcs.insert(definition.id, definition);
                        client.npc_state.npcs.insert(id, npc);
                    }
                }
            }
        }
        state.local_view = r.u32()? as u16;
        client.player_state = Some(state);
        {
            use gielinor_client::vm::Host;
            let mut stack = gielinor_client::vm::Stacks::default();
            stack.push_int(r.u32()? as i32)?;
            client.extended(7900, false, &mut stack)?;
        }
        let mut world = gielinor_client::world_entity::State::default();
        for _ in 0..r.u32()? {
            let id = r.u32()? as u16;
            let mut e = gielinor_client::world_entity::Entity::default();
            e.owner_type = r.u32()? as u8;
            e.pose.x = r.u32()? as i32;
            e.pose.z = r.u32()? as i32;
            e.pose.angle = r.u32()? as i32;
            world.visible.push(id);
            world.entities.insert(id, e);
        }
        client.world_entities.insert(0, world);
        let mut expected_actors = Vec::new();
        let mut expected_children = std::collections::HashSet::new();
        for _ in 0..r.u32()? {
            let kind = r.u32()?;
            let index = r.u32()? as u16;
            if kind == 6 {
                expected_children.insert(index);
            } else {
                expected_actors.push(gielinor_client::actor_submission::Key {
                    npc: kind == 1,
                    index,
                });
            }
        }
        let target = (target != 0).then_some(target);
        client
            .scripts
            .variables
            .from_server(target_varp, target.map_or(-1, i32::from));
        assert_eq!(
            gielinor_client::world_actors::submission_plan(&mut client, &scene, None, target)?,
            expected_actors,
            "actors case {case}"
        );
        assert_eq!(
            actors.root_child_submissions(&mut client, &scene)?,
            expected_children,
            "children case {case}"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!(
        "1024 original mixed actor/child admission cases matched runtime root submission plans"
    );
    Ok(())
}
