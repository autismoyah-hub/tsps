use gielinor_client::{
    buffer::Reader, crypto::crc32, disk_cache::DiskCache, editable_text_layout::Options,
    font::Font, interface_render::Renderer, interface_state::Interfaces, item::Definitions,
    raster::Raster, widget::Widget,
};
fn text(r: &mut Reader) -> Vec<u16> {
    let n = r.u16().unwrap() as usize;
    std::str::from_utf8(r.bytes(n).unwrap())
        .unwrap()
        .encode_utf16()
        .collect()
}
fn main() {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20).unwrap();
    let defs = Definitions::load(&mut cache).unwrap();
    let mut renderer = Renderer::load(&mut cache, 0.8).unwrap();
    let font = Font::load(&mut cache, 495).unwrap();
    let mut r = Reader::new(include_bytes!("../tests/fixtures/widget-editor-jvm.bin"));
    for case in 0..r.u32().unwrap() {
        let [x, y, width, height, color, selected, background, placeholder_color, shadow, ax, ay, line, mask, wrap, max_width, anchor, cursor, sx, sy, clock] =
            std::array::from_fn(|_| r.u32().unwrap() as i32);
        let focused = r.u8().unwrap() != 0;
        let value = text(&mut r);
        let placeholder = text(&mut r);
        let expected = r.u32().unwrap();
        let mut widget = Widget::empty(0, 12);
        widget.font = 495;
        widget.raw_position = [x, y];
        widget.raw_size = [width, height];
        widget.colors[0] = color as u32;
        widget.text_shadow = shadow != -1;
        widget.sprite_shadow = shadow as u32;
        let mut state = Interfaces::default();
        state.install_group(0, vec![widget]).unwrap();
        state.root = Some(0);
        let e = state.runtime.get_mut(&0).unwrap().editor.as_mut().unwrap();
        e.text.value = value;
        e.text.ready = true;
        e.text.anchor = anchor;
        e.text.cursor = cursor;
        e.text.focused = focused;
        e.text.clock = clock as u8;
        e.placeholder = placeholder;
        e.metrics = Some(font.metrics.clone());
        e.options = Options {
            width: max_width,
            max_lines: i32::MAX,
            wrap,
            mask,
            align: [ax, ay],
            line_height: line,
        };
        e.scroll = [sx, sy];
        e.colors = [selected as u32, background as u32, placeholder_color as u32];
        let layout = state.layout([160, 96]).unwrap();
        let mut raster = Raster::new(160, 96, 160 * 96).unwrap();
        raster.clear(0xff302b25);
        let report = renderer
            .draw(&mut cache, &defs, &state, &layout, &mut raster)
            .unwrap();
        assert!(report.pending.is_empty());
        let pixels = raster
            .pixels()
            .iter()
            .flat_map(|p| (p | 0xff000000).to_be_bytes())
            .collect::<Vec<_>>();
        assert_eq!(crc32(&pixels), expected, "case {case} focus {focused}");
    }
    assert_eq!(r.remaining(), 0);
    println!("2048 original ga.ae kind12 rasters pass cache widget renderer, including placeholders/focus/selection");
}
