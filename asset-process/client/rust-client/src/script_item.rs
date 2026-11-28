//! Item definition query opcodes from the client script dispatcher.
use crate::{
    interface_script::ints,
    item::{Definition, Definitions},
    vm::{Object, Stacks},
    Result,
};
pub fn execute(op: u16, s: &mut Stacks, items: &Definitions) -> Result<bool> {
    if !matches!(op,3931|3939|4200..=4209|4213..=4220|4222..=4224) {
        return Ok(false);
    }
    if op == 4223 {
        return Ok(true);
    }
    if op == 4224 {
        let id = s.pop_int()?;
        s.push_int(
            if id < 0 || items.items.keys().max().is_none_or(|&max| id as u32 > max) {
                -1
            } else {
                id
            },
        )?;
        return Ok(true);
    }
    let (id, index, sub) = if op == 4222 {
        let [id, index, sub] = ints(s)?;
        (id, index, sub)
    } else if matches!(op, 4201 | 4202) {
        let [id, index] = ints(s)?;
        (id, index, 0)
    } else {
        (s.pop_int()?, 0, 0)
    };
    let mut missing = Definition::decode(id as u32, &[0])?;
    missing.tradeable = false;
    let d = items.items.get(&(id as u32)).unwrap_or(&missing);
    match op {
        3931 => s.push_int(d.stockmarket as i32)?,
        3939 => s.push_int(d.searchable as i32)?,
        4200 => s.push_object(Object::String(d.name.clone()))?,
        4201 => s.push_object(Object::String(
            d.ground_operations
                .iter()
                .find(|o| o.index as i32 == index - 1 && o.sub.is_none() && o.condition.is_none())
                .map_or(String::new(), |o| o.text.clone()),
        ))?,
        4202 => s.push_object(Object::String(
            d.inventory_operations
                .get(index.wrapping_sub(1) as usize)
                .and_then(|v| v.clone())
                .unwrap_or_default(),
        ))?,
        4203 => s.push_int(d.price)?,
        4204 => s.push_int((d.stackable == 1) as i32)?,
        4205 => s.push_int(if d.note[1] == -1 && d.note[0] >= 0 {
            d.note[0]
        } else {
            id
        })?,
        4206 => s.push_int(if d.note[1] >= 0 && d.note[0] >= 0 {
            d.note[0]
        } else {
            id
        })?,
        4207 => s.push_int(d.members as i32)?,
        4208 => s.push_int(if d.placeholder[1] == -1 && d.placeholder[0] >= 0 {
            d.placeholder[0]
        } else {
            id
        })?,
        4209 => s.push_int(if d.placeholder[1] >= 0 && d.placeholder[0] >= 0 {
            d.placeholder[0]
        } else {
            id
        })?,
        4213 => s.push_int(d.shift_click_index().map_or(-1, |i| i as i32 + 1))?,
        4214..=4216 => s.push_int(d.wear_positions[(op - 4214) as usize])?,
        4217 => s.push_int(d.weight)?,
        4218 => s.push_object(Object::String(d.examine.clone()))?,
        4219 => s.push_int(d.category)?,
        4220 => s.push_int(d.tradeable as i32)?,
        4222 => {
            let i = index.wrapping_sub(1) as usize;
            let text = if d.inventory_operations.get(i).is_some_and(|v| v.is_some()) {
                d.sub_operations
                    .get(i)
                    .and_then(|v| v.as_ref())
                    .and_then(|v| v.get(sub.wrapping_sub(1) as usize))
                    .cloned()
                    .flatten()
                    .unwrap_or_default()
            } else {
                String::new()
            };
            s.push_object(Object::String(text))?;
        }
        _ => unreachable!(),
    }
    Ok(true)
}
