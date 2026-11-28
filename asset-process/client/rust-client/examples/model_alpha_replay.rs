//! Cache-backed pre-lit interface model after an animation changes face alpha.
use gielinor_client::{disk_cache::DiskCache, item_icon::Icons, model::Model, Result};
fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("cache path");
    let mut cache = DiskCache::open(path, 64 << 20)?;
    let icons = Icons::load(&mut cache, 0.8)?;
    let mut model = Model::decode(&[0; 18])?;
    model.vertices = vec![[-30., -30., 300.], [0., 30., 300.], [30., -30., 300.]];
    model.faces = vec![[0, 1, 2]];
    model.colors = vec![100];
    let colors = [[100, 100, -1]];
    model.alpha = Some(vec![253]);
    let expected = icons.draw_model(&model, &colors, &model.vertices, [100, 100], None)?;
    model.alpha = Some(vec![255]);
    let actual = icons.draw_model(&model, &colors, &model.vertices, [100, 100], None)?;
    assert_eq!(
        actual.pixels, expected.pixels,
        "animated encoded255 dispatches alpha253"
    );
    assert!(actual.pixels.iter().any(|&v| v != 0));
    model.alpha = Some(vec![0]);
    let opaque = icons.draw_model(&model, &colors, &model.vertices, [100, 100], None)?;
    assert_ne!(actual.pixels, opaque.pixels);
    model.alpha = Some(vec![254]);
    let shifted = icons.draw_model(&model, &colors, &model.vertices, [100, 100], None)?;
    assert!(
        shifted.pixels.iter().all(|&v| v == 0),
        "flat254 shifts the clear background"
    );
    println!("Cache-backed animated interface alpha255/253 equality, opaque distinction and flat254 background shift passed");
    Ok(())
}
