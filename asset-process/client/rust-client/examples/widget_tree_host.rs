use gielinor_client::{
    buffer::Reader, disk_cache::DiskCache, interface_script::execute, interface_state::Interfaces,
    item::Definitions, vm::Stacks, widget::Widget, Result,
};
use std::collections::HashMap;

fn nested_widget_opcodes_match_original_trace() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let items = Definitions {
        items: Default::default(),
    };
    let mut state = Interfaces::default();
    let mut child = Widget::empty(1, 0);
    child.parent = Some(0);
    state.install_group(0, vec![Widget::empty(0, 0), child])?;
    state.root = Some(0);
    let mut active = [None, None];
    let mut identities = HashMap::new();
    let mut input = Reader::new(include_bytes!("../tests/fixtures/widget-tree-jvm.bin"));
    for step in 0..input.u32()? {
        let n = input.u32()?;
        let op = input.u32()? as u16;
        let dot = input.u32()? != 0;
        let mut stack = Stacks::default();
        for _ in 2..n {
            stack.push_int(input.u32()? as i32)?;
        }
        assert!(
            execute(
                op,
                dot,
                &mut stack,
                &mut cache,
                &mut state,
                &mut active,
                [765, 503],
                &items
            )?,
            "step {step} opcode {op}"
        );
        let count = input.u32()?;
        let expected = (0..count)
            .map(|_| input.u32().map(|v| v as i32))
            .collect::<Result<Vec<_>>>()?;
        for value in expected.into_iter().rev() {
            assert_eq!(stack.pop_int()?, value, "step {step} opcode {op}");
        }
        assert!(stack.pop_int().is_err(), "stack residue step {step}");
        for (&id, r) in &state.runtime {
            identities.insert(id, (r.logical_id, r.child_index));
        }
        for slot in 0..2 {
            let selected = if input.u8()? != 0 {
                Some((input.u32()?, input.u32()? as i32))
            } else {
                None
            };
            assert_eq!(
                active[slot].map(|id| identities[&id]),
                selected,
                "active slot {slot} step {step} opcode {op}"
            );
        }
        let count = input.u32()? as usize;
        assert_eq!(state.dynamic.len(), count, "step {step} opcode {op}");
        for _ in 0..count {
            let index = input.u32()? as i32;
            let kind = input.u32()? as u8;
            let parent = input.u32()? as i32;
            let id = state.dynamic[&(0, index)];
            assert_eq!(
                (state.widgets[&id].kind, state.runtime[&id].tree_parent),
                (kind, parent),
                "step {step} slot {index}"
            );
            assert_eq!(
                state.widgets[&id].parent,
                Some(if parent == -1 {
                    0
                } else {
                    state.dynamic[&(0, parent)]
                })
            );
        }
        let layout = state.layout([765, 503])?;
        assert_eq!(
            layout.widgets.len(),
            state.widgets.len(),
            "unreachable nested widget step {step}"
        );
    }
    Ok(())
}

fn nested_layout_scroll_and_strict_creation() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let items = Definitions {
        items: Default::default(),
    };
    let mut state = Interfaces::default();
    let mut root = Widget::empty(0, 0);
    root.raw_size = [300, 200];
    state.install_group(0, vec![root])?;
    state.root = Some(0);
    let parent = state.create_child(0, 4, 0, true)?;
    state.widgets.get_mut(&parent).unwrap().raw_position = [30, 40];
    state.widgets.get_mut(&parent).unwrap().raw_size = [100, 80];
    state.widgets.get_mut(&parent).unwrap().scroll_size = [200, 160];
    state.runtime.get_mut(&parent).unwrap().scroll = [5, 7];
    let mut active = [Some(parent), None];
    let mut stack = Stacks::default();
    stack.push_int(4)?;
    stack.push_int(9)?;
    execute(
        106,
        false,
        &mut stack,
        &mut cache,
        &mut state,
        &mut active,
        [765, 503],
        &items,
    )?;
    let child = active[0].unwrap();
    state.widgets.get_mut(&child).unwrap().raw_position = [11, 13];
    state.widgets.get_mut(&child).unwrap().raw_size = [20, 10];
    let layout = state.layout([765, 503])?;
    assert_eq!(layout.widgets[&child].absolute, [36, 46]);
    assert_eq!(layout.widgets[&child].clip, [36, 46, 56, 56]);
    active[0] = Some(parent);
    let instance = state.runtime[&child].instance;
    stack.push_int(4)?;
    stack.push_int(9)?;
    assert!(execute(
        106,
        false,
        &mut stack,
        &mut cache,
        &mut state,
        &mut active,
        [765, 503],
        &items
    )
    .is_err());
    assert_eq!(state.runtime[&child].instance, instance);
    state.create_child(0, 4, 0, false)?;
    assert!(!state.widgets.contains_key(&child));
    assert_eq!(state.dynamic.len(), 1);
    Ok(())
}

fn nested_close_callbacks_follow_flat_array_order() -> Result<()> {
    use gielinor_client::vm::Value;
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let items = Definitions {
        items: Default::default(),
    };
    let mut state = Interfaces::default();
    state.install_group(0, vec![Widget::empty(0, 0)])?;
    state.root = Some(0);
    let parent = state.create_child(0, 7, 0, true)?;
    let mut active = [Some(parent), None];
    let mut stack = Stacks::default();
    stack.push_int(4)?;
    stack.push_int(2)?;
    execute(
        106,
        false,
        &mut stack,
        &mut cache,
        &mut state,
        &mut active,
        [765, 503],
        &items,
    )?;
    let child = active[0].unwrap();
    for (id, marker) in [(parent, 7), (child, 2)] {
        state
            .runtime
            .get_mut(&id)
            .unwrap()
            .listeners
            .insert(1423, (vec![Value::Int(marker)], None));
    }
    state.queue_lifecycle(1423)?;
    assert_eq!(state.next_event()?.unwrap().widget, child);
    assert_eq!(state.next_event()?.unwrap().widget, parent);
    assert!(state.next_event()?.is_none());
    Ok(())
}

fn main() -> Result<()> {
    nested_widget_opcodes_match_original_trace()?;
    nested_layout_scroll_and_strict_creation()?;
    nested_close_callbacks_follow_flat_array_order()?;
    println!("78 nested-widget Java transitions, layout, replacement and lifecycle order passed with actual cache");
    Ok(())
}
