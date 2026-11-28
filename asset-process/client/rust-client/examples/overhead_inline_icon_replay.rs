use gielinor_client::{
    buffer::Reader,
    crypto::crc32,
    font::Font,
    overhead::{self, Entry},
    raster::Raster,
};
fn bytes<'a>(r: &mut Reader<'a>) -> &'a [u8] {
    let n = r.u32().unwrap() as usize;
    r.bytes(n).unwrap()
}
fn main() {
    let mut r = Reader::new(include_bytes!("../tests/fixtures/overhead-icon-jvm.bin"));
    let mut assets = Reader::new(bytes(&mut r));
    let fallback = assets.u32().unwrap();
    let mut cache =
        gielinor_client::disk_cache::DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)
            .unwrap();
    let font = Font::load(&mut cache, fallback).unwrap();
    let icons = gielinor_client::font_icons::load(&mut cache).unwrap();
    for case in 0..r.u32().unwrap() {
        let count = r.u32().unwrap();
        let cycle = r.u32().unwrap() as i32;
        let disabled = r.u32().unwrap() != 0;
        let mut entries = Vec::new();
        for _ in 0..count {
            let [x, y, color, effect, remaining]: [i32; 5] =
                std::array::from_fn(|_| r.u32().unwrap() as i32);
            let length = r.u16().unwrap() as usize;
            let text = String::from_utf8(r.bytes(length).unwrap().to_vec()).unwrap();
            let n = r.u32().unwrap();
            let pattern = if n == 0 {
                None
            } else {
                Some((0..n).map(|_| r.u32().unwrap()).collect())
            };
            entries.push(Entry {
                position: [x, y],
                text,
                color: color as u8,
                effect: effect as u8,
                remaining,
                pattern,
            });
        }
        let expected_y: Vec<_> = (0..count).map(|_| r.u32().unwrap() as i32).collect();
        let expected_crc = r.u32().unwrap();
        let mut raster = Raster::from_pixels(320, 160, vec![0xff203040; 320 * 160]).unwrap();
        overhead::draw_with_icons(&font, &mut raster, &mut entries, cycle, disabled, &icons)
            .unwrap();
        assert_eq!(
            entries.iter().map(|e| e.position[1]).collect::<Vec<_>>(),
            expected_y,
            "case {case}"
        );
        let pixels: Vec<_> = raster
            .pixels()
            .iter()
            .flat_map(|p| p.to_be_bytes())
            .collect();
        assert_eq!(crc32(&pixels), expected_crc, "case {case}: {entries:?}");
    }
    assert_eq!(r.remaining(), 0);
    println!("4096 overhead inline-image effect rasters pass actual cache fonts and icons");
}
