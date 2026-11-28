//! Owned-fixture login followed by local roof preference/render acceptance.
use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, live_runtime::Connection};
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
    // Stop polling: all subsequent camera and preference changes stay local.
    client.canvas = [765, 503];
    let plane = client.data.position[0] as usize;
    let (scene, mesh) = live.world.as_ref().unwrap();
    let floor = mesh
        .triangles
        .iter()
        .find(|t| t.tag >> 63 != 0 && ((t.tag >> 28) & 3) as usize > plane)
        .expect("live cache has an upper floor");
    let target: [i32; 3] =
        std::array::from_fn(|i| (floor.vertices.iter().map(|v| v[i]).sum::<f32>() / 3.0) as i32);
    client.camera.locked = true;
    client.camera.eye = [target[0], target[1] - 1400, target[2] - 900];
    client.camera.rendered_angles =
        gielinor_client::camera::look_angles(client.camera.eye, target, false);
    assert_eq!(client.camera.roof_plane(scene, plane as u8, false), 3);
    let count = mesh.triangles.len();
    let upper = |_: u16, tag: u64| tag >> 63 != 0 && ((tag >> 28) & 3) as usize > plane;
    client.preferences.device.roofs_hidden = false;
    let shown = live.paint(&mut client, 765, 503)?;
    assert!(
        live.pick_where_view(upper).is_some(),
        "upper floor pick missing"
    );
    client.preferences.device.roofs_hidden = true;
    let hidden = live.paint(&mut client, 765, 503)?;
    assert!(
        live.pick_where_view(upper).is_none(),
        "hidden upper floor still pickable"
    );
    let changed = shown
        .pixels()
        .iter()
        .zip(hidden.pixels())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        changed > 100,
        "roof preference must change visible geometry: {changed}"
    );
    client.preferences.device.roofs_hidden = false;
    let restored = live.paint(&mut client, 765, 503)?;
    assert_eq!(shown.pixels(), restored.pixels(), "roof toggle restoration");
    assert_eq!(
        live.world.as_ref().unwrap().1.triangles.len(),
        count,
        "roof toggle rebuilt mesh"
    );
    assert!(
        client.outbound.is_empty(),
        "local replay must not send actions"
    );
    println!("Roof toggle changed {changed} pixels; upper floor picking removed/restored with unchanged {count}-triangle mesh");
    let canonical = (client.camera.eye, client.camera.rendered_angles);
    client
        .camera
        .apply(gielinor_client::camera::Update::Shake([0, 0, 32, 50]));
    client.camera.shake[0].elapsed = 3;
    let shaken = live.paint(&mut client, 765, 503)?;
    let shake_pixels = restored
        .pixels()
        .iter()
        .zip(shaken.pixels())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        shake_pixels > 100,
        "requested camera shake did not reach native rendering"
    );
    assert_eq!(
        (client.camera.eye, client.camera.rendered_angles),
        canonical
    );
    assert_eq!(
        client.camera.shake[0].elapsed, 3,
        "rendering advanced simulation clock"
    );
    client.camera.shake[0].requested = false;
    assert_eq!(
        restored.pixels(),
        live.paint(&mut client, 765, 503)?.pixels()
    );
    println!("Camera shake changed {shake_pixels} pixels, preserved canonical tracking/clock and restored exactly when disabled");
    Ok(())
}
