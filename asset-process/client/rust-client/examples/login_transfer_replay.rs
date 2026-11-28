use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection,
    title_input::Screen, Result,
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    time::{Duration, Instant},
};
fn exchange(socket: &mut TcpStream, reply: &[u8]) {
    socket
        .set_read_timeout(Some(Duration::from_secs(15)))
        .unwrap();
    let mut handshake = [0];
    socket.read_exact(&mut handshake).unwrap();
    assert_eq!(handshake, [14]);
    socket.write_all(&[0, 1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
    let mut header = [0; 3];
    socket.read_exact(&mut header).unwrap();
    assert_eq!(header[0], 16);
    let mut body = vec![0; u16::from_be_bytes([header[1], header[2]]) as usize];
    socket.read_exact(&mut body).unwrap();
    assert_eq!(&body[..8], &[0, 0, 0, 239, 0, 0, 0, 1]);
    socket.write_all(reply).unwrap();
}
fn main() -> Result<()> {
    let batched = std::env::args().any(|arg| arg == "--batched");
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let (tx, rx) = mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut first, _) = listener.accept().unwrap();
        exchange(&mut first, &[21, 0]);
        drop(first);
        let (mut second, _) = listener.accept().unwrap();
        tx.send(()).unwrap();
        exchange(&mut second, &[3]);
    });
    let cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let mut client = ClientState::load(cache)?;
    let password = "work/login-transfer-test.password";
    let test_password = std::env::var("OSRS_TEST_PASSWORD").map_err(|_| gielinor_client::Error::Invalid("OSRS_TEST_PASSWORD is missing or non-Unicode"))?;
        std::fs::write(password, test_password)?;
    let mut live = Connection::connect(
        &mut client,
        &address.to_string(),
        "transfer_probe",
        password,
        "work/kotlin-world/.data/client.key",
    )?;
    std::fs::remove_file(password)?;
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        assert!(
            Instant::now() < deadline,
            "First login transfer reply timed out"
        );
        live.poll_elapsed(&mut client, if batched { 0 } else { 1 })?;
        if live
            .title
            .as_ref()
            .is_some_and(|s| s.lines[0] == "You have only just left another world.")
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(live.title.as_ref().unwrap().lines[2], "3 seconds.");
    let steps = if batched {
        let mut steps = vec![0; 100];
        steps.extend(std::iter::repeat_n(10, 17));
        steps.push(9);
        steps.extend(std::iter::repeat_n(0, 100));
        steps
    } else { vec![1; 178] };
    for ticks in steps {
        live.poll_elapsed(&mut client, ticks)?;
        assert!(
            matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)),
            "Retry connected before countdown expiry"
        );
    }
    live.poll(&mut client)?;
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        assert!(Instant::now() < deadline, "Automatic login retry timed out");
        live.poll(&mut client)?;
        if live
            .title
            .as_ref()
            .is_some_and(|s| s.screen == Screen::InvalidCredentials)
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    rx.recv_timeout(Duration::from_secs(1)).unwrap();
    server.join().unwrap();
    println!("Live login bridge transfer replay passed: 180 exact client cycles, batched={batched}, no early retry, new socket and revision-239 login, subsequent rejection routed to title");
    Ok(())
}
