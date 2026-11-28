use gielinor_client::{
    buffer::Reader, crypto::crc32, editable_text_layout::Options, font::Font, raster::Raster,
};
fn main() {
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/editable-text-raster-jvm.bin"
    ));
    let n = r.u32().unwrap() as usize;
    let metrics = r.bytes(n).unwrap();
    let n = r.u32().unwrap() as usize;
    let _font = Font::decode(metrics, r.bytes(n).unwrap()).unwrap();
    let mut cache =
        gielinor_client::disk_cache::DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)
            .unwrap();
    let font = Font::load(&mut cache, 495).unwrap();
    let defs = gielinor_client::item::Definitions::load(&mut cache).unwrap();
    let mut renderer = gielinor_client::interface_render::Renderer::load(&mut cache, 0.8).unwrap();
    let mut checked = 0;
    for case in 0..r.u32().unwrap() {
        let [width, max_lines, wrap, mask, ax, ay, line_height] =
            std::array::from_fn(|_| r.u32().unwrap() as i32);
        let n = r.u16().unwrap() as usize;
        let text = std::str::from_utf8(r.bytes(n).unwrap()).unwrap();
        let text16 = text.encode_utf16().collect::<Vec<_>>();
        let count = r.u32().unwrap();
        r.bytes(count as usize * 8).unwrap();
        let options = Options {
            width,
            max_lines,
            wrap,
            mask,
            align: [ax, ay],
            line_height,
        };
        let [x, y, w, h, sx, sy, cursor, anchor, color, shadow, selected, background] =
            std::array::from_fn(|_| r.u32().unwrap() as i32);
        let caret = r.u8().unwrap() != 0;
        let show_last = r.u8().unwrap() != 0;
        let expected = r.u32().unwrap();
        if !show_last && mask == 1 {
            continue;
        }
        let mut state = gielinor_client::interface_state::Interfaces::default();
        let mut widget = gielinor_client::widget::Widget::empty(0, 12);
        widget.font = 495;
        widget.raw_position = [x, y];
        widget.raw_size = [w, h];
        widget.colors[0] = color as u32;
        widget.text_shadow = shadow != -1;
        widget.sprite_shadow = shadow as u32;
        state.install_group(0, vec![widget]).unwrap();
        state.root = Some(0);
        let editor = state.runtime.get_mut(&0).unwrap().editor.as_mut().unwrap();
        editor.text.ready = true;
        editor.text.value = text16;
        editor.text.cursor = cursor;
        editor.text.anchor = anchor;
        editor.text.focused = true;
        editor.text.clock = if caret { 0 } else { 30 };
        editor.options = options;
        editor.scroll = [sx, sy];
        editor.colors = [selected as u32, background as u32, 0];
        editor.metrics = Some(font.metrics.clone());
        let layout = state.layout([160, 96]).unwrap();
        let mut raster = Raster::new(160, 96, 160 * 96).unwrap();
        raster.clear(0xff302b25);
        let report = renderer
            .draw(&mut cache, &defs, &state, &layout, &mut raster)
            .unwrap();
        assert!(report.pending.is_empty());
        assert_eq!(report.drawn, 1);
        checked += 1;
        let pixels = raster
            .pixels()
            .iter()
            .flat_map(|p| (p | 0xff000000).to_be_bytes())
            .collect::<Vec<_>>();
        assert_eq!(
            crc32(&pixels),
            expected,
            "case {case} {options:?}  {text:?}"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("{checked} original editable-text rasters pass actual widget renderer");
}
