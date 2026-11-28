use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    crypto::Isaac,
    disk_cache::DiskCache,
    interface_state::Interfaces,
    protocol,
    script_runtime::CacheSource,
    vm::{Array, Object, Value},
    widget::Widget,
    Result,
};
use std::{cell::RefCell, rc::Rc};

fn main() -> Result<()> {
    let mut input = Reader::new(include_bytes!(
        "../tests/fixtures/widget-argument-cipher-order-jvm.bin"
    ));
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    client.scripts.source = CacheSource::new(DiskCache::open(
        "tests/fixtures/widget-argument-vm-cache",
        1 << 20,
    )?);
    let formats = [
        "", "i", "s", "is", "si", "sii", "ss", "x", "a", "b", "c", "d",
    ];
    let mut cipher = Some(Isaac::new(&[0; 4]));
    assert_eq!(input.u32()?, 384);
    for case in 0..384 {
        assert_eq!(input.u32()?, case / 48);
        let format = input.u32()? as usize;
        let present = input.u32()? != 0;
        let allowed = input.u32()? != 0;
        let result = input.u32()? as i32;
        let packets = input.u32()? as usize;
        let mut state = Interfaces::default();
        let mut widget = Widget::empty(0, 0);
        widget.flags = if allowed { 1 << 23 } else { 0 };
        state.install_group(0, if present { vec![widget] } else { vec![] })?;
        if present {
            state.runtime.get_mut(&0).unwrap().item = 777;
        }
        client.interfaces = state;
        client.outbound.clear();
        let mut args = vec![Value::Int(format as i32 + 1)];
        for (i, kind) in formats[format].chars().enumerate() {
            if kind == 'i' {
                args.push(Value::Int(if i == 0 { i32::MIN } else { -129 }));
                continue;
            }
            let object = match kind {
                'x' => Object::Null,
                'a' => Object::Array(Rc::new(RefCell::new(Array::Ints(vec![i32::MIN, -129])))),
                'b' => Object::Array(Rc::new(RefCell::new(Array::Strings(vec![
                    Some("A€😀".into()),
                    Some("B".into()),
                ])))),
                'c' => Object::Array(Rc::new(RefCell::new(Array::Strings(vec![None])))),
                'd' => Object::Array(Rc::new(RefCell::new(Array::Longs(vec![1, 2])))),
                _ => Object::String("A€😀".into()),
            };
            args.push(Value::Object(object));
        }
        let actual = client.run_listener(&args, &Default::default());
        assert_eq!(if actual.is_ok() { 1 } else { -1 }, result, "case {case}");
        client.outbound.push_back(protocol::Packet {
            opcode: 89,
            payload: Vec::new(),
        });
        assert_eq!(client.outbound.len(), packets);
        let mut written = 0;
        while let Some(event) = client.outbound.pop_event() {
            let opcode = match &event {
                gielinor_client::outbound::Event::Packet(packet) => Some(packet.opcode),
                _ => None,
            };
            if let Some(encoded) = gielinor_client::outbound::encode(event, &mut cipher)? {
                assert_eq!(opcode.unwrap(), input.u32()? as u16);
                let n = input.u32()? as usize;
                let expected = input.bytes(n)?;
                assert_eq!(encoded, expected, "case {case} encrypted bytes");
                written += 1;
            }
        }
        assert_eq!(written, packets);
    }
    assert_eq!(input.remaining(), 0);
    println!(
        "384 original argument operation/following keepalive cipher traces passed through ClientState VM host"
    );
    Ok(())
}
