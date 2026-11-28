#[path = "../tests/support/duplicate_observer.rs"]
mod fixture;
use gielinor_client::{
    actor_update::{Face, FaceTarget},
    client_state::ClientState,
    disk_cache::DiskCache,
    game_packet,
    protocol::Packet,
};
fn entity<'a>(
    client: &'a ClientState,
    case: &fixture::Case,
    retained: bool,
) -> &'a gielinor_client::world_entity::Entity {
    if case.observer == 1 {
        &client.world_entities[&0].entities[&case.id]
    } else if retained {
        &client.detached_world_entities[0].entities[&case.id]
    } else {
        &client.world_entities[&7].entities[&case.id]
    }
}
fn entity_mut<'a>(
    client: &'a mut ClientState,
    case: &fixture::Case,
    retained: bool,
) -> &'a mut gielinor_client::world_entity::Entity {
    if case.observer == 1 {
        client
            .world_entities
            .get_mut(&0)
            .unwrap()
            .entities
            .get_mut(&case.id)
            .unwrap()
    } else if retained {
        client.detached_world_entities[0]
            .entities
            .get_mut(&case.id)
            .unwrap()
    } else {
        client
            .world_entities
            .get_mut(&7)
            .unwrap()
            .entities
            .get_mut(&case.id)
            .unwrap()
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    client
        .npcs
        .insert(1, gielinor_client::npc::Definition::decode(1, &[0])?);

    client.player_state = None;
    let mut checked = 0;
    for case in fixture::cases()
        .into_iter()
        .filter(|case| case.dimensions[0] >= 24)
    {
        for retained in [false, true] {
            client.world_entities.clear();
            client.detached_world_entities.clear();
            client.active_world = 0;
            client.view_registry = Default::default();
            for (opcode, payload) in &case.packets {
                client.apply(
                    game_packet::decode(&Packet {
                        opcode: *opcode,
                        payload: payload.clone(),
                    })?
                    .unwrap(),
                )?;
            }
            let (base, npcs) = fixture::setup(&case, &mut client.world_entities);
            client.world_base = base;
            client.npc_state = npcs;
            for world in client.world_entities.values_mut() {
                for e in world.entities.values_mut() {
                    e.animation.reset();
                    e.default_animation.reset();
                }
            }
            let e = entity_mut(&mut client, &case, false);
            let base = e.rebuild.as_ref().unwrap().base;
            let mut observer = fixture::npc();
            observer.path.position = std::array::from_fn(|i| base[i] * 128 + 1600);
            observer.path.fine = observer.path.position.map(|v| v as f32);
            observer.turn.target = 777;
            observer.facing = Some(Face {
                target: FaceTarget::Entity {
                    kind: 1,
                    index: 41,
                    fallback: 777,
                },
                turn_on_walk: false,
                instant: false,
            });
            e.npcs.indices.push(42);
            e.npcs.npcs.insert(42, observer);
            if retained {
                client
                    .detached_world_entities
                    .push(client.world_entities.remove(&7).unwrap());
            }
            let mut previous = 777;
            for phase in 0..2 {
                if phase == 1 {
                    if let Some(own) = entity_mut(&mut client, &case, retained)
                        .npcs
                        .npcs
                        .get_mut(&41)
                    {
                        own.path.position[0] += 128;
                        own.path.position[1] -= 64;
                        own.path.fine = own.path.position.map(|v| v as f32);
                    }
                }
                client.actor_cycle = 10 + phase as i32;
                client.tick_actor_sequences()?;
                let expected = match case.angles[phase] {
                    -2 => 777,
                    -1 => previous,
                    v => v,
                };
                assert_eq!(
                    entity(&client, &case, retained).npcs.npcs[&42].turn.target as i32,
                    expected,
                    "observer={} mask={} rebased={} phase={} retained={}",
                    case.observer,
                    case.mask,
                    case.rebased,
                    phase,
                    retained
                );
                previous = expected;
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 1024);
    println!("{checked} cache-backed duplicate observer tick scenarios passed before/after target-copy mutation, including retained storage; 512 original in-bounds cases replayed twice");
    Ok(())
}
