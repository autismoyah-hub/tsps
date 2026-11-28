//! Dedicated-account integration probe. Never logs credentials or account tokens.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    login::{self, Authentication, Credentials, Platform, Request},
    protocol239::SERVER_PACKETS,
    rsa::RsaPublicKey,
    session::{Event, Session},
};
use std::{
    fs,
    io::Write,
    time::{Duration, Instant},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 7 {
        return Err("expected CACHE PUBLIC_KEY ADDRESS USERNAME PASSWORD_FILE CAPTURE".into());
    }
    let mut cache = DiskCache::open(&args[1], 64 << 20)?;
    let crcs = cache.login_crcs()?;
    let public = fs::read_to_string(&args[2])?;
    let modulus = public
        .lines()
        .find_map(|s| s.strip_prefix("Modulus: "))
        .ok_or("public modulus missing")?;
    let key = RsaPublicKey::from_hex(modulus.trim(), 65537)?;
    let secret = fs::read_to_string(&args[5])?;
    let platform = Platform {
        os: 1,
        is_64_bit: true,
        java_vendor: 4,
        java_version: [21, 0, 0],
        memory_mb: 512,
        processors: 2,
        client_name: "Rust239".into(),
        ..Default::default()
    };
    let request = Request {
        username: &args[4],
        credentials: Credentials::Login {
            authentication: Authentication::Password,
            secret: secret.trim(),
            token: false,
        },
        client_build: i32::MAX as u32,
        client_type: 1,
        platform_type: 0,
        resizable: false,
        low_detail: false,
        width: 765,
        height: 503,
        uuid: [0; 24],
        site_settings: "",
        affiliate: 0,
        deep_links: &[],
        platform: &platform,
        archive_crcs: &crcs,
    };
    let mut client = ClientState::load(cache)?;
    let mut session = Session::connect(args[3].parse()?, Duration::from_secs(120))?;
    let mut capture = fs::File::create(&args[6])?;
    capture.write_all(b"R239CAP1")?;
    let start = Instant::now();
    let mut tick = Instant::now();
    let mut keepalive = Instant::now();
    let mut local = None;
    let mut packets = 0;
    let mut unsupported = std::collections::BTreeMap::<u16, usize>::new();
    let mut failures = std::collections::BTreeMap::<String, usize>::new();
    while start.elapsed() < Duration::from_secs(120) && !session.is_closed() {
        for event in session.poll()? {
            match event {
                Event::Login(login::Event::ServerSeed(_)) => {
                    session.submit(&request, &key)?;
                    println!("Submitted dedicated-account login");
                }
                Event::Login(login::Event::Success(account)) => {
                    local = Some(account.player_index);
                    client.data.account_members = account.members;
                    println!("Authenticated; local index {}", account.player_index);
                }
                Event::Login(login::Event::Rejected(code)) => {
                    return Err(format!("Login rejected: {code}").into())
                }
                Event::Login(login::Event::DateOfBirth { in_client }) => {
                    return Err(format!("Login requires date of birth, in-client={in_client}").into())
                }
                Event::Login(login::Event::ProofOfWork(_)) => {
                    println!("Received proof-of-work challenge")
                }
                Event::ProofCompleted(_) => println!("Completed proof-of-work"),
                Event::Login(login::Event::Tokens(_)) => println!("Received login extension"),
                Event::Login(login::Event::Message(lines)) => {
                    return Err(format!("Login message: {lines:?}").into())
                }
                Event::Packet(packet) => {
                    packets += 1;
                    capture.write_all(&packet.opcode.to_be_bytes())?;
                    capture.write_all(&(packet.payload.len() as u32).to_be_bytes())?;
                    capture.write_all(&packet.payload)?;
                    let spec = SERVER_PACKETS
                        .iter()
                        .find(|s| s.opcode as u16 == packet.opcode)
                        .unwrap();
                    if spec.name == "REBUILD_NORMAL_V2" {
                        let index = if client.player_state.is_none() {
                            local
                        } else {
                            None
                        };
                        let rebuild =
                            gielinor_client::rebuild::Normal::decode(&packet.payload, index)?;
                        if let Some(index) = index {
                            client.initialize_players(&packet.payload, index)?;
                        }
                        client.world_base(rebuild.base());
                        println!(
                            "World rebuild: base {:?}, {} regions",
                            rebuild.base(),
                            rebuild.regions().len()
                        );
                    } else if spec.name == "REBUILD_REGION_V2" {
                        let rebuild = gielinor_client::rebuild::Instance::decode(&packet.payload)?;
                        client.world_base(rebuild.base());
                        println!(
                            "Instance rebuild: base {:?}, {} regions",
                            rebuild.base(),
                            rebuild.regions.len()
                        );
                    } else {
                        match gielinor_client::game_packet::decode(&packet) {
                            Ok(Some(update)) => {
                                if let Err(e) = client.apply(update) {
                                    *failures.entry(format!("{}: {e}", spec.name)).or_default() +=
                                        1;
                                }
                            }
                            Ok(None) => *unsupported.entry(packet.opcode).or_default() += 1,
                            Err(e) => {
                                *failures.entry(format!("{}: {e}", spec.name)).or_default() += 1;
                            }
                        }
                    }
                }
                Event::Disconnected => println!("Server disconnected"),
                _ => {}
            }
        }
        if session.is_established() {
            if keepalive.elapsed() >= Duration::from_secs(1) {
                session.send(89, &[])?;
                keepalive = Instant::now();
            }
            while let Some(event) = client.next_interface_event()? {
                if let Err(e) = client.run_event(event) {
                    *failures
                        .entry(format!("Interface on-load: {e}"))
                        .or_default() += 1;
                }
            }
            if tick.elapsed() >= Duration::from_millis(20) {
                if let Err(e) = client.tick_interfaces(false) {
                    *failures.entry(format!("Interface tick: {e}")).or_default() += 1;
                }
                while let Some(e) = client.script_failures.pop_front() {
                    *failures
                        .entry(format!("Interface listener: {e}"))
                        .or_default() += 1;
                }
                client.cycle = client.cycle.wrapping_add(1);
                client.input.begin_cycle();
                tick = Instant::now();
            }
            while let Some(event) = client.outbound.pop_event() {
                session.send_event(event)?;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    session.close();
    capture.flush()?;
    println!(
        "Received {packets} packets; players={}, NPCs={}, root={:?}",
        client.player_state.as_ref().map_or(0, |s| s.high.len()),
        client.npc_state.indices.len(),
        client.interfaces.root
    );
    for (opcode, count) in unsupported {
        let name = SERVER_PACKETS
            .iter()
            .find(|s| s.opcode as u16 == opcode)
            .unwrap()
            .name;
        println!("Unhandled {name} ({opcode}): {count}");
    }
    for (error, count) in failures {
        println!("Failed {error}: {count}");
    }
    if local.is_none() {
        return Err("No authenticated session".into());
    }
    Ok(())
}
