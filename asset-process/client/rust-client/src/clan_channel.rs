//! Clan-channel snapshots and script queries (GameClanChannel).
use crate::{
    buffer::Reader,
    interface_script::string,
    vm::{Array, Object, Stacks},
    Error, Result,
};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Debug)]
pub struct Member {
    pub name: Option<String>,
    pub rank: i8,
    pub world: u16,
}
#[derive(Clone, Debug)]
pub struct Channel {
    pub key: u64,
    pub revision: u64,
    pub name: Option<String>,
    pub kick: i8,
    pub talk: i8,
    pub members: Option<Vec<Member>>,
    sorted: RefCell<Option<Vec<i32>>>,
}
impl Channel {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let flags = r.u8()?;
        let version = if flags & 4 != 0 { r.u8()? } else { 2 };
        let key = r.u64()?;
        let revision = r.u64()?;
        let name = r.string()?;
        r.u8()?;
        let kick = r.u8()? as i8;
        let talk = r.u8()? as i8;
        let count = r.u16()?;
        let mut members = Vec::with_capacity(count as usize);
        for _ in 0..count {
            if flags & 1 != 0 {
                r.u64()?;
            }
            // ap defaults true in the constructor, even without flag 2.
            let name = r.string()?;
            let rank = r.u8()? as i8;
            let world = r.u16()?;
            if version >= 3 {
                r.u8()?;
            }
            members.push(Member {
                name: Some(name),
                rank,
                world,
            });
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("clan channel trailing bytes"));
        }
        Ok(Self {
            key,
            revision,
            name: Some(name),
            kick,
            talk,
            members: (count > 0).then_some(members),
            sorted: RefCell::new(None),
        })
    }
    pub fn members(&self) -> Result<&[Member]> {
        self.members
            .as_deref()
            .ok_or(Error::Invalid("null clan channel member list"))
    }
    pub fn sorted(&self) -> Result<Vec<i32>> {
        if let Some(values) = self.sorted.borrow().as_ref() {
            return Ok(values.clone());
        }
        let members = self.members()?;
        let mut keys = Array::Strings(
            members
                .iter()
                .map(|m| {
                    m.name
                        .as_ref()
                        .and_then(|name| crate::username::normalize(name, 12))
                })
                .collect(),
        );
        let mut values = Array::Ints((0..members.len() as i32).collect());
        crate::script_array::sort(&mut keys, Some(&mut values))?;
        let Array::Ints(values) = values else {
            unreachable!()
        };
        *self.sorted.borrow_mut() = Some(values.clone());
        Ok(values)
    }
}
#[derive(Default)]
pub struct Channels {
    pub channels: [Option<Rc<RefCell<Channel>>>; 4],
    pub guest: Option<Rc<RefCell<Channel>>>,
    active: Option<Rc<RefCell<Channel>>>,
    pub selected_slot: i32,
}
impl Channels {
    pub fn apply(&mut self, slot: i8, channel: Option<Channel>) -> Result<()> {
        let entry = if slot < 0 {
            &mut self.guest
        } else {
            self.channels
                .get_mut(slot as usize)
                .ok_or(Error::Invalid("clan channel slot"))?
        };
        *entry = channel.map(|c| Rc::new(RefCell::new(c)));
        Ok(())
    }
    pub fn execute(&mut self, op: u16, s: &mut Stacks) -> Result<bool> {
        if matches!(op, 3850 | 3851) {
            let slot = if op == 3851 { Some(s.pop_int()?) } else { None };
            let channel = if let Some(i) = slot {
                self.channels
                    .get(i as usize)
                    .ok_or(Error::Invalid("clan channel slot"))?
                    .clone()
            } else {
                self.guest.clone()
            };
            s.push_int(i32::from(channel.is_some()))?;
            if channel.is_some() {
                self.active = channel;
                if let Some(i) = slot {
                    self.selected_slot = i;
                }
            }
            return Ok(true);
        }
        if !matches!(op, 3852..=3858 | 3860 | 3861) {
            return Ok(false);
        }
        let channel = self
            .active
            .as_ref()
            .ok_or(Error::Invalid("no selected clan channel"))?
            .borrow();
        match op {
            3852 => s.push_object(channel.name.clone().map_or(Object::Null, Object::String))?,
            3853 => s.push_int(channel.kick as i32)?,
            3854 => s.push_int(channel.talk as i32)?,
            3855 => s.push_int(channel.members()?.len() as i32)?,
            3856..=3858 => {
                let i = s.pop_int()?;
                let m = channel
                    .members()?
                    .get(i as usize)
                    .ok_or(Error::Invalid("clan channel member index"))?;
                match op {
                    3856 => s.push_object(m.name.clone().map_or(Object::Null, Object::String))?,
                    3857 => s.push_int(m.rank as i32)?,
                    _ => s.push_int(m.world as i32)?,
                }
            }
            3860 => {
                let name = string(s)?;
                let mut found = -1;
                for (i, m) in channel.members()?.iter().enumerate() {
                    let actual = m
                        .name
                        .as_ref()
                        .ok_or(Error::Invalid("null clan channel display name"))?;
                    if crate::string_compare::equals_ignore_case_bmp(actual, &name) {
                        found = i as i32;
                        break;
                    }
                }
                s.push_int(found)?;
            }
            3861 => {
                let i = s.pop_int()?;
                s.push_int(
                    *channel
                        .sorted()?
                        .get(i as usize)
                        .ok_or(Error::Invalid("clan channel sorted index"))?,
                )?;
            }
            _ => unreachable!(),
        }
        Ok(true)
    }
}
#[derive(Clone, Debug)]
pub enum Change {
    Add(Member),
    Update {
        index: u16,
        member: Member,
    },
    Remove(u16),
    Metadata {
        name: Option<String>,
        ranks: Option<(i8, i8)>,
    },
}
#[derive(Clone, Debug)]
pub struct Delta {
    pub key: u64,
    pub revision: u64,
    pub changes: Vec<Change>,
}
fn optional_hash(r: &mut Reader<'_>) -> Result<()> {
    if r.u8()? != 255 {
        r.bytes(7)?;
    }
    Ok(())
}
fn nullable(r: &mut Reader<'_>) -> Result<Option<String>> {
    let value = r.string()?;
    Ok((!value.is_empty()).then_some(value))
}
impl Delta {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let key = r.u64()?;
        let revision = r.u64()?;
        let mut changes = Vec::new();
        loop {
            let op = r.u8()?;
            let change = match op {
                0 => break,
                1 => {
                    optional_hash(&mut r)?;
                    let name = nullable(&mut r)?;
                    let world = r.u16()?;
                    let rank = r.u8()? as i8;
                    r.u64()?;
                    Change::Add(Member { name, world, rank })
                }
                2 | 5 => {
                    if op == 5 {
                        r.u8()?;
                    }
                    let index = r.u16()?;
                    let rank = r.u8()? as i8;
                    let world = r.u16()?;
                    r.u64()?;
                    let name = Some(r.string()?);
                    if op == 5 {
                        r.u8()?;
                    }
                    Change::Update {
                        index,
                        member: Member { name, world, rank },
                    }
                }
                3 => {
                    let index = r.u16()?;
                    r.u8()?;
                    optional_hash(&mut r)?;
                    Change::Remove(index)
                }
                4 => {
                    let name = nullable(&mut r)?;
                    let ranks = if name.is_some() {
                        r.u8()?;
                        Some((r.u8()? as i8, r.u8()? as i8))
                    } else {
                        None
                    };
                    Change::Metadata { name, ranks }
                }
                _ => return Err(Error::Unsupported("clan channel delta type")),
            };
            changes.push(change);
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("clan channel delta trailing bytes"));
        }
        Ok(Self {
            key,
            revision,
            changes,
        })
    }
    pub fn apply(self, channel: &mut Channel) -> Result<()> {
        if self.key != channel.key || self.revision != channel.revision {
            return Err(Error::Invalid("clan channel delta identity or revision"));
        }
        let mut next = channel.clone();
        for change in self.changes {
            match change {
                Change::Add(member) => {
                    next.members
                        .as_mut()
                        .ok_or(Error::Invalid("null clan channel member list"))?
                        .push(member);
                    *next.sorted.borrow_mut() = None;
                }
                Change::Remove(index) => {
                    let members = next
                        .members
                        .as_mut()
                        .ok_or(Error::Invalid("null clan channel member list"))?;
                    if index as usize >= members.len() {
                        return Err(Error::Invalid("clan channel remove index"));
                    }
                    members.remove(index as usize);
                    *next.sorted.borrow_mut() = None;
                }
                Change::Update { index, member } => {
                    *next
                        .members
                        .as_mut()
                        .and_then(|m| m.get_mut(index as usize))
                        .ok_or(Error::Invalid("clan channel update index"))? = member;
                }
                Change::Metadata { name, ranks } => {
                    next.name = name;
                    if let Some((talk, kick)) = ranks {
                        next.talk = talk;
                        next.kick = kick;
                    }
                }
            }
        }
        next.revision = next.revision.wrapping_add(1);
        *channel = next;
        Ok(())
    }
}
impl Channels {
    /// Missing or newer state requests a full snapshot; stale deltas are ignored.
    pub fn delta(&mut self, slot: i8, delta: Delta) -> Result<Option<crate::protocol::Packet>> {
        let target = if slot < 0 {
            &self.guest
        } else {
            self.channels
                .get(slot as usize)
                .ok_or(Error::Invalid("clan channel slot"))?
        };
        if target
            .as_ref()
            .is_none_or(|c| (delta.revision as i64) > (c.borrow().revision as i64))
        {
            return Ok(Some(crate::protocol::Packet {
                opcode: 112,
                payload: vec![slot as u8],
            }));
        }
        let mut channel = target.as_ref().unwrap().borrow_mut();
        if (delta.revision as i64) < (channel.revision as i64) {
            return Ok(None);
        }
        delta.apply(&mut channel)?;
        Ok(None)
    }
}
impl Channels {
    /// 3819/3859 require an unranked channel member; 3821 has no rank restriction.
    pub fn management(
        &self,
        opcode: u16,
        slot: i32,
        index: i32,
        muted: bool,
    ) -> Result<Option<crate::protocol::Packet>> {
        let target = if opcode == 3859 && slot < 0 {
            &self.guest
        } else {
            self.channels
                .get(slot as usize)
                .ok_or(Error::Invalid("clan management slot"))?
        };
        let Some(channel) = target else {
            return Ok(None);
        };
        if index < 0 {
            return Ok(None);
        }
        let channel = channel.borrow();
        let Some(member) = channel.members()?.get(index as usize) else {
            return Ok(None);
        };
        if opcode != 3821 && member.rank != -1 {
            return Ok(None);
        }
        let name = member
            .name
            .as_ref()
            .ok_or(Error::Invalid("null clan management name"))?;
        let mut w = crate::writer::Writer::new();
        w.u8(slot as u8);
        w.u16(index as u16);
        if opcode == 3821 {
            w.u8(u8::from(muted));
        }
        w.string(name)?;
        let opcode = match opcode {
            3819 => 84,
            3821 => 96,
            3859 => 38,
            _ => return Err(Error::Invalid("clan management opcode")),
        };
        Ok(Some(crate::protocol::Packet {
            opcode,
            payload: w.into_vec(),
        }))
    }
}
