use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    scene::Scene,
    scripted_projectile::{Definition, Endpoint, Event},
    world_actors::Actors,
    Result,
};
fn main() -> Result<()> {
    let path = std::env::args().nth(1).expect("cache path");
    let mut cache=if let Some(address)=std::env::args().nth(2) {
        DiskCache::open_remote(path,64 << 20,address.parse().map_err(|_|gielinor_client::Error::Invalid("JS5 endpoint"))?)?
    } else {DiskCache::open(path,64 << 20)?};
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let scene = Scene::empty(0, 0)?;
    client.data.position = [0, 50, 50];
    client.actor_cycle = 100;
    assert!(client.scripts.source.event_script(86, 0, -1)?.is_none());
    let definition = Definition {
        spot: 0,
        source: Endpoint {
            tile: [0, 50, 50],
            offset: [12, 80, -20],
            entity: 0,
        },
        target: Endpoint {
            tile: [0, 52, 53],
            offset: [0, 40, 0],
            entity: 0,
        },
        start: 0,
        end: 8,
        curves: [0; 3],
    };
    for key in [65, 1, 64] {
        client.scripted_events.push_back((
            100,
            Event::Create {
                key,
                definition: definition.clone(),
            },
        ));
    }
    let mut actors = Actors::load(&mut client.scripts.source.cache)?;
    client.tick_actor_sequences()?;
    client.tick_projectiles(&scene)?;
    assert_eq!(client.scripted_projectiles.ordered_keys(), vec![64, 65, 1]);
    for effect in client.scripted_projectiles.effects.values() {
        assert!(effect.visible);
        assert_eq!(effect.motion.position, [6476.0, -80.0, 6444.0]);
    }
    let (triangles, pending) = actors.triangles(&mut client, &scene, &Default::default())?;
    assert!(pending.is_empty(), "{pending:?}");
    assert!(!triangles.is_empty());
    client.scripted_events.push_back((
        101,
        Event::Update {
            key: 64,
            target: Endpoint {
                tile: [-1; 3],
                ..Default::default()
            },
            pause: Some(3),
            stop: false,
        },
    ));
    client.tick_actor_sequences()?;
    client.tick_projectiles(&scene)?;
    assert_eq!(client.scripted_projectiles.definition.end, 109);
    for _ in 0..20 {
        client.tick_actor_sequences()?;
        client.tick_projectiles(&scene)?;
    }
    assert!(client.scripted_projectiles.effects.is_empty());
    let (expired, pending) = actors.triangles(&mut client, &scene, &Default::default())?;
    assert!(expired.is_empty());
    assert!(pending.is_empty());
    println!(
        "Scripted projectile integration: {} rendered triangles; shared pause and expiry passed",
        triangles.len()
    );

    // Region bases must not change the float trajectory or root-scene placement.
    client.world_base = [3200, 6400];
    let shifted_scene = Scene::empty(3200, 6400)?;
    client.actor_cycle = 200;
    let mut shifted = definition.clone();
    for endpoint in [&mut shifted.source, &mut shifted.target] {
        endpoint.tile[1] += 3200;
        endpoint.tile[2] += 6400;
    }
    client.scripted_events.push_back((
        200,
        Event::Create {
            key: 7,
            definition: shifted,
        },
    ));
    client.tick_actor_sequences()?;
    client.tick_projectiles(&shifted_scene)?;
    assert_eq!(
        client.scripted_projectiles.effects[&7].motion.position,
        [6476.0, -80.0, 6444.0]
    );
    let (shifted_mesh, pending) =
        actors.triangles(&mut client, &shifted_scene, &Default::default())?;
    assert!(pending.is_empty());
    assert_eq!(shifted_mesh.len() * 3, triangles.len());
    for (a, b) in shifted_mesh.iter().zip(&triangles) {
        assert_eq!(a.vertices, b.vertices, "root-base invariant geometry");
    }
    client.scripted_projectiles = Default::default();

    // A real kind-12 child zone record must preserve fractional transformed source coordinates.
    use gielinor_client::{
        game_packet::Update,
        world_entity::{Entity, Pose, State},
        writer::Writer,
        zone::{Packet, Record},
    };
    let mut entity = Entity::default();
    entity.dimensions = [24, 40];
    entity.pose = Pose {
        x: 2048,
        y: 0,
        z: 3072,
        angle: 177,
    };
    entity.rebuild = Some(gielinor_client::rebuild::WorldEntity {
        base: [8000, 8000],
        dimensions: [24, 40],
        chunks: vec![None; 60],
        regions: Vec::new(),
    });
    let transformed = entity.transform_horizontal([13 * 128 + 64, 21 * 128 + 64]);
    assert_ne!(transformed[0].fract(), 0.0);
    let mut worlds = State::default();
    worlds.visible.push(7);
    worlds.entities.insert(7, entity);
    client.world_entities.insert(0, worlds);
    client.view_registry.insert(7);
    client.apply(Update::ActiveWorld { id: 7, plane: 0 })?;
    let mut wire = Writer::new();
    wire.u16_le(12);
    wire.u16(0);
    wire.u16(0);
    wire.u32_middle((8016 << 14) | 8022);
    wire.u16_add(0, false);
    wire.u16(80);
    wire.u16_le(0);
    wire.bytes(&[0; 3]);
    wire.u16(8);
    wire.u16_le(8);
    wire.u8(0);
    wire.u16(0);
    wire.u16_le(0);
    wire.u16_add((-20i16) as u16, true);
    wire.u16_le(40);
    wire.bytes(&[0; 3]);
    wire.u16_le(0);
    client.apply(Update::Zone(Packet {
        absolute: None,
        origin: Some([0, 13, 21]),
        clear: false,
        records: vec![Record::Pending {
            kind: 12,
            bytes: wire.into_vec(),
        }],
    }))?;
    assert!(client.world_entities[&0].entities[&7]
        .zone
        .scripted_projectiles
        .is_empty());
    assert_eq!(client.scripted_events.len(), 1);
    client.tick_actor_sequences()?;
    let child_scene = Scene::empty(8000, 8000)?;
    let views = std::collections::HashMap::from([(7, &child_scene)]);
    client.tick_projectiles_in_views(&shifted_scene, &views)?;
    let effect = &client.scripted_projectiles.effects[&8];
    assert_eq!(
        effect.motion.origin,
        [transformed[0] + 12.0, -80.0, transformed[2] - 20.0]
    );
    let (child_mesh, pending) =
        actors.triangles(&mut client, &shifted_scene, &Default::default())?;
    assert!(pending.is_empty());
    assert!(!child_mesh.is_empty());
    for _ in 0..12 {
        client.tick_actor_sequences()?;
        client.tick_projectiles_in_views(&shifted_scene, &views)?;
    }
    assert!(client.scripted_projectiles.effects.is_empty());
    println!("Scripted world integration: translated root base, fractional child packet source, {} rendered triangles and expiry pass", child_mesh.len());
    Ok(())
}
