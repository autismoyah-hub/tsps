use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection, writer::Writer,
    Result,
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
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
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    let key = "work/packet-failure-fixture.key";
    let password = "work/packet-failure-fixture.password";
    std::fs::write(key, format!("Modulus: {}\n", modulus.to_str_radix(16)))?;
    std::fs::write(password, "fixture")?;
    for case in 0..6 {
        let mut cache = DiskCache::open_remote(
            "work/kotlin-world/.data/cache/js5",
            64 << 20,
            "127.0.0.1:43617".parse().unwrap(),
        )?;
        cache.wait_for_recovery(true);
        let mut client = ClientState::load(cache)?;
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        let (ready_tx, ready_rx) = mpsc::channel();
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
            ready_rx.recv_timeout(Duration::from_secs(20)).unwrap();
            let (opcode, payload) = match case {
                0 | 1 => (149, vec![]),
                2 => (47, vec![0, 1, 0]),
                3 => (67, vec![65]),
                4 => (49, vec![]),
                _ => (114, vec![0, 255, 255, 255, 255]),
            };
            let spec = if opcode == 149 {
                gielinor_client::protocol239::PacketSpec {
                    opcode: 149,
                    size: 0,
                    name: "fixture unknown",
                }
            } else {
                *gielinor_client::protocol239::SERVER_PACKETS
                    .iter()
                    .find(|s| s.opcode == opcode)
                    .unwrap()
            };
            let mut bytes =
                gielinor_client::protocol::encode(&spec, &payload, &mut cipher).unwrap();
            bytes.extend(gielinor_client::protocol::encode(tick, &[], &mut cipher).unwrap());
            socket.write_all(&bytes).unwrap();
            if case == 0 {
                drop(socket);
                let (mut resumed, _) = listener.accept().unwrap();
                handshake(&mut resumed, 18);
                resumed.write_all(&[3]).unwrap();
            } else {
                let mut scratch = [0; 256];
                loop {
                    match socket.read(&mut scratch) {
                        Ok(0) => break,
                        Ok(_) => (),
                        Err(error)
                            if matches!(
                                error.kind(),
                                std::io::ErrorKind::ConnectionReset
                                    | std::io::ErrorKind::ConnectionAborted
                            ) =>
                        {
                            break
                        }
                        Err(error) => panic!("socket close: {error}"),
                    }
                }
            }
        });
        let mut connection =
            Connection::connect(&mut client, &address.to_string(), "fixture", password, key)?;
        let deadline = Instant::now() + Duration::from_secs(45);
        let mut sent = false;
        let mut reconnecting = false;
        let mut ticks_at_failure = 0;
        loop {
            assert!(
                Instant::now() < deadline,
                "packet failure case{case} stalled"
            );
            connection.poll(&mut client)?;
            if !sent && connection.session.is_established() {
                client.logout_timer = if case == 1 { 100 } else { 0 };
                ticks_at_failure = client.server_tick;
                ready_tx.send(()).unwrap();
                sent = true;
            }
            reconnecting |= connection.is_reconnecting();
            if case == 0 {
                if reconnecting && !connection.is_reconnecting() {
                    break;
                }
            } else if case == 5 {
                if sent && client.server_tick > ticks_at_failure {
                    assert!(connection.session.is_established());
                    assert!(connection.logged_out().is_none());
                    connection.session.close();
                    break;
                }
            } else if connection.logged_out().is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
        server.join().unwrap();
        if case == 0 {
            assert!(reconnecting);
        } else if case == 5 {
            assert!(client.server_tick > ticks_at_failure);
        } else {
            assert_eq!(connection.logged_out(), Some(0));
            assert!(connection.session.is_closed());
            assert!(client.player_state.is_none());
            assert!(client.outbound.is_empty());
            if case == 2 {
                assert_eq!(
                    client.active_world, 0,
                    "missing world changed selected view"
                );
            }
            assert_eq!(
                client.server_tick, ticks_at_failure,
                "queued packet survived failure case{case}"
            );
            connection.poll(&mut client)?;
            assert_eq!(connection.logged_out(), Some(0));
        }
        println!("Packet failure actual Connection case{case} passed");
    }
    Ok(())
}
