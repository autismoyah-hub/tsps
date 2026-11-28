use gielinor_client::map_floor;
use gielinor_client::{
    disk_cache::DiskCache, image_decode, map_composite, map_geography, raster::Raster, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let composite = map_composite::Composite::decode(&cache.file(19, 1, 0, [0; 4])?, true)?;
    let floors = map_floor::Floors::load(&mut cache, 0.8)?;
    let mut raster = Raster::new(1024, 1024, 1024 * 1024)?;
    let mut count = 0;
    for x in 48..52 {
        for y in 48..52 {
            let destination = [x, y];
            if !composite.regions.contains_key(&destination)
                && !composite
                    .chunks
                    .iter()
                    .any(|c| c.region.destination == destination)
            {
                continue;
            }
            let group = ((x as u32) << 8) | y as u32;
            let geography = map_geography::Geography::decode(
                &cache.file(18, group, 0, [0; 4])?,
                &composite,
                destination,
            )?;
            let ground = image_decode::decode(&cache.file(20, group, 0, [0; 4])?, 1 << 20)?;
            let image = floors.render(&geography, &ground.pixels, 0xff000000, 4)?;
            image.draw_scaled(
                &mut raster,
                [((x - 48) * 256) as i32, ((51 - y) * 256) as i32, 256, 256],
                256,
            )?;
            count += 1;
        }
    }
    std::fs::write("work/map-floor-lumbridge.bmp", raster.bmp()?)?;
    println!(
        "Painted {count} actual map floor regions around Lumbridge; decorations and icons pending"
    );
    Ok(())
}
