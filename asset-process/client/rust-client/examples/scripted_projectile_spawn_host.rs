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
    use gielinor_client::buffer::Reader;
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/scripted-projectile-spawn-packet-jvm.bin"
    ));
    let count = r.u32()?;
    assert_eq!(count, 32);
    let n = r.u32()? as usize;
    let initial = r.bytes(n)?.to_vec();
    let mut cases = Vec::new();
    for _ in 0..count {
        let inputs: [i32; 5] = std::array::from_fn(|_| r.u32().unwrap() as i32);
        let n = r.u32()? as usize;
        let packet = r.bytes(n)?.to_vec();
        let expected: [f32; 6] = std::array::from_fn(|_| f32::from_bits(r.u32().unwrap()));
        cases.push((inputs, packet, expected));
    }
    assert_eq!(r.remaining(), 0);
    let modulus: num_bigint::BigInt = (num_bigint::BigInt::from(1) << 521usize) - 1;
    let key = "work/scripted-spawn-fixture.key";
    let password = "work/scripted-spawn-fixture.password";
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
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
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
        let create = gielinor_client::protocol239::SERVER_PACKETS
            .iter()
            .find(|s| s.opcode == 105)
            .unwrap();
        for _ in 0..3 {
            w.bytes(&gielinor_client::protocol::encode(tick, &[], &mut cipher).unwrap());
        }
        socket.write_all(w.as_slice()).unwrap();
        while let Ok(packet) = rx.recv_timeout(Duration::from_secs(45)) {
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
    for (case, ([x, z, dx, dz, _height], packet, expected)) in cases.into_iter().enumerate() {
        client.actor_cycle = 100;
        client.scripted_projectiles = Default::default();
        client.scripted_events.clear();
        client.npc_state = Default::default();
        client
            .npc_state
            .decode(&initial, false, [0, 0], &client.npcs)?;
        client.npc_state.npcs.get_mut(&1).unwrap().path.position = [x, z];
        let tick = client.server_tick;
        tx.send(packet).unwrap();
        let deadline = Instant::now() + Duration::from_secs(20);
        while client.server_tick == tick {
            live.poll(&mut client)?;
            assert!(live.logged_out().is_none(), "packet case{case} logout");
            assert!(Instant::now() < deadline, "case{case} timeout");
            thread::sleep(Duration::from_millis(1));
        }
        assert!(
            client.scripted_events.is_empty(),
            "case{case}: creation deferred past packet boundary"
        );
        assert!(
            client.scripted_projectiles.effects.contains_key(&0),
            "case{case}: missing packet-time effect"
        );
        let effect = &client.scripted_projectiles.effects[&0];
        assert_eq!(
            effect
                .motion
                .position
                .into_iter()
                .chain(effect.motion.origin)
                .collect::<Vec<_>>(),
            expected,
            "case{case} packet snapshot"
        );
        client.npc_state.npcs.get_mut(&1).unwrap().path.position = [x + dx, z + dz];
        client.actor_cycle = 101;
        client.draw_projectiles_in_views(
            &live.world.as_ref().unwrap().0,
            &Default::default(),
            1,
        )?;
        assert_eq!(
            client.scripted_projectiles.effects[&0].motion.origin,
            expected[3..],
            "case{case} source moved before draw"
        );
    }
    drop(tx);
    server.join().unwrap();
    live.session.close();
    println!("32 original packet105 source snapshots match actual Connection::poll with cache definitions; source actor movement before draw preserves creation origin");
    Ok(())
}
