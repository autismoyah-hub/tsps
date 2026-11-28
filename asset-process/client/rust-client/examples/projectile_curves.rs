use gielinor_client::{
    disk_cache::{name_hash, DiskCache},
    script::Script,
    Result,
};
fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("cache path");
    let mut cache = DiskCache::open(path, 64 << 20)?;
    let names = cache
        .reference(12)?
        .groups
        .iter()
        .filter_map(|g| g.name_hash.map(|h| (h, g.id)))
        .collect::<std::collections::HashMap<_, _>>();
    std::fs::create_dir_all("work/projectile-curves")?;
    for curve in -2i32..=65535 {
        let name = (curve.wrapping_shl(8).wrapping_add(86)).to_string();
        if let Some(&id) = names.get(&name_hash(&name)) {
            let bytes = cache.file(12, id, 0, [0; 4])?;
            let script = Script::decode(&bytes, 1_000_000)?;
            std::fs::write(format!("work/projectile-curves/{curve}-{id}.bin"), &bytes)?;
            println!(
                "Curve {curve}: group {id}, {:?}, arguments {}, locals {}, instructions {:?}",
                script.name, script.int_arguments, script.local_ints, script.instructions
            );
        }
    }
    Ok(())
}
