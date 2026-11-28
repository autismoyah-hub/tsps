use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, interface_packet::Update};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a = std::env::args().collect::<Vec<_>>();
    let mut client = ClientState::load(DiskCache::open(&a[1], 64 << 20)?)?;
    client.interface(Update::OpenTop(a[2].parse()?))?;
    while let Some(event) = client.next_interface_event()? {
        client.run_event(event)?;
    }
    for pair in a[3..].chunks_exact(2) {
        client.input.pointer = [pair[0].parse()?, pair[1].parse()?];
        client.input.button(0, true);
        let n = client.tick_interfaces(false)?;
        if let Some(error) = client.script_failures.pop_front() {
            return Err(error.into());
        }
        let activated = client.primary_interface_action(client.input.pointer)?;
        client.input.begin_cycle();
        client.input.button(0, false);
        client.tick_interfaces(false)?;
        if let Some(error) = client.script_failures.pop_front() {
            return Err(error.into());
        }
        client.input.begin_cycle();
        client
            .audio
            .tick(&mut client.scripts.source.cache, &client.preferences.device)?;
        let pcm = client.audio.mix(441);
        println!(
            "Audio: {} pending, active={}, {} nonzero samples",
            client.audio.pending_count(),
            client.audio.active(),
            pcm.iter().filter(|&&v| v != 0).count()
        );
        println!(
            "Pointer {:?}: {n} instructions, action={activated}, {} queued packets",
            client.input.pointer,
            client.outbound.len()
        );
    }
    Ok(())
}
