//! Stable iteration of cc's sixteen-bucket world-view registry.
#[derive(Clone, Debug)]
pub struct Registry {
    inserted: Vec<u16>,
}
impl Default for Registry {
    fn default() -> Self {
        Self { inserted: vec![0] }
    }
}
impl Registry {
    pub fn insert(&mut self, id: u16) {
        if !self.inserted.contains(&id) {
            self.inserted.push(id);
        }
    }
    pub fn remove(&mut self, id: u16) {
        self.inserted.retain(|&v| v != id);
    }
    pub fn ordered(&self) -> Vec<u16> {
        let mut ids = self.inserted.clone();
        ids.sort_by_key(|id| id & 15);
        ids
    }
    /// cc.ap selects the first containing child in registry order, then falls back to root.
    pub fn containing(
        &self,
        point: [i32; 2],
        mut bounds: impl FnMut(u16) -> Option<([i32; 2], [i32; 2])>,
    ) -> u16 {
        self.ordered()
            .into_iter()
            .find(|&id| {
                if id == 0 {
                    return false;
                }
                let Some((base, size)) = bounds(id) else {
                    return false;
                };
                point[0] >= base[0]
                    && point[1] >= base[1]
                    && point[0] < base[0].wrapping_add(size[0])
                    && point[1] < base[1].wrapping_add(size[1])
            })
            .unwrap_or(0)
    }
}
