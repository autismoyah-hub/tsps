//! Explicit offline visual/scenery replay, not server-authoritative gameplay.
#[cfg(not(feature = "custom-assets"))]
fn main() {
    eprintln!("build with --features custom-assets");
    std::process::exit(1);
}
#[cfg(feature = "custom-assets")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use gielinor_client::{
        asset_package::Package,
        authored_floor::{Floor, ORIGIN},
        disk_cache::DiskCache,
        environment_style::{visual_only, Bindings},
        object::Definition,
        rebuild::{Instance, WorldEntity},
        scene::Scene,
        world_mesh::WorldMesh,
    };
    fn pin(s: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
        if s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("expected SHA256".into());
        }
        let mut p = [0; 32];
        for (i, v) in p.iter_mut().enumerate() {
            *v = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)?;
        }
        Ok(p)
    }
    let a: Vec<_> = std::env::args().skip(1).collect();
    if a.len() != 8 {
        return Err(
            "expected PACKAGE PACKAGE_SHA FLOOR FLOOR_SHA BINDINGS BINDINGS_SHA CACHE JS5".into(),
        );
    }
    let package = Package::read(std::path::Path::new(&a[0]))?;
    let bindings = Bindings::read(std::path::Path::new(&a[4]), pin(&a[5])?, &package)?;
    let count = bindings.len();
    let visual = package
        .object_ids()
        .filter(|id| {
            visual_only(
                &Definition::decode_local(*id, package.object(*id).unwrap()).unwrap(),
                &package,
            )
        })
        .collect::<std::collections::HashSet<_>>();
    let floor = Floor::read(std::path::Path::new(&a[2]), pin(&a[3])?)?;
    let placements = floor.placements().to_vec();
    let mut cache = DiskCache::open_remote(&a[6], 64 << 20, a[7].parse()?)?;
    cache.wait_for_recovery(true);
    let crc = cache.login_crcs()?;
    cache.mount_package(package, pin(&a[1])?)?;
    cache.mount_floor(
        floor
            .with_bindings(bindings)?
            .with_motion(gielinor_client::environment_motion::Mode::All)?,
        pin(&a[3])?,
    )?;
    let mut all = std::collections::HashSet::new();
    let mut triangles = 0;
    for dx in [0, 24] {
        for dz in [0, 24] {
            let mut scene = Scene::load_instance(
                &mut cache,
                &Instance {
                    zone_x: ((ORIGIN[0] + dx) / 8 + 6) as u16,
                    zone_y: ((ORIGIN[1] + dz) / 8 + 6) as u16,
                    reload: false,
                    chunks: vec![None; 676],
                    regions: vec![],
                },
            )?;
            let expected = placements
                .iter()
                .filter(|p| {
                    visual.contains(&p.id)
                        && (p.tile[0] as i32) >= dx
                        && (p.tile[0] as i32) < dx + 104
                        && (p.tile[1] as i32) >= dz
                        && (p.tile[1] as i32) < dz + 104
                })
                .count();
            assert_eq!(scene.objects.len(), expected);
            for o in &scene.objects {
                assert!(visual.contains(&o.id));
                all.insert((o.id, o.x as i32 + dx, o.y as i32 + dz, o.shape, o.rotation));
            }
            {
                // Replay the declared machinery through ordinary location updates;
                // the live server remains responsible for placing these objects.
                let defs = Definition::load_all(&mut cache)?;
                for p in placements
                    .iter()
                    .filter(|p| (64950..=64956).contains(&p.id))
                {
                    let x = ORIGIN[0] + p.tile[0] as i32 - scene.base_x;
                    let z = ORIGIN[1] + p.tile[1] as i32 - scene.base_y;
                    if x >= 0 && z >= 0 && x < 104 && z < 104 {
                        scene.replace_location(
                            [0, scene.base_x+x, scene.base_y+z, 2],
                            &gielinor_client::location_update::Object {
                                id: p.id as i32,
                                shape: p.shape,
                                rotation: p.rotation,
                            },
                            &defs,
                        )?;
                    }
                }
                let mut mesh = WorldMesh::build(&scene, &mut cache, 3)?;
                assert!(!mesh.pending_assets);
                assert!(mesh.environment_motion_faces() > 0);
                let bind = mesh.triangles.clone();
                let centre=bind.iter().find(|t|((t.tag>>32).wrapping_sub(1)as u32)==64950).unwrap().model_bounds.unwrap().position;
                let camera=gielinor_client::render3d::Camera::look_at([centre[0]as f32+600.,-900.,centre[2]as f32-900.],[centre[0]as f32,-300.,centre[2]as f32],320.)?;
                let palette=gielinor_client::floor::palette(0.8)?;let mut frames=Vec::new();
                for seconds in [0., 1., 3.25, 120.5] {
                    mesh.sample_environment_motion(seconds)?;
                    let mut moved = 0;
                    let mut groups = std::collections::HashMap::new();
                    for (posed, original) in mesh.triangles.iter().zip(&bind) {
                        let id = (posed.tag >> 32).wrapping_sub(1) as u32;
                        if (64950..=64956).contains(&id) {
                            if posed.vertices != original.vertices {
                                moved += 1;
                            }
                            let bounds = posed.model_bounds.unwrap();
                            assert_eq!(
                                bounds.selection.is_some(),
                                original.model_bounds.unwrap().selection.is_some()
                            );
                            assert_eq!(bounds.angle, 0);
                            let order = posed.model_order.unwrap();
                            assert_eq!(order.batch, original.model_order.unwrap().batch);
                            if let Some(previous) = groups.insert(order.batch, bounds) {
                                assert_eq!(previous, bounds);
                            }
                            let r = bounds.radius as f64;
                            let h = bounds.height as f64;
                            let below = bounds.below as f64;
                            assert!(
                                (r * r + h * h).sqrt().ceil()
                                    + (r * r + below * below).sqrt().ceil()
                                    < 6000.
                            );
                            assert!(posed.vertices.iter().flatten().all(|v| v.is_finite()));
                        } else {
                            assert_eq!(posed.vertices, original.vertices);
                            assert_eq!(posed.model_bounds, original.model_bounds);
                            assert_eq!(posed.colors, original.colors);
                        }
                    }
                    if seconds > 0. {
                        assert!(moved > 0);
                    }
                    let motion=mesh.triangles.iter().filter(|t|(64950..=64956).contains(&((t.tag>>32).wrapping_sub(1)as u32))).copied().collect::<Vec<_>>();
                    let mut frame=gielinor_client::render3d::Frame::new(256,256)?;frame.clear(0);frame.draw(&camera,&motion,&palette,&std::collections::HashMap::new());
                    assert!(frame.pixels.iter().any(|&p|p!=0),"native motion replay must draw pixels");frames.push(frame.pixels);

                }
                assert_ne!(frames[0],frames[1],"native pixels must change across poses");
                mesh.sample_environment_motion(1.)?;
                let first = mesh.triangles.clone();
                mesh.sample_environment_motion(900.)?;
                mesh.sample_environment_motion(1.)?;
                for (a, b) in first.iter().zip(&mesh.triangles) {
                    assert_eq!(a.vertices, b.vertices);
                    assert_eq!(a.model_bounds, b.model_bounds);
                }

                for t in &mesh.triangles {
                    let id = (t.tag >> 32).wrapping_sub(1) as u32;
                    if visual.contains(&id) {
                        triangles += 1;
                        assert!(t.model_bounds.is_some_and(|b| b.selection.is_none()));
                    }
                }
            }
        }
    }
    assert_eq!(
        all.len(),
        placements.iter().filter(|p| visual.contains(&p.id)).count()
    );
    assert!(triangles > 0);
    let child = Scene::load_world_entity(
        &mut cache,
        &WorldEntity {
            base: ORIGIN,
            dimensions: [104, 104],
            chunks: vec![None; 676],
            regions: vec![],
        },
    )?;
    assert!(child.objects.is_empty());
    assert_eq!(
        WorldMesh::build(&child, &mut cache, 3)?.environment_motion_faces(),
        0
    );
    assert_eq!(crc, cache.login_crcs()?);
    println!("Verified {count} admitted bindings, {} visual placements across four root rebuilds, {triangles} non-selectable visual triangles; child unchanged; original CRCs unchanged. Declared machinery replayed through synthetic ordinary location updates. Authored motion sampled at four fixed times over four root rebuilds; common native draw bounds below the diameter limit, no drift or unrelated geometry changes. Native isolated machinery images are nonempty and change between poses. Offline replay only.",all.len());
    Ok(())
}
