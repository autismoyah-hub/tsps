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
    let scene = Scene::empty(0, 0)?;
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
    let mut worlds = gielinor_client::world_entity::State::default();
    for id in 7..13 {
        let mut entity = gielinor_client::world_entity::Entity::default();
        entity.pose.x = 6464;
        entity.pose.z = 6464;
        entity.owner_type = 0;
        worlds.entities.insert(id, entity);
        worlds.visible.push(id);
    }
    client.world_entities.insert(0, worlds);
    let expected = (7..12).collect::<std::collections::HashSet<_>>();
    assert_eq!(
        actors.root_child_submissions(&mut client, &scene)?,
        expected
    );
    assert!(
        gielinor_client::world_actors::submission_plan(&mut client, &scene, None, None)?.is_empty()
    );
    for entity in client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .values_mut()
    {
        entity.owner_type = 1;
    }
    assert_eq!(
        actors.root_child_submissions(&mut client, &scene)?,
        (7..11).collect()
    );
    assert_eq!(
        gielinor_client::world_actors::submission_plan(&mut client, &scene, None, None)?.len(),
        1
    );
    let (mut players, _) = gielinor_client::player_info::State::initialize(&[0; 4608], 1)?;
    players.local_view = 12;
    client.player_state = Some(players);
    assert_eq!(
        actors.root_child_submissions(&mut client, &scene)?,
        std::collections::HashSet::from([12, 7, 8, 9])
    );
    // Effects precede the late child category, and delayed graphics do not reserve slots.
    client.world_spots.push(
        gielinor_client::world_spot::Request {
            id: 0,
            height: 0,
            delay: 0,
            position: [0, 50, 50],
        }
        .start(0),
    );
    assert_eq!(
        actors.root_child_submissions(&mut client, &scene)?,
        std::collections::HashSet::from([12, 7, 8])
    );
    client.world_spots[0].animation.start = client.actor_cycle + 1;
    assert_eq!(
        actors.root_child_submissions(&mut client, &scene)?,
        std::collections::HashSet::from([12, 7, 8, 9])
    );
    client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .get_mut(&12)
        .unwrap()
        .pose
        .x = 0;
    assert_eq!(
        actors.root_child_submissions(&mut client, &scene)?,
        std::collections::HashSet::from([7, 8, 9, 10])
    );
    client.world_spots.clear();
    for entity in client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .values_mut()
    {
        entity.owner_type = 0;
        entity.pose.x = 6464;
    }
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
    let state = client.player_state.as_mut().unwrap();
    state.local_view = 0;
    state.high = vec![1, 2];
    for id in [1, 2] {
        let mut p = gielinor_client::player_info::Player::at(
            [0, 50, 50],
            Some(appearance.clone()),
            1,
            None,
        );
        p.path.position[0] += 1;
        state.players[id] = Some(p);
    }
    client.preferences.render_self = true;
    let target =
        gielinor_client::overhead::overlay_varps(&mut client.scripts.source.cache)?[1].unwrap();
    client.scripts.variables.from_server(target, -1);
    assert_eq!(
        actors.root_child_submissions(&mut client, &scene)?,
        (7..11).collect()
    );
    assert_eq!(
        gielinor_client::world_actors::submission_plan(&mut client, &scene, None, None)?,
        vec![gielinor_client::actor_submission::Key {
            npc: false,
            index: 1
        }]
    );
    client.scripts.variables.from_server(target, 2);
    assert_eq!(
        actors.root_child_submissions(&mut client, &scene)?,
        (7..10).collect()
    );
    assert_eq!(
        gielinor_client::world_actors::submission_plan(&mut client, &scene, None, Some(2))?,
        vec![
            gielinor_client::actor_submission::Key {
                npc: false,
                index: 1
            },
            gielinor_client::actor_submission::Key {
                npc: false,
                index: 2
            }
        ]
    );
    use gielinor_client::vm::Host;
    for value in [0, 1, 2, -1, i32::MAX] {
        let mut stack = gielinor_client::vm::Stacks::default();
        stack.push_int(239)?;
        stack.push_int(value)?;
        client.extended(7900, false, &mut stack)?;
        client.extended(7901, false, &mut stack)?;
        assert_eq!(stack.ints(), &[239, value.max(0)]);
        let expected = (7..7 + value.clamp(0, 3) as u16).collect();
        assert_eq!(
            actors.root_child_submissions(&mut client, &scene)?,
            expected
        );
    }
    client.player_state.as_mut().unwrap().local_view = 12;
    let mut stack = gielinor_client::vm::Stacks::default();
    stack.push_int(0)?;
    client.extended(7900, false, &mut stack)?;
    assert_eq!(
        actors.root_child_submissions(&mut client, &scene)?,
        std::collections::HashSet::from([12])
    );
    assert_eq!(
        gielinor_client::world_overlap::frame_order(
            &client.world_entities,
            12,
            client.world_entity_limit.get()
        )?,
        vec![(0, 12)]
    );
    stack.push_int(17)?;
    client.extended(7900, false, &mut stack)?;
    client.prepare_login()?;
    client.extended(7901, false, &mut stack)?;
    assert_eq!(stack.ints(), &[17]);
    println!("Root child admission: early/local/late categories compete with actors and world effects; boundary rejection and per-frame reset passed");
    Ok(())
}
