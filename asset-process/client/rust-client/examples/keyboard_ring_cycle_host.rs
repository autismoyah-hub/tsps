use gielinor_client::{
    client_state::ClientState, disk_cache::DiskCache, script_runtime::CacheSource, Result,
};
#[path = "../tests/support/keyboard_ring_cycle.rs"]
mod support;
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let mut client = ClientState::load(cache)?;
    client.scripts.source = CacheSource::new(DiskCache::open(
        "tests/fixtures/keyboard-ring-cycle-vm-cache",
        1 << 20,
    )?);
    client.canvas = [200, 200];
    let cases = support::cases()?;
    let mut batches = 0;
    for batch_mode in [false, true] {
        client.interfaces = support::install()?;
        let mut pending = support::input();
        let mut cycle = 0;
        while cycle < cases.len() {
            support::apply(&cases[cycle], &mut client.input, &mut pending)?;
            client.scripts.variables.set_player(0, 0);
            client.scripts.variables.set_player(1, 0);
            assert_eq!(client.tick_interfaces_elapsed(0, &mut pending, false)?, 0);
            assert_eq!(
                client.scripts.variables.player(0),
                0,
                "zero tick dispatched keys"
            );
            let count =
                if batch_mode && cycle + 1 < cases.len() && cases[cycle + 1].updates.is_empty() {
                    batches += 1;
                    2
                } else {
                    1
                };
            client.cycle = (cycle + count) as i32;
            client.tick_interfaces_elapsed(count as u32, &mut pending, false)?;
            assert!(
                client.script_failures.is_empty(),
                "cycle{cycle}: {:?}",
                client.script_failures
            );
            let expected = cases[cycle..cycle + count]
                .iter()
                .flat_map(|c| c.callbacks.iter())
                .collect::<Vec<_>>();
            assert_eq!(
                client.scripts.variables.player(0),
                expected.len() as i32,
                "mode{batch_mode} cycle{cycle} count"
            );
            let hash = expected.iter().fold(0i32, |h, &&(op, key, ch)| {
                [op, key, ch]
                    .into_iter()
                    .fold(h, |a, v| a.wrapping_mul(31).wrapping_add(v))
            });
            assert_eq!(
                client.scripts.variables.player(1),
                hash,
                "mode{batch_mode} cycle{cycle} key/character order"
            );
            cycle += count;
        }
    }
    assert!(batches > 0);
    println!("1024 original keyboard ring cycles match actual-cache elapsed widget ticks, including zero-tick retention and {batches} batched pairs");
    Ok(())
}
