//! Original physical-slot traversal for static model-data lighting (GameScene.cb/ce).
use crate::{model::Model, model_light::Normals, Result};
use std::collections::HashMap;
#[derive(Clone, Copy)]
pub enum Kind {
    Wall(u8),
    Game,
    Ground,
}
pub struct Object<'a> {
    pub tile: [usize; 3],
    pub size: [usize; 2],
    pub kind: Kind,
    pub model: &'a Model,
    pub ambient: i32,
    pub contrast: i32,
}
#[derive(Default)]
struct Slot {
    walls: [Option<usize>; 2],
    games: Vec<usize>,
    ground: Option<usize>,
}
fn merge(
    objects: &[Object<'_>],
    normals: &mut [Normals],
    a: usize,
    b: usize,
    offset: [i32; 3],
    hide: bool,
) {
    if a == b {
        return;
    }
    if a < b {
        let (left, right) = normals.split_at_mut(b);
        left[a].merge(
            objects[a].model,
            &mut right[0],
            objects[b].model,
            offset,
            hide,
        );
    } else {
        let (left, right) = normals.split_at_mut(a);
        right[0].merge(
            objects[a].model,
            &mut left[b],
            objects[b].model,
            offset,
            hide,
        );
    }
}
fn neighbors(
    a: usize,
    origin: [usize; 3],
    objects: &[Object<'_>],
    slots: &HashMap<[usize; 3], Slot>,
    alive: &[bool],
    normals: &mut [Normals],
    dimensions: [usize; 2],
    elevation: &impl Fn([usize; 3]) -> i32,
) {
    let [plane, x, z] = origin;
    let [sx, sz] = objects[a].size.map(|v| v as i32);
    let [x, z] = [x as i32, z as i32];
    for p in plane..=(plane + 1).min(3) {
        let same = p == plane;
        for nx in x - i32::from(!same)..=x + sx {
            for nz in z - 1..=z + sz {
                if nx < 0
                    || nz < 0
                    || nx >= dimensions[0] as i32
                    || nz >= dimensions[1] as i32
                    || (same && nx < x + sx && nz < z + sz && !(nz < z && x != nx))
                {
                    continue;
                }
                let tile = [p, nx as usize, nz as usize];
                let Some(slot) = slots.get(&tile) else {
                    continue;
                };
                let dy = elevation(tile).wrapping_sub(elevation(origin));
                for b in slot.walls.into_iter().flatten().filter(|b| alive[*b]) {
                    merge(
                        objects,
                        normals,
                        a,
                        b,
                        [
                            (nx - x) * 128 + (1 - sx) * 64,
                            dy,
                            (nz - z) * 128 + (1 - sz) * 64,
                        ],
                        same,
                    );
                }
                for &b in slot.games.iter().filter(|b| alive[**b]) {
                    let target = &objects[b];
                    merge(
                        objects,
                        normals,
                        a,
                        b,
                        [
                            (target.tile[1] as i32 - x) * 128 + (target.size[0] as i32 - sx) * 64,
                            dy,
                            (target.tile[2] as i32 - z) * 128 + (target.size[1] as i32 - sz) * 64,
                        ],
                        same,
                    );
                }
            }
        }
    }
}
pub fn lighting(
    objects: &[Object<'_>],
    dimensions: [usize; 2],
    elevation: impl Fn([usize; 3]) -> i32,
) -> Result<Vec<Vec<[i32; 3]>>> {
    let mut normals: Vec<_> = objects.iter().map(|o| o.model.normals()).collect();
    let mut alive = vec![true; objects.len()];
    let mut slots = HashMap::<[usize; 3], Slot>::new();
    for (index, object) in objects.iter().enumerate() {
        match object.kind {
            Kind::Wall(part) => {
                slots.entry(object.tile).or_default().walls[part as usize] = Some(index)
            }
            Kind::Ground => slots.entry(object.tile).or_default().ground = Some(index),
            Kind::Game => {
                for dx in 0..object.size[0] {
                    for dz in 0..object.size[1] {
                        let [p, x, z] = object.tile;
                        slots
                            .entry([p, x + dx, z + dz])
                            .or_default()
                            .games
                            .push(index);
                    }
                }
            }
        }
    }
    let mut order: Vec<_> = slots.keys().copied().collect();
    order.sort_unstable();
    for tile in order {
        let slot = &slots[&tile];
        if let Some(a) = slot.walls[0].filter(|a| alive[*a]) {
            neighbors(
                a,
                tile,
                objects,
                &slots,
                &alive,
                &mut normals,
                dimensions,
                &elevation,
            );
            if let Some(b) = slot.walls[1].filter(|b| alive[*b]) {
                neighbors(
                    b,
                    tile,
                    objects,
                    &slots,
                    &alive,
                    &mut normals,
                    dimensions,
                    &elevation,
                );
                merge(objects, &mut normals, a, b, [0; 3], false);
                alive[b] = false;
            }
            alive[a] = false;
        }
        for &a in &slot.games {
            if alive[a] {
                neighbors(
                    a,
                    tile,
                    objects,
                    &slots,
                    &alive,
                    &mut normals,
                    dimensions,
                    &elevation,
                );
                alive[a] = false;
            }
        }
        if let Some(a) = slot.ground.filter(|a| alive[*a]) {
            let [p, x, z] = tile;
            for (dx, dz) in [(0, 1), (1, -1), (1, 0), (1, 1)] {
                let Some(nz) = z.checked_add_signed(dz) else {
                    continue;
                };
                let target = [p, x + dx, nz];
                if let Some(b) = slots
                    .get(&target)
                    .and_then(|s| s.ground)
                    .filter(|b| alive[*b])
                {
                    merge(
                        objects,
                        &mut normals,
                        a,
                        b,
                        [
                            dx as i32 * 128,
                            elevation(target).wrapping_sub(elevation(tile)),
                            dz as i32 * 128,
                        ],
                        true,
                    );
                }
            }
            alive[a] = false;
        }
    }
    objects
        .iter()
        .zip(&normals)
        .map(|(o, n)| {
            o.model
                .light_with_normals(o.ambient, o.contrast, [-50, -10, -50], n)
        })
        .collect()
}
