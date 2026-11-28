use gielinor_client::{
    ambient_sound::Definition, client_state::ClientState, disk_cache::DiskCache, game_packet,
    protocol::Packet, Result,
};
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut c = ClientState::load(cache)?;
    let ids: Vec<_> = c
        .scripts
        .source
        .cache
        .reference(2)?
        .groups
        .iter()
        .find(|g| g.id == 15)
        .map(|g| g.files.iter().map(|f| f.id).collect())
        .unwrap_or_default();
    let mut definitions = 0;
    let mut pcm_nonzero = 0;
    for id in ids.iter().take(32) {
        let definition = Definition::load(&mut c.scripts.source.cache, *id)?;
        let mut payload = vec![128];
        payload.extend((*id as u16).to_be_bytes());
        let update = game_packet::decode(&Packet {
            opcode: 145,
            payload,
        })?
        .expect("start route");
        c.prepare_packet_assets(&update)?;
        c.apply(update)?;
        assert_eq!(c.audio.ambient.current_id(), Some(*id));
        let legacy = c.audio.ambient.legacy();
        c.preferences.device.option22 = !legacy;
        c.apply(game_packet::Update::AmbientStart {
            id: *id as u16,
            fade: true,
        })?;
        assert_eq!(c.audio.ambient.legacy(), legacy);
        c.audio
            .tick(&mut c.scripts.source.cache, &c.preferences.device)?;
        let mixer = c.audio.mixer();
        let pcm = std::thread::spawn(move || mixer.mix_stereo(44100))
            .join()
            .unwrap();
        pcm_nonzero += pcm.iter().filter(|v| **v != 0).count();
        c.apply(
            game_packet::decode(&Packet {
                opcode: 138,
                payload: vec![0],
            })?
            .expect("stop route"),
        )?;
        assert_eq!(c.audio.ambient.current_id(), None);
        assert!(!c.audio.active());
        assert!(c.audio.mix_stereo(32).iter().all(|v| *v == 0));
        definitions += 1;
        println!("Ambient definition {id}: {} loops, {} random groups; packet application and PCM-worker stop pass", definition.loops.len(), definition.random.len());
    }
    assert!(definitions > 0, "actual group15 definitions required");
    assert!(pcm_nonzero > 0, "actual ambient PCM required");
    println!("Actual-cache ambient host: {definitions} definitions; {pcm_nonzero} nonzero stereo samples; same-ID legacy preservation and immediate stop pass");
    Ok(())
}
