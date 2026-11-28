//! pr.cz visits the protocol index list separately for each NPC render priority.
pub fn ordered(indices: &[u16], mut priority: impl FnMut(u16) -> Option<u8>) -> Vec<u16> {
    let mut passes: [Vec<u16>; 3] = std::array::from_fn(|_| Vec::new());
    for &index in indices {
        let bucket = match priority(index) {
            Some(1) => 0,
            Some(0) => 1,
            Some(2) => 2,
            _ => continue,
        };
        passes[bucket].push(index);
    }
    passes.into_iter().flatten().collect()
}
