//! Cache-backed widget renderer: mixed default priorities, integer projection and alpha overrides.
use gielinor_client::{disk_cache::DiskCache, item_icon::Icons, model::Model, Result};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let icons = Icons::load(&mut cache, 0.8)?;
    let mut first = Model::decode(&[0; 18])?;
    first.vertices = vec![[-30., -30., 300.], [0., 30., 300.], [30., -30., 300.]];
    first.faces = vec![[0, 1, 2]];
    first.colors = vec![100];
    first.priority = 5;
    let mut second = first.clone();
    second.priority = 0;
    let a = [[100, 100, -1]];
    let b = [[12000, 12000, -1]];
    for orthographic in [None, Some(300)] {
        let expected = icons.draw_widget_models(
            &[(&first, &a, &first.vertices)],
            [100, 100],
            orthographic,
            0,
            0,
        )?;
        let other = icons.draw_widget_models(
            &[(&second, &b, &second.vertices)],
            [100, 100],
            orthographic,
            0,
            0,
        )?;
        assert_ne!(expected.pixels, other.pixels);
        let combined = icons.draw_widget_models(
            &[
                (&first, &a, &first.vertices),
                (&second, &b, &second.vertices),
            ],
            [100, 100],
            orthographic,
            0,
            0,
        )?;
        assert_eq!(
            combined.pixels, expected.pixels,
            "distinct default priorities are promoted across combined model parts"
        );
        let faded = icons.draw_widget_models(
            &[(&first, &a, &first.vertices)],
            [100, 100],
            orthographic,
            128,
            0,
        )?;
        assert_ne!(faded.pixels, expected.pixels);
        let hidden = icons.draw_widget_models(
            &[(&first, &a, &first.vertices)],
            [100, 100],
            orthographic,
            255,
            0,
        )?;
        assert!(hidden.pixels.iter().all(|&p| p == 0));
        first.alpha = Some(vec![255]);
        let encoded = icons.draw_widget_models(
            &[(&first, &a, &first.vertices)],
            [100, 100],
            orthographic,
            0,
            0,
        )?;
        first.alpha = Some(vec![253]);
        let normalized = icons.draw_widget_models(
            &[(&first, &a, &first.vertices)],
            [100, 100],
            orthographic,
            0,
            0,
        )?;
        assert_eq!(encoded.pixels, normalized.pixels);
        first.alpha = None;
    }
    println!("Widget perspective/orthographic projection, merged priorities, encoded alpha and actor overrides passed");
    Ok(())
}
