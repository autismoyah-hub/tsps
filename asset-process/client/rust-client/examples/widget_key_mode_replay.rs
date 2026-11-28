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
    let mut r = Reader::new(include_bytes!("../tests/fixtures/widget-key-mode-jvm.bin"));
    for case in 0..r.u32()? {
        let op = r.u32()? as u16;
        let key = r.u32()? as i32;
        let mode = r.u32()? as i32;
        let dot = r.u8()? != 0;
        assert_eq!(r.u32()?, 1);
        assert_eq!(r.u32()?, 0);
        let explicit = case % 3 == 0;
        let mut stack = Stacks::default();
        stack.push_int(key)?;
        stack.push_int(mode)?;
        if explicit {
            stack.push_int(dot as i32)?;
        }
        assert!(interface_script::execute(
            op + if explicit { 1000 } else { 0 },
            dot,
            &mut stack,
            &mut cache,
            &mut state,
            &mut active,
            [320, 240],
            &items
        )?);
        assert!(stack.pop_int().is_err());
        for _ in 0..4 {
            let query = r.u32()? as u16;
            let key = r.u32()? as i32;
            assert_eq!(r.u32()?, 1);
            assert_eq!(r.u32()?, 1);
            let expected = r.u32()? as i32;
            stack.push_int(key)?;
            if explicit {
                stack.push_int(dot as i32)?;
            }
            assert!(interface_script::execute(
                query + if explicit { 1000 } else { 0 },
                dot,
                &mut stack,
                &mut cache,
                &mut state,
                &mut active,
                [320, 240],
                &items
            )?);
            assert_eq!(stack.pop_int()?, expected, "case {case} query {query}");
            assert!(stack.pop_int().is_err());
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("4096 original key-policy script transitions pass active/explicit dispatch and non-editor no-op behavior");
    Ok(())
}
