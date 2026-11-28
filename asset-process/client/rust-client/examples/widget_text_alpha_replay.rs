use gielinor_client::{
    buffer::Reader, crypto::crc32, disk_cache::DiskCache, interface_render::Renderer,
    interface_state::Interfaces, item::Definitions, raster::Raster, widget::Widget, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let items = Definitions::load(&mut cache)?;
    let mut renderer = Renderer::load(&mut cache, 0.8)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-text-alpha-jvm.bin"
    ));
    let font = r.u32()? as i32;
    for _ in 0..2 {
        let n = r.u32()? as usize;
        r.bytes(n)?;
    }
    for case in 0..r.u32()? {
        let [opacity, x, y, width, height, color, ax, ay, line] =
            std::array::from_fn(|_| r.u32().unwrap() as i32);
        let shadow = r.u8()? != 0;
        let if3 = r.u8()? != 0;
        let n = r.u32()? as usize;
        let text = std::str::from_utf8(r.bytes(n)?).unwrap();
        let expected = r.u32()?;
        let mut w = Widget::empty(0, 4);
        w.if3 = if3;
        w.font = font;
        w.raw_position = [x, y];
        w.raw_size = [width, height];
        w.text = text.into();
        w.colors = [color as u32; 4];
        w.text_shadow = shadow;
        w.text_alignment = [ax as u8, ay as u8];
        w.line_height = line as u8;
        w.transparency = opacity as u8;
        let mut state = Interfaces::default();
        state.install_group(0, vec![w])?;
        state.root = Some(0);
        let layout = state.layout([160, 96])?;
        let mut raster = Raster::new(160, 96, 160 * 96)?;
        raster.clear(0xff302b25);
        let report = renderer.draw(&mut cache, &items, &state, &layout, &mut raster)?;
        assert!(report.pending.is_empty());
        assert_eq!(report.drawn, 1);
        let bytes: Vec<_> = raster
            .pixels()
            .iter()
            .flat_map(|p| (p | 0xff000000).to_be_bytes())
            .collect();
        assert_eq!(crc32(&bytes), expected, "case {case} opacity {opacity}");
    }
    assert_eq!(r.remaining(), 0);
    println!("4096 original ga.ae text opacity, wrapping, shadow and parent clipping rasters pass Renderer");
    Ok(())
}
