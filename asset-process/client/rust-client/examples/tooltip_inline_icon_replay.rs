use gielinor_client::{buffer::Reader, crypto::crc32, font::Font, raster::Raster};
fn main() -> gielinor_client::Result<()> {
    use gielinor_client::{
        disk_cache::DiskCache, interface_render::Renderer, interface_state::Interfaces,
        item::Definitions, widget::Widget,
    };
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let items = Definitions::load(&mut cache)?;
    let mut renderer = Renderer::load(&mut cache, 0.8)?;
    let mut interfaces = Interfaces::default();
    let mut widget = Widget::empty(0, 8);
    widget.if3 = false;
    interfaces.install_group(0, vec![widget])?;
    interfaces.root = Some(0);
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/tooltip-inline-icon-jvm.bin"
    ));
    assert_eq!(r.u32().unwrap(), 495);
    let n = r.u32().unwrap() as usize;
    let metrics = r.bytes(n).unwrap();
    let n = r.u32().unwrap() as usize;
    let _font = Font::decode(metrics, r.bytes(n).unwrap()).unwrap();
    let n = r.u32()? as usize;
    r.bytes(n)?;
    for case in 0..r.u32().unwrap() {
        let rect: [i32; 4] = std::array::from_fn(|_| r.u32().unwrap() as i32);
        let clip: [i32; 4] = std::array::from_fn(|_| r.u32().unwrap() as i32);
        let values: [i32; 5] = std::array::from_fn(|_| r.u32().unwrap() as i32);
        let age = r.u32().unwrap();
        let selected = r.u8().unwrap() != 0;
        let n = r.u32().unwrap() as usize;
        let text = std::str::from_utf8(r.bytes(n).unwrap()).unwrap();
        let expected = r.u32().unwrap();
        let mut raster = Raster::new(160, 96, 160 * 96).unwrap();
        raster.clear(0xff302b25);
        raster.set_clip(clip[0], clip[1], clip[2], clip[3]);
        interfaces.tooltip = Default::default();
        if selected {
            for _ in 0..=age {
                interfaces
                    .tooltip
                    .tick(Some((0, interfaces.runtime[&0].instance)));
            }
        }
        let w = interfaces.widgets.get_mut(&0).unwrap();
        w.raw_position = [rect[0], rect[1]];
        w.raw_size = [rect[2], rect[3]];
        w.text = text.to_owned();
        interfaces.runtime.get_mut(&0).unwrap().cs1_values = values.to_vec();
        let layout = interfaces.layout([160, 96])?;
        let report = renderer.draw(&mut cache, &items, &interfaces, &layout, &mut raster)?;
        assert!(report.pending.is_empty(), "{report:?}");
        assert_eq!(raster.clip(), clip.map(|v| v as usize));
        let pixels: Vec<_> = raster
            .pixels()
            .iter()
            .flat_map(|p| p.to_be_bytes())
            .collect();
        assert_eq!(crc32(&pixels), expected, "case {case} {rect:?} {text}");
    }
    assert_eq!(r.remaining(), 0);
    println!("4096 original inline-icon tooltip rasters pass actual interface renderer, including off-widget parent clipping");
    Ok(())
}
