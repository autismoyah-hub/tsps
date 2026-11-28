use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, vm::Stacks, widget::Widget,
    Result,
};
use std::collections::HashMap;
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    client.scripts.source = gielinor_client::script_runtime::CacheSource::new(DiskCache::open(
        "tests/fixtures/widget-drag-parent-lookup-vm-cache",
        1 << 20,
    )?);
    let mut script_id = 0;
    for (fixture, sparse) in [
        (
            include_bytes!("../tests/fixtures/widget-drag-parent-lookup-jvm.bin").as_slice(),
            false,
        ),
        (
            include_bytes!("../tests/fixtures/widget-drag-parent-sparse-jvm.bin").as_slice(),
            true,
        ),
    ] {
        client.interfaces = Default::default();
        client.active_widgets = [None; 2];
        let mut child = Widget::empty(if sparse { 2 } else { 1 }, 0);
        child.parent = Some(if sparse { 1 } else { 0 });
        client
            .interfaces
            .install_group(0, vec![Widget::empty(0, 0), child])?;
        client.interfaces.root = Some(0);
        let mut identities = HashMap::new();
        let mut selected_instances = [None; 2];
        let mut r = Reader::new(fixture);
        let total = r.u32()?;
        for step in 0..total {
            let n = r.u32()?;
            let op = r.u32()? as u16;
            let dot = r.u32()? != 0;
            let mut stack = Stacks::default();
            for _ in 2..n {
                stack.push_int(r.u32()? as i32)?;
            }
            let failed = r.u8()? != 0;
            script_id += 1;
            if op == 3109 {
                client.widget_drag = None;
            }
            stack = Stacks::default();
            let outcome = client.run_listener(
                &[gielinor_client::vm::Value::Int(script_id)],
                &Default::default(),
            );
            assert_eq!(outcome.is_err(), failed, "VM error step {step} opcode {op}");
            if !failed {
                outcome?;
            }
            if matches!(op, 200 | 201) {
                stack.push_int(client.scripts.variables.player(0))?;
            }
            if matches!(op, 100 | 200 | 201) {
                selected_instances[usize::from(dot)] = client.active_widgets[usize::from(dot)]
                    .map(|id| client.interfaces.runtime[&id].instance);
            }

            for (&id, runtime) in &client.interfaces.runtime {
                identities.insert(
                    id,
                    (runtime.logical_id, runtime.child_index, runtime.instance),
                );
            }
            if op == 3109 {
                let mut values = vec![client.widget_drag.is_some() as i32];
                if let Some(drag) = &client.widget_drag {
                    let identity = identities[&drag.widget];
                    assert_eq!(
                        Some(drag.instances[0]),
                        selected_instances[usize::from(dot)],
                        "step {step} source generation"
                    );
                    values.extend([
                        i32::from(Some(drag.widget) == client.active_widgets[usize::from(dot)]),
                        identity.0 as i32,
                        identity.1,
                        identities[&drag.parent].0 as i32,
                        drag.motion.offset[0],
                        drag.motion.offset[1],
                        drag.motion.age,
                        drag.motion.dragging as i32,
                        identities[&drag.parent].1,
                        client
                            .interfaces
                            .runtime
                            .get(&drag.parent)
                            .is_some_and(|r| r.instance == drag.instances[1])
                            as i32,
                    ]);
                }
                for value in values {
                    stack.push_int(value)?;
                }
            }
            let expected = (0..r.u32()?)
                .map(|_| r.u32().map(|v| v as i32))
                .collect::<Result<Vec<_>>>()?;
            let mut actual = Vec::new();
            while let Ok(value) = stack.pop_int() {
                actual.push(value);
            }
            actual.reverse();
            assert_eq!(actual, expected, "step {step} opcode {op}");
            for slot in 0..2 {
                let expected = if r.u8()? != 0 {
                    Some((r.u32()?, r.u32()? as i32))
                } else {
                    None
                };
                assert_eq!(
                    client.active_widgets[slot].map(|id| {
                        let i = identities[&id];
                        (i.0, i.1)
                    }),
                    expected
                );
            }
            let count = r.u32()? as usize;
            assert_eq!(client.interfaces.dynamic.len(), count);
            for _ in 0..count {
                let index = r.u32()? as i32;
                let kind = r.u32()? as u8;
                let parent = r.u32()? as i32;
                let bounds = [r.u32()? as i32, r.u32()? as i32];
                let id = client.interfaces.dynamic[&(0, index)];
                assert_eq!(
                    (
                        client.interfaces.widgets[&id].kind,
                        client.interfaces.runtime[&id].tree_parent
                    ),
                    (kind, parent)
                );
                assert_eq!(client.interfaces.runtime[&id].child_bounds, bounds);
            }
        }
        assert_eq!(r.remaining(), 0);
        println!("{total} actual VM parent-lookup transitions match Java");
    }
    Ok(())
}
