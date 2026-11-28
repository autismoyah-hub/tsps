//! Actor path storage and fractional movement from GameActor.dp/db/tc.
#[derive(Clone, Debug)]
pub struct Path {
    pub tiles: [[i32; 2]; 10],
    pub speeds: [i8; 10],
    pub len: usize,
    pub fine: [f32; 2],
    pub position: [i32; 2],
    pub path_at_start: i32,
    pub stalled: i32,
}
impl Default for Path {
    fn default() -> Self {
        Self {
            tiles: [[0; 2]; 10],
            speeds: [1; 10],
            len: 0,
            fine: [0.; 2],
            position: [0; 2],
            path_at_start: 0,
            stalled: 0,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Movement {
    pub size: i32,
    pub clickable: bool,
    pub turning: bool,
    pub interacting: bool,
    pub turn_speed: i32,
    pub blocked: bool,
}
impl Path {
    /// GameActor.fr runs after movement; it preserves stalled movement counters.
    pub fn restore_if_outside(
        &mut self,
        size: i32,
        base: [i32; 2],
        dimensions: [i32; 2],
        local_root: bool,
    ) -> bool {
        let fine = std::array::from_fn::<_, 2, _>(|axis| {
            self.position[axis].wrapping_sub(base[axis].wrapping_mul(128))
        });
        let outside = (0..2)
            .any(|axis| fine[axis] < 128 || fine[axis] >= (dimensions[axis] - 1) << 7)
            || (local_root && fine.iter().any(|v| *v < 1536 || *v >= 11776));
        if outside {
            self.position =
                self.tiles[0].map(|v| v.wrapping_mul(128).wrapping_add(size.wrapping_mul(64)));
            self.fine = self.position.map(|v| v as f32);
            self.clear();
        }
        outside
    }
    pub fn push(&mut self, tile: [i32; 2], speed: i8) {
        self.len = (self.len + 1).min(9);
        for i in (1..=self.len).rev() {
            self.tiles[i] = self.tiles[i - 1];
            self.speeds[i] = self.speeds[i - 1];
        }
        self.tiles[0] = tile;
        self.speeds[0] = speed;
    }
    pub fn teleport(&mut self, tile: [i32; 2], size: i32) {
        self.len = 0;
        self.path_at_start = 0;
        self.stalled = 0;
        self.tiles[0] = tile;
        self.position = tile.map(|v| v.wrapping_mul(128).wrapping_add(size.wrapping_mul(64)));
        self.fine = self.position.map(|v| v as f32);
    }
    pub fn clear(&mut self) {
        self.len = 0;
        self.path_at_start = 0;
    }
    /// Discrete movement bookkeeping before interpolation (GameActor.jc).
    pub fn prepare(&mut self, size: i32, blocked: bool) {
        if self.len == 0 {
            self.stalled = 0;
            return;
        }
        if blocked {
            self.stalled = self.stalled.wrapping_add(1);
            return;
        }
        let target = self.tiles[self.len - 1]
            .map(|v| v.wrapping_mul(128).wrapping_add(size.wrapping_mul(64)));
        let distance = (target[0] as f32 - self.fine[0])
            .abs()
            .max((target[1] as f32 - self.fine[1]).abs());
        if distance > 288.0 {
            self.position = target;
            self.fine = target.map(|v| v as f32);
        } else if self.stalled > 0 && self.len > 1 {
            self.stalled -= 1;
        }
        if self.position == target {
            self.len -= 1;
            if self.path_at_start > 0 {
                self.path_at_start -= 1;
            }
        }
    }
    pub fn speed(&self, movement: Movement) -> i32 {
        let mut speed = 4;
        if movement.clickable {
            if movement.turning && !movement.interacting && movement.turn_speed != 0 {
                speed = 2;
            }
            if self.len > 2 {
                speed = 6;
            }
            if self.len > 3 {
                speed = 8;
            }
        } else {
            if self.len > 1 {
                speed = 6;
            }
            if self.len > 2 {
                speed = 8;
            }
        }
        if self.stalled > 0 && self.len > 1 {
            speed = 8;
        }
        match self.speeds[self.len - 1] {
            2 => speed <<= 1,
            0 => speed >>= 1,
            _ => {}
        }
        speed
    }
    /// Advance fractional client ticks. Sequence/exact-move gates are supplied by the actor.
    pub fn advance(&mut self, mut ticks: f32, movement: Movement) {
        if movement.blocked {
            return;
        }
        while self.len > 0 && ticks > 0. {
            let target = self.tiles[self.len - 1].map(|v| {
                v.wrapping_mul(128)
                    .wrapping_add(movement.size.wrapping_mul(64))
            });
            let speed = self.speed(movement);
            let speed = speed as f32;
            let distance = speed * ticks;
            let mut remainder = 0.0f32;
            if self.fine[0] < target[0] as f32 {
                self.fine[0] += distance;
                if self.fine[0] > target[0] as f32 {
                    remainder = (self.fine[0] - target[0] as f32) / speed;
                    self.fine[0] = target[0] as f32;
                }
            } else if self.fine[0] > target[0] as f32 {
                self.fine[0] -= distance;
                if self.fine[0] < target[0] as f32 {
                    remainder = (target[0] as f32 - self.fine[0]) / speed;
                    self.fine[0] = target[0] as f32;
                }
            }
            // Preserve the original client's signed Y remainder, including diagonal cases.
            if self.fine[1] < target[1] as f32 {
                self.fine[1] += distance;
                if self.fine[1] > target[1] as f32 {
                    remainder = remainder.max((target[1] as f32 - self.fine[1]) / speed);
                    self.fine[1] = target[1] as f32;
                }
            } else if self.fine[1] > target[1] as f32 {
                self.fine[1] -= distance;
                if self.fine[1] < target[1] as f32 {
                    remainder = remainder.max((self.fine[1] - target[1] as f32) / speed);
                    self.fine[1] = target[1] as f32;
                }
            }
            ticks = remainder;
            self.position = self.fine.map(|v| v as i32);
            if self.position == target {
                self.len -= 1;
                if self.path_at_start > 0 {
                    self.path_at_start -= 1;
                }
            }
        }
    }
}
