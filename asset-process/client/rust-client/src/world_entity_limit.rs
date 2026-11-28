//! Per-category child-world scene draw limit (th.cz opcodes 7900/7901).
use crate::{vm::Stacks, Result};
#[derive(Clone, Copy, Debug)]
pub struct Limit(i32);
impl Default for Limit {
    fn default() -> Self {
        Self(30)
    }
}
impl Limit {
    pub fn get(self) -> usize {
        self.0 as usize
    }
    pub fn execute(&mut self, opcode: u16, stacks: &mut Stacks) -> Result<bool> {
        match opcode {
            7900 => self.0 = stacks.pop_int()?.max(0),
            7901 => stacks.push_int(self.0)?,
            _ => return Ok(false),
        }
        Ok(true)
    }
}
