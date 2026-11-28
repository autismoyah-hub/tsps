use gielinor_client::{
    buffer::Reader, crypto::crc32, disk_cache::DiskCache, interface_render::Renderer,
    interface_state::Interfaces, item::Definitions, raster::Raster, widget::Widget, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let items = Definitions::load(&mut cache)?;
    let mut renderer = Renderer::load(&mut cache, 0.8)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-hover-raster-jvm.bin"
    ));
    for case in 0..r.u32()? {
        let mut w = Widget::empty(0, 3);
        w.if3 = false;
        w.raw_position = [20, 10];
        w.raw_size = [50, 30];
        for color in &mut w.colors {
            *color = r.u32()?;
        }
        let condition = r.u8()? != 0;
        let hovered = r.u8()? != 0;
        w.filled = r.u8()? != 0;
        let expected = r.u32()?;
        // A redirect can select a widget even when its normal hover color is zero.
        w.mouse_over = 0;
        let mut state = Interfaces::default();
        state.install_group(0, vec![w])?;
        state.root = Some(0);
        state.runtime.get_mut(&0).unwrap().condition = condition;
        let layout = state.layout([160, 96])?;
        state.tick_tooltip(&layout, if hovered { [30, 20] } else { [0, 0] }, false);
        let mut raster = Raster::new(160, 96, 160 * 96)?;
        raster.clear(0xff302b25);
        let report = renderer.draw(&mut cache, &items, &state, &layout, &mut raster)?;
        assert!(report.pending.is_empty());
        let pixels: Vec<_> = raster
            .pixels()
            .iter()
            .flat_map(|p| p.to_be_bytes())
            .collect();
        assert_eq!(crc32(&pixels), expected, "case {case}");
    }
    assert_eq!(r.remaining(), 0);
    println!(
        "4096 original conditional/zero-fallback hover colors pass actual interface rendering"
    );
    Ok(())
}
