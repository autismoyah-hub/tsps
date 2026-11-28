//! Cache-backed item/model palette refresh and legacy varp replay.
use gielinor_client::{
    client_state::ClientState,
    disk_cache::DiskCache,
    game_packet::Update,
    item_icon::{Icons, Options},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ClientState::load(DiskCache::open(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
    )?)?;
    let varp = client
        .scripts
        .source
        .cache
        .group(2, 16, [0; 4])?
        .into_iter()
        .find_map(|(id, bytes)| {
            (gielinor_client::menu_settings::definition(&bytes).ok() == Some(1))
                .then_some(id as u16)
        });
    if let Some(varp) = varp {
        for (value, expected) in [
            (1, 0.9f64),
            (2, 0.8),
            (3, 0.7),
            (4, 0.6),
            (0, 0.6),
            (5, 0.6),
        ] {
            client.apply(Update::Varp { id: varp, value })?;
            assert_eq!(
                client.preferences.device.brightness.to_bits(),
                expected.to_bits(),
                "legacy brightness varp {value}"
            );
        }
    } else {
        println!("Cache contains no legacy brightness varp; mapping follows reference fg.gd, cache integration not exercised");
    }
    let cache = &mut client.scripts.source.cache;
    let mut icons = Icons::load(cache, 0.8)?;
    let initial = icons.get(cache, &client.items, 1277, Options::default())?;
    let cached = icons.get(cache, &client.items, 1277, Options::default())?;
    assert!(initial.pixels == cached.pixels);
    assert!(!icons.set_brightness(cache, 0.8)?);
    for gamma in [0.5, 0.7, 0.9, 1.0] {
        assert!(icons.set_brightness(cache, gamma)?);
        let updated = icons.get(cache, &client.items, 1277, Options::default())?;
        let mut fresh = Icons::load(cache, gamma)?;
        let expected = fresh.get(cache, &client.items, 1277, Options::default())?;
        assert!(
            updated.pixels == expected.pixels,
            "gamma {gamma}: refreshed and newly loaded icons differ"
        );
        let changed = updated
            .pixels
            .iter()
            .zip(&initial.pixels)
            .filter(|(a, b)| a != b)
            .count();
        assert!(changed > 10);
        println!("Item brightness {gamma}: {changed} changed pixels, exact fresh-provider match");
        assert!(!icons.set_brightness(cache, gamma)?);
    }
    icons.set_brightness(cache, 0.8)?;
    assert!(
        initial.pixels
            == icons
                .get(cache, &client.items, 1277, Options::default())?
                .pixels,
        "brightness restoration"
    );
    println!("Brightness cache replay passed; icon restoration exact");
    Ok(())
}
