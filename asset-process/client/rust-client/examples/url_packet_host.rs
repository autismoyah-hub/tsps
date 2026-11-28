use gielinor_client::{
    buffer::Reader, client_state::ClientState, crypto::Isaac, disk_cache::DiskCache, game_packet,
    protocol::Decoder, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut c = ClientState::load(cache)?;
    let mut r = Reader::new(include_bytes!("../tests/fixtures/url-packet-jvm.bin"));
    assert_eq!(r.u32()?, 32);
    let mut applied = 0;
    for case in 0..32 {
        let mut seed = [0; 4];
        for v in &mut seed {
            *v = r.u32()?;
        }
        let n = r.u32()? as usize;
        let wire = r.bytes(n)?;
        r.u32()?;
        let n = r.u32()?;
        r.bytes(n as usize)?;
        r.u32()?;
        let failed = r.u8()? != 0;
        let expected = if failed {
            None
        } else {
            let n = r.u32()?;
            let units = (0..n).map(|_| r.u16()).collect::<Result<Vec<_>>>()?;
            Some(String::from_utf16(&units).unwrap())
        };
        let mut decoder = Decoder::new(Some(Isaac::new(&seed)));
        decoder.feed(wire)?;
        let packet = decoder.next_server_packet()?.unwrap();
        let update = game_packet::decode(&packet);
        assert_eq!(update.is_err(), failed, "case{case}");
        if let Some(expected) = expected {
            c.apply(update?.unwrap())?;
            assert_eq!(c.remote.pending_urls.pop_front(), Some(expected));
            applied += 1;
        }
        assert!(c.remote.pending_urls.is_empty());
        assert_eq!(decoder.next_server_packet()?.unwrap().opcode, 83);
    }
    assert_eq!(r.remaining(), 0);
    assert_eq!(applied, 24);
    println!("32 original encrypted URL cases -> actual ClientState packet route;24 queued exact strings,8 malformed payloads rejected; following opcode preserved; no browser launch");
    Ok(())
}
