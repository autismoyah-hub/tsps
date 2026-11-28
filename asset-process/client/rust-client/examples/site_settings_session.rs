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
    let seed = std::array::from_fn(|i| {
        u32::from_be_bytes(plain[1 + i * 4..5 + i * 4].try_into().unwrap())
    });
    let mut encrypted = body[17 + len..].to_vec();
    gielinor_client::crypto::xtea(&mut encrypted, seed, true);
    let mut reader = gielinor_client::buffer::Reader::new(&encrypted);
    assert_eq!(reader.string().unwrap(), "fixture");
    reader.bytes(29).unwrap();
    assert_eq!(
        reader.string().unwrap(),
        if opcode == 16 { "" } else { "site-€" }
    );
    (seed, plain)
}

fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    let key = "work/site-settings-session.key";
    let password = "work/site-settings-session.password";
    std::fs::write(key, format!("Modulus: {}\n", modulus.to_str_radix(16)))?;
    std::fs::write(password, "fixture")?;
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
        let spec = gielinor_client::protocol239::SERVER_PACKETS
            .iter()
            .find(|s| s.opcode == 67)
            .unwrap();
        w.bytes(
            &gielinor_client::protocol::encode(spec, b"site-\x80\0trailing", &mut cipher).unwrap(),
        );
        let tick = gielinor_client::protocol239::SERVER_PACKETS
            .iter()
            .find(|s| s.opcode == 83)
            .unwrap();
        w.bytes(&gielinor_client::protocol::encode(tick, &[], &mut cipher).unwrap());
        socket.write_all(w.as_slice()).unwrap();
        drop(socket);
        let (mut resumed, _) = listener.accept().unwrap();
        handshake(&mut resumed, 18);
        resumed.write_all(&[3]).unwrap();
    });
    let mut connection =
        Connection::connect(&mut client, &address.to_string(), "fixture", password, key)?;
    let deadline = Instant::now() + Duration::from_secs(90);
    let mut reconnecting = false;
    loop {
        assert!(Instant::now() < deadline, "site setting reconnect stalled");
        connection.poll(&mut client)?;
        reconnecting |= connection.is_reconnecting();
        if reconnecting && !connection.is_reconnecting() {
            break;
        }
        thread::sleep(Duration::from_millis(1));
    }
    server.join().unwrap();
    assert_eq!(client.site_settings, "site-€");
    println!("Actual Connection socket login -> encrypted packet67 -> disconnect/reconnect: updated CP1252 site setting present in XTEA login body");
    Ok(())
}
