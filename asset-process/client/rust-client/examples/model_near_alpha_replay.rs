use gielinor_client::{disk_cache::DiskCache, item_icon::Icons, model::Model, Result};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let icons = Icons::load(&mut cache, 0.8)?;
    let mut model = Model::decode(&[0; 18])?;
    model.vertices = vec![[-30., -30., 30.], [0., 30., 300.], [30., -30., 300.]];
    model.faces = vec![[0, 1, 2]];
    model.colors = vec![100];
    model.alpha = Some(vec![0]);
    let colors = [[100, 200, 300]];
    let original =
        icons.draw_models_alpha(&[(&model, &colors, &model.vertices)], [100, 100], None, 0)?;
    let actor =
        icons.draw_models_alpha(&[(&model, &colors, &model.vertices)], [100, 100], None, 128)?;
    assert!(original.pixels.iter().any(|&p| p != 0));
    assert_eq!(
        original.pixels, actor.pixels,
        "clipped actor restores encoded alpha"
    );
    let hidden =
        icons.draw_models_alpha(&[(&model, &colors, &model.vertices)], [100, 100], None, 255)?;
    assert!(hidden.pixels.iter().all(|&p| p == 0));
    model.alpha = Some(vec![255]);
    let encoded =
        icons.draw_models_alpha(&[(&model, &colors, &model.vertices)], [100, 100], None, 0)?;
    model.alpha = Some(vec![253]);
    let normalized =
        icons.draw_models_alpha(&[(&model, &colors, &model.vertices)], [100, 100], None, 0)?;
    assert_ne!(
        encoded.pixels, normalized.pixels,
        "near clipping retains raw255"
    );
    model.vertices[0][2] = 300.;
    model.alpha = Some(vec![0]);
    let original =
        icons.draw_models_alpha(&[(&model, &colors, &model.vertices)], [100, 100], None, 0)?;
    let actor =
        icons.draw_models_alpha(&[(&model, &colors, &model.vertices)], [100, 100], None, 128)?;
    assert_ne!(
        original.pixels, actor.pixels,
        "normal actor retains override"
    );
    println!(
        "Cache-backed actor near alpha reset, raw255, full suppression and normal override passed"
    );
    Ok(())
}
