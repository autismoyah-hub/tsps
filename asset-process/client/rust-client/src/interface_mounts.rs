//! Interface mount table with the original GameHashTable(8) traversal order.
use crate::interface_packet::SubInterface;
use std::collections::HashMap;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Mounts {
    entries: HashMap<u32, SubInterface>,
    buckets: [Vec<u32>; 8],
}
impl Mounts {
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn get(&self, key: &u32) -> Option<&SubInterface> {
        self.entries.get(key)
    }
    pub fn contains_key(&self, key: &u32) -> bool {
        self.entries.contains_key(key)
    }
    pub fn insert(&mut self, key: u32, value: SubInterface) -> Option<SubInterface> {
        let old = self.remove(&key);
        self.buckets[(key & 7) as usize].push(key);
        self.entries.insert(key, value);
        old
    }
    pub fn remove(&mut self, key: &u32) -> Option<SubInterface> {
        let old = self.entries.remove(key)?;
        self.buckets[(key & 7) as usize].retain(|id| id != key);
        Some(old)
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        for bucket in &mut self.buckets {
            bucket.clear();
        }
    }
    pub fn keys(&self) -> impl Iterator<Item = &u32> {
        self.buckets.iter().flatten()
    }
    pub fn values(&self) -> impl Iterator<Item = &SubInterface> {
        self.keys().map(|id| &self.entries[id])
    }
    pub fn iter(&self) -> impl Iterator<Item = (&u32, &SubInterface)> {
        self.keys().map(|id| (id, &self.entries[id]))
    }
}
impl std::ops::Index<&u32> for Mounts {
    type Output = SubInterface;
    fn index(&self, key: &u32) -> &Self::Output {
        &self.entries[key]
    }
}
