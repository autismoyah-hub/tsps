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
    let models = client
        .scripts
        .source
        .cache
        .reference(7)?
        .groups
        .iter()
        .map(|g| g.id)
        .collect::<std::collections::HashSet<_>>();
    let id = client
        .spots
        .iter()
        .filter(|(_, d)| models.contains(&d.model))
        .map(|(&id, _)| id)
        .min()
        .expect("available spot") as u16;
    client.spots.get_mut(&(id as u32)).unwrap().sequence = -1;
    let npc = client.npc_state.npcs.get_mut(&1).unwrap();
    npc.spots.apply(0, id, 0, 0, 0);
    npc.idle_fallback.state.id = 0;
    npc.idle_fallback.state.loading = 1;
    let mut entity = gielinor_client::world_entity::Entity::default();
    entity.npcs = client.npc_state.clone();
    let mut world = gielinor_client::world_entity::State::default();
    world.entities.insert(7, entity);
    client.world_entities.insert(0, world);
    for view in [None, Some(7)] {
        let (mesh, pending) = actors.npc_triangles(&mut client, &scene, view)?;
        assert!(pending.is_empty(), "{pending:?}");
        assert!(
            !mesh.is_empty(),
            "available effect survives NPC body animation loading grace"
        );
        assert!(mesh
            .iter()
            .all(|t| t.model_order.unwrap().actor_spot == Some(0x4000000000000001)));
        assert!(!actors.heights.contains_key(&0x4000000000000001));
        assert!(actors.visible_npcs.is_empty(), "effect readiness does not mark the body visible");
    }
    client.scripts.variables.from_server(42, 1);
    for view in [None, Some(7)] {
        assert!(
            actors
                .npc_triangles(&mut client, &scene, view)?
                .0
                .is_empty(),
            "hidden morph suppresses actor and effects"
        );
    }
    println!("NPC attached effects survive body animation loading grace in root and child views; hidden morph remains suppressed");
    Ok(())
}
