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
        "tests/fixtures/widget-delete-vm-cache",
        1 << 20,
    )?);
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-delete-vm-expected.bin"
    ));
    let total = r.u32()?;
    let mut observations = 0;
    for case in 1..=total {
        client.interfaces = Default::default();
        client.active_widgets = [None; 2];
        let mut child = Widget::empty(1, 0);
        child.parent = Some(0);
        client
            .interfaces
            .install_group(0, vec![Widget::empty(0, 0), child])?;
        client.interfaces.root = Some(0);
        let failed = r.u8()? != 0;
        let result = client.run_listener(&[Value::Int(case as i32)], &Default::default());
        assert_eq!(result.is_err(), failed, "VM scenario {case}: {result:?}");
        if failed {
            assert!(result.unwrap_err().to_string().contains("delete widget"));
        }
        for index in 0..r.u32()? {
            let expected = r.u32()? as i32;
            assert_eq!(
                client.scripts.variables.player(index as u16),
                expected,
                "VM scenario {case} observation {index}"
            );
            observations += 1;
        }
        let count = r.u32()? as usize;
        assert_eq!(
            client.interfaces.dynamic.len(),
            count,
            "VM scenario {case} remaining child count"
        );
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
    }
    assert_eq!(r.remaining(), 0);
    println!("{total} actual VM retired-deletion scenarios, {observations} Java observations");
    Ok(())
}
