//! Scene resources follow world-view object lifetimes, not numeric view IDs.
use std::collections::BTreeMap;
pub struct Store<T> {
    entries: BTreeMap<(u16, u64), T>,
    selected: BTreeMap<u16, u64>,
}
impl<T> Default for Store<T> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            selected: BTreeMap::new(),
        }
    }
}
impl<T> Store<T> {
    pub fn clear(&mut self) {
        self.entries.clear();
        self.selected.clear();
    }
    pub fn insert(&mut self, key: (u16, u64), value: T) {
        self.entries.insert(key, value);
    }
    /// Publish a complete resource set only when every dependency is ready.
    /// Java GameClient.rw leaves the existing view intact while maps/models load.
    pub fn try_build(&mut self, key: (u16, u64), build: impl FnOnce(Option<&T>) -> crate::Result<Option<T>>) -> crate::Result<bool> {
        match build(self.entries.get(&key)) {
            Ok(Some(value)) => { self.entries.insert(key, value); Ok(true) }
            Ok(None) => Ok(false),
            Err(crate::Error::PendingAsset { .. }) => Ok(false),
            Err(error) => Err(error),
        }
    }
    pub fn get_instance(&self, id: u16, generation: u64) -> Option<&T> {
        self.entries.get(&(id, generation))
    }
    pub fn get_instance_mut(&mut self, id: u16, generation: u64) -> Option<&mut T> {
        self.entries.get_mut(&(id, generation))
    }
    /// cc.tt exposes the oldest remaining registration for a numeric ID.
    pub fn select_registered(&mut self, registrations: impl IntoIterator<Item = (u16, u64)>) {
        self.selected.clear();
        for (id, generation) in registrations {
            if self.entries.contains_key(&(id, generation)) {
                self.selected
                    .entry(id)
                    .and_modify(|old| *old = (*old).min(generation))
                    .or_insert(generation);
            }
        }
    }
    pub fn get(&self, id: &u16) -> Option<&T> {
        self.entries.get(&(*id, *self.selected.get(id)?))
    }
    pub fn iter(&self) -> impl Iterator<Item = (&u16, &T)> {
        self.selected
            .iter()
            .filter_map(|(id, g)| self.entries.get(&(*id, *g)).map(|v| (id, v)))
    }
    pub fn retain(&mut self, mut keep: impl FnMut(&(u16, u64), &mut T) -> bool) {
        self.entries.retain(|key, value| keep(key, value));
        self.selected
            .retain(|id, g| self.entries.contains_key(&(*id, *g)));
    }
}
