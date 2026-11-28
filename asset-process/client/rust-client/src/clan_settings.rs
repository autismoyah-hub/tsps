//! Full clan-settings snapshots and their client-script queries (GameClanSettings).
use crate::{
    buffer::Reader,
    interface_script::{ints, string},
    vm::{Array, Object, Stacks, Value},
    Error, Result,
};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Debug)]
pub struct Member {
    pub hash: Option<u64>,
    pub name: Option<String>,
    pub rank: i8,
    pub bits: i32,
    pub joined: u16,
    pub muted: bool,
}
#[derive(Clone, Debug)]
pub struct Banned {
    pub hash: Option<u64>,
    pub name: Option<String>,
}
#[derive(Clone, Debug)]
pub struct Settings {
    pub key: u64,
    pub revision: i32,
    pub created: i32,
    pub name: String,
    pub guests: bool,
    pub ranks: [i8; 4],
    pub hashes: bool,
    pub names: bool,
    pub members: Vec<Member>,
    pub banned: Vec<Banned>,
    pub owner: i32,
    pub deputy: i32,
    pub params: Vec<(u32, Value)>,
}
fn nullable(r: &mut Reader<'_>) -> Result<Option<String>> {
    let value = r.string()?;
    Ok((!value.is_empty()).then_some(value))
}
impl Settings {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let version = r.u8()?;
        if !(1..=6).contains(&version) {
            return Err(Error::Unsupported("clan settings version"));
        }
        let flags = r.u8()?;
        let hashes = flags & 1 != 0;
        let names = flags & 2 != 0;
        let revision = r.u32()? as i32;
        let mut created = r.u32()? as i32;
        if version <= 3 && created != 0 {
            created = created.wrapping_add(16912800);
        }
        let count = r.u16()? as usize;
        let banned_count = r.u8()? as usize;
        let name = r.string()?;
        if version >= 4 {
            r.u32()?;
        }
        let guests = r.u8()? == 1;
        let ranks = [r.u8()? as i8, r.u8()? as i8, r.u8()? as i8, r.u8()? as i8];
        let mut members = Vec::with_capacity(count);
        for _ in 0..count {
            members.push(Member {
                hash: if hashes { Some(r.u64()?) } else { None },
                name: if names { nullable(&mut r)? } else { None },
                rank: r.u8()? as i8,
                bits: if version >= 2 { r.u32()? as i32 } else { 0 },
                joined: if version >= 5 { r.u16()? } else { 0 },
                muted: version >= 6 && r.u8()? == 1,
            });
        }
        let mut banned = Vec::with_capacity(banned_count);
        for _ in 0..banned_count {
            banned.push(Banned {
                hash: if hashes { Some(r.u64()?) } else { None },
                name: if names { nullable(&mut r)? } else { None },
            });
        }
        let mut params = Vec::new();
        if version >= 3 {
            for _ in 0..r.u16()? {
                let key = r.u32()?;
                let value = match key >> 30 {
                    0 => Value::Int(r.u32()? as i32),
                    1 => Value::Long(r.u64()? as i64),
                    2 => Value::Object(Object::String(r.string()?)),
                    // The reference consumes no payload for tag 3.
                    _ => continue,
                };
                params.push((key & 0x3fffffff, value));
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("clan settings trailing bytes"));
        }
        let mut result = Self {
            key: 0,
            revision,
            created,
            name,
            guests,
            ranks,
            hashes,
            names,
            members,
            banned,
            owner: -1,
            deputy: -1,
            params,
        };
        result.update_owner();
        Ok(result)
    }
    pub fn update_owner(&mut self) {
        self.owner = -1;
        self.deputy = -1;
        if self.members.is_empty() {
            return;
        }
        let mut owner = 0;
        let mut rank = self.members[0].rank;
        for (i, m) in self.members.iter().enumerate().skip(1) {
            if m.rank > rank {
                if rank == 125 {
                    self.deputy = owner as i32;
                }
                owner = i;
                rank = m.rank;
            } else if self.deputy == -1 && m.rank == 125 {
                self.deputy = i as i32;
            }
        }
        self.owner = owner as i32;
        self.members[owner].rank = 126;
    }
    pub fn sorted(&self) -> Result<Vec<i32>> {
        if !self.names && !self.members.is_empty() {
            return Err(Error::Invalid("clan names absent"));
        }
        let mut keys = Array::Strings(
            self.members
                .iter()
                .map(|m| m.name.as_ref().map(|n| n.to_lowercase()))
                .collect(),
        );
        let mut values = Array::Ints((0..self.members.len() as i32).collect());
        crate::script_array::sort(&mut keys, Some(&mut values))?;
        let Array::Ints(values) = values else {
            unreachable!()
        };
        Ok(values)
    }
}
#[derive(Default)]
pub struct Clans {
    pub settings: [Option<Rc<RefCell<Settings>>>; 4],
    pub guest: Option<Rc<RefCell<Settings>>>,
    active: Option<Rc<RefCell<Settings>>>,
}
impl Clans {
    pub fn apply(&mut self, slot: i8, settings: Option<Settings>) -> Result<()> {
        let entry = if slot < 0 {
            &mut self.guest
        } else {
            self.settings
                .get_mut(slot as usize)
                .ok_or(Error::Invalid("clan slot"))?
        };
        *entry = settings.map(|s| Rc::new(RefCell::new(s)));
        Ok(())
    }
    pub fn execute(&mut self, op: u16, s: &mut Stacks) -> Result<bool> {
        if matches!(op, 3800 | 3801) {
            let clan = if op == 3800 {
                self.guest.clone()
            } else {
                let index = s.pop_int()?;
                self.settings
                    .get(index as usize)
                    .ok_or(Error::Invalid("clan slot"))?
                    .clone()
            };
            s.push_int(i32::from(clan.is_some()))?;
            // An unsuccessful lookup keeps the previously selected object.
            if clan.is_some() {
                self.active = clan;
            }
            return Ok(true);
        }
        if !matches!(op,3802..=3807|3809..=3818|3820|3822) {
            return Ok(false);
        }
        let clan = self
            .active
            .as_ref()
            .ok_or(Error::Invalid("no selected clan settings"))?
            .borrow();
        let member = |i: i32| {
            clan.members
                .get(i as usize)
                .ok_or(Error::Invalid("clan member index"))
        };
        match op {
            3802 => s.push_object(Object::String(clan.name.clone()))?,
            3803 => s.push_int(i32::from(clan.guests))?,
            3804..=3807 => s.push_int(clan.ranks[(op - 3804) as usize] as i32)?,
            3809 => s.push_int(clan.members.len() as i32)?,
            3810 => {
                let i = s.pop_int()?;
                if !clan.names {
                    return Err(Error::Invalid("clan names absent"));
                }
                s.push_object(member(i)?.name.clone().map_or(Object::Null, Object::String))?;
            }
            3811 => {
                let i = s.pop_int()?;
                s.push_int(member(i)?.rank as i32)?;
            }
            3812 => s.push_int(clan.banned.len() as i32)?,
            3813 => {
                let i = s.pop_int()?;
                if !clan.names {
                    return Err(Error::Invalid("clan names absent"));
                }
                s.push_object(
                    clan.banned
                        .get(i as usize)
                        .ok_or(Error::Invalid("clan banned index"))?
                        .name
                        .clone()
                        .map_or(Object::Null, Object::String),
                )?;
            }
            3814 => {
                let [i, start, end] = ints(s)?;
                let mask = if end == 31 {
                    -1i32
                } else {
                    1i32.wrapping_shl(end.wrapping_add(1) as u32)
                        .wrapping_sub(1)
                };
                s.push_int(((member(i)?.bits & mask) as u32).wrapping_shr(start as u32) as i32)?;
            }
            3815 => s.push_int(clan.owner)?,
            3816 => s.push_int(clan.deputy)?,
            3817 => {
                let name = string(s)?;
                let mut found = -1;
                if !name.is_empty() {
                    if !clan.names && !clan.members.is_empty() {
                        return Err(Error::Invalid("clan names absent"));
                    }
                    for (i, m) in clan.members.iter().enumerate() {
                        let n = m
                            .name
                            .as_ref()
                            .ok_or(Error::Invalid("null clan name lookup"))?;
                        if n == &name {
                            found = i as i32;
                            break;
                        }
                    }
                }
                s.push_int(found)?;
            }
            3818 => {
                let i = s.pop_int()?;
                s.push_int(
                    *clan
                        .sorted()?
                        .get(i as usize)
                        .ok_or(Error::Invalid("clan sorted index"))?,
                )?;
            }
            3820 => {
                let i = s.pop_int()?;
                s.push_int(member(i)?.joined as i32)?;
            }
            3822 => {
                let i = s.pop_int()?;
                s.push_int(i32::from(member(i)?.muted))?;
            }
            _ => unreachable!(),
        }
        Ok(true)
    }
}

impl Clans {
    pub fn delta(
        &mut self,
        slot: i8,
        delta: crate::clan_delta::Delta,
    ) -> Result<Option<crate::protocol::Packet>> {
        let target = if slot < 0 {
            &self.guest
        } else {
            self.settings
                .get(slot as usize)
                .ok_or(Error::Invalid("clan slot"))?
        };
        if target
            .as_ref()
            .is_none_or(|c| delta.revision > c.borrow().revision)
        {
            return Ok(Some(crate::protocol::Packet {
                opcode: 2,
                payload: vec![slot as u8],
            }));
        }
        let mut settings = target.as_ref().unwrap().borrow_mut();
        if delta.revision < settings.revision {
            return Ok(None);
        }
        delta.apply(&mut settings)?;
        Ok(None)
    }
}

impl Clans {
    pub fn parameter(&self, id: i32) -> Result<i32> {
        let clan = self
            .active
            .as_ref()
            .ok_or(Error::Invalid("no selected clan settings"))?
            .borrow();
        Ok(
            match clan
                .params
                .iter()
                .find(|(key, _)| *key == id as u32)
                .map(|(_, v)| v)
            {
                Some(Value::Int(value)) => *value,
                _ => -1,
            },
        )
    }
}
impl Clans {
    /// Actor clan flags exclude slot 2 and the guest clan (GamePlayer.ao).
    pub fn actor_member(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }
        let name = name.to_lowercase();
        self.settings
            .iter()
            .enumerate()
            .filter(|(slot, _)| *slot != 2)
            .filter_map(|(_, s)| s.as_ref())
            .any(|s| {
                s.borrow()
                    .members
                    .iter()
                    .any(|m| m.name.as_ref().is_some_and(|n| n.to_lowercase() == name))
            })
    }
}
