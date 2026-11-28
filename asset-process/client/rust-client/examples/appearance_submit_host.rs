use gielinor_client::{
    buffer::Reader, client_state::ClientState, crypto::Isaac, disk_cache::DiskCache,
    interface_state::Interfaces, player_model::Appearance, protocol, protocol239::CLIENT_PACKETS,
    widget::Widget, Result,
};

fn main() -> Result<()> {
    let mut input = Reader::new(include_bytes!(
        "../tests/fixtures/appearance-submit-wire-jvm.bin"
    ));
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    // An already initialized editor must submit without loading definition archives.
    client.scripts.source = gielinor_client::script_runtime::CacheSource::new(DiskCache::open(
        "tests/fixtures/event-script-cache",
        1 << 20,
    )?);
    for case in 0..input.u32()? {
        let mut appearance = Appearance::default();
        appearance.gender = input.u32()? as u8;
        for value in &mut appearance.equipment {
            *value = input.u32()?;
        }
        for value in &mut appearance.colors {
            *value = input.u32()? as u8;
        }
        let declared = input.u32()?;
        let enqueued = input.u32()? as usize;
        let length = input.u32()? as usize;
        let opcode_byte = input.u32()? as u8;
        let payload = input.bytes(length)?;
        assert_eq!(
            CLIENT_PACKETS
                .iter()
                .find(|spec| spec.opcode == 77)
                .unwrap()
                .size,
            declared as i32
        );
        assert_eq!(appearance.design_bytes(), payload, "case {case}");
        let encoded = protocol::encode_client(77, payload, &mut Some(Isaac::new(&[0; 4])))?;
        assert_eq!(encoded.len(), enqueued);
        assert_eq!(encoded[0], opcode_byte);
        assert_eq!(&encoded[1..], payload);
        for allowed in [false, true] {
            client.interfaces = Interfaces::default();
            let mut widget = Widget::empty(0, 4);
            widget.content = 326;
            widget.flags = if allowed { 2 } else { 0 };
            client.interfaces.install_group(0, vec![widget])?;
            client.appearance_editor = Some(appearance.clone());
            client.outbound.clear();
            assert_eq!(client.interface_operation(0, 1, None)?, allowed);
            assert_eq!(client.outbound.len(), if allowed { 2 } else { 1 });
            let packet = client.outbound.pop_front().unwrap();
            assert_eq!(packet.opcode, 77);
            assert_eq!(packet.payload, payload);
            assert_eq!(
                protocol::encode_client(
                    packet.opcode,
                    &packet.payload,
                    &mut Some(Isaac::new(&[0; 4]))
                )?,
                encoded
            );
            if allowed {
                assert_eq!(client.outbound.pop_front().unwrap().opcode, 47);
            }
        }
    }
    assert_eq!(input.remaining(), 0);
    println!("64 original appearance writer/cipher cases passed through128 ClientState content326 operations");
    Ok(())
}
