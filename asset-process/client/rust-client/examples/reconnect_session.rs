use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, inventory::Item, live_runtime::Connection,
    writer::Writer, Result,
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::{Duration, Instant},
};
fn handshake(socket: &mut TcpStream, opcode: u8) -> ([u32; 4], Vec<u8>) {
    socket
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut init = [0];
    socket.read_exact(&mut init).unwrap();
    assert_eq!(init, [14]);
    socket.write_all(&[0, 1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
    let mut header = [0; 3];
    socket.read_exact(&mut header).unwrap();
    assert_eq!(header[0], opcode);
    let mut body = vec![0; u16::from_be_bytes([header[1], header[2]]) as usize];
    socket.read_exact(&mut body).unwrap();
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    let private = num_bigint::BigInt::from(65537)
        .modinv(&(&modulus - 1))
        .unwrap();
    let len = u16::from_be_bytes([body[15], body[16]]) as usize;
    let plain = num_bigint::BigInt::from_signed_bytes_be(&body[17..17 + len])
        .modpow(&private, &modulus)
        .to_bytes_be()
        .1;
    assert_eq!(plain[0], 1);
    (
        std::array::from_fn(|i| {
            u32::from_be_bytes(plain[1 + i * 4..5 + i * 4].try_into().unwrap())
        }),
        plain,
    )
}
fn main() -> Result<()> {
    let cache = std::env::args().nth(1).expect("cache path");
    let full = std::env::args().nth(2).as_deref() == Some("full");
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    let key = "work/reconnect-fixture.key";
    std::fs::write(key, format!("Modulus: {}\n", modulus.to_str_radix(16)))?;
    let mut client = ClientState::load(DiskCache::open(cache, 64 << 20)?)?;
    client.initialize_players(&vec![0; 4608], 42)?;
    client.scripts.variables.set_player(100, 239);
    client
        .inventories
        .containers
        .insert(93, vec![Item { id: 1277, count: 1 }]);
    client
        .inventories
        .widgets
        .insert(9764864, vec![Item { id: 1279, count: 1 }]);
    assert!(client.reconnect_players(&vec![0; 4609], 42).is_err());
    assert!(!client.inventories.containers.is_empty());
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let server = thread::spawn(move || {
        let (mut first, _) = listener.accept().unwrap();
        let (seed, _) = handshake(&mut first, 16);
        let mut cipher = Some(gielinor_client::crypto::Isaac::new(
            &seed.map(|n| n.wrapping_add(50)),
        ));
        let mut w = Writer::new();
        w.bytes(&[2, 37, 1]);
        for byte in 0x23901234u32.to_be_bytes() {
            w.u8(byte.wrapping_add(cipher.as_mut().unwrap().next_word() as u8));
        }
        w.bytes(&[0, 0]);
        w.u16(42);
        w.u8(1);
        w.u64(1);
        w.u64(2);
        w.u64(3);
        let spec = gielinor_client::protocol239::SERVER_PACKETS
            .iter()
            .find(|s| s.opcode == 83)
            .unwrap();
        for _ in 0..3 {
            w.bytes(&gielinor_client::protocol::encode(spec, &[], &mut cipher).unwrap());
        }
        first.write_all(w.as_slice()).unwrap();
        drop(first);
        let (mut second, _) = listener.accept().unwrap();
        let (new_seed, plain) = handshake(&mut second, 18);
        assert_eq!(
            &plain[25..41],
            seed.into_iter()
                .flat_map(u32::to_be_bytes)
                .collect::<Vec<_>>()
        );
        assert_ne!(seed, new_seed);
        let mut cipher = Some(gielinor_client::crypto::Isaac::new(
            &new_seed.map(|n| n.wrapping_add(50)),
        ));
        let mut w = Writer::new();
        if full {
            w.bytes(&[2, 37, 0, 0, 0, 0, 0, 0, 0]);
            w.u16(43);
            w.u8(1);
            w.u64(1);
            w.u64(2);
            w.u64(3);
            for _ in 0..3 {
                w.bytes(&gielinor_client::protocol::encode(spec, &[], &mut cipher).unwrap());
            }
        } else {
            w.u8(15);
            w.u16(4608);
            w.bytes(&vec![0; 4608]);
        }
        second.write_all(w.as_slice()).unwrap();
        let mut window = [0; 6];
        second.read_exact(&mut window).unwrap();
        let mut decoder = gielinor_client::protocol::Decoder::new(Some(
            gielinor_client::crypto::Isaac::new(&new_seed),
        ));
        decoder.feed(&window).unwrap();
        let packet = decoder
            .next_packet(gielinor_client::protocol239::CLIENT_PACKETS)
            .unwrap()
            .unwrap();
        assert_eq!(packet.opcode, 10);
        assert_eq!(packet.payload, [1, 2, 253, 1, 247]);
        let logout = gielinor_client::protocol239::SERVER_PACKETS
            .iter()
            .find(|s| s.opcode == 58)
            .unwrap();
        second
            .write_all(&gielinor_client::protocol::encode(logout, &[2], &mut cipher).unwrap())
            .unwrap();
        drop(second);
        let (mut third, _) = listener.accept().unwrap();
        let (_, plain) = handshake(&mut third, 16);
        assert_eq!(plain[25], 0);
        assert_eq!(&plain[26..30], &0x23901234u32.to_be_bytes());
        assert!(plain.ends_with(b"fixture\0"));
        let message = b"Fixture login message\0Please retry\0Third line\0";
        let mut w = Writer::new();
        w.u8(29);
        w.u16(message.len() as u16);
        w.bytes(message);
        third.write_all(w.as_slice()).unwrap();
        drop(third);
        let (mut fourth, _) = listener.accept().unwrap();
        let (_, plain) = handshake(&mut fourth, 16);
        assert!(plain.ends_with(b"fixture\0"));
        fourth.write_all(&[56]).unwrap();
        drop(fourth);
        let (mut fifth, _) = listener.accept().unwrap();
        let (_, plain) = handshake(&mut fifth, 16);
        assert_eq!(plain[25], 1);
        assert_eq!(&plain[26..29], &12345u32.to_be_bytes()[1..]);
        assert!(plain.ends_with(b"fixture\0"));
        fifth.write_all(&[57]).unwrap();
        drop(fifth);
        let (mut fourth, _) = listener.accept().unwrap();
        let (seed, plain) = handshake(&mut fourth, 16);
        assert_eq!(plain[25], 3);
        assert_eq!(&plain[26..29], &654321u32.to_be_bytes()[1..]);
        assert!(plain.ends_with(b"fixture\0"));
        let mut w = Writer::new();
        w.bytes(&[2, 37, 0, 0, 0, 0, 0, 0, 0]);
        w.u16(43);
        w.u8(1);
        w.u64(1);
        w.u64(2);
        w.u64(3);
        let mut cipher = Some(gielinor_client::crypto::Isaac::new(
            &seed.map(|n| n.wrapping_add(50)),
        ));
        for _ in 0..3 {
            w.bytes(&gielinor_client::protocol::encode(spec, &[], &mut cipher).unwrap());
        }
        fourth.write_all(w.as_slice()).unwrap();
        let mut window = [0; 6];
        fourth.read_exact(&mut window).unwrap();
        let mut decoder = gielinor_client::protocol::Decoder::new(Some(
            gielinor_client::crypto::Isaac::new(&seed),
        ));
        decoder.feed(&window).unwrap();
        let packet = decoder
            .next_packet(gielinor_client::protocol239::CLIENT_PACKETS)
            .unwrap()
            .unwrap();
        assert_eq!(packet.opcode, 10);
        assert_eq!(packet.payload, [1, 2, 253, 1, 247]);
        fourth
            .write_all(&gielinor_client::protocol::encode(logout, &[1], &mut cipher).unwrap())
            .unwrap();
    });
    std::fs::write("work/reconnect-fixture.password", "fixture")?;
    let mut connection = Connection::connect(
        &mut client,
        &address.to_string(),
        "fixture",
        "work/reconnect-fixture.password",
        &key,
    )?;
    let deadline = Instant::now() + Duration::from_secs(90);
    let mut recovering = false;
    loop {
        assert!(Instant::now() < deadline, "native reconnect stalled");
        connection.poll(&mut client)?;
        recovering |= connection.is_reconnecting();
        if recovering && !connection.is_reconnecting() && client.inventories.containers.is_empty() {
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }
    assert!(connection.session.is_established());
    if full {
        assert!(client.player_state.is_none());
        assert_eq!(client.scripts.variables.player(100), 0);
        assert!(client.inventories.widgets.is_empty());
    } else {
        assert_eq!(client.player_state.as_ref().unwrap().local, 42);
        assert_eq!(client.scripts.variables.player(100), 239);
        assert_eq!(client.inventories.widgets[&9764864][0].id, 1279);
    }
    println!(
        "Reconnect response mode: {}",
        if full {
            "2 full reset"
        } else {
            "15 snapshot resume"
        }
    );
    while connection.logged_out().is_none() {
        assert!(Instant::now() < deadline);
        connection.poll(&mut client)?;
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(connection.logged_out(), Some(2));
    assert!(connection.session.is_closed());
    assert!(client.player_state.is_none());
    connection.poll(&mut client)?;
    assert!(!connection.is_reconnecting());
    let mut raster = gielinor_client::raster::Raster::new(765, 503, 1 << 20)?;
    connection.draw_title(&mut client, &mut raster)?;
    std::fs::write("work/reconnect-logout.bmp", raster.bmp()?)?;
    for attempt in 0..2 {
        for point in [[382, 301], [462, 291]] {
            client.input.begin_cycle();
            client.input.pointer = point;
            client.input.button(0, true);
            connection.poll(&mut client)?;
        }
        assert_eq!(
            connection.title.as_ref().unwrap().screen,
            gielinor_client::title_input::Screen::Credentials
        );
        client.input.begin_cycle();
        // Retrying preserves fields; clear them using the same keyboard path.
        for _ in 0..connection.title.as_ref().unwrap().username.len() {
            client.input.key_events.push((85, 0));
        }
        for c in "fixture".encode_utf16() {
            client.input.key_events.push((-1, c as i32));
        }
        client.input.key_events.push((80, 0));
        for _ in 0..20 {
            client.input.key_events.push((85, 0));
        }
        for c in "fixture".encode_utf16() {
            client.input.key_events.push((-1, c as i32));
        }
        connection.poll(&mut client)?;
        connection.draw_title(&mut client, &mut raster)?;
        std::fs::write("work/reconnect-credentials.bmp", raster.bmp()?)?;
        client.input.begin_cycle();
        client.input.pointer = [302, 321];
        client.input.button(0, true);
        let title_music_tasks = client.audio.music.pending_tasks();
        assert!(title_music_tasks > 0);
        connection.poll(&mut client)?;
        assert_eq!(client.audio.music.pending_tasks(), title_music_tasks);
        assert_eq!(client.scripts.variables.player(100), 0);
        assert!(client.inventories.widgets.is_empty());
        client.input.begin_cycle();
        while connection.logged_out().is_none() {
            assert!(Instant::now() < deadline, "title relogin stalled");
            connection.poll(&mut client)?;
            thread::sleep(Duration::from_millis(1));
        }
        if attempt == 0 {
            assert_eq!(
                connection.title.as_ref().unwrap().lines,
                ["Fixture login message", "Please retry", "Third line"]
            );
        } else {
            for (pin, trust) in [("012345", true), ("654321", false)] {
                assert_eq!(
                    connection.title.as_ref().unwrap().screen,
                    gielinor_client::title_input::Screen::Authenticator
                );
                if !trust {
                    client.input.begin_cycle();
                    client.input.pointer = [400, 280];
                    client.input.button(0, true);
                    connection.poll(&mut client)?;
                }
                client.input.begin_cycle();
                for c in pin.encode_utf16() {
                    client.input.character(c);
                }
                connection.poll(&mut client)?;
                assert_eq!(connection.title.as_ref().unwrap().trust_computer, trust);
                connection.draw_title(&mut client, &mut raster)?;
                std::fs::write("work/title-authenticator.bmp", raster.bmp()?)?;
                client.input.begin_cycle();
                client.input.pointer = [302, 321];
                client.input.button(0, true);
                connection.poll(&mut client)?;
                client.input.begin_cycle();
                while connection.logged_out().is_none() {
                    assert!(Instant::now() < deadline, "authenticator login stalled");
                    connection.poll(&mut client)?;
                    thread::sleep(Duration::from_millis(1));
                }
            }
            assert_eq!(connection.logged_out(), Some(1));
        }
    }
    server.join().unwrap();
    let mut refused = Connection::connect(
        &mut client,
        &address.to_string(),
        "fixture",
        "work/reconnect-fixture.password",
        &key,
    )?;
    assert_eq!(
        refused.title.as_ref().unwrap().screen,
        gielinor_client::title_input::Screen::Connecting
    );
    let refusal_deadline = Instant::now() + Duration::from_secs(10);
    while refused.logged_out().is_none() {
        assert!(Instant::now() < refusal_deadline);
        refused.poll(&mut client)?;
        thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        refused.title.as_ref().unwrap().screen,
        gielinor_client::title_input::Screen::Retry
    );
    assert_eq!(
        refused
            .title
            .as_ref()
            .unwrap()
            .lines
            .each_ref()
            .map(String::as_str),
        gielinor_client::login_response::response(-2, false)
            .lines
            .unwrap()
    );
    refused.draw_title(&mut client, &mut raster)?;
    std::fs::write("work/title-refused.bmp", raster.bmp()?)?;
    println!("Native reconnect/title: transport loss, opcode 18, snapshot reset, logout, credential editing, fresh password login, reset state, server message, retry, authenticator challenge/rejection/trust encoding, window status and initial refusal passed");
    Ok(())
}
