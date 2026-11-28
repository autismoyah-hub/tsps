//! Stable first-pass GameScene.bk/fn object order within each view's model slots.
pub type Key = [usize; 4];
pub fn key(plane: usize, tile: [usize; 2], bridge: bool, runtime: bool) -> Key {
    let lowered = bridge && !runtime;
    [
        plane.saturating_sub(usize::from(lowered)),
        tile[0],
        tile[1],
        usize::from(!(lowered && plane == 0)),
    ]
}
pub fn schedule(keys: &[Option<(u16, Key)>]) -> Vec<usize> {
    let mut result = (0..keys.len()).collect::<Vec<_>>();
    let mut views = std::collections::BTreeMap::<u16, Vec<usize>>::new();
    for (i, key) in keys.iter().enumerate() {
        if let Some((view, _)) = key {
            views.entry(*view).or_default().push(i);
        }
    }
    for slots in views.values() {
        let mut sorted = slots.clone();
        sorted.sort_by_key(|&i| keys[i].unwrap().1);
        for (&slot, index) in slots.iter().zip(sorted) {
            result[slot] = index;
        }
    }
    result
}
