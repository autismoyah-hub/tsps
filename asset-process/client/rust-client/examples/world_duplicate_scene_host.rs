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
    let key = "work/world-duplicate-scene-fixture.key";
    let password = "work/world-duplicate-scene-fixture.password";
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
    use gielinor_client::{buffer::Reader, game_packet, protocol::Packet};
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/world-duplicate-scene-views-jvm.bin"
    ));
    let count = r.u32()?;
    for case in 0..count {
        client.world_entities.clear();
        client.detached_world_entities.clear();
        client.active_world = 0;
        client.view_registry = Default::default();
        client.player_state = None;
        let id = r.u32()? as u16;
        let _order = r.u32()?;
        for _ in 0..r.u32()? {
            let opcode = r.u32()? as u16;
            let n = r.u32()? as usize;
            let payload = r.bytes(n)?.to_vec();
            client.apply(game_packet::decode(&Packet { opcode, payload })?.unwrap())?;
        }
        let expected = r.u32()? as usize;
        let direct = r.u32()? as usize;
        let nested = r.u32()? as usize;
        let _heights = [r.u32()?, r.u32()?];
        let remove_parent = r.u32()? as u16;
        live.paint(&mut client, 765, 503)?;
        for (parent, size) in [(0, direct), (7, nested)] {
            let generation = client.world_entities[&parent].entities[&id].spawn_generation;
            assert_eq!(
                live.instance_scene(id, generation).unwrap().width,
                size,
                "case{case} scene instance"
            );
        }
        assert_eq!(
            live.projectile_scenes()[&id].width,
            expected,
            "case{case} oldest registered scene before removal"
        );
        client.apply(
            game_packet::decode(&Packet {
                opcode: 47,
                payload: vec![0, remove_parent as u8, 0],
            })?
            .unwrap(),
        )?;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 122,
                payload: if remove_parent == 0 {
                    vec![1, 1, 0]
                } else {
                    vec![0]
                },
            })?
            .unwrap(),
        )?;
        let expected = r.u32()? as usize;
        let _height = r.u32()?;
        assert_ne!(r.u8()?, 0);
        let _retained_heights = [r.u32()?, r.u32()?];
        live.paint(&mut client, 765, 503)?;
        assert_eq!(
            live.projectile_scenes()[&id].width,
            expected,
            "case{case} surviving registered scene after removal"
        );
    }
    assert_eq!(r.remaining(), 0);
    drop(tx);
    server.join().unwrap();
    println!("{count} original duplicate scene registration lifetimes passed actual-cache native scene preparation");
    Ok(())
}
