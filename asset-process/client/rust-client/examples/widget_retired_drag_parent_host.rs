use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    vm::{Host, Stacks},
    widget::Widget,
    Result,
};
use std::collections::HashMap;
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    client.interfaces = Default::default();
    client.active_widgets = [None; 2];
    let mut child = Widget::empty(1, 0);
    child.parent = Some(0);
    client
        .interfaces
        .install_group(0, vec![Widget::empty(0, 0), child])?;
    client.interfaces.root = Some(0);
    let mut identities = HashMap::new();
    let mut selected_instances = [None; 2];
    let mut r = Reader::new(include_bytes!("../tests/fixtures/widget-retired-drag-parent-jvm.bin"));
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
        assert!(!failed);
        if op == 3109 {
            client.widget_drag = None;
        }
        if op == 9000 {
            client.interfaces.children.insert(
                987,
                gielinor_client::interface_packet::SubInterface {
                    parent: 987,
                    group: 0,
                    kind: 0,
                },
            );
            client.interfaces.close(987)?;
            let mut child = Widget::empty(1, 0);
            child.parent = Some(0);
            client
                .interfaces
                .install_group(0, vec![Widget::empty(0, 0), child])?;
        } else {
            client.extended(op, dot, &mut stack)?;
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
        let native_candidate = if op == 3109 {
            client
                .widget_drag
                .as_ref()
                .filter(|drag| {
                    client
                        .interfaces
                        .runtime
                        .get(&drag.widget)
                        .is_some_and(|r| r.instance == drag.instances[0])
                })
                .map(|drag| (drag.widget, drag.parent, drag.instances))
        } else {
            None
        };
        if op == 3109
            && client.widget_drag.as_ref().is_some_and(|drag| {
                [drag.widget, drag.parent]
                    .into_iter()
                    .zip(drag.instances)
                    .any(|(id, instance)| {
                        !client
                            .interfaces
                            .runtime
                            .get(&id)
                            .is_some_and(|r| r.instance == instance)
                    })
            })
        {
            assert!(client.tick_widget_drag()?.is_none());
            assert!(client.widget_drag.is_some());
            assert!(client.tick_widget_drag()?.is_none());
            assert!(client.widget_drag.is_none());
            assert!(client.outbound.is_empty());
        }
        if let Some((source, parent, instances)) = native_candidate {
            client.widget_drag = None;
            client.interfaces.widgets.get_mut(&0).unwrap().raw_size = [100, 100];
            let widget = client.interfaces.widgets.get_mut(&source).unwrap();
            widget.raw_position = [5, 5];
            widget.raw_size = [20, 20];
            assert!(
                client.begin_widget_drag([10, 10], &[])?,
                "native start step {step}"
            );
            let drag = client.widget_drag.as_ref().unwrap();
            assert_eq!(
                (drag.widget, drag.parent, drag.instances),
                (source, parent, instances),
                "native reference step {step}"
            );
            if !client
                .interfaces
                .runtime
                .get(&parent)
                .is_some_and(|r| r.instance == instances[1])
            {
                assert!(client.tick_widget_drag()?.is_none());
                assert!(client.widget_drag.is_some());
                assert!(client.tick_widget_drag()?.is_none());
                assert!(client.widget_drag.is_none());
                assert!(client.outbound.is_empty());
            }
            client.widget_drag = None;
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("{total} actual ClientState retained drag-parent transitions match Java");
    Ok(())
}
