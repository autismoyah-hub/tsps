//! Isolated-account login followed by a local graphics draw-phase replay.
//! No synthetic world state or interactions are sent to the server.
use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection};
use std::time::{Duration, Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let mut live = Connection::connect(
        &mut client,
        "127.0.0.1:43617",
        "rustclient239",
        "work/kotlin-world/test-account.password",
        "work/kotlin-world/.data/client.key",
    )?;
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        live.poll(&mut client)?;
        client.tick_actor_sequences()?;
        if live.world.is_some()
            && live.session.is_established()
            && !live.scripts_busy()
            && client
                .interfaces
                .widgets
                .values()
                .any(|w| w.content == 1337)
        {
            break;
        }
        assert!(Instant::now() < deadline, "login/scene timeout");
        std::thread::sleep(Duration::from_millis(20));
    }
    // Freeze network polling so server state cannot overwrite the local scenario.
    client.canvas = [765, 503];
    // This fixture isolates child picking from the root building's upper floors.
    client.preferences.device.roofs_hidden = true;
    // Settle the real login, then isolate one real-cache NPC in an empty root scene.
    live.paint(&mut client, 765, 503)?;
    client.player_state = None;
    client.world_entities.clear();
    client.npc_state = Default::default();

    use gielinor_client::{buffer::Reader, projectile::Request};
    client.world_spots.clear();
    client.projectiles.clear();
    client.scripted_projectiles = Default::default();
    client.scripted_events.clear();
    client.world_base = [0, 0];
    client.data.position = [0, 6, 6];
    client.input.pointer = [-100, -100];
    let scene = gielinor_client::scene::Scene::empty(0, 0)?;
    let mesh =
        gielinor_client::world_mesh::WorldMesh::build(&scene, &mut client.scripts.source.cache, 3)?;
    live.world = Some((scene, mesh));
    client.camera.locked = true;
    client.camera.eye = [832, -400, 256];
    client.camera.rendered_angles = [128, 0];
    client.camera.shake = Default::default();
    // Real cache supplies model/animation; the synthetic Java clock matrix is separate.
    let spot = 1u16;
    let sequence = client.spots[&(spot as u32)].sequence as u32;
    let sequences =
        gielinor_client::animation::Sequence::load_all(&mut client.scripts.source.cache)?;
    assert!(
        sequences[&sequence].cached_animation.is_none() && sequences[&sequence].lengths[0] >= 32
    );
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/projectile-draw-clock-jvm.bin"
    ));
    let cases = r.u32()?;
    assert_eq!(cases, 32);
    let len = r.u32()? as usize;
    r.bytes(len)?;
    let mut observations = 0;
    for case in 0..cases {
        let [_fill, delta, delay, plane, steps]: [i32; 5] =
            std::array::from_fn(|_| r.u32().unwrap() as i32);
        client.projectiles.clear();
        client.actor_cycle = 101 - delta;
        live.paint(&mut client, 765, 503)?;
        client.projectiles.push(
            Request {
                id: spot,
                source: [plane, 6, 6],
                target: [plane, 8, 8],
                heights: [48, 64],
                entities: [0, 0],
                start: 100 + delay,
                end: 108,
                slope: 32,
                offset: 40,
            }
            .start(0),
        );
        let mut elapsed = 0;
        for step in 0..steps {
            let [cycle, tx, tz]: [i32; 3] = std::array::from_fn(|_| r.u32().unwrap() as i32);
            let expected: [f64; 4] = std::array::from_fn(|_| {
                f64::from_be_bytes(r.bytes(8).unwrap().try_into().unwrap())
            });
            let moving = r.u8()? != 0;
            let present = r.u8()? != 0;
            let angles: [u32; 2] = [r.u32()?, r.u32()?];
            r.u32()?;
            r.u32()?;
            // The JVM contract matrix also has delta=0 with increasing cycles.
            // Native naturally has zero delta only without a simulation advance.
            if delta == 0 && step > 0 {
                continue;
            }
            client.actor_cycle = cycle + 1;
            if cycle >= 100 + delay && cycle <= 108 {
                elapsed += delta as u32;
            }
            if let Some(p) = client.projectiles.first_mut() {
                p.request.target = [plane, tx, tz];
            }
            live.paint(&mut client, 765, 503)?;
            assert_eq!(
                !client.projectiles.is_empty(),
                present,
                "case{case} step{step} lifetime"
            );
            if let Some(p) = client.projectiles.first() {
                for (actual, expected) in [
                    p.motion.position[0],
                    p.motion.position[1],
                    p.motion.position[2],
                    p.motion.vertical_speed,
                ]
                .into_iter()
                .zip(expected)
                {
                    assert!(
                        (actual - expected).abs() / expected.abs().max(1.0) < 1e-10,
                        "case{case} step{step}: {actual} != {expected}"
                    );
                }
                assert_eq!(p.motion.moving, moving, "case{case} step{step}");
                assert_eq!([p.motion.yaw as u32, p.motion.pitch as u32], angles);
                assert_eq!(
                    (p.clock.frame, p.clock.elapsed),
                    (0, elapsed),
                    "case{case} step{step} real-cache clock"
                );
            }
            observations += 1;
        }
    }
    assert_eq!(r.remaining(), 0);
    assert_eq!(observations, 200);
    println!("200 actual live-paint projectile draw observations match original ef.cv; cache model, changing target, deltas 0/1/2/5, delay, lifetime and planes. Scene occupancy covered by JVM/helper fixture separately.");
    Ok(())
}
