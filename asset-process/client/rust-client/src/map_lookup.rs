//! Java HashMap iteration order used by GameRenderOverview.aw for cache areas.
use crate::map_area::Area;
use std::collections::BTreeMap;
pub fn ordered(areas: &BTreeMap<i32, Area>) -> Vec<&Area> {
    let mut capacity = areas.len().max(1).next_power_of_two();
    let mut entries: Vec<&Area> = Vec::new();
    for area in areas.values() {
        if let Some(index) = entries
            .iter()
            .position(|old| old.internal_name == area.internal_name)
        {
            entries[index] = area;
        } else {
            entries.push(area);
        }
    }
    if entries.len() > capacity * 3 / 4 {
        capacity *= 2;
    }
    // Cache239's52 names have no tree bins. Stable sorting preserves insertion
    // order within each bucket, including the original resize split order.
    entries.sort_by_key(|area| {
        let hash = area
            .internal_name
            .encode_utf16()
            .fold(0u32, |h, c| h.wrapping_mul(31).wrapping_add(c as u32));
        ((hash ^ (hash >> 16)) as usize) & (capacity - 1)
    });
    entries
}
pub fn area_at(areas: &BTreeMap<i32, Area>, [plane, x, y]: [i32; 3]) -> Option<i32> {
    ordered(areas)
        .into_iter()
        .find(|area| area.contains_coord(plane, x, y))
        .map(|area| area.id)
}
