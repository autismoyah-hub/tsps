//! Repeated real-server rebuild acceptance on Agent 1's isolated account/ports.
use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection};
use std::{
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
fn step(client: &mut ClientState, live: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
    live.poll(client)?;
    client.tick_actor_sequences()?;
    if let (Some((scene, _)), Some((target, plane))) = (&live.world, client.camera_focus()) {
        if client.camera.mode == 0 {
            client.camera.tick_follow(
                scene,
                target,
                plane,
                client.preferences.camera_follow_height,
            );
        }
        client.camera.tick_locked(scene, plane);
    }
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
    let deadline = Instant::now() + Duration::from_secs(90);
    while !live.session.is_established() || live.world.is_none() || live.scripts_busy() {
        step(&mut client, &mut live)?;
        assert!(Instant::now() < deadline, "initial login timeout");
        std::thread::sleep(Duration::from_millis(20));
    }
    let started = Instant::now();
    let mut frames = 0;
    for (stage, tile) in [[3210, 3210], [10422, 123], [3222, 3218], [10422, 123]]
        .into_iter()
        .enumerate()
    {
        let old_base = client.world_base;
        let anchor =
            |point: [i32; 2], base: [i32; 2]| [point[0] + base[0] * 128, point[1] + base[1] * 128];
        let old_focus = anchor([client.camera.focus[0], client.camera.focus[2]], old_base);
        let old_eye = anchor([client.camera.eye[0], client.camera.eye[2]], old_base);
        let old_tracked = anchor(client.camera.tracked, old_base);
        let mut check_free = stage % 2 == 0;
        if check_free {
            client.camera.mode = 1;
            client.camera.locked = true;
        }
        teleport(tile)?;
        let deadline = Instant::now() + Duration::from_secs(60);
        let mut rendered = 0;
        while rendered < 8 {
            step(&mut client, &mut live)?;
            assert!(
                Instant::now() < deadline,
                "rebuild stage {stage} timed out at {:?}",
                client.data.position
            );
            if check_free && client.world_base != old_base {
                assert_eq!(
                    anchor(
                        [client.camera.focus[0], client.camera.focus[2]],
                        client.world_base
                    ),
                    old_focus
                );
                assert_eq!(
                    anchor(
                        [client.camera.eye[0], client.camera.eye[2]],
                        client.world_base
                    ),
                    old_eye
                );
                assert_eq!(
                    anchor(client.camera.tracked, client.world_base),
                    old_tracked
                );
                assert!(!client.camera.locked, "region rebuild retained camera lock");
                assert_eq!(client.camera.mode, 1, "rebuild left free-camera mode");
                println!("Region stage {stage}: free-camera world-space eye/focus/tracked point preserved; lock released");
                client.camera.mode = 0;
                check_free = false;
                continue;
            }
            let ready = client
                .player_state
                .as_ref()
                .and_then(|s| s.local_actor())
                .is_some_and(|p| p.position[1..] == tile)
                && live.world.as_ref().is_some_and(|(s, _)| {
                    [s.base_x, s.base_y] == client.world_base
                        && tile[0] >= s.base_x
                        && tile[0] < s.base_x + s.width as i32
                        && tile[1] >= s.base_y
                        && tile[1] < s.base_y + s.height as i32
                })
                && !live.scripts_busy();
            if ready {
                let raster = live.paint(&mut client, 765, 503)?;
                let visible = raster
                    .pixels()
                    .iter()
                    .filter(|&&c| c != 0 && c != 0xff000000 && c != 0xff10151a)
                    .count();
                assert!(
                    visible > 1000,
                    "empty rebuilt frame stage {stage}: {visible}"
                );
                assert!(
                    live.pick_where(|_| true).is_some(),
                    "no pickable rebuilt geometry"
                );
                assert!(
                    live.minimap_view(&client).is_some(),
                    "rebuilt minimap missing"
                );
                rendered += 1;
                frames += 1;
                if rendered == 8 {
                    let (scene, mesh) = live.world.as_ref().unwrap();
                    assert!(
                        scene.missing_regions.is_empty(),
                        "missing rebuilt cache regions"
                    );
                    assert_eq!(client.player_state.as_ref().unwrap().local_view, 0);
                    println!("Region stage {stage}: tile {tile:?}, base {:?}, {} objects, {} triangles, {visible} visible pixels, {} NPCs",
                        client.world_base,scene.objects.len(),mesh.triangles.len(),client.npc_state.npcs.len());
                    std::fs::write(format!("work/region-transition-{stage}.bmp"), raster.bmp()?)?;
                }
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    println!("Repeated region acceptance passed: 4 server teleports, {frames} rendered frames, {} packets, {:?}, single login",
        live.packets,started.elapsed());
    live.session.close();
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = std::panic::catch_unwind(run);
    // Restore only the dedicated account on errors and assertion failures too.
    let restore = teleport([10422, 123]);
    match result {
        Ok(result) => result.and(restore),
        Err(panic) => std::panic::resume_unwind(panic),
    }
}
