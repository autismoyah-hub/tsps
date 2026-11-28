//! Incremental clan settings records (ik and its fifteen hh record variants).
use crate::{
    buffer::Reader,
    clan_settings::{Banned, Member, Settings},
    vm::{Object, Value},
    Error, Result,
};
#[derive(Clone, Debug)]
pub enum Change {
    Add {
        hash: i64,
        name: Option<String>,
        joined: u16,
        banned: bool,
    },
    Rank(u16, i8),
    RemoveMember(u16),
    RemoveBanned(u16),
    Permissions(bool, [i8; 4]),
    MemberBits(u16, i32, u8, u8),
    Muted(u16, bool),
    Parameter(u32, Value),
    ParameterBits(u32, i32, u8, u8),
    Name(String),
    Owner(u16),
}
#[derive(Clone, Debug)]
pub struct Delta {
    pub key: u64,
    pub revision: i32,
    pub changes: Vec<Change>,
}
fn identity(r: &mut Reader<'_>) -> Result<(i64, Option<String>)> {
    let first = r.u8()?;
    let hash = if first == 255 {
        -1
    } else {
        let mut bytes = [0u8; 8];
        bytes[0] = first;
        bytes[1..].copy_from_slice(r.bytes(7)?);
        i64::from_be_bytes(bytes)
    };
    let name = r.string()?;
    Ok((hash, (!name.is_empty()).then_some(name)))
}
fn bitfield(old: i32, value: i32, start: u8, end: u8) -> i32 {
    let low = 1i32.wrapping_shl(start as u32).wrapping_sub(1);
    let high = if end == 31 {
        -1
    } else {
        1i32.wrapping_shl(end as u32 + 1).wrapping_sub(1)
    };
    let mask = high ^ low;
    (old & !mask) | (value.wrapping_shl(start as u32) & mask)
}
impl Delta {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let key = r.u64()?;
        let revision = r.u32()? as i32;
        let mut changes = Vec::new();
        loop {
            let op = r.u8()?;
            let change = match op {
                0 => break,
                1 | 3 | 13 => {
                    let (hash, name) = identity(&mut r)?;
                    let joined = if op == 13 { r.u16()? } else { 0 };
                    Change::Add {
                        hash,
                        name,
                        joined,
                        banned: op == 3,
                    }
                }
                2 => Change::Rank(r.u16()?, r.u8()? as i8),
                4 => Change::Permissions(
                    r.u8()? == 1,
                    [r.u8()? as i8, r.u8()? as i8, r.u8()? as i8, r.u8()? as i8],
                ),
                5 => Change::RemoveMember(r.u16()?),
                6 => Change::RemoveBanned(r.u16()?),
                7 => Change::MemberBits(r.u16()?, r.u32()? as i32, r.u8()?, r.u8()?),
                8 => Change::Parameter(r.u32()?, Value::Int(r.u32()? as i32)),
                9 => Change::Parameter(r.u32()?, Value::Long(r.u64()? as i64)),
                10 => {
                    let key = r.u32()?;
                    let name = r.string()?;
                    Change::Parameter(
                        key,
                        Value::Object(Object::String(name.chars().take(80).collect())),
                    )
                }
                11 => Change::ParameterBits(r.u32()?, r.u32()? as i32, r.u8()?, r.u8()?),
                12 => {
                    let name = r.string()?;
                    r.u32()?;
                    Change::Name(name)
                }
                14 => Change::Muted(r.u16()?, r.u8()? == 1),
                15 => Change::Owner(r.u16()?),
                _ => return Err(Error::Unsupported("clan settings delta type")),
            };
            changes.push(change);
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("clan settings delta trailing bytes"));
        }
        Ok(Self {
            key,
            revision,
            changes,
        })
    }
    pub fn apply(self, settings: &mut Settings) -> Result<()> {
        if self.key != settings.key || self.revision != settings.revision {
            return Err(Error::Invalid("clan settings delta identity or revision"));
        }
        let mut next = settings.clone();
        for change in self.changes {
            match change {
                Change::Add {
                    hash,
                    name,
                    joined,
                    banned,
                } => {
                    if (hash > 0) != next.hashes || name.is_some() != next.names {
                        return Err(Error::Invalid("clan identity flags"));
                    }
                    let hash = next.hashes.then_some(hash as u64);
                    if banned {
                        next.banned.push(Banned { hash, name });
                    } else {
                        let rank = if next.owner == -1 {
                            next.owner = next.members.len() as i32;
                            126
                        } else {
                            0
                        };
                        next.members.push(Member {
                            hash,
                            name,
                            rank,
                            bits: 0,
                            joined,
                            muted: false,
                        });
                    }
                }
                Change::RemoveMember(i) => {
                    if i as usize >= next.members.len() {
                        return Err(Error::Invalid("clan remove member index"));
                    }
                    next.members.remove(i as usize);
                    next.update_owner();
                }
                Change::RemoveBanned(i) => {
                    if i as usize >= next.banned.len() {
                        return Err(Error::Invalid("clan remove banned index"));
                    }
                    next.banned.remove(i as usize);
                }
                Change::Rank(i, rank) => {
                    if matches!(rank, 126 | 127) {
                        continue;
                    }
                    if i as i32 == next.owner
                        && (next.deputy == -1
                            || next
                                .members
                                .get(next.deputy as usize)
                                .is_none_or(|m| m.rank < 125))
                    {
                        continue;
                    }
                    let member = next
                        .members
                        .get_mut(i as usize)
                        .ok_or(Error::Invalid("clan rank member index"))?;
                    if member.rank != rank {
                        member.rank = rank;
                        next.update_owner();
                    }
                }
                Change::Owner(i) => {
                    if i as i32 == next.owner {
                        continue;
                    }
                    let rank = next
                        .members
                        .get(i as usize)
                        .ok_or(Error::Invalid("clan owner index"))?
                        .rank;
                    if rank != 126 {
                        next.members
                            .get_mut(next.owner as usize)
                            .ok_or(Error::Invalid("clan previous owner index"))?
                            .rank = 125;
                        next.deputy = next.owner;
                        next.members[i as usize].rank = 126;
                        next.owner = i as i32;
                    }
                }
                Change::Permissions(guests, ranks) => {
                    next.guests = guests;
                    next.ranks = ranks;
                }
                Change::MemberBits(i, value, start, end) => {
                    let m = next
                        .members
                        .get_mut(i as usize)
                        .ok_or(Error::Invalid("clan bits member index"))?;
                    m.bits = bitfield(m.bits, value, start, end);
                }
                Change::Muted(i, value) => {
                    next.members
                        .get_mut(i as usize)
                        .ok_or(Error::Invalid("clan muted member index"))?
                        .muted = value;
                }
                Change::Name(name) => next.name = name,
                Change::Parameter(key, value) => set_parameter(&mut next, key, value),
                Change::ParameterBits(key, value, start, end) => {
                    let old = next
                        .params
                        .iter()
                        .find(|(k, _)| *k == key)
                        .and_then(|(_, v)| {
                            if let Value::Int(v) = v {
                                Some(*v)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0);
                    set_parameter(&mut next, key, Value::Int(bitfield(old, value, start, end)));
                }
            }
        }
        next.revision = next.revision.wrapping_add(1);
        *settings = next;
        Ok(())
    }
}
fn set_parameter(settings: &mut Settings, key: u32, value: Value) {
    if let Some(index) = settings.params.iter().position(|(k, _)| *k == key) {
        match (&mut settings.params[index].1, &value) {
            (Value::Int(old), Value::Int(v)) => {
                *old = *v;
                return;
            }
            (Value::Long(old), Value::Long(v)) => {
                *old = *v;
                return;
            }
            (Value::Object(Object::String(old)), Value::Object(Object::String(v))) if old == v => {
                return
            }
            _ => {
                settings.params.remove(index);
            }
        }
    }
    settings.params.push((key, value));
}
