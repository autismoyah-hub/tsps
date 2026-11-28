//! Original pixel fixtures through cache-backed interface submission.
use gielinor_client::{
    buffer::Reader, disk_cache::DiskCache, interface_render::Renderer, interface_state::Interfaces,
    item::Definitions, raster::Raster, widget::Widget, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let items = Definitions::load(&mut cache)?;
    let mut renderer = Renderer::load(&mut cache, 0.8)?;
    use gielinor_client::sprite_pixels::Pixels;
    let sprite_id = cache
        .reference(8)?
        .groups
        .iter()
        .find(|g| g.name_hash == Some(gielinor_client::disk_cache::name_hash("compass")))
        .unwrap()
        .id;
    let sprite = Pixels::load(&mut cache, sprite_id)?;
    let mut interfaces = Interfaces::default();
    let mut widget = Widget::empty(0, 5);
    widget.sprites = [sprite_id as i32; 2];
    interfaces.install_group(0, vec![widget])?;
    interfaces.root = Some(0);
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-sprite-rotation-jvm.bin"
    ));
    for case in 0..r.u32()? {
        let rect: [i32; 4] = std::array::from_fn(|_| r.u32().unwrap() as i32);
        let clip: [i32; 4] = std::array::from_fn(|_| r.u32().unwrap() as i32);
        let angle = r.u32()? as u16;
        let opacity = r.u32()? as u8;
        let tiled = r.u8()? != 0;
        let if3 = r.u8()? != 0;
        r.u32()?; // Synthetic Java fixture pixels are covered in sprite_rotation tests.
        let w = interfaces.widgets.get_mut(&0).unwrap();
        w.raw_position = [rect[0], rect[1]];
        w.raw_size = [rect[2], rect[3]];
        w.sprite_angle = angle;
        w.sprite_tiling = tiled;
        w.if3 = if3;
        w.transparency = opacity;
        let layout = interfaces.layout([160, 96])?;
        let mut expected = Raster::new(160, 96, 160 * 96)?;
        expected.clear(0xff302b25);
        expected.set_clip(clip[0], clip[1], clip[2], clip[3]);
        sprite.draw_widget(&mut expected, rect, angle, tiled, if3, 256 - opacity as u16)?;
        let mut actual = Raster::new(160, 96, 160 * 96)?;
        actual.clear(0xff302b25);
        actual.set_clip(clip[0], clip[1], clip[2], clip[3]);
        let report = renderer.draw(&mut cache, &items, &interfaces, &layout, &mut actual)?;
        assert!(report.pending.is_empty(), "{report:?}");
        assert_eq!(
            actual.pixels(),
            expected.pixels(),
            "cache sprite case {case}"
        );
        assert_eq!(actual.clip(), clip.map(|v| v as usize));
    }
    assert_eq!(r.remaining(), 0);
    println!(
        "4096 cache compass sprite cases pass interface submission and parent-clip restoration"
    );
    Ok(())
}
