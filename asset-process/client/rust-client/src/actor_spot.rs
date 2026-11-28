//! Actor-attached effects: delayed start and removal at the first sequence wrap.
use crate::{
    animation::Sequence, animation_runtime::Clock, spot_animation::Definition, Error, Result,
};
use std::collections::{BTreeMap, HashMap};
#[derive(Clone, Debug)]
pub struct Effect {
    pub id: u16,
    pub height: i32,
    pub start: i32,
    pub clock: Clock,
}
#[derive(Clone, Debug, Default)]
pub struct Effects {
    insertion: Vec<u8>,
    pub slots: BTreeMap<u8, Effect>,
}
impl Effects {
    pub fn ordered_slots(&self) -> Vec<u8> {
        let mut keys = self
            .insertion
            .iter()
            .copied()
            .filter(|k| self.slots.contains_key(k))
            .collect::<Vec<_>>();
        for &key in self.slots.keys() {
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
        keys.sort_by_key(|key| key & 3);
        keys
    }
    pub fn ordered(&self) -> impl Iterator<Item = &Effect> {
        self.ordered_slots()
            .into_iter()
            .map(|key| &self.slots[&key])
    }
    pub fn clear(&mut self) {
        self.slots.clear();
        self.insertion.clear();
    }

    pub fn apply(&mut self, slot: u8, id: u16, height: u16, delay: u16, cycle: i32) {
        self.insertion
            .retain(|key| *key != slot && self.slots.contains_key(key));
        self.slots.remove(&slot);
        if id != 65535 {
            self.insertion.push(slot);
            self.slots.insert(
                slot,
                Effect {
                    id,
                    height: height as i16 as i32,
                    start: cycle.wrapping_add(delay as i32),
                    clock: Clock::default(),
                },
            );
        }
    }
    pub fn tick(
        &mut self,
        cycle: i32,
        definitions: &HashMap<u32, Definition>,
        sequences: &HashMap<u32, Sequence>,
    ) -> Result<Vec<(u32, usize)>> {
        let mut events = vec![];
        let mut remove = vec![];
        for slot in self.ordered_slots() {
            let effect = self.slots.get_mut(&slot).unwrap();
            if cycle < effect.start {
                continue;
            }
            let definition = definitions
                .get(&(effect.id as u32))
                .ok_or(Error::Invalid("actor spot definition"))?;
            if definition.sequence < 0 {
                remove.push(slot);
                continue;
            }
            let id = definition.sequence as u32;
            let sequence = sequences
                .get(&id)
                .ok_or(Error::Invalid("actor spot sequence"))?;
            let (flags, frames) = effect.clock.advance(sequence, 1)?;
            events.extend(frames.into_iter().map(|frame| (id, frame)));
            if flags & 1 != 0 {
                remove.push(slot);
            }
        }
        for slot in remove {
            self.slots.remove(&slot);
        }
        self.insertion.retain(|key| self.slots.contains_key(key));
        Ok(events)
    }
}

#[derive(Clone, Debug)]
pub struct Update {
    pub player: bool,
    pub index: u16,
    pub slot: u8,
    pub id: u16,
    pub height: u16,
    pub delay: u16,
}
impl Update {
    pub fn decode(opcode: u16, r: &mut crate::buffer::Reader<'_>) -> Result<Self> {
        use crate::buffer::Transform;
        let (player, index, slot, id, packed) = if opcode == 72 {
            let id = r.u16_add(false)?;
            let packed = r.u32()?;
            let slot = r.transformed(Transform::Sub)?;
            let index = r.u16()?;
            (true, index, slot, id, packed)
        } else {
            let slot = r.transformed(Transform::Add)?;
            let index = r.u16()?;
            let id = r.u16()?;
            let packed = r.u32_inverse_middle()?;
            (false, index, slot, id, packed)
        };
        Ok(Self {
            player,
            index,
            slot,
            id,
            height: (packed >> 16) as u16,
            delay: packed as u16,
        })
    }
}
