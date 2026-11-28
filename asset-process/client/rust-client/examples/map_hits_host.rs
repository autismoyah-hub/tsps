use gielinor_client::*;
fn main() -> Result<()> {
    let mut cache = disk_cache::DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let areas = map_area::load_all(&mut cache)?;
    let area = &areas[&0];
    let center = [3200 - area.bounds[0] * 64, 3200 - area.bounds[1] * 64];
    let mut renderer = map_view::Renderer::load(&mut cache)?;
    let mut settings = map_settings::Settings::default();
    renderer.image(
        &mut cache,
        area,
        true,
        center,
        2.0,
        [256, 256],
        0.8,
        &settings,
        &[],
        true,
    )?;
    let mut points = Vec::new();
    for y in (0..256).step_by(4) {
        for x in (0..256).step_by(4) {
            let hits = renderer.hits(area, center, 2.0, [0, 0, 256, 256], [0, 0], [x, y])?;
            if !hits.is_empty() {
                for key in &hits {
                    assert!(renderer.event(*key).is_some());
                }
                points.push(([x, y], hits));
            }
        }
    }
    assert!(!points.is_empty());
    settings.hidden = true;
    renderer.image(
        &mut cache,
        area,
        true,
        center,
        2.0,
        [256, 256],
        0.8,
        &settings,
        &[],
        true,
    )?;
    for (point, hits) in &points {
        assert_eq!(
            &renderer.hits(area, center, 2.0, [0, 0, 256, 256], [0, 0], *point)?,
            hits
        );
    }
    renderer.reset_area();
    assert!(renderer
        .hits(area, center, 2.0, [0, 0, 256, 256], [0, 0], [128, 128])?
        .is_empty());
    println!("{} icon-hit screen samples, event coordinates, global-hide position retention and area reset passed",points.len());
    Ok(())
}
