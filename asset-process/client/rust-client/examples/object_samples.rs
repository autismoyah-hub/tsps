use gielinor_client::{disk_cache::DiskCache, object::Definition};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    let mut cache = DiskCache::open(&a[1], 64 << 20)?;
    std::fs::create_dir_all(&a[2])?;
    let mut features = std::collections::HashSet::new();
    let mut total = 0;
    for (id, data) in cache.group(2, 6, [0; 4])? {
        if Definition::decode_local(id, &data)
            .is_ok_and(|d| d.description.is_some() || d.content_group.is_some())
        {
            continue;
        }
        if let Ok(d) = Definition::decode(id, &data, false, true) {
            let key = format!(
                "{}{}{}{}{}{}{}{}{}{}{}{}",
                d.model_shapes.is_some() as u8,
                d.solid as u8,
                d.transforms.is_some() as u8,
                d.contour,
                d.sound.ambient.is_some() as u8,
                d.sound.effects.is_empty() as u8,
                d.sound.fade_in.is_some() as u8,
                d.operations.iter().any(|o| o.condition.is_some()) as u8,
                d.operations.iter().any(|o| o.sub.is_some()) as u8,
                d.retexture.is_empty() as u8,
                d.scale != [128; 3],
                d.offset != [0; 3]
            );
            if features.insert(key) || id % 256 == 0 {
                std::fs::write(std::path::Path::new(&a[2]).join(format!("{id}.bin")), data)?;
                total += 1;
            }
        }
    }
    println!("Exported {total} stock-compatible definitions");
    Ok(())
}
