//! Named player script action (cf.fn, opcode 3107).
use crate::{player_info::State, protocol::Packet, username, world_action, Result};
#[derive(Debug)]
pub struct Outcome {
    pub packet: Option<Packet>,
    pub message: Option<String>,
}
pub fn resolve(state: Option<&State>, operation: i32, name: &str) -> Result<Outcome> {
    let missing = || Outcome {
        packet: None,
        message: Some(format!("Unable to find {name}, current world null.")),
    };
    let Some(state) = state else {
        return Ok(missing());
    };
    if state.local_view != 0 && !state.views.contains_key(&state.local_view) {
        return Ok(missing());
    }
    let target = username::normalize(name, 12);
    for &index in &state.high {
        if index == state.local {
            continue;
        }
        let Some(actor) = state.view_actor(state.local_view, index) else {
            continue;
        };
        let Some(appearance) = &actor.appearance else {
            continue;
        };
        if username::normalize(&appearance.name, 12) == target {
            let packet = if matches!(operation, 1 | 4 | 6 | 7) {
                Some(world_action::player(operation as u8, index, false)?)
            } else {
                None
            };
            return Ok(Outcome {
                packet,
                message: None,
            });
        }
    }
    Ok(Outcome {
        packet: None,
        message: Some(format!("Unable to find {name}")),
    })
}
