//! Isolated-account login followed by a local world-scene lifetime replay.
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
    // Keep upper root floors out of this draw-order scenario.
    client.preferences.device.roofs_hidden = true;
    // Settle the real login before constructing local world-scene scenarios.
    live.paint(&mut client, 765, 503)?;
    use gielinor_client::{buffer::Reader, game_packet, protocol::Packet, world_entity::State};
    client.world_entities.clear();
    client.world_entities.insert(0, State::default());
    client.active_world = 0;
    client.player_state.as_mut().unwrap().local_view = 0;
    let bytes = if let Some(path) = std::env::args().nth(1) {
        std::fs::read(path)?
    } else {
        include_bytes!("../tests/fixtures/world-scene-identity-jvm.bin").to_vec()
    };
    let mut r = Reader::new(&bytes);
    let count = r.u32()?;
    assert!(count > 0 && count <= 1024);
    for case in 0..count {
        let old_size = r.u32()? as usize;
        let size = r.u32()? as usize;
        let mode = r.u32()?;
        let _old_height = r.u32()?;
        let n = r.u32()? as usize;
        let initial = r.bytes(n)?.to_vec();
        let n = r.u32()? as usize;
        let replacement = r.bytes(n)?.to_vec();
        let expected: [i32; 5] = std::array::from_fn(|_| r.u32().unwrap() as i32);
        client.apply(
            game_packet::decode(&Packet {
                opcode: 122,
                payload: initial.clone(),
            })?
            .unwrap(),
        )?;
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
        client.apply(
            game_packet::decode(&Packet {
                opcode: 122,
                payload: replacement,
            })?
            .unwrap(),
        )?;
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
        client.apply(
            game_packet::decode(&Packet {
                opcode: 122,
                payload: vec![0],
            })?
            .unwrap(),
        )?;
        live.paint(&mut client, 765, 503)?;
        assert!(!live.projectile_scenes().contains_key(&7));
        client.apply(
            game_packet::decode(&Packet {
                opcode: 122,
                payload: initial,
            })?
            .unwrap(),
        )?;
        live.paint(&mut client, 765, 503)?;
        let scenes = live.projectile_scenes();
        let fresh = scenes
            .get(&7)
            .expect("fresh spawn scene available before rebuild");
        assert_eq!([fresh.width, fresh.height], [old_size, old_size]);
        assert_eq!([fresh.base_x, fresh.base_y], [0, 0]);
        assert!(fresh.objects.is_empty());
    }
    assert_eq!(r.remaining(), 0);
    println!("{count} original packet122 same-ID replacement scenes and {count} fresh spawn scenes match actual-cache live paint before rebuild; same-cycle old scene never survives replacement");
    Ok(())
}
