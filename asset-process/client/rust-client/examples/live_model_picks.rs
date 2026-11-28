//! Isolated login and local root-model menu selection regression.
//! No synthetic world state or interactions are sent to the server.
use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection,
};
use std::time::{Duration, Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClientState::load(DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?)?;
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
    let target = player.path.position;
    client.camera.pitch = 256;
    client.camera.yaw = 0;
    client.camera.tick_follow(&live.world.as_ref().unwrap().0, target, client.data.position[0] as usize, 50);
    // Two non-colocated actors: the existing tile-center expansion cannot supply
    // the second actor, so both entries must come from model selection regions.
    let npc = client.npc_state.npcs.values().find(|n|
        client.npcs.get(&(n.id as u32)).is_some_and(|d| d.size == 1 && d.interactable && !d.models.is_empty()))
        .expect("size-one live NPC").clone();
    client.npc_state.npcs.clear();
    client.npc_state.indices.clear();
    client.preferences.render_self = false;
    for (index, offset) in [(32000, -16), (32001, 16)] {
        let mut copy = npc.clone();
        copy.path.position = [target[0] + offset, target[1] + offset];
        copy.position = copy.path.position.map(|v| v >> 7);
        copy.appearance.name = format!("PickActor{index}");
        client.npc_state.indices.push(index);
        client.npc_state.npcs.insert(index, copy);
    }
    client.camera.pitch = 2048;
    live.paint(&mut client, 765, 503)?;
    let (start, _) = live.pick_where(|tag| tag == 0x4000000000000000 | 32000 || tag == 0x4000000000000000 | 32001)
        .expect("fixture actor visible");
    use gielinor_client::context_menu::Action;
    let mut found = None;
    'scan: for y in start[1]-30..start[1]+100 {
        for x in (start[0]-60..start[0]+60).step_by(2) {
            let entries = live.menu_entries(&mut client, [x,y])?;
            if [32000,32001].iter().all(|&index| entries.iter().any(|e| e.action == Action::ExamineNpc(index))) {
                for index in [32000,32001] {
                    assert_eq!(entries.iter().filter(|e| e.action == Action::ExamineNpc(index)).count(), 1);
                }
                found = Some([x,y]); break 'scan;
            }
        }
    }
    assert!(found.is_some(), "overlapping bounds must select both non-colocated NPCs");
    let point = found.unwrap();
    for index in [32000,32001] {
        client.npc_state.npcs.get_mut(&index).unwrap().visual.transparency.get_or_insert_with(Default::default)
            .set(client.actor_cycle, client.actor_cycle, client.actor_cycle, 255, 255, true);
    }
    live.paint(&mut client, 765, 503)?;
    assert!(live.pick_where(|tag| tag == 0x4000000000000000 | 32000 || tag == 0x4000000000000000 | 32001).is_none(), "invisible NPCs leave no pick pixels");
    let invisible = live.menu_entries(&mut client, point)?;
    for index in [32000,32001] { assert!(invisible.iter().any(|e| e.action == Action::ExamineNpc(index)), "invisible NPC retains original bounds selection"); }
    assert!(client.outbound.is_empty(), "synthetic state must not be transmitted");
    println!("Root model selection: two non-colocated NPC menus at {:?}, no duplicate examines", found.unwrap());
    Ok(())
}
