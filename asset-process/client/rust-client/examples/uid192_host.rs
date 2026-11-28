use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, game_packet,
    protocol::Packet, uid192::State, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut c = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!("../tests/fixtures/uid192-jvm.bin"));
    assert_eq!(r.u32()?, 32);
    for _ in 0..32 {
        let payload = r.bytes(28)?.to_vec();
        let cached = r.u8()? != 0;
        r.u8()?;
        assert_eq!(r.u32()?, 28);
        let present = r.u8()? != 0;
        let expected = if present {
            Some(<[u8; 24]>::try_from(r.bytes(24)?).unwrap())
        } else {
            None
        };
        assert_eq!(r.u32()?, 24);
        let login = r.bytes(24)?;
        c.uid192 = State::default();
        if cached {
            c.uid192.receive(Some([119; 24]));
        }
        c.apply(
            game_packet::decode(&Packet {
                opcode: 113,
                payload,
            })?
            .expect("UID route"),
        )?;
        assert_eq!(c.uid192.cached(), expected.as_ref());
        assert_eq!(c.uid192.login_bytes(), login);
    }
    println!("Actual-cache ClientState applies32 original UID192 CRC/cache/login cases, including invalid CRC preserving prior state");
    Ok(())
}
