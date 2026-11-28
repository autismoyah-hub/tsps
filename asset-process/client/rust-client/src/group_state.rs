//! Group snapshots, typed variables and queries (pq, wi, wy, GameRenderable.cu).
use crate::{
    buffer::Reader,
    disk_cache::DiskCache,
    vm::{Object, Stacks},
    Error, Result,
};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Int(i32),
    Long(i64),
    String(String),
    Null,
}
impl Value {
    fn read(r: &mut Reader<'_>, kind: u8, compact: bool) -> Result<Self> {
        Ok(match kind {
            0 => Self::Int(if compact {
                r.varint_le_signed()?
            } else {
                r.u32()? as i32
            }),
            1 => Self::Long(r.u64()? as i64),
            2 => Self::String(r.string()?),
            3 => Self::Null,
            _ => return Err(Error::Invalid("group value type")),
        })
    }
    fn push(&self, kind: u8, stack: &mut Stacks) -> Result<()> {
        match (kind, self) {
            (0, Self::Int(v)) => stack.push_int(*v),
            (1, Self::Long(v)) => stack.push_long(*v),
            (2, Self::String(v)) => stack.push_object(Object::String(v.clone())),
            _ => Err(Error::Invalid("group query value type")),
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Definition {
    pub kinds: [Vec<u8>; 2],
    pub defaults: [Vec<Value>; 2],
}
impl Definition {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let mut result = Self::default();
        loop {
            match r.u8()? {
                0 => break,
                opcode @ (7 | 8) => {
                    let scope = (opcode - 7) as usize;
                    let count = r.u16()? as usize;
                    if count > r.remaining() {
                        return Err(Error::Truncated);
                    }
                    result.kinds[scope].clear();
                    result.defaults[scope].clear();
                    for _ in 0..count {
                        let first = r.u8()?;
                        let id = if first < 252 {
                            first as u16
                        } else {
                            ((first as u16 - 252) << 8) | r.u8()? as u16
                        };
                        let kind = crate::database_defs::base_type(id)?;
                        result.kinds[scope].push(kind);
                        result.defaults[scope].push(Value::read(&mut r, kind, false)?);
                    }
                }
                _ => return Err(Error::Unsupported("group definition opcode")),
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("group definition trailing bytes"));
        }
        Ok(result)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Group {
    pub definition: u16,
    pub identity: i64,
    pub values: [Vec<Value>; 2],
}
#[derive(Clone, Debug)]
pub struct Full {
    pub slot: u8,
    pub group: Option<Group>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub definition: u16,
    pub slot: u8,
    pub identity: i64,
}
#[derive(Clone, Debug)]
pub struct Delta {
    pub slot: u8,
    pub key: u32,
    pub value: Value,
}
impl Delta {
    pub fn decode(bytes: &[u8]) -> Result<Vec<Self>> {
        let mut r = Reader::new(bytes);
        let mut result = Vec::new();
        while r.remaining() != 0 {
            let slot = r.u8()?;
            let key = r.u32()?;
            let value = Value::read(&mut r, ((key >> 16) & 3) as u8, false)?;
            result.push(Self { slot, key, value });
        }
        Ok(result)
    }
}
#[derive(Default)]
pub struct State {
    pub definitions: BTreeMap<u16, Definition>,
    pub slots: BTreeMap<u8, Group>,
    loaded: bool,
}
impl State {
    pub fn load_definitions(&mut self, cache: &mut DiskCache) -> Result<()> {
        if !self.loaded {
            // gh.ag constructs an empty definition when the archive has no file.
            if !cache.reference(2)?.groups.iter().any(|g| g.id == 75) {
                self.loaded = true;
                return Ok(());
            }
            let definitions = cache
                .group(2, 75, [0; 4])?
                .into_iter()
                .map(|(id, bytes)| Ok((id as u16, Definition::decode(&bytes)?)))
                .collect::<Result<_>>()?;
            self.definitions = definitions;
            self.loaded = true;
        }
        Ok(())
    }
    pub fn clear(&mut self) {
        self.slots.clear();
    }
    pub fn decode_full(&self, bytes: &[u8]) -> Result<Vec<Full>> {
        let mut r = Reader::new(bytes);
        let mut entries = Vec::new();
        while r.remaining() != 0 {
            let slot = r.u8()?;
            let id = r.unsigned_smart()? as i32 - 1;
            let group = if id == -1 {
                None
            } else {
                let identity = r.u64()? as i64;
                let empty = Definition::default();
                let definition = self.definitions.get(&(id as u16)).unwrap_or(&empty);
                let mut values: [Vec<Value>; 2] = Default::default();
                for (dest, kinds) in values.iter_mut().zip(&definition.kinds) {
                    for &kind in kinds {
                        dest.push(Value::read(&mut r, kind, true)?);
                    }
                }
                Some(Group {
                    definition: id as u16,
                    identity,
                    values,
                })
            };
            entries.push(Full { slot, group });
        }
        Ok(entries)
    }
    /// Java fires removal while the previous entry remains queryable.
    pub fn removal(&self, entry: &Full) -> Option<Event> {
        let old = self.slots.get(&entry.slot)?;
        if entry
            .group
            .as_ref()
            .is_some_and(|g| g.definition == old.definition && g.identity == old.identity)
        {
            return None;
        }
        Some(Event {
            definition: old.definition,
            slot: entry.slot,
            identity: -1,
        })
    }
    /// Fires addition only after the complete new snapshot has been installed.
    pub fn apply_full(&mut self, entry: Full) -> Option<Event> {
        if let Some(group) = entry.group {
            let changed = self.slots.get(&entry.slot).is_none_or(|old| {
                old.definition != group.definition || old.identity != group.identity
            });
            let event = Event {
                definition: group.definition,
                slot: entry.slot,
                identity: group.identity,
            };
            self.slots.insert(entry.slot, group);
            changed.then_some(event)
        } else {
            self.slots.remove(&entry.slot);
            None
        }
    }
    pub fn apply_delta(&mut self, delta: Delta) -> Result<()> {
        let Some(group) = self.slots.get_mut(&delta.slot) else {
            return Ok(());
        };
        if group.definition != ((delta.key >> 20) & 2047) as u16 {
            return Ok(());
        }
        let scope = ((delta.key >> 18) & 3) as usize;
        *group
            .values
            .get_mut(scope)
            .and_then(|v| v.get_mut((delta.key & 65535) as usize))
            .ok_or(Error::Invalid("group variable scope/index"))? = delta.value;
        Ok(())
    }
    pub fn find(&self, identity: i64) -> Option<(u8, &Group)> {
        self.slots
            .iter()
            .find(|(_, g)| g.identity == identity)
            .map(|(&slot, g)| (slot, g))
    }
    pub fn execute(&self, opcode: u16, stack: &mut Stacks) -> Result<bool> {
        match opcode {
            8501 => {
                let identity = stack.pop_long()?;
                let definition = stack.pop_int()?;
                stack.push_int(
                    self.find(identity)
                        .is_some_and(|(_, g)| g.definition as i32 == definition)
                        as i32,
                )?;
            }
            8520 => {
                let slot = stack.pop_int()?;
                let group = u8::try_from(slot).ok().and_then(|s| self.slots.get(&s));
                stack.push_long(group.map_or(-1, |g| g.identity))?;
            }
            8521 => {
                let identity = stack.pop_long()?;
                stack.push_int(self.find(identity).map_or(-1, |(slot, _)| slot as i32))?;
            }
            8560 | 8570 => {
                let key = stack.pop_int()? as u32;
                let identity = stack.pop_long()?;
                let definition = ((key >> 20) & 2047) as u16;
                let index = (key & 65535) as usize;
                let scope = usize::from(opcode == 8570);
                let value = if let Some((_, group)) = self.find(identity) {
                    if group.definition != definition {
                        return Err(Error::Invalid("group query definition mismatch"));
                    }
                    group.values[scope].get(index)
                } else {
                    self.definitions
                        .get(&definition)
                        .and_then(|d| d.defaults[scope].get(index))
                }
                .ok_or(Error::Invalid("group query index"))?;
                value.push(((key >> 16) & 3) as u8, stack)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
