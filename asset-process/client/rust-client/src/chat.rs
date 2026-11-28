//! Channel-limited message history and chat script queries (GameChatLineBuffer / ue).
use crate::{
    interface_script::{ints, string},
    vm::{Object, Stacks},
    Result,
};
use std::collections::{HashMap, VecDeque};
#[derive(Clone, Debug)]
pub struct Message {
    pub uid: i32,
    pub kind: i32,
    pub cycle: i32,
    pub name: String,
    pub prefix: String,
    pub text: String,
    pub friend: bool,
    pub ignored: bool,
}
#[derive(Default)]
pub struct Chat {
    pub public_filter: i32,
    pub private_filter: Option<i32>,
    pub trade_filter: i32,
    pub filter: String,
    channels: HashMap<i32, VecDeque<i32>>,
    messages: HashMap<i32, Message>,
    order: VecDeque<i32>,
    next: i32,
}
impl Chat {
    pub fn refresh_social(&mut self, social: &crate::social::Social) {
        for message in self.messages.values_mut() {
            let name = crate::social::strip_icon(&message.name);
            message.friend = social.is_friend(name);
            message.ignored = social.is_ignored(name);
        }
    }
    /// ue.ba(5001): invalid private modes fall back to FRIENDS (1).
    pub fn set_filters(
        &mut self,
        public: i32,
        private: i32,
        trade: i32,
    ) -> crate::protocol::Packet {
        self.public_filter = public;
        self.private_filter = Some(if (0..=2).contains(&private) {
            private
        } else {
            1
        });
        self.trade_filter = trade;
        crate::protocol::Packet {
            opcode: 59,
            payload: vec![
                public as u8,
                self.private_filter.unwrap() as u8,
                trade as u8,
            ],
        }
    }

    pub fn add(
        &mut self,
        kind: i32,
        cycle: i32,
        name: String,
        prefix: String,
        text: String,
    ) -> i32 {
        let channel = self.channels.entry(kind).or_default();
        if channel.len() == 100 {
            let old = channel.pop_back().unwrap();
            self.messages.remove(&old);
            self.order.retain(|&id| id != old);
        }
        let uid = self.next;
        self.next = self.next.wrapping_add(1);
        channel.push_front(uid);
        self.order.push_back(uid);
        self.messages.insert(
            uid,
            Message {
                uid,
                kind,
                cycle,
                name,
                prefix,
                text,
                friend: false,
                ignored: false,
            },
        );
        uid
    }
    pub fn count(&self, kind: i32) -> usize {
        self.channels.get(&kind).map_or(0, |c| c.len())
    }
    pub fn line(&self, kind: i32, line: i32) -> Option<&Message> {
        self.channels
            .get(&kind)?
            .get(line as usize)
            .and_then(|id| self.messages.get(id))
    }
    pub fn message(&self, uid: i32) -> Option<&Message> {
        self.messages.get(&uid)
    }
    pub fn neighbor(&self, uid: i32, newer: bool) -> i32 {
        let Some(index) = self.order.iter().position(|&id| id == uid) else {
            return -1;
        };
        let index = if newer {
            index.checked_add(1)
        } else {
            index.checked_sub(1)
        };
        index.and_then(|i| self.order.get(i)).copied().unwrap_or(-1)
    }
    pub fn execute(&mut self, op: u16, s: &mut Stacks) -> Result<bool> {
        match op {
            5000 => s.push_int(self.public_filter)?,
            5005 => s.push_int(self.private_filter.unwrap_or(-1))?,
            5016 => s.push_int(self.trade_filter)?,
            5017 => {
                let kind = s.pop_int()?;
                s.push_int(self.count(kind) as i32)?;
            }
            5018 | 5019 => {
                let uid = s.pop_int()?;
                s.push_int(self.neighbor(uid, op == 5018))?;
            }
            5021 => {
                self.filter = string(s)?
                    .to_lowercase()
                    .trim_matches(|c: char| c <= '\u{20}')
                    .to_owned()
            }
            5022 => s.push_object(Object::String(self.filter.clone()))?,
            5003 | 5004 | 5030 | 5031 => {
                let by_line = op == 5003 || op == 5030;
                let message = if by_line {
                    let [kind, line] = ints(s)?;
                    self.line(kind, line)
                } else {
                    self.message(s.pop_int()?)
                };
                s.push_int(message.map_or(-1, |m| if by_line { m.uid } else { m.kind }))?;
                s.push_int(message.map_or(0, |m| m.cycle))?;
                for value in [
                    message.map_or("", |m| m.name.as_str()),
                    message.map_or("", |m| m.prefix.as_str()),
                    message.map_or("", |m| m.text.as_str()),
                ] {
                    s.push_object(Object::String(value.into()))?;
                }
                s.push_int(message.map_or(0, |m| {
                    if m.friend {
                        1
                    } else if m.ignored {
                        2
                    } else {
                        0
                    }
                }))?;
                if op >= 5030 {
                    s.push_object(Object::String(String::new()))?;
                    s.push_int(0)?;
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
