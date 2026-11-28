#[path = "../tests/support/object_scene_order.rs"]
mod support;
use gielinor_client::{
    buffer::Reader,
    disk_cache::DiskCache,
    rebuild::{Chunk, WorldEntity},
    scene::Scene,
    world_mesh::WorldMesh,
};

fn poses() -> Vec<([i32; 3], [i32; 2], u8)> {
    (0..64)
        .map(|index| {
            let x = [4, 28, 52, 76, 99][index % 5];
            let z = [4, 28, 52, 76, 99][index / 5 % 5];
            (
                [x * 128, -1000, z * 128],
                [[1024, 2048, 3064, 4160][index % 4], index as i32 * 256],
                (index / 4 % 4) as u8,
            )
        })
        .collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut chunks = vec![None; 676];
    for plane in 0..4 {
        for (x, z) in [
            (0, 0),
            (0, 12),
            (12, 0),
            (12, 12),
            (5, 5),
            (5, 6),
            (5, 7),
            (6, 5),
            (6, 6),
            (6, 7),
            (7, 5),
            (7, 6),
            (7, 7),
        ] {
            chunks[(plane * 13 + x) * 13 + z] = Some(Chunk {
                plane: plane as u8,
                zone_x: 400,
                zone_y: 400,
                rotation: 0,
            });
        }
    }
    let scene = Scene::load_world_entity(
        &mut cache,
        &WorldEntity {
            base: [0, 0],
            dimensions: [104, 104],
            chunks,
            regions: vec![12850],
        },
    )?;
    let mesh = WorldMesh::build(&scene, &mut cache, 3)?;
    let objects = &mesh.object_submissions;
    assert!(!objects.is_empty());
    let poses = poses();
    let bytes = std::fs::read("tests/fixtures/cached-object-scene-order-jvm.bin")?;
    let mut reader = Reader::new(&bytes);
    assert_eq!(reader.u32()? as usize, poses.len());
    let mut checked = 0;
    let mut ordered_hits = 0;
    let model_camera = gielinor_client::render3d::Camera::angles([0; 3], 0, 0, 200.)?;
    let local = [[-40., -40., 0.], [0., 40., 0.], [40., -40., 0.]];
    let bounds = gielinor_client::model_bounds::Bounds::new(&local, [0, 0, 300])
        .with_selection(&local, 0, true);
    let ray = gielinor_client::viewport_steering::Ray {
        far: 3500,
        size: [200; 2],
        focal: 200,
        angles: [0; 2],
        eye: [0; 3],
        focus: [0; 3],
    };
    let mut cache = gielinor_client::terrain_submission::Cache::default();
    for (case, &(eye, angles, maximum)) in poses.iter().enumerate() {
        let visibility = cache.prepare(
            [765, 503],
            [256, 320],
            25,
            [
                ((angles[0].clamp(1, 4160) - 1) / 256) as usize,
                (angles[1] / 1024) as usize,
            ],
        )?;
        let view = gielinor_client::terrain_submission::View {
            root: true,
            size: [104, 104],
            camera: [eye[0] >> 7, eye[2] >> 7],
            eye_y: eye[1],
            radius: 25,
            maximum_plane: maximum as usize,
        };
        assert_eq!(reader.u32()? as usize, objects.len());
        let mut frame = gielinor_client::render3d::Frame::new(200, 200)?;
        let mut faces = Vec::new();
        for (index, submitted) in objects.iter().enumerate() {
            let expected = [reader.u8()? != 0, reader.u8()? != 0];
            let actual = gielinor_client::object_submission::scene_object(
                &view,
                visibility,
                &scene,
                &submitted.object,
                submitted.size,
            );
            let o = &submitted.object;
            assert_eq!(
                actual, expected,
                "cached object case{case} id{} plane{} tile{},{} size{:?}",
                o.id, o.plane, o.x, o.y, submitted.size
            );
            let tag = index as u64 + 1;
            let bridge = scene.tile(1, o.x as usize, o.y as usize).unwrap().flags & 2 != 0;
            let key = support::key(
                o.plane as usize,
                [o.x as usize, o.y as usize],
                bridge,
                o.runtime,
            );
            frame.model_admission(0, tag, index, [actual[0], false]);
            support::register(&mut frame, 0, tag, index, key);
            let mut face = gielinor_client::render3d::Triangle::selection_only(bounds, tag);
            face.model_order = Some(gielinor_client::model_order::Submission {
                actor_spot: None,
                source_ordinal: None, temporary_id: None,
                scene_height_offset: 0,
                mode: gielinor_client::model_order::Mode::Auto,
                priority: 0,
                has_priorities: false,
                has_alpha: false,
                batch: index,
            });
            faces.push(face);
            checked += 1;
        }
        let expected = (0..reader.u32()?)
            .map(|_| reader.u32().map(|id| u64::from(id) + 1))
            .collect::<gielinor_client::Result<Vec<_>>>()?;
        frame.draw(&model_camera, &faces, &[], &Default::default());
        let actual = frame
            .model_hits([100, 100], ray)
            .into_iter()
            .map(|(tag, _)| tag)
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "cached first-pass model order case{case}");
        ordered_hits += expected.len();
    }
    assert_eq!(reader.remaining(), 0);
    assert_eq!(ordered_hits, 921);
    println!("{checked} actual-cache object admission pairs and {ordered_hits} first-pass model selections match original scene traversal across64 poses");
    Ok(())
}
