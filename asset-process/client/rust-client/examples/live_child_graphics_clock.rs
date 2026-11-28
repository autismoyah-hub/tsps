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
        assert!(Instant::now() < deadline, "login/scene timeout: established={} closed={} world={} scripts_busy={} packets={} title={:?}", live.session.is_established(), live.session.is_closed(), live.world.is_some(), live.scripts_busy(), live.packets, live.title.as_ref().map(|title| (&title.screen, &title.lines)));
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
    use gielinor_client::{
        rebuild::WorldEntity,
        world_entity::{Definition, Entity, Pose, State},
    };
    let mut definition = Definition::decode(0, &[0])?;
    definition.bounds = [0, 0, 13312, 13312];
    definition.picking_type = 1;
    let mut entity = Entity {
        dimensions: [104, 104],
        owner_type: 0,
        definition: Some(definition),
        plane: 0,
        pose: Pose {
            x: 6656,
            y: 0,
            z: 6656,
            angle: 0,
        },
        rebuild: Some(WorldEntity {
            base: [0, 0],
            dimensions: [104, 104],
            chunks: vec![None; 4 * 13 * 13],
            regions: vec![],
        }),
        ..Entity::default()
    };
    let sequences =
        gielinor_client::animation::Sequence::load_all(&mut client.scripts.source.cache)?;
    let mut choices = client
        .spots
        .values()
        .filter_map(|d| {
            let seq = sequences.get(&(d.sequence as u32))?;
            (seq.cached_animation.is_none() && seq.lengths.first().is_some_and(|&n| n >= 32))
                .then_some((d.id, seq.id))
        })
        .collect::<Vec<_>>();
    choices.sort();
    let (spot, sequence) = choices[0];
    entity.world_spots.push(
        gielinor_client::world_spot::Request {
            id: spot as u16,
            height: 0,
            delay: 0,
            position: [0, 50, 50],
        }
        .start(0),
    );
    let mut state = State::default();
    state.visible.push(7);
    state.entities.insert(7, entity);
    client.world_entities.insert(0, state);
    client.view_registry.insert(7);
    client.world_spots.clear();
    client.projectiles.clear();
    client.world_base = [0, 0];
    client.data.position = [0, 50, 50];
    client.input.pointer = [-100, -100];
    let scene = gielinor_client::scene::Scene::empty(0, 0)?;
    let mesh =
        gielinor_client::world_mesh::WorldMesh::build(&scene, &mut client.scripts.source.cache, 3)?;
    live.world = Some((scene, mesh));
    client.camera.locked = true;
    client.camera.eye = [6464, -60, 2240];
    client.camera.rendered_angles = [0, 0];
    client.camera.shake = Default::default();
    // All scenarios use the actual paint path and its child scene admission/overlap rules.
    // Earlier empty child scenes reserve the five parent slots even without a rebuild mesh.
    for id in 8..13 {
        let mut e = Entity::default();
        e.pose = client.world_entities[&0].entities[&7].pose;
        e.owner_type = 0;
        e.definition = client.world_entities[&0].entities[&7].definition.clone();
        client
            .world_entities
            .get_mut(&0)
            .unwrap()
            .entities
            .insert(id, e);
    }
    for (fill, hidden, ticks, expected) in [
        (5, false, 3, 0),
        (0, false, 2, 2),
        (1, true, 4, 2),
        (0, false, 3, 5),
        (0, false, 0, 5),
    ] {
        let root = client.world_entities.get_mut(&0).unwrap();
        root.visible = (8..8 + fill).chain(std::iter::once(7)).collect();
        root.entities.get_mut(&7).unwrap().owner_type = if hidden { 1 } else { 0 };
        for _ in 0..ticks {
            client.tick_actor_sequences()?;
        }
        let before = client.world_entities[&0].entities[&7].world_spots[0]
            .animation
            .clock
            .elapsed;
        live.paint(&mut client, 765, 503)?;
        let clock = &client.world_entities[&0].entities[&7].world_spots[0]
            .animation
            .clock;
        assert_eq!(
            (clock.frame, clock.elapsed),
            (0, expected),
            "fill{fill} hidden{hidden} ticks{ticks} before{before}"
        );
    }
    println!("Actual live paint child graphic {spot}/sequence {sequence}: denied and hidden draws freeze; admitted delta resumes without catch-up; zero-tick redraw does not advance");
    Ok(())
}
