//! NPC query opcodes from vz.cg.
use crate::{
    interface_script::ints,
    npc::Definition,
    vm::{Object, Stacks},
    Error, Result,
};
use std::collections::HashMap;
pub fn execute(op: u16, s: &mut Stacks, defs: &HashMap<u32, Definition>) -> Result<bool> {
    if !matches!(op, 6754 | 6764 | 6765) {
        return Ok(false);
    }
    let (id, index) = if op == 6764 {
        let [id, index] = ints(s)?;
        (id, index)
    } else {
        (s.pop_int()?, 0)
    };
    let default = Definition::decode(id as u32, &[0])?;
    let d = defs.get(&(id as u32)).unwrap_or(&default);
    match op {
        6754 => s.push_object(Object::String(d.name.clone()))?,
        6765 => s.push_int(d.combat)?,
        _ => {
            if index < 0 && !d.head_icons.is_empty() {
                return Err(Error::Invalid("NPC head icon index"));
            }
            let (group, file) = d
                .head_icons
                .get(index as usize)
                .copied()
                .unwrap_or((-1, -1));
            s.push_int(group)?;
            s.push_int(file as i32)?;
        }
    }
    Ok(true)
}
