use gielinor_client::{
    audio_runtime::Audio, device_options::DeviceOptions, disk_cache::DiskCache,
    music_request::Request, raster::Raster, title::Assets, Result,
};
fn pump(
    audio: &mut Audio,
    cache: &mut DiskCache,
    options: &DeviceOptions,
    cycles: usize,
) -> Result<usize> {
    let mut nonzero = 0;
    for _ in 0..cycles {
        audio.tick(cache, options)?;
        nonzero += audio.mix_stereo(441).iter().filter(|&&v| v != 0).count();
    }
    Ok(nonzero)
}
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let mut audio = Audio::default();
    let mut options = DeviceOptions::default();
    audio.music.title(&mut cache, false)?;
    let title = pump(&mut audio, &mut cache, &options, 200)?;
    assert!(title > 22050);
    audio.music.title(&mut cache, true)?;
    pump(&mut audio, &mut cache, &options, 10)?;
    assert_eq!(pump(&mut audio, &mut cache, &options, 25)?, 0);
    let assets = Assets::load(&mut cache, 0)?;
    let mut on = Raster::new(765, 503, 765 * 503)?;
    let mut off = Raster::new(765, 503, 765 * 503)?;
    assets.draw_mute(&mut on, false)?;
    assets.draw_mute(&mut off, true)?;
    assert_ne!(on.pixels(), off.pixels());
    audio.music.title(&mut cache, false)?;
    assert!(pump(&mut audio, &mut cache, &options, 200)? > 22050);
    options.set_game(7, 0)?;
    pump(&mut audio, &mut cache, &options, 10)?;
    assert_eq!(pump(&mut audio, &mut cache, &options, 25)?, 0);
    options.set_game(7, 100)?;
    let background = cache
        .reference(6)?
        .groups
        .iter()
        .find(|g| g.name_hash == Some(gielinor_client::disk_cache::name_hash("scape main")))
        .unwrap()
        .id;
    Request::Songs(vec![background as i32], [0, 0, 0, 10])
        .apply(&mut audio.music, options.get_game(7)?);
    assert!(pump(&mut audio, &mut cache, &options, 200)? > 22050);
    let jingle = cache.reference(11)?.groups[0].id;
    Request::Jingle(jingle as i32, 0).apply(&mut audio.music, options.get_game(7)?);
    assert!(audio.music.jingle);
    let mut cycles = 0;
    while audio.music.jingle && cycles < 6000 {
        pump(&mut audio, &mut cache, &options, 1)?;
        cycles += 1;
    }
    assert!(!audio.music.jingle, "Jingle did not finish in 120 seconds");
    let resumed = pump(&mut audio, &mut cache, &options, 200)?;
    assert!(resumed > 22050, "Background did not resume");
    Request::Stop(0, 25).apply(&mut audio.music, options.get_game(7)?);
    pump(&mut audio, &mut cache, &options, 40)?;
    assert_eq!(pump(&mut audio, &mut cache, &options, 25)?, 0);
    println!("Native cache music runtime passed: title {title} nonzero samples, mute sprites and silence, volume stop, jingle {jingle} completed in {cycles} cycles, background resumed {resumed} nonzero samples, fade stop");
    Ok(())
}
