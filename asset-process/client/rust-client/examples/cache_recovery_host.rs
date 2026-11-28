//! Actual-cache recovery and per-widget readiness without native audio/window IO.
use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, interface_packet::Update,
    interface_render::Renderer, raster::Raster,
};
use std::time::{Duration, Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("expected CACHE_DIRECTORY IP:PORT".into());
    }
    let mut cache = DiskCache::open_remote(&args[0], 64 << 20, args[1].parse()?)?;
    cache.wait_for_recovery(true);
    gielinor_client::huffman::Huffman::new(&cache.named_file(10, "huffman", "")?)?;
    for archive in [3, 23, 12, 13] {
        if archive == 23 && !cache.has_archive(23)? {
            continue;
        }
        assert!(cache.archive_ready(archive)?);
    }
    println!("Startup archives ready: {:?}", cache.recovery_stats());
    let mut client = ClientState::load(cache)?;
    client.interface(Update::OpenTop(12))?;
    while let Some(event) = client.next_interface_event()? {
        client.run_event(event)?;
    }
    let mut renderer = Renderer::load(
        &mut client.scripts.source.cache,
        client.preferences.device.brightness,
    )?;
    client.scripts.source.cache.wait_for_recovery(false);
    let layout = client.interfaces.layout(client.canvas)?;
    let mut raster = Raster::new(
        client.canvas[0] as usize,
        client.canvas[1] as usize,
        16 << 20,
    )?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut observed_pending = false;
    loop {
        raster.clear(0xff29251f);
        let report = renderer.draw_with_variables(
            &mut client.scripts.source.cache,
            &client.items,
            &client.interfaces,
            &layout,
            &mut raster,
            Some(&client.scripts.variables),
        )?;
        if !report.pending.is_empty() {
            assert!(
                report.drawn > 0,
                "pending widget blocked the entire interface"
            );
            observed_pending = true;
        } else {
            assert!(
                observed_pending,
                "expected a cold sprite request in this pinned stale cache"
            );
            assert!(report.drawn > 0);
            println!("Recovered bank interface: {} widgets drawn, pending empty; unrelated widgets drew during pending; JS5 {:?}",report.drawn,client.scripts.source.cache.recovery_stats());
            break;
        }
        if Instant::now() >= deadline {
            return Err(format!("pending interface resources: {:?}", report.pending).into());
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}
