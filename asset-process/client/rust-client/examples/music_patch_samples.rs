use gielinor_client::{disk_cache::DiskCache, music_patch::Patch, Result};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let groups = cache
        .reference(15)?
        .groups
        .iter()
        .map(|g| g.id)
        .collect::<Vec<_>>();
    let mut out = Vec::new();
    out.extend_from_slice(&(groups.len() as u32).to_be_bytes());
    for id in &groups {
        let bytes = cache.file(15, *id, 0, [0; 4])?;
        Patch::decode(&bytes)?;
        out.extend_from_slice(&id.to_be_bytes());
        out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        out.extend(bytes);
    }
    std::fs::write("work/music-patch-input.bin", out)?;
    println!("Decoded {} cache instrument patches", groups.len());
    Ok(())
}
