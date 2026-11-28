use gielinor_client::{
    buffer::Reader,
    client_state::ClientState,
    disk_cache::DiskCache,
    interface_state::Interfaces,
    protocol,
    protocol239::CLIENT_PACKETS,
    script_runtime::CacheSource,
    vm::{Array, Object, Value},
    widget::Widget,
    Result,
};
use std::{cell::RefCell, rc::Rc};

fn main() -> Result<()> {
    let mut input = Reader::new(include_bytes!(
        "../tests/fixtures/widget-argument-operation-jvm.bin"
    ));
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5", 64 << 20,
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
    for case in 0..input.u32()? {
        let format = input.u32()? as usize;
        let present = input.u32()? != 0;
        let allowed = input.u32()? != 0;
        let result = input.u32()? as i32;
        let _int_len = input.u32()? as usize;
        let _object_len = input.u32()? as usize;
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
        assert_eq!(client.outbound.len(), packets);
        for packet in client.outbound.drain(..) {
            assert_eq!(packet.opcode, input.u32()? as u16);
            let n = input.u32()? as usize;
            let expected = input.bytes(n)?;
            let spec = CLIENT_PACKETS
                .iter()
                .find(|s| s.opcode as u16 == packet.opcode)
                .unwrap();
            let encoded = protocol::encode(spec, &packet.payload, &mut None)?;
            assert_eq!(&encoded[1..], expected, "case {case}");
        }
    }
    assert_eq!(input.remaining(), 0);
    println!(
        "48 original argument-bearing widget packet traces passed through ClientState VM host"
    );
    Ok(())
}
