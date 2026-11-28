use gielinor_client::{
    buffer::Reader,
    crypto::crc32,
    editable_text_layout::Options,
    editable_text_raster::{draw, Paint},
    font::Font,
    raster::Raster,
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
        let paint = Paint {
            rect: [x, y, w, h],
            scroll: [sx, sy],
            cursor: cursor as usize,
            anchor: anchor as usize,
            color: color as u32,
            shadow: (shadow != -1).then_some(shadow as u32),
            selected: selected as u32,
            background: background as u32,
            caret,
            show_last,
        };
        let mut raster = Raster::new(160, 96, 160 * 96).unwrap();
        raster.clear(0xff302b25);
        draw(&mut raster, &font, &text16, options, paint).unwrap();
        let pixels = raster
            .pixels()
            .iter()
            .flat_map(|p| (p | 0xff000000).to_be_bytes())
            .collect::<Vec<_>>();
        assert_eq!(
            crc32(&pixels),
            expected,
            "case {case} {options:?} {paint:?} {text:?}"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("4096 original editable-text selection/caret rasters pass actual cache font");
}
