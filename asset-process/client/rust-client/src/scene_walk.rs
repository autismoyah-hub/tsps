//! GameScene.mg/dc/qx/dh/dr: a click arms a scene before it can emit movement.
#[derive(Clone, Copy, Debug)]
pub enum Command {
    Arm { instance: (u16, u64), walk: bool },
    Accept { instance: (u16, u64) },
}
impl Command {
    pub fn instance(self) -> (u16, u64) {
        match self {
            Self::Arm { instance, .. } | Self::Accept { instance } => instance,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Selection {
    pub pending_pick: bool,
    pub walk_requested: bool,
    pub selected: [i32; 2],
    /// Last sampled cursor in viewport coordinates (GameScene.cu/bx).
    pub point: [i32; 2],
    /// Last sampled owning-view plane (GameScene.du), independent of roof culling.
    pub plane: i32,
}
impl Default for Selection {
    fn default() -> Self {
        Self {
            pending_pick: false,
            walk_requested: false,
            selected: [-1; 2],
            point: [0; 2],
            plane: 0,
        }
    }
}
impl Selection {
    /// cm updates tn only while the pointer lies within the viewport.
    pub fn sample(&mut self, plane: i32, point: [i32; 2], size: [i32; 2]) {
        if point[0] >= 0 && point[1] >= 0 && point[0] < size[0] && point[1] < size[1] {
            self.plane = plane;
            self.point = point;
        }
    }

    pub fn ready(&self) -> bool {
        self.walk_requested && self.selected[0] != -1
    }
    pub fn arm(&mut self, walk: bool) {
        if !self.ready() || walk {
            self.pending_pick = true;
            self.walk_requested = walk;
            self.selected = [-1; 2];
        }
    }
    pub fn accept(&mut self) {
        self.walk_requested = true;
    }
    pub fn finish_draw(&mut self, selected: [i32; 2]) {
        if self.pending_pick {
            self.pending_pick = false;
            self.selected = selected;
        }
    }
    pub fn clear(&mut self) {
        self.selected[0] = -1;
        self.walk_requested = false;
    }
    pub fn apply(&mut self, command: Command) {
        match command {
            Command::Arm { walk, .. } => self.arm(walk),
            Command::Accept { .. } => self.accept(),
        }
    }
}

/// GameScene.kj limits root terrain picks only when the local player owns the root view.
pub fn terrain_target(tile: [i32; 2], root: bool, player: Option<(u16, [i32; 2])>) -> [i32; 2] {
    let Some((0, player)) = player.filter(|_| root) else {
        return tile;
    };
    let delta = [
        player[0].wrapping_sub(tile[0]),
        player[1].wrapping_sub(tile[1]),
    ];
    let excess = (delta[0] as f64).hypot(delta[1] as f64) as i32 - 70;
    if excess <= 0 {
        return tile;
    }
    std::array::from_fn(|axis| {
        tile[axis]
            .wrapping_mul(70)
            .wrapping_add(player[axis].wrapping_mul(excess))
            / (70 + excess)
    })
}
