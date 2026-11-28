//! Geography icon scan order and source coordinates (GameWorldMapRegion.gl/uy).
use crate::{
    buffer::Reader, map_composite::Composite, map_element, map_geography::Tile, object, Error,
    Result,
};
use std::collections::HashMap;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub tile: [i32; 2],
    pub source: [i32; 3],
    pub coord: [i32; 3],
    pub object: u32,
    pub element: i32,
    pub dynamic: bool,
}
fn hash(tile: [i32; 2]) -> u32 {
    let h = ((tile[0] as u32) << 14) | tile[1] as u32;
    h ^ (h >> 16)
}
pub fn collect(
    bytes: &[u8],
    composite: &Composite,
    destination: [u16; 2],
    objects: &HashMap<u32, object::Definition>,
    elements: &HashMap<u32, map_element::Definition>,
    mut resolve: impl FnMut(&object::Definition) -> Result<Option<u32>>,
) -> Result<Vec<Location>> {
    let mut r = Reader::new(bytes);
    let mut result: Vec<Location> = Vec::new();
    let mut capacity = 16;
    let mut parts = Vec::new();
    if let Some(region) = composite.regions.get(&destination) {
        parts.push((region, [0, 0], [0, 0], 64));
    } else {
        for chunk in &composite.chunks {
            if chunk.region.destination == destination {
                parts.push((&chunk.region, chunk.source, chunk.destination, 8));
            }
        }
    }
    for (region, source_chunk, dest_chunk, side) in parts {
        if dest_chunk.iter().any(|&v| v >= 8) {
            return Err(Error::Invalid("map icon chunk position"));
        }
        for x in 0..side {
            for y in 0..side {
                let tile = Tile::read(&mut r, region.planes)?;
                let point = [dest_chunk[0] as i32 * 8 + x, dest_chunk[1] as i32 * 8 + y];
                let candidate = tile
                    .layers
                    .iter()
                    .enumerate()
                    .flat_map(|(plane, layer)| layer.decorations.iter().map(move |d| (plane, d)))
                    .find_map(|(plane, decoration)| {
                        let def = objects.get(&(decoration.id as u32))?;
                        let eligible =
                            def.transforms
                                .as_ref()
                                .map_or(def.map_icon.is_some(), |ids| {
                                    ids.iter().flatten().any(|id| {
                                        objects
                                            .get(&(*id as u32))
                                            .is_some_and(|d| d.map_icon.is_some())
                                    })
                                });
                        eligible.then_some((plane, def))
                    });
                let Some((plane, def)) = candidate else {
                    continue;
                };
                let dynamic = def.transforms.is_some();
                let element = if dynamic {
                    let id = resolve(def)?.ok_or(Error::Invalid("map icon null transform"))?;
                    objects
                        .get(&id)
                        .and_then(|d| d.map_icon)
                        .map_or(-1, |id| id as i32)
                } else {
                    def.map_icon.map_or(-1, |id| id as i32)
                };
                // A hidden first candidate still stops scanning the tile. Later objects do not win.
                if elements
                    .get(&(element as u32))
                    .is_some_and(|d| !d.world_map)
                {
                    continue;
                }
                let location = Location {
                    tile: point,
                    source: [
                        plane as i32 + region.plane as i32,
                        region.source[0] as i32 * 64 + point[0] + source_chunk[0] as i32 * 8,
                        region.source[1] as i32 * 64 + point[1] + source_chunk[1] as i32 * 8,
                    ],
                    coord: [
                        plane as i32,
                        destination[0] as i32 * 64 + point[0],
                        destination[1] as i32 * 64 + point[1],
                    ],
                    object: def.id,
                    element,
                    dynamic,
                };
                if let Some(index) = result.iter().position(|old| old.tile == point) {
                    result[index] = location;
                } else {
                    let bucket = hash(point) as usize & (capacity - 1);
                    let chain = result
                        .iter()
                        .filter(|old| hash(old.tile) as usize & (capacity - 1) == bucket)
                        .count();
                    if chain >= 8 {
                        if capacity < 64 {
                            capacity *= 2;
                        } else {
                            return Err(Error::Unsupported("map icon HashMap tree bin"));
                        }
                    }
                    result.push(location);
                    if result.len() > capacity * 3 / 4 {
                        capacity *= 2;
                    }
                }
            }
        }
    }
    if r.remaining() != 0 {
        return Err(Error::Invalid("map icon geography trailing bytes"));
    }
    // Cache239 has at most 39 entries and four per bucket; no tree bins.
    result.sort_by_key(|icon| hash(icon.tile) as usize & (capacity - 1));
    Ok(result)
}
pub fn refresh(
    locations: &mut [Location],
    objects: &HashMap<u32, object::Definition>,
    mut resolve: impl FnMut(&object::Definition) -> Result<Option<u32>>,
) -> Result<bool> {
    let mut changed = false;
    for icon in locations.iter_mut().filter(|i| i.dynamic) {
        let def = objects
            .get(&icon.object)
            .ok_or(Error::Invalid("map icon object missing"))?;
        let id = resolve(def)?.ok_or(Error::Invalid("map icon null transform"))?;
        let element = objects
            .get(&id)
            .and_then(|d| d.map_icon)
            .map_or(-1, |id| id as i32);
        changed |= icon.element != element;
        icon.element = element;
    }
    Ok(changed)
}
