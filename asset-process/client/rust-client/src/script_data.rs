//! Inventory, enum and font-backed script queries.
use crate::{
    disk_cache::DiskCache,
    enum_def,
    font::Metrics,
    interface_script::{ints, string},
    inventory::Inventories,
    vm::{Object, Stacks, Value},
    Error, Result,
};
use std::collections::HashMap;
#[derive(Default)]
pub struct Data {
    pub worlds: crate::world_list::List,
    pub mobile: bool,
    pub client_type: Option<i32>,
    pub platform_type: i32,
    pub mouse_reported: Option<[i32; 2]>,
    enums: HashMap<i32, enum_def::Definition>,
    capacities: HashMap<i32, i32>,
    fonts: HashMap<i32, Metrics>,
    /// Selected world membership (GameClient.cj), independent of account status.
    pub members: bool,
    /// Login account subscription (GameClient.ef == 1), used for social-list limits.
    pub account_members: bool,
    pub weight: i32,
    pub energy: i32,
    pub world: i32,
    pub world_flags: i32,
    pub staff_level: i32,
    pub reboot_timer: i32,
    pub reboot_message: Option<String>,
    pub server_string: String,
    pub moderator: bool,
    pub position: [i32; 3],
    pub boosted: [i32; 25],
    pub invisible: [i32; 25],
    pub base: [i32; 25],
    pub experience: [i32; 25],
}
fn push(s: &mut Stacks, v: Value) -> Result<()> {
    match v {
        Value::Int(v) => s.push_int(v),
        Value::Long(v) => s.push_long(v),
        Value::Object(v) => s.push_object(v),
    }
}
impl Data {
    pub fn reboot(&mut self, message: String, cycles: u16) {
        self.reboot_timer = cycles as i32 * 30;
        if message == "\u{18}" { self.reboot_message = None; }
        else if !message.is_empty() { self.reboot_message = Some(message); }
    }
    pub fn status_query(&self, op: u16, s: &mut Stacks) -> Result<bool> {
        match op {
            6518 => s.push_int(i32::from(self.mobile))?,
            6519 => s.push_int(self.client_type.unwrap_or(-1))?,
            6520 | 6521 => {},
            6522 | 6523 => {s.pop_object()?;s.pop_int()?;},
            6524 => s.push_int(-1)?,
            6525 | 6526 => s.push_int(1)?,
            6527 => s.push_int(self.platform_type)?,
            6531 => {s.push_int(239)?;s.push_int(1)?;},
            3326 | 3327 => s.push_int(self.mouse_reported.unwrap_or([-1, -1])[(op - 3326) as usize])?,
            3312 => s.push_int(self.members as i32)?,
            3317 => s.push_int(self.reboot_timer)?,
            3333 => s.push_object(Object::String(self.server_string.clone()))?,
            3334 => s.push_object(Object::String(self.reboot_message.clone().unwrap_or_default()))?,
            _ => return Ok(false),
        }
        Ok(true)
    }
    pub fn stat(&mut self, index: u8, boosted: u8, invisible: u8, experience: i32) -> Result<()> {
        let i = index as usize;
        if i >= 25 {
            return Err(Error::Invalid("stat index"));
        }
        self.boosted[i] = boosted as i32;
        self.invisible[i] = invisible as i32;
        self.experience[i] = experience;
        self.base[i] = 1;
        let mut points = 0;
        for level in 1..=98 {
            points += (level as f64 + 300.0 * libm::pow(2.0, level as f64 / 7.0)) as i32;
            if experience >= points / 4 {
                self.base[i] = level + 1;
            }
        }
        Ok(())
    }
    fn enumeration(&mut self, c: &mut DiskCache, id: i32) -> Result<&enum_def::Definition> {
        if !self.enums.contains_key(&id) {
            let d = enum_def::Definition::decode(&c.file(2, 8, id as u32, [0; 4])?)?;
            self.enums.insert(id, d);
        }
        Ok(&self.enums[&id])
    }
    pub fn execute(
        &mut self,
        op: u16,
        s: &mut Stacks,
        c: &mut DiskCache,
        inv: &Inventories,
        cycle: i32,
    ) -> Result<bool> {
        if self.worlds.execute(op, s)? || self.status_query(op, s)? { return Ok(true); }
        match op {
            8020 | 8021 => {
                let [kind, id] = ints(s)?;
                let array = self.enumeration(c, id)?.array(op == 8021, kind)?;
                s.push_object(Object::Array(array))?;
            }
            3300 => s.push_int(cycle)?,
            3301..=3303 | 3313..=3315 => {
                let [id, arg] = ints(s)?;
                let container = id.wrapping_add(if op >= 3313 { 32768 } else { 0 }) as u32;
                let kind = if op >= 3313 { op - 12 } else { op };
                s.push_int(match kind {
                    3301 => inv.item(container, arg).id,
                    3302 => inv.item(container, arg).count,
                    _ => inv.total(container, arg),
                })?;
            }
            3304 => {
                let id = s.pop_int()?;
                if !self.capacities.contains_key(&id) {
                    let size = crate::inventory_def::Definition::load(c, id as u32)?.capacity;
                    self.capacities.insert(id, size);
                }
                s.push_int(self.capacities[&id])?;
            }
            3305..=3307 => {
                let skill = s.pop_int()?;
                let values = match op {
                    3305 => &self.boosted,
                    3306 => &self.base,
                    _ => &self.experience,
                };
                s.push_int(
                    *values
                        .get(skill as usize)
                        .ok_or(Error::Invalid("skill index"))?,
                )?;
            }
            3308 => {
                s.push_int((self.position[0] << 28) + (self.position[1] << 14) + self.position[2])?
            }
            3309 => {
                let p = s.pop_int()?;
                s.push_int((p >> 14) & 16383)?;
            }
            3310 => {
                let p = s.pop_int()?;
                s.push_int(p >> 28)?;
            }
            3311 => {
                let p = s.pop_int()?;
                s.push_int(p & 16383)?;
            }
            3316 => s.push_int(if self.staff_level >= 2 {
                self.staff_level
            } else {
                0
            })?,
            3318 => s.push_int(self.world)?,
            3321 => s.push_int(self.energy / 100)?,
            3331 => s.push_int(self.energy)?,
            3332 => {
                let index = s.pop_int()?;
                let value = *self
                    .invisible
                    .get(index as usize)
                    .ok_or(Error::Invalid("skill index"))?;
                s.push_int(value)?;
            }
            3322 => s.push_int(self.weight)?,
            3323 => s.push_int(self.moderator as i32)?,
            3324 => s.push_int(self.world_flags)?,
            3325 => {
                let [base, x, plane, y] = ints(s)?;
                s.push_int(
                    base.wrapping_add(x << 14)
                        .wrapping_add(plane << 28)
                        .wrapping_add(y),
                )?;
            }
            3400 => {
                let [id, key] = ints(s)?;
                let d = self.enumeration(c, id)?;
                push(s, d.get(key))?;
            }
            3408 => {
                let [input, output, id, key] = ints(s)?;
                let d = self.enumeration(c, id)?;
                let v = if d.input as i32 == input && d.output as i32 == output {
                    d.get(key)
                } else {
                    match output {
                        115 => Value::Object(Object::String("null".into())),
                        207 => Value::Long(0),
                        _ => Value::Int(0),
                    }
                };
                push(s, v)?;
            }
            3411 => {
                let id = s.pop_int()?;
                s.push_int(self.enumeration(c, id)?.entries.len() as i32)?;
            }
            4108 | 4109 => {
                let text = string(s)?;
                let [width, font] = ints(s)?;
                if !self.fonts.contains_key(&font) {
                    self.fonts
                        .insert(font, Metrics::decode(&c.file(13, font as u32, 0, [0; 4])?)?);
                }
                let f = &self.fonts[&font];
                let lines = f.wrap(&text, Some(&[width]), &[])?;
                s.push_int(if op == 4108 {
                    lines.len() as i32
                } else {
                    lines
                        .iter()
                        .map(|line| f.width(line, &[]))
                        .max()
                        .unwrap_or(0)
                })?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
