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

    client.npc_state.indices.clear();
    client.npc_state.npcs.clear();
    let shape = viewport_shape::Shape {
        offset: [0; 2],
        size: [512, 334],
        focal: 1000,
    };
    client.camera.tracked = [50 * 128 + 64, 50 * 128 + 64 + 512];
    client.camera.tracked_height = 0;
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
        client.steering.request(0)?;
        actors.prepare_steering(&mut client, shape, Some(0));
        assert!(client.steering.scene[1].is_none());
        assert_eq!(
            client.steering.scene[0].unwrap().position,
            [50 * 128 + 64, 0, 50 * 128 + 64]
        );
        for _ in 0..3 {
            let plan = world_actors::submission_plan(&mut client, &crowded, None, None)?;
            assert_eq!(plan.len(), usize::from(count == 0));
            let (triangles, pending) =
                actors.triangles(&mut client, &crowded, &Default::default())?;
            assert!(pending.is_empty());
            let arrows = triangles
                .iter()
                .filter(|t| t.model_order.is_some_and(|o| o.source_ordinal.is_some()))
                .collect::<Vec<_>>();
            assert_eq!(arrows.is_empty(), count == 5);
            assert!(arrows.iter().all(|t| t.tag == 0 && t.view == 0));
            assert_eq!(crowded.temporary_slots().object_count(0, 50, 50), count);
        }
        let saved_players=client.player_state.take();
        let mut entity=world_entity::Entity::default();entity.pose.x=6464;entity.pose.z=6464;entity.owner_type=0;
        let mut children=world_entity::State::default();children.visible.push(7);children.entities.insert(7,entity);
        client.world_entities.insert(0,children);
        assert_eq!(actors.root_child_submissions(&mut client,&crowded)?.contains(&7),count==0);
        client.world_entities.clear();client.player_state=saved_players;
        for _ in 0..30 {
            client.steering.tick();
        }
        actors.prepare_steering(&mut client, shape, None);
        let plan = world_actors::submission_plan(&mut client, &crowded, None, None)?;
        assert_eq!(plan.len(), usize::from(count < 5));
    }
    client.steering.request(0)?;
    actors.prepare_steering(&mut client, shape, Some(8));
    let (triangles, _) = actors.triangles(&mut client, &scene, &Default::default())?;
    for ordinal in [0, 1] {
        assert!(triangles.iter().any(|t| t
            .model_order
            .is_some_and(|o| o.source_ordinal == Some(ordinal))));
    }
    println!("Steering scene models: active/preview meshes, duplicate suppression, static capacity before players, fresh frame slots and 30-tick expiration passed");
    Ok(())
}
