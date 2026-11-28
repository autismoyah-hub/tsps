//! Isolated-account login followed by local synthetic child-world render transitions.
//! No synthetic world state or interactions are sent to the server.
use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection,
    rebuild::{Chunk, WorldEntity}, world_entity::{Entity, State, Pose, Definition}};
use std::time::{Duration, Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5", 64 << 20, "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let mut live = Connection::connect(&mut client, "127.0.0.1:43617", "rustclient239",
        "work/kotlin-world/test-account.password", "work/kotlin-world/.data/client.key")?;
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        live.poll(&mut client)?;
        client.tick_actor_sequences()?;
        if live.world.is_some() && live.session.is_established() && !live.scripts_busy() &&
            client.interfaces.widgets.values().any(|w| w.content == 1337) { break; }
        assert!(Instant::now() < deadline, "login/scene timeout");
        std::thread::sleep(Duration::from_millis(20));
    }
    // Freeze network polling so server state cannot overwrite the local scenario.
    client.canvas = [765, 503];
    // This fixture isolates child picking from the root building's upper floors.
    client.preferences.device.roofs_hidden = true;
    let state = client.player_state.as_ref().unwrap();
    let player = state.root_actor(state.local).unwrap();
    let fine = [player.path.position[0] - client.world_base[0]*128,
        player.path.position[1] - client.world_base[1]*128];
    let target = player.path.position;
    client.camera.pitch = 256;
    client.camera.yaw = 0;
    client.camera.tick_follow(&live.world.as_ref().unwrap().0, target, client.data.position[0] as usize, 50);
    let mut chunks = vec![None; 4*3*5];
    for x in 0..3 { for y in 0..5 {
        chunks[x*5+y] = Some(Chunk { plane: 0, zone_x: 400+x as u16, zone_y: 400+y as u16, rotation: 0 });
    }}
    let mut definition = Definition::decode(0, &[0])?;
    definition.bounds = [0, 0, 512, 512];
    let mut entity = Entity { dimensions: [24,40], owner_type: 1, definition: Some(definition),
        plane: client.data.position[0] as u8,
        pose: Pose { x: fine[0], y: 0, z: fine[1], angle: 256 },
        rebuild: Some(WorldEntity { base: [6400,6400], dimensions: [24,40], chunks, regions: vec![12850] }),
        ..Entity::default() };
    let (&npc_index, npc) = client.npc_state.npcs.iter().find(|(_, n)|
        client.npcs.get(&(n.id as u32)).is_some_and(|d| d.minimap && d.interactable && !d.models.is_empty()))
        .expect("live minimap NPC");
    let mut npc = npc.clone();
    npc.path.position = [6412*128+64, 6420*128+64];
    npc.position = [6412,6420];
    let mut second_npc = npc.clone();
    second_npc.path.position[0] += 16;
    second_npc.path.position[1] += 16;
    second_npc.position = second_npc.path.position.map(|v| v >> 7);
    second_npc.appearance.name = "SecondChildPickNpc".into();
    entity.npcs.indices.push(32000);
    entity.npcs.npcs.insert(32000, second_npc);
    entity.npcs.indices.push(npc_index);
    entity.npcs.npcs.insert(npc_index, npc);
    let mut state = State::default(); state.visible.push(7); state.entities.insert(7, entity);
    client.world_entities.insert(0, state);
    client.view_registry.insert(7);
    use gielinor_client::context_menu::Action;
    client.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().owner_type = 0;
    {
        let def = client.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().definition.as_mut().unwrap();
        def.picking_type = 1; def.interactable = true; def.name = "PickFixture".into();
        def.operations[0] = Some("Board".into());
    }
    client.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().npcs.npcs.get_mut(&npc_index).unwrap().appearance.name = "ChildPickNpc".into();
    let mut npc_pick = None;
    for pitch in [2048, 1024, 3072] {
        client.camera.pitch = pitch;
        live.paint(&mut client, 765, 503)?;
        if let Some(pick) = live.pick_where_view(|view, tag| view == 7 && tag == 0x4000000000000000 | npc_index as u64) {
            npc_pick = Some(pick); break;
        }
    }
    let (point, _) = npc_pick.expect("child NPC must have visible pick pixels");
    let entries = live.menu_entries(&mut client, point)?;
    assert!(entries.iter().any(|e| e.action.scene() == (7, Action::ExamineNpc(npc_index)) && e.target.contains("ChildPickNpc")), "child NPC menu: {:?}", entries.iter().map(|e| (&e.target,e.action)).collect::<Vec<_>>());
    let mut overlapping = None;
    'scan: for y in point[1]-30..point[1]+100 {
        for x in (point[0]-60..point[0]+60).step_by(2) {
            let candidates = live.menu_entries(&mut client, [x,y])?;
            if [npc_index,32000].iter().all(|&id| candidates.iter().any(|e| e.action.scene() == (7, Action::ExamineNpc(id)))) {
                overlapping = Some([x,y]); break 'scan;
            }
        }
    }
    assert!(overlapping.is_some(), "child matrix regions must select both non-colocated NPCs");
    for mode in [0, 2, 3] {
        client.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().definition.as_mut().unwrap().picking_type = mode;
        let entries = live.menu_entries(&mut client, point)?;
        match mode {
            0 => assert!(entries.iter().any(|e| matches!(e.action, Action::WorldEntity { id: 7, operation: 1 }))),
            2 => assert!(entries.iter().any(|e| e.action.scene() == (7, Action::ExamineNpc(npc_index)))),
            // GameFloatProjection substitutes only this child's tags with the
            // suppression sentinel; dx still visits independent root hits.
            3 => assert!(entries.iter().all(|e| (e.action.scene().0 == 0
                || matches!(e.action.scene().1, Action::Walk(_) | Action::Heading(_)))
                && !matches!(e.action, Action::WorldEntity { id: 7, .. }))),
            _ => unreachable!(),
        }
    }
    let old_view = client.player_state.as_ref().unwrap().local_view;
    client.player_state.as_mut().unwrap().local_view = 7;
    assert!(live.menu_entries(&mut client, point)?.iter().any(|e| e.action.scene() == (7, Action::ExamineNpc(npc_index))), "local view overrides suppressed picking");
    client.player_state.as_mut().unwrap().local_view = old_view;
    client.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().definition.as_mut().unwrap().picking_type = 1;
    // Isolate child terrain coordinates from the closest-view movement policy.
    client.interaction_policy.apply(gielinor_client::interaction_policy::Update::Set { world: 0, input: 0, actions: 1 });
    client.interaction_policy.apply(gielinor_client::interaction_policy::Update::Set { world: 7, input: 1, actions: 1 });
    if let Some((point, _)) = live.pick_where_view(|view, tag| view == 7 && tag & 0x8000000000000000 != 0) {
        let entries = live.menu_entries(&mut client, point)?;
        assert!(entries.iter().any(|e| matches!(e.action.scene(), (7, Action::Walk([x,z])) if (6400..6424).contains(&x) && (6400..6440).contains(&z))), "child terrain coordinates at {point:?}: {:?}; views {:?}", entries.iter().map(|e| e.action).collect::<Vec<_>>(), client.view_registry.ordered());
    } else { panic!("child terrain pick required"); }
    let (point, _) = live.pick_where_view(|view, tag| view == 7 && tag != 0 && tag >> 60 == 0).expect("child object pick required");
    assert!(live.menu_entries(&mut client, point)?.iter().any(|e| matches!(e.action.scene(), (7, Action::Examine { kind: 1, .. }))), "child object examine view");
    println!("Rendered overlapping child NPC/object/terrain menus, all four pick modes and local-world override passed");
    for index in [npc_index,32000] {
        client.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().npcs.npcs.get_mut(&index).unwrap()
            .visual.transparency.get_or_insert_with(Default::default)
            .set(client.actor_cycle, client.actor_cycle, client.actor_cycle, 255, 255, true);
    }
    live.paint(&mut client, 765, 503)?;
    assert!(live.pick_where_view(|view,tag| view == 7 && (tag == 0x4000000000000000 | npc_index as u64 || tag == 0x4000000000000000 | 32000)).is_none());
    let invisible = live.menu_entries(&mut client, overlapping.unwrap())?;
    for index in [npc_index,32000] { assert!(invisible.iter().any(|e| e.action.scene() == (7,Action::ExamineNpc(index))), "invisible child selection"); }
    assert!(client.outbound.is_empty(), "synthetic actions must not be transmitted");
    Ok(())
}
