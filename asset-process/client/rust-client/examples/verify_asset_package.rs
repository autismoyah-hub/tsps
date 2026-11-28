//! Explicit local package admission probe. No login or cache writes.
#[cfg(not(feature = "custom-assets"))]
fn main() {
    eprintln!("build this probe with --features custom-assets");
    std::process::exit(1);
}

#[cfg(feature = "custom-assets")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use gielinor_client::{asset_package::Package, disk_cache::DiskCache};
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 && args.len() != 4 {
        return Err(
            "expected PACKAGE_DIRECTORY EXPECTED_SHA256 [CACHE_DIRECTORY JS5_ADDRESS]".into(),
        );
    }
    if args[1].len() != 64 || !args[1].bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("expected a 64-digit content identity from your content policy".into());
    }
    let mut expected = [0; 32];
    for (i, byte) in expected.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&args[1][i * 2..i * 2 + 2], 16)?;
    }
    let package = Package::read(std::path::Path::new(&args[0]))?;
    if package.identity() != expected {
        return Err("package content identity mismatch".into());
    }
    let models: Vec<_> = package.model_ids().collect();
    let objects: Vec<_> = package.object_ids().collect();
    if args.len() == 4 {
        let mut cache = DiskCache::open_remote(&args[2], 64 << 20, args[3].parse()?)?;
        cache.wait_for_recovery(true);
        let before = cache.login_crcs()?;
        cache.mount_package(package, expected)?;
        for &id in &models {
            gielinor_client::model::Model::decode(&cache.file(7, id, 0, [0; 4])?)?;
        }
        for &id in &objects {
            gielinor_client::object::Definition::decode_local(id, &cache.file(2, 6, id, [0; 4])?)?;
        }
        assert_eq!(before, cache.login_crcs()?);
        let mut reopened = cache.reopen()?;
        assert_eq!(
            cache.file(7, models[0], 0, [0; 4])?,
            reopened.file(7, models[0], 0, [0; 4])?
        );
        assert_eq!(before, reopened.login_crcs()?);
        println!(
            "Mounted and reopened; original login CRCs unchanged; JS5 {:?}",
            cache.recovery_stats()
        );
    }
    println!(
        "Verified {} models and {} objects against {}",
        models.len(),
        objects.len(),
        args[1]
    );
    Ok(())
}
