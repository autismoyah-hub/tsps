//! Temporary GameScene.bs submission footprint, before scene padding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Footprint {
    pub origin: [i32; 2],
    pub size: [i32; 2],
}
impl Footprint {
    /// Attached effects use GameScene.bb rather than radius-based actor insertion.
    pub fn spot(fine: [i32; 2]) -> Self {
        Self {
            origin: fine.map(|v| v >> 7),
            size: [1, 1],
        }
    }

    pub fn actor(fine: [i32; 2], radius: i32, angle: i32, stretch: bool) -> Self {
        let mut low = fine.map(|v| v.wrapping_sub(radius));
        let mut high = fine.map(|v| v.wrapping_add(radius));
        if stretch {
            if angle > 640 && angle < 1408 {
                high[1] = high[1].wrapping_add(128);
            }
            if angle > 1152 && angle < 1920 {
                high[0] = high[0].wrapping_add(128);
            }
            if angle > 1664 || angle < 384 {
                low[1] = low[1].wrapping_sub(128);
            }
            if angle > 128 && angle < 896 {
                low[0] = low[0].wrapping_sub(128);
            }
        }
        let origin = low.map(|v| v >> 7);
        let high = high.map(|v| v >> 7);
        Self {
            origin,
            size: std::array::from_fn(|i| high[i].wrapping_sub(origin[i]).wrapping_add(1)),
        }
    }
    pub fn zone(self, padding: i32) -> [i32; 2] {
        self.origin.map(|v| v.wrapping_add(padding) >> 3)
    }
}
