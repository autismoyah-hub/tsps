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
fn server(listener: TcpListener) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
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
        socket.write_all(&[3]).unwrap();
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
    let proxy = std::env::args().any(|arg| arg == "--proxy");
    let redirect = proxy || std::env::args().any(|arg| arg == "--redirect");
    let http = redirect || std::env::args().any(|arg| arg == "--http");
    let first = TcpListener::bind("127.0.0.1:0")?;
    let address = first.local_addr()?;
    let second = TcpListener::bind(("127.0.0.2", address.port()))?;
    let first_worker = server(first);
    let second_worker = server(second);
    let root =
        std::path::PathBuf::from(format!("work/world-selector-replay-{}", std::process::id()));
    std::fs::create_dir_all(&root)?;
    let key = root.join("fixture.key");
    let password = root.join("fixture.password");
    let list = root.join("worlds.bin");
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    std::fs::write(&key, format!("Modulus: {}\n", modulus.to_str_radix(16)))?;
    std::fs::write(&password, "fixture")?;
    let mut wire = vec![0, 0, 0, 0, 0, 3];
    for (id, population, flags) in [(301u16, -1i16, 0u32), (302, 1981, 0), (303, 123, 1)] {
        wire.extend(id.to_be_bytes());
        wire.extend(flags.to_be_bytes());
        wire.extend(b"127.0.0.2\0Fixture world\0");
        wire.push(0);
        wire.extend(population.to_be_bytes());
    }
    std::fs::write(&list, &wire)?;
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let http_worker = if http {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        client.data.worlds.set_url(Some(format!(
            "http://{}/worlds.bin",
            listener.local_addr()?
        )));
        let proxy_listener = if proxy {
            Some(TcpListener::bind("127.0.0.1:0")?)
        } else {
            None
        };
        let proxy_address = proxy_listener.as_ref().map(|p| p.local_addr().unwrap());
        let origin_address = listener.local_addr()?;
        let file = list.clone();
        Some(std::thread::spawn(move || {
            for request in 0..if redirect { 6 } else { 3 } {
                let endpoint = if proxy && request % 2 == 1 {
                    proxy_listener.as_ref().unwrap()
                } else {
                    &listener
                };
                let (mut socket, _) = endpoint.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(10)))
                    .unwrap();
                let mut header = Vec::new();
                let mut byte = [0];
                while !header.ends_with(b"\r\n\r\n") {
                    socket.read_exact(&mut byte).unwrap();
                    header.push(byte[0]);
                    assert!(header.len() < 16384);
                }
                if redirect {
                    let header = String::from_utf8(header).unwrap();
                    if proxy {
                        assert!(
                            header.lines().any(|line| line
                                .eq_ignore_ascii_case(&format!("Host: {origin_address}"))),
                            "original Host survives endpoint routing"
                        );
                    }
                    if request % 2 == 0 {
                        assert!(header.starts_with("GET /worlds.bin HTTP/1.1\r\n"));
                        if let Some(address) = proxy_address {
                            write!(socket,"HTTP/1.1 305 Use Proxy\r\nLocation: http://{address}/ignored\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                        } else {
                            socket.write_all(b"HTTP/1.1 302 Found\r\nLocation: url:http:./redirect/../next.bin\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                        }
                        continue;
                    }
                    assert!(header.starts_with(if proxy {
                        "GET /worlds.bin HTTP/1.1\r\n"
                    } else {
                        "GET /next.bin HTTP/1.1\r\n"
                    }));
                }
                let bytes = std::fs::read(&file).unwrap();
                write!(
                    socket,
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    bytes.len()
                )
                .unwrap();
                socket.write_all(&bytes).unwrap();
            }
        }))
    } else {
        client.data.worlds.set_source(Some(list.clone()));
        None
    };
    client.canvas = [765, 503];
    let mut live = Connection::connect(
        &mut client,
        &address.to_string(),
        "world_selector_probe",
        password.to_str().unwrap(),
        key.to_str().unwrap(),
    )?;
    wait(&mut live, &mut client, |live| {
        live.title
            .as_ref()
            .is_some_and(|s| s.screen == Screen::InvalidCredentials)
    })?;
    click(&mut live, &mut client, [382, 276])?;
    assert_eq!(live.title.as_ref().unwrap().screen, Screen::Welcome);
    client.data.world_flags = 33555460;
    let saved = "world_selector_probe".to_owned();
    client.preferences.saved.remembered_username = Some(saved.clone());
    live.title.as_mut().unwrap().remember = true;
    click(&mut live, &mut client, [462, 291])?;
    assert_eq!(live.title.as_ref().unwrap().screen, Screen::WorldWarning);
    assert!(live.title.as_ref().unwrap().lines[0].contains("Beta"));
    let mut warning = Raster::new(765, 503, 1 << 20)?;
    live.draw_title(&mut client, &mut warning)?;
    std::fs::write("work/world-warning-title.bmp", warning.bmp()?)?;
    click(&mut live, &mut client, [462, 321])?;
    assert_eq!(live.title.as_ref().unwrap().screen, Screen::Welcome);
    assert_eq!(live.title.as_ref().unwrap().username, saved);
    assert!(live.title.as_ref().unwrap().password.is_empty());
    assert_eq!(
        client.preferences.saved.remembered_username.as_deref(),
        Some(saved.as_str())
    );
    click(&mut live, &mut client, [462, 291])?;
    assert_eq!(live.title.as_ref().unwrap().screen, Screen::WorldWarning);
    click(&mut live, &mut client, [302, 321])?;
    assert_eq!(live.title.as_ref().unwrap().screen, Screen::Credentials);
    live.title.as_mut().unwrap().password = "fixture".into();
    let mut raster = Raster::new(765, 503, 1 << 20)?;
    live.draw_title(&mut client, &mut raster)?;
    std::fs::write("work/world-selector-title.bmp", raster.bmp()?)?;
    click(&mut live, &mut client, [55, 480])?;
    wait(&mut live, &mut client, |live| live.world_selector.is_some())?;
    client.input.key(13, true);
    live.poll(&mut client)?;
    client.input.key(13, false);
    client.input.begin_cycle();
    assert!(live.world_selector.is_none());
    std::fs::write(&list, [0u8])?;
    click(&mut live, &mut client, [55, 480])?;
    let deadline = Instant::now() + Duration::from_secs(10);
    while client.data.worlds.loading() && Instant::now() < deadline {
        live.poll(&mut client)?;
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(!client.data.worlds.loading());
    assert!(live.world_selector.is_none());
    assert_eq!(client.data.worlds.worlds.len(), 3);
    std::fs::write(&list, &wire)?;
    click(&mut live, &mut client, [55, 480])?;
    wait(&mut live, &mut client, |live| live.world_selector.is_some())?;
    live.draw_title(&mut client, &mut raster)?;
    click(&mut live, &mut client, [280, 10])?;
    assert_eq!(
        client
            .data
            .worlds
            .worlds
            .iter()
            .map(|w| w.id)
            .collect::<Vec<_>>(),
        [303, 302, 301]
    );
    click(&mut live, &mut client, [295, 10])?;
    assert_eq!(
        client
            .data
            .worlds
            .worlds
            .iter()
            .map(|w| w.id)
            .collect::<Vec<_>>(),
        [301, 302, 303]
    );
    for y in [238, 262] {
        client.input.pointer = [382, y];
        live.draw_title(&mut client, &mut raster)?;
        assert_eq!(live.world_selector.as_ref().unwrap().hovered, None);
        click(&mut live, &mut client, [382, y])?;
        assert!(live.world_selector.is_some());
    }
    client.input.pointer = [382, 286];
    live.draw_title(&mut client, &mut raster)?;
    assert_eq!(live.world_selector.as_ref().unwrap().hovered, Some(2));
    std::fs::write("work/world-selector-hover.bmp", raster.bmp()?)?;
    click(&mut live, &mut client, [382, 286])?;
    assert!(live.world_selector.is_none());
    assert_eq!(client.data.world, 303);
    assert_eq!(client.data.world_flags, 1);
    assert!(client.data.members);
    click(&mut live, &mut client, [302, 321])?;
    wait(&mut live, &mut client, |live| {
        live.title
            .as_ref()
            .is_some_and(|s| s.screen == Screen::InvalidCredentials)
    })?;
    assert_eq!(client.data.world, 303);
    assert_eq!(client.data.world_flags, 1);
    assert!(
        client.data.members,
        "selected membership must survive prepare_login"
    );
    assert_eq!(client.data.worlds.worlds.len(), 3);
    if let Some(worker) = http_worker {
        worker.join().unwrap();
    }
    first_worker.join().unwrap();
    second_worker.join().unwrap();
    for path in [key, password, list] {
        std::fs::remove_file(path)?;
    }
    std::fs::remove_dir(root)?;
    println!("World-list load (HTTP={http}, redirect={redirect}, proxy={proxy}), title selector, header sort, offline/full exclusion, host change, retained custom port and login-reset membership passed");
    Ok(())
}
