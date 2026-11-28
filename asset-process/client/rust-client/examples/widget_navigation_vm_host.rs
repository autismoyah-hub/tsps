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
        "tests/fixtures/widget-navigation-vm-cache",
        1 << 20,
    )?);
    client.interfaces = Default::default();
    client.active_widgets = [None; 2];
    let mut child = Widget::empty(1, 0);
    child.parent = Some(0);
    client
        .interfaces
        .install_group(0, vec![Widget::empty(0, 0), child])?;
    client.interfaces.root = Some(0);
    let executed = client.run_listener(&[Value::Int(1)], &Default::default())?;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-navigation-vm-expected.bin"
    ));
    let count = r.u32()?;
    for index in 0..count {
        assert_eq!(
            client.scripts.variables.player(index as u16),
            r.u32()? as i32,
            "VM observation {index}"
        );
    }
    assert_eq!(r.remaining(), 0);
    assert!(client.interfaces.dynamic.is_empty());
    assert_eq!(client.interfaces.widgets.len(), 2);
    println!("{executed} actual ClientState VM instructions; {count} Java widget-navigation observations");
    Ok(())
}
