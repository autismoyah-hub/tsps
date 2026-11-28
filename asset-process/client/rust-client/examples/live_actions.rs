//! Explicit dedicated-account walk/interaction integration check (no synthetic success).
use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection, route, world_action,
};
use std::time::{Duration, Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    if a.len() != 6 {
        return Err("expected CACHE ADDRESS USERNAME PASSWORD_FILE PUBLIC_KEY".into());
    }
    let mut client = ClientState::load(DiskCache::open(&a[1], 64 << 20)?)?;
    let mut live = Connection::connect(&mut client, &a[2], &a[3], &a[4], &a[5])?;
    let start = Instant::now();
    let mut target = None;
    let mut moved = false;
    let mut interaction = None;
    let mut action_message_start = 0;
    let mut last_message = None;
    let mut sent = Instant::now();
    while start.elapsed() < Duration::from_secs(120) {
        live.poll(&mut client)?;
        if let Some(message) = client.game_messages.back() {
            if last_message.as_ref() != Some(message) {
                println!("Server game message: {}", message.text);
                last_message = Some(message.clone());
            }
        }
        if let Some((scene, _)) = &live.world {
            if start.elapsed() > Duration::from_secs(10) && target.is_none() {
                let p = client.data.position;
                let local = [p[1] - scene.base_x, p[2] - scene.base_y];
                for delta in [[1, 0], [0, 1], [-1, 0], [0, -1]] {
                    if route::can_step(&scene.collision[p[0] as usize], local, 1, delta) {
                        let dest = [p[1] + delta[0], p[2] + delta[1]];
                        client.outbound.push_back(world_action::walk(
                            dest[0] as u16,
                            dest[1] as u16,
                            0,
                        )?);
                        println!("Sent walk from {:?} to {dest:?}", [p[1], p[2]]);
                        target = Some(dest);
                        sent = Instant::now();
                        break;
                    }
                }
            }
        }
        if let Some(target) = target {
            let p = client.data.position;
            if !moved && [p[1], p[2]] == target {
                moved = true;
                println!("Server confirmed player movement to {target:?}");
            }
            if moved && interaction.is_none() {
                let npc = client
                    .npc_state
                    .npcs
                    .iter()
                    .filter(|(_, n)| {
                        client.npcs.get(&(n.id as u32)).is_some_and(|d| {
                            d.operations.iter().any(|op| {
                                op.index == 0
                                    && op.sub.is_none()
                                    && op.text.eq_ignore_ascii_case("talk-to")
                            })
                        })
                    })
                    .min_by_key(|(_, n)| {
                        (n.position[0] - p[1]).abs() + (n.position[1] - p[2]).abs()
                    });
                if let Some((&index, npc)) = npc {
                    println!("Sent Talk-to: NPC index {index}, definition {}", npc.id);
                    action_message_start = client.game_messages.len();
                    client
                        .outbound
                        .push_back(world_action::npc_first(index, 0, false));
                    interaction = Some((
                        index,
                        client
                            .interfaces
                            .children
                            .iter()
                            .map(|(&parent, sub)| (parent, sub.group))
                            .collect::<std::collections::BTreeMap<_, _>>(),
                    ));
                    sent = Instant::now();
                }
            }
            if moved && sent.elapsed() > Duration::from_secs(10) {
                println!(
                    "After action: position {:?}, {} subinterfaces, {} packets",
                    client.data.position,
                    client.interfaces.children.len(),
                    live.packets
                );
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    live.session.close();
    if !moved {
        return Err("server did not confirm requested movement".into());
    }
    let Some((_, before)) = interaction else {
        return Err("no Talk-to target found".into());
    };
    let after = client
        .interfaces
        .children
        .iter()
        .map(|(&parent, sub)| (parent, sub.group))
        .collect::<std::collections::BTreeMap<_, _>>();
    let changes = after
        .iter()
        .filter(|(parent, group)| before.get(parent) != Some(group))
        .collect::<Vec<_>>();
    println!("Server interface changes after Talk-to: {changes:?}");
    let tanner_response = client
        .game_messages
        .iter()
        .skip(action_message_start)
        .any(|m| m.text == "Bring cowhides. The tanner tans; he does not keep a shop.");
    println!("Expected tanner response received: {tanner_response}");
    if changes.is_empty() && !tanner_response {
        return Err("Talk-to sent, but no server interface change confirmed".into());
    }
    Ok(())
}
