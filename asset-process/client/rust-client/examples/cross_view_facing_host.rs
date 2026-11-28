use gielinor_client::{client_state::ClientState, disk_cache::DiskCache, Result};
#[path = "../tests/support/cross_view_facing.rs"]
mod fixture;
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    let (packet, cases) = fixture::cases()?;
    for base in [[0, 0], [3200, 6400]] {
        for (number, case) in cases.iter().enumerate() {
            let setup = fixture::setup(case, &packet, &client.npcs, base)?;
            client.player_state = Some(setup.players);
            client.npc_state = setup.npcs;
            client.world_entities = setup.worlds;
            client.view_registry = setup.registry;
            client.detached_world_entities.clear();
            client.world_base = base;
            client.actor_cycle = 10;
            client.tick_actor_sequences()?;
            let turn = if case.observer_kind == 2 {
                client
                    .player_state
                    .as_ref()
                    .unwrap()
                    .view_actor(case.observer, 1)
                    .unwrap()
                    .turn
            } else if case.observer == 0 {
                client.npc_state.npcs[&1].turn
            } else {
                client.world_entities[&0].entities[&case.observer].npcs.npcs[&1].turn
            };
            assert_eq!(
                [turn.target as i32, turn.angle as i32],
                case.expected,
                "cross-view original tick case{number}"
            );
        }
    }
    println!("576 original logged-in actor/world-entity facing traces match 1152 cache-loaded ClientState cases at zero and nonzero bases");
    Ok(())
}
