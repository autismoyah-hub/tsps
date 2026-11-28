//! Isolated-account login followed by local synthetic child-world render transitions.
//! No synthetic world state or interactions are sent to the server.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    live_runtime::Connection,
    rebuild::{Chunk, WorldEntity},
    world_entity::{Definition, Entity, Pose, State},
};
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
    let state = client.player_state.as_ref().unwrap();
    let player = state.root_actor(state.local).unwrap();
    let fine = [
        player.path.position[0] - client.world_base[0] * 128,
        player.path.position[1] - client.world_base[1] * 128,
    ];
    let target = player.path.position;
    client.camera.pitch = 256;
    client.camera.yaw = 0;
    client.camera.tick_follow(
        &live.world.as_ref().unwrap().0,
        target,
        client.data.position[0] as usize,
        50,
    );
    let mut chunks = vec![None; 4 * 3 * 5];
    for x in 0..3 {
        for y in 0..5 {
            chunks[x * 5 + y] = Some(Chunk {
                plane: 0,
                zone_x: 400 + x as u16,
                zone_y: 400 + y as u16,
                rotation: 0,
            });
        }
    }
    let mut definition = Definition::decode(0, &[0])?;
    definition.bounds = [0, 0, 512, 512];
    let mut entity = Entity {
        dimensions: [24, 40],
        owner_type: 1,
        definition: Some(definition),
        plane: client.data.position[0] as u8,
        pose: Pose {
            x: fine[0],
            y: 0,
            z: fine[1],
            angle: 256,
        },
        rebuild: Some(WorldEntity {
            base: [6400, 6400],
            dimensions: [24, 40],
            chunks,
            regions: vec![12850],
        }),
        ..Entity::default()
    };
    let (&npc_index, npc) = client
        .npc_state
        .npcs
        .iter()
        .find(|(_, n)| {
            client
                .npcs
                .get(&(n.id as u32))
                .is_some_and(|d| d.minimap && d.interactable && !d.models.is_empty())
        })
        .expect("live minimap NPC");
    let mut npc = npc.clone();
    npc.path.position = [6412 * 128 + 64, 6420 * 128 + 64];
    npc.position = [6412, 6420];
    entity.npcs.indices.push(npc_index);
    entity.npcs.npcs.insert(npc_index, npc);
    let mut state = State::default();
    state.visible.push(7);
    state.entities.insert(7, entity);
    client.world_entities.insert(0, state);
    use gielinor_client::context_menu::Action;
    client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .get_mut(&7)
        .unwrap()
        .owner_type = 0;
    {
        let def = client
            .world_entities
            .get_mut(&0)
            .unwrap()
            .entities
            .get_mut(&7)
            .unwrap()
            .definition
            .as_mut()
            .unwrap();
        def.picking_type = 1;
        def.interactable = true;
        def.name = "PickFixture".into();
        def.operations[0] = Some("Board".into());
    }
    client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .get_mut(&7)
        .unwrap()
        .npcs
        .npcs
        .get_mut(&npc_index)
        .unwrap()
        .appearance
        .name = "ChildPickNpc".into();

    use gielinor_client::interaction_policy::Update as Policy;
    client.view_registry.insert(7);
    client.interaction_policy.apply(Policy::Set {
        world: 0,
        input: 1,
        actions: 1,
    });
    client.interaction_policy.apply(Policy::Set {
        world: 7,
        input: 2,
        actions: 1,
    });
    client.camera.pitch = 2048;
    live.paint(&mut client, 765, 503)?;
    client.camera.follow_target = Some((3, (6412 << 14) | 6420));
    let local = client.camera_focus_local().expect("child tile focus");
    assert_eq!(
        (local.view, local.fine),
        (7, [12 * 128 + 64, 20 * 128 + 64])
    );
    let owner = &client.world_entities[&0].entities[&7];
    let expected = live.projectile_scenes()[&7]
        .elevation(0, local.fine[0], local.fine[1])
        .unwrap_or(0)
        .wrapping_add(
            live.world
                .as_ref()
                .unwrap()
                .0
                .elevation(client.data.position[0] as usize, owner.pose.x, owner.pose.z)
                .unwrap_or(0),
        );
    let (_, plane, height) = live.camera_follow_target(&client).expect("follow height");
    assert_eq!(plane, 0);
    assert_eq!(height, expected);
    client.camera.set_target_height(height, 50);
    assert_eq!(client.camera.focus[1], height.wrapping_sub(50));
    client.camera.follow_target = None;
    client.camera.tick_follow(
        &live.world.as_ref().unwrap().0,
        target,
        client.data.position[0] as usize,
        50,
    );
    live.paint(&mut client, 765, 503)?;

    let (point, _) = live
        .pick_where_view(|view, tag| view == 7 && tag & (1 << 63) != 0)
        .expect("child terrain pick");
    let entries = live.menu_entries(&mut client, point)?;
    let action = entries
        .iter()
        .find(|e| e.option == "Set heading" && e.action.scene().0 == 7)
        .expect("child heading entry")
        .action;
    assert!(
        entries
            .iter()
            .any(|e| matches!(e.action.scene(), (0, Action::Walk(_)))),
        "root walk survives overlapping child steering: {:?}",
        entries.iter().map(|e| e.action).collect::<Vec<_>>()
    );
    client.outbound.clear();
    client.menu_action(action)?;
    let packet = client.outbound.pop_front().unwrap();
    assert_eq!(packet.opcode, 44);
    assert_eq!(packet.payload, [client.steering.direction]);
    assert_eq!(client.steering.remaining, 30);
    client.interaction_policy.apply(Policy::Set {
        world: 7,
        input: 1,
        actions: 1,
    });
    let entries = live.menu_entries(&mut client, point)?;
    assert!(!entries
        .iter()
        .any(|e| matches!(e.action.scene().1, Action::Heading(_))));
    assert_eq!(
        entries
            .iter()
            .filter(|e| matches!(e.action.scene().1, Action::Walk(_)))
            .count(),
        1
    );
    client.interaction_policy.apply(Policy::Set {
        world: 0,
        input: 2,
        actions: 1,
    });
    let entries = live.menu_entries(&mut client, point)?;
    assert!(entries
        .iter()
        .any(|e| matches!(e.action.scene(), (0, Action::Heading(_)))));
    // The synthetic child floor covers root indicators; isolate their root-scene pixels.
    client.world_entities.clear();
    client.view_registry.remove(7);
    client.input.pointer=[-100,-100];
    client.steering.remaining=0;
    let before=live.paint(&mut client,765,503)?.pixels().to_vec();
    let mut changed=false;
    for direction in 0..16 {
        client.steering.request(direction)?;
        let raster=live.paint(&mut client,765,503)?;
        if raster.pixels()!=before.as_slice() {std::fs::write("work/steering-active.bmp",raster.bmp()?)?;changed=true;break;}
    }
    assert!(changed,"active 3D steering arrow must change visible world pixels");
    for _ in 0..30 {client.steering.tick();}
    assert_eq!(live.paint(&mut client,765,503)?.pixels(),before.as_slice(),"steering expiration restores world exactly");
    // Choose an unobstructed heading entry through the actual viewport menu path.
    client.input.pointer=[-100,-100];
    live.paint(&mut client,765,503)?;
    let mut preview=None;
    for y in (40..300).step_by(30) {for x in (40..500).step_by(30) {
        if let Some(Action::Heading(d))=live.menu_entries(&mut client,[x,y])?.last().map(|e|e.action.scene().1) {
            client.input.pointer=[x,y];
            let raster=live.paint(&mut client,765,503)?;
            if raster.pixels()!=before.as_slice() {preview=Some(([x,y],d));break;}
        }
    }if preview.is_some(){break;}}
    let (preview_point,direction)=preview.expect("visible hover arrow from top heading entry");
    assert_eq!(client.steering.scene[1].unwrap().angle,direction as u16*128);
    client.steering.request(direction)?;
    live.paint(&mut client,765,503)?;
    assert!(client.steering.scene[0].is_some());assert!(client.steering.scene[1].is_none());
    client.steering.remaining=0;client.input.pointer=[-100,-100];
    assert_eq!(live.paint(&mut client,765,503)?.pixels(),before.as_slice());
    println!("Live 3D steering preview: top menu heading at {preview_point:?}, same-direction suppression and exact restoration passed");
    println!("Live 3D steering model: visible active arrow and exact restoration after30ticks passed");
    println!("Viewport steering: root/child policy selection, overlapping root walk, heading packet and policy restoration passed at {point:?}");
    Ok(())
}
