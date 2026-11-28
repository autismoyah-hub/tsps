//! Four-plane 64x64 terrain blocks, from lb.ap and px.am/kx.ar/oe.ad.
use crate::{buffer::Reader, Error, Result};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Tile {
    pub height: i32,
    pub overlay: i16,
    pub overlay_shape: u8,
    pub overlay_rotation: u8,
    pub flags: u8,
    pub underlay: u16,
}
pub struct Region {
    /// Indexed by plane * 4096 + x * 64 + y, matching the gamepack's arrays.
    pub tiles: Vec<Tile>,
    pub extension_flags: u8,
    /// None denotes procedural/default height; Some is the normalized explicit delta.
    pub height_rules: Vec<Option<u8>>,
}
impl Region {
    pub fn decode(data: &[u8], base_x: i32, base_y: i32) -> Result<Self> {
        let mut r = Reader::new(data);
        let mut tiles = vec![Tile::default(); 4 * 64 * 64];
        let mut height_rules = vec![None; tiles.len()];
        for plane in 0..4 {
            for x in 0..64 {
                for y in 0..64 {
                    let i = plane * 4096 + x * 64 + y;
                    let below = if plane == 0 {
                        0
                    } else {
                        tiles[i - 4096].height
                    };
                    let t = &mut tiles[i];
                    loop {
                        match r.u16()? {
                            0 => {
                                t.height = if plane == 0 {
                                    procedural_height(
                                        base_x.wrapping_add(x as i32),
                                        base_y.wrapping_add(y as i32),
                                    )
                                } else {
                                    below - 240
                                };
                                break;
                            }
                            1 => {
                                let mut height = r.u8()? as i32;
                                if height == 1 {
                                    height = 0;
                                }
                                height_rules[i] = Some(height as u8);
                                t.height = below - 8 * height;
                                break;
                            }
                            opcode @ 2..=49 => {
                                t.overlay = r.u16()? as i16;
                                t.overlay_shape = ((opcode - 2) / 4) as u8;
                                t.overlay_rotation = ((opcode - 2) & 3) as u8;
                            }
                            opcode @ 50..=81 => {
                                t.flags = (opcode - 49) as u8;
                            }
                            opcode => {
                                t.underlay = opcode - 81;
                            }
                        }
                    }
                }
            }
        }
        // ds.bf consumes an optional extension byte and skips an extra plane.
        let extension_flags = if r.remaining() == 0 { 0 } else { r.u8()? };
        if extension_flags & !1 != 0 {
            return Err(Error::Unsupported("terrain extension flags"));
        }
        if extension_flags & 1 != 0 {
            for _ in 0..4096 {
                loop {
                    match r.u16()? {
                        0 => break,
                        1 => {
                            r.u8()?;
                            break;
                        }
                        2..=49 => {
                            r.u16()?;
                        }
                        _ => {}
                    }
                }
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("terrain trailing bytes"));
        }
        Ok(Self {
            tiles,
            extension_flags,
            height_rules,
        })
    }
    pub fn tile(&self, plane: usize, x: usize, y: usize) -> Option<&Tile> {
        if plane >= 4 || x >= 64 || y >= 64 {
            None
        } else {
            self.tiles.get(plane * 4096 + x * 64 + y)
        }
    }
}

fn noise(x: i32, y: i32) -> i32 {
    let n = x.wrapping_add(y.wrapping_mul(57));
    let n = n ^ n.wrapping_shl(13);
    let value = n
        .wrapping_mul(n)
        .wrapping_mul(15731)
        .wrapping_add(789221)
        .wrapping_mul(n)
        .wrapping_add(1376312589)
        & 0x7fffffff;
    (value >> 19) & 255
}
fn smooth(x: i32, y: i32) -> i32 {
    let (xm, xp, ym, yp) = (
        x.wrapping_sub(1),
        x.wrapping_add(1),
        y.wrapping_sub(1),
        y.wrapping_add(1),
    );
    let corners = noise(xm, ym) + noise(xp, ym) + noise(xm, yp) + noise(xp, yp);
    let sides = noise(xm, y) + noise(xp, y) + noise(x, ym) + noise(x, yp);
    corners / 16 + sides / 8 + noise(x, y) / 4
}
fn interpolate(a: i32, b: i32, fraction: i32, scale: i32) -> i32 {
    // Exact integer entries used from fk.ap for scales 1, 2 and 4.
    let cosine = match fraction * 1024 / scale {
        0 => 65536,
        256 => 46340,
        512 => 0,
        768 => -46340,
        _ => unreachable!(),
    };
    let weight = (65536 - cosine) >> 1;
    ((a * (65536 - weight)) >> 16) + ((b * weight) >> 16)
}
fn interpolated(x: i32, y: i32, scale: i32) -> i32 {
    // Java integer division truncates toward zero, including negative inputs.
    let (ix, iy) = (x / scale, y / scale);
    let (fx, fy) = (x & (scale - 1), y & (scale - 1));
    let a = interpolate(smooth(ix, iy), smooth(ix.wrapping_add(1), iy), fx, scale);
    let b = interpolate(
        smooth(ix, iy.wrapping_add(1)),
        smooth(ix.wrapping_add(1), iy.wrapping_add(1)),
        fx,
        scale,
    );
    interpolate(a, b, fy, scale)
}
pub fn procedural_height(x: i32, y: i32) -> i32 {
    let (x, y) = (x.wrapping_add(932731), y.wrapping_add(556238));
    let value = interpolated(x.wrapping_add(45365), y.wrapping_add(91923), 4) - 128
        + ((interpolated(x.wrapping_add(10294), y.wrapping_add(37821), 2) - 128) >> 1)
        + ((interpolated(x, y, 1) - 128) >> 2);
    -8 * ((value as f64 * 0.3) as i32 + 35).clamp(10, 60)
}

#[derive(Debug, PartialEq, Eq)]
pub struct Location {
    pub object: u32,
    pub plane: u8,
    pub x: u8,
    pub y: u8,
    pub shape: u8,
    pub rotation: u8,
}
/// Delta-coded object placements from map group file 1. This decodes placement
/// records; instantiating models and applying collision are separate operations.
pub fn locations(data: &[u8], max_objects: usize) -> Result<Vec<Location>> {
    let mut r = Reader::new(data);
    let mut object = -1i64;
    let mut result = Vec::new();
    loop {
        let delta = r.incremental_smart()?;
        if delta == 0 {
            break;
        }
        object += delta as i64;
        if object > i32::MAX as i64 {
            return Err(Error::Invalid("map object id overflow"));
        }
        let mut packed = 0u32;
        loop {
            let delta = r.unsigned_smart()? as u32;
            if delta == 0 {
                break;
            }
            packed = packed
                .checked_add(delta - 1)
                .ok_or(Error::Invalid("map position overflow"))?;
            if packed >= 4 * 4096 {
                return Err(Error::Invalid("map object plane"));
            }
            let attributes = r.u8()?;
            if result.len() >= max_objects {
                return Err(Error::Invalid("map object count limit"));
            }
            result.push(Location {
                object: object as u32,
                plane: (packed >> 12) as u8,
                x: ((packed >> 6) & 63) as u8,
                y: (packed & 63) as u8,
                shape: attributes >> 2,
                rotation: attributes & 3,
            });
        }
    }
    if r.remaining() != 0 {
        return Err(Error::Invalid("map locations trailing bytes"));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer::Writer;
    #[test]
    fn chained_object_ids_and_planes() {
        let mut w = Writer::new();
        w.unsigned_smart(32767).unwrap();
        w.unsigned_smart(32767).unwrap();
        w.unsigned_smart(100).unwrap();
        w.unsigned_smart(3 * 4096 + 63 * 64 + 11 + 1).unwrap();
        w.u8(22 * 4 + 3);
        w.unsigned_smart(1).unwrap();
        w.u8(4);
        w.u8(0);
        w.u8(0);
        let bytes = w.into_vec();
        let objects = locations(&bytes, 100).unwrap();
        assert_eq!(
            objects[0],
            Location {
                object: 65633,
                plane: 3,
                x: 63,
                y: 11,
                shape: 22,
                rotation: 3
            }
        );
        assert_eq!(
            objects[1],
            Location {
                object: 65633,
                plane: 3,
                x: 63,
                y: 11,
                shape: 1,
                rotation: 0
            }
        );
        assert!(locations(&bytes, 1).is_err());
        assert!(locations(&bytes[..bytes.len() - 1], 100).is_err());
    }
}
