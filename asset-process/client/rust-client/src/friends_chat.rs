//! Revision 239.1 GameFriendsChatManager snapshots and incremental membership.
use crate::{buffer::Reader, username, Error, Result};
#[derive(Clone, Debug)]
pub struct Member {
    pub name: String,
    pub world: i32,
    pub rank: i32,
    pub sequence: i32,
}
#[derive(Clone, Debug)]
pub struct Channel {
    pub name: String,
    pub owner: String,
    pub minimum_kick: i32,
    pub rank: i32,
    pub members: Vec<Member>,
    sequence: i32,
    pub sort: crate::social_sort::Chain,
}
impl Default for Channel {
    fn default() -> Self {
        Self {
            name: String::new(),
            owner: String::new(),
            minimum_kick: 0,
            rank: 0,
            members: Vec::new(),
            sequence: 1,
            sort: Default::default(),
        }
    }
}
fn canonical(text: &str) -> String {
    let mut value = 0u64;
    for c in text.encode_utf16() {
        value *= 37;
        value += match c {
            65..=90 => (c - 64) as u64,
            97..=122 => (c - 96) as u64,
            48..=57 => (c - 21) as u64,
            _ => 0,
        };
        if value >= 177917621779460413 {
            break;
        }
    }
    while value != 0 && value % 37 == 0 {
        value /= 37;
    }
    crate::chat_packet::base37(value).unwrap_or_default()
}
fn same(a: &str, b: &str) -> bool {
    username::normalize(a, 12).is_some_and(|a| Some(a) == username::normalize(b, 12))
}
impl Channel {
    pub fn contains(&self, name: &str) -> bool {
        self.members.iter().any(|m| same(&m.name, name))
    }
    fn local_rank(&mut self, index: usize, local: &str) {
        if same(&self.members[index].name, local) {
            self.rank = self.members[index].rank;
        }
    }
    pub fn snapshot(&mut self, bytes: &[u8], local: &str) -> Result<()> {
        let mut r = Reader::new(bytes);
        self.owner = canonical(&r.string()?);
        self.name = crate::chat_packet::base37(r.u64()?)
            .ok_or(Error::Invalid("invalid friends-chat name"))?;
        self.minimum_kick = r.u8()? as i8 as i32;
        let count = r.unsigned_smart()? as i32 - 1;
        if count >= 0 {
            if count > 500 {
                return Err(Error::Invalid("friends-chat capacity"));
            }
            self.members.clear();
            for _ in 0..count {
                let name = r.string()?;
                if self.contains(&name) {
                    return Err(Error::Invalid("duplicate friends-chat member"));
                }
                let world = r.u16()? as i32;
                let rank = r.u8()? as i8 as i32;
                r.string()?;
                self.members.push(Member {
                    name,
                    world,
                    rank,
                    sequence: self.sequence,
                });
                self.sequence = self.sequence.wrapping_add(1);
                self.local_rank(self.members.len() - 1, local);
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("friends-chat snapshot trailing bytes"));
        }
        Ok(())
    }
    pub fn delta(&mut self, bytes: &[u8], local: &str) -> Result<()> {
        let mut r = Reader::new(bytes);
        let name = r.string()?;
        let world = r.u16()? as i32;
        let rank = r.u8()? as i8 as i32;
        let index = self.members.iter().position(|m| same(&m.name, &name));
        if rank == -128 {
            if let Some(i) = index {
                if self.members[i].world == world {
                    self.members.remove(i);
                }
            }
        } else {
            r.string()?;
            let i = if let Some(i) = index {
                i
            } else {
                if self.members.len() >= 500 {
                    return Err(Error::Invalid("friends-chat capacity"));
                }
                self.members.push(Member {
                    name,
                    world,
                    rank,
                    sequence: 0,
                });
                self.members.len() - 1
            };
            let member = &mut self.members[i];
            member.world = world;
            member.rank = rank;
            member.sequence = self.sequence;
            self.sequence = self.sequence.wrapping_add(1);
            self.local_rank(i, local);
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("friends-chat delta trailing bytes"));
        }
        Ok(())
    }
}
/// Friends-chat script reads; an absent channel retains the original empty/zero defaults.
pub fn execute(
    channel: Option<&Channel>,
    op: u16,
    s: &mut crate::vm::Stacks,
    local: &str,
    social: &crate::social::Social,
) -> Result<bool> {
    use crate::vm::Object;
    match op {
        3611 | 3625 => s.push_object(Object::String(
            channel
                .map_or("", |c| if op == 3611 { &c.name } else { &c.owner })
                .into(),
        ))?,
        3612 => s.push_int(channel.map_or(0, |c| c.members.len() as i32))?,
        3616 => s.push_int(channel.map_or(0, |c| c.minimum_kick))?,
        3618 => s.push_int(channel.map_or(0, |c| c.rank))?,
        3613 | 3614 | 3615 | 3624 | 3626 | 3627 => {
            let index = s.pop_int()?;
            if channel.is_some() && index < 0 {
                return Err(Error::Invalid("negative friends-chat member index"));
            }
            let member = channel.and_then(|c| c.members.get(index as usize));
            if op == 3613 {
                s.push_object(Object::String(
                    member.map_or("", |m| m.name.as_str()).into(),
                ))?;
            } else {
                s.push_int(member.map_or(0, |m| match op {
                    3614 => m.world,
                    3615 => m.rank,
                    3624 => same(&m.name, local) as i32,
                    3626 => social.is_friend(&m.name) as i32,
                    _ => social.is_ignored(&m.name) as i32,
                }))?;
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}
/// Packet framing supplies the one-byte payload length used by these commands.
pub fn command(op: u16, name: &str, present: bool) -> Result<Option<crate::protocol::Packet>> {
    if (op == 3617 && !present) || (op == 3619 && name.is_empty()) {
        return Ok(None);
    }
    let opcode = match op {
        3617 => 101,
        3619 | 3620 => 20,
        _ => return Err(Error::Invalid("friends-chat command")),
    };
    let mut w = crate::writer::Writer::new();
    if op != 3620 {
        w.string(name)?;
    }
    Ok(Some(crate::protocol::Packet {
        opcode,
        payload: w.into_vec(),
    }))
}

impl Channel {
    pub fn sort_members(&mut self, world: i32) -> Result<()> {
        let mut members: Vec<_> = self
            .members
            .drain(..)
            .map(|m| {
                (
                    crate::username::normalize(&m.name, 12).unwrap_or_default(),
                    m,
                )
            })
            .collect();
        let chain = &self.sort;
        let result = crate::java_sort::sort(&mut members, |(an, a), (bn, b)| {
            chain.compare(
                crate::social_sort::User {
                    name: an,
                    world: a.world,
                    rank: a.rank,
                    sequence: a.sequence,
                },
                crate::social_sort::User {
                    name: bn,
                    world: b.world,
                    rank: b.rank,
                    sequence: b.sequence,
                },
                world,
                |reverse| if reverse { bn.cmp(an) } else { an.cmp(bn) },
            )
        });
        self.members = members.into_iter().map(|(_, m)| m).collect();
        result
    }
}
