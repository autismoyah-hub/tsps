//! Isolated title-world selection, async list loading, sorting and fresh endpoint login.
use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection, raster::Raster,
    title_input::Screen, Result,
};
use std::{
    io::{Read, Write},
    net::TcpListener,
    time::{Duration, Instant},
};
fn server(listener: TcpListener, code: u8, count: usize) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for _ in 0..count {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(20)))
                .unwrap();
            let mut hello = [0];
            socket.read_exact(&mut hello).unwrap();
            assert_eq!(hello, [14]);
            socket.write_all(&[0, 1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
            let mut header = [0; 3];
            socket.read_exact(&mut header).unwrap();
            assert_eq!(header[0], 16);
            let mut body = vec![0; u16::from_be_bytes([header[1], header[2]]) as usize];
            socket.read_exact(&mut body).unwrap();
            assert!(body.len() > 100);
            socket.write_all(&[code]).unwrap();
        }
    })
}
fn wait(
    live: &mut Connection,
    client: &mut ClientState,
    condition: impl Fn(&Connection) -> bool,
) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        live.poll(client)?;
        if condition(live) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    panic!("world-selector replay timed out")
}
fn click(live: &mut Connection, client: &mut ClientState, point: [i32; 2]) -> Result<()> {
    client.input.begin_cycle();
    client.input.pointer = point;
    client.input.button(0, true);
    live.poll(client)?;
    client.input.button(0, false);
    client.input.begin_cycle();
    Ok(())
}
fn main() -> Result<()> {
    let first = TcpListener::bind("127.0.0.1:0")?;
    let address = first.local_addr()?;
    let second = TcpListener::bind(("127.0.0.2", address.port()))?;
    let first_worker = server(first, 12, 3);
    let second_worker = server(second, 3, 1);
    let root =
        std::path::PathBuf::from(format!("work/title-members-replay-{}", std::process::id()));
    std::fs::create_dir_all(&root)?;
    let key = root.join("fixture.key");
    let password = root.join("fixture.password");
    let list = root.join("worlds.bin");
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    std::fs::write(&key, format!("Modulus: {}\n", modulus.to_str_radix(16)))?;
    std::fs::write(&password, "fixture")?;
    let mut wire = gielinor_client::writer::Writer::new();
    wire.u32(0);
    wire.u16(1);
    wire.u16(301);
    wire.u32(0);
    wire.string("127.0.0.2")?;
    wire.string("Free fixture world")?;
    wire.u8(0);
    wire.u16(100);
    std::fs::write(&list, wire.into_vec())?;
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    client.data.worlds.set_source(Some(list.clone()));
    client.canvas = [765, 503];
    let mut live = Connection::connect(
        &mut client,
        &address.to_string(),
        "members_probe",
        password.to_str().unwrap(),
        key.to_str().unwrap(),
    )?;
    let members = |live: &Connection| {
        live.title
            .as_ref()
            .is_some_and(|s| s.screen == Screen::MembersRequired)
    };
    wait(&mut live, &mut client, members)?;
    assert_eq!(live.title.as_ref().unwrap().password, "fixture");
    click(&mut live, &mut client, [382, 326])?;
    assert_eq!(live.title.as_ref().unwrap().screen, Screen::Credentials);
    assert_eq!(live.title.as_ref().unwrap().password, "fixture");
    click(&mut live, &mut client, [302, 321])?;
    wait(&mut live, &mut client, members)?;
    click(&mut live, &mut client, [382, 276])?;
    let mut raster = Raster::new(765, 503, 1 << 20)?;
    live.draw_title(&mut client, &mut raster)?;
    assert!(live.title.as_ref().unwrap().world_list_loading);
    std::fs::write("work/title-members-loading.bmp", raster.bmp()?)?;
    wait(&mut live, &mut client, |live| live.world_selector.is_some())?;
    client.input.key(13, true);
    live.poll(&mut client)?;
    client.input.key(13, false);
    client.input.begin_cycle();
    assert!(live.world_selector.is_none());
    assert_eq!(live.title.as_ref().unwrap().screen, Screen::Credentials);
    assert_eq!(live.title.as_ref().unwrap().password, "fixture");
    click(&mut live, &mut client, [302, 321])?;
    wait(&mut live, &mut client, members)?;
    click(&mut live, &mut client, [382, 276])?;
    wait(&mut live, &mut client, |live| live.world_selector.is_some())?;
    client.input.pointer = [382, 262];
    live.draw_title(&mut client, &mut raster)?;
    assert_eq!(live.world_selector.as_ref().unwrap().hovered, Some(0));
    click(&mut live, &mut client, [382, 262])?;
    assert!(live.world_selector.is_none());
    assert_eq!(live.title.as_ref().unwrap().screen, Screen::Credentials);
    assert_eq!(live.title.as_ref().unwrap().password, "fixture");
    click(&mut live, &mut client, [302, 321])?;
    wait(&mut live, &mut client, |live| {
        live.title
            .as_ref()
            .is_some_and(|s| s.screen == Screen::InvalidCredentials)
    })?;
    assert_eq!(client.data.world, 301);
    assert!(!client.data.members);
    first_worker.join().unwrap();
    second_worker.join().unwrap();
    for path in [key, password, list] {
        std::fs::remove_file(path)?;
    }
    std::fs::remove_dir(root)?;
    println!("Response12 Back, async loading, selector Escape and selection preserve credentials; fresh login reaches selected host");
    Ok(())
}
