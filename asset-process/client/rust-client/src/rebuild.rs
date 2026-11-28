//! Revision 239 world rebuilds (dy.af, uk.df, wn.ni and wn.hg).
use crate::{
    buffer::{BitReader, Reader, Transform},
    Error, Result,
};
use std::collections::HashSet;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coord {
    pub plane: u8,
    pub x: u16,
    pub y: u16,
}
impl Coord {
    pub fn unpack(value: u32) -> Self {
        Self {
            plane: ((value >> 28) & 3) as u8,
            x: ((value >> 14) & 16383) as u16,
            y: (value & 16383) as u16,
        }
    }
    pub fn pack(self) -> u32 {
        ((self.plane as u32 & 3) << 28) | ((self.x as u32 & 16383) << 14) | (self.y as u32 & 16383)
    }
}
pub struct PlayerInit {
    pub local_index: u16,
    pub local: Coord,
    pub remote_regions: Vec<Option<Coord>>,
}
impl PlayerInit {
    pub fn decode(data: &[u8], local_index: u16) -> Result<(Self, usize)> {
        if !(1..2048).contains(&local_index) {
            return Err(Error::Invalid("local player index"));
        }
        let mut r = BitReader::new(data);
        let local = Coord::unpack(r.bits(30)?);
        let mut remote_regions = vec![None; 2048];
        for (id, region) in remote_regions.iter_mut().enumerate().skip(1) {
            if id != local_index as usize {
                let raw = r.bits(18)?;
                *region = Some(Coord {
                    plane: (raw >> 16) as u8,
                    x: ((raw >> 8) & 255) as u16,
                    y: (raw & 255) as u16,
                });
            }
        }
        Ok((
            Self {
                local_index,
                local,
                remote_regions,
            },
            r.bytes_consumed(),
        ))
    }
}
pub struct Normal {
    pub world_area: i16,
    pub zone_x: u16,
    pub zone_y: u16,
    pub players: Option<PlayerInit>,
}
impl Normal {
    /// Initial login is explicit, avoiding payload-size heuristics.
    pub fn decode(data: &[u8], login_index: Option<u16>) -> Result<Self> {
        let (players, offset) = if let Some(index) = login_index {
            let (p, n) = PlayerInit::decode(data, index)?;
            (Some(p), n)
        } else {
            (None, 0)
        };
        let mut r = Reader::new(&data[offset..]);
        let world_area = r.u16_le()? as i16;
        let zone_y = r.u16_add(false)?;
        let zone_x = r.u16_add(false)?;
        if r.remaining() != 0 {
            return Err(Error::Invalid("normal rebuild trailing bytes"));
        }
        Ok(Self {
            world_area,
            zone_x,
            zone_y,
            players,
        })
    }
    pub fn base(&self) -> (i32, i32) {
        ((self.zone_x as i32 - 6) * 8, (self.zone_y as i32 - 6) * 8)
    }
    pub fn regions(&self) -> Vec<u32> {
        let mut ids = Vec::new();
        for x in (self.zone_x as i32 - 6) >> 3..=(self.zone_x as i32 + 6) >> 3 {
            for y in (self.zone_y as i32 - 6) >> 3..=(self.zone_y as i32 + 6) >> 3 {
                ids.push(((x << 8) + y) as u32);
            }
        }
        ids
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chunk {
    pub plane: u8,
    pub zone_x: u16,
    pub zone_y: u16,
    pub rotation: u8,
}
impl Chunk {
    pub fn unpack(value: u32) -> Self {
        Self {
            plane: ((value >> 24) & 3) as u8,
            zone_x: ((value >> 14) & 1023) as u16,
            zone_y: ((value >> 3) & 2047) as u16,
            rotation: ((value >> 1) & 3) as u8,
        }
    }
    pub fn region(self) -> u32 {
        ((self.zone_x as u32 >> 3) << 8) + (self.zone_y as u32 >> 3)
    }
}
pub struct Instance {
    pub zone_x: u16,
    pub zone_y: u16,
    pub reload: bool,
    pub chunks: Vec<Option<Chunk>>,
    pub regions: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct WorldEntity {
    pub base: [i32; 2],
    pub dimensions: [usize; 2],
    pub chunks: Vec<Option<Chunk>>,
    pub regions: Vec<u32>,
}
impl WorldEntity {
    pub fn decode(data: &[u8], dimensions: [usize; 2]) -> Result<Self> {
        if dimensions.iter().any(|&v| v > 104 || v % 8 != 0) {
            return Err(Error::Invalid("world entity rebuild dimensions"));
        }
        let mut r = Reader::new(data);
        let base = [r.u16()? as i32, r.u16()? as i32];
        let count = r.u16()? as usize;
        let offset = r.position();
        let mut bits = BitReader::new(&data[offset..]);
        let mut chunks = Vec::new();
        let mut regions = Vec::new();
        let mut seen = HashSet::new();
        for _ in 0..4 * (dimensions[0] / 8) * (dimensions[1] / 8) {
            let chunk = if bits.bits(1)? != 0 {
                Some(Chunk::unpack(bits.bits(26)?))
            } else {
                None
            };
            if let Some(chunk) = chunk {
                if seen.insert(chunk.region()) {
                    regions.push(chunk.region());
                }
            }
            chunks.push(chunk);
        }
        if count != regions.len() || offset + bits.bytes_consumed() != data.len() {
            return Err(Error::Invalid(
                "world entity rebuild region count or length",
            ));
        }
        Ok(Self {
            base,
            dimensions,
            chunks,
            regions,
        })
    }
    pub fn chunk(&self, plane: usize, x: usize, y: usize) -> Option<Chunk> {
        let [width, height] = self.dimensions.map(|v| v / 8);
        if plane >= 4 || x >= width || y >= height {
            None
        } else {
            self.chunks[(plane * width + x) * height + y]
        }
    }
}
impl Instance {
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let zone_y = r.u16()?;
        let zone_x = r.u16()?;
        let reload = r.transformed(Transform::Add)? == 1;
        let count = r.u16()? as usize;
        let offset = r.position();
        let mut bits = BitReader::new(&data[offset..]);
        let mut chunks = Vec::with_capacity(676);
        let mut regions = Vec::new();
        let mut seen = HashSet::new();
        for _ in 0..676 {
            let chunk = if bits.bits(1)? != 0 {
                Some(Chunk::unpack(bits.bits(26)?))
            } else {
                None
            };
            if let Some(c) = chunk {
                if seen.insert(c.region()) {
                    regions.push(c.region());
                }
            }
            chunks.push(chunk);
        }
        if regions.len() != count || offset + bits.bytes_consumed() != data.len() {
            return Err(Error::Invalid("instance rebuild region count or length"));
        }
        Ok(Self {
            zone_x,
            zone_y,
            reload,
            chunks,
            regions,
        })
    }
    pub fn base(&self) -> (i32, i32) {
        ((self.zone_x as i32 - 6) * 8, (self.zone_y as i32 - 6) * 8)
    }
    pub fn chunk(&self, plane: usize, x: usize, y: usize) -> Option<Chunk> {
        if plane >= 4 || x >= 13 || y >= 13 {
            None
        } else {
            self.chunks[plane * 169 + x * 13 + y]
        }
    }
}
pub fn rotate_tile(x: u8, y: u8, rotation: u8) -> Result<(u8, u8)> {
    if x > 7 || y > 7 || rotation > 3 {
        return Err(Error::Invalid("chunk tile rotation"));
    }
    Ok(match rotation {
        0 => (x, y),
        1 => (y, 7 - x),
        2 => (7 - x, 7 - y),
        _ => (7 - y, x),
    })
}

/// ds.ak/hu.ae: rotate an object's anchor after accounting for its footprint.
pub fn rotate_object(
    x: u8,
    y: u8,
    rotation: u8,
    size: [u8; 2],
    orientation: u8,
) -> Result<(i32, i32)> {
    if x > 7 || y > 7 || rotation > 3 || orientation > 3 || size.contains(&0) {
        return Err(Error::Invalid("chunk object rotation"));
    }
    let [mut width, mut height] = size.map(i32::from);
    if orientation & 1 != 0 {
        std::mem::swap(&mut width, &mut height);
    }
    let (x, y) = (i32::from(x), i32::from(y));
    Ok(match rotation {
        0 => (x, y),
        1 => (y, 7 - x - (width - 1)),
        2 => (7 - x - (width - 1), 7 - y - (height - 1)),
        _ => (7 - y - (height - 1), x),
    })
}
