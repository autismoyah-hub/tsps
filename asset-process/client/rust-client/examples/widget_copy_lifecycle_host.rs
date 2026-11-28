use gielinor_client::{
    disk_cache::DiskCache,
    interface_script::execute,
    interface_state::Interfaces,
    item::Definitions,
    vm::{Stacks, Value},
    widget::Widget,
    Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let items = Definitions::load(&mut cache)?;
    let mut state = Interfaces::default();
    state.install_group(0, vec![Widget::empty(0, 0)])?;
    state.root = Some(0);
    let source = state.create_child(0, 0, 4, false)?;
    state
        .runtime
        .get_mut(&source)
        .unwrap()
        .listeners
        .insert(1423, (vec![Value::Int(77)], None));
    state.widgets.get_mut(&source).unwrap().text = "copy source".into();
    let mut active = [None, None];
    let mut stacks = Stacks::default();
    for n in [0, 0, 1] {
        stacks.push_int(n)?;
    }
    assert!(execute(
        105,
        false,
        &mut stacks,
        &mut cache,
        &mut state,
        &mut active,
        [765, 503],
        &items
    )?);
    let copied = active[0].unwrap();
    assert_ne!(
        state.runtime[&source].instance, state.runtime[&copied].instance,
        "copied widget must own a new identity"
    );
    assert_eq!(state.widgets[&copied].text, "copy source");
    state.queue_lifecycle(1423)?;
    let mut seen = vec![];
    while let Some(event) = state.next_event()? {
        seen.push((event.widget, event.context.widget));
    }
    assert_eq!(seen, vec![(source, Some((0, 0))), (copied, Some((0, 1)))]);
    state.delete_child(source)?;
    state.queue_lifecycle(1423)?;
    assert_eq!(state.next_event()?.unwrap().widget, copied);
    assert!(state.next_event()?.is_none());
    println!(
        "Copied widget identity, listener contexts and source deletion verified through opcode105"
    );
    let mut oracle = gielinor_client::buffer::Reader::new(include_bytes!(
        "../tests/fixtures/widget-copy-editor-jvm.bin"
    ));
    for _ in 0..oracle.u32()? {
        let sample = oracle.u32()?;
        let mut state = Interfaces::default();
        state.install_group(0, vec![Widget::empty(0, 0)])?;
        let source = state.create_child(0, 0, 12, false)?;
        let r = state.runtime.get_mut(&source).unwrap();
        r.listeners.insert(1423, (vec![Value::Int(77)], None));
        for opcode in 1436..=1439 {
            r.listeners
                .insert(opcode, (vec![Value::Int(sample as i32)], None));
        }
        let e = r.editor.as_mut().unwrap();
        e.text.focused = sample & 1 != 0;
        e.text.enabled = sample & 2 != 0;
        e.text.anchor = 7;
        e.text.cursor = 3;
        r.key_modes
            .as_mut()
            .unwrap()
            .set_key(65, (sample % 4) as i32);
        r.key_modes
            .as_mut()
            .unwrap()
            .set_char(65, ((sample + 1) % 4) as i32);
        let mut active = [None, None];
        let mut stacks = Stacks::default();
        for n in [0, 0, 1] {
            stacks.push_int(n)?;
        }
        execute(
            105,
            false,
            &mut stacks,
            &mut cache,
            &mut state,
            &mut active,
            [765, 503],
            &items,
        )?;
        let target = active[0].unwrap();
        let r = &state.runtime[&target];
        let e = r.editor.as_ref().unwrap();
        assert_eq!(
            r.instance != state.runtime[&source].instance,
            oracle.u8()? != 0
        );
        assert_eq!(e.text.focused, oracle.u8()? != 0, "focus case{sample}");
        assert_eq!(e.text.enabled, oracle.u8()? != 0, "enabled case{sample}");
        assert_eq!(e.text.anchor, oracle.u32()? as i32);
        assert_eq!(e.text.cursor, oracle.u32()? as i32);
        assert_eq!(r.key_modes.as_ref().unwrap().key(65) as u32, oracle.u32()?);
        assert_eq!(
            r.key_modes.as_ref().unwrap().character(65) as u32,
            oracle.u32()?
        );
        let mask = (1436..=1439).fold(0, |mask, op| {
            mask | ((state.listener(target, op).is_some() as u32) << (op - 1436))
        });
        assert_eq!(mask, oracle.u32()?);
        assert_eq!(state.listener(target, 1423).is_some(), oracle.u8()? != 0);
        assert_eq!(
            state.runtime[&source].editor.as_ref().unwrap().text.anchor,
            7
        );
    }
    assert_eq!(oracle.remaining(), 0);
    println!("16 original copied-editor resets, key modes and callback states verified through opcode105");
    Ok(())
}
