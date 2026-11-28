use gielinor_client::{buffer::Reader, camera::State};
fn main() -> gielinor_client::Result<()> {
    let bytes = include_bytes!("../tests/fixtures/camera-coordinate-jvm.bin");
    let mut client = gielinor_client::client_state::ClientState::load(
        gielinor_client::disk_cache::DiskCache::open(
            "work/kotlin-world/.data/cache/js5",
            64 << 20,
        )?,
    )?;
    use gielinor_client::{protocol::Packet, scene::Scene};
    let scene = Scene::empty(3200, 3200).unwrap();
    let mut r = Reader::new(bytes);
    for case in 0..r.u32().unwrap() {
        client.camera = State::default();
        let state = &mut client.camera;
        state.unlocked_pitch = r.u8().unwrap() != 0;
        state.eye = std::array::from_fn(|_| r.u32().unwrap() as i32);
        state.rendered_angles = std::array::from_fn(|_| r.u32().unwrap() as i32);
        state.target_heights = [r.u32().unwrap() as i32; 2];
        let opcode = r.u16().unwrap();
        let len = r.u8().unwrap() as usize;
        let packet = Packet {
            opcode,
            payload: r.bytes(len).unwrap().to_vec(),
        };
        client.apply(gielinor_client::game_packet::decode(&packet)?.unwrap())?;
        let state = &mut client.camera;
        state.resolve_snaps(&scene, 0);
        for step in 0..25 {
            if step != 0 {
                state.tick_locked(&scene, 0);
            }
            let eye: [i32; 3] = std::array::from_fn(|_| r.u32().unwrap() as i32);
            let angles: [i32; 2] = std::array::from_fn(|_| r.u32().unwrap() as i32);
            assert_eq!(state.eye, eye, "case {case} step {step} position");
            assert_eq!(
                state.rendered_angles, angles,
                "case {case} step {step} angles"
            );
        }
    }
    assert_eq!(r.remaining(), 0);
    println!("25600 original packet141 camera states passed through ClientState with actual-cache bootstrap");
    Ok(())
}
