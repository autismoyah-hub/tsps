//! Actual-cache static shared lighting survives runtime removal of its neighbor.
use gielinor_client::{
    disk_cache::DiskCache,
    object::Definition,
    rebuild::Normal,
    scene::{PlacedObject, Scene},
    world_mesh::WorldMesh,
};
fn tag(o: &PlacedObject) -> u64 {
    ((o.id as u64 + 1) << 32) | ((o.plane as u64) << 28) | ((o.x as u64) << 14) | o.y as u64
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open(std::env::args().nth(1).expect("cache path"), 64 << 20)?;
    let mut scene = Scene::load_normal(
        &mut cache,
        &Normal {
            world_area: -1,
            zone_x: 400,
            zone_y: 400,
            players: None,
        },
    )?;
    let definitions = Definition::load_all(&mut cache)?;
    let objects: Vec<_> = scene
        .objects
        .iter()
        .filter(|o| {
            let d = &definitions[&o.id];
            !o.rejected
                && (o.shape <= 3 || o.shape >= 9)
                && d.non_flat_shading
                && d.animation.is_none()
                && d.transforms.is_none()
                && o.x > 0
                && o.y > 0
                && o.x < 103
                && o.y < 103
        })
        .cloned()
        .collect();
    assert!(!objects.is_empty());
    let before = WorldMesh::build(&scene, &mut cache, 3)?;
    let target = &objects[0];
    let removed_tag = tag(target);
    let tags: std::collections::HashSet<_> = objects.iter().map(tag).collect();
    let affected = before
        .triangles
        .iter()
        .filter(|t| tags.contains(&t.tag))
        .count();
    assert!(affected > 0);
    let plane = scene.object_scene_plane(target).unwrap();
    scene.replace_location(
        [
            plane as i32,
            scene.base_x + target.x as i32,
            scene.base_y + target.y as i32,
            gielinor_client::location_update::category(target.shape)? as i32,
        ],
        &gielinor_client::location_update::Object {
            id: -1,
            shape: target.shape,
            rotation: target.rotation,
        },
        &definitions,
    )?;
    let after = WorldMesh::build(&scene, &mut cache, 3)?;
    let signature = |mesh: &WorldMesh| {
        mesh.triangles
            .iter()
            .filter(|t| t.tag != removed_tag)
            .map(|t| (t.tag, t.vertices, t.colors))
            .collect::<Vec<_>>()
    };
    assert_eq!(signature(&before), signature(&after));
    println!("{} static shared-light objects, {affected} faces; runtime removal preserves remaining geometry and lighting", objects.len());
    Ok(())
}
