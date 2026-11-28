//! Original pixel fixtures through cache-backed interface submission.
use gielinor_client::{
    buffer::Reader, crypto::crc32, disk_cache::DiskCache, interface_render::Renderer,
    interface_state::Interfaces, item::Definitions, raster::Raster, widget::Widget, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let items = Definitions::load(&mut cache)?;
    let mut renderer = Renderer::load(&mut cache, 0.8)?;
    let mut interfaces = Interfaces::default();
    interfaces.install_group(0, vec![Widget::empty(0, 9)])?;
    interfaces.root = Some(0);
    let mut r = Reader::new(include_bytes!("../tests/fixtures/widget-line-jvm.bin"));
    let mut checked = 0;
    for case in 0..r.u32()? {
        let a = [r.u32()? as i32, r.u32()? as i32];
        let b = [r.u32()? as i32, r.u32()? as i32];
        let width = r.u32()? as u8;
        let color = r.u32()?;
        let clip = [
            r.u32()? as i32,
            r.u32()? as i32,
            r.u32()? as i32,
            r.u32()? as i32,
        ];
        let expected = r.u32()?;
        // ga.aa culls IF3 lines by endpoint bounds before executing yu.ej.
        if (0..2).any(|axis| {
            a[axis].min(b[axis]) >= clip[axis + 2] || a[axis].max(b[axis]) + 1 <= clip[axis]
        }) {
            continue;
        }
        let w = interfaces.widgets.get_mut(&0).unwrap();
        w.raw_position = a;
        w.raw_size = [b[0] - a[0], b[1] - a[1]];
        w.line_width = width;
        w.colors[0] = color;
        let layout = interfaces.layout([64, 48])?;
        let mut raster = Raster::new(64, 48, 64 * 48)?;
        raster.clear(0xff302b25);
        raster.set_clip(clip[0], clip[1], clip[2], clip[3]);
        let report = renderer.draw(&mut cache, &items, &interfaces, &layout, &mut raster)?;
        assert!(report.pending.is_empty(), "{report:?}");
        let pixels: Vec<_> = raster
            .pixels()
            .iter()
            .flat_map(|p| p.to_be_bytes())
            .collect();
        assert_eq!(crc32(&pixels), expected, "interface fixture {case}");
        assert_eq!(
            raster.clip(),
            clip.map(|v| v as usize),
            "caller clip restored"
        );
        checked += 1;
    }
    assert!(checked > 500);
    println!("{checked} original clipped line pixel fixtures pass actual interface submission");
    Ok(())
}
