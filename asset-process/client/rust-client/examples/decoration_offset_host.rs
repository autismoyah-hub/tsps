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
use std::collections::BTreeMap;
fn original_offsets(shape: u8, rotation: u8) -> [[i32; 2]; 2] {
    let data = include_bytes!("../tests/fixtures/decoration-offsets-jvm.bin");
    for record in data[4..].chunks_exact(52) {
        let r: Vec<i32> = record
            .chunks_exact(4)
            .map(|b| i32::from_be_bytes(b.try_into().unwrap()))
            .collect();
        if r[..5] == [0, 0, 16, shape as i32, rotation as i32] {
            return [[r[7], r[8]], [r[9], r[10]]];
        }
    }
    panic!("missing original offset record");
}
fn inspect(scene: &Scene, cache: &mut DiskCache) -> Result<(), Box<dyn std::error::Error>> {
    let mesh = WorldMesh::build(scene, cache, 3)?;
    let mut total = 0;
    for (index, o) in scene.objects.iter().enumerate() {
        let expected = original_offsets(o.shape, o.rotation);
        assert_eq!(scene.decoration_part_offsets(index), Some(expected));
        let tag = ((o.id as u64 + 1) << 32)
            | ((o.plane as u64) << 28)
            | ((o.x as u64) << 14)
            | o.y as u64;
        let mut batches = BTreeMap::new();
        for t in mesh.triangles.iter().filter(|t| t.tag == tag) {
            batches.entry(t.model_order.unwrap().batch).or_insert(t);
        }
        assert_eq!(batches.len(), if o.shape == 8 { 2 } else { 1 });
        for (part, t) in batches.values().enumerate() {
            let local = t.local_vertices.unwrap();
            let base = scene
                .height(o.plane as usize, o.x as usize, o.y as usize)
                .unwrap();
            for vertex in 0..3 {
                let translation = std::array::from_fn::<_, 3, _>(|axis| {
                    t.vertices[vertex][axis] - local[vertex][axis]
                });
                assert_eq!(
                    translation,
                    [
                        (o.x as i32 * 128 + 64 + expected[part][0]) as f32,
                        base as f32,
                        (o.y as i32 * 128 + 64 + expected[part][1]) as f32
                    ],
                    "shape {} rotation {} part {}",
                    o.shape,
                    o.rotation,
                    part
                );
            }
            total += 1;
        }
    }
    assert_eq!(total, 24);
    println!("Actual-cache decoration offsets: 20 placements, 24 distinct model parts match persisted translations");
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let defs = Definition::load_all(&mut cache)?;
    let mut candidates = defs
        .values()
        .filter(|d| {
            d.animation.is_none()
                && d.transforms.is_none()
                && d.size.iter().all(|&n| (1..=4).contains(&n))
                && !d.model_ids(4).is_empty()
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|d| (d.size == [1, 1], d.id));
    let mut models = Models::default();
    let mut chosen = None;
    for d in candidates {
        let mut valid = true;
        for r in 0..8 {
            valid &= models
                .object(&mut cache, d, 4, r)?
                .is_some_and(|m| !m.faces.is_empty());
        }
        if valid {
            chosen = Some(d.id);
            break;
        }
    }
    let id = chosen.ok_or("no static decoration model")?;
    println!(
        "Actual-cache decoration definition {id}, size {:?}",
        defs[&id].size
    );
    let region = Region {
        tiles: vec![Tile::default(); 4 * 4096],
        extension_flags: 0,
        height_rules: vec![Some(30); 4 * 4096],
    };
    let mut scene = Scene::empty_sized(0, 0, 16, 16)?;
    for x in 0..2 {
        for z in 0..2 {
            scene.add_chunk(
                &region,
                Chunk {
                    plane: 0,
                    rotation: 0,
                    zone_x: 0,
                    zone_y: 0,
                },
                0,
                x * 8,
                z * 8,
            )?;
        }
    }
    for shape in 4..=8 {
        for rotation in 0..4 {
            scene.objects.push(PlacedObject {
                initial_index: None,
                rejected: false,
                runtime: false,
                id,
                plane: 0,
                x: 2 + rotation * 3,
                y: 2 + (shape - 4) * 2,
                shape,
                rotation,
            });
        }
    }
    scene.add_object_collision(&defs)?;
    inspect(&scene, &mut cache)?;
    for shape in 4..=8 {
        for rotation in 0..4 {
            scene.replace_location(
                [0, 2 + rotation as i32 * 3, 2 + (shape as i32 - 4) * 2, 1],
                &Object {
                    id: id as i32,
                    shape,
                    rotation: (rotation + 1) & 3,
                },
                &defs,
            )?;
        }
    }
    inspect(&scene, &mut cache)?;
    Ok(())
}
