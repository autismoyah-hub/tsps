//! GameScene.fn/aw/nh model traversal, including locations and item piles.
//!
//! Admission is supplied per logical tile. Wall orientations constrain object
//! deferral. Boundary, decoration and ordinary models share a typed call stream;
//! Callers supply occlusion and terrain visibility; terrain drawing remains separate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Boundary {
    pub models: [Option<usize>; 2],
    pub secondary: u8,

    pub occluded: [bool; 2],
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Decoration {
    pub models: [Option<usize>; 2],
    pub mask: u16,
    pub orientation: u8,
    pub center: [i32; 2],
    pub occluded: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderCall {
    Object(usize),
    Boundary(usize),
    Decoration(usize),
    Ground(usize),
    Item(usize),
    Terrain(usize),
}
impl RenderCall {
    fn object(self) -> Option<usize> {
        if let Self::Object(id) = self {
            Some(id)
        } else {
            None
        }
    }
}
fn emit_boundary(tile: &Tile, part: usize, occlusion: Option<usize>, calls: &mut Vec<RenderCall>) {
    if let Some(boundary) = tile.boundary {
        if occlusion.is_some_and(|index| boundary.occluded[index]) {
            return;
        }
        if let Some(id) = boundary.models[part] {
            calls.push(RenderCall::Boundary(id));
        }
    }
}
fn emit_ground(tile: &Tile, calls: &mut Vec<RenderCall>) {
    if let Some(id) = tile.ground {
        calls.push(RenderCall::Ground(id));
    }
}
fn emit_items(tile: &Tile, calls: &mut Vec<RenderCall>) {
    calls.extend(tile.items.iter().flatten().copied().map(RenderCall::Item));
}
fn emit_decoration(tile: &Tile, phase: Option<(u8, [i32; 2], bool)>, calls: &mut Vec<RenderCall>) {
    let Some(decor) = tile.decoration.filter(|d| !d.occluded) else {
        return;
    };
    let mut emit = |part| {
        if let Some(id) = decor.models[part] {
            calls.push(RenderCall::Decoration(id));
        }
    };
    if let Some((mask, camera, completion)) = phase {
        if decor.mask & u16::from(mask) != 0 {
            emit(0);
        } else if decor.mask == 256 {
            let mut x = decor.center[0].wrapping_sub(camera[0]);
            let mut z = decor.center[1].wrapping_sub(camera[1]);
            if matches!(decor.orientation, 1 | 2) {
                x = x.wrapping_neg();
            }
            if matches!(decor.orientation, 2 | 3) {
                z = z.wrapping_neg();
            }
            emit(if (z < x) ^ completion { 0 } else { 1 });
        }
    } else {
        emit(0);
        if decor.mask == 256 {
            emit(1);
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tile {
    pub present: bool,
    pub force_ground: bool,
    /// Java retains pending state outside the next frame's traversal bounds.
    pub pending: u8,
    pub wall_mask: u8,
    pub wall_target: u8,
    pub admitted: bool,
    pub bridge: bool,
    pub edges: u8,
    /// Primary wall orientation; zero when no wall is present.
    pub wall: u8,
    pub boundary: Option<Boundary>,
    pub decoration: Option<Decoration>,
    /// Original hn completion mask survives owner removal and replacement.
    pub end_mask: u8,
    /// Supplied terrain callback after paint-plane and occlusion checks.
    pub terrain_call: Option<usize>,
    /// Saved bridge terrain uses plane-zero occlusion and ignores paint-plane gating.
    pub bridge_terrain_call: Option<usize>,
    pub ground: Option<usize>,
    /// Item models in original secondary, tertiary, primary invocation order.
    pub items: [Option<usize>; 3],
    pub raised_items: bool,
    /// A terrain surface is present and its tile occlusion check passed.
    pub visible_surface: bool,
    pub objects: Vec<usize>,
}
#[derive(Clone, Debug)]
pub struct Object {
    pub start: [i32; 2],
    pub end: [i32; 2],
    pub center: [i32; 2],
}
pub struct Scene {
    pub dimensions: [usize; 3],
    pub tiles: Vec<Tile>,
    pub objects: Vec<Object>,
    pub camera_tile: [i32; 2],
    pub camera_fine: [i32; 2],
    pub bounds: [[i32; 2]; 2],
    pub planes: std::ops::Range<usize>,
    pub radius: i32,
    pub offset: [i32; 2],
}
impl Scene {
    pub fn empty(dimensions: [usize; 3]) -> Self {
        Self {
            dimensions,
            tiles: vec![Tile::default(); dimensions.iter().product()],
            objects: Vec::new(),
            camera_tile: [0; 2],
            camera_fine: [0; 2],
            bounds: [[0, dimensions[1] as i32], [0, dimensions[2] as i32]],
            planes: 0..dimensions[0],
            radius: 25,
            offset: [0; 2],
        }
    }
    /// GameScene.gx resets saved data when an absent logical slot is allocated.
    pub fn allocate(&mut self, p: usize, x: usize, z: usize) {
        let i = self.index(p, x, z);
        if !self.tiles[i].present {
            self.tiles[i] = Tile {
                present: true,
                ..Tile::default()
            };
        }
    }
    /// Insert a non-null ordinary object, preflighting the complete footprint.
    /// Rejected insertions leave object IDs and all tile state unchanged.
    pub fn insert(&mut self, p: usize, object: Object) -> Option<usize> {
        if p >= self.dimensions[0]
            || object.start.iter().any(|&v| v < 0)
            || (0..2).any(|a| {
                object.end[a] < object.start[a] || object.end[a] >= self.dimensions[a + 1] as i32
            })
        {
            return None;
        }
        for x in object.start[0]..=object.end[0] {
            for z in object.start[1]..=object.end[1] {
                let t = &self.tiles[self.index(p, x as usize, z as usize)];
                if t.present && t.objects.len() >= 5 {
                    return None;
                }
            }
        }
        let id = self.objects.len();
        for x in object.start[0]..=object.end[0] {
            for z in object.start[1]..=object.end[1] {
                for below in 0..=p {
                    self.allocate(below, x as usize, z as usize);
                }
                let i = self.index(p, x as usize, z as usize);
                let mask = u8::from(x > object.start[0])
                    | (u8::from(x < object.end[0]) << 2)
                    | (u8::from(z > object.start[1]) << 3)
                    | (u8::from(z < object.end[1]) << 1);
                self.tiles[i].objects.push(id);
                self.tiles[i].edges |= mask;
            }
        }
        self.objects.push(object);
        Some(id)
    }
    /// GameScene.bn visits only the object's recorded logical plane. References
    /// in other slots across mixed bridges remain, as do all allocated tiles.
    pub fn remove(&mut self, p: usize, id: usize) {
        let Some(object) = self.objects.get(id).cloned() else {
            return;
        };
        for x in object.start[0]..=object.end[0] {
            for z in object.start[1]..=object.end[1] {
                let Some(i) = self.at(p, x, z) else {
                    continue;
                };
                if !self.tiles[i].present {
                    continue;
                }
                self.tiles[i].objects.retain(|&v| v != id);
                self.tiles[i].edges = self.tiles[i].objects.iter().fold(0, |edges, &other| {
                    let other = &self.objects[other];
                    edges
                        | u8::from(x > other.start[0])
                        | (u8::from(x < other.end[0]) << 2)
                        | (u8::from(z > other.start[1]) << 3)
                        | (u8::from(z < other.end[1]) << 1)
                });
            }
        }
    }
    pub fn insert_wall(&mut self, p: usize, x: usize, z: usize, orientation: u8) {
        for below in 0..=p {
            self.allocate(below, x, z);
        }
        let i = self.index(p, x, z);
        self.tiles[i].wall = orientation;
    }
    /// Preserve the old physical plane zero in the hidden plane-three slot.
    pub fn lower_bridge(&mut self, x: usize, z: usize) {
        assert_eq!(self.dimensions[0], 4);
        let root = self.index(0, x, z);
        let saved = self.tiles[root].clone();
        for p in 0..3 {
            let here = self.index(p, x, z);
            let above = self.index(p + 1, x, z);
            self.tiles[here] = self.tiles[above].clone();
        }
        self.allocate(0, x, z);
        self.tiles[root].bridge = true;
        let top = self.index(3, x, z);
        self.tiles[top] = saved;
        self.tiles[top].present = false;
    }
    /// Prepare the software view using native logical slots and logical heights.
    pub fn prepare(
        &mut self,
        view: &crate::terrain_submission::View,
        visibility: &mut crate::scene_visibility::Visibility,
        camera_fine: [i32; 2],
        mut height: impl FnMut(usize, usize, usize) -> i32,
    ) {
        assert_eq!(
            view.size,
            [self.dimensions[1] as i32, self.dimensions[2] as i32]
        );
        self.camera_tile = view.camera;
        self.camera_fine = camera_fine;
        if view.root {
            self.radius = view.radius;
            self.offset = [0; 2];
            self.bounds = std::array::from_fn(|a| {
                [
                    (view.camera[a] - view.radius).max(0),
                    (view.camera[a] + view.radius).min(view.size[a]),
                ]
            });
        } else {
            let middle = view.size.map(|n| n / 2);
            self.radius = middle[0].max(middle[1]);
            self.offset = std::array::from_fn(|a| view.camera[a] - middle[a]);
            self.bounds = std::array::from_fn(|a| [0, view.size[a]]);
        }
        for p in 0..self.dimensions[0] {
            for x in 0..self.dimensions[1] {
                for z in 0..self.dimensions[2] {
                    let i = self.index(p, x, z);
                    let tile = &mut self.tiles[i];
                    tile.admitted = tile.present
                        && view.slot_admitted(
                            visibility,
                            [x as i32, z as i32],
                            if tile.force_ground { 0 } else { p },
                            height(p, x, z),
                        );
                }
            }
        }
    }
    pub fn index(&self, p: usize, x: usize, z: usize) -> usize {
        (p * self.dimensions[1] + x) * self.dimensions[2] + z
    }
    fn coordinate(&self, i: usize) -> (usize, i32, i32) {
        let z = i % self.dimensions[2];
        let x = i / self.dimensions[2] % self.dimensions[1];
        (
            i / (self.dimensions[1] * self.dimensions[2]),
            x as i32,
            z as i32,
        )
    }
    fn at(&self, p: usize, x: i32, z: i32) -> Option<usize> {
        (p < self.dimensions[0]
            && x >= 0
            && z >= 0
            && (x as usize) < self.dimensions[1]
            && (z as usize) < self.dimensions[2])
            .then(|| self.index(p, x as usize, z as usize))
    }
    fn neighbors(&self, p: usize, x: i32, z: i32, far: bool) -> Vec<(usize, u8)> {
        let [cx, cz] = self.camera_tile;
        let [[xmin, xmax], [zmin, zmax]] = self.bounds;
        let candidates = if far {
            [
                (x - 1, z, 1, x <= cx && x > xmin),
                (x + 1, z, 4, x >= cx && x < xmax - 1),
                (x, z - 1, 8, z <= cz && z > zmin),
                (x, z + 1, 2, z >= cz && z < zmax - 1),
            ]
        } else {
            [
                (x + 1, z, 4, x < cx && x >= xmin && x < xmax - 1),
                (x, z + 1, 2, z < cz && z >= zmin && z < zmax - 1),
                (x - 1, z, 1, x > cx && x > xmin && x < xmax),
                (x, z - 1, 8, z > cz && z > zmin && z < zmax),
            ]
        };
        candidates
            .into_iter()
            .filter_map(|(x, z, mask, valid)| {
                valid.then(|| self.at(p, x, z).map(|i| (i, mask))).flatten()
            })
            .collect()
    }
    /// GameScene.bk/fn visits the saved bridge slot before each admitted owner.
    /// A runtime plane-three object can therefore be invoked more than once.
    pub fn first_schedule(&self) -> Vec<usize> {
        self.first_schedule_with_spots(&[])
    }
    pub fn first_schedule_with_spots(&self, spots: &[(usize, usize)]) -> Vec<usize> {
        self.first_render_calls_with_spots(spots)
            .into_iter()
            .filter_map(RenderCall::object)
            .collect()
    }
    pub fn first_render_calls_with_spots(&self, spots: &[(usize, usize)]) -> Vec<RenderCall> {
        let mut calls = Vec::new();
        let [[xmin, xmax], [zmin, zmax]] = self.bounds;
        for p in self.planes.clone() {
            for x in xmin..xmax {
                for z in zmin..zmax {
                    let Some(i) = self.at(p, x, z) else {
                        continue;
                    };
                    let tile = &self.tiles[i];
                    if !tile.present || !tile.admitted {
                        continue;
                    }
                    if tile.bridge {
                        if let Some(saved) = self.at(3, x, z) {
                            self.first_tile(saved, x, z, spots, &mut calls);
                        }
                    }
                    self.first_tile(i, x, z, spots, &mut calls);
                }
            }
        }
        calls
    }
    fn first_tile(
        &self,
        i: usize,
        x: i32,
        z: i32,
        spots: &[(usize, usize)],
        calls: &mut Vec<RenderCall>,
    ) {
        let tile = &self.tiles[i];
        calls.extend(tile.terrain_call.map(RenderCall::Terrain));
        // Original first pass tests the primary wall occlusion for both parts.
        emit_boundary(tile, 0, Some(0), calls);
        emit_boundary(tile, 1, Some(0), calls);
        emit_decoration(tile, None, calls);
        emit_ground(tile, calls);
        emit_items(tile, calls);
        calls.extend(
            tile.objects
                .iter()
                .copied()
                .filter(|&id| self.objects[id].start == [x, z])
                .map(RenderCall::Object),
        );
        if self.has_spot_tile(i, spots) {
            calls.extend(
                spots
                    .iter()
                    .map(|&(_, id)| id)
                    .filter(|&id| self.objects[id].start == [x, z])
                    .map(RenderCall::Object),
            );
        }
    }
    fn has_spot_tile(&self, tile: usize, spots: &[(usize, usize)]) -> bool {
        let (plane, x, z) = self.coordinate(tile);
        spots
            .iter()
            .any(|&(p, id)| p == plane && self.objects[id].start == [x, z])
    }
    /// Ordered object indices, retaining repeated saved-under-bridge calls.
    pub fn schedule(&mut self) -> Vec<usize> {
        self.schedule_with_spots(&[])
    }
    pub fn schedule_with_spots(&mut self, spots: &[(usize, usize)]) -> Vec<usize> {
        self.render_calls_with_spots(spots)
            .into_iter()
            .filter_map(RenderCall::object)
            .collect()
    }
    pub fn render_calls_with_spots(&mut self, spots: &[(usize, usize)]) -> Vec<RenderCall> {
        assert_eq!(self.tiles.len(), self.dimensions.iter().product());
        let mut state = Traversal {
            spots,
            scene: self,
            flags: self.tiles.iter().map(|t| t.pending).collect(),
            queue: Queue::new(self.tiles.len()),
            seen: vec![false; self.objects.len()],
            calls: Vec::new(),
            wall_masks: self.tiles.iter().map(|t| t.wall_mask).collect(),
            wall_targets: self.tiles.iter().map(|t| t.wall_target).collect(),
            end_masks: self.tiles.iter().map(|t| t.end_mask).collect(),
            remaining: 0,
        };
        let [[xmin, xmax], [zmin, zmax]] = self.bounds;
        for p in self.planes.clone() {
            for x in xmin..xmax {
                for z in zmin..zmax {
                    if let Some(i) = self.at(p, x, z) {
                        let tile = &self.tiles[i];
                        if tile.present {
                            if tile.admitted {
                                state.flags[i] = (state.flags[i]
                                    | 6
                                    | if tile.objects.is_empty() && !self.has_spot_tile(i, spots) {
                                        0
                                    } else {
                                        8
                                    })
                                    & !16;
                                state.wall_masks[i] = 0;
                                state.wall_targets[i] = 0;
                                state.remaining += 1;
                            } else {
                                state.flags[i] &= !22;
                            }
                        }
                    }
                }
            }
        }
        let [cx, cz] = self.camera_tile;
        'passes: for strict in [true, false] {
            for p in self.planes.clone() {
                for dx in -(self.offset[0].abs() + self.radius)..=0 {
                    for dz in -(self.offset[1].abs() + self.radius)..=0 {
                        for (x, z) in [
                            (cx + dx, cz + dz),
                            (cx + dx, cz - dz),
                            (cx - dx, cz + dz),
                            (cx - dx, cz - dz),
                        ] {
                            if x >= xmin && x < xmax && z >= zmin && z < zmax {
                                if let Some(i) = self.at(p, x, z) {
                                    if self.tiles[i].present && state.flags[i] & 2 != 0 {
                                        state.visit(i, strict);
                                    }
                                }
                            }
                        }
                        if state.remaining == 0 {
                            break 'passes;
                        }
                    }
                }
            }
        }
        let Traversal {
            flags,
            wall_masks,
            wall_targets,
            end_masks,
            calls,
            ..
        } = state;
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            tile.pending = flags[i];
            tile.wall_mask = wall_masks[i];
            tile.wall_target = wall_targets[i];
            tile.end_mask = end_masks[i];
        }
        calls
    }
}

// Re-enqueuing an intrusive Java tile node moves it to the tail; it does not
// leave a duplicate entry or preserve its previous position.
struct Queue {
    previous: Vec<usize>,
    next: Vec<usize>,
    linked: Vec<bool>,
    head: usize,
    tail: usize,
}
impl Queue {
    const NONE: usize = usize::MAX;
    fn new(n: usize) -> Self {
        Self {
            previous: vec![Self::NONE; n],
            next: vec![Self::NONE; n],
            linked: vec![false; n],
            head: Self::NONE,
            tail: Self::NONE,
        }
    }
    fn remove(&mut self, i: usize) {
        let (a, b) = (self.previous[i], self.next[i]);
        if a == Self::NONE {
            self.head = b;
        } else {
            self.next[a] = b;
        }
        if b == Self::NONE {
            self.tail = a;
        } else {
            self.previous[b] = a;
        }
        self.linked[i] = false;
    }
    fn push(&mut self, i: usize) {
        if self.linked[i] {
            self.remove(i);
        }
        self.previous[i] = self.tail;
        self.next[i] = Self::NONE;
        if self.tail == Self::NONE {
            self.head = i;
        } else {
            self.next[self.tail] = i;
        }
        self.tail = i;
        self.linked[i] = true;
    }
    fn pop(&mut self) -> Option<usize> {
        if self.head == Self::NONE {
            return None;
        }
        let i = self.head;
        self.remove(i);
        Some(i)
    }
}
struct Traversal<'a> {
    spots: &'a [(usize, usize)],
    remaining: i32,
    wall_masks: Vec<u8>,
    wall_targets: Vec<u8>,
    end_masks: Vec<u8>,
    scene: &'a Scene,
    flags: Vec<u8>,
    queue: Queue,
    seen: Vec<bool>,
    calls: Vec<RenderCall>,
}
impl Traversal<'_> {
    fn pending(&self, i: usize) -> bool {
        self.scene.tiles[i].present && self.flags[i] & 4 != 0
    }
    fn footprint(&self, p: usize, oid: usize) -> Vec<usize> {
        let object = &self.scene.objects[oid];
        (object.start[0]..=object.end[0])
            .flat_map(|x| {
                (object.start[1]..=object.end[1]).filter_map(move |z| self.scene.at(p, x, z))
            })
            .collect()
    }
    fn edge(&self, oid: usize, x: i32, z: i32) -> u8 {
        let object = &self.scene.objects[oid];
        u8::from(x > object.start[0])
            | (u8::from(x < object.end[0]) << 2)
            | (u8::from(z > object.start[1]) << 3)
            | (u8::from(z < object.end[1]) << 1)
    }
    fn visit(&mut self, start: usize, mut strict: bool) {
        self.queue.push(start);
        while let Some(i) = self.queue.pop() {
            if self.flags[i] & 4 == 0 {
                continue;
            }
            let (p, x, z) = self.scene.coordinate(i);
            let far = self.scene.neighbors(p, x, z, true);
            let near = self.scene.neighbors(p, x, z, false);
            let tile = &self.scene.tiles[i];
            if self.flags[i] & 2 != 0 {
                if strict {
                    if p > 0 && self.scene.at(p - 1, x, z).is_some_and(|j| self.pending(j)) {
                        continue;
                    }
                    if far.iter().any(|&(j, mask)| {
                        self.pending(j) && (self.flags[j] & 2 != 0 || tile.edges & mask == 0)
                    }) {
                        continue;
                    }
                } else {
                    strict = true;
                }
                self.flags[i] &= !2;
                if tile.bridge {
                    if let Some(saved) = self.scene.at(3, x, z) {
                        let saved_tile = &self.scene.tiles[saved];
                        self.calls
                            .extend(saved_tile.bridge_terrain_call.map(RenderCall::Terrain));
                        // Saved bridge wall primary is invoked without an occlusion test.
                        emit_boundary(saved_tile, 0, None, &mut self.calls);
                        self.calls
                            .extend(saved_tile.objects.iter().copied().map(RenderCall::Object));
                    }
                }
                self.calls
                    .extend(tile.terrain_call.map(RenderCall::Terrain));
                let [cx, cz] = self.scene.camera_tile;
                let sector = (if x == cx {
                    1
                } else if cx < x {
                    2
                } else {
                    0
                }) + (if z == cz {
                    3
                } else if cz > z {
                    6
                } else {
                    0
                });
                if tile.wall & [160, 192, 80, 96, 0, 144, 80, 48, 160][sector] != 0 {
                    let (mask, targets) = match tile.wall {
                        16 => (3, [0, 0, 2, 0, 0, 2, 1, 1, 0]),
                        32 => (6, [2, 0, 0, 2, 0, 0, 0, 4, 4]),
                        64 => (12, [0, 4, 4, 8, 0, 0, 8, 0, 0]),
                        128 => (9, [1, 1, 0, 0, 0, 8, 0, 0, 8]),
                        _ => panic!("invalid deferred wall orientation"),
                    };
                    self.wall_masks[i] = mask;
                    self.wall_targets[i] = targets[sector];
                    self.flags[i] |= 16;
                }
                if tile.wall != 0 || tile.boundary.is_some() || tile.decoration.is_some() {
                    self.end_masks[i] = [76, 8, 137, 4, 0, 1, 38, 2, 19][sector];
                    let start_mask = [19, 55, 38, 155, 255, 110, 137, 205, 76][sector];
                    if tile.wall & start_mask != 0 {
                        emit_boundary(tile, 0, Some(0), &mut self.calls);
                    }
                    if tile.boundary.is_some_and(|b| b.secondary & start_mask != 0) {
                        emit_boundary(tile, 1, Some(1), &mut self.calls);
                    }
                    emit_decoration(
                        tile,
                        Some((start_mask, self.scene.camera_fine, false)),
                        &mut self.calls,
                    );
                }
                if tile.visible_surface {
                    emit_ground(tile, &mut self.calls);
                    if !tile.raised_items {
                        emit_items(tile, &mut self.calls);
                    }
                }
                for &(j, mask) in &near {
                    if tile.edges & mask != 0 && self.pending(j) {
                        self.queue.push(j);
                    }
                }
            }
            if self.flags[i] & 16 != 0
                && !tile.objects.iter().any(|&oid| {
                    !self.seen[oid]
                        && self.edge(oid, x, z) & self.wall_masks[i] == self.wall_targets[i]
                })
            {
                emit_boundary(tile, 0, Some(0), &mut self.calls);
                self.flags[i] &= !16;
            }
            if self.flags[i] & 8 != 0 {
                self.flags[i] &= !8;
                let mut eligible = Vec::new();
                for &oid in &tile.objects {
                    if self.seen[oid] {
                        continue;
                    }
                    if self.footprint(p, oid).iter().any(|&j| {
                        let (_, x, z) = self.scene.coordinate(j);
                        self.flags[j] & 2 != 0
                            || (self.flags[j] & 16 != 0
                                && self.edge(oid, x, z) & self.wall_masks[j]
                                    == (self.wall_masks[i] ^ self.wall_targets[i]))
                    }) {
                        self.flags[i] |= 8;
                        continue;
                    }
                    let object = &self.scene.objects[oid];
                    let [cx, cz] = self.scene.camera_tile;
                    let distance = (cx - object.start[0]).max(object.end[0] - cx)
                        + (cz - object.start[1]).max(object.end[1] - cz);
                    let dx = object.center[0].wrapping_sub(self.scene.camera_fine[0]);
                    let dz = object.center[1].wrapping_sub(self.scene.camera_fine[1]);
                    let squared = dx.wrapping_mul(dx).wrapping_add(dz.wrapping_mul(dz));
                    eligible.push((oid, distance, squared));
                }
                if self.scene.has_spot_tile(i, self.spots) {
                    for &(_, oid) in self.spots {
                        if eligible.len() >= 55 {
                            break;
                        }
                        let object = &self.scene.objects[oid];
                        if self.seen[oid] || object.start != [x, z] {
                            continue;
                        }
                        let [cx, cz] = self.scene.camera_tile;
                        let distance = (cx - object.start[0]).max(object.end[0] - cx)
                            + (cz - object.start[1]).max(object.end[1] - cz);
                        let dx = object.center[0].wrapping_sub(self.scene.camera_fine[0]);
                        let dz = object.center[1].wrapping_sub(self.scene.camera_fine[1]);
                        eligible.push((
                            oid,
                            distance,
                            dx.wrapping_mul(dx).wrapping_add(dz.wrapping_mul(dz)),
                        ));
                    }
                }
                eligible.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)));
                for (oid, _, _) in eligible {
                    if self.seen[oid] {
                        continue;
                    }
                    self.seen[oid] = true;
                    self.calls.push(RenderCall::Object(oid));
                    for j in self.footprint(p, oid) {
                        if self.flags[j] & 16 != 0 || (j != i && self.flags[j] & 4 != 0) {
                            self.queue.push(j);
                        }
                    }
                }
                if self.flags[i] & 8 != 0 {
                    continue;
                }
            }
            if self.flags[i] & 16 != 0 || far.iter().any(|&(j, _)| self.pending(j)) {
                continue;
            }
            self.flags[i] &= !4;
            self.remaining -= 1;
            if tile.raised_items {
                emit_items(tile, &mut self.calls);
            }
            let end_mask = self.end_masks[i];
            if end_mask != 0 {
                emit_decoration(
                    tile,
                    Some((end_mask, self.scene.camera_fine, true)),
                    &mut self.calls,
                );
            }
            // Original completion order is secondary then primary.
            if tile.boundary.is_some_and(|b| b.secondary & end_mask != 0) {
                emit_boundary(tile, 1, Some(1), &mut self.calls);
            }
            if tile.wall & end_mask != 0 {
                emit_boundary(tile, 0, Some(0), &mut self.calls);
            }
            if p + 1 < self.scene.planes.end {
                if let Some(j) = self.scene.at(p + 1, x, z) {
                    if self.pending(j) {
                        self.queue.push(j);
                    }
                }
            }
            for (j, _) in near {
                if self.pending(j) {
                    self.queue.push(j);
                }
            }
        }
    }
}
