//! Collision routing from gf.om/as/ax/tg, including nearest-tile fallback.
use crate::{collision::CollisionMap, Error, Result};
use std::collections::VecDeque;
const W: u32 = 1076101384;
const E: u32 = 1076101504;
const S: u32 = 1076101378;
const N: u32 = 1076101408;
const SW: u32 = 1076101390;
const SE: u32 = 1076101507;
const NW: u32 = 1076101432;
const NE: u32 = 1076101600;
const WE: u32 = 1076101438;
const EE: u32 = 1076101603;
const SS: u32 = 1076101519;
const NN: u32 = 1076101624;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    pub reached: bool,
    pub destination: [i32; 2],
    pub steps: Vec<[i32; 2]>,
    pub waypoints: Vec<[i32; 2]>,
}
#[derive(Clone, Copy, Debug)]
pub struct Search {
    pub width: usize,
    pub height: usize,
    pub nearest: bool,
    pub destination_size: [i32; 2],
}
impl Default for Search {
    fn default() -> Self {
        Self {
            width: 32,
            height: 32,
            nearest: true,
            destination_size: [1, 1],
        }
    }
}
/// Check the newly occupied edges of an actor's square footprint.
pub fn can_step(map: &CollisionMap, [x, y]: [i32; 2], size: i32, [dx, dy]: [i32; 2]) -> bool {
    if size < 1
        || size > 128
        || !(-1..=1).contains(&dx)
        || !(-1..=1).contains(&dy)
        || (dx == 0 && dy == 0)
        || !(-1_000_000..=1_000_000).contains(&x)
        || !(-1_000_000..=1_000_000).contains(&y)
    {
        return false;
    }
    let clear = |x, y, mask| map.flag_at(x, y).is_some_and(|flags| flags & mask == 0);
    if size == 1 {
        return match (dx, dy) {
            (-1, 0) => clear(x - 1, y, W),
            (1, 0) => clear(x + 1, y, E),
            (0, -1) => clear(x, y - 1, S),
            (0, 1) => clear(x, y + 1, N),
            (-1, -1) => clear(x - 1, y - 1, SW) && clear(x - 1, y, W) && clear(x, y - 1, S),
            (1, -1) => clear(x + 1, y - 1, SE) && clear(x + 1, y, E) && clear(x, y - 1, S),
            (-1, 1) => clear(x - 1, y + 1, NW) && clear(x - 1, y, W) && clear(x, y + 1, N),
            (1, 1) => clear(x + 1, y + 1, NE) && clear(x + 1, y, E) && clear(x, y + 1, N),
            _ => false,
        };
    }
    match (dx, dy) {
        (-1, 0) => {
            clear(x - 1, y, SW)
                && clear(x - 1, y + size - 1, NW)
                && (1..size - 1).all(|i| clear(x - 1, y + i, WE))
        }
        (1, 0) => {
            clear(x + size, y, SE)
                && clear(x + size, y + size - 1, NE)
                && (1..size - 1).all(|i| clear(x + size, y + i, EE))
        }
        (0, -1) => {
            clear(x, y - 1, SW)
                && clear(x + size - 1, y - 1, SE)
                && (1..size - 1).all(|i| clear(x + i, y - 1, SS))
        }
        (0, 1) => {
            clear(x, y + size, NW)
                && clear(x + size - 1, y + size, NE)
                && (1..size - 1).all(|i| clear(x + i, y + size, NN))
        }
        (-1, -1) => {
            clear(x - 1, y - 1, SW)
                && (1..size).all(|i| clear(x - 1, y - 1 + i, WE) && clear(x - 1 + i, y - 1, SS))
        }
        (1, -1) => {
            clear(x + size, y - 1, SE)
                && (1..size).all(|i| clear(x + size, y - 1 + i, EE) && clear(x + i, y - 1, SS))
        }
        (-1, 1) => {
            clear(x - 1, y + size, NW)
                && (1..size).all(|i| clear(x - 1, y + i, WE) && clear(x - 1 + i, y + size, NN))
        }
        (1, 1) => {
            clear(x + size, y + size, NE)
                && (1..size).all(|i| clear(x + i, y + size, NN) && clear(x + size, y + i, EE))
        }
        _ => false,
    }
}
impl Search {
    pub fn find(
        &self,
        map: &CollisionMap,
        start: [i32; 2],
        size: i32,
        target: [i32; 2],
    ) -> Result<Option<Route>> {
        if self.width < 2
            || self.height < 2
            || self.width > 512
            || self.height > 512
            || size < 1
            || size as usize > self.width / 2
            || size as usize > self.height / 2
            || start
                .into_iter()
                .chain(target)
                .any(|v| !(-1_000_000..=1_000_000).contains(&v))
            || self
                .destination_size
                .iter()
                .any(|&v| !(1..=128).contains(&v))
        {
            return Err(Error::Invalid("route dimensions or coordinate"));
        }
        let origin = [
            start[0] - (self.width / 2) as i32,
            start[1] - (self.height / 2) as i32,
        ];
        let index = |p: [i32; 2]| {
            let x = p[0] - origin[0];
            let y = p[1] - origin[1];
            if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
                Some(x as usize * self.height + y as usize)
            } else {
                None
            }
        };
        let mut directions = vec![0u8; self.width * self.height];
        let mut distance = vec![u32::MAX; directions.len()];
        let i = index(start).unwrap();
        directions[i] = 99;
        distance[i] = 0;
        let mut queue = VecDeque::from([start]);
        let mut reached = false;
        let mut end = start;
        const MOVES: [([i32; 2], u8); 8] = [
            ([-1, 0], 2),
            ([1, 0], 8),
            ([0, -1], 1),
            ([0, 1], 4),
            ([-1, -1], 3),
            ([1, -1], 9),
            ([-1, 1], 6),
            ([1, 1], 12),
        ];
        while let Some(p) = queue.pop_front() {
            end = p;
            if p == target {
                reached = true;
                break;
            }
            let d = distance[index(p).unwrap()] + 1;
            for (delta, back) in MOVES {
                let next = [p[0] + delta[0], p[1] + delta[1]];
                if next[0] - origin[0] > self.width as i32 - size
                    || next[1] - origin[1] > self.height as i32 - size
                {
                    continue;
                }
                if let Some(i) = index(next) {
                    if directions[i] == 0 && can_step(map, p, size, delta) {
                        directions[i] = back;
                        distance[i] = d;
                        queue.push_back(next);
                    }
                }
            }
        }
        if !reached {
            if !self.nearest {
                return Ok(None);
            }
            let mut best = None;
            for x in target[0] - 10..=target[0] + 10 {
                for y in target[1] - 10..=target[1] + 10 {
                    let Some(i) = index([x, y]) else {
                        continue;
                    };
                    if distance[i] >= 100 {
                        continue;
                    }
                    let dx = (target[0] - x)
                        .max(x - (target[0] + self.destination_size[0] - 1))
                        .max(0);
                    let dy = (target[1] - y)
                        .max(y - (target[1] + self.destination_size[1] - 1))
                        .max(0);
                    let score = (dx * dx + dy * dy, distance[i]);
                    if best.is_none_or(|s| score < s) {
                        best = Some(score);
                        end = [x, y];
                    }
                }
            }
            if best.is_none() {
                return Ok(None);
            }
        }
        let mut route = Route {
            reached,
            destination: end,
            steps: Vec::new(),
            waypoints: Vec::new(),
        };
        if end == start {
            return Ok(Some(route));
        }
        let mut current = end;
        let mut previous = directions[index(end).unwrap()];
        route.waypoints.push(end);
        while current != start {
            route.steps.push(current);
            let direction =
                directions[index(current).ok_or(Error::Invalid("route predecessor"))?];
            if direction == 0 {
                return Err(Error::Invalid("route predecessor"));
            }
            if direction != previous {
                previous = direction;
                route.waypoints.push(current);
            }
            if direction & 2 != 0 {
                current[0] += 1;
            } else if direction & 8 != 0 {
                current[0] -= 1;
            }
            if direction & 1 != 0 {
                current[1] += 1;
            } else if direction & 4 != 0 {
                current[1] -= 1;
            }
        }
        route.steps.reverse();
        route.waypoints.reverse();
        Ok(Some(route))
    }
}
