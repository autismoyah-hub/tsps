//! Isolated-account login followed by local synthetic child-world render transitions.
//! No synthetic world state or interactions are sent to the server.
use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection,
    rebuild::{Chunk, WorldEntity}, world_entity::{Entity, State, Pose, Definition}};
use std::time::{Duration, Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote("work/kotlin-world/.data/cache/js5", 64 << 20, "127.0.0.1:43617".parse()?)?;
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
        assert!(Instant::now() < deadline, "login/scene timeout: established={} closed={} world={} scripts_busy={} packets={} title={:?}", live.session.is_established(), live.session.is_closed(), live.world.is_some(), live.scripts_busy(), live.packets, live.title.as_ref().map(|title| (&title.screen, &title.lines)));
        std::thread::sleep(Duration::from_millis(20));
    }
    // Freeze network polling so server state cannot overwrite the local scenario.
    client.canvas = [765, 503];
    // Roof preference has its own replay; keep this low camera's view unobstructed.
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
    entity.npcs.indices.push(npc_index);
    entity.npcs.npcs.insert(npc_index, npc);
    let mut state = State::default(); state.visible.push(7); state.entities.insert(7, entity);
    client.world_entities.insert(0, state);
    let hidden = live.paint(&mut client, 765, 503)?;
    assert_eq!(live.minimap_view(&client).unwrap().entities.len(), 0, "overlapping player hides child minimap");
    std::fs::write("work/world-overlap-hidden.bmp", hidden.bmp()?)?;
    // Category 0 is exempt even when the local root player intersects.
    client.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().owner_type = 0;
    let visible = live.paint(&mut client, 765, 503)?;
    let minimap = live.minimap_view(&client).unwrap();
    assert_eq!(minimap.entities.len(), 1);
    assert!(minimap.entities[0].markers.iter().any(|(_, dot)| *dot == 1), "visible child NPC minimap marker");
    assert!(live.pick_where_view(|view, _| view == 7).is_some(), "child geometry rendered");
    let saved_limit=client.world_entity_limit.get() as i32;
    let mut limit_stack=gielinor_client::vm::Stacks::default();limit_stack.push_int(0)?;
    gielinor_client::vm::Host::extended(&mut client,7900,false,&mut limit_stack)?;
    live.paint(&mut client,765,503)?;
    assert!(live.pick_where_view(|view,_|view==7).is_none(),"opcode7900 limit zero removes nonlocal child rendering");
    limit_stack.push_int(saved_limit)?;gielinor_client::vm::Host::extended(&mut client,7900,false,&mut limit_stack)?;
    assert_eq!(visible.pixels(),live.paint(&mut client,765,503)?.pixels(),"restoring draw limit restores child frame");
    println!("Live child draw limit: script7900 hides and restores child geometry");
    // Only one fine-coordinate unit separates an admitted child from Scene.bs rejection.
    let saved_camera=std::mem::replace(&mut client.camera,Default::default());
    client.camera.pitch=256;client.camera.yaw=0;client.camera.focus[0]=60;client.camera.focus[2]=fine[1];
    client.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().pose.x=60;
    client.camera.tick_follow(&live.world.as_ref().unwrap().0,[client.world_base[0]*128+60,client.world_base[1]*128+fine[1]],client.data.position[0] as usize,50);
    live.paint(&mut client,765,503)?;
    assert!(live.pick_where_view(|view,_|view==7).is_some(),"child remains visible at the last valid edge coordinate");
    client.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().pose.x=59;
    client.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().zone.locations.push_back(gielinor_client::location_update::Event::Clear([0,6400,6400]));
    live.paint(&mut client,765,503)?;
    assert!(live.pick_where_view(|view,_|view==7).is_none(),"out-of-bounds child insertion must suppress geometry");
    assert!(client.world_entities[&0].entities[&7].zone.locations.is_empty(),"rejected child still consumes location updates");
    client.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().pose.x=fine[0];
    client.camera=saved_camera;
    assert_eq!(visible.pixels(),live.paint(&mut client,765,503)?.pixels(),"restoring admitted child restores the frame");
    println!("Live child-scene admission: one-unit boundary crossing suppresses geometry and restores exactly");
    let changed = hidden.pixels().iter().zip(visible.pixels()).filter(|(a,b)| a != b).count();
    assert!(changed > 100, "collapsed/normal raster transition: {changed}");
    std::fs::write("work/world-overlap-visible.bmp", visible.bmp()?)?;
    // Re-entering overlap must reproduce the first frame exactly.
    client.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().owner_type = 1;
    let again = live.paint(&mut client, 765, 503)?;
    assert_eq!(hidden.pixels(), again.pixels());
    assert_eq!(live.minimap_view(&client).unwrap().entities.len(), 0);
    // Hints remain independently visible over a hidden entity; their flash
    // clock is the simulation cycle even when the render count differs.
    client.view_registry.insert(7);
    let saved_hint=client.hint.clone(); let saved_cycles=[client.actor_cycle,client.cycle];
    client.hint.kind=4;client.hint.index=7;client.actor_cycle=5;client.cycle=15;
    assert!(live.minimap_view(&client).unwrap().hint.is_some(), "hidden entity hint on simulation phase");
    client.actor_cycle=15;client.cycle=5;
    assert!(live.minimap_view(&client).unwrap().hint.is_none(), "hint off despite render phase");
    client.actor_cycle=5;client.hint.kind=1;client.hint.index=npc_index;
    let listener=client.local_minimap_fine();
    let root=client.npc_state.npcs.remove(&npc_index).unwrap();
    let e=&client.world_entities[&0].entities[&7];
    let p=e.transform_horizontal([12*128+64,20*128+64]);
    let expected=gielinor_client::minimap::offset([p[0] as i32+client.world_base[0]*128,p[2] as i32+client.world_base[1]*128],listener,0.03125);
    assert_eq!(live.minimap_view(&client).unwrap().hint,Some(expected),"child NPC hint");
    let root_fine=root.path.position;client.npc_state.npcs.insert(npc_index,root);
    assert_eq!(live.minimap_view(&client).unwrap().hint,Some(gielinor_client::minimap::offset(root_fine,listener,0.03125)),"root duplicate hint priority");
    client.hint=saved_hint.clone();client.actor_cycle=saved_cycles[0];client.cycle=saved_cycles[1];
    println!("Live world hints: hidden entity target, child NPC target, root duplicate priority, simulation blink phase passed");
    // Scene arrows are independent of hidden child geometry, unlike actor overlays.
    let saved_scene_pitch=client.camera.pitch;client.camera.pitch=2048;
    client.actor_cycle=5;client.hint.kind=0;
    let scene_no_hint=live.paint(&mut client,765,503)?;
    client.hint.kind=4;client.hint.index=7;client.hint.height=0;
    let scene_entity_hint=live.paint(&mut client,765,503)?;
    let entity_arrow_pixels=scene_no_hint.pixels().iter().zip(scene_entity_hint.pixels()).filter(|(a,b)|a!=b).count();
    assert!(entity_arrow_pixels>10,"hidden-world entity scene arrow missing: {entity_arrow_pixels}");
    client.actor_cycle=15;client.hint.kind=0;
    let off_baseline=live.paint(&mut client,765,503)?;client.hint.kind=4;
    assert!(off_baseline.pixels()==live.paint(&mut client,765,503)?.pixels(),"entity arrow off phase");
    client.actor_cycle=5;client.hint.kind=2;client.hint.tile=[6412,6420];client.hint.offset=[64,64];
    let scene_tile_hint=live.paint(&mut client,765,503)?;
    let tile_arrow_pixels=scene_no_hint.pixels().iter().zip(scene_tile_hint.pixels()).filter(|(a,b)|a!=b).count();
    assert!(tile_arrow_pixels>10,"child tile scene arrow missing: {tile_arrow_pixels}");
    std::fs::write("work/world-scene-hint.bmp",scene_tile_hint.bmp()?)?;
    client.hint.kind=0;let missing_baseline=live.paint(&mut client,765,503)?;
    client.hint.kind=4;client.hint.index=65535;
    assert!(missing_baseline.pixels()==live.paint(&mut client,765,503)?.pixels(),"missing entity hint draws nothing");
    client.hint=saved_hint.clone();client.actor_cycle=saved_cycles[0];client.cycle=saved_cycles[1];
    println!("Live scene hints: {entity_arrow_pixels} hidden-entity pixels, {tile_arrow_pixels} child-tile pixels, off-phase/missing-target suppression passed");
    client.camera.pitch=saved_scene_pitch;
    let again=live.paint(&mut client,765,503)?;
    // Removing a hidden actor must not change the collapsed scene raster.
    client.world_entities.get_mut(&0).unwrap().entities.get_mut(&7).unwrap().npcs.npcs.clear();
    let without_npc = live.paint(&mut client, 765, 503)?;
    assert_eq!(again.pixels(), without_npc.pixels());
    // Two overlapping entities, away from actors: prior submissions in the
    // same client cycle count, including a hidden scene, but never self.
    client.actor_cycle += 1;
    let state = client.world_entities.get_mut(&0).unwrap();
    let entity = state.entities.get_mut(&7).unwrap();
    entity.pose.x = 100000; entity.pose.z = 100000;
    let second = entity.clone();
    state.entities.insert(8, second); state.visible.push(8);
    live.paint(&mut client, 765, 503)?;
    assert_eq!(live.minimap_view(&client).unwrap().entities.len(), 1, "first cycle draw");
    live.paint(&mut client, 765, 503)?;
    assert_eq!(live.minimap_view(&client).unwrap().entities.len(), 0, "same-cycle prior hidden scene blocks overlap");
    client.actor_cycle += 1;
    live.paint(&mut client, 765, 503)?;
    assert_eq!(live.minimap_view(&client).unwrap().entities.len(), 1, "next cycle clears old submission stamps");
    // An empty but submitted entity scene can collapse an overlapping root NPC.
    // Choose an NPC that the actual root frame proves is visible.
    let (_, tag) = live.pick_where(|tag| tag & 0xf000_0000_0000_0000 == 0x4000_0000_0000_0000)
        .expect("visible root NPC for overlap render");
    let index = tag as u16;
    let npc = &client.npc_state.npcs[&index];
    let definition_id = npc.id as u32;
    let npc_fine = [npc.path.position[0]-client.world_base[0]*128, npc.path.position[1]-client.world_base[1]*128];
    let old_def = client.npcs[&definition_id].clone();
    client.npcs.get_mut(&definition_id).unwrap().hide_overlap = false;
    let state = client.world_entities.get_mut(&0).unwrap();
    state.visible = vec![7]; state.entities.remove(&8);
    let entity = state.entities.get_mut(&7).unwrap();
    entity.owner_type = 0; entity.pose.x = npc_fine[0]; entity.pose.z = npc_fine[1];
    entity.dimensions = [8,8]; entity.rebuild_generation += 1;
    entity.rebuild = Some(WorldEntity { base: [6400,6400], dimensions: [8,8], chunks: vec![None;4], regions: vec![] });
    client.actor_cycle += 1;
    let npc_normal = live.paint(&mut client, 765, 503)?;
    client.npcs.get_mut(&definition_id).unwrap().hide_overlap = true;
    let npc_flat = live.paint(&mut client, 765, 503)?;
    let npc_changed = npc_normal.pixels().iter().zip(npc_flat.pixels()).filter(|(a,b)| a != b).count();
    assert!(npc_changed > 10, "root NPC flattening must affect visible pixels: {npc_changed}");
    client.npcs.get_mut(&definition_id).unwrap().hide_overlap = false;
    let npc_restored = live.paint(&mut client, 765, 503)?;
    assert_eq!(npc_normal.pixels(), npc_restored.pixels());
    client.npcs.insert(definition_id, old_def);
    println!("Live root NPC overlap: {npc_changed} changed pixels, exact restoration");
    // Freeze actor cycle and network while exercising the live palette/texture refresh.
    let saved_brightness=client.preferences.device.brightness;
    let brightness_base=live.paint(&mut client,765,503)?;
    client.preferences.device.set(6,100)?;
    let brighter=live.paint(&mut client,765,503)?;
    let brightness_pixels=brightness_base.pixels().iter().zip(brighter.pixels()).filter(|(a,b)|a!=b).count();
    assert!(brightness_pixels>100,"live scene brightness setting ignored: {brightness_pixels}");
    client.preferences.device.brightness=saved_brightness;
    assert!(brightness_base.pixels()==live.paint(&mut client,765,503)?.pixels(),"live brightness restoration differs");
    println!("Live brightness: {brightness_pixels} changed pixels and exact restoration");
    println!("Live renderer overlap transition passed: {changed} changed pixels, minimap hidden/visible/hidden, exact collapsed repeat");
    Ok(())
}
