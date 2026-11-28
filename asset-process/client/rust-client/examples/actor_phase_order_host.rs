use gielinor_client::{
    actor_sequence, animation::Sequence, animation_readiness::Readiness, client_state::ClientState,
    disk_cache::DiskCache, player_info, world_entity, Result,
};
use std::collections::{BTreeMap, HashSet};
#[path = "../tests/support/actor_phase_order.rs"]
mod fixture;
fn action(frame: u16, sequence: &Sequence) -> actor_sequence::State {
    actor_sequence::State {
        id: sequence.id as i32,
        frame: frame as i32 - 1,
        elapsed: sequence.lengths[frame as usize - 1] as u32,
        ..Default::default()
    }
}
fn choice(player: bool, index: u16) -> usize {
    (if player { 0 } else { 2 }) + usize::from(index == 9)
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
                    .then_some((s.id, f, sounds[0].id))
            })
        })
        .collect::<Vec<_>>();
    candidates.sort();
    let mut ids = HashSet::new();
    let choices = candidates
        .into_iter()
        .filter(|v| ids.insert(v.2))
        .take(4)
        .collect::<Vec<_>>();
    assert_eq!(choices.len(), 4);
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
        client.world_entities.clear();
        client.detached_world_entities.clear();
        client.npc_state = Default::default();
        client.view_registry = Default::default();
        client.world_base = [0; 2];
        client.actor_cycle = 10;
        let mut players = player_info::State::initialize(&[0; 4608], 41)?.0;
        let template = players.players[41].as_ref().unwrap().clone();
        players.high = case.high.clone();
        players.views.clear();
        let mut root = world_entity::State::default();
        for &view in &case.views {
            client.view_registry.insert(view);
            let dimensions = if view == 0 { [104; 2] } else { [24; 2] };
            let mut members = BTreeMap::new();
            let mut npcs = gielinor_client::npc_info::State::default();
            npcs.decode(&case.packet, false, [0; 2], &client.npcs)?;
            for player in [true, false] {
                let indices = if player {
                    case.high.clone()
                } else {
                    npcs.indices.clone()
                };
                for index in indices {
                    if player && !case.present(view, index) {
                        continue;
                    }
                    let (id, frame, _) = choices[choice(player, index)];
                    let sequence = &sequences[&id];
                    if player {
                        let mut p = template.clone();
                        p.position = [0, 12, 12];
                        p.path.tiles[0] = [12; 2];
                        p.path.position = [1600; 2];
                        p.path.fine = [1600.; 2];
                        p.sequence = action(frame, sequence);
                        members.insert(index, p);
                    } else {
                        let npc = npcs.npcs.get_mut(&index).unwrap();
                        npc.animations = [-1; 15];
                        npc.sequence = action(frame, sequence);
                    }
                }
            }
            players.views.insert(
                view,
                player_info::View {
                    base: [0; 2],
                    dimensions,
                    players: members,
                },
            );
            if view == 0 {
                client.npc_state = npcs;
            } else {
                let mut entity = world_entity::Entity::default();
                entity.dimensions = dimensions;
                entity.npcs = npcs;
                root.visible.push(view);
                root.entities.insert(view, entity);
            }
        }
        client.world_entities.insert(0, root);
        client.player_state = Some(players);
        client.tick_actor_sequences()?;
        let observed = client.audio.pending_effects().collect::<Vec<_>>();
        let expected = case
            .expected
            .iter()
            .map(|&(p, v, i)| (choices[choice(p, i)].2 as u32, Some(v)))
            .collect::<Vec<_>>();
        assert_eq!(
            observed, expected,
            "case{number} actual ClientState sound enqueue order"
        );
    }
    println!("32 original logged-in actor phase traces match ClientState actual-cache sound enqueue order; sequence/frame/sound choices {choices:?}; queue inspected without playback");
    Ok(())
}
