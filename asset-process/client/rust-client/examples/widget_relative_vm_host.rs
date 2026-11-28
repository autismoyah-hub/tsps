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
        "tests/fixtures/widget-relative-vm-cache",
        1 << 20,
    )?);
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-relative-vm-expected.bin"
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
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("widget navigation child array"));
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
    }
    assert_eq!(r.remaining(), 0);
    println!("{total} actual VM retired-navigation scenarios, {observations} Java observations");
    Ok(())
}
