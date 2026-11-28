use gielinor_client::{
    animation::Sequence, client_state::ClientState, disk_cache::DiskCache, npc_runtime, Result,
};
use std::collections::{HashMap, VecDeque};
#[path = "../tests/support/npc_registry_order.rs"]
mod fixture;
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let sequences = Sequence::load_all(&mut cache)?;
    let client = ClientState::load(cache)?;
    let mut choices = sequences
        .values()
        .flat_map(|s| {
            s.sounds.keys().filter_map(move |&f| {
                (f > 0 && s.cached_animation.is_none() && f as usize + 1 < s.frames.len())
                    .then_some((s.id, f))
            })
        })
        .collect::<Vec<_>>();
    choices.sort();
    let (sequence, frame) = choices[0];
    let mut case = 0;
    fixture::verify(&client.npcs, |state, world, expected| {
        for npc in state.npcs.values_mut() {
            npc.animations = [-1; 15];
            npc.sequence.id = sequence as i32;
            npc.sequence.frame = frame as i32 - 1;
            npc.sequence.elapsed = sequences[&sequence].lengths[frame as usize - 1] as u32;
            npc.sequence.delay = 0;
        }
        let mut events = VecDeque::new();
        npc_runtime::tick_in_view_with_readiness(
            world,
            state,
            &client.npcs,
            &sequences,
            &client.spots,
            &HashMap::new(),
            case,
            [0; 2],
            if world == 0 { [104; 2] } else { [24; 2] },
            None,
            &mut events,
            &mut |_| true,
        )?;
        assert_eq!(
            events.iter().map(|event| event.1).collect::<Vec<_>>(),
            expected,
            "case{case} world{world} NPC frame-sound order"
        );
        case += 1;
        Ok(())
    })?;
    println!("64 original NPC packet/registry orders match actual-cache NPC simulation frame events, sequence{sequence} frame{frame}");
    Ok(())
}
