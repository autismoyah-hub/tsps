use gielinor_client::{
    buffer::Reader,
    disk_cache::{name_hash, DiskCache},
    interface_render::Renderer,
    interface_state::Interfaces,
    item::Definitions,
    raster::Raster,
    widget::Widget,
    Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let items = Definitions::load(&mut cache)?;
    let font = cache
        .reference(13)?
        .groups
        .iter()
        .find(|g| g.name_hash == Some(name_hash("p11_full")))
        .unwrap()
        .id as i32;
    let mut renderer = Renderer::load(&mut cache, 0.8)?;
    let mut r = Reader::new(include_bytes!("../tests/fixtures/widget-quantity-jvm.bin"));
    for case in 0..r.u32()? {
        let count = r.u32()? as i32;
        let len = r.u32()? as usize;
        let quantity = std::str::from_utf8(r.bytes(len)?).unwrap();
        let id = if case % 2 == 0 { 995 } else { 1277 };
        let item = &items.items[&id];
        let text = if (item.stackable == 1 || count != 1) && count != -1 {
            format!("<col=ff9040>{}</col> x{quantity}", item.name)
        } else {
            item.name.clone()
        };
        let mut widget = Widget::empty(0, 4);
        widget.if3 = true;
        widget.font = font;
        widget.raw_size = [if case % 3 == 0 { 140 } else { 320 }, 64];
        widget.colors = [0x55aa88; 4];
        widget.text_shadow = case % 2 == 0;
        widget.text_alignment = [(case % 3) as u8, ((case / 3) % 3) as u8];
        let mut state = Interfaces::default();
        state.install_group(0, vec![widget])?;
        state.root = Some(0);
        state.runtime.get_mut(&0).unwrap().item = id as i32;
        state.runtime.get_mut(&0).unwrap().count = count;
        let layout = state.layout([320, 64])?;
        let mut actual = Raster::new(320, 64, 320 * 64)?;
        actual.clear(0xff302b25);
        let report = renderer.draw(&mut cache, &items, &state, &layout, &mut actual)?;
        assert!(report.pending.is_empty());
        assert_eq!(report.drawn, 1);
        state.runtime.get_mut(&0).unwrap().item = -1;
        state.widgets.get_mut(&0).unwrap().text = text;
        let mut expected = Raster::new(320, 64, 320 * 64)?;
        expected.clear(0xff302b25);
        renderer.draw(&mut cache, &items, &state, &layout, &mut expected)?;
        assert_eq!(
            actual.pixels(),
            expected.pixels(),
            "case {case}, count {count}"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("4096 Java quantity strings pass cache widget rendering, wrapping, alignment and stackability selection");
    Ok(())
}
