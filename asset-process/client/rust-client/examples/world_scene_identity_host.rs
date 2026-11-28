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

fn pump_packet(
    tx: &mpsc::Sender<(u16, Vec<u8>)>,
    live: &mut Connection,
    client: &mut ClientState,
    opcode: u16,
    packet: Vec<u8>,
) -> Result<()> {
    let tick = client.server_tick;
    tx.send((opcode, packet)).unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    while client.server_tick == tick {
        live.poll(client)?;
        assert!(live.logged_out().is_none());
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}

fn pump(
    tx: &mpsc::Sender<(u16, Vec<u8>)>,
    live: &mut Connection,
    client: &mut ClientState,
    packet: Vec<u8>,
) -> Result<()> {
    pump_packet(tx, live, client, 122, packet)
}

fn main() -> Result<()> {
    use gielinor_client::buffer::Reader;
    let bytes = if let Some(path) = std::env::args().nth(1) {
        std::fs::read(path)?
    } else {
        include_bytes!("../tests/fixtures/world-scene-identity-jvm.bin").to_vec()
    };
    let mut r = Reader::new(&bytes);
    let count = r.u32()?;
    assert!(count > 0 && count <= 1024);
    let mut cases = Vec::new();
    for _ in 0..count {
        let old_size = r.u32()? as usize;
        let size = r.u32()? as usize;
        let mode = r.u32()?;
        let height = r.u32()?;
        let n = r.u32()? as usize;
        let initial = r.bytes(n)?.to_vec();
        let n = r.u32()? as usize;
        let replacement = r.bytes(n)?.to_vec();
        let expected: [i32; 5] = std::array::from_fn(|_| r.u32().unwrap() as i32);
        cases.push((old_size, size, mode, height, initial, replacement, expected));
    }
    assert_eq!(r.remaining(), 0);
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    let key = "work/world-scene-identity-fixture.key";
    let password = "work/world-scene-identity-fixture.password";
    std::fs::write(key, format!("Modulus: {}\n", modulus.to_str_radix(16)))?;
    std::fs::write(password, "fixture")?;
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let (tx, rx) = mpsc::channel::<(u16, Vec<u8>)>();
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
        while let Ok((opcode, packet)) = rx.recv_timeout(Duration::from_secs(45)) {
            let create = gielinor_client::protocol239::SERVER_PACKETS
                .iter()
                .find(|s| s.opcode as u16 == opcode)
                .unwrap();
            let mut bytes =
                gielinor_client::protocol::encode(create, &packet, &mut cipher).unwrap();
            bytes.extend(gielinor_client::protocol::encode(tick, &[], &mut cipher).unwrap());
            socket.write_all(&bytes).unwrap();
        }
    });
    let mut live =
        Connection::connect(&mut client, &address.to_string(), "fixture", password, key)?;
    let deadline = Instant::now() + Duration::from_secs(45);
    while !live.session.is_established() || client.server_tick < 3 {
        live.poll(&mut client)?;
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(1));
    }
    let scene = gielinor_client::scene::Scene::empty(0, 0)?;
    let mesh =
        gielinor_client::world_mesh::WorldMesh::build(&scene, &mut client.scripts.source.cache, 3)?;
    live.world = Some((scene, mesh));
    client.world_base = [0, 0];
    client.data.position = [0, 6, 6];
    client.world_entities.clear();
    client.active_world = 0;
    client
        .world_entities
        .insert(0, gielinor_client::world_entity::State::default());
    for (case, (old_size, size, mode, _height, initial, replacement, expected)) in
        cases.into_iter().enumerate()
    {
        pump(&tx, &mut live, &mut client, initial.clone())?;
        // Represent an already rebuilt old scene before the ID is replaced.
        client
            .world_entities
            .get_mut(&0)
            .unwrap()
            .entities
            .get_mut(&7)
            .unwrap()
            .rebuild = Some(gielinor_client::rebuild::WorldEntity {
            base: [123, 456],
            dimensions: [old_size, old_size],
            chunks: vec![None; 4 * (old_size / 8) * (old_size / 8)],
            regions: vec![],
        });
        live.paint(&mut client, 765, 503)?;
        {
            let scenes = live.projectile_scenes();
            let old = scenes.get(&7).expect("old scene prepared");
            assert_eq!([old.width, old.height], [old_size, old_size]);
            assert_eq!([old.base_x, old.base_y], [123, 456]);
        }
        let old_generation = client.world_entities[&0].entities[&7].spawn_generation;
        pump(&tx, &mut live, &mut client, replacement)?;
        assert!(
            !live.projectile_scenes().contains_key(&7),
            "case{case}: stale scene after packet dispatch"
        );
        // No intervening draw or simulation tick: this is the same-ID lifetime boundary.
        live.paint(&mut client, 765, 503)?;
        {
            let scenes = live.projectile_scenes();
            let fresh = scenes
                .get(&7)
                .expect("fresh empty scene prepared before rebuild");
            assert_eq!(
                [
                    fresh.width as i32,
                    fresh.height as i32,
                    fresh.base_x,
                    fresh.base_y,
                    fresh.elevation(0, 128, 128).unwrap_or(-1)
                ],
                expected,
                "replacement case{case} mode{mode}"
            );
            assert_eq!([fresh.width, fresh.height], [size, size]);
            assert!(fresh.objects.is_empty());
        }
        let fresh = &client.world_entities[&0].entities[&7];
        assert!(fresh.rebuild.is_none());
        assert_ne!(
            fresh.spawn_generation, old_generation,
            "replacement generation case{case}"
        );
        if size <= 104 {
            // A pending terrain rebuild preserves the live scene until preparation.
            let spawn = client.world_entities[&0].entities[&7].spawn_generation;
            pump_packet(&tx, &mut live, &mut client, 47, vec![0, 7, 0])?;
            let bits = 4 * (size / 8) * (size / 8);
            let mut rebuild = vec![0; 6 + (bits + 7) / 8];
            rebuild[1] = 53;
            rebuild[3] = 71;
            pump_packet(&tx, &mut live, &mut client, 109, rebuild)?;
            pump_packet(&tx, &mut live, &mut client, 47, vec![0, 0, 0])?;
            pump(&tx, &mut live, &mut client, vec![1, 1, 0])?;
            {
                let scenes = live.projectile_scenes();
                let retained = scenes.get(&7).expect("pending rebuild retains scene");
                assert_eq!([retained.base_x, retained.base_y], [0, 0]);
                assert_eq!(
                    client.world_entities[&0].entities[&7].spawn_generation,
                    spawn
                );
            }
            live.paint(&mut client, 765, 503)?;
            {
                let scenes = live.projectile_scenes();
                let rebuilt = scenes.get(&7).unwrap();
                assert_eq!([rebuilt.base_x, rebuilt.base_y], [53, 71]);
            }
        }
        pump(&tx, &mut live, &mut client, vec![0])?;
        live.paint(&mut client, 765, 503)?;
        assert!(!live.projectile_scenes().contains_key(&7));
        pump(&tx, &mut live, &mut client, initial)?;
        live.paint(&mut client, 765, 503)?;
        let scenes = live.projectile_scenes();
        let fresh = scenes
            .get(&7)
            .expect("fresh spawn scene available before rebuild");
        assert_eq!([fresh.width, fresh.height], [old_size, old_size]);
        assert_eq!([fresh.base_x, fresh.base_y], [0, 0]);
        assert!(fresh.objects.is_empty());
    }
    drop(tx);
    server.join().unwrap();
    live.session.close();
    println!("{count} original packet122 scene replacements and {count} fresh spawns match actual Connection.poll and cache preparation; old cached scenes are invalidated before the next paint; supported pending rebuilds preserve the live scene until preparation");
    Ok(())
}
