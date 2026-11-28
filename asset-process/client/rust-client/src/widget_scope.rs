//! Script3135..3140 widget shortcut scope (GameClanChannel.bc / ps).
use crate::{vm::Stacks, Error, Result};
#[derive(Debug)]
pub struct Scope {
    pub mode: i32,
    pub target: i32,
}
impl Default for Scope {
    fn default() -> Self {
        Self {
            mode: 0,
            target: -1,
        }
    }
}
impl Scope {
    pub fn execute(&mut self, op: u16, s: &mut Stacks, active: Option<u32>) -> Result<bool> {
        match op {
            3135 => {
                crate::interface_script::ints::<2>(s)?;
            }
            3136 | 3137 => {
                let target = s.pop_int()?;
                self.mode = if op == 3136 { 3 } else { 2 };
                self.target = target;
            }
            3138 => self.mode = 0,
            3139 => self.mode = 1,
            3140 => {
                let target = active.ok_or(Error::Invalid("shortcut scope active widget"))?;
                self.mode = 3;
                self.target = target as i32;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
    pub fn allows(&self, id: u32) -> bool {
        match self.mode {
            0 => true,
            2 => (id >> 16) as i32 == self.target,
            3 => id as i32 == self.target,
            _ => false,
        }
    }
}
