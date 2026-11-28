//! Menu-related varp side effects from revision 239 fg.gd.
use crate::{buffer::Reader, Result};
#[derive(Clone, Debug)]
pub struct Settings {
    pub left_click_menu: bool,
    pub chat_effects: i32,
    pub player_attack: i32,
    pub npc_attack: i32,
    pub combat_target: i32,
    pub follower_varps: Vec<u16>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            left_click_menu: false,
            chat_effects: 0,
            player_attack: 3,
            npc_attack: 3,
            combat_target: -1,
            follower_varps: Vec::new(),
        }
    }
}
pub fn definition(bytes: &[u8]) -> Result<u16> {
    let mut r = Reader::new(bytes);
    let mut kind = 0;
    loop {
        match r.u8()? {
            0 => return Ok(kind),
            5 => kind = r.u16()?,
            _ => {}
        }
    }
}
impl Settings {
    pub fn apply(&mut self, id: u16, kind: u16, value: i32) {
        match kind {
            5 => self.left_click_menu = value == 1,
            6 => self.chat_effects = value,
            17 => {
                if !self.follower_varps.contains(&id) {
                    self.follower_varps.push(id);
                }
            }
            18 => self.player_attack = if (0..=4).contains(&value) { value } else { 0 },
            19 => self.combat_target = if value == -1 { -1 } else { value & 2047 },
            22 => self.npc_attack = if (0..=4).contains(&value) { value } else { 0 },
            _ => {}
        }
    }
    pub fn npc_priority(
        &self,
        attack: bool,
        follower_low: bool,
        local_level: i32,
        definition_level: i32,
    ) -> Option<bool> {
        if attack && self.npc_attack == 3 {
            return None;
        }
        Some(
            follower_low
                || (attack && self.npc_attack == 1)
                || (self.npc_attack == 0 && definition_level > local_level),
        )
    }
    /// None suppresses an attack; true moves an action below ordinary actions.
    pub fn player_priority(
        &self,
        attack: bool,
        priority: bool,
        local_level: i32,
        other_level: i32,
        teams: [i32; 2],
        clan: bool,
    ) -> Option<bool> {
        if !attack {
            return Some(priority);
        }
        if self.player_attack == 3 {
            return None;
        }
        let mut low =
            self.player_attack == 1 || (self.player_attack == 0 && other_level > local_level);
        if teams[0] != 0 && teams[1] != 0 {
            low = teams[0] == teams[1];
        } else if self.player_attack == 4 && clan {
            low = true;
        }
        Some(low)
    }
}
pub fn combat_color(other: i32, local: i32) -> u32 {
    match local.wrapping_sub(other) {
        i32::MIN..=-10 => 16711680,
        -9..=-7 => 16723968,
        -6..=-4 => 16740352,
        -3..=-1 => 16756736,
        0 => 16776960,
        1..=3 => 12648192,
        4..=6 => 8453888,
        7..=9 => 4259584,
        _ => 65280,
    }
}

pub fn npc_target(
    name: &str,
    definition_level: i32,
    override_level: i32,
    local_level: i32,
) -> String {
    let mut target = format!("<col=ffff00>{name}");
    if definition_level != 0 && override_level != 0 {
        let level = if override_level == -1 {
            definition_level
        } else {
            override_level
        };
        target.push_str(&format!(
            "<col={:x}>  (level-{level})",
            combat_color(level, local_level)
        ));
    }
    target
}

pub fn opens_on_left(one_button: bool, count: usize, opcode: i32, force_left: bool) -> bool {
    let opcode = if opcode >= 2000 {
        opcode - 2000
    } else {
        opcode
    };
    (one_button && count > 2 || opcode == 1007) && !force_left
}
