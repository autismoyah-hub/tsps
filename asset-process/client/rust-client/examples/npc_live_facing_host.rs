use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, Result};
#[path = "../tests/support/npc_live_facing.rs"]
mod fixture;
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let client = ClientState::load(cache)?;
    fixture::verify(&client.npcs);
    println!("32 original NPC live-target facing traces match actual-cache definition7 simulation; action/pose animation disabled to isolate facing and movement");
    Ok(())
}
