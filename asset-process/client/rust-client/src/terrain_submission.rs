//! Active software terrain admission, including lowered bridge slots.
use crate::{scene::Scene, scene_visibility::Visibility};

/// Keep one configuration's relative tile queries across frames. Camera movement
/// changes the queried offsets, not the visibility map. Resize/zoom/angle bins
/// replace the cache, so old configurations cannot accumulate indefinitely.
#[derive(Default)]
pub struct Cache {
    current: Option<([i32; 7], Visibility)>,
}
impl Cache {
    pub fn prepare(
        &mut self,
        size: [i32; 2],
        zoom: [i32; 2],
        radius: i32,
        bins: [usize; 2],
    ) -> crate::Result<&mut Visibility> {
        let key = [
            size[0],
            size[1],
            zoom[0],
            zoom[1],
            radius,
            bins[0] as i32,
            bins[1] as i32,
        ];
        if self.current.as_ref().is_none_or(|(old, _)| *old != key) {
            self.current = Some((key, Visibility::new(size, zoom, radius, bins)?));
        }
        Ok(&mut self.current.as_mut().unwrap().1)
    }
}

pub struct View {
    pub root: bool,
    pub size: [i32; 2],
    pub camera: [i32; 2],
    pub eye_y: i32,
    pub radius: i32,
    pub maximum_plane: usize,
}
impl View {
    pub fn admitted(
        &self,
        visibility: &mut Visibility,
        physical: usize,
        tile: [i32; 2],
        flags: [u8; 4],
        heights: [i32; 4],
        bridge: bool,
    ) -> bool {
        if physical >= 4 || (0..2).any(|axis| tile[axis] < 0 || tile[axis] >= self.size[axis]) {
            return false;
        }
        // GameScene.at preserves plane zero underneath the lowered plane-zero
        // slot. GameScene.fn reaches that paint recursively after the owner passes.
        let slot = physical.saturating_sub(usize::from(bridge));
        let owner = slot + usize::from(bridge);
        let minimum = if flags[owner] & 8 != 0 { 0 } else { slot };
        self.slot_admitted(visibility, tile, minimum, heights[slot])
    }
    /// The active traversal's predicate after resolving a scene slot.
    pub fn slot_admitted(
        &self,
        visibility: &mut Visibility,
        tile: [i32; 2],
        minimum: usize,
        height: i32,
    ) -> bool {
        if (0..2).any(|axis| tile[axis] < 0 || tile[axis] >= self.size[axis]) {
            return false;
        }
        if minimum > self.maximum_plane {
            return false;
        }
        if !self.root {
            return true;
        }
        let offset = [tile[0] - self.camera[0], tile[1] - self.camera[1]];
        if offset.iter().any(|&n| n < -self.radius || n >= self.radius) {
            return false;
        }
        // bk uses the logical slot's cb height, before fn resolves its physical
        // paint height. In particular, a bridge does not use cb[physical] here.
        height.wrapping_sub(self.eye_y) >= 2000 || visibility.visible(offset)
    }

    pub fn terrain_tag(&self, visibility: &mut Visibility, scene: &Scene, tag: u64) -> bool {
        if tag & (1 << 63) == 0 {
            return true;
        }
        let plane = ((tag >> 28) & 3) as usize;
        let x = ((tag >> 14) & 16383) as usize;
        let z = (tag & 16383) as usize;
        if x >= scene.width || z >= scene.height {
            return false;
        }
        let flags = std::array::from_fn(|plane| scene.tile(plane, x, z).unwrap().flags);
        let heights = std::array::from_fn(|plane| scene.height(plane, x, z).unwrap());
        self.admitted(
            visibility,
            plane,
            [x as i32, z as i32],
            flags,
            heights,
            flags[1] & 2 != 0,
        )
    }
}
