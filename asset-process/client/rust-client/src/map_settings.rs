//! World-map flashing/filter controls and element metadata script queries.
use crate::{map_element::Definition, vm::Stacks, Result};
use std::collections::{BTreeSet, HashMap};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub max_flashes: i32,
    pub period: i32,
    pub perpetual: bool,
    pub hidden: bool,
    pub flash_count: i32,
    pub flash_cycle: i32,
    pub flashing: Option<BTreeSet<i32>>,
    pub elements: BTreeSet<i32>,
    pub categories: BTreeSet<i32>,
    pub category_elements: BTreeSet<i32>,
    pub excluded: BTreeSet<i32>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            max_flashes: 3,
            period: 50,
            perpetual: false,
            hidden: false,
            flash_count: -1,
            flash_cycle: -1,
            flashing: None,
            elements: Default::default(),
            categories: Default::default(),
            category_elements: Default::default(),
            excluded: Default::default(),
        }
    }
}
impl Settings {
    /// Java advances flashes per rendered detailed-map frame, even if icons are hidden.
    pub fn frame(&mut self) {
        if self.flashing.is_some() {
            self.flash_cycle = self.flash_cycle.wrapping_add(1);
            if self.flash_cycle % self.period == 0 {
                self.flash_cycle = 0;
                self.flash_count = self.flash_count.wrapping_add(1);
            }
            if self.flash_count >= self.max_flashes && !self.perpetual {
                self.flashing = None;
            }
        }
    }

    pub fn execute(
        &mut self,
        op: u16,
        s: &mut Stacks,
        defs: &HashMap<u32, Definition>,
    ) -> Result<bool> {
        match op {
            6624 => {
                let n = s.pop_int()?;
                if n >= 1 {
                    self.max_flashes = n;
                }
            }
            6625 => self.max_flashes = 3,
            6626 => {
                let n = s.pop_int()?;
                if n >= 1 {
                    self.period = n;
                }
            }
            6627 => self.period = 50,
            6628 => self.perpetual = s.pop_int()? == 1,
            6629 | 6630 => {
                let id = s.pop_int()?;
                let mut values = BTreeSet::new();
                if op == 6629 {
                    values.insert(id);
                } else {
                    let count = defs.keys().max().map_or(0, |n| n + 1);
                    for i in 0..count {
                        if defs.get(&i).map_or(-1, |d| d.category) == id {
                            values.insert(i as i32);
                        }
                    }
                }
                self.flashing = Some(values);
                self.flash_count = 0;
                self.flash_cycle = 0;
            }
            6631 => self.flashing = None,
            6632 => self.hidden = s.pop_int()? != 1,
            6633 | 6634 => {
                let enabled = s.pop_int()? == 1;
                let id = s.pop_int()?;
                let set = if op == 6633 {
                    &mut self.elements
                } else {
                    &mut self.categories
                };
                if enabled {
                    set.remove(&id);
                } else {
                    set.insert(id);
                }
                if op == 6634 {
                    let count = defs.keys().max().map_or(0, |n| n + 1);
                    for i in 0..count {
                        if defs.get(&i).map_or(-1, |d| d.category) == id {
                            if enabled {
                                self.category_elements.remove(&(i as i32));
                            } else {
                                self.category_elements.insert(i as i32);
                            }
                        }
                    }
                }
                self.excluded = self
                    .elements
                    .union(&self.category_elements)
                    .copied()
                    .collect();
            }
            6635 => s.push_int((!self.hidden) as i32)?,
            6636 => {
                let id = s.pop_int()?;
                s.push_int((!self.elements.contains(&id)) as i32)?;
            }
            6637 => {
                let id = s.pop_int()?;
                s.push_int((!self.categories.contains(&id)) as i32)?;
            }
            6693..=6696 => {
                let id = s.pop_int()?;
                let def = defs.get(&(id as u32));
                match op {
                    6693 => s.push_object(crate::vm::Object::String(
                        def.and_then(|d| d.name.clone()).unwrap_or_default(),
                    ))?,
                    6694 => s.push_int(def.map_or(0, |d| d.text_size as i32))?,
                    6695 => s.push_int(def.map_or(-1, |d| d.category))?,
                    _ => s.push_int(def.and_then(|d| d.sprites[0]).map_or(-1, |id| id as i32))?,
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
