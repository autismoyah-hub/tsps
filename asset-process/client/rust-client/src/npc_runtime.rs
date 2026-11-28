//! Shared root/child-world NPC simulation.
use crate::{animation::Sequence, npc::Definition, npc_info::State, Error, Result};
use std::collections::{HashMap, VecDeque};
pub fn tick(
    state: &mut State,
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
    tick_with_readiness(state,definitions,sequences,spots,targets,cycle,base,dimensions,root_local_index,events,&mut |_|true)
}
pub fn tick_with_readiness(
    state: &mut State,
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
    tick_in_view_with_readiness(0, state, definitions, sequences, spots, targets,
        cycle, base, dimensions, root_local_index, events, ready)
}
/// Registered NPC iteration has a distinct bucket count in child world views.
pub fn tick_in_view_with_readiness(
    view: u16,
    state: &mut State,
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
    tick_in_view_with_targets(view,state,definitions,sequences,spots,crate::actor_target::Targets::local(targets),cycle,base,dimensions,root_local_index,events,ready)
}
pub fn tick_in_view_with_targets(
    view: u16,
    state: &mut State,
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
    // kh.bu reads live actor coordinates; prior NPCs may already have moved.
    for index in state.iteration_order(view) {
        let npc = state.npcs.get_mut(&index).unwrap();
        let def = &definitions[&(npc.id as u32)];
        let blocked = sequences.get(&(npc.sequence.id as u32)).is_some_and(|s| {
            npc.sequence.delay == 0
                && if npc.path.path_at_start > 0 {
                    s.precedence == 0
                } else {
                    s.priority_moving == 0
                }
        });
        let exact_start = npc.exact.as_ref().map_or(-1, |e| e.start);
        crate::actor_turn::select_resolved(
            &mut npc.turn,
            &mut npc.facing,
            &npc.path,
            def.size as i32,
            blocked,
            |kind, index| targets.angle(kind,index,npc.path.position),
        );
        let forced = npc.exact.as_ref().is_some_and(|exact| {
            exact.tick(
                &mut npc.path,
                def.size as i32,
                cycle,
                npc.sequence.allows_exact_step(sequences),
            )
        });
        if forced {
            npc.turn.target = npc.exact.as_ref().unwrap().direction;
        }
        if forced && cycle > exact_start {
            npc.turn.angle = npc.exact.as_ref().unwrap().direction;
            npc.turn.target = npc.turn.angle;
        }
        let poses = npc.animations;
        if !forced {
            let movement = crate::actor_path::Movement {
                size: def.size as i32,
                clickable: def.rotation,
                turning: npc.turn.angle != npc.turn.target,
                interacting: npc.facing.as_ref().is_some_and(|f| {
                    matches!(
                        f.target,
                        crate::actor_update::FaceTarget::Entity { kind: 1..=3, .. }
                    )
                }),
                turn_speed: (def.turn_speed) as i32,
                blocked,
            };
            let pose = crate::actor_pose::movement(
                poses,
                &npc.path,
                movement,
                npc.turn.angle,
                npc.turn.target,
            );
            npc.pose.set(pose, poses[0], false);
            npc.path.prepare(def.size as i32, blocked);
            npc.path.advance(1.0, movement);
        }
        if npc.path.restore_if_outside(
            def.size as i32,
            base,
            dimensions,
            root_local_index == Some(index),
        ) {
            npc.sequence.reset();
            npc.exact = None;
            npc.spots.clear();
        }
        targets.update(1,index,npc.path.position);
        let (exact_start, exact_end) = npc.exact.as_ref().map_or((0, 0), |e| (e.start, e.end));
        let pose = npc.turn.tick(
            def.turn_speed,
            npc.freezes.active,
            npc.pose.state.id,
            [poses[0], poses[1], poses[2], poses[3]],
        );
        npc.pose.set(pose, poses[0], def.restart_idle);
        npc.angle = npc.turn.angle;
        let suspended = npc.sequence.suspend_for_path(
            sequences,
            npc.path.path_at_start,
            exact_start,
            exact_end,
            cycle,
        )?;
        let frozen = npc
            .freezes
            .tick_pose(cycle, &mut npc.sequence, &npc.pose.state);
        let pose_id = npc.pose.state.id;
        for frame in npc.pose.tick(sequences, frozen, ready(pose_id))? {
            if sequences[&(pose_id as u32)]
                .sounds
                .contains_key(&(frame as u16))
            {
                events.push_back((false, index, pose_id as u32, frame));
            }
        }
        npc.idle_fallback
            .tick_fallback(poses[0], &npc.pose.state, sequences, frozen, ready(poses[0]))?;
        for (id, frame) in npc.spots.tick(cycle, &spots, sequences)? {
            if sequences[&id].sounds.contains_key(&(frame as u16)) {
                events.push_back((false, index, id, frame));
            }
        }
        if suspended {
            continue;
        }
        let id = npc.sequence.id;
        for frame in npc.sequence.tick(
            sequences,
            npc.path.path_at_start,
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
                events.push_back((false, index, id as u32, frame));
            }
        }
        if id >= 0 && npc.sequence.id < 0 && npc.pose.state.id == poses[0] && def.restart_idle {
            npc.pose.state.frame = 0;
            npc.pose.state.elapsed = 0;
            npc.pose.state.loops = 0;
        }
    }
    Ok(())
}
