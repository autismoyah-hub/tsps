//! Actual-cache background model load, unavailable replacement and login lifetime.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    floor, game_packet,
    protocol::Packet,
    render3d::{Camera, Frame},
    writer::Writer,
    Result,
};
fn request(c: &mut ClientState, id: i32) -> Result<()> {
    let mut w = Writer::new();
    w.u32_inverse_middle(id as u32);
    c.apply(
        game_packet::decode(&Packet {
            opcode: 135,
            payload: w.into_vec(),
        })?
        .unwrap(),
    )
}
fn main() -> Result<()> {
    let mut c = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    request(&mut c, 0)?;
    c.skybox.load(&mut c.scripts.source.cache)?;
    assert_eq!(c.skybox.displayed, 0);
    let triangles = c.skybox.triangles.as_ref().unwrap();
    assert!(!triangles.is_empty());
    let count = triangles.len();
    let palette = floor::palette(0.8)?;
    let textures = std::collections::HashMap::new();
    let mut visible = 0;
    for yaw in [0, 4096, 8192, 12288] {
        let mut frame = Frame::new(320, 240)?;
        frame.draw_background(
            &Camera::angles([50, 20, -30], 0, yaw, 180.)?,
            triangles,
            &palette,
            &textures,
        );
        visible += frame.triangles;
    }
    assert!(visible > 0);
    request(&mut c, i32::MAX)?;
    c.skybox.load(&mut c.scripts.source.cache)?;
    assert_eq!(c.skybox.displayed, 0);
    assert_eq!(c.skybox.triangles.as_ref().unwrap().len(), count);
    request(&mut c, -1)?;
    assert!(c.skybox.triangles.is_none());
    request(&mut c, 1)?;
    c.skybox.load(&mut c.scripts.source.cache)?;
    assert_eq!(c.skybox.displayed, 1);
    c.prepare_login()?;
    assert_eq!(c.skybox.requested, 1);
    assert!(c.skybox.triangles.is_none());
    c.skybox.load(&mut c.scripts.source.cache)?;
    assert_eq!(c.skybox.displayed, 1);
    println!("Skybox cache: {count} model faces, {visible} submitted background triangles across four yaw angles; missing replacement, clear and login persistence passed");
    Ok(())
}
