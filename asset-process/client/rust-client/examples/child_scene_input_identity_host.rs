use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    scene::Scene,
    world_actors::Actors,
    world_entity::{Entity, Pose, State},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let actors = Actors::load(&mut client.scripts.source.cache)?;
    client.player_state = None;
    client.npc_state = Default::default();
    client.projectiles.clear();
    client.world_spots.clear();
    client.data.position = [0, 2, 2];
    client.actor_cycle = 10;
    let mut stack = gielinor_client::vm::Stacks::default();
    stack.push_int(6)?;
    client.world_entity_limit.execute(7900, &mut stack)?;
    let mut root = State::default();
    for (id, owner_type) in [(7, 0), (23, 1), (9, 0), (31, 1), (32, 0), (33, 1)] {
        root.visible.push(id);
        root.entities.insert(
            id,
            Entity {
                owner_type,
                pose: Pose {
                    x: 320,
                    y: 0,
                    z: 320,
                    angle: 0,
                },
                ..Default::default()
            },
        );
    }
    client.world_entities.clear();
    client.world_entities.insert(0, root);
    // A registered nested view must never enter the root's temporary input table.
    let mut nested = State::default();
    nested.visible.push(70);
    nested.entities.insert(
        70,
        Entity {
            owner_type: 0,
            pose: Pose {
                x: 320,
                y: 0,
                z: 320,
                angle: 0,
            },
            ..Default::default()
        },
    );
    client.world_entities.insert(7, nested);
    let scene = Scene::empty(0, 0)?;
    for variant in 0..3 {
        client.steering.scene = [None, None];
        client.world_spots.clear();
        if variant >= 1 {
            client.world_spots.push(
                gielinor_client::world_spot::Request {
                    id: 3,
                    height: 0,
                    delay: 0,
                    position: [0, 2, 2],
                }
                .start(0),
            );
        }
        if variant == 2 {
            client.steering.scene[1] = Some(gielinor_client::steering_scene::Placement {
                position: [320, 0, 320],
                angle: 0,
                active: false,
            });
        }
        let (records, tags, children) =
            actors.temporary_scene_objects(&mut client, &scene, None)?;
        assert_eq!(records.len(), 5);
        assert!(tags.iter().all(Option::is_none));
        let expected = match variant {
            0 => vec![(0, 7), (1, 9), (2, 32), (3, 23), (4, 31)],
            1 => vec![(0, 7), (1, 9), (2, 32), (4, 23)],
            _ => vec![(1, 7), (2, 9), (3, 32)],
        };
        assert_eq!(children, expected, "early/late/capacity variant {variant}");
        let (_, _, nested_children) =
            actors.temporary_scene_objects(&mut client, &Scene::empty(0, 0)?, Some(7))?;
        assert!(nested_children.is_empty());
    }
    println!("Three cache-backed child input tables preserve early/late identities, effect holes, five-slot capacity and root-only ownership");
    Ok(())
}
