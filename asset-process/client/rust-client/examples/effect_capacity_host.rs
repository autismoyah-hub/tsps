use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    object, projectile,
    scene::{PlacedObject, Scene},
    world_actors::Actors,
    world_entity, world_spot, Result,
};
fn main() -> Result<()> {
    let cache = std::env::args().nth(1).expect("cache path");
    let mut source=if let Some(endpoint)=std::env::args().nth(2) {
        DiskCache::open_remote(cache,64 << 20,endpoint.parse().map_err(|_| gielinor_client::Error::Invalid("JS5 endpoint"))?)?
    } else {DiskCache::open(cache,64 << 20)?};
    source.wait_for_recovery(true);
    let mut c=ClientState::load(source)?;
    c.data.position = [0, 50, 50];
    let mut actors = Actors::load(&mut c.scripts.source.cache)?;
    let models = c
        .scripts
        .source
        .cache
        .reference(7)?
        .groups
        .iter()
        .map(|g| g.id)
        .collect::<std::collections::HashSet<_>>();
    let id = c
        .spots
        .iter()
        .filter(|(_, d)| models.contains(&d.model))
        .map(|(&id, _)| id)
        .min()
        .unwrap() as u16;
    c.spots.get_mut(&(id as u32)).unwrap().sequence = -1;
    let request = world_spot::Request {
        id,
        height: 0,
        delay: 0,
        position: [0, 50, 50],
    };
    let mut scene = Scene::empty(0, 0)?;
    c.world_spots.push(request.clone().start(0));
    let one = actors.triangles(&mut c, &scene, &Default::default())?.0;
    assert!(!one.is_empty());
    let count = one.len();
    for _ in 0..5 {
        c.world_spots.push(request.clone().start(0));
    }
    assert_eq!(
        actors
            .triangles(&mut c, &scene, &Default::default())?
            .0
            .len(),
        5 * count
    );
    // Render against the last simulated cycle, not the next actor tick.
    let draw_cycle=c.actor_cycle.wrapping_sub(1);
    // Delayed entries reserve no capacity before their start cycle.
    for effect in &mut c.world_spots {effect.animation.start=draw_cycle+1;}
    assert!(actors.triangles(&mut c,&scene,&Default::default())?.0.is_empty());
    for effect in &mut c.world_spots {effect.animation.start=draw_cycle;}
    c.world_spots[0].animation.start = draw_cycle+1;
    assert_eq!(
        actors
            .triangles(&mut c, &scene, &Default::default())?
            .0
            .len(),
        5 * count
    );
    let mut p = projectile::Request {
        id,
        source: [0, 50, 50],
        target: [0, 51, 50],
        heights: [0; 2],
        entities: [0; 2],
        start: 0,
        end: 20,
        slope: 0,
        offset: 0,
    }
    .start(0);
    p.motion.moving = true;
    p.motion.position = [6465., 6464., 0.];
    c.projectiles.push(p);
    let mixed = actors.triangles(&mut c, &scene, &Default::default())?.0;
    assert_eq!(mixed.len(), 5 * count);
    assert_eq!(
        mixed[0].model_bounds.unwrap().position[0],
        6465,
        "projectile insertion precedes world effects"
    );
    assert_eq!(mixed[count].model_bounds.unwrap().position[0], 6464);
    // Four static objects leave room only for the first projectile.
    let mut definition = object::Definition::decode(999999, &[0], false, false)?;
    definition.models = vec![1];
    definition.model_shapes = Some(vec![10]);
    let definitions = std::collections::HashMap::from([(definition.id, definition)]);
    for _ in 0..4 {
        scene.objects.push(PlacedObject {
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
    scene.add_object_collision(&definitions)?;
    let limited = actors.triangles(&mut c, &scene, &Default::default())?.0;
    assert_eq!(limited.len(), count);
    assert!(limited
        .iter()
        .all(|t| t.model_bounds.unwrap().position[0] == 6465));
    assert_eq!(scene.temporary_slots().object_count(0, 50, 50), 4);
    let mut entity = world_entity::Entity::default();
    entity.world_spots = vec![request.start(0); 6];
    let mut worlds = world_entity::State::default();
    worlds.entities.insert(7, entity);
    c.world_entities.insert(0, worlds);
    assert_eq!(
        actors.npc_triangles(&mut c, &scene, Some(7))?.0.len(),
        count
    );
    println!("Temporary effects: five-object limits, delayed starts, projectile-before-world-effect insertion, static occupancy and child effects passed");
    Ok(())
}
