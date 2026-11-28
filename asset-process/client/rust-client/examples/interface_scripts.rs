use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, interface_packet::Update,
    interface_render::Renderer, raster::Raster,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    let cache = DiskCache::open(&a[1], 64 << 20)?;
    let mut client = ClientState::load(cache)?;
    let group = a[2].parse()?;
    client.interface(Update::OpenTop(group))?;
    let mut passed = 0;
    let mut failed = 0;
    while let Some(event) = client.next_interface_event()? {
        match client.run_event(event) {
            Ok(n) => {
                passed += 1;
                println!("Completed initialization script ({n} instructions)");
            }
            Err(e) => {
                failed += 1;
                println!("{e}");
            }
        }
    }
    for _ in 0..a.get(4).map(|v| v.parse::<u32>()).transpose()?.unwrap_or(0) {
        client.tick_interfaces(false)?;
        while let Some(e) = client.script_failures.pop_front() {
            failed += 1;
            println!("{e}");
        }
        client.cycle += 1;
        client.input.begin_cycle();
    }
    let layout = client.interfaces.layout(client.canvas)?;
    let mut renderer = Renderer::load(&mut client.scripts.source.cache, 0.8)?;
    let mut raster = Raster::new(765, 503, 1 << 20)?;
    raster.clear(0xff29251f);
    let report = renderer.draw(
        &mut client.scripts.source.cache,
        &client.items,
        &client.interfaces,
        &layout,
        &mut raster,
    )?;
    std::fs::write(&a[3], raster.bmp()?)?;
    println!(
        "Scripts passed={passed}, failed={failed}; {} widgets; {report:?}",
        layout.widgets.len()
    );
    if failed > 0 {
        return Err("unported initialization operations".into());
    }
    Ok(())
}
