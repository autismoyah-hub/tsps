//! Actual-cache model retention after original mixed-bridge removal semantics.
use gielinor_client::{
    disk_cache::DiskCache,
    location_update::Object,
    object::Definition,
    object_model::Models,
    rebuild::Chunk,
    scene::{PlacedObject, Scene},
    terrain::{Region, Tile},
    world_mesh::WorldMesh,
};

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
            d.support_items == 1
                && d.animation.is_none()
                && d.transforms.is_none()
                && d.size == [2, 1]
                && !d.model_ids(10).is_empty()
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|d| d.id);
    let mut models = Models::default();
    let mut selected = None;
    for definition in candidates {
        if models
            .object(&mut cache, definition, 10, 0)?
            .is_some_and(|model| {
                !model.faces.is_empty() && model.vertices.iter().any(|v| v[1] < 0.)
            })
        {
            selected = Some(definition.id);
            break;
        }
    }
    let id = selected.ok_or("no static 2x1 item-support model")?;
    let mut region = Region {
        tiles: vec![Tile::default(); 4 * 4096],
        extension_flags: 0,
        height_rules: vec![Some(0); 4 * 4096],
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
    scene.objects.push(PlacedObject {
        initial_index: None,
        rejected: false,
        runtime: false,
        id,
        plane: 1,
        x: 4,
        y: 4,
        shape: 10,
        rotation: 0,
    });
    scene.add_object_collision(&definitions)?;
    let old_identity = scene.object_traversal_id(0).unwrap();
    scene.replace_location(
        [0, 5, 4, 2],
        &Object {
            id: id as i32,
            shape: 10,
            rotation: 0,
        },
        &definitions,
    )?;
    let before = WorldMesh::build(&scene, &mut cache, 3)?;
    let old_tag = ((id as u64 + 1) << 32) | (1 << 28) | (4 << 14) | 4;
    let original = before
        .triangles
        .iter()
        .filter(|t| t.tag == old_tag)
        .map(|t| (t.vertices, t.colors, t.alpha))
        .collect::<Vec<_>>();
    assert!(!original.is_empty());
    scene.replace_location(
        [0, 4, 4, 2],
        &Object {
            id: -1,
            shape: 10,
            rotation: 0,
        },
        &definitions,
    )?;
    let after = WorldMesh::build(&scene, &mut cache, 3)?;
    let retained = after
        .triangles
        .iter()
        .filter(|t| t.tag == old_tag)
        .map(|t| (t.vertices, t.colors, t.alpha))
        .collect::<Vec<_>>();
    assert_eq!(
        retained, original,
        "surviving logical tile lost its original model"
    );
    assert!(after
        .object_submissions
        .iter()
        .any(|s| s.traversal_id == Some(old_identity)));
    assert_eq!(scene.location([0, 4, 4, 2]).id, -1);
    assert_eq!(scene.objects.len(), 1);
    let height = before.item_support[&[0, 4, 4]];
    assert!(height > 0);
    assert_eq!(before.item_support.get(&[1, 5, 4]), Some(&height));
    assert_eq!(after.item_support.get(&[1, 5, 4]), Some(&height));
    assert!(!after.item_support.contains_key(&[0, 4, 4]));
    assert_eq!(
        after.item_support.get(&[0, 5, 4]),
        before.item_support.get(&[0, 5, 4])
    );
    println!("Mixed-bridge cached model {id}: {} retained triangles, surviving item-support height {height}, live lookup remains removed", retained.len());
    Ok(())
}
