//! Isolated-account login followed by a local graphics draw-phase replay.
//! No synthetic world state or interactions are sent to the server.
use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection};
use std::time::{Duration, Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
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
        assert!(Instant::now() < deadline, "login/scene timeout: established={} closed={} world={} scripts_busy={} packets={} title={:?}", live.session.is_established(), live.session.is_closed(), live.world.is_some(), live.scripts_busy(), live.packets, live.title.as_ref().map(|title| (&title.screen, &title.lines)));
        std::thread::sleep(Duration::from_millis(20));
    }
    // Freeze network polling so server state cannot overwrite the local scenario.
    client.canvas = [765, 503];
    // Keep upper root floors out of this draw-order scenario.
    client.preferences.device.roofs_hidden = true;
    // Settle the real login before constructing local graphics scenarios.
    live.paint(&mut client, 765, 503)?;
    use gielinor_client::{buffer::Reader, game_packet, protocol::Packet, world_entity::State};
    let players = client.player_state.as_ref().unwrap();
    let player = players.root_actor(players.local).unwrap();
    let fine = [
        player.path.position[0] - client.world_base[0] * 128,
        player.path.position[1] - client.world_base[1] * 128,
    ];
    client.active_world = 0;
    client.player_state.as_mut().unwrap().local_view = 0;
    client.world_entities.clear();
    client.world_entities.insert(0, State::default());
    let mut r = Reader::new(include_bytes!(
        "../tests/fixtures/scene-draw-identity-jvm.bin"
    ));
    assert_eq!(r.u32()?, 16);
    for case in 0..16 {
        for _ in 0..4 {
            r.u32()?;
        }
        let n = r.u32()? as usize;
        let initial = r.bytes(n)?.to_vec();
        let n = r.u32()? as usize;
        let replacement = r.bytes(n)?.to_vec();
        for _ in 0..5 {
            r.u32()?;
        }
        let stamp = r.u32()? as i32;
        let scale = f32::from_bits(r.u32()?);
        assert_ne!(stamp, 100);
        assert_eq!(scale, 1.0);
        client.actor_cycle = 100 + case;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 122,
                payload: initial.clone(),
            })?
            .unwrap(),
        )?;
        hide(&mut live, &mut client, fine)?;
        client.apply(
            game_packet::decode(&Packet {
                opcode: 122,
                payload: replacement,
            })?
            .unwrap(),
        )?;
        limit(&mut client, 0)?;
        live.paint(&mut client, 765, 503)?;
        assert_eq!(
            live.minimap_view(&client).unwrap().entities.len(),
            1,
            "case{case}: replacement inherits hidden minimap state without redraw"
        );
        hide(&mut live, &mut client, fine)?;
        // Replace the old scene and list another overlapping late scene first.
        let mut both = vec![0];
        let mut other = initial[1..].to_vec();
        other[1] = 8;
        both.extend(other);
        both.extend(&initial[1..]);
        client.apply(
            game_packet::decode(&Packet {
                opcode: 122,
                payload: both,
            })?
            .unwrap(),
        )?;
        for id in [7, 8] {
            let e = client
                .world_entities
                .get_mut(&0)
                .unwrap()
                .entities
                .get_mut(&id)
                .unwrap();
            e.owner_type = 1;
            e.pose.x = 100000;
            e.pose.z = 100000;
            e.definition.as_mut().unwrap().bounds = [0, 0, 512, 512];
        }
        assert_eq!(client.world_entities[&0].visible, vec![8, 7]);
        limit(&mut client, 1)?;
        live.paint(&mut client, 765, 503)?;
        assert_eq!(
            live.minimap_view(&client).unwrap().entities.len(),
            2,
            "case{case}: replacement inherits a previous scene submission stamp"
        );
    }
    assert_eq!(r.remaining(), 0);
    println!("16 original replacement draw identities match actual-cache live paint: hidden-state reset without redraw and same-cycle prior-submission reset");
    Ok(())
}
fn limit(client: &mut ClientState, value: i32) -> gielinor_client::Result<()> {
    let mut stack = gielinor_client::vm::Stacks::default();
    stack.push_int(value)?;
    client.world_entity_limit.execute(7900, &mut stack)?;
    Ok(())
}
fn hide(
    live: &mut Connection,
    client: &mut ClientState,
    fine: [i32; 2],
) -> gielinor_client::Result<()> {
    let e = client
        .world_entities
        .get_mut(&0)
        .unwrap()
        .entities
        .get_mut(&7)
        .unwrap();
    e.owner_type = 1;
    e.pose.x = fine[0];
    e.pose.z = fine[1];
    e.pose.angle = 0;
    e.definition.as_mut().unwrap().bounds = [0, 0, 512, 512];
    limit(client, 30)?;
    live.paint(client, 765, 503)?;
    assert_eq!(
        live.minimap_view(client).unwrap().entities.len(),
        0,
        "old child must be overlap-hidden"
    );
    Ok(())
}
