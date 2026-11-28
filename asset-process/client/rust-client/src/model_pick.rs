//! Original GameModel.av, jf.ac and ic.ar selection geometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub center: [i32; 3],
    pub half: [i32; 3],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    pub min: [i32; 3],
    pub max: [i32; 3],
    pub single_tile: bool,
}
impl Shape {
    pub fn new(vertices: &[[f32; 3]], angle: u16, single_tile: bool) -> Self {
        let radians = (angle & 2047) as f64 * 0.0030679615;
        let sin = (radians.sin() * 65536.0) as i32;
        let cos = (radians.cos() * 65536.0) as i32;
        let mut min = [0i32; 3];
        let mut max = [0i32; 3];
        for v in vertices {
            let [x, y, z] = v.map(|v| v as i32);
            let rotated = [
                x.wrapping_mul(cos).wrapping_add(z.wrapping_mul(sin)) >> 16,
                y,
                z.wrapping_mul(cos).wrapping_sub(x.wrapping_mul(sin)) >> 16,
            ];
            for i in 0..3 {
                min[i] = min[i].min(rotated[i]);
                max[i] = max[i].max(rotated[i]);
            }
        }
        Self {
            min,
            max,
            single_tile,
        }
    }
    pub fn union(self, other: Self) -> Self {
        Self {
            min: std::array::from_fn(|i| self.min[i].min(other.min[i])),
            max: std::array::from_fn(|i| self.max[i].max(other.max[i])),
            single_tile: self.single_tile,
        }
    }
    pub fn bounds(self) -> Bounds {
        let center = std::array::from_fn(|i| self.max[i].wrapping_add(self.min[i]) / 2);
        let mut half =
            std::array::from_fn(|i| self.max[i].wrapping_sub(self.min[i]).wrapping_add(1) / 2);
        for i in [0, 2] {
            half[i] = half[i].max(32) + if self.single_tile { 8 } else { 0 };
        }
        Bounds { center, half }
    }
}
impl Bounds {
    pub fn new(vertices: &[[f32; 3]], angle: u16, single_tile: bool) -> Self {
        Shape::new(vertices, angle, single_tile).bounds()
    }
    /// Integer separating-axis test, including Java's wrapping arithmetic.
    pub fn intersects(self, position: [i32; 3], endpoints: [[i32; 3]; 2]) -> bool {
        let [near, far] = endpoints;
        let delta: [i32; 3] = std::array::from_fn(|i| {
            (near[i].wrapping_add(far[i]) / 2)
                .wrapping_sub(position[i].wrapping_add(self.center[i]))
        });
        let ray: [i32; 3] = std::array::from_fn(|i| far[i].wrapping_sub(near[i]) / 2);
        let abs = ray.map(i32::wrapping_abs);
        for i in 0..3 {
            if delta[i].wrapping_abs() > self.half[i].wrapping_add(abs[i]) {
                return false;
            }
        }
        for (a, b) in [(1, 2), (2, 0), (0, 1)] {
            let cross = ray[a]
                .wrapping_mul(delta[b])
                .wrapping_sub(ray[b].wrapping_mul(delta[a]));
            let limit = self.half[a]
                .wrapping_mul(abs[b])
                .wrapping_add(self.half[b].wrapping_mul(abs[a]));
            if cross.wrapping_abs() > limit {
                return false;
            }
        }
        true
    }
    /// Distance to the ray's first slab intersection; -1 when it misses.
    pub fn depth(self, position: [i32; 3], endpoints: [[i32; 3]; 2]) -> i32 {
        let [near, far] = endpoints;
        let ray: [f32; 3] = std::array::from_fn(|i| far[i].wrapping_sub(near[i]) as f32);
        let (mut enter, mut leave) = (0.0f32, 1.0f32);
        for i in 0..3 {
            let center = position[i].wrapping_add(self.center[i]);
            let min = center.wrapping_sub(self.half[i]);
            let max = center.wrapping_add(self.half[i]);
            if ray[i].abs() < 1.0e-9 {
                if near[i] < min || near[i] > max {
                    return -1;
                }
            } else {
                let inv = 1.0 / ray[i];
                let mut a = min.wrapping_sub(near[i]) as f32 * inv;
                let mut b = max.wrapping_sub(near[i]) as f32 * inv;
                if a > b {
                    std::mem::swap(&mut a, &mut b);
                }
                enter = enter.max(a);
                leave = leave.min(b);
                if enter > leave {
                    return -1;
                }
            }
        }
        let [x, y, z] = std::array::from_fn::<_, 3, _>(|i| ray[i] * enter + near[i] as f32);
        ((z * z + (x * x + y * y)) as f64).sqrt() as i32
    }
}

/// GameMenu.al coalesces consecutive tags before dl.au's stable depth order.
pub struct Hits<T = u64>(Vec<(T, i32)>);
impl<T> Default for Hits<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
impl<T: Copy + Eq + std::hash::Hash> Hits<T> {
    pub fn push(&mut self, tag: T, depth: i32) {
        if let Some(last) = self.0.last_mut().filter(|last| last.0 == tag) {
            last.1 = last.1.min(depth);
        } else {
            self.0.push((tag, depth));
        }
    }
    pub fn sorted(mut self) -> Vec<(T, i32)> {
        self.0.sort_by(|a, b| b.1.cmp(&a.1));
        let mut seen = std::collections::HashSet::new();
        self.0.reverse();
        self.0.retain(|&(tag, _)| seen.insert(tag));
        self.0.reverse();
        self.0
    }
}
