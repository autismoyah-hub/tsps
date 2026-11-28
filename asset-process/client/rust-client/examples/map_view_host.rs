use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_render::Renderer,
    raster::Raster,
    vm::{Host, Stacks},
    widget::Widget,
    Result,
};
fn command(c: &mut ClientState, op: u16, value: i32) -> Result<()> {
    let mut s = Stacks::default();
    s.push_int(value)?;
    assert_eq!(
        c.extended(op, false, &mut s)?,
        gielinor_client::vm::Control::Continue
    );
    assert!(s.ints().is_empty());
    Ok(())
}
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    c.canvas = [765, 503];
    let mut widget = Widget::empty(0, 0);
    widget.content = 1400;
    widget.raw_size = c.canvas;
    c.interfaces.install_group(0, vec![widget])?;
    c.interfaces.root = Some(0);
    let mut renderer = Renderer::load(&mut c.scripts.source.cache, 0.8)?;
    let layout = c.interfaces.layout(c.canvas)?;
    for id in [0, 1, 25, 0] {
        command(&mut c, 6602, id)?;
        if id == 0 {
            command(&mut c, 6607, 3200 << 14 | 3200)?;
        }
        c.world_map.zoom = 1.5;
        c.world_map.target_zoom = 1.5;
        let image = c
            .world_map
            .detailed_image(&mut c.scripts.source.cache, true, c.canvas, 0.8)?
            .unwrap();
        let again = c
            .world_map
            .detailed_image(&mut c.scripts.source.cache, true, c.canvas, 0.8)?
            .unwrap();
        assert!(std::sync::Arc::ptr_eq(&image, &again));
        assert_eq!(c.world_map.display_size, [510, 336]);
        renderer.map_detail.insert(0, image);
        let mut raster = Raster::new(765, 503, 765 * 503)?;
        let report = renderer.draw(
            &mut c.scripts.source.cache,
            &c.items,
            &c.interfaces,
            &layout,
            &mut raster,
        )?;
        assert_eq!(report.drawn, 1);
        assert!(report.pending.is_empty());
        assert!(raster.pixels().iter().any(|&v| v != raster.pixels()[0]));
        if id == 0 {
            std::fs::write("work/map-view-lumbridge.bmp", raster.bmp()?)?;
        }
    }
    let base = c
        .world_map
        .detailed_image(&mut c.scripts.source.cache, true, c.canvas, 0.8)?
        .unwrap();
    c.heatmap.enabled(true);
    c.heatmap.refresh(
        [0, 0],
        std::iter::repeat([3200 * 128, 3200 * 128]).take(128),
        77,
    );
    let hot = c.world_map.compose_heatmap(base.clone(), &mut c.heatmap)?;
    assert_ne!(base.pixels, hot.pixels);
    let again = c.world_map.compose_heatmap(base.clone(), &mut c.heatmap)?;
    assert!(std::sync::Arc::ptr_eq(&hot, &again));
    std::fs::write(
        "work/map-view-heatmap.bmp",
        Raster::from_pixels(hot.size[0], hot.size[1], hot.pixels.clone())?.bmp()?,
    )?;
    c.heatmap.refresh(
        [0, 0],
        std::iter::repeat([3220 * 128, 3200 * 128]).take(128),
        78,
    );
    let moved = c.world_map.compose_heatmap(base.clone(), &mut c.heatmap)?;
    assert_ne!(hot.pixels, moved.pixels);
    c.heatmap.enabled(false);
    let plain = c.world_map.compose_heatmap(base.clone(), &mut c.heatmap)?;
    assert!(std::sync::Arc::ptr_eq(&base, &plain));
    c.world_map.target_zoom = 3.0;
    c.tick_interfaces(false)?;
    assert_eq!(c.world_map.zoom, 1.55);
    c.interfaces.root = None;
    let zoom = c.world_map.zoom;
    c.tick_interfaces(false)?;
    assert_eq!(c.world_map.zoom, zoom);
    c.world_map.zoom = 8.0;
    c.world_map.target_zoom = 8.0;
    for x in 48..56 {
        for y in 48..54 {
            command(&mut c, 6607, ((x * 64 + 32) << 14) | (y * 64 + 32))?;
            c.world_map
                .detailed_image(&mut c.scripts.source.cache, true, [256, 256], 0.8)?;
            assert!(c.world_map.renderer.as_ref().unwrap().cached_bytes() <= 36 << 20);
        }
    }
    let image = c
        .world_map
        .detailed_image(&mut c.scripts.source.cache, true, [256, 256], 0.8)?
        .unwrap();
    c.prepare_login()?;
    let again = c
        .world_map
        .detailed_image(&mut c.scripts.source.cache, true, [256, 256], 0.8)?
        .unwrap();
    assert!(std::sync::Arc::ptr_eq(&image, &again));
    println!("Detailed maps0/1/25 through content1400, fractional zoom, frame reuse, visible-interface motion, 48-region cache pressure and login retention passed; fixed/geography icons, labels and heatmap compositing active");
    Ok(())
}
