use gielinor_client::{
    buffer::Reader, disk_cache::DiskCache, interface_script::execute, interface_state::Interfaces,
    item::Definitions, vm::Stacks, widget::Widget, Result,
};
use std::collections::HashMap;

fn selected_widget_lifetime_matches_original_trace() -> Result<()> {
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
    let mut input = Reader::new(include_bytes!(
        "../tests/fixtures/widget-selected-lifetime-jvm.bin"
    ));
    let total = input.u32()?;
    for step in 0..total {
        let n = input.u32()?;
        let op = input.u32()? as u16;
        let dot = input.u32()? != 0;
        let mut stack = Stacks::default();
        for _ in 2..n {
            stack.push_int(input.u32()? as i32)?;
        }
        let handled = if op == 9000 {
            state.children.insert(
                987,
                gielinor_client::interface_packet::SubInterface {
                    parent: 987,
                    group: 0,
                    kind: 0,
                },
            );
            state.close(987)?;
            let mut child = Widget::empty(1, 0);
            child.parent = Some(0);
            state.install_group(0, vec![Widget::empty(0, 0), child])?;
            true
        } else {
            execute(
                op,
                dot,
                &mut stack,
                &mut cache,
                &mut state,
                &mut active,
                [765, 503],
                &items,
            )?
        };
        assert!(handled, "step {step} opcode {op}");
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

fn main() -> Result<()> {
    selected_widget_lifetime_matches_original_trace()
}
