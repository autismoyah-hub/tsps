//! Local actor input for be.ae / content328, distinct from widget-owned composition.
use crate::{
    actor_freeze::Freezes, actor_sequence::State, actor_visual::Tint, model::Model,
    player_info::Player, player_model::Appearance,
};

pub use crate::actor_pose::select;

pub fn sequences(
    freezes: &Freezes,
    action: &State,
    pose: &State,
    idle: i32,
) -> (Option<State>, Option<State>) {
    let action = freezes.displayed(action).filter(|s| s.id >= 0).cloned();
    let pose = freezes
        .displayed_pose(pose)
        .filter(|s| s.id >= 0 && (action.is_none() || s.id != idle))
        .cloned();
    (action, pose)
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub appearance: Appearance,
    pub action: Option<State>,
    pub pose: Option<State>,
    pub idle: i32,
    pub attachment: Option<(Model, Vec<[i32; 3]>)>,
    pub tint: Tint,
    pub alpha: u8,
    pub cycle: i32,
}
impl Snapshot {
    pub fn capture(player: &Player, cycle: i32, height: Option<i32>) -> Option<Self> {
        let appearance = player.appearance.as_ref()?;
        let idle = appearance.animations[0];
        let (action, pose) = select(
            &player.freezes,
            &player.sequence,
            &player.pose.state,
            &player.idle_fallback.state,
            idle,
            false,
            |_| true,
        )?;
        let (action, pose) = (action.cloned(), pose.cloned());
        let attachment = player
            .attachment
            .as_ref()
            .filter(|a| a.placement.active(cycle))
            .zip(height)
            .map(|(a, height)| {
                (
                    a.relative_model(
                        [player.path.position[0], height, player.path.position[1]],
                        player.turn.target,
                    ),
                    a.colors.clone(),
                )
            });
        Some(Self {
            appearance: appearance.appearance.clone(),
            action,
            pose,
            idle,
            attachment,
            tint: player.visual.tint,
            alpha: player.visual.alpha(cycle),
            cycle,
        })
    }
}

/// qu.ck: only ordinary idle players use the crowded-scene shortcut.
pub fn crowded_pose(
    player_count: usize,
    low_detail: bool,
    ordinary: bool,
    pose: i32,
    idle: i32,
    active_attachment: bool,
) -> bool {
    ordinary
        && !active_attachment
        && pose == idle
        && (player_count > 200 || low_detail && player_count > 50)
}
