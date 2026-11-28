//! Outgoing packet construction order, including abandoned ISAAC reservations.
use crate::protocol::Packet;
use std::{
    collections::VecDeque,
    ops::{Index, RangeFull},
};

#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    Packet(Packet),
    /// Original xt.ag writes the encrypted opcode before payload serialization.
    DiscardCipher,
}
fn packet(event: Event) -> Option<Packet> {
    match event {
        Event::Packet(packet) => Some(packet),
        Event::DiscardCipher => None,
    }
}
/// Packet-only accessors support offline inspection and discard reservation markers.
/// Network consumers must use `pop_event` to preserve the full construction stream.
#[derive(Default, Debug)]
pub struct Queue {
    events: VecDeque<Event>,
}
impl Queue {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push_back(&mut self, packet: Packet) {
        self.events.push_back(Event::Packet(packet));
    }
    pub fn discard_cipher(&mut self) {
        self.events.push_back(Event::DiscardCipher);
    }
    /// Connection consumers must retain reservations as well as completed packets.
    pub fn pop_event(&mut self) -> Option<Event> {
        self.events.pop_front()
    }
    pub fn clear(&mut self) {
        self.events.clear();
    }
    /// Packet inspection helpers exclude abandoned constructions.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Packet> {
        self.events.iter().filter_map(|event| match event {
            Event::Packet(packet) => Some(packet),
            Event::DiscardCipher => None,
        })
    }
    pub fn len(&self) -> usize {
        self.iter().count()
    }
    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }
    pub fn pop_front(&mut self) -> Option<Packet> {
        while let Some(event) = self.events.pop_front() {
            if let Some(packet) = packet(event) {
                return Some(packet);
            }
        }
        None
    }
    pub fn pop_back(&mut self) -> Option<Packet> {
        while let Some(event) = self.events.pop_back() {
            if let Some(packet) = packet(event) {
                return Some(packet);
            }
        }
        None
    }
    pub fn drain(&mut self, _: RangeFull) -> impl DoubleEndedIterator<Item = Packet> + '_ {
        self.events.drain(..).filter_map(packet)
    }
}
impl Index<usize> for Queue {
    type Output = Packet;
    fn index(&self, index: usize) -> &Packet {
        self.iter().nth(index).expect("outbound packet index")
    }
}
impl IntoIterator for Queue {
    type Item = Packet;
    type IntoIter = std::iter::FilterMap<
        std::collections::vec_deque::IntoIter<Event>,
        fn(Event) -> Option<Packet>,
    >;
    fn into_iter(self) -> Self::IntoIter {
        self.events.into_iter().filter_map(packet)
    }
}
/// Encode one construction in queue order; abandoned nodes emit no bytes.
pub fn encode(
    event: Event,
    cipher: &mut Option<crate::crypto::Isaac>,
) -> crate::Result<Option<Vec<u8>>> {
    match event {
        Event::Packet(packet) => {
            crate::protocol::encode_client(packet.opcode, &packet.payload, cipher).map(Some)
        }
        Event::DiscardCipher => {
            if let Some(cipher) = cipher {
                cipher.next_word();
            }
            Ok(None)
        }
    }
}
