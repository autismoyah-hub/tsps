//! World-map script-event context (hb and bf.bl 6697..6699).
use crate::{vm::Stacks, Result};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub element: i32,
    pub source: [i32; 3],
    pub destination: [i32; 3],
}
fn packed([plane, x, y]: [i32; 3]) -> i32 {
    if plane == -1 {
        -1
    } else {
        plane.wrapping_shl(28) | x.wrapping_shl(14) | y
    }
}
impl Event {
    pub fn query(self, op: u16, stack: &mut Stacks) -> Result<bool> {
        let value = match op {
            6697 => self.element,
            6698 => packed(self.source),
            6699 => packed(self.destination),
            _ => return Ok(false),
        };
        stack.push_int(value)?;
        Ok(true)
    }
}
