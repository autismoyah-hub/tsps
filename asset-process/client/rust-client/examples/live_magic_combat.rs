//! Real magic packet/synchronization/render acceptance on Agent 1's isolated server.
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
fn supply(client: &ClientState, item: i32) -> Result<(), Box<dyn std::error::Error>> {
    let have = client.inventories.total(93, item);
    if have >= 20 {
        return Ok(());
    }
    let mut socket = TcpStream::connect("127.0.0.1:43618")?;
    socket.set_read_timeout(Some(Duration::from_secs(5)))?;
    writeln!(socket, "cmd rustclient239 invadd {item} {}", 20 - have)?;
    let mut reply = String::new();
    BufReader::new(socket).read_line(&mut reply)?;
    if !reply.starts_with("OK") {
        return Err(format!("rune supply rejected: {reply}").into());
    }
    Ok(())
}
fn wind_strike(
    client: &mut ClientState,
    live: &mut Connection,
    next: &mut Instant,
) -> Result<u32, Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut opened = false;
    loop {
        step(client, live, next)?;
        if !live.scripts_busy() {
            let layout = client.interfaces.layout(client.canvas)?;
            for id in &layout.order {
                let p = &layout.widgets[id];
                let w = &client.interfaces.widgets[id];
                if p.hidden {
                    continue;
                }
                let point = [
                    p.absolute[0] + p.geometry.size[0] / 2,
                    p.absolute[1] + p.geometry.size[1] / 2,
                ];
                if !layout.hit_test(point).any(|hit| hit == *id) {
                    continue;
                }
                if !opened && w.actions.iter().any(|s| s == "Magic") {
                    client.input.pointer = point;
                    client.input.button(0, true);
                    client.tick_interfaces(false)?;
                    client.input.begin_cycle();
                    client.input.button(0, false);
                    client.tick_interfaces(false)?;
                    client.input.begin_cycle();
                    if let Some(entry) = client
                        .interface_menu(point)?
                        .into_iter()
                        .find(|e| e.option == "Magic")
                    {
                        println!("Magic tab {:?}", entry.action);
                        client.menu_action(entry.action)?;
                        opened = true;
                    }
                    break;
                }
                if opened
                    && format!("{} {} {}", w.spell_name, w.data_text, w.text)
                        .to_ascii_lowercase()
                        .contains("wind strike")
                {
                    if let Some(entry) = client
                        .interface_menu(point)?
                        .into_iter()
                        .find(|e| matches!(e.action, Action::SelectSpell(_)))
                    {
                        if let Action::SelectSpell(id) = entry.action {
                            println!("Visible Wind Strike selection {id}: {}", entry.target);
                            client.menu_action(entry.action)?;
                            return Ok(id);
                        }
                    }
                }
            }
        }
        assert!(
            Instant::now() < deadline,
            "visible Wind Strike selection timeout; Magic opened {opened}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}
fn cast(
    client: &mut ClientState,
    spell: u32,
    index: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    client.menu_action(Action::SelectSpell(spell))?;
    let npc = client
        .npc_state
        .npcs
        .get(&index)
        .ok_or("cast target removed")?;
    let entry = client
        .selected_spell
        .as_ref()
        .ok_or("spell selection missing")?
        .world_entry(1, index, npc.position.map(|v| v as u16), "Rat")
        .ok_or("spell cannot target NPC")?;
    let before = client.outbound.len();
    client.menu_action(entry.action)?;
    assert!(
        client.outbound.len() > before,
        "spell cast emitted no packet"
    );
    println!(
        "Wind Strike queued opcodes {:?}",
        client.outbound.iter().map(|p| p.opcode).collect::<Vec<_>>()
    );
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
    supply(&client, 556)?;
    supply(&client, 558)?;
    let rune_deadline = Instant::now() + Duration::from_secs(20);
    while client.inventories.total(93, 556) < 20 || client.inventories.total(93, 558) < 20 {
        step(&mut client, &mut live, &mut next)?;
        assert!(
            Instant::now() < rune_deadline,
            "rune synchronization timeout"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    let runes_before = [
        client.inventories.total(93, 556),
        client.inventories.total(93, 558),
    ];
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
    let (index, _operation, name, tile) = client
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
    let spell = wind_strike(&mut client, &mut live, &mut next)?;
    cast(&mut client, spell, index)?;
    let mut last_cast = Instant::now();
    let mut casts = 1;
    println!("Magic target {name} index {index}, NPC tile {tile:?}, player tile {stand:?}");
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
        if last_cast.elapsed() > Duration::from_secs(3)
            && client.npc_state.npcs.contains_key(&index)
        {
            cast(&mut client, spell, index)?;
            last_cast = Instant::now();
            casts += 1;
        }
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
                std::fs::write("work/live-magic-combat.bmp", raster.bmp()?)?;
            }
        }
        if (started.elapsed() > Duration::from_secs(15) || removed)
            && npc_damage > 0
            && hits.iter().filter(|h| h.0).count() >= 1
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
    if npc_hits < 1 || npc_damage == 0 || animations.is_empty() || bars.is_empty() {
        return Err("magic acceptance lacked confirmed hit/health/animation evidence".into());
    }
    println!(
        "Magic projectile observations: {} unique, {} moving samples, {} expired",
        projectiles.len(),
        motion_samples.len(),
        expired.len()
    );
    if projectiles.is_empty() || motion_samples.len() < 2 || expired.is_empty() {
        return Err("magic acceptance lacked projectile receipt/motion/expiry".into());
    }
    teleport([10422, 123])?;
    let runes_after = [
        client.inventories.total(93, 556),
        client.inventories.total(93, 558),
    ];
    assert!(
        (0..2).all(|i| runes_after[i] < runes_before[i]),
        "magic acceptance requires both rune types consumed"
    );
    println!("Magic rune consumption: {runes_before:?} -> {runes_after:?}, {casts} cast requests");
    live.session.close();
    println!("Real magic acceptance passed; dedicated account returned to hub");
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
