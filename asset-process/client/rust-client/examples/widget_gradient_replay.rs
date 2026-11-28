use gielinor_client::{
    buffer::Reader, crypto::crc32, disk_cache::DiskCache, interface_render::Renderer,
    interface_state::Interfaces, item::Definitions, raster::Raster, widget::Widget, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let items = Definitions::load(&mut cache)?;
    let mut renderer = Renderer::load(&mut cache, 0.8)?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-gradient-render-jvm.bin"
    ));
    for case in 0..r.u32()? {
        let mut w = Widget::empty(0, 3);
        w.fill_mode = r.u32()? as u8;
        w.transparency = r.u32()? as u8;
        w.transparency_bottom = r.u32()? as i32;
        w.if3 = false;
        w.raw_position = [20, 10];
        w.raw_size = [50, 30];
        for color in &mut w.colors {
            *color = r.u32()?;
        }
        let condition = r.u8()? != 0;
        let hovered = r.u8()? != 0;
        w.filled = r.u8()? != 0;
        let expected = r.u32()?;
        // A redirect can select a widget even when its normal hover color is zero.
        w.mouse_over = 0;
        let mut state = Interfaces::default();
        state.install_group(0, vec![w])?;
        state.root = Some(0);
        state.runtime.get_mut(&0).unwrap().condition = condition;
        let layout = state.layout([160, 96])?;
        state.tick_tooltip(&layout, if hovered { [30, 20] } else { [0, 0] }, false);
        let mut raster = Raster::new(160, 96, 160 * 96)?;
        raster.clear(0xff302b25);
        let report = renderer.draw(&mut cache, &items, &state, &layout, &mut raster)?;
        assert!(report.pending.is_empty());
        let pixels: Vec<_> = raster
            .pixels()
            .iter()
            .flat_map(|p| p.to_be_bytes())
            .collect();
        assert_eq!(crc32(&pixels), expected, "case {case}");
    }
    assert_eq!(r.remaining(), 0);
    println!("4096 original gradient/solid/outline widget rasters pass actual renderer");
    let mut state = Interfaces::default();
    state.install_group(0, vec![Widget::empty(0, 3)])?;
    let mut active = [Some(0), Some(0)];
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-gradient-script-jvm.bin"
    ));
    for case in 0..r.u32()? {
        let opcode = r.u32()? as u16;
        let value = r.u32()? as i32;
        let dot = r.u8()? != 0;
        assert_eq!(r.u32()?, 1);
        assert_eq!(r.u32()?, 0);
        let expected = [r.u32()? as i32, r.u32()? as i32, r.u32()? as i32];
        let explicit = case % 2 == 0;
        let mut stack = gielinor_client::vm::Stacks::default();
        stack.push_int(value)?;
        if explicit {
            stack.push_int(0)?;
        }
        assert!(gielinor_client::interface_script::execute(
            opcode + if explicit { 1000 } else { 0 },
            dot,
            &mut stack,
            &mut cache,
            &mut state,
            &mut active,
            [160, 96],
            &items
        )?);
        assert!(stack.pop_int().is_err());
        for (query, expected) in [1612, 1610, 1613].into_iter().zip(expected) {
            if explicit {
                stack.push_int(0)?;
            }
            assert!(gielinor_client::interface_script::execute(
                query + if explicit { 1000 } else { 0 },
                dot,
                &mut stack,
                &mut cache,
                &mut state,
                &mut active,
                [160, 96],
                &items
            )?);
            assert_eq!(
                stack.pop_int()?,
                expected,
                "script case {case} query {query}"
            );
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("4096 original gradient script updates and queries pass explicit and active-widget dispatch");
    Ok(())
}
