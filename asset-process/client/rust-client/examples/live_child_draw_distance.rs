//! Isolated-account login followed by a local long-distance actor visibility replay.
//! No synthetic world state or interactions are sent to the server.
use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection};
use std::time::{Duration, Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
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
    let mut npc = client.npc_state.npcs.values().next().unwrap().clone();
    npc.id = 16299;
    npc.path.teleport([50, 80], 1);
    npc.path.position = [6464, 10240];
    npc.path.fine = [6464., 10240.];
    npc.position = [50, 80];
    npc.sequence = Default::default();
    npc.pose = Default::default();
    npc.spots.clear();
    npc.animations = client.npcs[&16299].animations;
    client.npc_state.indices = vec![1];
    client.npc_state.npcs.clear();
    client.npc_state.npcs.insert(1, npc.clone());
    client.player_state = None;
    client.world_entities.clear();
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
    entity.npcs.indices.push(1);
    entity.npcs.npcs.insert(1, npc);
    client.npc_state.npcs.clear();
    client.npc_state.indices.clear();
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
    use gielinor_client::vm::Host;
    let mut stack = gielinor_client::vm::Stacks::default();
    for value in [25, 90, 25] {
        stack.push_int(14)?;
        stack.push_int(value)?;
        client.extended(3212, false, &mut stack)?;
        let image = live.paint(&mut client, 765, 503)?;
        let hit = live.pick_where_view(|view, tag| view == 7 && tag == 0x4000000000000001);
        assert!(hit.is_some(), "child matrix model remains rendered");
        if let Some((point, _)) = hit {
            let menu = live.menu_entries(&mut client, point)?;
            assert_eq!(
                menu.iter().any(|e| matches!(
                    e.action.scene(),
                    (
                        7,
                        gielinor_client::context_menu::Action::Npc { index: 1, .. }
                    ) | (7, gielinor_client::context_menu::Action::ExamineNpc(1))
                )),
                value == 90,
                "far child model picking menu at distance {value}"
            );
            std::fs::write("work/child-draw-distance-live.bmp", image.bmp()?)?;
        }
    }
    println!("Live child draw-distance3212 preference: NPC at8000 depth remains rendered; selection absent25/present90/absent25 passed");
    Ok(())
}
