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
        "../tests/fixtures/widget-argument-capacity-jvm.bin"
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
    let lengths = [
        9983, 9984, 9985, 10000, 3, 9982, 9983, 9984, 32767, 32768, 9982, 9983, 9984, 1, 4992,
        4993, 1996, 1997,
    ];
    let mut cipher = Some(Isaac::new(&[0; 4]));
    assert_eq!(input.u32()?, 18);
    for case in 0..18 {
        assert_eq!(input.u32()?, 0);
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
        let script = match format {
            0..=4 | 14..=15 => 3,
            5..=9 | 16..=17 => 9,
            10..=12 => 10,
            _ => 11,
        };
        let object = match format {
            0..=3 => Object::String("A".repeat(lengths[format])),
            4 => Object::String("A\0B".into()),
            5..=9 => Object::Array(Rc::new(RefCell::new(Array::Ints(vec![0; lengths[format]])))),
            10..=12 => Object::Array(Rc::new(RefCell::new(Array::Strings(vec![
                Some(
                    String::new()
                );
                lengths[format]
            ])))),
            13 => Object::Array(Rc::new(RefCell::new(Array::Strings(vec![None])))),
            14..=15 => Object::String("😀".repeat(lengths[format])),
            _ => Object::Array(Rc::new(RefCell::new(Array::Ints(vec![
                i32::MIN;
                lengths[format]
            ])))),
        };
        let args = vec![Value::Int(script), Value::Object(object)];
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
        "18 original argument capacity boundaries and following encrypted keepalive match ClientState VM host"
    );
    Ok(())
}
