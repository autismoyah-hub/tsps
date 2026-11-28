use gielinor_client::{
    buffer::Reader, disk_cache::DiskCache, interface_script, interface_state::Interfaces,
    item::Definitions, vm::Stacks, widget::Widget, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let items = Definitions::load(&mut cache)?;
    let mut state = Interfaces::default();
    state.install_group(0, vec![Widget::empty(0, 4), Widget::empty(1, 12)])?;
    let mut active = [Some(0), Some(1)];
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/widget-model-flag-jvm.bin"
    ));
    for case in 0..r.u32()? {
        let value = r.u32()? as i32;
        let dot = r.u8()? != 0;
        assert_eq!(r.u32()?, 1);
        assert_eq!(r.u32()?, 0);
        let explicit = case % 3 == 0;
        let mut stack = Stacks::default();
        stack.push_int(value)?;
        if explicit {
            stack.push_int(dot as i32)?;
        }
        assert!(interface_script::execute(
            if explicit { 2127 } else { 1127 },
            dot,
            &mut stack,
            &mut cache,
            &mut state,
            &mut active,
            [320, 240],
            &items
        )?);
        assert!(stack.pop_int().is_err());
        for selected in [false, true] {
            assert_eq!(r.u32()?, 1);
            assert_eq!(r.u32()?, 1);
            let expected = r.u32()? as i32;
            if explicit {
                stack.push_int(selected as i32)?;
            }
            assert!(interface_script::execute(
                if explicit { 2614 } else { 1614 },
                selected,
                &mut stack,
                &mut cache,
                &mut state,
                &mut active,
                [320, 240],
                &items
            )?);
            assert_eq!(stack.pop_int()?, expected, "case {case}");
            assert!(stack.pop_int().is_err());
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("1024 original model flag transitions pass active/explicit dispatch");
    Ok(())
}
