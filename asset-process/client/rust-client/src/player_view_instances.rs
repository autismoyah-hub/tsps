use super::{State, View};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Registry traversal order, including distinct registered instances with equal IDs.
pub fn instance_descriptors(
    ids: &[u16],
    registered_count: usize,
    root_base: [i32; 2],
    worlds: &HashMap<u16, crate::world_entity::State>,
    detached: &[crate::world_entity::State],
) -> Vec<(u16, u64, [i32; 2], [i32; 2])> {
    let mut entries = Vec::new();
    for (rank, &id) in ids.iter().enumerate() {
        let bucket = if rank < registered_count {
            (id & 15) as usize
        } else {
            16
        };
        if id == 0 {
            entries.push((bucket, 0, rank, id, root_base, [104; 2]));
            continue;
        }
        for entity in worlds
            .values()
            .chain(detached.iter())
            .filter_map(|w| w.entities.get(&id))
            .filter(|e| e.registered)
        {
            entries.push((
                bucket,
                entity.spawn_generation,
                rank,
                id,
                entity.rebuild.as_ref().map_or([0; 2], |r| r.base),
                entity.dimensions,
            ));
        }
    }
    entries.sort_by_key(|&(bucket, generation, rank, ..)| (bucket, generation, rank));
    entries
        .into_iter()
        .map(|(_, generation, _, id, base, size)| (id, generation, base, size))
        .collect()
}

impl State {
    /// Geometry and actors must come from the same selected registration.
    pub fn local_view_state(&self) -> Option<&View> {
        self.views.get(&self.local_view)
    }
    /// Unregister one original world-view identity, preserving equal-ID siblings.
    pub fn unregister_instance(&mut self, id: u16, generation: Option<u64>) {
        let canonical = self.instance_generations.get(&id).copied().unwrap_or(0);
        if generation.is_none() || generation == Some(canonical) || canonical == 0 {
            self.views.remove(&id);
            self.instance_generations.remove(&id);
            if generation.is_some() {
                if let Some(key) = self
                    .duplicate_views
                    .keys()
                    .copied()
                    .find(|&(view, _)| view == id)
                {
                    let view = self.duplicate_views.remove(&key).unwrap();
                    self.views.insert(id, view);
                    self.instance_generations.insert(id, key.1);
                }
            }
        } else if let Some(generation) = generation {
            self.duplicate_views.remove(&(id, generation));
        }
        if generation.is_none() {
            self.duplicate_views.retain(|&(view, _), _| view != id);
        }
        self.instance_order.retain(|&(view, current)| {
            view != id || generation.is_some_and(|removed| current != removed && current != 0)
        });
        self.view_order = self.instance_order.iter().map(|&(view, _)| view).collect();
        // ka.ec resolves the selected numeric ID through cc.tt each time;
        // promotion therefore also updates the selected local-player lookup.
    }
    /// Unlink registration while a represented entity still owns this view.
    /// Retired actors are deliberately excluded from logical player updates.
    pub fn unregister_referenced_instance(&mut self, id: u16, generation: Option<u64>, keep: bool) {
        if keep {
            if let Some(generation) = generation {
                let view = if self.instance_generations.get(&id) == Some(&generation) {
                    self.views.remove(&id)
                } else {
                    self.duplicate_views.remove(&(id, generation))
                };
                if let Some(view) = view {
                    self.retired_views.insert((id, generation), view);
                }
            }
        }
        self.unregister_instance(id, generation);
    }
    pub fn retain_view_references(&mut self, references: &BTreeSet<(u16, u64)>) {
        self.retired_views.retain(|key, _| references.contains(key));
    }
    /// Object-reference lookup, separate from global registered-view lookup.
    pub fn referenced_view(&self, id: u16, generation: u64) -> Option<&View> {
        self.instance_view(id, generation)
            .or_else(|| self.retired_views.get(&(id, generation)))
    }
    pub fn referenced_view_mut(&mut self, id: u16, generation: u64) -> Option<&mut View> {
        if self.retired_views.contains_key(&(id, generation)) {
            self.retired_views.get_mut(&(id, generation))
        } else {
            self.instance_view_mut(id, generation)
        }
    }
    /// Native drawing receives the root entity's view object, not cc.tt's result.
    pub fn root_owned_view(
        &self,
        id: u16,
        worlds: &HashMap<u16, crate::world_entity::State>,
    ) -> Option<&View> {
        let generation = worlds.get(&0)?.entities.get(&id)?.spawn_generation;
        self.referenced_view(id, generation)
    }
    pub fn root_owned_actor(
        &self,
        id: u16,
        index: u16,
        worlds: &HashMap<u16, crate::world_entity::State>,
    ) -> Option<&super::Player> {
        if id == 0 {
            self.root_actor(index)
        } else {
            self.root_owned_view(id, worlds)?.players.get(&index)
        }
    }
    pub fn instance_view(&self, id: u16, generation: u64) -> Option<&View> {
        if self.instance_generations.get(&id).copied().unwrap_or(0) == generation
            || self.instance_generations.get(&id).copied().unwrap_or(0) == 0
        {
            self.views.get(&id)
        } else {
            self.duplicate_views.get(&(id, generation))
        }
    }
    pub fn instance_view_mut(&mut self, id: u16, generation: u64) -> Option<&mut View> {
        if self.instance_generations.get(&id).copied().unwrap_or(0) == generation
            || self.instance_generations.get(&id).copied().unwrap_or(0) == 0
        {
            self.views.get_mut(&id)
        } else {
            self.duplicate_views.get_mut(&(id, generation))
        }
    }
    pub fn instance_actor(&self, id: u16, generation: u64, index: u16) -> Option<&super::Player> {
        if id == 0 {
            self.root_actor(index)
        } else {
            self.instance_view(id, generation)?.players.get(&index)
        }
    }
    pub fn instances(&self) -> impl Iterator<Item = ((u16, u64), &View)> {
        self.views
            .iter()
            .map(|(&id, view)| {
                (
                    (id, self.instance_generations.get(&id).copied().unwrap_or(0)),
                    view,
                )
            })
            .chain(self.duplicate_views.iter().map(|(&key, view)| (key, view)))
    }
    pub(super) fn reconcile_instances(&mut self, descriptors: &[(u16, u64, [i32; 2], [i32; 2])]) {
        let mut old = std::mem::take(&mut self.duplicate_views);
        for (id, view) in std::mem::take(&mut self.views) {
            old.insert(
                (id, self.instance_generations.get(&id).copied().unwrap_or(0)),
                view,
            );
        }
        self.instance_generations.clear();
        self.instance_order.clear();
        self.view_order.clear();
        let mut seen = BTreeSet::new();
        for &(id, generation, base, dimensions) in descriptors {
            if !seen.insert((id, generation)) {
                continue;
            }
            // Generation zero is the legacy caller's unqualified view descriptor.
            let mut view = old
                .remove(&(id, generation))
                .or_else(|| old.remove(&(id, 0)))
                .unwrap_or_else(|| View {
                    base,
                    dimensions,
                    players: BTreeMap::new(),
                });
            view.base = base;
            view.dimensions = dimensions;
            self.instance_order.push((id, generation));
            self.view_order.push(id);
            if let std::collections::btree_map::Entry::Vacant(entry) =
                self.instance_generations.entry(id)
            {
                entry.insert(generation);
                self.views.insert(id, view);
            } else {
                self.duplicate_views.insert((id, generation), view);
            }
        }
    }
    pub(super) fn select_local_instance(&mut self) {
        let selected = if self.instance_order.is_empty() {
            self.view_order
                .iter()
                .rev()
                .copied()
                .filter(|&id| id != 0)
                .find(|id| {
                    self.views
                        .get(id)
                        .is_some_and(|v| v.players.contains_key(&self.local))
                })
                .map(|id| (id, 0))
        } else {
            self.instance_order
                .iter()
                .rev()
                .copied()
                .filter(|&(id, _)| id != 0)
                .find(|&(id, generation)| {
                    self.instance_view(id, generation)
                        .is_some_and(|v| v.players.contains_key(&self.local))
                })
        };
        self.local_view = selected.map_or(0, |(id, _)| id);
    }
}
