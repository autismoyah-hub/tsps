//! Owned loopback HTTP/game endpoints; synthetic credentials; real Connection/cache integration.
use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection, token_auth::Config,
    Result,
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    thread,
    time::{Duration, Instant},
};
fn accept(listener: &TcpListener) -> TcpStream {
    listener.set_nonblocking(true).unwrap();
    let end = Instant::now() + Duration::from_secs(30);
    loop {
        match listener.accept() {
            Ok((s, _)) => {
                s.set_nonblocking(false).unwrap();
                s.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
                return s;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                assert!(Instant::now() < end, "owned endpoint accept timeout");
                thread::sleep(Duration::from_millis(2));
            }
            Err(e) => panic!("owned endpoint: {e}"),
        }
    }
}
fn http(
    listener: &TcpListener,
    status: u32,
    content_type: &str,
    body: &str,
) -> (String, Vec<(String, String)>, Vec<u8>) {
    let mut socket = accept(listener);
    let mut head = Vec::new();
    while !head.ends_with(b"\r\n\r\n") {
        let mut b = [0];
        socket.read_exact(&mut b).unwrap();
        head.push(b[0]);
        assert!(head.len() < 65536);
    }
    let text = String::from_utf8(head).unwrap();
    let mut lines = text.split("\r\n");
    let request = lines.next().unwrap().to_owned();
    let headers: Vec<_> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(k, v)| (k.to_owned(), v.trim().to_owned()))
        .collect();
    let n = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("Content-Length"))
        .map_or(0, |(_, v)| v.parse::<usize>().unwrap());
    let mut input = vec![0; n];
    socket.read_exact(&mut input).unwrap();
    write!(socket,"HTTP/1.1 {status} Test\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
    (request, headers, input)
}
fn handshake(socket: &mut TcpStream, expected: &str, token: bool) {
    let mut byte = [0];
    socket.read_exact(&mut byte).unwrap();
    assert_eq!(byte, [14]);
    socket.write_all(&[0, 1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
    let mut header = [0; 3];
    socket.read_exact(&mut header).unwrap();
    assert_eq!(header[0], 16);
    let mut body = vec![0; u16::from_be_bytes([header[1], header[2]]) as usize];
    socket.read_exact(&mut body).unwrap();
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    let private = num_bigint::BigInt::from(65537)
        .modinv(&(&modulus - 1))
        .unwrap();
    let n = u16::from_be_bytes([body[15], body[16]]) as usize;
    let plain = num_bigint::BigInt::from_signed_bytes_be(&body[17..17 + n])
        .modpow(&private, &modulus)
        .to_bytes_be()
        .1;
    assert_eq!(plain[0], 1);
    assert_eq!(
        plain[30],
        if token { 2 } else { 0 },
        "RSA login credential mode"
    );
    assert_eq!(&plain[31..plain.len() - 1], expected.as_bytes());
    assert_eq!(plain.last(), Some(&0));
}
fn main() -> Result<()> {
    let key = "work/token-login-fixture.key";
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    std::fs::write(key, format!("Modulus: {}\n", modulus.to_str_radix(16)))?;
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let password = "work/token-login-fixture.password";
    std::fs::write(password, "fixture-password")?;
    for case in 0..5 {
        let auth = TcpListener::bind("127.0.0.1:0")?;
        let auth_address = auth.local_addr()?;
        let game = TcpListener::bind("127.0.0.1:0")?;
        let game_address = game.local_addr()?;
        let config = Config {
            auth_base: format!("http://{auth_address}/"),
            session_base: format!("http://{auth_address}/"),
            basic_authorization: "synthetic-basic".into(),
            access_token: if case == 1 {
                String::new()
            } else {
                "old-access".into()
            },
            refresh_token: "old-refresh".into(),
            session_id: "synthetic-session".into(),
            account_id: "account".into(),
        };
        let auth_worker = thread::spawn(move || {
            if case == 4 {
                return;
            }
            if case == 1 {
                let (line, headers, body) = http(
                    &auth,
                    200,
                    "application/json",
                    r#"{"token":"session-play"}"#,
                );
                assert!(line.starts_with("POST /game-session/v1/tokens "));
                assert_eq!(body, br#"{"accountId":"account"}"#);
                assert!(headers
                    .iter()
                    .any(|(k, v)| k.eq_ignore_ascii_case("Authorization")
                        && v == "Bearer synthetic-session"));
                return;
            }
            let rounds = if case == 0 { 2 } else { 1 };
            for round in 0..rounds {
                let (line, headers, body) = http(
                    &auth,
                    if case == 2 { 401 } else { 200 },
                    "application/json",
                    r#"{"access_token":"new-access","refresh_token":"new-refresh"}"#,
                );
                assert!(line.starts_with("POST /shield/oauth/token?"));
                assert!(line.contains(if round == 0 {
                    "refresh_token=old-refresh"
                } else {
                    "refresh_token=new-refresh"
                }));
                assert!(body.is_empty());
                assert!(headers
                    .iter()
                    .any(|(k, v)| k.eq_ignore_ascii_case("Authorization")
                        && v == "Basic synthetic-basic"));
                if case == 2 {
                    return;
                }
                let (line, headers, body) = http(
                    &auth,
                    200,
                    "application/json",
                    if case == 3 {
                        "invalid"
                    } else if round == 0 {
                        r#"{"token":"first-play"}"#
                    } else {
                        r#"{"token":"retry-play"}"#
                    },
                );
                assert!(line.starts_with("GET /public/v1/games/YCfdbvr2pM1zUYMxJRexZY/play "));
                assert!(body.is_empty());
                assert!(headers
                    .iter()
                    .any(|(k, v)| k.eq_ignore_ascii_case("Authorization")
                        && v == "Bearer new-access"));
            }
        });
        let game_copy = game.try_clone()?;
        let game_worker = if case < 2 || case == 4 {
            Some(thread::spawn(move || {
                let mut socket = accept(&game_copy);
                handshake(
                    &mut socket,
                    if case == 0 {
                        "first-play"
                    } else if case == 1 {
                        "session-play"
                    } else {
                        "fixture-password"
                    },
                    case != 4,
                );
                if case == 0 || case == 4 {
                    socket.write_all(&[23]).unwrap();
                    drop(socket);
                    let mut socket = accept(&game_copy);
                    handshake(
                        &mut socket,
                        if case == 4 {
                            "fixture-password"
                        } else {
                            "retry-play"
                        },
                        case != 4,
                    );
                    socket.write_all(&[3]).unwrap();
                } else {
                    socket.write_all(&[3]).unwrap();
                }
            }))
        } else {
            None
        };
        client.preferences.required_eula=if case==2 {3} else {-1};
        client.preferences.saved.accepted_eula=-1;
        let mut live = if case == 4 {
            Connection::connect(
                &mut client,
                &game_address.to_string(),
                "fixture-token-user",
                password,
                key,
            )?
        } else {
            Connection::connect_token(
                &mut client,
                &game_address.to_string(),
                "fixture-token-user",
                config,
                key,
            )?
        };
        if case==2 {
            use gielinor_client::title_input::Screen;
            assert_eq!(live.title.as_ref().unwrap().screen,Screen::Legal);
            assert!(live.session.is_closed());
            let ready=Instant::now()+Duration::from_secs(30);
            loop {
                let mut raster=live.paint(&mut client,765,503)?;
                live.draw_title(&mut client,&mut raster)?;
                if raster.pixels().iter().copied().collect::<std::collections::HashSet<_>>().len()>100 {
                    std::fs::write("work/w07-title-legal-screen.bmp",raster.bmp()?)?;break;
                }
                assert!(Instant::now()<ready);thread::sleep(Duration::from_millis(20));
            }
            for (point,expected) in [([462,311],Screen::LegalDeclined),([382,321],Screen::Legal),([302,311],Screen::Token)] {
                client.input.begin_cycle();client.input.pressed_buttons=1;client.input.press_position=point;
                live.poll_elapsed(&mut client,1)?;
                assert_eq!(live.title.as_ref().unwrap().screen,expected);
                assert!(live.session.is_closed());
            }
            assert_eq!(client.preferences.saved.accepted_eula,3);
            let saved=client.preferences.encode_saved()?;
            assert_eq!(gielinor_client::saved_preferences::SavedPreferences::decode(&saved)?.accepted_eula,3);
            let profile_root=std::path::Path::new("work/w07-title-legal-profile");
            let (mut profile,_)=gielinor_client::saved_preferences::Profile::open(profile_root,"synthetic-policy",b"synthetic-key")?;
            profile.save(&saved)?;
            let (_,restored)=gielinor_client::saved_preferences::Profile::open(profile_root,"synthetic-policy",b"synthetic-key")?;
            assert_eq!(gielinor_client::saved_preferences::SavedPreferences::decode(&restored.unwrap())?.accepted_eula,3);
            client.input.begin_cycle();client.input.key_events.push((84,0));live.poll_elapsed(&mut client,1)?;client.input.begin_cycle();
            println!("Policy revision gate: decline/back/accept/Play dispatch and persisted preference roundtrip pass");
        }
        let deadline = Instant::now() + Duration::from_secs(60);
        loop {
            live.poll(&mut client)?;
            if live.logged_out().is_some() {
                break;
            }
            assert!(Instant::now() < deadline, "token login poll timeout");
            thread::sleep(Duration::from_millis(2));
        }
        auth_worker.join().unwrap();
        if let Some(worker) = game_worker {
            worker.join().unwrap();
        }
        if case == 2 || case == 3 {
            game.set_nonblocking(true)?;
            assert!(
                matches!(game.accept(),Err(e)if e.kind()==std::io::ErrorKind::WouldBlock),
                "game connection opened before successful token acquisition"
            );
        }
        assert!(live.session.is_closed());
        let title = live.title.as_ref().unwrap();
        assert_eq!(
            title.screen,
            if case < 2 || case == 4 {
                gielinor_client::title_input::Screen::InvalidCredentials
            } else {
                gielinor_client::title_input::Screen::Retry
            }
        );
        if case == 2 {
            assert_eq!(
                live.title.as_ref().unwrap().lines,
                ["Failed to login.", "Please try again.", ""].map(str::to_owned)
            );
            let title_deadline = Instant::now() + Duration::from_secs(30);
            let raster = loop {
                let mut raster = live.paint(&mut client, 765, 503)?;
                assert!(live.draw_title(&mut client, &mut raster)?);
                if raster
                    .pixels()
                    .iter()
                    .copied()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    > 100
                {
                    break raster;
                }
                assert!(
                    Instant::now() < title_deadline,
                    "token rejection title assets did not become ready"
                );
                thread::sleep(Duration::from_millis(20));
            };
            std::fs::write("work/w07-token-title-rejected.bmp", raster.bmp()?)?;
            client.input.begin_cycle();
            client.input.key_events.push((84, 0));
            live.poll_elapsed(&mut client, 1)?;
            client.input.begin_cycle();
            assert_eq!(live.title.as_ref().unwrap().screen, gielinor_client::title_input::Screen::Token);
            assert!(live.session.is_closed());
            for label in ["", "fixture-token-user"] {
                live.title.as_mut().unwrap().token_label=label.into();
                let mut raster=live.paint(&mut client,765,503)?;
                assert!(live.draw_title(&mut client,&mut raster)?);
                std::fs::write(format!("work/w07-token-title-play-{}.bmp",if label.is_empty(){"plain"}else{"label"}),raster.bmp()?)?;
            }
            let mut assets=Vec::new();
            for name in ["titlebutton_large","play_now_text"] {
                let bytes=client.scripts.source.cache.named_file(8,name,"")?;
                assets.extend_from_slice(&(bytes.len() as u32).to_be_bytes());assets.extend_from_slice(&bytes);
            }
            std::fs::write("work/w07-token-title-assets.bin", assets)?;
            println!("Token title runtime retry returns to Play; two actual-cache label layouts rendered");
        }
        println!("Synthetic token connection case{case}: acquisition/rotation/RSA flag or failure gate passed");
    }
    println!("5 actual-cache Connection flows pass owned native HTTP and game-socket endpoints: 4 token flows plus password retry; no external account authentication attempted");
    Ok(())
}
