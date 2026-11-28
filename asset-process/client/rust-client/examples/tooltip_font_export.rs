//! Export the actual p12_full cache font for original-JVM tooltip raster fixtures.
use gielinor_client::{disk_cache::{name_hash, DiskCache}, Result};
use std::io::Write;
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let id = cache.reference(13)?.groups.iter().find(|g| g.name_hash == Some(name_hash("p12_full"))).unwrap().id;
    let mut out = std::fs::File::create("work/tooltip-font.bin").unwrap();
    out.write_all(&id.to_be_bytes()).unwrap();
    for archive in [13, 8] {
        let bytes = cache.file(archive, id, 0, [0;4])?;
        out.write_all(&(bytes.len() as u32).to_be_bytes()).unwrap();
        out.write_all(&bytes).unwrap();
    }
    println!("p12_full font {id} exported for tooltip oracle");
    Ok(())
}
