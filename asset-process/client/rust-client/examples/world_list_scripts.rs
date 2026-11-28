//! World-list script dispatch and full-login reset integration against the local cache.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    vm::{Host, Stacks},
    writer::Writer,
    Result,
};
use std::time::{Duration, Instant};
fn query(client: &mut ClientState, op: u16, args: &[i32]) -> Result<Stacks> {
    let mut s = Stacks::default();
    for &v in args {
        s.push_int(v)?;
    }
    client.extended(op, false, &mut s)?;
    Ok(s)
}
fn main() -> Result<()> {
    let mut wire = Writer::new();
    wire.u32(239);
    wire.u16(2);
    for id in [302, 301] {
        wire.u16(id);
        wire.u32(1);
        wire.string("127.0.0.1")?;
        wire.string("Fixture world")?;
        wire.u8(0);
        wire.u16(50);
    }
    let path = std::path::PathBuf::from("work/world-list-script-fixture.bin");
    std::fs::write(&path, wire.as_slice())?;
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    assert_eq!(query(&mut client, 6519, &[])?.ints(), [1]);
    assert_eq!(query(&mut client, 6527, &[])?.ints(), [0]);
    assert_eq!(query(&mut client, 6531, &[])?.ints(), [239, 1]);
    client.data.worlds.set_source(Some(path.clone()));
    assert_eq!(query(&mut client, 6500, &[])?.ints(), [0]);
    let deadline = Instant::now() + Duration::from_secs(5);
    while query(&mut client, 6500, &[])?.ints() == [0] {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
    std::fs::remove_file(path)?;
    assert_eq!(query(&mut client, 6501, &[])?.ints(), [301, 1, 0, 50]);
    client.prepare_login()?;
    assert_eq!(
        query(&mut client, 6502, &[])?.ints(),
        [302, 1, 0, 50],
        "list and iterator must survive full login reset"
    );
    query(&mut client, 6507, &[0, 1, -1, 0])?;
    assert_eq!(query(&mut client, 6501, &[])?.ints(), [302, 1, 0, 50]);
    assert_eq!(query(&mut client, 6506, &[301])?.ints(), [301, 1, 0, 50]);
    assert_eq!(query(&mut client, 6511, &[99])?.ints(), [-1, 0, 0, 0]);
    println!("World-list fetch, sort, iteration, lookup and login-reset preservation passed through ClientState script host");
    Ok(())
}
