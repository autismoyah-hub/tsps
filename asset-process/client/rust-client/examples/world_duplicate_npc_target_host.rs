#[path = "../tests/support/duplicate_npc_target.rs"]
mod fixture;
use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, game_packet, protocol::Packet,
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse()?,
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let cases = fixture::cases();
    for case in &cases {
        client.world_entities.clear();
        client.detached_world_entities.clear();
        client.active_world = 0;
        client.view_registry = Default::default();
        for (opcode, payload) in &case.packets {
            client.apply(
                game_packet::decode(&Packet {
                    opcode: *opcode,
                    payload: payload.clone(),
                })?
                .unwrap(),
            )?;
        }
        fixture::verify(case, &mut client.world_entities, &client.view_registry);
    }
    println!("{} original duplicate NPC target traversal and facing cases passed cache-backed packet application",cases.len());
    Ok(())
}
