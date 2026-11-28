//! Actual-cache terrain shadows remain baked across runtime location changes.
use gielinor_client::{
    disk_cache::DiskCache,
    object::Definition,
    rebuild::{Chunk, WorldEntity},
    scene::Scene,
    world_mesh::WorldMesh,
};
fn terrain(mesh: &WorldMesh) -> Vec<(u64, [i32; 3])> {
    mesh.triangles
        .iter()
        .filter(|t| t.tag & (1 << 63) != 0)
        .map(|t| (t.tag, t.colors))
        .collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open(std::env::args().nth(1).expect("cache path"), 64 << 20)?;
    let mut chunks = vec![None; 4 * 3 * 5];
    for x in 0..3 {
        for y in 0..5 {
            chunks[x * 5 + y] = Some(Chunk {
                plane: 0,
                zone_x: 400 + x as u16,
                zone_y: 400 + y as u16,
                rotation: 0,
            });
        }
    }
    let rebuild = WorldEntity {
        base: [6400, 6400],
        dimensions: [24, 40],
        chunks,
        regions: vec![12850],
    };
    let mut scene = Scene::load_world_entity(&mut cache, &rebuild)?;
    let defs = Definition::load_all(&mut cache)?;
    let before = terrain(&WorldMesh::build(&scene, &mut cache, 0)?);
    assert!(!before.is_empty());
    let placements = scene.objects.clone();
    let mut removed = 0;
    for object in placements.iter().filter(|o| !o.rejected) {
        let Some(plane) = scene.object_scene_plane(object) else {
            continue;
        };
        let category = gielinor_client::location_update::category(object.shape)?;
        scene.replace_location(
            [
                plane as i32,
                scene.base_x + i32::from(object.x),
                scene.base_y + i32::from(object.y),
                category as i32,
            ],
            &gielinor_client::location_update::Object {
                id: -1,
                shape: object.shape,
                rotation: object.rotation,
            },
            &defs,
        )?;
        removed += 1;
    }
    assert!(removed > 0);
    scene.add_object_collision(&defs)?;
    assert_eq!(before, terrain(&WorldMesh::build(&scene, &mut cache, 0)?));
    // Same loaded terrain in a fresh scene, deliberately omitting every caster.
    let mut bare = Scene::empty_sized(6400, 6400, 24, 40)?;
    let files = cache.group(5, 12850, [0; 4])?;
    let region = gielinor_client::terrain::Region::decode(
        &files.iter().find(|(id, _)| *id == 0).unwrap().1,
        3200,
        3200,
    )?;
    for x in 0..3 {
        for y in 0..5 {
            bare.add_chunk(&region, rebuild.chunks[x * 5 + y].unwrap(), 0, x * 8, y * 8)?;
        }
    }
    let unshadowed = terrain(&WorldMesh::build(&bare, &mut cache, 0)?);
    assert_eq!(before.len(), unshadowed.len());
    let changed = before
        .iter()
        .zip(&unshadowed)
        .filter(|(a, b)| a != b)
        .count();
    assert!(changed > 0);
    println!(
        "{changed} terrain faces shadowed; {removed} runtime removals preserve baked lighting"
    );
    Ok(())
}
