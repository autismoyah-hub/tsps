//! Original GameWorldMapRegion.bf: region clipping around retained icon hit positions.
use crate::{map_label::Label, Result};
pub struct Symbol {
    pub element: i32,
    pub center: [i32; 2],
    pub sprite_size: [i32; 2],
    pub alignment: [Option<u8>; 2],
    pub label: Option<Label>,
}
pub fn contains([x, y, w, h]: [i32; 4], point: [i32; 2]) -> bool {
    point[0] >= x && point[1] >= y && point[0] < x.wrapping_add(w) && point[1] < y.wrapping_add(h)
}
impl Symbol {
    pub fn hit(&self, rect: [i32; 4], point: [i32; 2]) -> Result<bool> {
        if self.element < 0 || !contains(rect, point) {
            return Ok(false);
        }
        Ok(
            crate::map_icon::sprite_hit(self.sprite_size, self.center, point, self.alignment)?
                || self
                    .label
                    .as_ref()
                    .is_some_and(|label| label.hit(self.center, point)),
        )
    }
}
