//! Script configured widget shortcuts (gu.am, oi.ar, cg.ad).
use crate::{interface_script::ints, vm::Stacks, Error, Result};
#[derive(Clone, Debug, Default)]
pub struct Bindings {
    pub slots: Option<Box<Slots>>,
}
#[derive(Clone, Debug, Default)]
pub struct Slots {
    pub keys: [Option<Vec<(i8, i8)>>; 11],
    pub rate: [i32; 11],
    pub delay: [i32; 11],
    pub next: Option<[i32; 11]>,
}
impl Bindings {
    pub fn active(&self) -> bool {
        self.slots
            .as_ref()
            .is_some_and(|s| s.keys.iter().any(Option::is_some))
    }
    pub fn set(&mut self, index: usize, keys: Option<Vec<(i8, i8)>>) {
        if self.slots.is_none() && keys.is_none() {
            return;
        }
        self.slots.get_or_insert_with(Default::default).keys[index] = keys;
    }
    pub fn execute(&mut self, op: u16, explicit: bool, s: &mut Stacks) -> Result<bool> {
        let index = |v: i32| -> Result<usize> {
            if (1..=10).contains(&v) {
                Ok((v - 1) as usize)
            } else {
                Err(Error::Invalid("widget shortcut index"))
            }
        };
        match op {
            1350 => {
                let keys = if explicit {
                    let [key, modifier] = ints(s)?;
                    Some(vec![(key as i8, modifier as i8)])
                } else {
                    let pairs = ints::<10>(s)?;
                    let keys = pairs
                        .chunks_exact(2)
                        .take_while(|p| p[0] >= 0)
                        .map(|p| (p[0] as i8, p[1] as i8))
                        .collect::<Vec<_>>();
                    (!keys.is_empty()).then_some(keys)
                };
                let slot = index(s.pop_int()?)?;
                self.set(slot, keys);
            }
            1351 => {
                let [key, modifier] = ints(s)?;
                self.set(10, Some(vec![(key as i8, modifier as i8)]));
            }
            1352 | 1353 => {
                let (slot, rate, delay) = if op == 1352 {
                    let [slot, rate, delay] = ints(s)?;
                    (index(slot)?, rate, delay)
                } else {
                    let [delay, rate] = ints(s)?;
                    (10, rate, delay)
                };
                let slots = self
                    .slots
                    .as_mut()
                    .ok_or(Error::Invalid("widget shortcut rate without binding"))?;
                slots.rate[slot] = rate;
                slots.delay[slot] = delay;
            }
            1354 | 1355 => {
                let slot = if op == 1354 { index(s.pop_int()?)? } else { 10 };
                let slots = self
                    .slots
                    .as_mut()
                    .ok_or(Error::Invalid("widget shortcut ignore without binding"))?;
                slots.next.get_or_insert([0; 11])[slot] = i32::MAX;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

impl Bindings {
    /// Inspect one operation before its listener runs; later slots observe any
    /// binding changes made by that listener in the same interface tick.
    pub fn poll(
        &mut self,
        index: usize,
        cycle: i32,
        held: impl Fn(i32) -> bool,
        pressed: impl Fn(i32) -> bool,
    ) -> bool {
        let Some(slots) = self.slots.as_mut() else {
            return false;
        };
        let mut any = false;
        let mut trigger = false;
        if let Some(keys) = &slots.keys[index] {
            for &(key, mask) in keys {
                let key = key as i32;
                let mask = mask as u8;
                if !pressed(key) && !(slots.next.is_some() && held(key)) {
                    continue;
                }
                any = true;
                if slots.next.as_ref().is_some_and(|n| n[index] > cycle) {
                    break;
                }
                if (mask & 8 == 0 || (!held(86) && !held(82) && !held(81)))
                    && (mask & 2 == 0 || held(86))
                    && (mask & 1 == 0 || held(82))
                    && (mask & 4 == 0 || held(81))
                {
                    trigger = true;
                    break;
                }
            }
        }
        if !any {
            if let Some(next) = &mut slots.next {
                next[index] = 0;
            }
        }
        trigger
    }
    /// Java reads the repeat settings after the operation listener returns.
    pub fn dispatched(&mut self, index: usize, cycle: i32) {
        let Some(slots) = self.slots.as_mut() else {
            return;
        };
        let next = slots.next.get_or_insert([0; 11]);
        next[index] = if slots.rate[index] == 0 {
            i32::MAX
        } else {
            cycle
                .wrapping_add(slots.rate[index])
                .wrapping_add(if next[index] == 0 {
                    slots.delay[index]
                } else {
                    0
                })
        };
    }
    pub fn tick(
        &mut self,
        cycle: i32,
        enabled: bool,
        held: impl Fn(i32) -> bool,
        pressed: impl Fn(i32) -> bool,
    ) -> Vec<u8> {
        let mut actions = Vec::new();
        if enabled {
            for index in 0..11 {
                if self.poll(index, cycle, &held, &pressed) {
                    actions.push(index as u8 + 1);
                    self.dispatched(index, cycle);
                }
            }
        }
        actions
    }
}
