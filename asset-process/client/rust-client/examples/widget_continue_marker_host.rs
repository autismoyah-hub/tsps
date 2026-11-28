use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    vm::{Host, Stacks},
    widget::Widget,
    Result,
};
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
    let mut r = Reader::new(include_bytes!("../tests/fixtures/widget-continue-marker-jvm.bin"));
    let total = r.u32()?;
    for step in 0..total {
        let n = r.u32()?;
        let op = r.u32()? as u16;
        let dot = r.u32()? != 0;
        let mut stack = Stacks::default();
        for _ in 2..n {
            stack.push_int(r.u32()? as i32)?;
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
        } else if op == 9001 {
            client.interfaces.set_continue_widget(None);
        } else {
            client.extended(op, dot, &mut stack)?;
        }

        if op == 1121 || op == 2121 {
            let packet = client.outbound.pop_front().expect("continue packet");
            stack.push_int(packet.opcode as i32)?;
            for byte in packet.payload {
                stack.push_int(byte as i32)?;
            }
            assert!(client.outbound.is_empty());
        }
        let expected = (0..r.u32()?)
            .map(|_| r.u32().map(|v| v as i32))
            .collect::<Result<Vec<_>>>()?;
        for value in expected.into_iter().rev() {
            assert_eq!(stack.pop_int()?, value, "step {step} opcode {op}");
        }
        assert!(stack.pop_int().is_err());
        assert_eq!(
            client.interfaces.continue_widget.is_none(),
            r.u8()? != 0,
            "availability step {step}"
        );
        for id in 0..2 {
            assert_eq!(
                client.interfaces.is_continue_widget(id),
                r.u8()? != 0,
                "waiting id {id} step {step} opcode {op}"
            );
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("{total} actual ClientState continue-marker identity transitions match Java");
    Ok(())
}
