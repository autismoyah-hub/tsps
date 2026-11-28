//! Cache-backed bridge placement regression; no networking or cache mutation.
use gielinor_client::{
    disk_cache::DiskCache,
    object::Definition,
    object_model::Models,
    rebuild::Chunk,
    scene::{PlacedObject, Scene},
    terrain::{Region, Tile},
    world_mesh::WorldMesh,
};

fn scene(bridge: bool) -> gielinor_client::Result<Scene> {
    terrain_scene([0, if bridge { 2 } else { 0 }, 0, 0], false)
}

fn terrain_scene(flags: [u8; 4], floors: bool) -> gielinor_client::Result<Scene> {
    let mut region = Region {
        tiles: vec![Tile::default(); 4 * 4096],
        extension_flags: 0,
        height_rules: vec![Some(30); 4 * 4096],
    };
    for (plane, &flags) in flags.iter().enumerate() {
        for tile in &mut region.tiles[plane * 4096..(plane + 1) * 4096] {
            tile.flags = flags;
            tile.underlay = u16::from(floors);
        }
    }
    let mut scene = Scene::empty_sized(0, 0, 16, 16)?;
    for plane in 0..4 {
        for x in [0, 8] {
            for y in [0, 8] {
                scene.add_chunk(
                    &region,
                    Chunk {
                        plane: plane as u8,
                        rotation: 0,
                        zone_x: 0,
                        zone_y: 0,
                    },
                    plane,
                    x,
                    y,
                )?;
            }
        }
    }
    Ok(scene)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let defs = Definition::load_all(&mut cache)?;
    let mut models = Models::default();
    for shape in [0, 4, 22] {
        let mut candidates = defs
            .values()
            .filter(|d| {
                d.animation.is_none() && d.transforms.is_none() && !d.model_ids(shape).is_empty()
            })
            .collect::<Vec<_>>();
        candidates.sort_by_key(|d| d.id);
        let mut id = None;
        for d in candidates {
            if models
                .object(&mut cache, d, shape, 0)?
                .is_some_and(|m| !m.faces.is_empty())
            {
                id = Some(d.id);
                break;
            }
        }
        let id = id.ok_or("no cache singleton model")?;
        for bridge in [false, true] {
            let mut overwritten = scene(bridge)?;
            let mut expected = scene(bridge)?;
            for rotation in [0, 1] {
                overwritten.objects.push(PlacedObject {
                    initial_index: None, rejected: false,
                    runtime: false,
                    id,
                    plane: u8::from(bridge),
                    x: 4,
                    y: 4,
                    shape,
                    rotation,
                });
            }
            expected.objects.push(PlacedObject {
                initial_index: None, rejected: false,
                runtime: false,
                id,
                plane: u8::from(bridge),
                x: 4,
                y: 4,
                shape,
                rotation: 1,
            });
            overwritten.add_object_collision(&defs)?;
            expected.add_object_collision(&defs)?;
            let category = gielinor_client::location_update::category(shape)? as i32;
            assert_eq!(overwritten.location([0, 4, 4, category]).rotation, 1);
            let a = WorldMesh::build(&overwritten, &mut cache, 0)?;
            let b = WorldMesh::build(&expected, &mut cache, 0)?;
            assert!(!a.triangles.is_empty());
            assert_eq!(a.triangles.len(), b.triangles.len());
            for (a, b) in a.triangles.iter().zip(&b.triangles) {
                assert_eq!(a.tag, b.tag);
                assert_eq!(a.vertices, b.vertices);
                assert_eq!(a.colors, b.colors);
            }
            assert_eq!(a.item_support, b.item_support);
            println!(
                "Singleton shape{shape} cache{id} bridge{bridge}: latest model only,{} triangles",
                a.triangles.len()
            );
        }
    }
    Ok(())
}
