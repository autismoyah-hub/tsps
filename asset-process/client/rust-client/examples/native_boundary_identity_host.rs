use gielinor_client::{
    disk_cache::DiskCache,
    location_update::Object,
    object::Definition,
    object_model::Models,
    rebuild::Chunk,
    scene::{PlacedObject, Scene},
    scene_object_layout::Call,
    scene_visibility::Visibility,
    terrain::{Region, Tile},
    terrain_submission::View,
    world_mesh::WorldMesh,
};
use std::collections::HashSet;
fn inspect(
    scene: &mut Scene,
    cache: &mut DiskCache,
    expected: usize,
) -> Result<HashSet<usize>, Box<dyn std::error::Error>> {
    let mesh = WorldMesh::build(scene, cache, 3)?;
    assert_eq!(mesh.boundary_submissions.len(), expected);
    let ids = mesh
        .boundary_submissions
        .iter()
        .map(|s| s.identity)
        .collect::<HashSet<_>>();
    assert_eq!(ids.len(), expected);
    for (index, object) in scene
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.shape == 2)
    {
        let parts = (0..2)
            .map(|part| {
                let id = scene.boundary_part_identity(index, part).unwrap();
                mesh.boundary_submissions
                    .iter()
                    .find(|s| s.identity == id)
                    .unwrap()
            })
            .collect::<Vec<_>>();
        assert_ne!(parts[0].batch, parts[1].batch);
        let tags = parts
            .iter()
            .map(|s| mesh.triangles[s.batch].tag)
            .collect::<Vec<_>>();
        assert_eq!(
            tags[0], tags[1],
            "wall parts share owner tag for {}",
            object.id
        );
        for part in parts {
            assert_eq!(
                mesh.triangles[part.batch].model_order.unwrap().batch,
                part.batch
            );
        }
    }
    let view = View {
        root: false,
        size: [16; 2],
        camera: [4; 2],
        eye_y: -1000,
        radius: 25,
        maximum_plane: 3,
    };
    let mut visibility = Visibility::new([765, 503], [256, 320], 25, [0, 0])?;
    let calls = scene
        .draw_render_calls(&view, &mut visibility, [512; 2], &[])
        .unwrap();
    let first = calls[0]
        .iter()
        .filter_map(|call| {
            if let Call::Boundary(id) = call {
                Some(*id)
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();
    assert_eq!(
        first, ids,
        "first pass invokes both parts, including saved bridge walls"
    );
    assert!(calls.iter().flatten().all(|call| match call {
        Call::Boundary(id) => ids.contains(id),
        _ => true,
    }));
    println!(
        "Cache boundary registry: {} parts, {} triangles; distinct batches preserve shared tags",
        expected,
        mesh.triangles.len()
    );
    Ok(ids)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let definitions = Definition::load_all(&mut cache)?;
    let mut candidates = definitions
        .values()
        .filter(|d| {
            d.animation.is_none()
                && d.transforms.is_none()
                && d.size == [1, 1]
                && !d.model_ids(2).is_empty()
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|d| d.id);
    let mut models = Models::default();
    let mut chosen = None;
    for d in candidates {
        let mut valid = true;
        for rotation in [4, 1, 5, 2] {
            valid &= models
                .object(&mut cache, d, 2, rotation)?
                .is_some_and(|m| !m.faces.is_empty());
        }
        if valid {
            chosen = Some(d.id);
            break;
        }
    }
    let id = chosen.ok_or("no static two-part wall model")?;
    let mut region = Region {
        tiles: vec![Tile::default(); 4 * 4096],
        extension_flags: 0,
        height_rules: vec![Some(30); 4 * 4096],
    };
    region.tiles[4096 + 4 * 64 + 4].flags = 2;
    let mut scene = Scene::empty_sized(0, 0, 16, 16)?;
    for plane in 0..4 {
        scene.add_chunk(
            &region,
            Chunk {
                plane: plane as u8,
                rotation: 0,
                zone_x: 0,
                zone_y: 0,
            },
            plane,
            0,
            0,
        )?;
    }
    for plane in 0..2 {
        scene.objects.push(PlacedObject {
            initial_index: None,
            rejected: false,
            runtime: false,
            id,
            plane,
            x: 4,
            y: 4,
            shape: 2,
            rotation: 0,
        });
    }
    scene.add_object_collision(&definitions)?;
    let saved = [
        scene.boundary_part_identity(0, 0).unwrap(),
        scene.boundary_part_identity(0, 1).unwrap(),
    ];
    let initial = inspect(&mut scene, &mut cache, 4)?;
    scene.replace_location(
        [0, 4, 4, 0],
        &Object {
            id: id as i32,
            shape: 2,
            rotation: 1,
        },
        &definitions,
    )?;
    let replaced = inspect(&mut scene, &mut cache, 4)?;
    assert_eq!(
        initial
            .intersection(&replaced)
            .copied()
            .collect::<HashSet<_>>(),
        saved.into_iter().collect()
    );
    scene.replace_location(
        [0, 4, 4, 0],
        &Object {
            id: -1,
            shape: 2,
            rotation: 0,
        },
        &definitions,
    )?;
    assert_eq!(
        inspect(&mut scene, &mut cache, 2)?,
        saved.into_iter().collect()
    );
    println!("Three actual-cache wall populations preserve part identities, bridge retention and replacement");
    Ok(())
}
