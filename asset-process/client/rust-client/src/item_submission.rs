//! Ground-item source identities survive deferred model loading.
use crate::{item_scene::Pile, render3d::Triangle, scene::Scene, zone};
use std::collections::{HashMap, HashSet};

pub struct Source {
    pub pile: Pile,
    pub tags: [Option<u64>; 3],
}

pub fn tag(id: u16, plane: usize, tile: [usize; 2]) -> u64 {
    0x1000000000000000
        | ((id as u64) << 32)
        | ((plane as u64) << 28)
        | ((tile[0] as u64) << 14)
        | tile[1] as u64
}

pub fn sources(zone: &zone::State, scene: &Scene, support: &HashMap<[u8; 3], i32>) -> Vec<Source> {
    zone.piles
        .iter()
        .filter_map(|(position, items)| {
            let [plane, x, z] = *position;
            let x = x.checked_sub(scene.base_x)?;
            let z = z.checked_sub(scene.base_y)?;
            if !(0..4).contains(&plane)
                || x < 0
                || z < 0
                || x as usize >= scene.width
                || z as usize >= scene.height
            {
                return None;
            }
            let mut selected = [None; 3];
            let mut count = 0;
            // zone::State already maintains Java's highest-value primary at index 0.
            for item in items {
                if selected.contains(&Some(item.id)) {
                    continue;
                }
                selected[count] = Some(item.id);
                count += 1;
                if count == 3 {
                    break;
                }
            }
            if count == 0 {
                return None;
            }
            let tile = [x as usize, z as usize];
            Some(Source {
                pile: Pile {
                    plane: plane as usize,
                    tile,
                    present: selected.map(|id| id.is_some()),
                    support: support
                        .get(&[plane as u8, x as u8, z as u8])
                        .copied()
                        .unwrap_or(0),
                },
                tags: selected.map(|id| id.map(|id| tag(id, plane as usize, tile))),
            })
        })
        .collect()
}

/// Geometry batches are globally rebased before registration; source IDs are not.
pub fn keys(scene: &Scene, sources: &[Source]) -> Vec<(u64, usize)> {
    sources
        .iter()
        .flat_map(|source| {
            source
                .tags
                .iter()
                .enumerate()
                .filter_map(move |(part, tag)| {
                    Some((
                        (*tag)?,
                        scene.item_identity(source.pile.plane, source.pile.tile, part)?,
                    ))
                })
        })
        .collect()
}
pub fn batches<'a>(
    keys: &[(u64, usize)],
    triangles: impl IntoIterator<Item = &'a Triangle>,
    view: u16,
) -> Vec<(usize, u64, usize)> {
    let identities = keys.iter().copied().collect::<HashMap<_, _>>();
    let mut seen = HashSet::new();
    triangles
        .into_iter()
        .filter(|t| t.view == view)
        .filter_map(|triangle| {
            let identity = *identities.get(&triangle.tag)?;
            let batch = triangle.model_order.as_ref()?.batch;
            seen.insert((identity, triangle.tag, batch))
                .then_some((identity, triangle.tag, batch))
        })
        .collect()
}
