use crate::{
    buffer::Reader, interface_state::Interfaces, vm::Value, widget::Widget,
    widget_layout::Geometry, Result,
};

#[test]
fn original_automatic_resize_flat_and_nested_traces() -> Result<()> {
    for (nested, bytes) in [
        (
            false,
            include_bytes!("../tests/fixtures/widget-automatic-resize-jvm.bin").as_slice(),
        ),
        (
            true,
            include_bytes!("../tests/fixtures/widget-automatic-tree-jvm.bin").as_slice(),
        ),
    ] {
        let mut input = Reader::new(bytes);
        for _ in 0..input.u32()? {
            let sample = input.u32()?;
            let notify = sample & 1 != 0;
            let listener = sample & 2 != 0;
            let mutate = sample & 4 != 0;
            let axis = sample / 8;
            let count = if nested { 3 } else { 2 };
            let mut state = Interfaces::default();
            state.install_group(
                0,
                (0..count)
                    .map(|id| {
                        let mut widget = Widget::empty(id, 0);
                        widget.raw_size = [
                            if axis == 1 || axis == 3 { 120 } else { 100 },
                            if axis >= 2 { 90 } else { 80 },
                        ];
                        if nested && id == 1 {
                            widget.parent = Some(0);
                        }
                        widget
                    })
                    .collect(),
            )?;
            state.root = Some(0);
            for id in 0..count {
                let runtime = state.runtime.get_mut(&id).unwrap();
                runtime.aligned.set(Some(Geometry {
                    position: [0, 0],
                    size: [100, 80],
                }));
                if listener {
                    runtime
                        .listeners
                        .insert(1427, (vec![Value::Int(10 + id as i32)], None));
                }
            }
            state.resize_canvas([765, 503], notify)?;
            state.resize_canvas([765, 503], notify)?;
            for id in 0..count {
                let expected = [input.u32()? as i32, input.u32()? as i32];
                assert_eq!(state.runtime[&id].aligned.get().unwrap().size, expected);
            }
            if mutate {
                for runtime in state.runtime.values_mut() {
                    runtime.listeners.insert(1427, (vec![Value::Int(99)], None));
                    let mut geometry = runtime.aligned.get().unwrap();
                    geometry.size[0] = 777;
                    runtime.aligned.set(Some(geometry));
                }
            }
            let queue = state.resize_events.borrow();
            assert_eq!(
                queue.len(),
                input.u32()? as usize,
                "nested {nested} sample {sample}"
            );
            for queued in queue.iter() {
                let marker = input.u32()? as i32;
                let id = input.u32()?;
                let width = input.u32()? as i32;
                assert!(matches!(queued.event.arguments[0], Value::Int(v) if v == marker));
                assert_eq!(queued.event.widget, id);
                assert_eq!(state.runtime[&id].aligned.get().unwrap().size[0], width);
                assert_eq!(queued.event.context.resize_depth, 0);
                assert!(queued.valid(&state));
            }
        }
        assert_eq!(input.remaining(), 0);
    }
    Ok(())
}

#[test]
fn script_realignment_suppresses_automatic_notifications() -> Result<()> {
    let mut state = Interfaces::default();
    let mut root = Widget::empty(0, 0);
    root.raw_size = [100, 80];
    let mut child = Widget::empty(1, 4);
    child.parent = Some(0);
    child.size_alignment = [1, 1];
    state.install_group(0, vec![root, child])?;
    for runtime in state.runtime.values_mut() {
        runtime.listeners.insert(1427, (vec![Value::Int(42)], None));
    }
    state.align_script_widget(0, [765, 503], true)?;
    assert_eq!(state.runtime[&1].aligned.get().unwrap().size, [100, 80]);
    assert!(state.resize_events.borrow().is_empty());
    Ok(())
}
