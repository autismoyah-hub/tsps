//! Terrain model identity and Java's selection-plane paint admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Submission {
    pub physical_plane: usize,
    pub tile: [usize; 2],
    /// Shaped tiles do not use the hidden-paint selection-plane gate.
    pub shaped: bool,
    pub visible_paint: bool,
}
impl Submission {
    pub fn identity(self, size: [usize; 2]) -> usize {
        (self.physical_plane * size[0] + self.tile[0]) * size[1] + self.tile[1]
    }
    pub fn tag(self) -> u64 {
        (1 << 63)
            | ((self.physical_plane as u64) << 28)
            | ((self.tile[0] as u64) << 14)
            | self.tile[1] as u64
    }
}
/// Geometry visibility is supplied separately from traversal's surface occupancy.
/// Real scene occlusion is not yet applied by this native adapter.
pub fn sync(
    scene: &mut crate::object_traversal::Scene,
    submissions: &[Submission],
    selection_plane: i32,
) {
    for tile in &mut scene.tiles {
        tile.terrain_call = None;
        tile.bridge_terrain_call = None;
    }
    let size = [scene.dimensions[1], scene.dimensions[2]];
    for &submission in submissions {
        let [x, z] = submission.tile;
        if submission.physical_plane >= 4 || x >= size[0] || z >= size[1] {
            continue;
        }
        let bridge = scene.tiles[scene.index(0, x, z)].bridge;
        let logical = if bridge {
            (submission.physical_plane + 3) % 4
        } else {
            submission.physical_plane
        };
        let index = scene.index(logical, x, z);
        let identity = submission.identity(size);
        let tile = &mut scene.tiles[index];
        // GameScene.gx clears the saved bridge payload when logical plane three
        // is allocated. Static mesh metadata must not restore that old terrain.
        if bridge && submission.physical_plane == 0 && tile.present {
            continue;
        }
        if submission.shaped || submission.visible_paint || logical as i32 <= selection_plane {
            tile.terrain_call = Some(identity);
        }
        if bridge && submission.physical_plane == 0 {
            tile.bridge_terrain_call = Some(identity);
        }
    }
}
