use gielinor_client::*;
// Cache-backed live actor render integration: morph hiding, custom bodies and player copies.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    npc_info::{Appearance, Npc},
    scene::Scene,
    Result,
};
use world_actors::Actors;
fn main() -> Result<()> {
    let cache = std::env::args().nth(1).expect("cache path");
    let mut client = ClientState::load(DiskCache::open(cache, 64 << 20)?)?;
    client.data.position = [0, 50, 50];
    let scene = Scene::empty(0, 0)?;
    let mut actors = Actors::load(&mut client.scripts.source.cache)?;
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
    let template = client.npc_state.npcs.remove(&1).unwrap();
    for (index, priority) in [(1, 1), (2, 0)] {
        let mut definition = client.npcs[&16299].clone();
        definition.id = 60000 + index as u32;
        definition.priority = priority;
        definition.size = 1;
        definition.transforms = None;
        let mut npc = template.clone();
        npc.id = definition.id as u16;
        client.npcs.insert(definition.id, definition);
        client.npc_state.npcs.insert(index, npc);
    }
    client.npc_state.indices = vec![2, 1];
    let kits = identity_kit::Kit::load_all(&mut client.scripts.source.cache)?;
    let base = player_model::Appearance::standard(&kits, 0);
    let appearance = player_appearance::Update {
        appearance: base,
        skull: -1,
        overhead: -1,
        animations: [-1; 7],
        name: "TileFixture".into(),
        combat: 3,
        skill: 0,
        hidden: false,
        force_refresh: false,
        actions: Default::default(),
        team: 0,
    };
    let (mut players, _) = player_info::State::initialize(&[0; 4608], 1)?;
    for index in [1, 2] {
        players.players[index] = Some(player_info::Player::at(
            [0, 50, 50],
            Some(appearance.clone()),
            1,
            None,
        ));
    }
    players.high = vec![2, 1];
    client.player_state = Some(players);
    client.preferences.render_self = true;
    use actor_submission::Key;
    let p = |index| Key { npc: false, index };
    let n = |index| Key { npc: true, index };
    assert_eq!(
        world_actors::submission_plan(&mut client, &scene, None, None)?,
        vec![p(1)]
    );
    let (triangles, pending) = actors.triangles(&mut client, &scene, &Default::default())?;
    assert!(pending.is_empty());
    assert!(!triangles.is_empty());
    assert!(triangles.iter().all(|t| t.tag == 0x2000000000000001));
    client.preferences.render_self = false;
    assert_eq!(
        world_actors::submission_plan(&mut client, &scene, None, None)?,
        vec![n(1)]
    );
    let (triangles, pending) = actors.triangles(&mut client, &scene, &Default::default())?;
    assert!(pending.is_empty());
    assert!(!triangles.is_empty());
    assert!(triangles.iter().all(|t| t.tag == 0x4000000000000001));
    client.npcs.get_mut(&60001).unwrap().transforms = Some(vec![None]);
    assert_eq!(
        world_actors::submission_plan(&mut client, &scene, None, None)?,
        vec![p(2)]
    );
    client.npcs.get_mut(&60001).unwrap().transforms = None;
    client.npcs.get_mut(&60001).unwrap().size = 2;
    client.preferences.render_self = true;
    assert_eq!(
        world_actors::submission_plan(&mut client, &scene, None, None)?,
        vec![p(1), n(1)]
    );
    client.npcs.get_mut(&60001).unwrap().size = 1;
    let model_id = client.npcs[&16299].models[0];
    let mut models = item_model::Models::default();
    let model = (*models.raw(&mut client.scripts.source.cache, model_id)?).clone();
    client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(2)
        .unwrap()
        .attachment = Some(location_merge::Attachment {
        placement: location_merge::Placement {
            start: 0,
            end: 10,
            centre: [50 * 128 + 64, 0, 50 * 128 + 64],
            bounds: [49, 49, 51, 51],
        },
        model,
        colors: Vec::new(),
    });
    client.actor_cycle = 0;
    assert_eq!(
        world_actors::submission_plan(&mut client, &scene, None, None)?,
        vec![p(1), p(2)]
    );
    client.actor_cycle = 10;
    assert_eq!(
        world_actors::submission_plan(&mut client, &scene, None, None)?,
        vec![p(1)]
    );
    client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(2)
        .unwrap()
        .attachment = None;
    client.preferences.render_self = true;
    for npc in client.npc_state.npcs.values_mut() {
        npc.path.position[0] += 1;
    }
    for (_, player) in client.player_state.as_mut().unwrap().root_iter_mut() {
        player.path.position[0] += 1;
    }
    assert_eq!(
        world_actors::submission_plan(&mut client, &scene, None, None)?,
        vec![p(1), n(1), p(2), n(2)]
    );
    assert_eq!(
        world_actors::submission_plan(&mut client, &scene, None, Some(2))?,
        vec![p(1), p(2), n(1), n(2)]
    );
    let (triangles, pending) = actors.triangles(&mut client, &scene, &Default::default())?;
    assert!(pending.is_empty());
    let plan = world_actors::submission_plan(&mut client, &scene, None, None)?;
    let mut ordered = triangles.clone();
    ordered.sort_by_key(|t| {
        plan.iter()
            .position(|key| key.npc == (t.tag >> 60 == 4) && key.index == t.tag as u16)
            .unwrap_or(usize::MAX)
    });
    let fine = client
        .player_state
        .as_ref()
        .unwrap()
        .root_actor(1)
        .unwrap()
        .path
        .position;
    let camera = render3d::Camera::angles([fine[0], -200, fine[1] - 600], 768, 0, 256.)?;
    let palette = floor::palette(0.8)?;
    let textures = Default::default();
    let mut expected = render3d::Frame::new(320, 240)?;
    world_actors::register_root_order(&mut expected, &camera, &client, &scene, &ordered);
    expected.draw(&camera, &ordered, &palette, &textures);
    let mut actual = render3d::Frame::new(320, 240)?;
    world_actors::register_root_order(&mut actual, &camera, &client, &scene, &triangles);
    actors.register_submission_order(&mut actual, &mut client, &scene, None)?;
    actual.draw(&camera, &triangles, &palette, &textures);
    assert_eq!(actual.pixels, expected.pixels);
    assert!(actual.tags.iter().any(|&tag| tag != 0));
    let mut entity = world_entity::Entity::default();
    entity.npcs = client.npc_state.clone();
    let mut worlds = world_entity::State::default();
    worlds.entities.insert(7, entity);
    client.world_entities.insert(0, worlds);
    let state = client.player_state.as_mut().unwrap();
    let copies = state.root_iter().map(|(i, p)| (i, p.clone())).collect();
    state.views.insert(
        7,
        player_info::View {
            base: [0, 0],
            dimensions: [104, 104],
            players: copies,
        },
    );
    assert_eq!(
        world_actors::submission_plan(&mut client, &scene, Some(7), None)?,
        vec![p(1), n(1), p(2), n(2)]
    );
    // Static occupancy is copied for each render pass, then actors reserve it in Java order.
    let mut definition = object::Definition::decode(999999, &[0], false, false)?;
    definition.models = vec![1];
    definition.model_shapes = Some(vec![10]);
    let definitions = std::collections::HashMap::from([(definition.id, definition)]);
    for count in [0, 4, 5] {
        let mut crowded = Scene::empty(0, 0)?;
        for _ in 0..count {
            crowded.objects.push(scene::PlacedObject {
                initial_index: None,
                rejected: false,
                runtime: false,
                id: 999999,
                plane: 0,
                x: 50,
                y: 50,
                shape: 10,
                rotation: 0,
            });
        }
        crowded.add_object_collision(&definitions)?;
        let expected = match count {
            0 => vec![p(1), n(1), p(2), n(2)],
            4 => vec![p(1)],
            _ => vec![],
        };
        for view in [None, Some(7)] {
            for _ in 0..3 {
                assert_eq!(
                    world_actors::submission_plan(&mut client, &crowded, view, None)?,
                    expected
                );
            }
        }
        let (mesh, pending) = actors.triangles(&mut client, &crowded, &Default::default())?;
        assert!(pending.is_empty());
        if count == 5 {
            assert!(mesh.is_empty());
        }
        if count == 4 {
            assert!(mesh.iter().all(|t| t.tag == 0x2000000000000001));
        }
        assert_eq!(crowded.temporary_slots().object_count(0, 50, 50), count);
    }
    // More than five off-center actors compete for the same tile without center deduplication.
    let state = client.player_state.as_mut().unwrap();
    let template = state.root_actor(1).unwrap().clone();
    for index in 3..10 {
        state.players[index] = Some(template.clone());
        state.high.push(index as u16);
    }
    assert_eq!(
        world_actors::submission_plan(&mut client, &scene, None, None)?,
        vec![p(1), n(1), p(2), p(3), p(4)]
    );
    // A rejected stretched actor still stamps its centered tile before capacity preflight.
    let mut edge = Scene::empty(0, 0)?;
    for _ in 0..5 {
        edge.objects.push(scene::PlacedObject {
            initial_index: None,
            rejected: false,
            runtime: false,
            id: 999999,
            plane: 0,
            x: 50,
            y: 49,
            shape: 10,
            rotation: 0,
        });
    }
    edge.add_object_collision(&definitions)?;
    for (_, player) in client.player_state.as_mut().unwrap().root_iter_mut() {
        player.path.position = [50 * 128 + 64; 2];
    }
    for npc in client.npc_state.npcs.values_mut() {
        npc.path.position = [50 * 128 + 64; 2];
    }
    let local = client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap();
    local.sequence.stretch = true;
    local.turn.angle = 0;
    assert!(world_actors::submission_plan(&mut client, &edge, None, None)?.is_empty());
    client.preferences.render_self = false;
    assert_eq!(
        world_actors::submission_plan(&mut client, &edge, None, None)?,
        vec![n(1)]
    );
    println!("Actor tile/capacity host: root and child actor interleaving, static limits, mesh rejection and fresh per-frame occupancy passed");
    Ok(())
}
