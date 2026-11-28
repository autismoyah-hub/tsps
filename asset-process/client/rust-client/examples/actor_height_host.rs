use gielinor_client::*;
// Cache-backed animation height offsets for root/child actors, attachments and effects.
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

    client.player_state.as_mut().unwrap().high = vec![1];
    client.player_state.as_mut().unwrap().players[2] = None;
    client.npc_state.indices = vec![1];
    client.npc_state.npcs.remove(&2);
    let npc = client.npc_state.npcs.get_mut(&1).unwrap();
    npc.path.teleport([52, 50], 1);
    npc.position = [52, 50];
    let sequence = animation::Sequence::load_all(&mut client.scripts.source.cache)?
        .remove(&13)
        .unwrap();
    assert_eq!(sequence.extra, 8);
    let models = client
        .scripts
        .source
        .cache
        .reference(7)?
        .groups
        .iter()
        .map(|g| g.id)
        .collect::<std::collections::HashSet<_>>();
    let spot = client
        .spots
        .iter()
        .filter(|(_, d)| models.contains(&d.model))
        .map(|(&id, _)| id)
        .min()
        .unwrap() as u16;
    client.spots.get_mut(&(spot as u32)).unwrap().sequence = -1;
    let p = client
        .player_state
        .as_mut()
        .unwrap()
        .root_actor_mut(1)
        .unwrap();
    p.sequence.id = 13;
    p.sequence.frame = 0;
    p.spots.apply(0, spot, 0, 0, 0);
    let mut attachment_model = model::Model::combine(&[])?;
    attachment_model.vertices = vec![[-10., -10., 0.], [0., 10., 0.], [10., -10., 0.]];
    attachment_model.faces = vec![[0, 1, 2]];
    attachment_model.colors = vec![100];
    p.attachment = Some(location_merge::Attachment {
        placement: location_merge::Placement {
            start: 0,
            end: 100,
            centre: [p.path.position[0], 0, p.path.position[1]],
            bounds: [50, 50, 50, 50],
        },
        model: attachment_model,
        colors: vec![[100; 3]],
    });
    let n = client.npc_state.npcs.get_mut(&1).unwrap();
    n.sequence.id = 13;
    n.sequence.frame = 0;
    n.spots.apply(0, spot, 0, 0, 0);
    client.npcs.get_mut(&(n.id as u32)).unwrap().overlay_height = 200;
    client.world_spots.push(
        world_spot::Request {
            id: spot,
            height: 0,
            delay: 0,
            position: [0, 60, 60],
        }
        .start(0),
    );
    let mut entity = world_entity::Entity::default();
    entity.npcs = client.npc_state.clone();
    entity.world_spots = client.world_spots.clone();
    let mut worlds = world_entity::State::default();
    worlds.entities.insert(7, entity);
    client.world_entities.insert(0, worlds);
    let p = client.player_state.as_mut().unwrap();
    let copies = p.root_iter().map(|(i, p)| (i, p.clone())).collect();
    p.views.insert(
        7,
        player_info::View {
            base: [0, 0],
            dimensions: [104, 104],
            players: copies,
        },
    );
    let player_tag = 0x2000000000000001;
    let npc_tag = 0x4000000000000001;
    for world in [None, Some(7)] {
        let triangles = if world.is_none() {
            actors
                .triangles(&mut client, &scene, &Default::default())?
                .0
        } else {
            let mut out = actors.npc_triangles(&mut client, &scene, world)?.0;
            model_order::append(
                &mut out,
                actors.player_triangles(&mut client, &scene, world)?,
            );
            out
        };
        let heights = actors.overlay_heights(&client, world);
        assert_eq!(heights[&player_tag], actors.heights[&player_tag] + 8);
        assert_eq!(heights[&npc_tag], 208);
        for tag in [player_tag, npc_tag] {
            let owned = triangles
                .iter()
                .filter(|t| {
                    t.tag == tag || t.model_order.is_some_and(|o| o.actor_spot == Some(tag))
                })
                .collect::<Vec<_>>();
            assert!(!owned.is_empty());
            if tag == player_tag {
                assert!(
                    owned.iter().any(|t| t.colors == [100; 3]),
                    "attachment triangle must be present"
                );
            }
            assert!(owned
                .iter()
                .any(|t| t.model_order.unwrap().actor_spot == Some(tag)));
            for t in owned {
                assert_eq!(t.model_bounds.unwrap().position[1], -8);
                assert_eq!(t.model_order.unwrap().scene_height_offset, 8);
            }
        }
        let world_effects = triangles
            .iter()
            .filter(|t| t.tag == 0 && t.model_order.is_some_and(|o| o.actor_spot.is_none()))
            .collect::<Vec<_>>();
        assert!(!world_effects.is_empty());
        assert!(world_effects
            .iter()
            .all(|t| t.model_bounds.unwrap().position[1] == 0
                && t.model_order.unwrap().scene_height_offset == 0));
    }
    // Fully transparent bodies retain displaced bounds and submission identity.
    for world in [None, Some(7)] {
        let state = client.player_state.as_mut().unwrap();
        let player = if let Some(id) = world {
            state
                .views
                .get_mut(&id)
                .unwrap()
                .players
                .get_mut(&1)
                .unwrap()
        } else {
            state.root_actor_mut(1).unwrap()
        };
        player
            .visual
            .transparency
            .get_or_insert_with(Default::default)
            .set(0, 0, 0, 255, 255, true);
        let npc = if world.is_none() {
            client.npc_state.npcs.get_mut(&1).unwrap()
        } else {
            client
                .world_entities
                .get_mut(&0)
                .unwrap()
                .entities
                .get_mut(&7)
                .unwrap()
                .npcs
                .npcs
                .get_mut(&1)
                .unwrap()
        };
        npc.visual
            .transparency
            .get_or_insert_with(Default::default)
            .set(0, 0, 0, 255, 255, true);
        let mut triangles = actors.npc_triangles(&mut client, &scene, world)?.0;
        model_order::append(
            &mut triangles,
            actors.player_triangles(&mut client, &scene, world)?,
        );
        for tag in [player_tag, npc_tag] {
            let marker = triangles
                .iter()
                .find(|t| t.tag == tag && t.local_vertices.is_none())
                .expect("transparent body selection marker");
            assert_eq!(
                marker.model_bounds.unwrap().position[1],
                -8,
                "transparent body must retain animation height"
            );
            assert_eq!(marker.model_order.unwrap().scene_height_offset, 8);
        }
    }
    // The fallback's loading grace hides the body, but its effect retains the offset.
    for world in [None, Some(7)] {
        let n = if world.is_none() {
            client.npc_state.npcs.get_mut(&1).unwrap()
        } else {
            client
                .world_entities
                .get_mut(&0)
                .unwrap()
                .entities
                .get_mut(&7)
                .unwrap()
                .npcs
                .npcs
                .get_mut(&1)
                .unwrap()
        };
        n.sequence = Default::default();
        n.pose = Default::default();
        n.idle_fallback.state.id = 13;
        n.idle_fallback.state.loading = 1;
        let out = actors.npc_triangles(&mut client, &scene, world)?.0;
        let owned = out
            .iter()
            .filter(|t| t.model_order.is_some_and(|o| o.actor_spot == Some(npc_tag)))
            .collect::<Vec<_>>();
        assert!(!owned.is_empty());
        assert!(owned
            .iter()
            .all(|t| t.model_bounds.unwrap().position[1] == -8));
        assert!(!actors.heights.contains_key(&npc_tag));
    }
    println!("Actor heights including transparent body bounds: sequence13 offset8 in root/child bodies, attachments and effects, independent world effects, NPC overlay override200+8, player overlays, and loading-grace effects passed");
    Ok(())
}
