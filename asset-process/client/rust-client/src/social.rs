//! Friends/ignores, rename reconciliation and friend ordering (bz, GameFriendContainer, qf).
use crate::{
    buffer::Reader,
    interface_script::string,
    username,
    vm::{Object, Stacks},
    Error, Result,
};
/// GameIndexDataBase.hn recognizes only the icons present in revision 239.1's kl enum.
pub fn strip_icon(name: &str) -> &str {
    for id in [0, 1, 2, 3, 10, 22, 41, 42, 43, 44, 45, 46, 47, 48, 49, 52] {
        if let Some(rest) = name.strip_prefix(&format!("<img={id}>")) {
            return rest;
        }
    }
    name
}
#[derive(Clone, Debug)]
pub struct Name {
    pub current: String,
    pub previous: String,
    clean: String,
    old: Option<String>,
    order: i32,
}
impl Name {
    fn new(current: String, previous: String) -> Result<Self> {
        let clean =
            username::normalize(&current, 12).ok_or(Error::Invalid("invalid social username"))?;
        let old = username::normalize(&previous, 12);
        Ok(Self {
            current,
            previous,
            clean,
            old,
            order: 0,
        })
    }
}
#[derive(Clone, Debug)]
pub struct Friend {
    pub name: Name,
    pub world: i32,
    pub rank: u8,
    pub flags: u8,
    pub sequence: i32,
}
#[derive(Clone, Debug)]
pub struct FriendUpdate {
    pub rename: bool,
    pub current: String,
    pub previous: String,
    pub world: u16,
    pub rank: u8,
    pub flags: u8,
}
#[derive(Clone, Debug)]
pub enum IgnoreUpdate {
    Remove(String),
    Upsert {
        rename: bool,
        current: String,
        previous: String,
    },
}
pub fn decode_friends(bytes: &[u8]) -> Result<Vec<FriendUpdate>> {
    let mut r = Reader::new(bytes);
    let mut updates = Vec::new();
    while r.remaining() > 0 {
        let rename = r.u8()? == 1;
        let current = r.string()?;
        let previous = r.string()?;
        let world = r.u16()?;
        let rank = r.u8()?;
        let flags = r.u8()?;
        if world > 0 {
            r.string()?;
            r.u8()?;
            r.u32()?;
        }
        r.string()?;
        updates.push(FriendUpdate {
            rename,
            current,
            previous,
            world,
            rank,
            flags,
        });
    }
    Ok(updates)
}
pub fn decode_ignores(bytes: &[u8]) -> Result<Vec<IgnoreUpdate>> {
    let mut r = Reader::new(bytes);
    let mut updates = Vec::new();
    while r.remaining() > 0 {
        let flags = r.u8()?;
        let current = r.string()?;
        if flags == 4 {
            updates.push(IgnoreUpdate::Remove(current));
        } else {
            let previous = r.string()?;
            r.string()?;
            updates.push(IgnoreUpdate::Upsert {
                rename: flags & 1 != 0,
                current,
                previous,
            });
        }
    }
    Ok(updates)
}
#[derive(Clone, Debug)]
pub struct LoginChange {
    pub name: String,
    pub world: u16,
    pub at: i64,
}
#[derive(Clone)]
pub struct Social {
    pub state: u8,
    pub friends: Vec<Friend>,
    pub ignores: Vec<Name>,
    pub login_changes: Vec<LoginChange>,
    next: i32,
    pub friend_sort: crate::social_sort::Chain,
    pub ignore_sort: crate::social_sort::Chain,
}
impl Default for Social {
    fn default() -> Self {
        Self {
            state: 0,
            friends: Vec::new(),
            ignores: Vec::new(),
            login_changes: Vec::new(),
            next: 1,
            friend_sort: Default::default(),
            ignore_sort: Default::default(),
        }
    }
}
fn reconcile<T>(
    list: &mut Vec<T>,
    name: &Name,
    rename: bool,
    get: impl Fn(&T) -> &Name,
) -> Option<usize> {
    let mut current = list.iter().position(|v| get(v).clean == name.clean);
    if rename {
        if let Some(old) = name
            .old
            .as_ref()
            .and_then(|old| list.iter().position(|v| &get(v).clean == old))
        {
            if Some(old) != current {
                if current.is_some() {
                    list.remove(old);
                    current = list.iter().position(|v| get(v).clean == name.clean);
                } else {
                    current = Some(old);
                }
            }
        }
    }
    current
}
fn clear_previous<T>(list: &mut [T], index: usize, get: impl Fn(&mut T) -> &mut Name) {
    let old = get(&mut list[index]).old.clone();
    if old.is_some() {
        for (i, value) in list.iter_mut().enumerate() {
            let name = get(value);
            if i != index && name.old == old {
                name.old = None;
                name.previous.clear();
            }
        }
    }
}
impl Social {
    pub fn apply_friends(
        &mut self,
        updates: Vec<FriendUpdate>,
        world: i32,
        now: i64,
    ) -> Result<()> {
        let mut next = self.clone();
        for u in updates {
            let name = Name::new(u.current, u.previous)?;
            let index = reconcile(&mut next.friends, &name, u.rename, |f| &f.name);
            let existing = index.is_some();
            let index = match index {
                Some(i) => i,
                None => {
                    if next.friends.len() >= 400 {
                        continue;
                    }
                    next.friends.push(Friend {
                        name: name.clone(),
                        world: -1,
                        rank: 0,
                        flags: 0,
                        sequence: 0,
                    });
                    next.friends.len() - 1
                }
            };
            let f = &mut next.friends[index];
            if f.world != u.world as i32 {
                if existing {
                    let mut cancelled = false;
                    next.login_changes.retain(|change| {
                        let opposite = username::normalize(&change.name, 12).as_ref()
                            == Some(&name.clean)
                            && ((u.world != 0 && change.world == 0)
                                || (u.world == 0 && change.world != 0));
                        cancelled |= opposite;
                        !opposite
                    });
                    if !cancelled {
                        next.login_changes.push(LoginChange {
                            name: name.current.clone(),
                            world: u.world,
                            // The injected GamePendingLogin.vy hook adds five seconds.
                            at: (now as i32).wrapping_add(5) as i64,
                        });
                    }
                }
                f.sequence = next.next;
                next.next = next.next.wrapping_add(1);
                if f.world == -1 && u.world == 0 {
                    f.sequence = f.sequence.wrapping_neg();
                }
                f.world = u.world as i32;
            }
            f.name = name;
            f.rank = u.rank;
            f.flags = u.flags;
            clear_previous(&mut next.friends, index, |f| &mut f.name);
        }
        next.sort_friends(world)?;
        next.state = 2;
        *self = next;
        Ok(())
    }
    pub fn apply_ignores(&mut self, updates: Vec<IgnoreUpdate>) -> Result<()> {
        let mut next = self.clone();
        for update in updates {
            match update {
                IgnoreUpdate::Remove(name) => {
                    let clean = username::normalize(&name, 12)
                        .ok_or(Error::Invalid("invalid ignored username"))?;
                    next.ignores.retain(|n| n.clean != clean);
                }
                IgnoreUpdate::Upsert {
                    rename,
                    current,
                    previous,
                } => {
                    let name = Name::new(current, previous)?;
                    let index = reconcile(&mut next.ignores, &name, rename, |n| n);
                    let index = match index {
                        Some(i) => {
                            let mut name = name;
                            name.order = next.ignores[i].order;
                            next.ignores[i] = name;
                            i
                        }
                        None => {
                            if next.ignores.len() >= 400 {
                                continue;
                            }
                            let mut name = name;
                            name.order = next.ignores.len() as i32;
                            next.ignores.push(name);
                            next.ignores.len() - 1
                        }
                    };
                    clear_previous(&mut next.ignores, index, |n| n);
                }
            }
        }
        *self = next;
        Ok(())
    }
    pub fn drain_notifications(&mut self, now: i64) -> Vec<String> {
        let mut messages = Vec::new();
        self.login_changes.retain(|change| {
            if change.at < now - 5 {
                // The Java pending world field is a signed short.
                let world = change.world as i16;
                if world >= 0 {
                    messages.push(format!(
                        "{} has logged {}.",
                        change.name,
                        if world > 0 { "in" } else { "out" }
                    ));
                }
                false
            } else {
                true
            }
        });
        messages
    }
    pub fn is_friend(&self, name: &str) -> bool {
        let Some(clean) = username::normalize(name, 12) else {
            return false;
        };
        self.friends
            .iter()
            .any(|f| f.name.clean == clean || f.name.old.as_ref() == Some(&clean))
    }
    pub fn is_ignored(&self, name: &str) -> bool {
        let Some(clean) = username::normalize(name, 12) else {
            return false;
        };
        self.ignores
            .iter()
            .any(|n| n.clean == clean || n.old.as_ref() == Some(&clean))
    }
    pub fn execute(&self, op: u16, s: &mut Stacks, local_name: &str) -> Result<bool> {
        match op {
            3600 => s.push_int(match self.state {
                0 => -2,
                1 => -1,
                _ => self.friends.len() as i32,
            })?,
            3601 | 3622 => {
                let index = s.pop_int()?;
                let name = if self.state == 2 {
                    if op == 3601 {
                        self.friends.get(index as usize).map(|f| &f.name)
                    } else {
                        self.ignores.get(index as usize)
                    }
                } else {
                    None
                };
                s.push_object(Object::String(
                    name.map_or("", |n| n.current.as_str()).into(),
                ))?;
                s.push_object(Object::String(
                    name.map_or("", |n| n.previous.as_str()).into(),
                ))?;
            }
            3602 | 3603 => {
                let index = s.pop_int()?;
                let friend = if self.state == 2 {
                    self.friends.get(index as usize)
                } else {
                    None
                };
                s.push_int(friend.map_or(0, |f| if op == 3602 { f.world } else { f.rank as i32 }))?;
            }
            3621 => s.push_int(if self.state == 2 {
                self.ignores.len() as i32
            } else {
                -1
            })?,
            3609 | 3623 => {
                let name = string(s)?;
                let name = strip_icon(&name);
                let value = if op == 3623 {
                    self.is_ignored(&name)
                } else {
                    self.is_friend(&name)
                        || username::normalize(&name, 12)
                            .is_some_and(|n| Some(n) == username::normalize(local_name, 12))
                };
                s.push_int(i32::from(value))?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
#[derive(Default)]
pub struct Command {
    pub packet: Option<crate::protocol::Packet>,
    pub message: Option<String>,
    pub changed: bool,
}
impl Social {
    /// Original bz add/remove decisions, including current-name-only removal.
    pub fn command(
        &mut self,
        op: u16,
        name: &str,
        rank: i32,
        local: &str,
        member: bool,
    ) -> Result<Command> {
        let mut result = Command::default();
        if op == 3604 {
            let mut w = crate::writer::Writer::new();
            w.transformed(rank as u8, crate::buffer::Transform::Sub);
            w.string(name)?;
            result.packet = Some(crate::protocol::Packet {
                opcode: 55,
                payload: w.into_vec(),
            });
            return Ok(result);
        }
        let Some(clean) = username::normalize(name, 12) else {
            return Ok(result);
        };
        let self_name = Some(&clean) == username::normalize(local, 12).as_ref();
        let opcode = match op {
            3605 | 3607 => {
                let ignore = op == 3607;
                let full = if ignore {
                    self.ignores.len() >= 400 || (!member && self.ignores.len() >= 100)
                } else {
                    self.friends.len() >= 400 || (!member && self.friends.len() >= 200)
                };
                result.message = if full {
                    Some(if ignore {"Your ignore list is full. Max of 100 for free users, and 400 for members"}else{"Your friend list is full. Max of 200 for free users, and 400 for members"}.into())
                } else if self_name {
                    Some(
                        if ignore {
                            "You can't add yourself to your own ignore list"
                        } else {
                            "You can't add yourself to your own friend list"
                        }
                        .into(),
                    )
                } else if if ignore {
                    self.is_ignored(name)
                } else {
                    self.is_friend(name)
                } {
                    Some(format!(
                        "{name} is already on your {} list",
                        if ignore { "ignore" } else { "friend" }
                    ))
                } else if if ignore {
                    self.is_friend(name)
                } else {
                    self.is_ignored(name)
                } {
                    Some(format!(
                        "Please remove {name} from your {} list first",
                        if ignore { "friend" } else { "ignore" }
                    ))
                } else {
                    None
                };
                if result.message.is_some() {
                    return Ok(result);
                }
                if ignore {
                    82
                } else {
                    28
                }
            }
            3606 => {
                let Some(index) = self.friends.iter().position(|f| f.name.clean == clean) else {
                    return Ok(result);
                };
                self.friends.remove(index);
                result.changed = true;
                65
            }
            3608 => {
                let Some(index) = self.ignores.iter().position(|n| n.clean == clean) else {
                    return Ok(result);
                };
                self.ignores.remove(index);
                result.changed = true;
                9
            }
            _ => return Err(Error::Invalid("social command opcode")),
        };
        let mut w = crate::writer::Writer::new();
        w.string(name)?;
        result.packet = Some(crate::protocol::Packet {
            opcode,
            payload: w.into_vec(),
        });
        Ok(result)
    }
}

impl Social {
    pub fn sort_friends(&mut self, world: i32) -> Result<()> {
        let chain = &self.friend_sort;
        crate::java_sort::sort(&mut self.friends, |a, b| {
            chain.compare(
                crate::social_sort::User {
                    name: &a.name.clean,
                    world: a.world,
                    rank: a.rank as i32,
                    sequence: a.sequence,
                },
                crate::social_sort::User {
                    name: &b.name.clean,
                    world: b.world,
                    rank: b.rank as i32,
                    sequence: b.sequence,
                },
                world,
                |reverse| {
                    let (a, b) = if reverse { (b, a) } else { (a, b) };
                    (b.world == world)
                        .cmp(&(a.world == world))
                        .then_with(|| (b.world != 0).cmp(&(a.world != 0)))
                        .then_with(|| (b.flags & 2 != 0).cmp(&(a.flags & 2 != 0)))
                        .then_with(|| (b.flags & 1 != 0).cmp(&(a.flags & 1 != 0)))
                        .then_with(|| {
                            if a.world != 0 {
                                a.sequence.wrapping_sub(b.sequence).cmp(&0)
                            } else {
                                b.sequence.wrapping_sub(a.sequence).cmp(&0)
                            }
                        })
                },
            )
        })
    }
    pub fn sort_ignores(&mut self) -> Result<()> {
        let chain = &self.ignore_sort;
        crate::java_sort::sort(&mut self.ignores, |a, b| {
            chain.compare(
                crate::social_sort::User {
                    name: &a.clean,
                    world: 0,
                    rank: 0,
                    sequence: 0,
                },
                crate::social_sort::User {
                    name: &b.clean,
                    world: 0,
                    rank: 0,
                    sequence: 0,
                },
                0,
                |reverse| {
                    if reverse {
                        b.order.wrapping_sub(a.order).cmp(&0)
                    } else {
                        a.order.wrapping_sub(b.order).cmp(&0)
                    }
                },
            )
        })
    }
}
impl Social {
    /// GamePlayer.ef checks current names before aliases and requires world > 0.
    pub fn is_friend_online(&self, name: &str) -> bool {
        let Some(clean) = username::normalize(name, 12) else {
            return false;
        };
        self.friends
            .iter()
            .find(|f| f.name.clean == clean)
            .or_else(|| {
                self.friends
                    .iter()
                    .find(|f| f.name.old.as_ref() == Some(&clean))
            })
            .is_some_and(|f| f.world > 0)
    }
}
