//! Synthetic loopback transport drives the native asynchronous root rebuild path.
use gielinor_client::{
    buffer::Reader, client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection,
    scene::Scene, writer::Writer,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    client.preferences.saved.accepted_eula = client.preferences.required_eula;
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    let key = "work/root-walk-fixture.key";
    let password = "work/root-walk-fixture.password";
    std::fs::write(key, format!("Modulus: {}\n", modulus.to_str_radix(16)))?;
    std::fs::write(password, "fixture")?;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let (send, receive) = std::sync::mpsc::channel::<Option<Vec<u8>>>();
    let (ack_send, ack_receive) = std::sync::mpsc::channel();
    let server = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let (seed, _) = handshake(&mut socket, 16);
        let mut cipher = Some(gielinor_client::crypto::Isaac::new(
            &seed.map(|v| v.wrapping_add(50)),
        ));
        let mut w = Writer::new();
        w.bytes(&[2, 37, 0, 0, 0, 0, 0, 0, 0]);
        w.u16(42);
        w.u8(1);
        w.u64(1);
        w.u64(2);
        w.u64(3);
        let noop = gielinor_client::protocol239::SERVER_PACKETS
            .iter()
            .find(|p| p.opcode == 83)
            .unwrap();
        for _ in 0..3 {
            w.bytes(&gielinor_client::protocol::encode(noop, &[], &mut cipher).unwrap());
        }
        socket.write_all(w.as_slice()).unwrap();
        let rebuild = gielinor_client::protocol239::SERVER_PACKETS
            .iter()
            .find(|p| p.opcode == 125)
            .unwrap();
        while let Some(payload) = receive.recv_timeout(Duration::from_secs(90)).unwrap() {
            socket
                .write_all(
                    &gielinor_client::protocol::encode(rebuild, &payload, &mut cipher).unwrap(),
                )
                .unwrap();
            ack_send.send(()).unwrap();
        }
    });
    let mut live =
        Connection::connect(&mut client, &address.to_string(), "fixture", password, key)?;
    let deadline = Instant::now() + Duration::from_secs(90);
    while !live.session.is_established() || live.packets < 3 {
        live.poll_elapsed(&mut client, 0)?;
        assert!(Instant::now() < deadline, "synthetic login deadline");
        thread::sleep(Duration::from_millis(2));
    }
    let mut r = Reader::new(include_bytes!("../tests/fixtures/world-walk-clear-jvm.bin"));
    let mut checked = 0;
    for case in 0..r.u32()? {
        let id = r.u32()?;
        let size = r.u32()?;
        let initial = (
            r.u8()? != 0,
            r.u8()? != 0,
            [r.u32()? as i32, r.u32()? as i32],
        );
        let expected = (
            r.u8()? != 0,
            r.u8()? != 0,
            [r.u32()? as i32, r.u32()? as i32],
            r.u8()? != 0,
        );
        if id != 0 || size != 104 {
            continue;
        }
        for superseded in [false, true] {
            let mut scene = Scene::empty(0, 0)?;
            scene.walk.pending_pick = initial.0;
            scene.walk.walk_requested = initial.1;
            scene.walk.selected = initial.2;
            scene.walk.point = [100, 200];
            let mesh = gielinor_client::world_mesh::WorldMesh::build(
                &scene,
                &mut client.scripts.source.cache,
                0,
            )?;
            live.world = Some((scene, mesh));
            client.world_base = [0, 0];
            client.data.position = [0, 50, 50];
            client.outbound.clear();
            client.scene_walk_commands.clear();
            let start_packets = live.packets;
            send.send(Some(empty_rebuild(400)))?;
            ack_receive.recv_timeout(Duration::from_secs(10))?;
            loop {
                live.poll_elapsed(&mut client, 0)?;
                assert!(Instant::now() < deadline, "first rebuild deadline");
                if live.packets > start_packets {
                    break;
                }
                thread::sleep(Duration::from_millis(2));
            }
            assert!(
                live.world.is_none(),
                "background scene starts after packet processing"
            );
            assert_eq!(
                live.dispatch_scene_walks(&mut client)?,
                0,
                "no walk while loading"
            );
            let zone = if superseded { 402 } else { 400 };
            if superseded {
                send.send(Some(empty_rebuild(zone)))?;
                ack_receive.recv_timeout(Duration::from_secs(10))?;
            }
            let target_packets = start_packets + if superseded { 2 } else { 1 };
            while live.packets < target_packets || live.world.is_none() {
                live.poll_elapsed(&mut client, 0)?;
                assert!(
                    Instant::now() < deadline,
                    "root rebuild completion deadline"
                );
                thread::sleep(Duration::from_millis(2));
            }
            let scene = &live.world.as_ref().unwrap().0;
            let actual = (
                &scene.walk.pending_pick,
                &scene.walk.walk_requested,
                scene.walk.selected,
                scene.walk.ready(),
            );
            assert_eq!(
                (*actual.0, *actual.1, actual.2, actual.3),
                expected,
                "case{case} superseded{superseded}: root selection survives geometry rebuild"
            );
            assert_eq!(scene.walk.point, [100, 200]);
            let base = (zone as i32 - 6) * 8;
            assert_eq!([scene.base_x, scene.base_y], [base, base]);
            client.outbound.clear();
            assert_eq!(
                live.dispatch_scene_walks(&mut client)?,
                usize::from(expected.3)
            );
            if expected.3 {
                let packet = client.outbound.pop_front().unwrap();
                let expected_packet = gielinor_client::world_action::walk(
                    (base + expected.2[0]) as u16,
                    (base + expected.2[1]) as u16,
                    0,
                )?;
                assert_eq!(
                    (packet.opcode, packet.payload),
                    (expected_packet.opcode, expected_packet.payload)
                );
            }
            assert_eq!(
                live.dispatch_scene_walks(&mut client)?,
                0,
                "root walk consumed once"
            );
            client.outbound.clear();
            checked += 1;
        }
    }
    assert_eq!(r.remaining(), 0);
    assert_eq!(checked, 16);
    send.send(None)?;
    server.join().unwrap();
    live.session.close();
    println!("16 original root selection states survive native packet125 asynchronous and superseded rebuilds");
    Ok(())
}
fn empty_rebuild(zone: u16) -> Vec<u8> {
    let mut w = Writer::new();
    w.u16(zone);
    w.u16(zone);
    w.u8(129);
    w.u16(0);
    w.bytes(&[0; 85]);
    w.into_vec()
}
