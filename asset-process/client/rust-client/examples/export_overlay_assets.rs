use gielinor_client::{
    actor_overlay::Definitions,
    disk_cache::{name_hash, DiskCache},
    Result,
};
use std::{collections::BTreeSet, io::Write};
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let mut cache = DiskCache::open(&args[1], 64 << 20)?;
    let defs = Definitions::load(&mut cache)?;
    let mut sprites = BTreeSet::new();
    let mut fonts = BTreeSet::new();
    let fallback = cache
        .reference(13)?
        .groups
        .iter()
        .find(|g| g.name_hash == Some(name_hash("p11_full")))
        .unwrap()
        .id;
    fonts.insert(fallback);
    for d in defs.hits.values() {
        for id in d.sprites {
            if id >= 0 {
                sprites.insert(id as u32);
            }
        }
        if d.font >= 0 {
            fonts.insert(d.font as u32);
        }
    }
    for d in defs.bars.values() {
        for id in d.sprites {
            if id >= 0 {
                sprites.insert(id as u32);
            }
        }
    }
    let mut out = std::fs::File::create(&args[2])?;
    out.write_all(&fallback.to_be_bytes())?;
    out.write_all(&(fonts.len() as u32).to_be_bytes())?;
    for id in fonts {
        out.write_all(&id.to_be_bytes())?;
        for archive in [13, 8] {
            let data = cache.file(archive, id, 0, [0; 4])?;
            out.write_all(&(data.len() as u32).to_be_bytes())?;
            out.write_all(&data)?;
        }
    }
    out.write_all(&(sprites.len() as u32).to_be_bytes())?;
    for id in sprites {
        let data = cache.file(8, id, 0, [0; 4])?;
        out.write_all(&id.to_be_bytes())?;
        out.write_all(&(data.len() as u32).to_be_bytes())?;
        out.write_all(&data)?;
    }
    println!("Exported overlay fonts and sprites");
    Ok(())
}
