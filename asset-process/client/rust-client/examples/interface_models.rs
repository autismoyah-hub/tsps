use gielinor_client::{
    disk_cache::DiskCache, interface_render::Renderer, interface_state::Interfaces,
    item::Definitions, raster::Raster, widget::Widget,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    let mut cache = DiskCache::open(&args[1], 64 << 20)?;
    let defs = Definitions::load(&mut cache)?;
    let mut state = Interfaces::default();
    let ids = [4151, 11802, 1127, 995, 11840, 6570, 5698, 4151];
    let widgets = ids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let d = &defs.items[id];
            let mut w = Widget::empty(i as u32, 6);
            w.raw_position = [(i % 4) as i32 * 190, (i / 4) as i32 * 240];
            w.raw_size = [190, 240];
            w.model_angles = d.angles.map(|a| a as u16);
            w.model_offset = d.offset;
            w.model_zoom = (d.zoom / 3).max(250) as u16;
            w.model_orthographic = i >= 4;
            w
        })
        .collect();
    state.install_group(0, widgets)?;
    state.root = Some(0);
    for (i, &id) in ids.iter().enumerate() {
        let runtime = state.runtime.get_mut(&(i as u32)).unwrap();
        runtime.item = id as i32;
        runtime.count = 1;
    }
    let mut raster = Raster::new(760, 480, 1 << 20)?;
    raster.clear(0xff302b25);
    let layout = state.layout([760, 480])?;
    let mut renderer = Renderer::load(&mut cache, 0.8)?;
    let report = renderer.draw(&mut cache, &defs, &state, &layout, &mut raster)?;
    assert!(report.pending.is_empty(), "{report:?}");
    assert_eq!(report.drawn, 8);
    std::fs::write(&args[2], raster.bmp()?)?;
    println!("{report:?}");
    Ok(())
}
