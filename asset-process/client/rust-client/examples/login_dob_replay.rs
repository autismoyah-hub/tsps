//! Isolated inline-DOB form, fresh login, rejection, and cancellation replay.
use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection,
    title_input::Screen, Result,
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
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

fn wait(live: &mut Connection, client: &mut ClientState, screen: Screen) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        live.poll(client)?;
        if live.title.as_ref().is_some_and(|s| s.screen == screen) {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    panic!(
        "expected {screen:?}, got {:?}",
        live.title.as_ref().map(|s| s.screen)
    );
}
fn key(live: &mut Connection, client: &mut ClientState, key: i32, ch: i32) -> Result<()> {
    client.input.key_events.push((key, ch));
    live.poll(client)?;
    client.input.key_events.clear();
    Ok(())
}
fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let worker = std::thread::spawn(move || {
        let (mut first, _) = listener.accept().unwrap();
        let (seed, plain) = handshake(&mut first, 16);
        assert_eq!(plain[25], 2);
        first.write_all(&[61, 1]).unwrap();
        assert_eq!(
            first.read(&mut [0; 1]).unwrap(),
            0,
            "DOB prompt must close without resubmitting"
        );
        let (mut second, _) = listener.accept().unwrap();
        let (next, plain) = handshake(&mut second, 16);
        assert_ne!(seed, next);
        assert_eq!(plain[25], 4);
        assert_eq!(
            u32::from_be_bytes(plain[26..30].try_into().unwrap()),
            8_388_607
        );
        assert!(plain.ends_with(b"fixture\0"));
        second.write_all(&[71]).unwrap();
        assert_eq!(second.read(&mut [0; 1]).unwrap(), 0);
        let (mut third, _) = listener.accept().unwrap();
        let (_, plain) = handshake(&mut third, 16);
        assert_eq!(
            plain[25], 2,
            "cancelled DOB must not leak into password login"
        );
        third.write_all(&[3]).unwrap();
    });
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    let key_file = "work/dob-fixture.key";
    let password = "work/dob-fixture.password";
    std::fs::write(key_file, format!("Modulus: {}\n", modulus.to_str_radix(16)))?;
    let test_password = std::env::var("OSRS_TEST_PASSWORD").map_err(|_| gielinor_client::Error::Invalid("OSRS_TEST_PASSWORD is missing or non-Unicode"))?;
        std::fs::write(password, test_password)?;
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let mut live = Connection::connect(
        &mut client,
        &address.to_string(),
        "dob_probe",
        password,
        key_file,
    )?;
    std::fs::remove_file(password)?;
    std::fs::remove_file(key_file)?;
    wait(&mut live, &mut client, Screen::InlineDateOfBirth)?;
    key(&mut live, &mut client, 84, 0)?;
    assert_eq!(
        live.title.as_ref().unwrap().lines[1],
        "Please ensure all characters are populated."
    );
    for ch in b"31129999" {
        key(&mut live, &mut client, -1, *ch as i32)?;
    }
    key(&mut live, &mut client, 84, 0)?;
    assert_eq!(
        live.title.as_ref().unwrap().screen,
        Screen::ConnectingDateOfBirth
    );
    wait(&mut live, &mut client, Screen::InlineDateOfBirth)?;
    client.preferences.mouse_camera = false;
    client.input.pressed_buttons = 4;
    client.input.press_position = [gielinor_client::title::center(client.canvas[0]) + 80, 321];
    live.poll(&mut client)?;
    client.input.pressed_buttons = 0;
    assert_eq!(live.title.as_ref().unwrap().screen, Screen::Credentials);
    assert_eq!(live.title.as_ref().unwrap().date_of_birth, [None; 8]);
    assert!(live.title.as_ref().unwrap().password.is_empty());
    {
        let title = live.title.as_mut().unwrap();
        title.password = "fixture".into();
        title.focus = 1;
    }
    key(&mut live, &mut client, 84, 0)?;
    wait(&mut live, &mut client, Screen::InvalidCredentials)?;
    worker.join().unwrap();
    println!("Inline DOB: missing-field validation, editing, fresh socket/type4 age sentinel, response71 state, cancellation and clean password submission passed");
    Ok(())
}
