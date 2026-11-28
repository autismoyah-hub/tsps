//! Item-layer allocation and identity, independent of inventory-model readiness.
use crate::object_traversal::Scene;

#[derive(Clone, Copy, Debug)]
pub struct Pile {
    pub plane: usize,
    pub tile: [usize; 2],
    /// Source renderables in primary, secondary, tertiary order.
    pub present: [bool; 3],
    pub support: i32,
}

#[derive(Default)]
pub struct Layers {
    active: Vec<usize>,
}

impl Layers {
    /// GameScene.bx clears item layers, retaining allocated logical tiles.
    /// GameScene.ep allocates only the pile's plane, even with null renderables.
    pub fn replace(&mut self, scene: &mut Scene, piles: &[Pile]) {
        for index in self.active.drain(..) {
            scene.tiles[index].items = [None; 3];
            scene.tiles[index].raised_items = false;
        }
        for pile in piles {
            let [x, z] = pile.tile;
            if pile.plane >= scene.dimensions[0]
                || x >= scene.dimensions[1]
                || z >= scene.dimensions[2]
            {
                continue;
            }
            scene.allocate(pile.plane, x, z);
            let index = scene.index(pile.plane, x, z);
            let tile = &mut scene.tiles[index];
            // fn/nh invoke the secondary and tertiary models before the primary.
            tile.items = [1, 2, 0].map(|part| pile.present[part].then_some(index * 3 + part));
            tile.raised_items = pile.support != 0;
            self.active.push(index);
        }
    }
}

/// Stable within a view and independent of geometry availability or pile value.
pub fn identity(scene: &Scene, plane: usize, tile: [usize; 2], part: usize) -> Option<usize> {
    (plane < scene.dimensions[0]
        && tile[0] < scene.dimensions[1]
        && tile[1] < scene.dimensions[2]
        && part < 3)
        .then(|| scene.index(plane, tile[0], tile[1]) * 3 + part)
}
