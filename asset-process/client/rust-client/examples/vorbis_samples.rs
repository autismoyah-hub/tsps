use gielinor_client::{audio_vorbis, disk_cache::DiskCache, writer::Writer};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    let ids = cache
        .reference(4)?
        .groups
        .iter()
        .filter(|g| g.files.iter().any(|f| f.id == 1))
        .map(|g| g.id)
        .collect::<Vec<_>>();
    println!("{} compressed sound effects", ids.len());
    let mut records = Vec::new();
    let mut samples = 0;
    for id in ids {
        let data = cache.file(4, id, 1, [0; 4])?;
        let sound = audio_vorbis::decode_embedded(&data).map_err(|e| format!("sound {id}: {e}"))?;
        samples += sound.samples.len();
        if records.len() < 32 {
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
    println!("Decoded {samples} PCM samples");
    Ok(())
}
