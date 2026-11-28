use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection, writer::Writer,
    Result,
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
    let root =
        std::path::PathBuf::from(format!("work/receive-timeout-host-{}", std::process::id()));
    std::fs::create_dir_all(&root)?;
    let key = root.join("fixture.key");
    let password = root.join("fixture.password");
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    std::fs::write(&key, format!("Modulus: {}\n", modulus.to_str_radix(16)))?;
    std::fs::write(&password, "fixture")?;
    for case in 0..2 {
        let mut cache = DiskCache::open_remote(
            "work/kotlin-world/.data/cache/js5",
            64 << 20,
            "127.0.0.1:43617".parse().unwrap(),
        )?;
        cache.wait_for_recovery(true);
        let mut client = ClientState::load(cache)?;
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let (seed, _) = handshake(&mut socket, 16);
            let mut cipher = Some(gielinor_client::crypto::Isaac::new(
                &seed.map(|n| n.wrapping_add(50)),
            ));
            let mut w = Writer::new();
            w.bytes(&[2, 37, 0, 0, 0, 0, 0, 0, 0]);
            w.u16(42);
            w.u8(1);
            w.u64(1);
            w.u64(2);
            w.u64(3);
            let tick = gielinor_client::protocol239::SERVER_PACKETS
                .iter()
                .find(|s| s.opcode == 83)
                .unwrap();
            for _ in 0..3 {
                w.bytes(&gielinor_client::protocol::encode(tick, &[], &mut cipher).unwrap());
            }
            socket.write_all(w.as_slice()).unwrap();
            let mut scratch = [0; 512];
            loop {
                match socket.read(&mut scratch) {
                    Ok(0) => break,
                    Ok(_) => (),
                    Err(e)
                        if matches!(
                            e.kind(),
                            std::io::ErrorKind::ConnectionReset
                                | std::io::ErrorKind::ConnectionAborted
                        ) =>
                    {
                        break
                    }
                    Err(e) => panic!("idle fixture socket: {e}"),
                }
            }
            if case == 0 {
                let (mut resumed, _) = listener.accept().unwrap();
                handshake(&mut resumed, 18);
                resumed.write_all(&[3]).unwrap();
            }
        });
        let mut connection = Connection::connect(
            &mut client,
            &address.to_string(),
            "fixture",
            password.to_str().unwrap(),
            key.to_str().unwrap(),
        )?;
        let deadline = Instant::now() + Duration::from_secs(45);
        while client.server_tick < 3 {
            assert!(Instant::now() < deadline);
            connection.poll_elapsed(&mut client, 0)?;
            thread::sleep(Duration::from_millis(1));
        }
        assert!(connection.session.is_established());
        connection.poll_elapsed(&mut client, 5000)?; // World loading is outside aft's game-state30 deadline.
        assert!(connection.session.is_established());
        assert!(connection.logged_out().is_none());
        let scene = gielinor_client::scene::Scene::empty_sized(0, 0, 1, 1)?;
        let mesh = gielinor_client::world_mesh::WorldMesh::build(
            &scene,
            &mut client.scripts.source.cache,
            0,
        )?;
        connection.world = Some((scene, mesh));
        connection.poll_elapsed(&mut client, 750)?;
        assert!(
            connection.session.is_established(),
            "case{case} expired at750"
        );
        connection.poll_elapsed(&mut client, 0)?;
        assert!(
            connection.session.is_established(),
            "zero-tick poll aged deadline"
        );
        client.logout_timer = if case == 1 { 100 } else { 0 };
        connection.poll_elapsed(&mut client, 1)?;
        if case == 0 {
            assert!(connection.is_reconnecting());
            while connection.is_reconnecting() {
                assert!(Instant::now() < deadline);
                connection.poll_elapsed(&mut client, 0)?;
                thread::sleep(Duration::from_millis(1));
            }
            assert!(connection.session.is_closed());
        } else {
            assert_eq!(connection.logged_out(), Some(0));
            assert!(connection.session.is_closed());
            assert!(client.player_state.is_none());
            assert!(client.outbound.is_empty());
        }
        server.join().unwrap();
        println!("Receive-timeout actual-cache Connection case{case}:loading exclusion,750/751 boundary,zero-tick poll and recovery passed");
    }
    for path in [key, password] {
        std::fs::remove_file(path)?;
    }
    std::fs::remove_dir(root)?;
    Ok(())
}
