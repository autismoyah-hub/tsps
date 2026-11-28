//! Original rl16 temporary-actor depth key and stable far-to-near order.
pub fn depth(projected: f32, local_player: bool) -> i32 {
    (projected as i32).wrapping_sub(if local_player { 12 } else { 0 })
}
pub fn order(projected: &[(f32, bool)]) -> Vec<usize> {
    let mut indices = (0..projected.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| std::cmp::Reverse(depth(projected[i].0, projected[i].1)));
    indices
}

/// Reorder temporary renderables within each view's existing submission slots.
/// Static models and other views retain their positions until full scene traversal
/// (including child-scene recursion and alpha zones) supplies their own schedule.
pub fn schedule(keys: &[Option<(u16, usize, i32)>]) -> Vec<usize> {
    let keys = keys
        .iter()
        .enumerate()
        .map(|(i, key)| key.map(|(view, zone, depth)| (view, zone, depth, i)))
        .collect::<Vec<_>>();
    schedule_with_source(&keys)
}
/// Scene insertion ordinal is the stable tie-breaker before separately built
/// player/NPC meshes are joined. It never overrides zone or projected depth.
pub fn schedule_with_source(keys: &[Option<(u16, usize, i32, usize)>]) -> Vec<usize> {
    let mut result = (0..keys.len()).collect::<Vec<_>>();
    let mut views = std::collections::BTreeMap::<u16, Vec<usize>>::new();
    for (i, key) in keys.iter().enumerate() {
        if let Some((view, _, _, _)) = key {
            views.entry(*view).or_default().push(i);
        }
    }
    for positions in views.values() {
        let mut sorted = positions.clone();
        sorted.sort_by_key(|&i| {
            let (_, zone, depth, source) = keys[i].unwrap();
            (std::cmp::Reverse(zone), std::cmp::Reverse(depth), source)
        });
        for (&slot, index) in positions.iter().zip(sorted) {
            result[slot] = index;
        }
    }
    result
}
