//! Java map icon grouping/iteration and nearest-coordinate selection.
use crate::{Error, Result};
pub struct Index<T> {
    capacity: usize,
    groups: Vec<(i32, Vec<T>)>,
}
impl<T> Default for Index<T> {
    fn default() -> Self {
        Self {
            capacity: 16,
            groups: Vec::new(),
        }
    }
}
fn hash(value: i32) -> u32 {
    let value = value as u32;
    value ^ (value >> 16)
}
impl<T> Index<T> {
    pub fn rebuild(&mut self, icons: impl IntoIterator<Item = (i32, T)>) -> Result<()> {
        self.groups.clear();
        for (id, icon) in icons {
            if id < 0 {
                continue;
            }
            if let Some((_, values)) = self.groups.iter_mut().find(|(key, _)| *key == id) {
                values.push(icon);
            } else {
                let bucket = hash(id) as usize & (self.capacity - 1);
                let chain = self
                    .groups
                    .iter()
                    .filter(|(key, _)| hash(*key) as usize & (self.capacity - 1) == bucket)
                    .count();
                if chain >= 8 {
                    if self.capacity < 64 {
                        self.capacity *= 2;
                    } else {
                        return Err(Error::Unsupported("map element HashMap tree bin"));
                    }
                }
                self.groups.push((id, vec![icon]));
                if self.groups.len() > self.capacity * 3 / 4 {
                    self.capacity *= 2;
                }
            }
        }
        self.groups
            .sort_by_key(|(id, _)| hash(*id) as usize & (self.capacity - 1));
        Ok(())
    }
    pub fn elements(&self) -> impl Iterator<Item = (i32, &T)> {
        self.groups
            .iter()
            .flat_map(|(id, values)| values.iter().map(move |value| (*id, value)))
    }
    pub fn get(&self, id: i32) -> Option<&[T]> {
        self.groups
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, values)| values.as_slice())
    }
}
pub fn nearest(point: [i32; 3], icons: impl IntoIterator<Item = [i32; 3]>) -> Option<[i32; 3]> {
    let mut best = None;
    let mut distance = -1;
    for coord in icons {
        let dx = coord[1].wrapping_sub(point[1]);
        let dy = coord[2].wrapping_sub(point[2]);
        let d = dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy));
        if d == 0 {
            return Some(coord);
        }
        if best.is_none() || d < distance {
            best = Some(coord);
            distance = d;
        }
    }
    best
}
