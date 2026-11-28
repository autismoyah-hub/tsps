//! Dedicated-account real-server banker dialogue -> Continue -> choice -> bank.
use gielinor_client::{
    client_state::ClientState, context_menu::Action, disk_cache::DiskCache,
    live_runtime::Connection,
};
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    time::{Duration, Instant},
};
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
    let mut stage = 0;
    let mut changed = Instant::now();
    let mut continued = false;
    let mut item = -1;
    let mut inventory_before = 0;
    let mut bank_before = std::collections::HashMap::new();
    let mut bank_id = 0;

    while Instant::now() < deadline {
        live.poll(&mut client)?;
        client.tick_actor_sequences()?;
        if stage == 0 && live.world.is_some() && live.session.is_established() {
            let mut admin = TcpStream::connect("127.0.0.1:43618")?;
            admin.set_read_timeout(Some(Duration::from_secs(5)))?;
            admin.write_all(b"cmd rustclient239 tele 0 162 1 54 59\n")?;
            let mut response = String::new();
            BufReader::new(admin).read_line(&mut response)?;
            assert!(response.starts_with("OK"), "{response}");
            println!("Dedicated account moved beside hub banker");
            stage = 1;
            changed = Instant::now();
        }
        if stage == 1
            && changed.elapsed() > Duration::from_secs(2)
            && client.data.position[1..] == [10422, 123]
        {
            let p = client.data.position;
            if let Some((&index, npc)) = client
                .npc_state
                .npcs
                .iter()
                .filter(|(_, n)| {
                    client
                        .npcs
                        .get(&(n.id as u32))
                        .is_some_and(|d| d.name == "Banker")
                })
                .min_by_key(|(_, n)| (n.position[0] - p[1]).abs() + (n.position[1] - p[2]).abs())
            {
                println!(
                    "Talk-to banker {index}, definition {}, tile {:?}",
                    npc.id, npc.position
                );
                client.menu_action(Action::Npc {
                    index,
                    operation: 1,
                    sub: 0,
                })?;
                stage = 2;
                changed = Instant::now();
            }
        }
        if (stage == 2 || stage == 3) && !live.scripts_busy() {
            let layout = client.interfaces.layout(client.canvas)?;
            let wanted = if stage == 2 { 231 } else { 219 };
            let candidates: Vec<_> = layout
                .order
                .iter()
                .filter_map(|id| {
                    let w = &client.interfaces.widgets[id];
                    let r = &client.interfaces.runtime[id];
                    let p = &layout.widgets[id];
                    let choice =
                        stage == 2 || w.text.contains("I'd like to access my bank account");
                    (r.logical_id >> 16 == wanted
                        && choice
                        && !p.hidden
                        && p.clip[0] < p.clip[2]
                        && p.clip[1] < p.clip[3])
                        .then_some((
                            *id,
                            [(p.clip[0] + p.clip[2]) / 2, (p.clip[1] + p.clip[3]) / 2],
                            w.text.clone(),
                        ))
                })
                .collect();
            for (id, point, text) in candidates {
                let menu = client.interface_menu(point)?;
                let valid = if stage == 2 {
                    matches!(menu.last().map(|e| e.action), Some(Action::Continue(_)))
                } else {
                    menu.last()
                        .is_some_and(|e| !matches!(e.action, Action::Cancel))
                };
                if !valid {
                    continue;
                }
                println!("Visible dialogue stage {stage}, widget {id}, text {text:?}, action {:?}, point {point:?}",menu.last().unwrap().action);
                assert!(client.primary_interface_action(point)?);
                if stage == 2 {
                    assert!(
                        client.outbound.iter().any(|p| p.opcode == 115),
                        "Continue did not emit resume packet"
                    );
                    continued = true;
                }
                stage += 1;
                changed = Instant::now();
                break;
            }
        }
        if stage == 4
            && client.interfaces.children.values().any(|s| s.group == 12)
            && !live.scripts_busy()
        {
            assert!(continued);
            println!("Server confirmed bank group 12 after visible banker Continue and first dialogue choice; {} packets",live.packets);
            item = [1277, 1279, 1205]
                .into_iter()
                .find(|id| client.inventories.total(93, *id) > 0)
                .ok_or("No existing acceptance sword available")?;
            inventory_before = client.inventories.total(93, item);
            bank_before = client
                .inventories
                .containers
                .keys()
                .map(|&id| (id, client.inventories.total(id, item)))
                .collect();
            stage = 5;
        }
        if (stage == 5 || stage == 7) && !live.scripts_busy() {
            let wanted = if stage == 5 {
                "Deposit-1"
            } else {
                "Withdraw-1"
            };
            let layout = client.interfaces.layout(client.canvas)?;
            let points: Vec<_> = layout
                .order
                .iter()
                .filter_map(|id| {
                    let p = &layout.widgets[id];
                    let r = &client.interfaces.runtime[id];
                    (r.item == item && !p.hidden && p.clip[0] < p.clip[2] && p.clip[1] < p.clip[3])
                        .then_some([(p.clip[0] + p.clip[2]) / 2, (p.clip[1] + p.clip[3]) / 2])
                })
                .collect();
            for point in points {
                if let Some(entry) = client
                    .interface_menu(point)?
                    .into_iter()
                    .find(|e| e.option == wanted)
                {
                    println!(
                        "Visible bank action {} on item {item}, point {point:?}, {:?}",
                        entry.option, entry.action
                    );
                    if let Action::Interface { id, .. } = entry.action {
                        let r=&client.interfaces.runtime[&id];
                        println!("Bank operation identity {}:{}, item {}, mask {}, content {}, listener {:?}",r.logical_id,r.child_index,r.item,client.interfaces.operation_mask(id),client.interfaces.widgets[&id].content,client.interfaces.listener(id,1409));
                    }
                    if let Action::Interface { id, .. } = entry.action {
                        if client.interfaces.operation_mask(id)==0 {
                            let r=&client.interfaces.runtime[&id];
                            let events:Vec<_>=client.interfaces.events.iter().filter(|((logical,child),_)|*logical>>16==r.logical_id>>16 && *child==r.child_index).collect();
                            println!("Bank same-group/slot server event records: {events:?}");
                            live.session.close();return Err("Rendered bank item has no allowed operations; do not bypass server mask".into());
                        }
                    }
                    client.menu_action(entry.action)?;
                    println!("Bank action outbound {:?}",client.outbound.iter().map(|p|(p.opcode,&p.payload)).collect::<Vec<_>>());
                    stage += 1;
                    changed = Instant::now();
                    break;
                }
            }
        }
        if stage == 6 && client.inventories.total(93, item) == inventory_before - 1 {
            if let Some((&id, _)) = client.inventories.containers.iter().find(|(id, _)| {
                **id != 93
                    && **id != 94
                    && client.inventories.total(**id, item)
                        == bank_before.get(*id).copied().unwrap_or(0) + 1
            }) {
                bank_id = id;
                println!("Server confirmed deposit of one item {item}: inventory {}, bank container {id} count {}",inventory_before-1,client.inventories.total(id,item));
                stage = 7;
                changed = Instant::now();
            }
        }
        if stage == 8
            && client.inventories.total(93, item) == inventory_before
            && client.inventories.total(bank_id, item)
                == bank_before.get(&bank_id).copied().unwrap_or(0)
        {
            println!("Server confirmed withdrawal of one item {item}; inventory and bank quantities restored");
            client.menu_action(Action::Close)?;
            live.poll(&mut client)?;
            live.session.close();
            return Ok(());
        }
        if changed.elapsed() > Duration::from_secs(15) {
            println!(
                "Waiting stage {stage}, player {:?}, groups {:?}",
                client.data.position,
                client
                    .interfaces
                    .children
                    .values()
                    .map(|s| s.group)
                    .collect::<Vec<_>>()
            );
            println!("Inventory totals {:?}; recent messages {:?}",client.inventories.containers.keys().map(|id|(*id,client.inventories.total(*id,item))).collect::<Vec<_>>(),client.game_messages.iter().rev().take(3).map(|m|&m.text).collect::<Vec<_>>());
            let layout = client.interfaces.layout(client.canvas)?;
            for id in &layout.order {
                let r = &client.interfaces.runtime[id];
                if [219, 231].contains(&(r.logical_id >> 16)) || r.item == item {
                    println!(
                        "dialogue widget {id}: {:?}, hidden={}, clip={:?}, flags={}",
                        client.interfaces.widgets[id].text,
                        layout.widgets[id].hidden,
                        layout.widgets[id].clip,
                        client.interfaces.interaction_flags(*id)
                    );
                }
            }
            changed = Instant::now();
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    live.session.close();
    Err(format!("Dialogue/bank acceptance timed out at stage {stage}").into())
}
