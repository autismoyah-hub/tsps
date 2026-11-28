use gielinor_client::map_floor;
use gielinor_client::{
    disk_cache::DiskCache, image_decode, map_composite, map_geography, raster::Raster, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let composite = map_composite::Composite::decode(&cache.file(19, 1, 0, [0; 4])?, true)?;
    let floors = map_floor::Floors::load(&mut cache, 0.8)?;
    let definitions = gielinor_client::object::Definition::load_all(&mut cache)?;
    let sprites = gielinor_client::minimap::Assets::load(&mut cache)?.scenes;
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
            let mut image = floors.render(&geography, &ground.pixels, 0xff000000, 4)?;
            let mut region = Raster::from_pixels(256, 256, image.pixels)?;
            gielinor_client::map_decoration::region(
                &mut region,
                &geography,
                4,
                &definitions,
                &sprites,
            )?;
            image.pixels = region.pixels().to_vec();
            image.draw_scaled(
                &mut raster,
                [((x - 48) * 256) as i32, ((51 - y) * 256) as i32, 256, 256],
                256,
            )?;
            count += 1;
        }
    }
    std::fs::write("work/map-decorations-lumbridge.bmp", raster.bmp()?)?;
    println!(
        "Painted {count} actual map regions around Lumbridge with walls and map-scene sprites; icons pending"
    );
    Ok(())
}
