use gielinor_client::{
    disk_cache::DiskCache, raster::Raster, title::Assets, title_input::State, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let mut assets = Assets::load(&mut cache, 0)?;
    let mut raster = Raster::new(765, 503, 1 << 20)?;
    let state = State::default();
    assets.draw(&mut raster, &state, 1)?;
    let baseline = raster.pixels().to_vec();
    for cycle in 1..=300 {
        assets.draw_fire(&mut raster, cycle);
    }
    assets.draw(&mut raster, &state, 300)?;
    assets.draw_fire(&mut raster, 300);
    let changed = baseline
        .iter()
        .zip(raster.pixels())
        .filter(|(a, b)| a != b)
        .count();
    assert!(changed > 1000, "title fire not visible: {changed}");
    std::fs::write("work/title-fire-cache.bmp", raster.bmp()?)?;
    println!("Actual-cache title fire: {changed} changed pixels after 300 ticks");
    Ok(())
}
