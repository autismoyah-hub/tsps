//! Cache-backed live actor render integration: morph hiding, custom bodies and player copies.
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
    for (index, priority) in [(8, 2), (2, 0), (5, 1), (1, 0), (9, 1)] {
        let mut definition = client.npcs[&16299].clone();
        definition.id = 60000 + index as u32;
        definition.priority = priority;
        definition.transforms = None;
        let mut npc = template.clone();
        npc.id = definition.id as u16;
        npc.path.position[0] += index as i32 * 150;
        client.npcs.insert(definition.id, definition);
        client.npc_state.npcs.insert(index, npc);
    }
    client.npc_state.indices = vec![8, 2, 5, 1, 10];
    let check = |triangles: Vec<gielinor_client::render3d::Triangle>| {
        let mut ids = Vec::new();
        for triangle in triangles {
            let index = triangle.tag as u16;
            if !ids.contains(&index) {
                ids.push(index);
            }
        }
        assert_eq!(ids, vec![5, 2, 1, 8]);
    };
    let (triangles, pending) = actors.npc_triangles(&mut client, &scene, None)?;
    assert!(pending.is_empty());
    check(triangles);
    let mut entity = gielinor_client::world_entity::Entity::default();
    entity.npcs = client.npc_state.clone();
    let mut world = gielinor_client::world_entity::State::default();
    world.entities.insert(7, entity);
    client.world_entities.insert(0, world);
    let (triangles, pending) = actors.npc_triangles(&mut client, &scene, Some(7))?;
    assert!(pending.is_empty());
    check(triangles);
    println!("Actual-cache NPC body submissions: root and child views preserve priority/index order; missing and unlisted actors omitted");
    Ok(())
}
