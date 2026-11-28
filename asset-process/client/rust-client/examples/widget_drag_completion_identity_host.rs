use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, interface_action,
    script_runtime::CacheSource, vm::Value, widget::Widget, Result,
};
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    client.canvas = [600, 600];
    client.scripts.source = CacheSource::new(DiskCache::open(
        "tests/fixtures/widget-drag-completion-cache",
        1 << 20,
    )?);
    let mut r = gielinor_client::buffer::Reader::new(include_bytes!(
        "../tests/fixtures/widget-drag-completion-identity-jvm.bin"
    ));
    for _ in 0..r.u32()? {
        let mode = r.u32()? as i32;
        let trace = (0..r.u32()?)
            .map(|_| r.u32().map(|v| v as i32))
            .collect::<Result<Vec<_>>>()?;
        assert_eq!(&trace[..3], &[1, 2, 1]);
        client.interfaces = Default::default();
        client.widget_drag = None;
        client.outbound.clear();
        let mut parent = Widget::empty(0, 0);
        parent.raw_size = [500, 500];
        client.interfaces.install_group(0, vec![parent])?;
        client.interfaces.root = Some(0);
        let source = client.interfaces.create_child(0, 0, 4, true)?;
        let target = client.interfaces.create_child(0, 1, 4, true)?;
        for (id, x, flags, item) in [(source, 20, 1 << 17, 100), (target, 40, 1 << 20, 101)] {
            let w = client.interfaces.widgets.get_mut(&id).unwrap();
            w.raw_position = [x, 20];
            w.raw_size = [50, 50];
            w.flags = flags;
            w.drag_zone = 0;
            w.drag_threshold = 0;
            client.interfaces.runtime.get_mut(&id).unwrap().item = item;
        }
        client
            .interfaces
            .runtime
            .get_mut(&source)
            .unwrap()
            .listeners
            .insert(1410, (vec![Value::Int(mode + 1)], None));
        client.input.pointer = [25, 25];
        client.input.buttons = 1;
        assert!(client.begin_widget_drag([25, 25], &[])?);
        client.input.pointer = [45, 25];
        client.input.buttons = 0;
        client.tick_widget_drag()?;
        assert_eq!(
            client.outbound.pop_front(),
            Some(interface_action::drag(
                (trace[3] as u32, trace[4], trace[5]),
                (trace[6] as u32, trace[7], trace[8])
            )),
            "completion mutation {mode}"
        );
        assert!(client.outbound.is_empty());
        assert!(client.widget_drag.is_none());
    }
    assert_eq!(r.remaining(), 0);
    println!("4 actual ClientState drag completion identity cases including item mutation and child deletion/replacement");
    Ok(())
}
