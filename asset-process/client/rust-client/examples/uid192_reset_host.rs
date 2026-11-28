//! UID192 globals survive full login resets; use synthetic oracle values only.
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
    let mut client = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!("../tests/fixtures/uid192-jvm.bin"));
    assert_eq!(r.u32()?, 32);
    for case in 0..32 {
        let payload = r.bytes(28)?.to_vec();
        let cached = r.u8()? != 0;
        r.u8()?;
        assert_eq!(r.u32()?, 28);
        let expected = if r.u8()? != 0 {
            Some(<[u8; 24]>::try_from(r.bytes(24)?).unwrap())
        } else {
            None
        };
        assert_eq!(r.u32()?, 24);
        let login = r.bytes(24)?;
        client.uid192 = State::default();
        if cached {
            client.uid192.receive(Some([119; 24]));
        }
        client.apply(
            game_packet::decode(&Packet {
                opcode: 113,
                payload,
            })?
            .expect("UID route"),
        )?;
        for reset in 0..2 {
            client.prepare_login()?;
            assert!(
                client.uid192.cached() == expected.as_ref(),
                "UID cached state lost in case {case}, reset {reset}"
            );
            assert!(
                client.uid192.login_bytes() == login,
                "UID login output changed in case {case}, reset {reset}"
            );
        }
    }
    assert_eq!(r.remaining(), 0);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "rust-client-uid192-reset-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&root)?;
    let primary = root.join("synthetic-primary.dat");
    let alternate = root.join("synthetic-alternate.dat");
    std::fs::write(&primary, [vec![0x37; 24], vec![0x99]].concat())?;
    std::fs::write(&alternate, [0x55; 24])?;
    client.uid192 = State::with_path(primary.clone());
    client.prepare_login()?;
    assert!(client.uid192.cached().is_none());
    assert!(
        client.uid192.login_bytes() == [0x37; 24],
        "retained file fallback lost on reset"
    );
    // Connection::connect may configure storage again; the original open handle wins.
    client.uid192.configure_storage(alternate.clone());
    client.apply(game_packet::Update::Uid192(Some([0; 24])))?;
    client.prepare_login()?;
    assert!(
        client.uid192.login_bytes() == [0; 24],
        "cached zeros replaced by file fallback"
    );
    let bytes = std::fs::read(&primary)?;
    assert!(
        bytes.len() == 25 && bytes[..24] == [0; 24] && bytes[24] == 0x99,
        "original retained file was not updated"
    );
    assert!(
        std::fs::read(&alternate)? == [0x55; 24],
        "one-time storage configuration was lost"
    );
    client.apply(game_packet::Update::Uid192(None))?;
    client.prepare_login()?;
    assert!(
        client.uid192.login_bytes() == [0; 24],
        "invalid CRC or reset changed cached zeros"
    );
    client.apply(game_packet::Update::Uid192(Some([0xa5; 24])))?;
    client.prepare_login()?;
    assert!(client.uid192.login_bytes() == [0xa5; 24]);
    let bytes = std::fs::read(&primary)?;
    assert!(bytes.len() == 25 && bytes[..24] == [0xa5; 24] && bytes[24] == 0x99);
    // Only these two synthetic files and their newly created empty directory are removed.
    client.uid192 = State::default();
    std::fs::remove_file(primary)?;
    std::fs::remove_file(alternate)?;
    std::fs::remove_dir(root)?;
    println!("32 JVM UID expectations survive two Rust full-login resets; retained file fallback, cached zeros, persistence and one-time configuration passed");
    Ok(())
}
