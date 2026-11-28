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
    cache.mount_floor(floor.with_bindings(bindings)?, pin(&a[3])?)?;
    let mut all = std::collections::HashSet::new();
    let mut triangles = 0;
    for dx in [0, 24] {
        for dz in [0, 24] {
            let scene = Scene::load_instance(
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
            if dx == 0 && dz == 0 {
                let mesh = WorldMesh::build(&scene, &mut cache, 3)?;
                assert!(!mesh.pending_assets);
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
    assert_eq!(crc, cache.login_crcs()?);
    println!("Verified {count} admitted bindings, {} visual placements across four root rebuilds, {triangles} non-selectable visual triangles; child unchanged; original CRCs unchanged. Offline replay only.",all.len());
    Ok(())
}
