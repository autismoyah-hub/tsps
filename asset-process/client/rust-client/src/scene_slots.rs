//! Persistent GameScene slot occupancy and force-ground bits, separate from terrain.
#[derive(Clone, Copy, Default, Debug)]
struct Slot {
    occupied: bool,
    force_ground: bool,
    objects: u8,
}
#[derive(Clone, Debug)]
pub struct Slots {
    width: usize,
    height: usize,
    slots: Vec<Slot>,
}
impl Slots {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            slots: vec![Slot::default(); 4 * width * height],
        }
    }
    fn index(&self, plane: usize, x: usize, y: usize) -> Option<usize> {
        (plane < 4 && x < self.width && y < self.height)
            .then_some((plane * self.width + x) * self.height + y)
    }
    pub fn occupied(&self, plane: usize, x: usize, y: usize) -> bool {
        self.index(plane, x, y)
            .is_some_and(|i| self.slots[i].occupied)
    }
    pub fn render_plane(&self, plane: usize, x: usize, y: usize) -> Option<usize> {
        self.index(plane, x, y)
            .map(|i| if self.slots[i].force_ground { 0 } else { plane })
    }
    pub fn mark_force_ground(&mut self, plane: usize, x: usize, y: usize, force: bool) {
        if let Some(i) = self.index(plane, x, y) {
            self.slots[i].force_ground = force;
        }
    }
    /// gx only resets a newly created slot. Removing a location does not remove its slot.
    pub fn insert(&mut self, plane: usize, x: usize, y: usize, size: [usize; 2], below: bool) {
        if plane >= 4 || x + size[0] > self.width || y + size[1] > self.height {
            return;
        }
        for tx in x..x + size[0] {
            for ty in y..y + size[1] {
                for p in (if below { 0 } else { plane })..=plane {
                    let i = self.index(p, tx, ty).unwrap();
                    if !self.slots[i].occupied {
                        self.slots[i] = Slot {
                            occupied: true,
                            force_ground: false,
                            objects: 0,
                        };
                    }
                }
            }
        }
    }
    pub fn object_count(&self, plane: usize, x: usize, y: usize) -> u8 {
        self.index(plane, x, y).map_or(0, |i| self.slots[i].objects)
    }
    pub(crate) fn set_object_count(&mut self, plane: usize, x: usize, y: usize, count: usize) {
        if let Some(i) = self.index(plane,x,y) {
            self.slots[i].objects = u8::try_from(count).expect("scene object count");
        }
    }
    /// GameScene.nl preflights every tile before allocating or incrementing any slot.
    pub fn insert_object(
        &mut self,
        plane: usize,
        x: usize,
        y: usize,
        size: [usize; 2],
        present: bool,
    ) -> bool {
        if !present {
            return true;
        }
        if plane >= 4
            || x.checked_add(size[0]).is_none_or(|v| v > self.width)
            || y.checked_add(size[1]).is_none_or(|v| v > self.height)
        {
            return false;
        }
        for tx in x..x + size[0] {
            for ty in y..y + size[1] {
                if self.occupied(plane, tx, ty) && self.object_count(plane, tx, ty) >= 5 {
                    return false;
                }
            }
        }
        self.insert(plane, x, y, size, true);
        for tx in x..x + size[0] {
            for ty in y..y + size[1] {
                let i = self.index(plane, tx, ty).unwrap();
                self.slots[i].objects += 1;
            }
        }
        true
    }
    pub fn remove_object(&mut self, plane: usize, x: usize, y: usize, size: [usize; 2]) {
        for tx in x..x.saturating_add(size[0]).min(self.width) {
            for ty in y..y.saturating_add(size[1]).min(self.height) {
                if let Some(i) = self.index(plane, tx, ty) {
                    self.slots[i].objects = self.slots[i].objects.saturating_sub(1);
                }
            }
        }
    }
    /// at moves physical slots down, creates the bridge root if absent, and hides old plane zero.
    pub fn lower_bridge(&mut self, x: usize, y: usize) {
        let Some(root) = self.index(0, x, y) else {
            return;
        };
        let previous = self.slots[root];
        for plane in 0..3 {
            let here = self.index(plane, x, y).unwrap();
            let above = self.index(plane + 1, x, y).unwrap();
            self.slots[here] = self.slots[above];
        }
        self.insert(0, x, y, [1, 1], false);
        let top = self.index(3, x, y).unwrap();
        self.slots[top] = Slot {
            occupied: false,
            ..previous
        };
    }
}
