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
    // Keep upper root floors out of this draw-order scenario.
    client.preferences.device.roofs_hidden = true;
    // Settle the real login before constructing local graphics scenarios.
    live.paint(&mut client, 765, 503)?;
    use gielinor_client::{
        animation::Sequence,
        animation_runtime::Clock,
        buffer::Reader,
        world_entity::{Definition, Entity, Pose, State},
        world_spot,
    };
    let sequences = Sequence::load_all(&mut client.scripts.source.cache)?;
    let mut choices = client
        .spots
        .values()
        .flat_map(|spot| {
            let seq = sequences.get(&(spot.sequence as u32));
            seq.into_iter().flat_map(move |seq| {
                seq.sounds.iter().filter_map(move |(&frame, sounds)| {
                    (frame > 0
                        && seq.cached_animation.is_none()
                        && (frame as usize) < seq.frames.len()
                        && sounds.len() == 1
                        && sounds[0].range > 0
                        && sounds[0].loops > 0)
                        .then(|| (spot.id, seq.id, frame, sounds[0].id))
                })
            })
        })
        .collect::<Vec<_>>();
    choices.sort();
    let (spot, sequence, frame, sound) = choices[0];
    let effect = || {
        let mut e = world_spot::Request {
            id: spot as u16,
            height: 0,
            delay: 0,
            position: [0, 6, 6],
        }
        .start(0);
        e.animation.clock = Clock {
            frame: frame as usize - 1,
            elapsed: sequences[&sequence].lengths[frame as usize - 1] as u32,
            ..Default::default()
        };
        e
    };
    client.npc_state = Default::default();
    client.projectiles.clear();
    client.world_base = [0, 0];
    client.data.position = [0, 6, 6];
    let players = client.player_state.as_mut().unwrap();
    players.high.clear();
    players.players.fill(None);
    players.views.clear();
    let scene = gielinor_client::scene::Scene::empty(0, 0)?;
    let mesh =
        gielinor_client::world_mesh::WorldMesh::build(&scene, &mut client.scripts.source.cache, 3)?;
    live.world = Some((scene, mesh));
    client.camera.locked = true;
    client.camera.eye = [832, -200, 256];
    client.camera.rendered_angles = [128, 0];
    client.camera.shake = Default::default();
    client.input.pointer = [-100, -100];
    let mut r = Reader::new(include_bytes!("../tests/fixtures/root-child-draw-jvm.bin"));
    assert_eq!(r.u32()?, 162);
    for case in 0..162 {
        let order: [u32; 3] = std::array::from_fn(|_| r.u32().unwrap());
        let local = r.u32()? as u16;
        let limit = r.u32()? as i32;
        let nested_kind = r.u32()? as u8;
        let n = r.u32()?;
        let expected = (0..n)
            .map(|_| Ok((sound as u32, Some(r.u32()? as u16))))
            .collect::<gielinor_client::Result<Vec<_>>>()?;
        assert_eq!((r.u32()?, r.u32()?), (0, 0), "original nested clock");
        client.audio = Default::default();
        client.world_entities.clear();
        client.world_spots = vec![effect()];
        client.player_state.as_mut().unwrap().local_view = local;
        let mut stack = gielinor_client::vm::Stacks::default();
        stack.push_int(limit)?;
        client.world_entity_limit.execute(7900, &mut stack)?;
        let mut root = State::default();
        for i in order {
            let id = [7, 23, 9][i as usize];
            let mut e = Entity::default();
            e.dimensions = [16, 16];
            e.owner_type = [0, 1, 2][i as usize];
            e.definition = Some(Definition::decode(0, &[0])?);
            e.pose = Pose {
                x: 400 + i as i32 * 256,
                y: 0,
                z: 400,
                angle: 0,
            };
            e.rebuild = Some(gielinor_client::rebuild::WorldEntity {
                base: [0, 0],
                dimensions: [16, 16],
                chunks: vec![None; 16],
                regions: vec![],
            });
            e.world_spots.push(effect());
            root.visible.push(id);
            root.entities.insert(id, e);
            client.view_registry.insert(id);
        }
        client.world_entities.insert(0, root);
        let mut nested = Entity::default();
        nested.dimensions = [16, 16];
        nested.owner_type = nested_kind;
        nested.definition = Some(Definition::decode(0, &[0])?);
        nested.pose = Pose {
            x: 832,
            y: 0,
            z: 832,
            angle: 0,
        };
        nested.rebuild = Some(gielinor_client::rebuild::WorldEntity {
            base: [0, 0],
            dimensions: [16, 16],
            chunks: vec![None; 16],
            regions: vec![],
        });
        nested.world_spots.push(effect());
        let before = (
            nested.world_spots[0].animation.clock.frame,
            nested.world_spots[0].animation.clock.elapsed,
        );
        let mut child = State::default();
        child.visible.push(31);
        child.entities.insert(31, nested);
        client.world_entities.insert(7, child);
        client.view_registry.insert(31);

        client.tick_actor_sequences()?;
        live.paint(&mut client, 765, 503)?;
        let nested = &client.world_entities[&7].entities[&31].world_spots[0]
            .animation
            .clock;
        assert_eq!(
            (nested.frame, nested.elapsed),
            before,
            "case{case}: nested graphic must not tick in root draw"
        );

        assert_eq!(
            client.audio.pending_effects().collect::<Vec<_>>(),
            expected,
            "original draw phase case{case}"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("162 original root-owned child draw orders and untouched nested clocks match live paint with cache graphic {spot}/sequence {sequence}/frame {frame}/sound {sound}");
    Ok(())
}
