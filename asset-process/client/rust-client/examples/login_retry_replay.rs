//! Loopback retry bounds and account-status response screens.
use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection,
    title_input::Screen, Result,
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
fn exchange(socket: &mut TcpStream, reply: &[u8]) -> Vec<u8> {
    socket.set_nonblocking(false).unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(3)))
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
    assert_eq!(&body[..8], &[0, 0, 0, 239, 0, 0, 0, 1]);
    socket.write_all(reply).unwrap();
    body
}
fn main() -> Result<()> {
    for replies in [
        vec![vec![61]],
        vec![vec![61, 0]],
        vec![vec![61, 1]],
        vec![vec![16]],
        vec![vec![55]],
        vec![vec![72]],
        vec![vec![73]],
        vec![vec![4]],
        vec![vec![18]],
        vec![vec![32]],
        vec![vec![23], vec![3]],
        vec![vec![23], vec![23]],
        vec![vec![23], vec![21, 0], vec![23]],
    ] {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let address = listener.local_addr()?;
        listener.set_nonblocking(true)?;
        let count = Arc::new(AtomicUsize::new(0));
        let worker_count = count.clone();
        let expected = replies.len();
        let final_code = replies.last().unwrap()[0];
        let inline_dob = replies.last().unwrap().as_slice() == [61, 1];
        let worker = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(10);
            let mut bodies = Vec::new();
            while bodies.len() < replies.len() && Instant::now() < deadline {
                match listener.accept() {
                    Ok((mut socket, _)) => {
                        worker_count.fetch_add(1, Ordering::SeqCst);
                        bodies.push(exchange(&mut socket, &replies[bodies.len()]));
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(1))
                    }
                    Err(e) => panic!("{e}"),
                }
            }
            for pair in bodies.windows(2) {
                assert_ne!(
                    pair[0], pair[1],
                    "fresh socket must install fresh random login seed"
                );
            }
            bodies.len()
        });
        let mut client = ClientState::load(DiskCache::open(
            "work/kotlin-world/.data/cache/js5",
            64 << 20,
        )?)?;
        let password = "work/login-retry-test.password";
        let test_password = std::env::var("OSRS_TEST_PASSWORD").map_err(|_| gielinor_client::Error::Invalid("OSRS_TEST_PASSWORD is missing or non-Unicode"))?;
        std::fs::write(password, test_password)?;
        let mut live = Connection::connect(
            &mut client,
            &address.to_string(),
            "retry_probe",
            password,
            "work/kotlin-world/.data/client.key",
        )?;
        std::fs::remove_file(password)?;
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            live.poll(&mut client)?;
            if live.title.as_ref().is_some_and(|s| {
                matches!(
                    s.screen,
                    Screen::Retry
                        | Screen::InvalidCredentials
                        | Screen::AccountStatus(_)
                        | Screen::Message
                        | Screen::LauncherRequired
                        | Screen::Privacy
                        | Screen::DateOfBirth
                        | Screen::InlineDateOfBirth
                )
            }) {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        let screen = live.title.as_ref().unwrap().screen;
        // Rejected status must remain inert, without a third response-23 retry.
        for _ in 0..100 {
            live.poll(&mut client)?;
        }
        let actual = count.load(Ordering::SeqCst);
        assert_eq!(
            actual, expected,
            "response 23 should retry exactly once, including across transfer delays"
        );
        assert_eq!(
            screen,
            match final_code {
                61 =>
                    if inline_dob {
                        Screen::InlineDateOfBirth
                    } else {
                        Screen::DateOfBirth
                    },
                16 => Screen::LauncherRequired,
                55 => Screen::Privacy,
                72 => Screen::DateOfBirth,
                73 => Screen::Message,
                3 => Screen::InvalidCredentials,
                4 => Screen::AccountStatus(0),
                18 => Screen::AccountStatus(1),
                32 => Screen::AccountStatus(2),
                _ => Screen::Retry,
            }
        );
        assert_eq!(worker.join().unwrap(), expected);
        println!("Login replies: {expected} sockets, fresh requests, final reply {final_code}, bounded retry verified");
    }
    Ok(())
}
