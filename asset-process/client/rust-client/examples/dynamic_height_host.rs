use gielinor_client::{
    animation_runtime::Animations,
    disk_cache::DiskCache,
    location_animation,
    object::Definition,
    object_model::Models,
    scene::{PlacedObject, Scene},
    world_mesh::WorldMesh,
    Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let defs = Definition::load_all(&mut cache)?;
    let mut candidates = defs
        .values()
        .filter(|d| {
            d.animation.is_none()
                && d.transforms.is_none()
                && d.size.iter().all(|&v| v <= 2)
                && !d.model_ids(10).is_empty()
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|d| d.id);
    let mut models = Models::default();
    let mut chosen = None;
    for def in candidates {
        if models
            .object(&mut cache, def, 10, 0)?
            .is_some_and(|m| !m.faces.is_empty())
        {
            chosen = Some(def.id);
            break;
        }
    }
    let id = chosen.expect("static model");
    let mut animations = Animations::load(&mut cache)?;
    assert_eq!(animations.sequences[&13].extra, 8);
    {
        let s = animations.sequences.get_mut(&13).unwrap();
        s.lengths.fill(1000);
        s.loop_frames = -1;
        s.max_loops = 1;
    }
    for shape in [10, 11] {
        for rotation in 0..4 {
            let mut scene = Scene::empty_sized(0, 0, 16, 16)?;
            scene.objects.push(PlacedObject {
                initial_index: None,
                rejected: false,
                runtime: false,
                id,
                plane: 0,
                x: 4,
                y: 4,
                shape,
                rotation,
            });
            scene.location_animations.insert(
                [0, 4, 4, 2],
                location_animation::parts(shape, rotation, Some(13), 1)?,
            );
            let mut baseline = WorldMesh::build(&scene, &mut cache, 0)?;
            assert!(!baseline.triangles.is_empty());
            animations.sequences.get_mut(&13).unwrap().extra = 0;
            baseline.advance(&scene, &mut cache, &mut animations, 0)?;
            for offset in [-128, 8, 127] {
                let mut mesh = WorldMesh::build(&scene, &mut cache, 0)?;
                let initial = mesh.triangles.clone();
                animations.sequences.get_mut(&13).unwrap().extra = offset;
                mesh.advance(&scene, &mut cache, &mut animations, 0)?;
                assert_eq!(mesh.triangles.len(), baseline.triangles.len());
                for (a, b) in mesh.triangles.iter().zip(&baseline.triangles) {
                    assert_eq!(a.tag, b.tag);
                    assert_eq!(a.local_vertices, b.local_vertices);
                    assert_eq!(
                        a.model_bounds.unwrap().position[1],
                        b.model_bounds.unwrap().position[1] - offset as i32,
                        "location height shape={shape} rotation={rotation} offset={offset}"
                    );
                    assert_eq!(a.model_order.unwrap().scene_height_offset, offset as i32);
                    for (v, w) in a.vertices.iter().zip(b.vertices) {
                        assert_eq!(*v, [w[0], w[1] - offset as f32, w[2]]);
                    }
                    for (v, w) in a
                        .texture_mapping
                        .unwrap()
                        .vertices
                        .iter()
                        .zip(b.texture_mapping.unwrap().vertices)
                    {
                        assert_eq!(*v, [w[0], w[1] - offset as f32, w[2]]);
                    }
                }
                mesh.advance(&scene, &mut cache, &mut animations, 100_000)?;
                for (a, b) in mesh.triangles.iter().zip(initial) {
                    assert_eq!(a.vertices, b.vertices, "completed sequence restores base");
                    assert_eq!(a.model_order.unwrap().scene_height_offset, 0);
                }
            }
            // GameDynamicObject.as retains the offset while frame assets are unavailable.
            let saved_frames = animations.sequences[&13].frames.clone();
            {
                let sequence = animations.sequences.get_mut(&13).unwrap();
                sequence.extra = 8;
                for frame in &mut sequence.frames {
                    *frame = 60000 << 16 | (*frame & 65535);
                }
            }
            let mut missing = WorldMesh::build(&scene, &mut cache, 0)?;
            let base = missing.triangles.clone();
            missing.advance(&scene, &mut cache, &mut animations, 0)?;
            for (a, b) in missing.triangles.iter().zip(base) {
                assert_eq!(
                    a.local_vertices, b.local_vertices,
                    "missing frame keeps base geometry"
                );
                assert_eq!(a.model_order.unwrap().scene_height_offset, 8);
                for (v, w) in a.vertices.iter().zip(b.vertices) {
                    assert_eq!(
                        *v,
                        [w[0], w[1] - 8., w[2]],
                        "missing frames preserve height offset"
                    );
                }
            }
            animations.sequences.get_mut(&13).unwrap().frames = saved_frames;
        }
    }
    println!("Dynamic location height:24 offset and8 missing-asset cache replays, signed offsets, four orientations, shapes10/11, texture coordinates, local bounds and completed-sequence restoration passed");
    Ok(())
}
