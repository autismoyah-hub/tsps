//! Location add/remove decoding and pending-spawn coalescing (gl.dw).
use crate::{
    buffer::{Reader, Transform},
    Error, Result,
};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Object {
    pub id: i32,
    pub shape: u8,
    pub rotation: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub at: u8,
    pub object: Object,
    pub operations: u8,
    pub labels: Option<[Option<String>; 5]>,
}
pub fn category(shape: u8) -> Result<u8> {
    Ok(match shape {
        0..=3 => 0,
        4..=8 => 1,
        9..=21 => 2,
        22 => 3,
        _ => return Err(Error::Invalid("location shape")),
    })
}
impl Change {
    pub fn decode(kind: u8, raw: &[u8]) -> Result<Self> {
        let mut r = Reader::new(raw);
        let result = match kind {
            0 => {
                let value = r.u8()?;
                let at = r.transformed(Transform::Neg)?;
                Self {
                    at,
                    object: Object {
                        id: -1,
                        shape: value >> 2,
                        rotation: value & 3,
                    },
                    operations: 31,
                    labels: None,
                }
            }
            7 => {
                let count = r.transformed(Transform::Add)?;
                let mut labels = (count > 0).then(|| std::array::from_fn(|_| None));
                for _ in 0..count {
                    let index = r.u8()?;
                    let text = r.string()?;
                    if index < 5 {
                        labels.as_mut().unwrap()[index as usize] = Some(text);
                    }
                }
                let operations = r.transformed(Transform::Sub)?;
                let value = r.transformed(Transform::Add)?;
                let at = r.transformed(Transform::Neg)?;
                let id = r.u16_add(true)? as i32;
                Self {
                    at,
                    object: Object {
                        id,
                        shape: value >> 2,
                        rotation: value & 3,
                    },
                    operations,
                    labels,
                }
            }
            _ => return Err(Error::Invalid("location record kind")),
        };
        category(result.object.shape)?;
        if r.remaining() != 0 {
            return Err(Error::Invalid("location record trailing bytes"));
        }
        Ok(result)
    }
}
#[derive(Clone, Debug)]
pub struct Pending {
    pub original: Object,
    pub replacement: Object,
    pub operations: u8,
    pub labels: Option<[Option<String>; 5]>,
    pub delay: i32,
    pub lifetime: i32,
    pub animation: i32,
}
#[derive(Clone, Debug, Default)]
pub struct Queue {
    pub entries: BTreeMap<[i32; 4], Pending>,
    order: Vec<[i32; 4]>,
}
impl Queue {
    pub fn merge(
        &mut self,
        key: [i32; 4],
        start: u16,
        end: u16,
        original: impl FnOnce([i32; 4]) -> Object,
    ) {
        if !self.entries.contains_key(&key) {
            self.order.push(key);
        }
        let entry = self.entries.entry(key).or_insert_with(|| Pending {
            original: original(key),
            replacement: Object {
                id: -1,
                shape: 0,
                rotation: 0,
            },
            operations: 31,
            labels: None,
            delay: 0,
            lifetime: -1,
            animation: -1,
        });
        entry.replacement = Object {
            id: -1,
            shape: 0,
            rotation: 0,
        };
        entry.operations = 31;
        entry.labels = None;
        entry.delay = start as i32 + 1;
        entry.lifetime = end as i32 + 1;
    }
    pub fn apply(
        &mut self,
        origin: [i32; 3],
        change: Change,
        original: impl FnOnce([i32; 4]) -> Object,
    ) -> Result<()> {
        let key = [
            origin[0],
            origin[1] + ((change.at >> 4) & 7) as i32,
            origin[2] + (change.at & 7) as i32,
            category(change.object.shape)? as i32,
        ];
        if !self.entries.contains_key(&key) {
            self.order.push(key);
        }
        let entry = self.entries.entry(key).or_insert_with(|| Pending {
            original: original(key),
            replacement: change.object.clone(),
            operations: 31,
            labels: None,
            delay: 0,
            lifetime: -1,
            animation: -1,
        });
        entry.replacement = change.object;
        entry.operations = change.operations;
        entry.labels = change.labels;
        entry.delay = 0;
        entry.lifetime = -1;
        Ok(())
    }
    pub fn clear(&mut self, origin: [i32; 3]) {
        for (key, p) in &mut self.entries {
            if key[0] == origin[0]
                && (origin[1]..origin[1] + 8).contains(&key[1])
                && (origin[2]..origin[2] + 8).contains(&key[2])
            {
                p.lifetime = 0;
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    Merge([i32; 3], crate::location_merge::Request),
    Animate([i32; 3], crate::location_animation::Request),
    Change([i32; 3], Change),
    Clear([i32; 3]),
}
impl Queue {
    /// One pending-spawn cycle. A false callback result retains a model that is not ready.
    pub fn tick(
        &mut self,
        base: [i32; 2],
        apply: impl FnMut([i32; 4], &Object, i32) -> Result<bool>,
    ) -> Result<usize> {
        self.advance(base, 1, apply)
    }
    pub fn advance(
        &mut self,
        base: [i32; 2],
        ticks: u32,
        mut apply: impl FnMut([i32; 4], &Object, i32) -> Result<bool>,
    ) -> Result<usize> {
        let mut changed = 0;
        for _ in 0..ticks.max(1) {
            changed += self.tick_ready(base, ticks != 0, &mut apply)?;
        }
        Ok(changed)
    }
    fn tick_ready(
        &mut self,
        base: [i32; 2],
        elapsed: bool,
        mut apply: impl FnMut([i32; 4], &Object, i32) -> Result<bool>,
    ) -> Result<usize> {
        let mut changed = 0;
        for key in self.order.clone() {
            let Some(p) = self.entries.get_mut(&key) else {
                continue;
            };
            if elapsed && p.lifetime > 0 {
                p.lifetime -= 1;
            }
            if p.lifetime == 0 {
                if apply(key, &p.original, p.animation)? {
                    self.entries.remove(&key);
                    changed += 1;
                }
            } else {
                if elapsed && p.delay > 0 {
                    p.delay -= 1;
                }
                if p.delay != 0
                    || !(1..=102).contains(&(key[1] - base[0]))
                    || !(1..=102).contains(&(key[2] - base[1]))
                {
                    continue;
                }
                if apply(key, &p.replacement, p.animation)? {
                    p.delay = -1;
                    changed += 1;
                    if (p.replacement.id == -1 && p.original.id == -1)
                        || p.replacement == p.original
                    {
                        self.entries.remove(&key);
                    }
                }
            }
        }
        self.order.retain(|k| self.entries.contains_key(k));
        Ok(changed)
    }
    pub fn rebuild(&mut self, base: [i32; 2]) {
        self.entries.retain(|k, p| {
            let keep = (0..104).contains(&(k[1] - base[0])) && (0..104).contains(&(k[2] - base[1]));
            if keep && p.lifetime != 0 {
                p.delay = 0;
            }
            keep
        });
        self.order.retain(|k| self.entries.contains_key(k));
    }
}

impl Queue {
    /// Java dx.af scans the owning world's pending list, in insertion order, by tile and object ID.
    pub fn menu_state(&self, plane:i32, tile:[u16;2], id:u16)->Option<&Pending> {
        self.order.iter().filter(|key|key[0]==plane&&key[1]==tile[0] as i32&&key[2]==tile[1] as i32)
            .filter_map(|key|self.entries.get(key)).find(|pending|pending.replacement.id==id as i32)
    }
}
