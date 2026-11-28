use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, widget::Widget, Result,
};
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let mut oracle = Reader::new(include_bytes!("../tests/fixtures/widget-drag-refresh-jvm.bin"));
    for case in 0..oracle.u32()? {
        client.interfaces = Default::default();
        client.widget_drag = None;
        client.menu = None;
        let base = 65534u32 << 16;
        let mut parent = Widget::empty(base, 0);
        parent.if3 = true;
        parent.raw_size = [500, 400];
        let mut source = Widget::empty(base + 1, 4);
        source.if3 = true;
        source.parent = Some(base);
        source.raw_position = [20, 20];
        source.raw_size = [50, 50];
        source.drag_zone = if case == 1 { 100 } else { 0 };
        source.drag_threshold = if case == 2 { 100 } else { 0 };
        client
            .interfaces
            .install_group(65534, vec![parent, source])?;
        client.interfaces.root = Some(65534);
        client
            .interfaces
            .runtime
            .get_mut(&(base + 1))
            .unwrap()
            .drag_parent = Some(base);
        assert!(client.begin_widget_drag([25, 25], &[])?);
        let source = client.interfaces.widgets.get_mut(&(base + 1)).unwrap();
        if case == 0 || case == 4 {
            source.raw_position = [120, 20];
        } else if case == 1 {
            source.drag_zone = 0;
        } else if case == 2 {
            source.drag_threshold = 0;
        } else {
            source.hidden = true;
        }
        client.input.buttons = if case == 4 { 0 } else { 1 };
        client.input.pointer = if case == 0 || case == 4 {
            [125, 25]
        } else {
            [45, 25]
        };
        assert_eq!(oracle.u32()?, case);
        let origin = [oracle.u32()? as i32, oracle.u32()? as i32];
        let dragging = oracle.u8()? != 0;
        let active = oracle.u8()? != 0;
        let mouse = [oracle.u32()? as i32, oracle.u32()? as i32];
        let outcome = client.tick_widget_drag()?;
        assert_eq!(client.widget_drag.is_some(), active, "active case {case}");
        if let Some(drag) = client.widget_drag.as_ref() {
            assert!(outcome.is_none());
            assert_eq!(drag.motion.origin, origin, "origin case {case}");
            assert_eq!(drag.motion.dragging, dragging, "live threshold case {case}");
        } else {
            assert_eq!(
                outcome.expect("release default action").0,
                mouse,
                "release location case {case}"
            );
        }
    }
    println!("Actual ClientState: drag source moves and live dead-zone/dead-time mutations follow Java per-tick reads");
    Ok(())
}
