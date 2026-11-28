use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    script_runtime::CacheSource,
    vm::{Host, Stacks, Value},
    widget::Widget,
    Result,
};
use std::collections::HashMap;
fn roots(client: &mut ClientState) -> Result<()> {
    let mut child = Widget::empty(1, 0);
    child.parent = Some(0);
    client
        .interfaces
        .install_group(0, vec![Widget::empty(0, 0), child])?;
    client.interfaces.root = Some(0);
    Ok(())
}
fn query(client: &mut ClientState, op: u16, dot: bool, id: Option<u32>) -> Result<i32> {
    let mut stack = Stacks::default();
    if let Some(id) = id {
        stack.push_int(id as i32)?;
    }
    client.extended(op, dot, &mut stack)?;
    stack.pop_int()
}
fn main() -> Result<()> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    client.scripts.source =
        CacheSource::new(DiskCache::open("tests/fixtures/widget-geometry-vm-cache", 1 << 20)?);
    client.canvas = [765, 503];
    let mut script_id = 0;
    let mut observations = 0;
    for fixture in [
        include_bytes!("../tests/fixtures/widget-retired-geometry-jvm.bin").as_slice(),
        include_bytes!("../tests/fixtures/widget-geometry-opcode-jvm.bin").as_slice(),
        include_bytes!("../tests/fixtures/widget-static-geometry-jvm.bin").as_slice(),
    ] {
        client.interfaces = Default::default();
        client.active_widgets = [None; 2];
        roots(&mut client)?;
        let mut identities = HashMap::new();
        let mut reader = Reader::new(fixture);
        let total = reader.u32()?;
        for step in 0..total {
            script_id += 1;
            let n = reader.u32()?;
            let opcode = reader.u32()? as u16;
            let dot = reader.u32()? != 0;
            for _ in 2..n {
                reader.u32()?;
            }
            assert_eq!(reader.u8()?, 0);
            let mut actual = vec![];
            match opcode {
                9000 => {
                    client.interfaces.children.insert(
                        987,
                        gielinor_client::interface_packet::SubInterface {
                            parent: 987,
                            group: 0,
                            kind: 0,
                        },
                    );
                    client.interfaces.close(987)?;
                    roots(&mut client)?;
                }
                9001 | 9002 => {
                    let parent = client.interfaces.dynamic[&(0, 3)];
                    let child = client.interfaces.dynamic[&(0, 7)];
                    if opcode == 9001 {
                        client.interfaces.widgets.get_mut(&parent).unwrap().raw_size = [300, 200];
                        let widget = client.interfaces.widgets.get_mut(&child).unwrap();
                        widget.raw_size = [10, 0];
                        widget.size_alignment = [1, 0];
                    }
                    let layout = client.interfaces.layout(client.canvas)?;
                    let geometry = layout.widgets.get(&child).map(|p| p.geometry);
                    if opcode == 9001 {
                        actual.extend([
                            geometry.is_some() as i32,
                            geometry.map_or(-99, |g| g.size[0]),
                        ]);
                    } else {
                        actual.extend(geometry.unwrap().size);
                    }
                }
                9003 | 9004 => {
                    if opcode == 9003 {
                        client.interfaces.widgets.get_mut(&0).unwrap().raw_size = [300, 200];
                        let child = client.interfaces.widgets.get_mut(&1).unwrap();
                        child.raw_size = [10, 0];
                        child.size_alignment = [1, 0];
                        client.interfaces.layout(client.canvas)?;
                    }
                    actual.push(query(&mut client, 2502, dot, Some(0))?);
                    actual.push(query(&mut client, 2502, dot, Some(1))?);
                    actual.push(query(&mut client, 1502, dot, None)?);
                }
                _ => {
                    client.run_listener(&[Value::Int(script_id)], &Default::default())?;
                }
            }
            let expected = (0..reader.u32()?)
                .map(|_| reader.u32().map(|v| v as i32))
                .collect::<Result<Vec<_>>>()?;
            observations += expected.len();
            if opcode < 9000 {
                actual
                    .extend((0..expected.len()).map(|i| client.scripts.variables.player(i as u16)));
            }
            assert_eq!(
                actual, expected,
                "script {script_id} step {step} opcode {opcode}"
            );
            for (&id, r) in &client.interfaces.runtime {
                identities.insert(id, (r.logical_id, r.child_index));
            }
            for slot in 0..2 {
                let selected = if reader.u8()? != 0 {
                    Some((reader.u32()?, reader.u32()? as i32))
                } else {
                    None
                };
                assert_eq!(
                    client.active_widgets[slot].map(|id| identities[&id]),
                    selected
                );
            }
            let count = reader.u32()? as usize;
            assert_eq!(client.interfaces.dynamic.len(), count);
            for _ in 0..count {
                let index = reader.u32()? as i32;
                let kind = reader.u32()? as u8;
                let parent = reader.u32()? as i32;
                let bounds = [reader.u32()? as i32, reader.u32()? as i32];
                let id = client.interfaces.dynamic[&(0, index)];
                let runtime = &client.interfaces.runtime[&id];
                assert_eq!(
                    (
                        client.interfaces.widgets[&id].kind,
                        runtime.tree_parent,
                        runtime.child_bounds
                    ),
                    (kind, parent, bounds)
                );
            }
        }
        assert_eq!(reader.remaining(), 0);
    }
    println!("{script_id} actual VM geometry transitions; {observations} Java observations");
    Ok(())
}
