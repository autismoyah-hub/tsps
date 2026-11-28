//! Read-only offline geography and constructed marker checks, not live gameplay.
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
        environment_minimap as map,
        environment_style::Bindings,
        location_update::Object,
        object::Definition,
        rebuild::{Instance, WorldEntity},
        scene::Scene,
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
    if a.len() != 10 {
        return Err("expected PACKAGE PACKAGE_SHA FLOOR FLOOR_SHA BINDINGS BINDINGS_SHA CACHE JS5 MAP_FUNCTIONS MAP_SHA".into());
    }
    let package = Package::read(std::path::Path::new(&a[0]))?;
    let bindings = Bindings::read(std::path::Path::new(&a[4]), pin(&a[5])?, &package)?;
    let functions = map::MapFunctions::read(std::path::Path::new(&a[8]), pin(&a[9])?, &package)?;
    let entries = functions.bindings().collect::<Vec<_>>();
    let floor = Floor::read(std::path::Path::new(&a[2]), pin(&a[3])?)?
        .with_bindings(bindings)?
        .with_map_functions(functions)?;
    let mut cache = DiskCache::open_remote(&a[6], 64 << 20, a[7].parse()?)?;
    cache.wait_for_recovery(true);
    let crc = cache.login_crcs()?;
    cache.mount_package(package, pin(&a[1])?)?;
    cache.mount_floor(floor, pin(&a[3])?)?;
    let mut pixels = 0;
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
            let geography =
                map::scene_geography(&scene, &mut cache)?.ok_or("missing admitted geography")?;
            let count = geography.iter().filter(|&&v| v != 0).count();
            assert!(count > 0);
            pixels += count;
            if dx == 0 && dz == 0 {
                // These verification positions are synthetic; server gameplay positions are not inferred.
                let package = Package::read(std::path::Path::new(&a[0]))?;
                let bindings = Bindings::read(std::path::Path::new(&a[4]), pin(&a[5])?, &package)?;
                let original = Definition::load_all(&mut cache)?;
                let defs = bindings.for_render(&original)?;
                let mut positions = Vec::new();
                for (index, &(id, element)) in entries.iter().enumerate() {
                    let def = &defs[&id];
                    let shape = def
                        .model_shapes
                        .as_ref()
                        .and_then(|v| v.first())
                        .copied()
                        .unwrap_or(10);
                    let layer = match shape {
                        0..=3 => 0,
                        4..=8 => 1,
                        22 => 3,
                        _ => 2,
                    };
                    let position = [scene.base_x + 4 + index as i32 * 8, scene.base_y + 6];
                    scene.replace_location(
                        [0, position[0], position[1], layer],
                        &Object {
                            id: id as i32,
                            shape,
                            rotation: 0,
                        },
                        &original,
                    )?;
                    positions.push((position, element));
                }
                drop(defs);
                drop(package);
                let mut icons = Vec::new();
                map::append_map_icons(&scene, &mut cache, 0, &mut icons)?;
                assert_eq!(icons.len(), entries.len());
                let elements = gielinor_client::map_element::Definition::load_all(&mut cache)?;
                for (position, element) in &positions {
                    let sprite = gielinor_client::sprite_pixels::Pixels::load(
                        &mut cache,
                        elements[element].sprites[0].ok_or("missing original sprite")?,
                    )?;
                    let actual = &icons
                        .iter()
                        .find(|(p, _)| p == position)
                        .ok_or("missing semantic marker")?
                        .1;
                    assert_eq!(actual.pixels, sprite.pixels);
                    assert_eq!(actual.size, sprite.size);
                }
                map::append_map_icons(&scene, &mut cache, 0, &mut icons)?;
                assert_eq!(icons.len(), entries.len());
                let mut upper = Vec::new();
                map::append_map_icons(&scene, &mut cache, 1, &mut upper)?;
                assert!(upper.is_empty());
                assert_eq!(
                    geography,
                    map::scene_geography(&scene, &mut cache)?.unwrap()
                );
                let mut child = Scene::load_world_entity(
                    &mut cache,
                    &WorldEntity {
                        base: ORIGIN,
                        dimensions: [104, 104],
                        chunks: vec![None; 676],
                        regions: vec![],
                    },
                )?;
                child.objects = scene.objects.clone();
                assert!(map::scene_geography(&child, &mut cache)?.is_none());
                let mut child_icons = Vec::new();
                map::append_map_icons(&child, &mut cache, 0, &mut child_icons)?;
                assert!(child_icons.is_empty());
            }
        }
    }
    assert_eq!(crc, cache.login_crcs()?);
    println!("Verified {pixels} authored geography pixels across four root rebuilds; {} original semantic sprites at constructed test positions; no duplicates, upper-plane or child leakage; unchanged CRCs. Offline replay only.",entries.len());
    Ok(())
}
