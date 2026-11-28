use gielinor_client::{
    disk_cache::{name_hash, DiskCache},
    Result,
};
use std::io::Write;
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let mut cache = DiskCache::open(&args[1], 64 << 20)?;
    let ids = gielinor_client::minimap::sprite_groups(&cache.file(17, 3, 0, [0; 4])?)?;
    let font = cache
        .reference(13)?
        .groups
        .iter()
        .find(|g| g.name_hash == Some(name_hash("p12_full")))
        .unwrap()
        .id;
    let mut out = std::fs::File::create(&args[2])?;
    for archive in [13, 8] {
        let bytes = cache.file(archive, font, 0, [0; 4])?;
        out.write_all(&(bytes.len() as u32).to_be_bytes())?;
        out.write_all(&bytes)?;
    }
    for index in 3..=5 {
        let id = ids[index].unwrap();
        let bytes = cache.file(8, id, 0, [0; 4])?;
        out.write_all(&id.to_be_bytes())?;
        out.write_all(&(bytes.len() as u32).to_be_bytes())?;
        out.write_all(&bytes)?;
    }
    println!("Exported player name font and head-icon sheets");
    Ok(())
}
