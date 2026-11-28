use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    vm::{Host, Stacks},
    widget::Widget,
    Result,
};
fn op(c: &mut ClientState, code: u16, args: &[i32]) -> Result<()> {
    let mut s = Stacks::default();
    s.push_int(239)?;
    for &v in args {
        s.push_int(v)?;
    }
    c.extended(code, false, &mut s)?;
    assert_eq!(s.ints(), &[239]);
    Ok(())
}
fn tick(c: &mut ClientState, cycle: i32) -> Result<Vec<u8>> {
    c.cycle = cycle;
    c.tick_interfaces(true)?;
    assert!(c.script_failures.is_empty(), "{:?}", c.script_failures);
    Ok(c.outbound
        .drain(..)
        .filter(|p| p.opcode == 47)
        .map(|p| p.payload[8])
        .collect())
}
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    c.canvas = [100, 100];
    let mut w = Widget::empty(0, 5);
    w.if3 = true;
    w.flags = 2 | (10 << 11);
    w.raw_size = [20, 20];
    w.data_text = "Test spell".into();
    w.spell_action = "Cast".into();
    c.interfaces.install_group(0, vec![w])?;
    c.interfaces.root = Some(0);
    c.active_widgets[0] = Some(0);
    op(&mut c, 2350, &[1, 35, 0, 0])?;
    op(&mut c, 2352, &[1, 3, 2, 0])?;
    c.input.key(35, true);
    assert_eq!(tick(&mut c, 10)?, vec![1]);
    c.input.begin_cycle();
    for cycle in 11..15 {
        assert!(tick(&mut c, cycle)?.is_empty());
    }
    assert_eq!(tick(&mut c, 15)?, vec![1]);
    op(&mut c, 3139, &[])?;
    assert!(tick(&mut c, 18)?.is_empty());
    op(&mut c, 3138, &[])?;
    assert_eq!(tick(&mut c, 19)?, vec![1]);
    c.input.key(35, false);
    c.input.begin_cycle();
    assert!(tick(&mut c, 20)?.is_empty());
    c.input.key(35, true);
    assert_eq!(tick(&mut c, 21)?, vec![1]);
    c.input.begin_cycle();
    op(&mut c, 2354, &[1, 0])?;
    assert!(tick(&mut c, 30)?.is_empty());
    c.input.key(35, false);
    c.input.begin_cycle();
    tick(&mut c, 31)?;
    c.input.key(35, true);
    assert_eq!(tick(&mut c, 32)?, vec![1]);
    c.input.key(35, false);
    c.input.begin_cycle();
    op(&mut c, 2351, &[36, 0, 0])?;
    c.input.key(36, true);
    tick(&mut c, 33)?;
    assert_eq!(c.selected_spell.as_ref().unwrap().logical, 0);
    c.input.lose_focus();
    c.input.begin_cycle();
    assert!(tick(&mut c, 34)?.is_empty());
    assert_eq!(
        c.interfaces.runtime[&0]
            .shortcuts
            .slots
            .as_ref()
            .unwrap()
            .next,
        Some([0; 11])
    );
    c.interfaces.runtime.get_mut(&0).unwrap().listeners.insert(
        1409,
        (
            vec![
                gielinor_client::vm::Value::Int(1),
                gielinor_client::vm::Value::Object(gielinor_client::vm::Object::String(
                    "event_opbase".into(),
                )),
            ],
            None,
        ),
    );
    c.scripts.source = gielinor_client::script_runtime::CacheSource::new(DiskCache::open(
        "work/widget-shortcut-cache",
        1 << 20,
    )?);
    c.input.key(35, true);
    assert_eq!(tick(&mut c, 40)?, vec![1]);
    assert_eq!(
        c.scripts.variables.player(0),
        0,
        "shortcut operation label must be empty"
    );
    assert_eq!(
        c.interfaces.runtime[&0]
            .shortcuts
            .slots
            .as_ref()
            .unwrap()
            .next
            .as_ref()
            .unwrap()[0],
        51,
        "listener changes repeat timing before scheduling"
    );
    c.interface_operation(0, 1, None)?;
    assert_eq!(
        c.scripts.variables.player(0),
        10,
        "normal menu label retained"
    );
    c.outbound.clear(); // The preceding explicit menu operation was already checked.
    c.input.begin_cycle();
    c.input.key(35, true);
    c.input.key(35, false);
    assert!(c.input.pressed.is_empty());
    assert_eq!(
        tick(&mut c, 60)?,
        vec![1],
        "a completed tap remains in the per-cycle key event stream"
    );
    println!("Shortcut host:2350/2352 binding, operation47 dispatch, held repeat/initial delay,3138/3139 scope, ignore-held/release reset, target selection and focus loss passed");
    Ok(())
}
