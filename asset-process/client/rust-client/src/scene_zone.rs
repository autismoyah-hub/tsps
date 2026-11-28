//! Original GameScene.sn zone selection and rl18 near-to-far ordering.
pub fn distance(zone: [i32; 2], camera: [i32; 2], padding: i32) -> i32 {
    let d = std::array::from_fn::<_, 2, _>(|i| {
        zone[i]
            .wrapping_mul(8)
            .wrapping_add(4)
            .wrapping_sub(padding)
            .wrapping_shl(7)
            .wrapping_sub(camera[i])
    });
    d[0].wrapping_mul(d[0])
        .wrapping_add(d[1].wrapping_mul(d[1]))
}
pub fn selected(
    dimensions: [i32; 2],
    camera: [i32; 2],
    padding: i32,
    draw_distance: i32,
) -> Vec<[i32; 2]> {
    let center = camera.map(|n| (n >> 7).wrapping_add(padding) >> 3);
    let radius = draw_distance >> 3;
    let mut zones = Vec::new();
    // Clip the iteration window before visiting cells. This preserves Java's
    // x/y insertion order while bounding work by the scene rather than a setting.
    let low: [i32; 2] = std::array::from_fn(|i| center[i].saturating_sub(radius).max(0));
    let high: [i32; 2] = std::array::from_fn(|i| {
        center[i]
            .saturating_add(radius)
            .min((dimensions[i] >> 3) - 1)
    });
    if radius >= 0 {
        for x in low[0]..=high[0] {
            for y in low[1]..=high[1] {
                zones.push([x, y]);
            }
        }
    }
    zones.sort_by_key(|&zone| distance(zone, camera, padding));
    zones
}

/// All scene zones in stable near-to-far order, without introducing visibility culling.
pub fn ordered(dimensions: [i32; 2], camera: [i32; 2], padding: i32) -> Vec<[i32; 2]> {
    let mut zones = Vec::new();
    for x in 0..dimensions[0] >> 3 {
        for y in 0..dimensions[1] >> 3 {
            zones.push([x, y]);
        }
    }
    zones.sort_by_key(|&zone| distance(zone, camera, padding));
    zones
}
