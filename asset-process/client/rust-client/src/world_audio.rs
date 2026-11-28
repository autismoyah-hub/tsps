//! Original nf.ag spatial sound placement and world-entity audibility gates.
use crate::{area_sound::Request, world_entity};
pub struct Context<'a> {
    pub root: Option<&'a world_entity::State>,
    pub base: [i32; 2],
    pub local_view: u16,
}
impl Context<'_> {
    pub fn volume(
        &self,
        request: &Request,
        world: u16,
        listener: [i32; 2],
        effects: i32,
        area: i32,
        cross_world: bool,
    ) -> i32 {
        if request.packed == 0 {
            return effects;
        }
        let source = self.root.and_then(|r| r.entities.get(&world));
        let local = self
            .root
            .and_then(|r| r.entities.get(&self.local_view))
            .map(|_| self.local_view);
        if source.is_some() && local != Some(world) && (local.is_none() || !cross_world) {
            return 0;
        }
        // oe.bj searches only the root entity registry. Transform a tile corner,
        // truncate to integer fine units, then quantize before adding its centre.
        let corner = [
            ((request.packed >> 16) & 255) << 7,
            ((request.packed >> 8) & 255) << 7,
        ];
        let fine = source.map_or([corner[0], corner[1]], |e| {
            let p = e.transform_horizontal(corner);
            [p[0] as i32, p[2] as i32]
        });
        let mut positioned = request.clone();
        positioned.tile = [
            self.base[0].wrapping_add(fine[0] >> 7),
            self.base[1].wrapping_add(fine[1] >> 7),
        ];
        positioned.volume(listener, effects, area)
    }
}
