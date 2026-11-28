//! Animation frame sound selection and root-world placement (zy.ag).
use crate::{animation::Sound, area_sound::Request};
/// The original samples an integer in 1..=100 and compares half-open cumulative weights.
pub fn select(sounds: &[Sound], draw: u32) -> Option<&Sound> {
    if sounds.len() == 1 {
        return sounds.first();
    }
    let mut weight = 0;
    for sound in sounds {
        let previous = weight;
        weight += sound.location as u32;
        if previous <= draw && draw < weight {
            return Some(sound);
        }
    }
    None
}
pub fn request(sound: &Sound, local: bool, fine: [i32; 2], base: [i32; 2]) -> Option<Request> {
    if sound.range == 0 && !local {
        return None;
    }
    let tile = if sound.range == 0 {
        [0; 2]
    } else {
        fine.map(|v| v.wrapping_sub(64) >> 7)
    };
    let packed = if sound.range == 0 {
        0
    } else {
        ((tile[0] - base[0]) << 16)
            .wrapping_add((tile[1] - base[1]) << 8)
            .wrapping_add(sound.range as i32)
    };
    Some(Request {
        id: sound.id,
        loops: sound.loops,
        delay: 0,
        radius: sound.range,
        inner: sound.retain,
        tile,
        packed,
    })
}
pub fn admitted(request: &Request, effects: i32, area: i32, count: usize) -> bool {
    count < 50
        && if request.radius > 0 {
            area != 0
        } else {
            effects != 0
        }
}

/// GameClient.nc compares the actor index with di for both players and NPCs,
/// independently of which registered world contains the actor.
pub fn local_actor(index: u16, local_player: u16) -> bool {
    index == local_player
}
