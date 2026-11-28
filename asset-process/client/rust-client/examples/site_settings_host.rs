use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, game_packet,
    protocol::Packet, writer::Writer, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut c = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!("../tests/fixtures/site-settings-jvm.bin"));
    assert_eq!(r.u32()?, 260);
    let mut resets = 0;
    for case in 0..260 {
        let n = r.u32()?;
        let packet = Packet {
            opcode: 67,
            payload: r.bytes(n as usize)?.to_vec(),
        };
        let failed = r.u8()? != 0;
        let n = r.u32()?;
        let units = (0..n).map(|_| r.u16()).collect::<Result<Vec<_>>>()?;
        let expected = String::from_utf16(&units).unwrap();
        let n = r.u32()?;
        let encoded = r.bytes(n as usize)?;
        c.site_settings = "previous".into();
        let update = game_packet::decode(&packet);
        assert_eq!(update.is_err(), failed, "case{case}");
        if !failed {
            c.apply(update?.unwrap())?;
        }
        assert_eq!(c.site_settings, expected, "case{case}");
        if [0, 128, 255, 257, 258, 259].contains(&case) {
            c.prepare_login()?;
            resets += 1;
            assert_eq!(c.site_settings, expected, "reset case{case}");
            verify_login(&c.site_settings, encoded)?;
        }
        let mut w = Writer::new();
        w.string(&c.site_settings)?;
        assert_eq!(w.as_slice(), encoded);
    }
    assert_eq!(r.remaining(), 0);
    assert_eq!(resets, 6);
    println!("260 original site settings applied through ClientState;6 actual login resets retain value; original login string bytes match");
    Ok(())
}

fn verify_login(value: &str, expected: &[u8]) -> Result<()> {
    use gielinor_client::{
        crypto::xtea,
        login::{Authentication, Credentials, Platform, Request},
        rsa::RsaPublicKey,
    };
    let key = RsaPublicKey::from_hex(&"f".repeat(256), 65537)?;
    let platform = Platform::default();
    let crcs = [0; 23];
    let seed = [1, 2, 3, 4];
    for reconnect in [false, true] {
        let request = Request {
            username: "fixture",
            credentials: if reconnect {
                Credentials::Reconnect {
                    previous_seed: [5, 6, 7, 8],
                }
            } else {
                Credentials::Login {
                    authentication: Authentication::Password,
                    secret: "test",
                    token: false,
                }
            },
            client_build: i32::MAX as u32,
            client_type: 1,
            platform_type: 0,
            resizable: false,
            low_detail: false,
            width: 765,
            height: 503,
            uuid: [0; 24],
            site_settings: value,
            affiliate: 0,
            deep_links: &[],
            platform: &platform,
            archive_crcs: &crcs,
        };
        let wire = request.encode(&key, seed, 17)?;
        let length = u16::from_be_bytes([wire[18], wire[19]]) as usize;
        let mut private = wire[20 + length..].to_vec();
        xtea(&mut private, seed, true);
        let mut r = Reader::new(&private);
        assert_eq!(r.string()?, "fixture");
        r.bytes(29)?;
        assert_eq!(r.bytes(expected.len())?, expected);
    }
    Ok(())
}
