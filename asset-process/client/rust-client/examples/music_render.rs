use gielinor_client::{
    disk_cache::DiskCache, music_player::Player, music_samples::Samples, music_track::Track, Result,
};
use std::{io::Write, sync::Arc};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let track = Track::decode(&cache.named_file(6, "scape main", "")?)?;
    let mut samples = Samples::default();
    let mut player = Player::new(&track.midi, 22050, true)?;
    player.synth.set_program(9, 128);
    for (&id, used) in &track.instruments {
        let instrument = samples.instrument(&mut cache, id as u32, &used.notes)?;
        let loaded = instrument.samples.iter().filter(|s| s.is_some()).count();
        println!("Loaded instrument {id}: {loaded} notes");
        player.synth.instruments.insert(id, Arc::new(instrument));
    }
    let mut pcm = Vec::new();
    let mut peak = 0i32;
    let mut nonzero = 0usize;
    for _ in 0..1500 {
        let mut out = vec![0i32; 882];
        player.mix(&mut out, true);
        for value in out {
            let value = (value.clamp(-8388608, 8388607) >> 8) as i16;
            peak = peak.max((value as i32).abs());
            nonzero += usize::from(value != 0);
            pcm.extend_from_slice(&value.to_le_bytes());
        }
    }
    assert!(nonzero > 22050, "Rendered music unexpectedly silent");
    let mut file = std::fs::File::create("work/music-native-title.wav")?;
    file.write_all(b"RIFF")?;
    file.write_all(&(pcm.len() as u32 + 36).to_le_bytes())?;
    file.write_all(b"WAVEfmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&2u16.to_le_bytes())?;
    file.write_all(&22050u32.to_le_bytes())?;
    file.write_all(&88200u32.to_le_bytes())?;
    file.write_all(&4u16.to_le_bytes())?;
    file.write_all(&16u16.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&(pcm.len() as u32).to_le_bytes())?;
    file.write_all(&pcm)?;
    println!("Rendered 30 seconds of native stereo title music: {nonzero} nonzero samples, peak {peak}, CRC {:08x}",gielinor_client::crypto::crc32(&pcm));
    Ok(())
}
