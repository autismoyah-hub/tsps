//! Item search and result iteration used by CS2 opcodes 4210–4212.
use crate::{
    item::Definitions,
    vm::{Object, Stacks},
    Error, Result,
};
#[derive(Default)]
pub struct Search {
    results: Vec<u16>,
    position: usize,
    pub overflow: bool,
}
impl Search {
    pub fn find(&mut self, items: &Definitions, query: &str, searchable_only: bool) -> i32 {
        self.results.clear();
        self.position = 0;
        self.overflow = false;
        let query = query.to_lowercase();
        let mut ids = items.items.keys().copied().collect::<Vec<_>>();
        ids.sort_unstable();
        let mut names = Vec::new();
        for id in ids {
            let d = &items.items[&id];
            if (!searchable_only || d.searchable)
                && d.note[1] == -1
                && d.placeholder[1] == -1
                && d.name.to_lowercase().contains(&query)
            {
                if self.results.len() == 250 {
                    self.results.clear();
                    self.overflow = true;
                    return -1;
                }
                self.results.push(id as u16);
                names.push(d.name.encode_utf16().collect::<Vec<_>>());
            }
        }
        if !names.is_empty() {
            let last = names.len() - 1;
            sort(&mut names, &mut self.results, 0, last);
        }
        self.results.len() as i32
    }
    pub fn next_id(&mut self) -> i32 {
        if let Some(&id) = self.results.get(self.position) {
            self.position += 1;
            id as i32
        } else {
            -1
        }
    }
    pub fn reset(&mut self) {
        self.position = 0;
    }
    pub fn execute(&mut self, op: u16, s: &mut Stacks, items: &Definitions) -> Result<bool> {
        match op {
            4210 => {
                let query = match s.pop_object()? {
                    Object::String(v) => v,
                    _ => return Err(Error::Invalid("item search string")),
                };
                let only = s.pop_int()? == 1;
                s.push_int(self.find(items, &query, only))?;
            }
            4211 => s.push_int(self.next_id())?,
            4212 => self.reset(),
            _ => return Ok(false),
        }
        Ok(true)
    }
}
// Preserve the original quicksort's alternating treatment of equal names.
fn sort(names: &mut [Vec<u16>], ids: &mut [u16], lo: usize, hi: usize) {
    if lo >= hi {
        return;
    }
    let middle = (lo + hi) / 2;
    names.swap(middle, hi);
    ids.swap(middle, hi);
    let pivot = names[hi].clone();
    let mut target = lo;
    for i in lo..hi {
        let order = names[i].cmp(&pivot);
        if order.is_lt() || (order.is_eq() && i & 1 == 1) {
            names.swap(i, target);
            ids.swap(i, target);
            target += 1;
        }
    }
    names.swap(target, hi);
    ids.swap(target, hi);
    if target > lo {
        sort(names, ids, lo, target - 1);
    }
    if target < hi {
        sort(names, ids, target + 1, hi);
    }
}
