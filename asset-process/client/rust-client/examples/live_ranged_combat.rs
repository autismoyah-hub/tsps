//! Real ranged packet/synchronization/render acceptance on Agent 1's isolated server.
use gielinor_client::{
    client_state::ClientState, context_menu::Action, disk_cache::DiskCache,
    live_runtime::Connection,
};
use std::{
    collections::HashSet,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    time::{Duration, Instant},
};
fn teleport(tile: [i32; 2]) -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = TcpStream::connect("127.0.0.1:43618")?;
    socket.set_read_timeout(Some(Duration::from_secs(5)))?;
    writeln!(
        socket,
        "cmd rustclient239 tele 0 {} {} {} {}",
        tile[0] >> 6,
        tile[1] >> 6,
        tile[0] & 63,
        tile[1] & 63
    )?;
    let mut reply = String::new();
    BufReader::new(socket).read_line(&mut reply)?;
    if !reply.starts_with("OK") {
        return Err(format!("isolated teleport rejected: {reply}").into());
    }
    Ok(())
}
fn step(
    client: &mut ClientState,
    live: &mut Connection,
    next: &mut Instant,
) -> Result<(), Box<dyn std::error::Error>> {
    live.poll(client)?;
    if !live.scripts_busy() && client.interfaces.scripts.is_empty() {
        client.tick_interfaces(false)?;
    }
    client.input.begin_cycle();
    while Instant::now() >= *next {
        client.tick_actor_sequences()?;
        if let (Some((scene, _)), Some((target, plane))) = (&live.world, client.camera_focus()) {
            client.camera.tick_follow(
                scene,
                target,
                plane,
                client.preferences.camera_follow_height,
            );
            client.camera.tick_locked(scene, plane);
        }
        *next += Duration::from_millis(20);
    }
    Ok(())
}
fn equip(
    client: &mut ClientState,
    live: &mut Connection,
    next: &mut Instant,
    item: i32,
    slot: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    if client.inventories.item(94, slot).id == item {
        return Ok(());
    }
    if client.inventories.total(93, item) == 0 {
        let mut socket = TcpStream::connect("127.0.0.1:43618")?;
        socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        writeln!(
            socket,
            "cmd rustclient239 invadd {item} {}",
            if item == 882 { 100 } else { 1 }
        )?;
        let mut reply = String::new();
        BufReader::new(socket).read_line(&mut reply)?;
        if !reply.starts_with("OK") {
            return Err(format!("supply rejected: {reply}").into());
        }
    }
    let ready_deadline = Instant::now() + Duration::from_secs(30);
    let tab = loop {
        step(client, live, next)?;
        let layout = client.interfaces.layout(client.canvas)?;
        if let Some(tab) = layout.order.iter().rev().find_map(|id| {
            let p = &layout.widgets[id];
            let w = &client.interfaces.widgets[id];
            let point = [
                p.absolute[0] + p.geometry.size[0] / 2,
                p.absolute[1] + p.geometry.size[1] / 2,
            ];
            (!p.hidden
                && w.actions.iter().any(|a| a == "Inventory")
                && layout.hit_test(point).any(|hit| hit == *id))
            .then_some((*id, point))
        }) {
            break tab;
        }
        assert!(
            Instant::now() < ready_deadline,
            "Inventory tab stayed hidden"
        );
        std::thread::sleep(Duration::from_millis(20));
    };
    println!("Inventory pointer click widget {} at {:?}", tab.0, tab.1);
    client.input.pointer = tab.1;
    client.input.button(0, true);
    client.tick_interfaces(false)?;
    client.input.begin_cycle();
    client.input.button(0, false);
    client.tick_interfaces(false)?;
    client.input.begin_cycle();
    let tab_entry = client
        .interface_menu(tab.1)?
        .into_iter()
        .find(|e| e.option == "Inventory")
        .ok_or("Inventory menu action missing after pointer event")?;
    println!("Dispatch Inventory tab {:?}", tab_entry.action);
    client.menu_action(tab_entry.action)?;
    let deadline = Instant::now() + Duration::from_secs(25);
    let mut sent = false;
    while client.inventories.item(94, slot).id != item {
        step(client, live, next)?;
        if !sent && !live.scripts_busy() {
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
                if let Some(entry) = client.interface_menu(point)?.into_iter().find(|e| {
                    (e.option == "Wield" || e.option == "Wear")
                        && match e.action {
                            Action::Interface { id, operation, .. } => {
                                client.interfaces.operation_mask(id) & (1 << (operation - 1)) != 0
                            }
                            _ => false,
                        }
                }) {
                    println!(
                        "Equipment action {} item {item}: {:?}",
                        entry.option, entry.action
                    );
                    if let Action::Interface { id, operation, .. } = &entry.action {
                        let r = &client.interfaces.runtime[id];
                        println!(
                            "Equipment widget {}:{} item {} mask {:08x} operation {}",
                            r.logical_id,
                            r.child_index,
                            r.item,
                            client.interfaces.operation_mask(*id),
                            operation
                        );
                    }
                    client.menu_action(entry.action)?;
                    sent = true;
                    break;
                }
            }
        }
        assert!(
            Instant::now() < deadline,
            "equipment timeout for item {item}; sent {sent}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    println!("Server confirmed equipment item {item} slot {slot}");
    Ok(())
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
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
    client.canvas = [765, 503];
    client.camera.pitch = 2048;
    let mut next = Instant::now();
    let deadline = Instant::now() + Duration::from_secs(90);
    while !live.session.is_established()
        || live.world.is_none()
        || live.scripts_busy()
        || !client
            .interfaces
            .widgets
            .values()
            .any(|w| w.actions.iter().any(|a| a == "Inventory"))
    {
        step(&mut client, &mut live, &mut next)?;
        assert!(Instant::now() < deadline, "login timeout");
        std::thread::sleep(Duration::from_millis(20));
    }
    let prior_weapon = client.inventories.item(94, 3).id;
    equip(&mut client, &mut live, &mut next, 841, 3)?;
    equip(&mut client, &mut live, &mut next, 882, 13)?;
    teleport([3233, 3230])?;
    let deadline = Instant::now() + Duration::from_secs(45);
    loop {
        step(&mut client, &mut live, &mut next)?;
        if client.data.position[1..] == [3233, 3230]
            && live.world.as_ref().is_some_and(|(s, _)| s.base_x < 3300)
            && !live.scripts_busy()
        {
            break;
        }
        assert!(Instant::now() < deadline, "combat region timeout");
        std::thread::sleep(Duration::from_millis(20));
    }
    let (index, operation, name, tile) = client
        .npc_state
        .npcs
        .iter()
        .filter_map(|(&index, n)| {
            let d = client.npcs.get(&(n.id as u32))?;
            if !(1..=5).contains(&d.combat) {
                return None;
            }
            let op = d.operations.iter().find(|o| {
                o.sub.is_none() && o.condition.is_none() && o.text.eq_ignore_ascii_case("Attack")
            })?;
            Some((index, op.index + 1, d.name.clone(), n.position))
        })
        .min_by_key(|(_, _, _, p)| (p[0] - 3233).abs() + (p[1] - 3230).abs())
        .ok_or("no low-level attackable NPC in loaded region")?;
    let scene = &live.world.as_ref().unwrap().0;
    let local = [tile[0] - scene.base_x, tile[1] - scene.base_y];
    let direction = [[1, 0], [-1, 0], [0, 1], [0, -1]]
        .into_iter()
        .find(|&d| gielinor_client::route::can_step(&scene.collision[0], local, 1, d))
        .ok_or("no open adjacent target tile")?;
    let stand = [tile[0] + direction[0], tile[1] + direction[1]];
    teleport(stand)?;
    let deadline = Instant::now() + Duration::from_secs(20);
    while client.data.position[1..] != stand || live.scripts_busy() {
        step(&mut client, &mut live, &mut next)?;
        assert!(Instant::now() < deadline, "adjacent teleport timeout");
        std::thread::sleep(Duration::from_millis(20));
    }
    client.game_messages.clear();
    let before = client.outbound.len();
    client.menu_action(Action::Npc {
        index,
        operation,
        sub: 0,
    })?;
    assert!(
        client.outbound.len() > before,
        "Attack action emitted no packet"
    );
    println!("Attack {name} index {index}, operation {operation}, NPC tile {tile:?}, player tile {stand:?}; queued opcodes {:?}",client.outbound.iter().map(|p|p.opcode).collect::<Vec<_>>());
    let started = Instant::now();
    let mut hits = HashSet::new();
    let mut animations = HashSet::new();
    let mut bars = HashSet::new();
    let mut frames = 0;
    let mut npc_damage = 0;
    let mut player_damage = 0;
    let mut removed = false;
    let mut projectiles = HashSet::new();
    let mut motion_samples = HashSet::new();
    let mut expired = HashSet::new();
    while started.elapsed() < Duration::from_secs(35) {
        step(&mut client, &mut live, &mut next)?;
        for p in &client.projectiles {
            let key = (p.request.id, p.request.start, p.request.end);
            if projectiles.insert(key) {
                println!(
                    "Server projectile {key:?} source {:?} target {:?}",
                    p.request.source, p.request.target
                );
            }
            if p.motion.moving {
                motion_samples.insert((key, p.motion.position.map(|x| x.to_bits())));
            }
        }
        for &key in &projectiles {
            if client.actor_cycle > key.2
                && !client
                    .projectiles
                    .iter()
                    .any(|p| (p.request.id, p.request.start, p.request.end) == key)
            {
                expired.insert(key);
            }
        }
        if let Some(p) = client.player_state.as_ref().and_then(|s| s.local_actor()) {
            if p.sequence.id >= 0 {
                animations.insert(p.sequence.id);
            }
            for h in &p.overlays.hits.slots {
                if hits.insert((false, h.kind, h.value, h.end)) {
                    player_damage += h.value.max(0);
                    println!("Player hit kind {} value {}", h.kind, h.value);
                }
            }
        }
        if let Some(n) = client.npc_state.npcs.get(&index) {
            for h in &n.overlays.hits.slots {
                if hits.insert((true, h.kind, h.value, h.end)) {
                    npc_damage += h.value.max(0);
                    println!("NPC hit kind {} value {}", h.kind, h.value);
                }
            }
            for b in &n.overlays.bars.entries {
                for u in &b.updates {
                    bars.insert((b.definition.id, u.start, u.fill));
                }
            }
        } else {
            removed = true;
        }
        if live.world.is_some() {
            let raster = live.paint(&mut client, 765, 503)?;
            frames += 1;
            if frames == 10 || frames % 50 == 0 {
                std::fs::write("work/live-ranged-combat.bmp", raster.bmp()?)?;
            }
        }
        if (started.elapsed() > Duration::from_secs(15) || removed)
            && npc_damage > 0
            && hits.iter().filter(|h| h.0).count() >= 2
            && !animations.is_empty()
            && !bars.is_empty()
            && !expired.is_empty()
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let npc_hits = hits.iter().filter(|h| h.0).count();
    println!("Combat observations: {frames} frames, {npc_hits} NPC hits, NPC damage {npc_damage}, player damage {player_damage}, {} health updates, animations {animations:?}, removed {removed}, {:?}",bars.len(),started.elapsed());
    for message in &client.game_messages {
        println!("Combat message: {}", message.text);
    }
    if npc_hits < 2 || npc_damage == 0 || animations.is_empty() || bars.is_empty() {
        return Err("ranged acceptance lacked confirmed hit/health/animation evidence".into());
    }
    println!(
        "Ranged projectile observations: {} unique, {} moving samples, {} expired",
        projectiles.len(),
        motion_samples.len(),
        expired.len()
    );
    if projectiles.is_empty() || motion_samples.len() < 2 || expired.is_empty() {
        return Err("ranged acceptance lacked projectile receipt/motion/expiry".into());
    }
    teleport([10422, 123])?;
    if prior_weapon >= 0 && prior_weapon != 841 {
        equip(&mut client, &mut live, &mut next, prior_weapon, 3)?;
    }
    live.session.close();
    println!("Real ranged acceptance passed; dedicated account returned to hub");
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = std::panic::catch_unwind(run);
    let restore = teleport([10422, 123]);
    match result {
        Ok(result) => result.and(restore),
        Err(panic) => std::panic::resume_unwind(panic),
    }
}
