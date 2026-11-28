//! GameActor.as chooses a height offset independently of body-model loading grace.
use crate::{actor_freeze::Freezes, actor_sequence::State};
pub fn selected<'a>(
    freezes: &'a Freezes,
    action: &'a State,
    pose: &'a State,
    fallback: &'a State,
    idle: i32,
    mut ready: impl FnMut(i32) -> bool,
) -> Option<&'a State> {
    let action = freezes
        .displayed(action)
        .filter(|s| s.id >= 0 && ready(s.id));
    let pose = freezes
        .displayed_pose(pose)
        .filter(|s| s.id >= 0 && ready(s.id) && (action.is_none() || s.id != idle));
    pose.or(action)
        .or_else(|| (fallback.id >= 0).then_some(fallback))
}
pub fn offset(
    sequences: &std::collections::HashMap<u32, crate::animation::Sequence>,
    freezes: &Freezes,
    action: &State,
    pose: &State,
    fallback: &State,
    idle: i32,
) -> i32 {
    selected(freezes, action, pose, fallback, idle, |id| {
        sequences
            .get(&(id as u32))
            .is_some_and(|s| s.cached_animation.is_some() || !s.frames.is_empty())
    })
    .and_then(|s| sequences.get(&(s.id as u32)))
    .map_or(0, |s| s.extra as i32)
}
pub fn player(
    sequences: &std::collections::HashMap<u32, crate::animation::Sequence>,
    p: &crate::player_info::Player,
) -> i32 {
    offset(
        sequences,
        &p.freezes,
        &p.sequence,
        &p.pose.state,
        &p.idle_fallback.state,
        p.appearance.as_ref().map_or(-1, |a| a.animations[0]),
    )
}
pub fn npc(
    sequences: &std::collections::HashMap<u32, crate::animation::Sequence>,
    n: &crate::npc_info::Npc,
) -> i32 {
    offset(
        sequences,
        &n.freezes,
        &n.sequence,
        &n.pose.state,
        &n.idle_fallback.state,
        n.animations[0],
    )
}

fn available_offset(sequences:&std::collections::HashMap<u32,crate::animation::Sequence>, freezes:&Freezes,action:&State,pose:&State,fallback:&State,idle:i32,assets:&crate::animation_readiness::Readiness)->i32 {
    selected(freezes,action,pose,fallback,idle,|id| sequences.get(&(id as u32)).is_some_and(|s|
        assets.known(s).unwrap_or(s.cached_animation.is_some()||!s.frames.is_empty())))
        .and_then(|s|sequences.get(&(s.id as u32))).map_or(0,|s|s.extra as i32)
}
pub fn player_ready(sequences:&std::collections::HashMap<u32,crate::animation::Sequence>,p:&crate::player_info::Player,assets:&crate::animation_readiness::Readiness)->i32 {
    available_offset(sequences,&p.freezes,&p.sequence,&p.pose.state,&p.idle_fallback.state,p.appearance.as_ref().map_or(-1,|a|a.animations[0]),assets)
}
pub fn npc_ready(sequences:&std::collections::HashMap<u32,crate::animation::Sequence>,n:&crate::npc_info::Npc,assets:&crate::animation_readiness::Readiness)->i32 {
    available_offset(sequences,&n.freezes,&n.sequence,&n.pose.state,&n.idle_fallback.state,n.animations[0],assets)
}
