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
        "tests/fixtures/widget-initial-geometry-vm-cache",
        1 << 20,
    )?);
    client.canvas = [765, 503];
    let mut input = Reader::new(include_bytes!(
        "../tests/fixtures/widget-initial-geometry-jvm.bin"
    ));
    let total = input.u32()?;
    for case in 0..total {
        let root = input.u32()? != 0;
        let _dot = input.u32()? != 0;
        let parent = input.u32()? as i32;
        let mode = input.u32()? as i8;
        let _phase = input.u32()?;
        let expected = [
            input.u32()? as i32,
            input.u32()? as i32,
            input.u32()? as i32,
            input.u32()? as i32,
        ];
        client.interfaces = Default::default();
        client.active_widgets = [None, None];
        client.interfaces.install_group(
            0,
            (0..2)
                .map(|id| {
                    let mut w = Widget::empty(id, 4);
                    w.parent = if id == 1 && parent >= 0 {
                        Some(parent as u32)
                    } else {
                        None
                    };
                    w.raw_size = if id == 0 { [300, 200] } else { [10, 20] };
                    w.raw_position = [15, 25];
                    w.size_alignment = if id == 0 { [0, 0] } else { [mode, mode] };
                    w
                })
                .collect(),
        )?;
        client.interfaces.root = root.then_some(0);
        client.run_listener(&[Value::Int(case as i32 + 1)], &Default::default())?;
        let actual = std::array::from_fn::<_, 4, _>(|i| client.scripts.variables.player(i as u16));
        assert_eq!(actual, expected, "case{case}");
    }
    assert_eq!(input.remaining(), 0);
    println!(
        "{total} initial geometry VM scripts; {} Java observations",
        total * 4
    );
    Ok(())
}
