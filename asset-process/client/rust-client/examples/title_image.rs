use gielinor_client::{disk_cache::DiskCache, image_decode, raster::Raster, Result};
fn main() -> Result<()> {
    let mut cache = DiskCache::open("work/kotlin-world/.data/cache/js5", 64 << 20)?;
    let bytes = cache.named_file(10, "title.jpg", "")?;
    std::fs::write("work/title.jpg", &bytes)?;
    let image = image_decode::decode(&bytes, 1 << 20)?;
    let raster = Raster::from_pixels(image.size[0], image.size[1], image.pixels.clone())?;
    std::fs::write("work/title-native.bmp", raster.bmp()?)?;
    let mut assets = Vec::new();
    let mut append = |bytes: Vec<u8>| {
        assets.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        assets.extend(bytes);
    };
    append(bytes);
    for name in ["logo", "titlebox", "titlebutton"] {
        append(cache.named_file(8, name, "")?);
    }
    let id = cache
        .reference(13)?
        .groups
        .iter()
        .find(|g| g.name_hash == Some(gielinor_client::disk_cache::name_hash("b12_full")))
        .unwrap()
        .id;
    append(cache.file(13, id, 0, [0; 4])?);
    append(cache.file(8, id, 0, [0; 4])?);
    std::fs::write("tests/fixtures/title-assets.bin", assets)?;
    let mut form = Vec::new();
    let mut append = |bytes: Vec<u8>| {
        form.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        form.extend(bytes);
    };
    let id = cache
        .reference(13)?
        .groups
        .iter()
        .find(|g| g.name_hash == Some(gielinor_client::disk_cache::name_hash("p11_full")))
        .unwrap()
        .id;
    append(cache.file(13, id, 0, [0; 4])?);
    append(cache.file(8, id, 0, [0; 4])?);
    for name in [
        "options_radio_buttons,0",
        "options_radio_buttons,4",
        "options_radio_buttons,2",
        "options_radio_buttons,6",
    ] {
        append(cache.named_file(8, name, "")?);
    }
    let id = cache
        .reference(13)?
        .groups
        .iter()
        .find(|g| g.name_hash == Some(gielinor_client::disk_cache::name_hash("p12_full")))
        .unwrap()
        .id;
    append(cache.file(13, id, 0, [0; 4])?);
    append(cache.file(8, id, 0, [0; 4])?);
    std::fs::write("tests/fixtures/title-form-assets.bin", form)?;
    let mut title = Raster::new(765, 503, 1 << 20)?;
    gielinor_client::title::Assets::load(&mut cache, 0)?.notification(
        &mut title,
        ["", "You were disconnected from the server.", ""],
    )?;
    std::fs::write("work/title-notification.bmp", title.bmp()?)?;
    if let Ok(expected) = std::fs::read("work/title-jvm.bin") {
        let mut values = expected
            .chunks_exact(4)
            .map(|b| u32::from_be_bytes(b.try_into().unwrap()));
        assert_eq!(values.next(), Some(image.size[0] as u32));
        assert_eq!(values.next(), Some(image.size[1] as u32));
        let expected: Vec<_> = values.collect();
        assert_eq!(expected.len(), image.pixels.len());
        let mut different = 0usize;
        let mut max_delta = 0u32;
        let mut total_delta = 0u64;
        for (&a, &b) in image.pixels.iter().zip(&expected) {
            different += usize::from(a != b);
            for shift in [0, 8, 16] {
                let delta = ((a >> shift) & 255).abs_diff((b >> shift) & 255);
                max_delta = max_delta.max(delta);
                total_delta += delta as u64;
            }
        }
        println!("JPEG JVM comparison: {different}/{} pixels differ, maximum channel delta {max_delta}, mean channel delta {}", expected.len(), total_delta as f64 / (expected.len() * 3) as f64);
        assert_eq!(different, 0, "native JPEG decode differs from target JVM");
    }
    println!("Decoded title image {}x{}", image.size[0], image.size[1]);
    Ok(())
}
