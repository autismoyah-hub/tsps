use gielinor_client::{
    disk_cache::DiskCache,
    font::{Font, Style},
    raster::Raster,
};
use std::io::Write;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    let mut c = DiskCache::open(&a[1], 64 << 20)?;
    let ids = c
        .reference(13)?
        .groups
        .iter()
        .map(|g| g.id)
        .collect::<Vec<_>>();
    let mut out = std::fs::File::create(&a[2])?;
    out.write_all(&(ids.len() as u32).to_be_bytes())?;
    let mut raster = Raster::new(1000, 900, 1 << 20)?;
    raster.clear(0xff28251f);
    for (i, id) in ids.iter().enumerate() {
        let data = c.file(13, *id, 0, [0; 4])?;
        let sprites = c.file(8, *id, 0, [0; 4])?;
        let font = Font::decode(&data, &sprites)?;
        out.write_all(&id.to_be_bytes())?;
        for bytes in [&data, &sprites] {
            out.write_all(&(bytes.len() as u32).to_be_bytes())?;
            out.write_all(bytes)?;
        }
        font.draw(&mut raster,&format!("Font {id}: <col=ffcc66>Welcome to Gielinor</col> — <u>Attack</u> <str>Goblin</str> (level-2)"),20,35+i as i32*40,Style::default(),&[])?;
    }
    std::fs::write(&a[3], raster.bmp()?)?;
    println!("Decoded and rendered {} native cache fonts", ids.len());
    Ok(())
}
