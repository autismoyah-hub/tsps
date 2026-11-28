use gielinor_client::{
    buffer::Reader,
    disk_cache::DiskCache,
    rebuild::{Chunk, WorldEntity},
    scene::Scene,
    world_mesh::WorldMesh,
};
use std::io::Write;

fn integer(out: &mut impl Write, value: i32) -> std::io::Result<()> {
    out.write_all(&value.to_be_bytes())
}
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
    if std::env::args().any(|arg| arg == "export") {
        let mut out = std::fs::File::create("work/w11-object-cache-input.bin")?;
        integer(&mut out, 104)?;
        for plane in 0..4 {
            for x in 0..=104 {
                for z in 0..=104 {
                    integer(&mut out, scene.height(plane, x, z).unwrap())?;
                }
            }
        }
        for plane in 0..4 {
            for x in 0..104 {
                for z in 0..104 {
                    out.write_all(&[scene.tile(plane, x, z).unwrap().flags])?;
                }
            }
        }
        integer(&mut out, objects.len() as i32)?;
        for submitted in objects {
            let o = &submitted.object;
            assert!(!o.runtime);
            for value in [
                o.plane as i32,
                o.x as i32,
                o.y as i32,
                submitted.size[0] as i32,
                submitted.size[1] as i32,
            ] {
                integer(&mut out, value)?;
            }
        }
        integer(&mut out, poses.len() as i32)?;
        for &(eye, angles, maximum) in &poses {
            for value in [eye[0], eye[1], eye[2], angles[0], angles[1], maximum as i32] {
                integer(&mut out, value)?;
            }
        }
        println!("Exported {} actual-cache scene objects and {} camera poses for the original JVM traversal", objects.len(), poses.len());
        return Ok(());
    }
    let bytes = std::fs::read("work/w11-object-cache-jvm.bin")?;
    let mut reader = Reader::new(&bytes);
    assert_eq!(reader.u32()? as usize, poses.len());
    let mut checked = 0;
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
        for submitted in objects {
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
            checked += 1;
        }
    }
    assert_eq!(reader.remaining(), 0);
    println!("{checked} actual-cache object pass admissions match original scene traversal across 64 camera poses");
    Ok(())
}
