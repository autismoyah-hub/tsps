use gielinor_client::{
    animation_runtime::Animations, disk_cache::DiskCache, rebuild::Normal, scene::Scene,
    world_mesh::WorldMesh,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("cache path required")?;
    let mut cache = DiskCache::open(path, 64 << 20)?;
    let mut scene = Scene::load_normal(
        &mut cache,
        &Normal {
            world_area: -1,
            zone_x: 400,
            zone_y: 400,
            players: None,
        },
    )?;
    let mut mesh = WorldMesh::build(&scene, &mut cache, 1)?;
    let mut animations = Animations::load(&mut cache)?;
    let before = mesh
        .triangles
        .iter()
        .map(|t| (t.vertices, t.alpha))
        .collect::<Vec<_>>();
    let mut changed = std::collections::HashSet::new();
    for tick in 0..120 {
        mesh.advance(&scene, &mut cache, &mut animations, 1)?;
        for (i, t) in mesh.triangles.iter().enumerate() {
            assert!(
                t.vertices.iter().flatten().all(|v| v.is_finite()),
                "nonfinite triangle at tick {tick}"
            );
            if (t.vertices, t.alpha) != before[i] {
                changed.insert(i);
            }
        }
    }
    assert!(mesh.animated_objects() > 0);
    assert!(!changed.is_empty(), "animation did not affect any triangle");
    let snapshot = mesh
        .triangles
        .iter()
        .map(|t| (t.vertices, t.alpha))
        .collect::<Vec<_>>();
    let mut rebuilt = WorldMesh::build(&scene, &mut cache, 1)?;
    rebuilt.retain_clocks(&mesh, &animations);
    rebuilt.advance(&scene, &mut cache, &mut animations, 0)?;
    assert_eq!(
        snapshot,
        rebuilt
            .triangles
            .iter()
            .map(|t| (t.vertices, t.alpha))
            .collect::<Vec<_>>(),
        "mesh rebuild reset animation clocks"
    );
    let definitions = gielinor_client::object::Definition::load_all(&mut cache)?;
    let candidate = scene
        .objects
        .iter()
        .find(|o| o.plane <= 1 && definitions[&o.id].animation.is_some())
        .ok_or("no animated location")?;
    let sequence = definitions[&candidate.id].animation.unwrap() as i32;
    let request = gielinor_client::location_animation::Request {
        at: 0,
        shape: candidate.shape,
        rotation: candidate.rotation,
        sequence,
    };
    let key = [
        candidate.plane as i32,
        scene.base_x + candidate.x as i32,
        scene.base_y + candidate.y as i32,
        gielinor_client::location_update::category(candidate.shape)? as i32,
    ];
    assert!(scene.animate_location(key, request, &definitions)?);
    let mut overridden = WorldMesh::build(&scene, &mut cache, 1)?;
    overridden.retain_clocks(&rebuilt, &animations);
    for _ in 0..24 {
        overridden.advance(&scene, &mut cache, &mut animations, 1)?;
    }
    assert!(overridden.triangles.iter().all(|t| t
        .vertices
        .iter()
        .flatten()
        .all(|v| v.is_finite())));
    println!("Location override {key:?}, sequence {sequence}; preserved unrelated animation clocks through mesh rebuild");
    println!("120 native animation ticks: {} animated objects, {} distinct changed triangles, {} scene triangles",mesh.animated_objects(),changed.len(),mesh.triangles.len());
    Ok(())
}
