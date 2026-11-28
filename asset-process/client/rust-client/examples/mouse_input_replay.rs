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
    let root = std::path::PathBuf::from(format!("work/mouse-input-replay-{}", std::process::id()));
    std::fs::create_dir_all(&root)?;
    let key = root.join("fixture.key");
    let password = root.join("fixture.password");
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    std::fs::write(&key, format!("Modulus: {}\n", modulus.to_str_radix(16)))?;
    std::fs::write(&password, "fixture")?;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let (send, received) = std::sync::mpsc::channel();
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let done = stop.clone();
    let server = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let (seed, _) = handshake(&mut socket, 16);
        let mut w = Writer::new();
        w.bytes(&[2, 37, 0, 0, 0, 0, 0, 0, 0]);
        w.u16(42);
        w.u8(0);
        w.u64(1);
        w.u64(2);
        w.u64(3);
        let mut cipher = Some(gielinor_client::crypto::Isaac::new(
            &seed.map(|n| n.wrapping_add(50)),
        ));
        let spec = gielinor_client::protocol239::SERVER_PACKETS
            .iter()
            .find(|p| p.opcode == 83)
            .unwrap();
        for _ in 0..3 {
            w.bytes(&gielinor_client::protocol::encode(spec, &[], &mut cipher).unwrap());
        }
        socket.write_all(w.as_slice()).unwrap();
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .unwrap();
        let mut decoder = gielinor_client::protocol::Decoder::new(Some(
            gielinor_client::crypto::Isaac::new(&seed),
        ));
        let deadline = Instant::now() + Duration::from_secs(30);
        while !done.load(std::sync::atomic::Ordering::Acquire) {
            assert!(Instant::now() < deadline);
            let mut bytes = [0; 2048];
            match socket.read(&mut bytes) {
                Ok(0) => break,
                Ok(n) => {
                    decoder.feed(&bytes[..n]).unwrap();
                    while let Some(packet) = decoder
                        .next_packet(gielinor_client::protocol239::CLIENT_PACKETS)
                        .unwrap()
                    {
                        if matches!(packet.opcode, 35 | 110) {
                            send.send(packet).unwrap();
                        }
                    }
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                    ) =>
                {
                    ()
                }
                Err(e) => panic!("socket {e}"),
            }
        }
    });
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let mut live = Connection::connect(
        &mut client,
        &address.to_string(),
        "mouse_probe",
        password.to_str().unwrap(),
        key.to_str().unwrap(),
    )?;
    let deadline = Instant::now() + Duration::from_secs(20);
    while !live.session.is_established() {
        assert!(Instant::now() < deadline);
        live.poll(&mut client)?;
        thread::sleep(Duration::from_millis(1));
    }
    let scene = gielinor_client::scene::Scene::empty_sized(0, 0, 1, 1)?;
    let mesh =
        gielinor_client::world_mesh::WorldMesh::build(&scene, &mut client.scripts.source.cache, 0)?;
    live.world = Some((scene, mesh));
    live.start_mouse_recorder()?;
    live.mouse_pointer([100, 200]);
    live.poll_elapsed(&mut client, 1)?;
    thread::sleep(Duration::from_millis(70));
    live.mouse_click(1, [120, 220]);
    live.poll_elapsed(&mut client, 0)?;
    assert!(received.try_recv().is_err());
    live.poll_elapsed(&mut client, 1)?;
    live.poll_elapsed(&mut client, 0)?;
    let movement = received.recv_timeout(Duration::from_secs(5)).unwrap();
    let click = received.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(movement.opcode, 110);
    assert_eq!(click.opcode, 35);
    assert_eq!(click.payload, [0, 248, 126, 255, 0, 92, 128]);
    assert_eq!(client.data.mouse_reported, Some([100, 200]));
    use gielinor_client::vm::{Host, Stacks};
    for (op, value) in [(3326, 100), (3327, 200)] {
        let mut stacks = Stacks::default();
        client.extended(op, false, &mut stacks)?;
        assert_eq!(stacks.pop_int()?, value);
    }
    client.preferences.mouse_camera = true;
    live.mouse_click(4, [500, 500]);
    live.poll_elapsed(&mut client, 1)?;
    assert_eq!(live.mouse_click_packets, 1);
    live.mouse_pointer([-1, -1]);
    live.poll_elapsed(&mut client, 1)?;
    thread::sleep(Duration::from_millis(70));
    live.mouse_click(2, [-500, 9999]);
    live.poll_elapsed(&mut client, 1)?;
    live.poll_elapsed(&mut client, 0)?;
    let movement = received.recv_timeout(Duration::from_secs(5)).unwrap();
    let click = received.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(movement.opcode, 110);
    assert_eq!(click.opcode, 35);
    assert_eq!(&click.payload[..2], &[0, 128]);
    assert_eq!(&click.payload[4..], &[1, 119, 128]);
    assert_eq!(client.data.mouse_reported, Some([-1, -1]));
    assert_eq!(live.mouse_click_packets, 2);
    client.prepare_login()?;
    assert_eq!(client.data.mouse_reported, Some([-1, -1]));
    stop.store(true, std::sync::atomic::Ordering::Release);
    server.join().unwrap();
    for path in [key, password] {
        std::fs::remove_file(path)?;
    }
    std::fs::remove_dir(root)?;
    println!("Mouse sampling, tick gating, ISAAC wire packets110/35, middle-camera suppression, outside coordinates and script queries passed");
    Ok(())
}
