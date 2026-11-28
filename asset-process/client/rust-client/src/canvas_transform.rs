//! Fit a fixed logical canvas into a native client area without clipping or distortion.
#[derive(Clone, Copy, Debug)]
pub struct Fit {
    pub origin: [i32; 2],
    pub size: [i32; 2],
    pub logical: [i32; 2],
}
impl Fit {
    pub fn new(logical: [i32; 2], physical: [i32; 2]) -> Self {
        let logical = logical.map(|v| v.max(1));
        let physical = physical.map(|v| v.max(1));
        let size =
            if physical[0] as i64 * logical[1] as i64 <= physical[1] as i64 * logical[0] as i64 {
                [
                    physical[0],
                    (physical[0] as i64 * logical[1] as i64 / logical[0] as i64).max(1) as i32,
                ]
            } else {
                [
                    (physical[1] as i64 * logical[0] as i64 / logical[1] as i64).max(1) as i32,
                    physical[1],
                ]
            };
        Self {
            origin: [(physical[0] - size[0]) / 2, (physical[1] - size[1]) / 2],
            size,
            logical,
        }
    }
    pub fn to_logical(self, point: [i32; 2]) -> [i32; 2] {
        std::array::from_fn(|i| {
            ((point[i] as i64 - self.origin[i] as i64) * self.logical[i] as i64)
                .div_euclid(self.size[i] as i64) as i32
        })
    }
    pub fn to_physical(self, point: [i32; 2]) -> [i32; 2] {
        std::array::from_fn(|i| {
            self.origin[i]
                + ((2 * point[i] as i64 + 1) * self.size[i] as i64 / (2 * self.logical[i] as i64))
                    as i32
        })
    }
}
