use gielinor_client::{disk_cache::DiskCache, music_track::Track, Result};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let mut records = Vec::new();
    for archive in [6, 11] {
        let groups = cache
            .reference(archive)?
            .groups
            .iter()
            .map(|g| g.id)
            .collect::<Vec<_>>();
        for id in groups.into_iter().take(64) {
            let packed = cache.file(archive, id, 0, [0; 4])?;
            let track = Track::decode(&packed)?;
            records.push((packed, track.midi));
        }
    }
    let mut out = Vec::new();
    out.extend_from_slice(&(records.len() as u32).to_be_bytes());
    for (packed, midi) in records {
        for bytes in [packed, midi] {
            out.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
            out.extend(bytes);
        }
    }
    std::fs::write("work/music-track-input.bin", out)?;
    println!("Decoded 128 packed cache music/jingle tracks for JVM comparison");
    Ok(())
}
