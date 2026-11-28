//! Shared root/child-world player simulation.
use crate::{animation::Sequence, npc::Definition, player_info::Player, Error, Result};
use std::collections::{HashMap, VecDeque};
/// GameClient.iw traverses the global high-player list inside every view.
pub fn ordered<'a, T: 'a>(
    high: &[u16],
    players: impl Iterator<Item = (u16, &'a mut T)>,
) -> Vec<(u16, &'a mut T)> {
    let ranks: HashMap<_, _> = high.iter().enumerate().map(|(rank, &id)| (id, rank)).collect();
    let mut ordered = players.filter_map(|(id, player)| ranks.get(&id).map(|&rank| (rank, id, player))).collect::<Vec<_>>();
    ordered.sort_by_key(|&(rank, _, _)| rank);
    ordered.into_iter().map(|(_, id, player)| (id, player)).collect()
}
pub fn tick<'a>(
    players: impl Iterator<Item = (u16, &'a mut Player)>,
    definitions: &HashMap<u32, Definition>,
    sequences: &HashMap<u32, Sequence>,
    spots: &HashMap<u32, crate::spot_animation::Definition>,
    targets: &HashMap<(u16, i32), [i32; 2]>,
    cycle: i32,
    base: [i32; 2],
    dimensions: [i32; 2],
    root_local_index: Option<u16>,
    events: &mut VecDeque<(bool, u16, u32, usize)>,
) -> Result<()> {
    tick_with_readiness(players,definitions,sequences,spots,targets,cycle,base,dimensions,root_local_index,events,&mut |_|true)
}
pub fn tick_with_readiness<'a>(
    players: impl Iterator<Item = (u16, &'a mut Player)>,
    definitions: &HashMap<u32, Definition>,
    sequences: &HashMap<u32, Sequence>,
    spots: &HashMap<u32, crate::spot_animation::Definition>,
    targets: &HashMap<(u16, i32), [i32; 2]>,
    cycle: i32,
    base: [i32; 2],
    dimensions: [i32; 2],
    root_local_index: Option<u16>,
    events: &mut VecDeque<(bool, u16, u32, usize)>,
    ready: &mut impl FnMut(i32)->bool,
) -> Result<()> {
    tick_with_targets(players,definitions,sequences,spots,crate::actor_target::Targets::local(targets),cycle,base,dimensions,root_local_index,events,ready)
}
pub fn tick_with_targets<'a>(
    players: impl Iterator<Item = (u16, &'a mut Player)>,
    definitions: &HashMap<u32, Definition>,
    sequences: &HashMap<u32, Sequence>,
    spots: &HashMap<u32, crate::spot_animation::Definition>,
    mut targets: crate::actor_target::Targets,
    cycle: i32,
    base: [i32; 2],
    dimensions: [i32; 2],
    root_local_index: Option<u16>,
    events: &mut VecDeque<(bool, u16, u32, usize)>,
    ready: &mut impl FnMut(i32)->bool,
) -> Result<()> {
    // kh.bu sees positions already changed by earlier players in this view.
    for (index, player) in players {
        if player
            .attachment
            .as_ref()
            .is_some_and(|a| cycle >= a.placement.end)
        {
            player.attachment = None;
        }
        let size = player
            .appearance
            .as_ref()
            .and_then(|a| a.appearance.npc)
            .and_then(|id| definitions.get(&(id as u32)))
            .map_or(1, |d| d.size as i32);
        let blocked = sequences
            .get(&(player.sequence.id as u32))
            .is_some_and(|s| {
                player.sequence.delay == 0
                    && if player.path.path_at_start > 0 {
                        s.precedence == 0
                    } else {
                        s.priority_moving == 0
                    }
            });
        let exact_start = player.exact.as_ref().map_or(-1, |e| e.start);
        crate::actor_turn::select_resolved(
            &mut player.turn,
            &mut player.facing,
            &player.path,
            size,
            blocked,
            |kind, index| targets.angle(kind,index,player.path.position),
        );
        let forced = player.exact.as_ref().is_some_and(|exact| {
            exact.tick(
                &mut player.path,
                size,
                cycle,
                player.sequence.allows_exact_step(sequences),
            )
        });
        if forced {
            player.turn.target = player.exact.as_ref().unwrap().direction;
        }
        if forced && cycle > exact_start {
            player.turn.angle = player.exact.as_ref().unwrap().direction;
            player.turn.target = player.turn.angle;
        }
        let poses = player.appearance.as_ref().map_or([-1; 15], |a| {
            let p = a.animations;
            [
                p[0], p[1], p[1], p[2], p[3], p[4], p[5], p[6], -1, -1, -1, -1, -1, -1, -1,
            ]
        });
        if !forced {
            let movement = crate::actor_path::Movement {
                size: size,
                clickable: true,
                turning: player.turn.angle != player.turn.target,
                interacting: player.facing.as_ref().is_some_and(|f| {
                    matches!(
                        f.target,
                        crate::actor_update::FaceTarget::Entity { kind: 1..=3, .. }
                    )
                }),
                turn_speed: (32) as i32,
                blocked,
            };
            let pose = crate::actor_pose::movement(
                poses,
                &player.path,
                movement,
                player.turn.angle,
                player.turn.target,
            );
            player.pose.set(pose, poses[0], false);
            player.path.prepare(size, blocked);
            player.path.advance(1.0, movement);
        }
        if player
            .path
            .restore_if_outside(size, base, dimensions, Some(index) == root_local_index)
        {
            player.sequence.reset();
            player.exact = None;
            player.spots.clear();
        }
        targets.update(2,index,player.path.position);
        let (exact_start, exact_end) = player.exact.as_ref().map_or((0, 0), |e| (e.start, e.end));
        let pose = player.turn.tick(
            32,
            player.freezes.active,
            player.pose.state.id,
            [poses[0], poses[1], poses[2], poses[3]],
        );
        player.pose.set(pose, poses[0], false);
        let suspended = player.sequence.suspend_for_path(
            sequences,
            player.path.path_at_start,
            exact_start,
            exact_end,
            cycle,
        )?;
        let frozen = player
            .freezes
            .tick_pose(cycle, &mut player.sequence, &player.pose.state);
        let pose_id = player.pose.state.id;
        for frame in player.pose.tick(sequences, frozen, ready(pose_id))? {
            if sequences[&(pose_id as u32)]
                .sounds
                .contains_key(&(frame as u16))
            {
                events.push_back((true, index, pose_id as u32, frame));
            }
        }
        player.idle_fallback.tick_fallback(
            poses[0],
            &player.pose.state,
            sequences,
            frozen,
            ready(poses[0]),
        )?;
        for (id, frame) in player.spots.tick(cycle, &spots, sequences)? {
            if sequences[&id].sounds.contains_key(&(frame as u16)) {
                events.push_back((true, index, id, frame));
            }
        }
        if suspended {
            continue;
        }
        let id = player.sequence.id;
        for frame in player.sequence.tick(
            sequences,
            player.path.path_at_start,
            exact_start,
            exact_end,
            cycle,
            frozen,
            ready(id),
        )? {
            if sequences[&(id as u32)].sounds.contains_key(&(frame as u16)) {
                if events.len() >= 16384 {
                    return Err(Error::Invalid("actor sound event queue overflow"));
                }
                events.push_back((true, index, id as u32, frame));
            }
        }
    }
    Ok(())
}
