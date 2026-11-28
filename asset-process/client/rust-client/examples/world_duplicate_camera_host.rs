#[path = "../tests/support/duplicate_camera.rs"]
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
        let (base, npcs) = fixture::setup(case, &mut client.world_entities);
        client.world_base = base;
        client.data.position[0] = 0;
        client.npc_state = npcs;
        client.camera.follow_target = Some((1, 41));
        fixture::check(case, client.camera_focus_local(), &client.world_entities);
    }
    println!("{} original duplicate NPC camera selection, projection and plane cases passed cache-backed packet application",cases.len());
    Ok(())
}
