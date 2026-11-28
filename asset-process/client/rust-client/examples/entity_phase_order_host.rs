use gielinor_client::{
    actor_sequence, animation::Sequence, animation_readiness::Readiness, client_state::ClientState,
    disk_cache::DiskCache, world_entity, Result,
};
use std::collections::HashSet;
#[path = "../tests/support/entity_phase_order.rs"]
mod fixture;
fn action(frame: u16, sequence: &Sequence) -> actor_sequence::State {
    actor_sequence::State {
        id: sequence.id as i32,
        frame: frame as i32 - 1,
        elapsed: sequence.lengths[frame as usize - 1] as u32,
        ..Default::default()
    }
}
fn choice(index: u16) -> usize {
    [7, 23, 9, 25, 11, 27]
        .iter()
        .position(|&id| id == index)
        .unwrap()
}
fn main() -> Result<()> {
    let mut cache = DiskCache::open_remote(
        "work/kotlin-world/.data/cache/js5",
        64 << 20,
        "127.0.0.1:43617".parse().unwrap(),
    )?;
    cache.wait_for_recovery(true);
    let sequences = Sequence::load_all(&mut cache)?;
    let mut client = ClientState::load(cache)?;
    let mut candidates = sequences
        .values()
        .flat_map(|s| {
            s.sounds.iter().filter_map(move |(&f, sounds)| {
                (f > 0
                    && s.cached_animation.is_none()
                    && f as usize + 1 < s.frames.len()
                    && sounds.len() == 1
                    && sounds[0].range > 0
                    && sounds[0].loops > 0)
                    .then(|| (s.id, f, sounds[0].id))
            })
        })
        .collect::<Vec<_>>();
    candidates.sort();
    let mut ids = HashSet::new();
    let choices = candidates
        .into_iter()
        .filter(|v| ids.insert(v.2))
        .take(6)
        .collect::<Vec<_>>();
    assert_eq!(choices.len(), 6);
    let mut readiness = Readiness::default();
    for &(id, _, _) in &choices {
        for _ in 0..128 {
            if readiness.ready(&mut client.scripts.source.cache, &sequences[&id]) {
                break;
            }
            client.scripts.source.cache.wait_for_recovery(true);
        }
        assert!(readiness.ready(&mut client.scripts.source.cache, &sequences[&id]));
    }
    for (number, case) in fixture::cases()?.into_iter().enumerate() {
        client.audio = Default::default();
        client.player_state = None;
        client.npc_state = Default::default();
        client.world_entities.clear();
        client.detached_world_entities.clear();
        client.view_registry = Default::default();
        client.world_base = [0; 2];
        client.actor_cycle = 10;
        for view in case.views() {
            client.view_registry.insert(view);
        }
        for parent in [0, 7, 23] {
            let mut state = world_entity::State::default();
            state.visible = case.visible(parent).to_vec();
            for &id in &state.visible {
                let mut entity = world_entity::Entity::default();
                entity.dimensions = [24; 2];
                entity.pose.x = 1600 + 16 * id as i32;
                entity.pose.z = 1600;
                let (sequence, frame, _) = choices[choice(id)];
                entity.default_animation = action(frame, &sequences[&sequence]);
                state.entities.insert(id, entity);
            }
            client.world_entities.insert(parent, state);
        }
        client.tick_actor_sequences()?;
        let observed = client.audio.pending_effects().collect::<Vec<_>>();
        let expected = case
            .expected
            .iter()
            .map(|&(parent, id)| (choices[choice(id)].2 as u32, Some(parent)))
            .collect::<Vec<_>>();
        assert_eq!(observed, expected, "original entity phase case{number}");
    }
    println!("16 original logged-in world-entity phase traces match cache-loaded ClientState sound queue order; six distinct cached sounds {choices:?}");
    Ok(())
}
