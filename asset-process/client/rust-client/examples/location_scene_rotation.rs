//! Actual-cache shape-11 orientation in static and animated scene geometry.
use gielinor_client::{
    animation_runtime::Animations,
    disk_cache::DiskCache,
    location_animation::{parts, scene_vertex},
    model_order,
    object::Definition,
    scene::{PlacedObject, Scene},
    world_mesh::WorldMesh,
};
fn check(a: &WorldMesh, b: &WorldMesh, centre: [f32; 2]) {
    assert!(!a.triangles.is_empty());
    for mesh in [a, b] {
        let first = mesh.triangles[0].model_order.expect("location order");
        for t in &mesh.triangles {
            let order = t.model_order.expect("location face order");
            assert_eq!(order.mode, model_order::Mode::Auto);
            assert_eq!(order.batch, first.batch);
            assert!(t.alpha == 0 || order.has_alpha);
        }
    }
    assert_eq!(a.triangles.len(), b.triangles.len());
    let mut changed = false;
    for (left, right) in a.triangles.iter().zip(&b.triangles) {
        assert_eq!(left.colors, right.colors);
        assert_eq!(left.texture, right.texture);
        for (v, w) in left.vertices.iter().zip(right.vertices) {
            let mut expected = scene_vertex(11, [v[0] - centre[0], v[1], v[2] - centre[1]]);
            expected[0] += centre[0];
            expected[2] += centre[1];
            // World-coordinate float rounding can cross the integer cast boundary
            // during animated deformation; the pure JVM vertex suite is exact.
            for axis in 0..3 {
                assert!(
                    (w[axis] - expected[axis]).abs() <= 1.01,
                    "{w:?} != {expected:?}"
                );
            }
            changed |= *v != w;
        }
    }
    assert!(changed);
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open(std::env::args().nth(1).expect("cache path"), 64 << 20)?;
    let defs = Definition::load_all(&mut cache)?;
    let mut animations = Animations::load(&mut cache)?;
    let mut ids: Vec<_> = defs.keys().copied().collect();
    ids.sort_unstable();
    for (animated, runtime) in [(false, false), (false, true), (true, false), (true, true)] {
        let id = *ids
            .iter()
            .find(|id| {
                let d = &defs[*id];
                d.transforms.is_none()
                    && d.animation.is_some() == animated
                    && d.size.iter().all(|s| *s <= 4)
                    && !d.model_ids(10).is_empty()
                    && d.animation.is_none_or(|seq| {
                        animations
                            .sequences
                            .get(&(seq as u32))
                            .is_some_and(|s| !s.frames.is_empty())
                    })
            })
            .expect("suitable object");
        for rotation in 0..4 {
            let mut scenes = Vec::new();
            let mut meshes = Vec::new();
            for shape in [10, 11] {
                let mut scene = Scene::empty_sized(0, 0, 16, 16)?;
                if runtime {
                    scene.add_object_collision(&defs)?;
                    scene.replace_location(
                        [0, 4, 4, 2],
                        &gielinor_client::location_update::Object {
                            id: id as i32,
                            shape,
                            rotation,
                        },
                        &defs,
                    )?;
                } else {
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
                    scene.add_object_collision(&defs)?;
                }
                if animated {
                    scene.location_animations.insert(
                        [0, 4, 4, 2],
                        parts(shape, rotation, defs[&id].animation.map(u32::from), 1)?,
                    );
                }
                meshes.push(WorldMesh::build(&scene, &mut cache, 0)?);
                scenes.push(scene);
            }
            let size = defs[&id].dimensions(rotation);
            let centre = [512.0 + size[0] as f32 * 64.0, 512.0 + size[1] as f32 * 64.0];
            check(&meshes[0], &meshes[1], centre);
            if animated {
                for ticks in [1, 3, 7] {
                    for i in 0..2 {
                        meshes[i].advance(&scenes[i], &mut cache, &mut animations, ticks)?;
                    }
                    check(&meshes[0], &meshes[1], centre);
                }
            }
        }
        println!("Object {id}: animated={animated}, runtime={runtime}, four rotations preserve extra scene orientation");
    }
    // Boundary and wall-decoration pairs share a pick tag but remain separate renderables.
    for (shape, model_shape) in [(2, 2), (8, 4)] {
        let id = *ids
            .iter()
            .find(|id| {
                let d = &defs[*id];
                d.animation.is_none()
                    && d.transforms.is_none()
                    && !d.model_ids(model_shape).is_empty()
            })
            .expect("two-part location definition");
        let mut scene = Scene::empty_sized(0, 0, 16, 16)?;
        scene.add_object_collision(&defs)?;
        scene.replace_location(
            [0, 4, 4, if shape == 2 { 0 } else { 1 }],
            &gielinor_client::location_update::Object {
                id: id as i32,
                shape,
                rotation: 0,
            },
            &defs,
        )?;
        let mesh = WorldMesh::build(&scene, &mut cache, 0)?;
        let batches = mesh
            .triangles
            .iter()
            .map(|t| t.model_order.unwrap().batch)
            .collect::<std::collections::HashSet<_>>();
        let tags = mesh
            .triangles
            .iter()
            .map(|t| t.tag)
            .collect::<std::collections::HashSet<_>>();
        assert_eq!(
            batches.len(),
            2,
            "shape {shape} must retain two renderables"
        );
        assert_eq!(tags.len(), 1, "the pair shares one interaction tag");
        println!("Object {id}: shape {shape} retains two batches under one pick tag");
    }
    Ok(())
}
