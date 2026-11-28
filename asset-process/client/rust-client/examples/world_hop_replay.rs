//! Two loopback worlds verify actual packet-91 handoff and a fresh normal login.
use gielinor_client::{
    client_state::ClientState, crypto::Isaac, disk_cache::DiskCache, inventory::Item,
    live_runtime::Connection, protocol, protocol239::SERVER_PACKETS, writer::Writer, Result,
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
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

fn account(socket: &mut TcpStream, seed: [u32; 4], hop: bool) {
    let mut cipher = Some(Isaac::new(&seed.map(|n| n.wrapping_add(50))));
    let mut w = Writer::new();
    w.bytes(&[2, 37, 0, 0, 0, 0, 0, 0, 0]);
    w.u16(if hop { 43 } else { 42 });
    w.u8(if hop { 0 } else { 1 });
    w.u64(1);
    w.u64(2);
    w.u64(3);
    let mut payload = Writer::new();
    if hop {
        payload.string("new-world").unwrap();
    } else {
        payload.string("127.0.0.2").unwrap();
        payload.u16(239);
        payload.u32(0x80000001);
    }
    let opcode = if hop { 24 } else { 91 };
    let spec = SERVER_PACKETS.iter().find(|s| s.opcode == opcode).unwrap();
    w.bytes(&protocol::encode(spec, payload.as_slice(), &mut cipher).unwrap());
    if !hop {
        let mut stale = Writer::new();
        stale.string("stale-old-world").unwrap();
        w.bytes(
            &protocol::encode(
                SERVER_PACKETS.iter().find(|s| s.opcode == 24).unwrap(),
                stale.as_slice(),
                &mut cipher,
            )
            .unwrap(),
        );
    }
    socket.write_all(w.as_slice()).unwrap();
}
fn main() -> Result<()> {
    let retry = std::env::args().any(|a| a == "--retry");
    let reject = std::env::args().any(|a| a == "--reject");
    let first = TcpListener::bind("127.0.0.1:0")?;
    let address = first.local_addr()?;
    let second = TcpListener::bind(("127.0.0.2", address.port()))?;
    let (allow_tx, allow_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut source, _) = first.accept().unwrap();
        let (old_seed, _) = handshake(&mut source, 16);
        account(&mut source, old_seed, false);
        drop(source);
        let (mut target, _) = second.accept().unwrap();
        let (mut new_seed, mut plain) = handshake(&mut target, 16);
        if retry {
            target.write_all(&[23]).unwrap();
            drop(target);
            let (mut next, _) = second.accept().unwrap();
            let (seed, block) = handshake(&mut next, 16);
            assert_ne!(new_seed, seed);
            target = next;
            new_seed = seed;
            plain = block;
        }
        assert_ne!(old_seed, new_seed);
        assert!(
            plain
                .windows(b"hop-fixture".len())
                .any(|v| v == b"hop-fixture"),
            "normal credential block missing"
        );
        allow_rx.recv_timeout(Duration::from_secs(30)).unwrap();
        if reject {
            target.write_all(&[3]).unwrap();
        } else {
            account(&mut target, new_seed, true);
        }
        done_rx.recv_timeout(Duration::from_secs(30)).unwrap();
    });
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    let key = "work/world-hop-fixture.key";
    let password = "work/world-hop-fixture.password";
    std::fs::write(key, format!("Modulus: {}\n", modulus.to_str_radix(16)))?;
    let test_password = std::env::var("OSRS_TEST_PASSWORD").map_err(|_| gielinor_client::Error::Invalid("OSRS_TEST_PASSWORD is missing or non-Unicode"))?;
        std::fs::write(password, test_password)?;
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    client.data.server_string = "before".into();
    client.scripts.variables.set_player(100, 239);
    client
        .inventories
        .containers
        .insert(93, vec![Item { id: 1277, count: 1 }]);
    let mut live = Connection::connect(
        &mut client,
        &address.to_string(),
        "hop_probe",
        password,
        key,
    )?;
    std::fs::remove_file(key)?;
    std::fs::remove_file(password)?;
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut allowed = false;
    loop {
        assert!(Instant::now() < deadline, "world hop timed out");
        live.poll_elapsed(&mut client, 1)?;
        if client.data.world == 239 && !allowed {
            assert_eq!(client.data.world_flags, 0x80000001u32 as i32);
            assert!(client.data.members);
            assert!(client.data.account_members, "source login account subscription");
            assert_eq!(
                client.data.server_string, "before",
                "processed old-world packet after transfer"
            );
            assert!(!live.is_reconnecting());
            assert_eq!(live.status_lines(), Some(vec!["Please wait..."]));
            let mut raster = gielinor_client::raster::Raster::new(765, 503, 765 * 503)?;
            assert!(!live.draw_title(&mut client, &mut raster)?, "world hop started the title screen/music");
            allow_tx.send(()).unwrap();
            allowed = true;
        }
        if allowed && !reject && live.title.is_none() && client.data.server_string == "new-world" {
            break;
        }
        if allowed && reject && live.logged_out().is_some() {
            assert_eq!(
                live.title.as_ref().unwrap().screen,
                gielinor_client::title_input::Screen::InvalidCredentials
            );
            assert!(!live.is_reconnecting());
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(client.data.members, "target world membership must survive account replies");
    assert_eq!(client.data.account_members, reject, "target account subscription is independent of world membership");
    if reject {
        assert!(!client.inventories.containers.is_empty());
        assert_eq!(client.scripts.variables.player(100), 239);
    } else {
        assert!(client.inventories.containers.is_empty());
        assert_eq!(client.scripts.variables.player(100), 0);
    }
    assert_eq!(client.data.world, 239);
    assert_eq!(client.data.world_flags, 0x80000001u32 as i32);
    done_tx.send(()).unwrap();
    server.join().unwrap();
    println!("World-hop packet replay passed (retry={retry}, reject={reject}): new loopback host, retained custom port, normal opcode16 with fresh seed, stale packets discarded, reset only on success and world flags retained");
    Ok(())
}
