//! Original fn.bd world-list script queries and me.as/wy.ax ordering.
use crate::{
    interface_script::ints,
    vm::{Object, Stacks},
    Result,
};
use std::cmp::Ordering;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct World {
    pub index: i32,
    pub id: i32,
    pub properties: i32,
    pub location: i32,
    pub population: i32,
    pub activity: String,
    pub host: String,
}
#[derive(Clone, Debug)]
enum Source {
    File(std::path::PathBuf),
    Url(String),
}
#[derive(Debug)]
pub struct List {
    pub worlds: Vec<World>,
    cursor: usize,
    pub sort_keys: [i32; 4],
    pub sort_directions: [i32; 4],
    source: Option<Source>,
    pending: Option<std::thread::JoinHandle<Result<Vec<u8>>>>,
    pub last_error: Option<String>,
}
impl Default for List {
    fn default() -> Self {
        Self {
            worlds: Vec::new(),
            cursor: 0,
            sort_keys: [0, 1, 2, 3],
            sort_directions: [1; 4],
            source: None,
            pending: None,
            last_error: None,
        }
    }
}
impl List {
    /// Custom worlds may supply the original binary world-list format locally.
    pub fn set_source(&mut self, path: Option<std::path::PathBuf>) {
        self.source = path.map(Source::File);
        self.pending = None;
    }
    /// HTTP(S) retrieval shares the existing worker and transactional list installation.
    pub fn set_url(&mut self, url: Option<String>) {
        self.source = url.map(Source::Url);
        self.pending = None;
    }
    pub fn install(&mut self, bytes: &[u8]) -> Result<()> {
        let mut worlds = decode(bytes)?;
        sort_columns(&mut worlds, self.sort_keys, self.sort_directions);
        self.worlds = worlds;
        self.last_error = None;
        // GameAABB.ag does not reset the iterator when replacing the list.
        Ok(())
    }
    pub fn poll_load(&mut self) -> bool {
        if let Some(job) = self.pending.as_ref() {
            if !job.is_finished() {
                return false;
            }
            let result = self
                .pending
                .take()
                .unwrap()
                .join()
                .map_err(|_| crate::Error::Io("world-list worker panicked".into()))
                .and_then(|v| v)
                .and_then(|bytes| self.install(&bytes));
            return match result {
                Ok(()) => true,
                Err(error) => {
                    self.last_error = Some(error.to_string());
                    false
                }
            };
        }
        if let Some(source) = self.source.clone() {
            self.pending = Some(std::thread::spawn(move || {
                let path = match source {
                    Source::Url(url) => return crate::world_http::fetch(&url),
                    Source::File(path) => path,
                };
                use std::io::Read;
                let mut bytes = Vec::new();
                std::fs::File::open(path)?
                    .take((8 << 20) + 1)
                    .read_to_end(&mut bytes)?;
                if bytes.len() > 8 << 20 {
                    return Err(crate::Error::Invalid("world-list size limit"));
                }
                Ok(bytes)
            }));
        }
        false
    }

    pub fn configured(&self) -> bool {
        self.source.is_some()
    }
    pub fn loading(&self) -> bool {
        self.pending.is_some()
    }
    /// ds.af promotes a header while retaining the other three sort priorities.
    pub fn sort_column(&mut self, column: i32, direction: i32) {
        if !(0..4).contains(&column) || !(0..=1).contains(&direction) {
            return;
        }
        let mut keys = [column; 4];
        let mut directions = [direction; 4];
        let mut next = 1;
        for i in 0..4 {
            if self.sort_keys[i] != column && next < 4 {
                keys[next] = self.sort_keys[i];
                directions[next] = self.sort_directions[i];
                next += 1;
            }
        }
        self.sort_keys = keys;
        self.sort_directions = directions;
        sort_columns(&mut self.worlds, keys, directions);
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }
    pub fn execute(&mut self, op: u16, s: &mut Stacks) -> Result<bool> {
        let world = match op {
            6500 => {
                let ready = self.poll_load();
                s.push_int(i32::from(ready))?;
                return Ok(true);
            }
            6501 | 6502 => {
                if op == 6501 {
                    self.cursor = 0;
                }
                let world = self.worlds.get(self.cursor);
                if world.is_some() {
                    self.cursor += 1;
                }
                world
            }
            6506 => {
                let id = s.pop_int()?;
                self.worlds.iter().find(|w| w.id == id)
            }
            6511 => {
                let index = s.pop_int()?;
                usize::try_from(index).ok().and_then(|i| self.worlds.get(i))
            }
            6507 => {
                let [primary, reverse, secondary, reverse_secondary] = ints(s)?;
                self.sort(primary, reverse == 1, secondary, reverse_secondary == 1);
                return Ok(true);
            }
            _ => return Ok(false),
        };
        if let Some(w) = world {
            for v in [w.id, w.properties, w.location, w.population] {
                s.push_int(v)?;
            }
            for text in [&w.activity, &w.host] {
                s.push_object(Object::String(text.clone()))?;
            }
        } else {
            for v in [-1, 0, 0, 0] {
                s.push_int(v)?;
            }
            for _ in 0..2 {
                s.push_object(Object::String(String::new()))?;
            }
        }
        Ok(true)
    }
    /// Reproduce the original middle-pivot partition, including equal-key permutations.
    pub fn sort(&mut self, primary: i32, reverse: bool, secondary: i32, reverse_secondary: bool) {
        let mut ranges = vec![(0, self.worlds.len())];
        while let Some((start, end)) = ranges.pop() {
            if end - start < 2 {
                continue;
            }
            let last = end - 1;
            self.worlds.swap((start + last) / 2, last);
            let mut split = start;
            for i in start..last {
                let mut order = compare(&self.worlds[i], &self.worlds[last], primary, reverse);
                if reverse {
                    order = order.reverse();
                }
                if order == Ordering::Equal && secondary != -1 {
                    order = compare(
                        &self.worlds[i],
                        &self.worlds[last],
                        secondary,
                        reverse_secondary,
                    );
                    if reverse_secondary {
                        order = order.reverse();
                    }
                }
                if order != Ordering::Greater {
                    self.worlds.swap(i, split);
                    split += 1;
                }
            }
            self.worlds.swap(split, last);
            ranges.push((split + 1, end));
            ranges.push((start, split));
        }
    }
}
fn compare(a: &World, b: &World, key: i32, reverse: bool) -> Ordering {
    match key {
        1 => {
            let population = |n| if !reverse && n == -1 { 2001 } else { n };
            population(a.population).cmp(&population(b.population))
        }
        2 => a.location.cmp(&b.location),
        3 => match (a.activity.as_str(), b.activity.as_str()) {
            ("-", "-") => Ordering::Equal,
            ("-", _) => {
                if reverse {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            (_, "-") => {
                if reverse {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (a, b) => a.encode_utf16().cmp(b.encode_utf16()),
        },
        4..=7 => {
            let mask = match key {
                4 => 8,
                5 => 2,
                6 => 4,
                _ => 1,
            };
            (a.properties & mask != 0).cmp(&(b.properties & mask != 0))
        }
        _ => a.id.cmp(&b.id),
    }
}

/// GameAABB.ag: version header, unsigned world ID, flags, host/activity, region and signed population.
pub fn decode(bytes: &[u8]) -> Result<Vec<World>> {
    let mut r = crate::buffer::Reader::new(bytes);
    r.u32()?;
    let count = r.u16()? as usize;
    let mut worlds = Vec::with_capacity(count);
    for index in 0..count {
        let id = r.u16()? as i32;
        let properties = r.u32()? as i32;
        let host = r.string()?;
        let activity = r.string()?;
        let location = r.u8()? as i32;
        let population = r.u16()? as i16 as i32;
        worlds.push(World {
            index: index as i32,
            id,
            properties,
            location,
            population,
            activity,
            host,
        });
    }
    Ok(worlds)
}
fn sort_columns(worlds: &mut [World], keys: [i32; 4], directions: [i32; 4]) {
    let cmp = |a: &World, b: &World| {
        for i in 0..4 {
            let value = |w: &World| match keys[i] {
                2 => w.index,
                1 => {
                    if w.population == -1 && directions[i] == 1 {
                        2001
                    } else {
                        w.population
                    }
                }
                3 => i32::from(w.properties & 1 != 0),
                _ => w.id,
            };
            let mut order = value(a).cmp(&value(b));
            if directions[i] == 0 {
                order = order.reverse();
            }
            if order != Ordering::Equal {
                return order;
            }
        }
        Ordering::Equal
    };
    let mut ranges = vec![(0, worlds.len())];
    while let Some((start, end)) = ranges.pop() {
        if end - start < 2 {
            continue;
        }
        let pivot = worlds[(start + end - 1) / 2].clone();
        worlds.swap(start, (start + end - 1) / 2);
        let mut left = start;
        let mut right = end - 1;
        loop {
            while cmp(&worlds[right], &pivot) == Ordering::Greater {
                right -= 1;
            }
            while cmp(&worlds[left], &pivot) == Ordering::Less {
                left += 1;
            }
            if left >= right {
                break;
            }
            worlds.swap(left, right);
            left += 1;
            right -= 1;
        }
        ranges.push((right + 1, end));
        ranges.push((start, right + 1));
    }
}
