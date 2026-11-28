use gielinor_client::{audio_synth::Effect, disk_cache::DiskCache, writer::Writer};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    let groups = cache
        .reference(4)?
        .groups
        .iter()
        .filter(|g| g.files.iter().any(|f| f.id == 0))
        .map(|g| g.id)
        .collect::<Vec<_>>();
    let mut records = Vec::new();
    let mut decoded = 0;
    for &id in &groups {
        let data = cache.file(4, id, 0, [0; 4])?;
        Effect::decode(&data).map_err(|e| format!("sound {id}: {e}"))?;
        decoded += 1;
        if records.len() < 128 && ((id % 37 == 0) || id < 16) {
            records.push((id, data));
        }
    }
    let mut w = Writer::new();
    w.u32(records.len() as u32);
    for (id, data) in records {
        w.u32(id);
        w.u32(data.len() as u32);
        w.bytes(&data);
    }
    std::fs::write(&a[2], w.into_vec())?;
    println!("Decoded {decoded} synthesized sound effects");
    Ok(())
}
