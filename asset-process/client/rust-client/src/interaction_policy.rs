//! Per-world input and action policies from cc, sg, sz and GameClient.xq.
use crate::{buffer::Reader, context_menu::Action, Error, Result};
use std::collections::HashMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Update {
    Set { world: i16, input: u8, actions: u8 },
    Reset(i16),
}
impl Update {
    pub fn decode(op: u16, r: &mut Reader<'_>) -> Result<Self> {
        let world = r.u16()? as i16;
        Ok(if op == 66 {
            Self::Reset(world)
        } else {
            Self::Set {
                world,
                input: r.u8()?,
                actions: r.u8()?,
            }
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Object,
    Ground,
    Npc,
    Player,
    ExamineObject,
    ExamineGround,
    ExamineNpc,
}
/// GameClient.xq additionally gates nonlocal scene actions using config jl.
pub fn entity_action(collision_type: u8, local: bool, action: Action) -> bool {
    if local { return true; }
    let (_, action) = action.scene();
    let examine = matches!(action, Action::ExamineNpc(_) | Action::Examine { .. }
        | Action::WorldEntity { operation: 6, .. });
    let ordinary = matches!(action, Action::Npc { .. } | Action::Object { .. }
        | Action::Ground { .. } | Action::CastWorld { kind: 0 | 1 | 3, .. }
        | Action::WorldEntity { operation: 1..=5, .. });
    (!examine || collision_type != 0) && (!ordinary || collision_type == 2)
}
#[derive(Clone, Debug)]
pub struct Policies {
    default: [u8; 2],
    overrides: HashMap<i16, [u8; 2]>,
}
impl Default for Policies {
    fn default() -> Self {
        Self {
            default: [1, 1],
            overrides: HashMap::new(),
        }
    }
}
impl Policies {
    pub fn apply(&mut self, update: Update) {
        match update {
            Update::Set {
                world: -2,
                input,
                actions,
            } => self.default = [input, actions],
            Update::Set {
                world,
                input,
                actions,
            } => {
                self.overrides.insert(world, [input, actions]);
            }
            Update::Reset(world) => {
                self.overrides.remove(&world);
            }
        }
    }
    pub fn remove(&mut self, world: i16) {
        self.overrides.remove(&world);
    }
    pub fn get(&self, world: i16) -> [u8; 2] {
        let v = self.overrides.get(&world).copied().unwrap_or(self.default);
        // An unknown wire enum becomes null in Java and inherits the global policy.
        [
            if v[0] < 3 { v[0] } else { self.default[0] },
            if v[1] < 4 { v[1] } else { self.default[1] },
        ]
    }
    pub fn allows(
        &self,
        world: i16,
        category: Category,
        moving: bool,
        local: bool,
    ) -> Result<bool> {
        allows(self.get(world)[1], category, moving, world == 0, local)
    }
    pub fn menu_opcode(&self, world: i16, opcode: i32, moving: bool, local: bool) -> Result<bool> {
        let op = if opcode >= 2000 {
            opcode - 2000
        } else {
            opcode
        };
        let category = match op {
            1..=6 | 1001 => Category::Object,
            16..=22 => Category::Ground,
            7..=13 => Category::Npc,
            14 | 15 | 44..=51 => Category::Player,
            1002 => Category::ExamineObject,
            1004 => Category::ExamineGround,
            1003 => Category::ExamineNpc,
            _ => return Ok(true),
        };
        self.allows(world, category, moving, local)
    }
    pub fn root_action(&self, action: Action) -> Result<bool> {
        self.action(0, action, false, true)
    }
    /// GameClient.xq evaluates the entry's scene and the selected local view.
    pub fn action(&self, world: i16, action: Action, moving: bool, local: bool) -> Result<bool> {
        let category = match action {
            Action::InView { view, target } => return self.action(view as i16, target.action(), moving, local),
            Action::Object { .. } => Category::Object,
            Action::Ground { .. } => Category::Ground,
            Action::Npc { .. } => Category::Npc,
            Action::Player { .. } => Category::Player,
            Action::ExamineNpc(_) => Category::ExamineNpc,
            Action::Examine { kind: 0, .. } => Category::ExamineNpc,
            Action::Examine { kind: 1, .. } => Category::ExamineObject,
            Action::Examine { kind: 2, .. } => Category::ExamineGround,
            Action::CastWorld { kind, .. } => match kind {
                0 => Category::Object,
                1 => Category::Npc,
                2 => Category::Player,
                3 => Category::Ground,
                _ => return Err(Error::Invalid("spell target kind")),
            },
            Action::Walk(_) => return Ok(self.get(world)[0] == 1),
            Action::Heading(_) => return Ok(self.get(world)[0] == 2),
            _ => return Ok(true),
        };
        self.allows(world, category, moving, local)
    }
}
pub fn allows(
    profile: u8,
    category: Category,
    moving: bool,
    root: bool,
    local: bool,
) -> Result<bool> {
    let ordinary = match profile {
        0 => 0,
        1 => 507,
        2 => 338,
        3 => 499,
        _ => return Err(Error::Invalid("null interaction policy")),
    };
    let mask = if category == Category::Player && profile != 0 {
        507
    } else {
        ordinary
    };
    let examine = matches!(
        category,
        Category::ExamineObject | Category::ExamineGround | Category::ExamineNpc
    );
    let bits = if examine {
        (if root { 2 } else { 0 }) | (if moving { 16 } else { 64 }) | (if local { 256 } else { 0 })
    } else {
        (if root { 1 } else { 0 }) | (if moving { 8 } else { 32 }) | (if local { 128 } else { 0 })
    };
    Ok(mask & bits != 0)
}
