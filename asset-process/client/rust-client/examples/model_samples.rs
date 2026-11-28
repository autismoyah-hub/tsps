use gielinor_client::{disk_cache::DiskCache, model::Model};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let mut cache = DiskCache::open(&args[1], 64 << 20)?;
    std::fs::create_dir_all(&args[2])?;
    let ids = cache
        .reference(7)?
        .groups
        .iter()
        .map(|g| g.id)
        .collect::<Vec<_>>();
    let mut kinds = std::collections::BTreeMap::new();
    let mut total = 0;
    for id in ids {
        let data = cache.file(7, id, 0, [0; 4])?;
        let m = Model::decode(&data)?;
        let key = format!(
            "{:?}-{}{}{}{}{}{}{}",
            m.format,
            m.vertex_skins.is_some() as u8,
            m.face_skins.is_some() as u8,
            m.bone_weights.is_some() as u8,
            m.face_textures.is_some() as u8,
            m.priorities.is_some() as u8,
            m.alpha.is_some() as u8,
            m.face_flags.is_some() as u8
        );
        let n = kinds.entry(key.clone()).or_insert(0);
        if *n < 2 {
            std::fs::write(
                std::path::Path::new(&args[2]).join(format!("{id}.bin")),
                data,
            )?;
            *n += 1;
            total += 1;
            println!("{id}: {key}");
        }
    }
    println!("Exported {total} samples");
    Ok(())
}
