use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_render::Renderer,
    raster::Raster,
    vm::{Host, Stacks},
    widget::Widget,
    Result,
};
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let mut widget = Widget::empty(0, 0);
    widget.content = 1401;
    widget.raw_size = [765, 503];
    c.interfaces.install_group(0, vec![widget])?;
    c.interfaces.root = Some(0);
    let mut renderer = Renderer::load(&mut c.scripts.source.cache, 0.8)?;
    let layout = c.interfaces.layout([765, 503])?;
    let mut initial = None;
    for id in [0, 1, 25, 0] {
        let mut stack = Stacks::default();
        stack.push_int(id)?;
        c.extended(6602, false, &mut stack)?;
        assert_eq!(c.world_map.current, Some(id));
        assert!(stack.ints().is_empty());
        let image = c
            .world_map
            .overview_image(&mut c.scripts.source.cache)?
            .unwrap();
        let again = c
            .world_map
            .overview_image(&mut c.scripts.source.cache)?
            .unwrap();
        assert!(std::sync::Arc::ptr_eq(&image, &again));
        renderer.map_overview = Some(image);
        renderer.map_overview_flash = c
            .world_map
            .overview_markers(&mut c.scripts.source.cache, true)?;
        assert!(c.world_map.renderer.as_ref().unwrap().ready(id));
        let mut query = Stacks::default();
        c.extended(6639, false, &mut query)?;
        assert_eq!(query.ints().len(), 2);

        let mut raster = Raster::new(765, 503, 765 * 503)?;
        raster.clear(0xff302b25);
        let report = renderer.draw(
            &mut c.scripts.source.cache,
            &c.items,
            &c.interfaces,
            &layout,
            &mut raster,
        )?;
        assert_eq!(report.drawn, 1);
        assert!(report.pending.is_empty(), "{report:?}");
        assert!(raster.pixels().iter().any(|&v| v != 0xff302b25));
        if id == 0 {
            if let Some(pixels) = &initial {
                assert_eq!(raster.pixels(), pixels);
            } else {
                initial = Some(raster.pixels().to_vec());
                std::fs::write("work/map-overview-main.bmp", raster.bmp()?)?;
            }
        }
    }

    let area = c.world_map.area().unwrap().clone();
    let map = c.world_map.renderer.as_mut().unwrap();
    let key = map.icon_start(&area)?.pop_front().unwrap();
    let icon = map.event(key).unwrap();
    c.world_map.settings.flashing = Some([icon.element].into_iter().collect());
    c.world_map.settings.flash_cycle = 1;
    c.world_map.settings.period = 50;
    c.world_map.settings.hidden = true;
    c.world_map.settings.excluded.insert(icon.element);
    renderer.map_overview_flash = c
        .world_map
        .overview_markers(&mut c.scripts.source.cache, true)?;
    assert!(!renderer
        .map_overview_flash
        .as_ref()
        .unwrap()
        .points
        .is_empty());
    let mut flash = Raster::new(765, 503, 765 * 503)?;
    flash.clear(0xff302b25);
    renderer.draw(
        &mut c.scripts.source.cache,
        &c.items,
        &c.interfaces,
        &layout,
        &mut flash,
    )?;
    assert_ne!(flash.pixels(), initial.as_ref().unwrap());
    std::fs::write("work/map-overview-flash.bmp", flash.bmp()?)?;
    assert_eq!(c.world_map.settings.flash_cycle, 1);
    c.world_map.settings.flash_cycle = 25;
    renderer.map_overview_flash = c
        .world_map
        .overview_markers(&mut c.scripts.source.cache, true)?;
    let mut off = Raster::new(765, 503, 765 * 503)?;
    off.clear(0xff302b25);
    renderer.draw(
        &mut c.scripts.source.cache,
        &c.items,
        &c.interfaces,
        &layout,
        &mut off,
    )?;
    assert_eq!(off.pixels(), initial.as_ref().unwrap());
    c.prepare_login()?;
    assert_eq!(c.world_map.current, Some(0));
    let image = c
        .world_map
        .overview_image(&mut c.scripts.source.cache)?
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(
        &image,
        renderer.map_overview.as_ref().unwrap()
    ));
    println!("Map selection through ClientState, three actual PNG overviews, content1401 dispatch, image-cache reuse, overview-only query readiness, flash projection/pulse/filter independence and login preservation passed");
    Ok(())
}
