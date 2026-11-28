use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, npc_info::Npc, scene::Scene, world_actors,
};
fn npc(point: i32) -> Npc {
    Npc {
        visual: Default::default(),
        path: {
            let mut p = gielinor_client::actor_path::Path::default();
            p.position = [point; 2];
            p
        },
        sequence: Default::default(),
        pose: Default::default(),
        idle_fallback: Default::default(),
        spots: Default::default(),
        overlays: Default::default(),
        turn: Default::default(),
        facing: None,
        freezes: Default::default(),
        exact: None,
        id: 16299,
        position: [point >> 7; 2],
        angle: 0,
        spawn_cycle: None,
        steps: Vec::new(),
        jump: false,
        blocks: Vec::new(),
        appearance: Default::default(),
        animations: [-1; 15],
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
    let mut actors = world_actors::Actors::load(&mut client.scripts.source.cache)?;
    let scene = Scene::empty(0, 0)?;
    client.player_state = None;
    client.data.position = [0, 0, 0];
    client.actor_cycle = 10;
    let mut actor = npc(320);
    actor.idle_fallback.state.id = 0;
    actor.idle_fallback.state.loading = 1;

    client.npc_state.indices = vec![41];
    client.npc_state.insertion_order = vec![41];
    client.npc_state.npcs.clear();
    client.npc_state.npcs.insert(41, actor);
    client.world_spots = (0..6)
        .map(|_| {
            gielinor_client::world_spot::Request {
                id: 3,
                height: 0,
                delay: 0,
                position: [0, 2, 2],
            }
            .start(0)
        })
        .collect();
    for variant in 0..5 {
        if variant == 1 {
            let mut projectile = gielinor_client::projectile::Request {
                id: 3,
                source: [0, 2, 2],
                target: [0, 2, 2],
                heights: [0; 2],
                entities: [0; 2],
                start: 0,
                end: 100,
                slope: 0,
                offset: 0,
            }
            .start(0);
            projectile.motion.moving = true;
            projectile.motion.position = [320., 320., 0.];
            client.projectiles.push(projectile);
        }
        if variant == 2 {
            client.scripted_projectiles.definition.spot = 3;
            client.scripted_projectiles.definition.source.tile = [0, 2, 2];
            client.scripted_projectiles.effects.insert(
                9,
                gielinor_client::scripted_projectile::Effect {
                    serial: 1,
                    visible: true,
                    motion: gielinor_client::scripted_projectile::Motion::new([320., 0., 320.]),
                    clock: Default::default(),
                    sequence: -1,
                    pending_shared: false,
                },
            );
        }
        if variant >= 3 {
            client.steering.scene = [
                Some(gielinor_client::steering_scene::Placement {
                    position: [320, 0, 320],
                    angle: 0,
                    active: true,
                }),
                Some(gielinor_client::steering_scene::Placement {
                    position: [320, 0, 320],
                    angle: 0,
                    active: false,
                }),
            ];
            if variant == 4 {
                client.steering.scene[0] = None;
            }
        }
        let (records, tags) = actors.temporary_draw_objects(&mut client, &scene, None)?;
        assert_eq!(
            records.len(),
            5,
            "body plus four accepted standalone effects"
        );
        let arrow_count = if variant == 3 {
            2
        } else if variant == 4 {
            1
        } else {
            0
        };
        assert_eq!(tags[arrow_count], Some(0x4000000000000029));
        assert_eq!(tags.iter().filter(|tag| tag.is_some()).count(), 1);
        let (triangles, pending) = actors.triangles(&mut client, &scene, &Default::default())?;
        assert!(pending.is_empty(), "pending resources {pending:?}");
        assert!(!triangles.is_empty());
        let batches =
            gielinor_client::object_submission::temporary_effect_batches(triangles.iter(), 0);
        assert_eq!(
            batches
                .iter()
                .map(|&(id, (tag, _))| {
                    assert_eq!(tag, 0);
                    id
                })
                .collect::<Vec<_>>(),
            (0..5).filter(|&id| id != arrow_count).collect::<Vec<_>>()
        );
        assert!(
            triangles
                .iter()
                .all(|t| t.model_order.is_some_and(|o| o.temporary_id.is_some())),
            "unavailable body must not produce geometry"
        );
        println!("Five-slot capacity preserves four cached standalone/steering batches after an unavailable NPC body: {} triangles, temporary identities exclude the unavailable body",triangles.len());
    }
    Ok(())
}
