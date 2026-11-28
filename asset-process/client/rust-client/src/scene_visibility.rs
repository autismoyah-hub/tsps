//! Active GameScene.kq/kg visibility map, configured by ee.ak/GameScene.pf.
use std::collections::HashMap;

pub struct Visibility {
    pub heights: [i32; 17],
    width: i32,
    far: i32,
    pitch: usize,
    yaw: usize,
    cells: HashMap<(usize, usize, i32, i32), bool>,
}
impl Visibility {
    pub fn new(
        size: [i32; 2],
        zoom: [i32; 2],
        radius: i32,
        bins: [usize; 2],
    ) -> crate::Result<Self> {
        if size.iter().any(|&n| n <= 0)
            || !(25..=90).contains(&radius)
            || bins[0] > 16
            || bins[1] >= 16
        {
            return Err(crate::Error::Invalid("scene visibility parameters"));
        }
        let height = size[1];
        let scale = zoom[0] + (height - 334).clamp(0, 100) * (zoom[1] - zoom[0]) / 100;
        let heights = std::array::from_fn(|index| {
            let angle = 128 + index as i32 * 256;
            let distance = scale * (600 + 3 * (angle / 8)) / 256;
            crate::model_bounds::trig(angle, 0)[0].wrapping_mul(distance) >> 16
        });
        Ok(Self {
            heights,
            width: size[0] * 334 / height,
            far: radius * 210,
            pitch: bins[0],
            yaw: bins[1],
            cells: HashMap::new(),
        })
    }
    fn cell(&mut self, pitch: usize, yaw: usize, x: i32, z: i32) -> bool {
        let key = (pitch, yaw, x, z);
        if let Some(&result) = self.cells.get(&key) {
            return result;
        }
        let [sp, cp, sy, cy] = crate::model_bounds::trig(pitch as i32 * 256 + 1, yaw as i32 * 1024);
        let x = x * 128;
        let z = z * 128;
        let px = z.wrapping_mul(sy).wrapping_add(x.wrapping_mul(cy)) >> 16;
        let pz = z.wrapping_mul(cy).wrapping_sub(x.wrapping_mul(sy)) >> 16;
        let result = (-500..=800).step_by(128).any(|offset| {
            let y = self.heights[pitch] + offset;
            let depth = y.wrapping_mul(sp).wrapping_add(pz.wrapping_mul(cp)) >> 16;
            if depth < 50 || depth > self.far {
                return false;
            }
            let py = y.wrapping_mul(cp).wrapping_sub(pz.wrapping_mul(sp)) >> 16;
            let sx = self.width / 2 + px * 128 / depth;
            let sy = 167 + py * 128 / depth;
            (0..=self.width).contains(&sx) && (0..=334).contains(&sy)
        });
        self.cells.insert(key, result);
        result
    }
    /// Tile offset from the clamped root camera tile. GameScene's focus offset
    /// cancels between kq's query and cm's sample coordinates.
    pub fn visible(&mut self, offset: [i32; 2]) -> bool {
        // The original final-map dirty/value bitsets cover bins 0..15.
        // Extended pitch bin16 reads the untouched false value.
        if self.pitch == 16 {
            return false;
        }
        for dx in -1..=1 {
            for dz in -1..=1 {
                for (pitch, yaw) in [
                    (self.pitch, self.yaw),
                    (self.pitch, (self.yaw + 1) % 16),
                    (self.pitch + 1, self.yaw),
                    (self.pitch + 1, (self.yaw + 1) % 16),
                ] {
                    if self.cell(pitch, yaw, offset[0] + dx, offset[1] + dz) {
                        return true;
                    }
                }
            }
        }
        false
    }
}
