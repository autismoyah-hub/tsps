//! Tile collision bitfields, ported from go (GameCollisionData).
use crate::{Error, Result};
pub const UNLOADED: u32 = 0x40000000;
pub const FLOOR: u32 = 0x200000;
pub const FLOOR_DECORATION: u32 = 0x40000;
pub const OBJECT: u32 = 0x100;
pub const PROJECTILE_OBJECT: u32 = 0x20000;

pub struct CollisionMap {
    width: usize,
    height: usize,
    inset: i32,
    flags: Vec<u32>,
}
impl CollisionMap {
    pub fn new(width: usize, height: usize, padded: bool) -> Result<Self> {
        let padding = if padded { 6 } else { 0 };
        let width = width
            .checked_add(padding)
            .ok_or(Error::Invalid("collision dimensions"))?;
        let height = height
            .checked_add(padding)
            .ok_or(Error::Invalid("collision dimensions"))?;
        let count = width
            .checked_mul(height)
            .ok_or(Error::Invalid("collision dimensions"))?;
        if count > 16 << 20 || width > i32::MAX as usize || height > i32::MAX as usize {
            return Err(Error::Invalid("collision size limit"));
        }
        let mut map = Self {
            width,
            height,
            inset: if padded { -1 } else { 0 },
            flags: vec![0; count],
        };
        map.reset();
        Ok(map)
    }
    pub fn reset(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.flags[x * self.height + y] = if x >= 1
                    && y >= 1
                    && x < self.width.saturating_sub(5)
                    && y < self.height.saturating_sub(5)
                {
                    UNLOADED
                } else {
                    0xffffff
                };
            }
        }
    }
    /// X-major order, matching the Java int[width][height] storage.
    pub fn flags(&self) -> &[u32] {
        &self.flags
    }
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        let x = i64::from(x) - i64::from(self.inset);
        let y = i64::from(y) - i64::from(self.inset);
        if x < 0 || y < 0 || x >= self.width as i64 || y >= self.height as i64 {
            None
        } else {
            Some(x as usize * self.height + y as usize)
        }
    }
    pub fn flag_at(&self, x: i32, y: i32) -> Option<u32> {
        self.index(x, y).map(|i| self.flags[i])
    }
    pub fn change(&mut self, x: i32, y: i32, mask: u32, add: bool) -> Result<()> {
        let i = self
            .index(x, y)
            .ok_or(Error::Invalid("collision position"))?;
        if add {
            self.flags[i] |= mask;
        } else {
            self.flags[i] &= !mask;
        }
        Ok(())
    }
    pub fn wall(
        &mut self,
        x: i32,
        y: i32,
        shape: u8,
        rotation: u8,
        projectile: bool,
        add: bool,
    ) -> Result<()> {
        if shape > 3 || rotation > 3 {
            return Err(Error::Invalid("collision wall shape/rotation"));
        }
        const STRAIGHT: [(i32, i32, u32, u32); 4] = [
            (-1, 0, 128, 8),
            (0, 1, 2, 32),
            (1, 0, 8, 128),
            (0, -1, 32, 2),
        ];
        const DIAGONAL: [(i32, i32, u32, u32); 4] = [
            (-1, 1, 1, 16),
            (1, 1, 4, 64),
            (1, -1, 16, 1),
            (-1, -1, 64, 4),
        ];
        let mut changes = Vec::with_capacity(4);
        let mut edge = |entry: (i32, i32, u32, u32)| -> Result<()> {
            let (dx, dy, own, other) = entry;
            let nx = x
                .checked_add(dx)
                .ok_or(Error::Invalid("collision coordinate overflow"))?;
            let ny = y
                .checked_add(dy)
                .ok_or(Error::Invalid("collision coordinate overflow"))?;
            changes.push((
                self.index(x, y)
                    .ok_or(Error::Invalid("collision wall position"))?,
                own,
            ));
            changes.push((
                self.index(nx, ny)
                    .ok_or(Error::Invalid("collision wall neighbor"))?,
                other,
            ));
            Ok(())
        };
        match shape {
            0 => edge(STRAIGHT[rotation as usize])?,
            1 | 3 => edge(DIAGONAL[rotation as usize])?,
            2 => {
                edge(STRAIGHT[rotation as usize])?;
                edge(STRAIGHT[((rotation + 1) & 3) as usize])?;
            }
            _ => unreachable!(),
        }
        for (index, mask) in changes {
            let mask = if projectile { mask | (mask << 9) } else { mask };
            if add {
                self.flags[index] |= mask;
            } else {
                self.flags[index] &= !mask;
            }
        }
        Ok(())
    }
    /// Rectangle footprints are clipped to the grid as in go.ac.
    pub fn rectangle(
        &mut self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        projectile: bool,
        add: bool,
    ) {
        let x = i64::from(x) - i64::from(self.inset);
        let y = i64::from(y) - i64::from(self.inset);
        let mask = OBJECT | if projectile { PROJECTILE_OBJECT } else { 0 };
        for dx in x.max(0)..(x + i64::from(width)).min(self.width as i64) {
            for dy in y.max(0)..(y + i64::from(height)).min(self.height as i64) {
                let index = dx as usize * self.height + dy as usize;
                if add {
                    self.flags[index] |= mask;
                } else {
                    self.flags[index] &= !mask;
                }
            }
        }
    }
}
