//! Original user-comparator chain semantics (xt, xx, el, ex, ee, eh, en, eq, ev, ey, ec).
use std::cmp::Ordering;
#[derive(Clone, Debug, Default)]
pub struct Chain(Vec<(u8, bool)>);
#[derive(Clone, Copy)]
pub struct User<'a> {
    pub name: &'a str,
    pub world: i32,
    pub rank: i32,
    pub sequence: i32,
}
impl Chain {
    pub fn clear(&mut self) {
        self.0.clear();
    }
    pub fn add(&mut self, kind: u8, ascending: bool) {
        if self.0.last().is_none_or(|&(k, _)| !matches!(k, 0 | 1 | 3)) {
            self.0.push((kind, ascending));
        }
    }
    pub fn compare(
        &self,
        a: User,
        b: User,
        world: i32,
        natural: impl Fn(bool) -> Ordering,
    ) -> Ordering {
        if self.0.is_empty() {
            return natural(false);
        }
        for &(kind, ascending) in &self.0 {
            if kind == 0 {
                return natural(!ascending);
            }
            let value = match kind {
                1 => Some(a.name.cmp(b.name)),
                2 => (a.world != b.world).then(|| a.world.wrapping_sub(b.world).cmp(&0)),
                3 => Some(a.sequence.wrapping_sub(b.sequence).cmp(&0)),
                4 => {
                    ((a.world != 0) != (b.world != 0)).then(|| (b.world != 0).cmp(&(a.world != 0)))
                }
                5 => (a.world != 0 && b.world != 0).then(|| a.name.cmp(b.name)),
                6 => (a.world != 0 && b.world != 0)
                    .then(|| a.sequence.wrapping_sub(b.sequence).cmp(&0)),
                7 => ((a.world == world) != (b.world == world))
                    .then(|| (b.world == world).cmp(&(a.world == world))),
                8 => (a.world == world && b.world == world).then(|| a.name.cmp(b.name)),
                9 => (a.world == world && b.world == world)
                    .then(|| a.sequence.wrapping_sub(b.sequence).cmp(&0)),
                10 => (a.rank != b.rank).then(|| a.rank.wrapping_sub(b.rank).cmp(&0)),
                _ => None,
            };
            if let Some(value) = value {
                return if ascending {
                    value
                } else if matches!(kind, 3 | 6 | 9) {
                    b.sequence.wrapping_sub(a.sequence).cmp(&0)
                } else {
                    value.reverse()
                };
            }
        }
        Ordering::Equal
    }
}
