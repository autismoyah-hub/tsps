//! Relative actor insertion and per-frame tile-center deduplication.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Key {
    pub npc: bool,
    pub index: u16,
}
pub fn ordered(
    high: &[u16],
    local: u16,
    target: Option<u16>,
    render_self: bool,
    npcs: &[(u16, u8)],
) -> Vec<Key> {
    let mut out = Vec::new();
    if render_self {
        out.push(Key {
            npc: false,
            index: local,
        });
    }
    if let Some(index) = target {
        out.push(Key { npc: false, index });
    }
    out.extend(npcs.iter().filter(|v| v.1 == 1).map(|v| Key {
        npc: true,
        index: v.0,
    }));
    out.extend(
        high.iter()
            .copied()
            .filter(|&i| i != local && Some(i) != target)
            .map(|index| Key { npc: false, index }),
    );
    for priority in [0, 2] {
        out.extend(npcs.iter().filter(|v| v.1 == priority).map(|v| Key {
            npc: true,
            index: v.0,
        }));
    }
    out
}
#[derive(Clone, Copy, Debug)]
pub struct Actor {
    pub fine: [i32; 2],
    pub center_dedup: bool,
}
pub fn retained(
    dimensions: [i32; 2],
    order: &[Key],
    mut actor: impl FnMut(Key) -> Option<Actor>,
) -> Vec<Key> {
    let mut tiles = std::collections::HashSet::new();
    let mut out = Vec::new();
    for &key in order {
        let Some(actor) = actor(key) else {
            continue;
        };
        let tile = actor.fine.map(|v| v >> 7);
        if (0..2).any(|i| tile[i] < 0 || tile[i] >= dimensions[i]) {
            continue;
        }
        if actor.center_dedup && actor.fine.iter().all(|v| v & 127 == 64) && !tiles.insert(tile) {
            continue;
        }
        out.push(key);
    }
    out
}
