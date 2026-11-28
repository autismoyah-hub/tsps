//! Read-only authored terrain replay; this is not a live gameplay probe.
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
        rebuild::{Instance, WorldEntity},
        scene::Scene,
    };
    fn pin(s: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
        if s.len() != 64 || !s.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("expected SHA256 identity".into());
        }
        let mut p = [0; 32];
        for (i, v) in p.iter_mut().enumerate() {
            *v = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16)?;
        }
        Ok(p)
    }
    let a: Vec<_> = std::env::args().skip(1).collect();
    if a.len() != 6 && !(a.len() == 7 && a[6] == "--c020-contracts") {
        return Err("expected PACKAGE_DIRECTORY PACKAGE_SHA256 FLOOR_FILE FLOOR_SHA256 CACHE_DIRECTORY JS5_ADDRESS_OR_PACKAGE_ONLY [--c020-contracts]".into());
    }
    let contracts = a.len() == 7;
    if contracts && a[3].to_ascii_lowercase() != "f1ef5627db2e9ef152515647e61e9bdc5c3a97b18e8f03a38cba91c5d6572027" {
        return Err("C-020 coordinate replay requires the audited floor identity".into());
    }
    let expected = Floor::read(std::path::Path::new(&a[2]), pin(&a[3])?)?;
    let floor = Floor::read(std::path::Path::new(&a[2]), pin(&a[3])?)?;
    let (mut cache, crc) = if a[5] == "package-only" {
        (DiskCache::open_package_only(&a[0],64 << 20,pin(&a[1])?)?,None)
    } else {
        let package = Package::read(std::path::Path::new(&a[0]))?;
        let mut cache = DiskCache::open_remote(&a[4], 64 << 20, a[5].parse()?)?;
        cache.wait_for_recovery(true);
        let crc = cache.login_crcs()?;
        cache.mount_package(package, pin(&a[1])?)?;
        (cache,Some(crc))
    };
    cache.mount_floor(floor, pin(&a[3])?)?;
    let mut covered = vec![false; 128 * 128];
    for dx in [0, 24] {
        for dz in [0, 24] {
            let base = [ORIGIN[0] + dx, ORIGIN[1] + dz];
            let scene = if crc.is_none() {Scene::authored_floor_study_at(&mut cache,base)?} else {Scene::load_instance(
                &mut cache,
                &Instance {
                    zone_x: (base[0] / 8 + 6) as u16,
                    zone_y: (base[1] / 8 + 6) as u16,
                    reload: false,
                    chunks: vec![None; 676],
                    regions: vec![],
                },
            )?};
            for x in 0..104 {
                for z in 0..104 {
                    let fx = x + dx as usize;
                    let fz = z + dz as usize;
                    let v = expected.vertex(fx, fz).unwrap();
                    assert_eq!(scene.height(0, x, z), Some(-(v.height as i32)));
                    let tile = scene.tile(0, x, z).unwrap();
                    assert_eq!(tile.underlay, v.underlay);
                    assert_eq!(tile.flags, if v.walkable { 0 } else { 1 });
                    covered[fx * 128 + fz] = true;
                }
            }
            assert!(
                scene.objects.is_empty(),
                "offline terrain replay cannot create functional objects"
            );
            if contracts {
                // Preserved a31c18d architecture_terrain.rs assertions, expressed
                // in authored coordinates across four different root origins.
                let local = |x: usize, z: usize| (x - dx as usize, z - dz as usize);
                for (x, z) in [(47, 58), (55, 60), (69, 61)] {
                    let (x, z) = local(x, z);
                    assert_eq!(scene.tile(0, x, z).unwrap().underlay, 0);
                    assert_ne!(scene.collision[0].flag_at(x as i32, z as i32).unwrap() & 0x200000, 0);
                }
                for (x, z) in [(52, 60), (53, 61)] {
                    let (x, z) = local(x, z);
                    assert_ne!(scene.tile(0, x, z).unwrap().underlay, 0);
                    assert_eq!(scene.height(0, x, z), Some(-240));
                }
                for (x, z) in [(69, 58), (70, 60), (70, 62), (69, 65)] {
                    for (corner_x, corner_z) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let (x, z) = local(x + corner_x, z + corner_z);
                        assert_eq!(scene.height(0, x, z), Some(-112));
                    }
                }
                // The source integration's empty platform obligations are
                // independent of its runtime-object overlap/rebuild policy.
                let (x, z) = local(52, 46);
                assert_eq!(scene.collision[0].flag_at(x as i32, z as i32).unwrap() & 0x40200000, 0);
                // The source's bank assertion is the pinned floor obstruction,
                // not a server object's footprint or an interaction probe.
                let (x,z)=local(49,50);
                let bank=scene.collision[0].flag_at(x as i32,z as i32).unwrap();
                assert_ne!(bank & 0x200000,0);
                assert_eq!(bank & 0x40000000,0);
                for x in 50..55 { for z in 44..49 {
                    let (x, z) = local(x, z);
                    assert_eq!(scene.height(0, x, z), Some(-240));
                }}
            }
        }
    }
    assert!(covered.iter().all(|v| *v));
    if crc.is_some() {
    let child = Scene::load_world_entity(
        &mut cache,
        &WorldEntity {
            base: ORIGIN,
            dimensions: [104, 104],
            chunks: vec![None; 676],
            regions: vec![],
        },
    )?;
    assert_eq!(child.height(0, 2, 2), Some(0));
    assert_eq!(child.tile(0, 2, 2).unwrap().underlay, 0);
    println!("Child terrain unchanged.");
    }
    let definitions = gielinor_client::floor::Floors::load(&mut cache)?;
    for id in 5801..=5807 {
        assert!(definitions.underlays.contains_key(&id));
    }
    if let Some(crc)=crc {
        assert_eq!(crc, cache.login_crcs()?);
        println!("Original login CRCs unchanged.");
    } else {
        println!("Package-only terrain study: no original cache/JS5 reads; original login CRCs and child-world reconstruction were not checked.");
    }
    println!("Verified all 16384 authored tiles across four scene origins; seven materials decoded. Offline replay only.");
    if contracts {
        println!("C-020 pinned coordinate contracts passed in all four root origins: hidden recess collision, raised dry floor, four water-spawn corner sets, loaded dry platform, and loaded bank floor obstruction. Gameplay object overlap policy is not covered by this terrain-only replay.");
    }
    Ok(())
}
