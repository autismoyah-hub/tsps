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
    assert_eq!(mesh.ground_submissions.len(), expected);
    let ids = mesh
        .ground_submissions
        .iter()
        .map(|s| s.identity)
        .collect::<HashSet<_>>();
    assert_eq!(ids.len(), expected);
    for (index, _) in scene
        .objects
        .iter()
        .enumerate()
        .filter(|(_, o)| o.shape == 22)
    {
        let id = scene.ground_identity(index).unwrap();
        let part = mesh
            .ground_submissions
            .iter()
            .find(|s| s.identity == id)
            .unwrap();
        assert_eq!(
            mesh.triangles[part.batch].model_order.unwrap().batch,
            part.batch
        );
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
            if let Call::Ground(id) = call {
                Some(*id)
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();
    assert_eq!(
        first, ids,
        "first pass invokes ground decorations, including the saved bridge"
    );
    assert!(calls.iter().flatten().all(|call| match call {
        Call::Ground(id) => ids.contains(id),
        _ => true,
    }));
    assert_eq!(
        calls[1]
            .iter()
            .filter(|call| matches!(call, Call::Ground(_)))
            .count(),
        expected - 1,
        "only the active surface draws ground decorations in the later pass"
    );
    println!(
        "Cache ground registry: {} parts, {} triangles; stable identities survive bridge retention",
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
                && !d.model_ids(22).is_empty()
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|d| d.id);
    let mut models = Models::default();
    let mut chosen = None;
    for d in candidates {
        let mut valid = true;
        for rotation in 0..4 {
            valid &= models
                .object(&mut cache, d, 22, rotation)?
                .is_some_and(|m| !m.faces.is_empty());
        }
        if valid {
            chosen = Some(d.id);
            break;
        }
    }
    let id = chosen.ok_or("no static ground decoration model")?;
    let mut region = Region {
        tiles: vec![Tile::default(); 4 * 4096],
        extension_flags: 0,
        height_rules: vec![Some(30); 4 * 4096],
    };
    region.tiles[4096 + 4 * 64 + 4].flags = 2;
    region.tiles[4 * 64 + 4].underlay = 1;
    region.tiles[4096 + 4 * 64 + 4].underlay = 1;
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
            shape: 22,
            rotation: 0,
        });
    }
    scene.add_object_collision(&definitions)?;
    let saved = [scene.ground_identity(0).unwrap()];
    let initial = inspect(&mut scene, &mut cache, 2)?;
    scene.replace_location(
        [0, 4, 4, 3],
        &Object {
            id: id as i32,
            shape: 22,
            rotation: 1,
        },
        &definitions,
    )?;
    let replaced = inspect(&mut scene, &mut cache, 2)?;
    assert_eq!(
        initial
            .intersection(&replaced)
            .copied()
            .collect::<HashSet<_>>(),
        saved.into_iter().collect()
    );
    scene.replace_location(
        [0, 4, 4, 3],
        &Object {
            id: -1,
            shape: 22,
            rotation: 0,
        },
        &definitions,
    )?;
    assert_eq!(
        inspect(&mut scene, &mut cache, 1)?,
        saved.into_iter().collect()
    );
    println!("Three actual-cache ground populations preserve identities, bridge retention, surface gating and replacement");
    Ok(())
}
