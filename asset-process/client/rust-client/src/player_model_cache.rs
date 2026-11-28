//! Shared player-composition model cache (GamePlayerComposition.ap/ao).
use std::collections::{HashMap, VecDeque};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Cached(u64),
    Build,
    Unavailable,
}
pub struct Cache<T> {
    models: HashMap<u64, T>,
    order: VecDeque<u64>,
}
impl<T> Default for Cache<T> {
    fn default() -> Self {
        Self {
            models: HashMap::new(),
            order: VecDeque::new(),
        }
    }
}
impl<T> Cache<T> {
    pub fn contains(&self, key: u64) -> bool {
        self.models.contains_key(&key)
    }
    pub fn get(&self, key: u64) -> &T {
        &self.models[&key]
    }
    pub fn remove(&mut self, key: u64) {
        self.models.remove(&key);
        self.order.retain(|&v| v != key);
    }
    pub fn insert(&mut self, key: u64, value: T) {
        self.remove(key);
        if self.models.len() == 260 {
            if let Some(old) = self.order.pop_front() {
                self.models.remove(&old);
            }
        }
        self.models.insert(key, value);
        self.order.push_back(key);
    }
    fn touch(&mut self, key: u64) -> bool {
        if !self.contains(key) {
            return false;
        }
        self.order.retain(|&v| v != key);
        self.order.push_back(key);
        true
    }
    /// Cache hits do not change the actor's last-built key. The caller publishes
    /// a new last-built key only after successful assembly and insertion.
    pub fn choose(&mut self, target: u64, last_built: u64, ready: bool) -> Choice {
        if self.touch(target) {
            Choice::Cached(target)
        } else if ready {
            Choice::Build
        } else if last_built != u64::MAX && self.touch(last_built) {
            Choice::Cached(last_built)
        } else {
            Choice::Unavailable
        }
    }
}
