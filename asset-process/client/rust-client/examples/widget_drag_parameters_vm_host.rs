use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, script_runtime::CacheSource,
    vm::Value, widget::Widget, Result,
};
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    client.scripts.source = CacheSource::new(DiskCache::open(
        "tests/fixtures/widget-drag-parameters-vm-cache",
        1 << 20,
    )?);
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-drag-parameters-vm-expected.bin"
    ));
    let total = r.u32()?;
    let mut observations = 0;
    for case in 1..=total {
        client.interfaces = Default::default();
        client.active_widgets = [None; 2];
        client.widget_drag = None;
        let mut child = Widget::empty(1, 0);
        child.parent = Some(0);
        client
            .interfaces
            .install_group(0, vec![Widget::empty(0, 0), child])?;
        client.interfaces.root = Some(0);
        assert_eq!(r.u8()?, 0);
        client.run_listener(&[Value::Int(case as i32)], &Default::default())?;
        let expected = (0..r.u32()?)
            .map(|_| r.u32().map(|n| n as i32))
            .collect::<Result<Vec<_>>>()?;
        observations += expected.len();
        let mut actual = vec![client.widget_drag.is_some() as i32];
        if let Some(drag) = &client.widget_drag {
            actual.extend([
                i32::from(client.active_widgets.contains(&Some(drag.widget))),
                client.interfaces.runtime[&drag.parent].logical_id as i32,
                drag.motion.offset[0],
                drag.motion.offset[1],
                drag.motion.age,
                drag.motion.dragging as i32,
                drag.motion.dead_zone,
                drag.motion.dead_time,
            ]);
            if let Some(runtime) = client.interfaces.runtime.get(&drag.widget) {
                assert_eq!(runtime.instance, drag.instances[0]);
            } else {
                assert!(!client
                    .interfaces
                    .runtime
                    .values()
                    .any(|runtime| runtime.instance == drag.instances[0]));
            }
            assert_eq!(
                client.interfaces.runtime[&drag.parent].instance,
                drag.instances[1]
            );
        }
        assert_eq!(actual, expected, "VM scenario {case}");
        let count = r.u32()? as usize;
        assert_eq!(client.interfaces.dynamic.len(), count);
        assert_eq!(client.interfaces.widgets.len(), count + 2);
        for _ in 0..count {
            let index = r.u32()? as i32;
            let kind = r.u32()? as u8;
            let parent = r.u32()? as i32;
            let bounds = [r.u32()? as i32, r.u32()? as i32];
            let id = client.interfaces.dynamic[&(0, index)];
            assert_eq!(
                (
                    client.interfaces.widgets[&id].kind,
                    client.interfaces.runtime[&id].tree_parent
                ),
                (kind, parent)
            );
            assert_eq!(client.interfaces.runtime[&id].child_bounds, bounds);
        }
        if client
            .widget_drag
            .as_ref()
            .is_some_and(|drag| !client.interfaces.runtime.contains_key(&drag.widget))
        {
            assert!(client.tick_widget_drag()?.is_none());
            assert!(client.widget_drag.is_some());
            assert!(client.tick_widget_drag()?.is_none());
            assert!(client.widget_drag.is_none());
            assert!(client.outbound.is_empty());
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("{total} actual VM retained-drag scenarios; {observations} Java observations");
    Ok(())
}
