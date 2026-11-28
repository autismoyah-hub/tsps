//! Dynamic location renderable selection from GameSceneTilePaint.dc / cj.dl.
use crate::{
    buffer::{Reader, Transform},
    location_update::category,
    Error, Result,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub at: u8,
    pub shape: u8,
    pub rotation: u8,
    pub sequence: i32,
}
impl Request {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let packed = r.u8()?;
        let request = Self {
            shape: packed >> 2,
            rotation: packed & 3,
            at: r.transformed(Transform::Sub)?,
            sequence: r.u16_add(true)? as i32,
        };
        category(request.shape)?;
        if r.remaining() != 0 {
            return Err(Error::Invalid("location animation trailing bytes"));
        }
        Ok(request)
    }
    pub fn key(&self, origin: [i32; 3]) -> Result<[i32; 4]> {
        Ok([
            origin[0],
            origin[1] + ((self.at >> 4) & 7) as i32,
            origin[2] + (self.at & 7) as i32,
            category(self.shape)? as i32,
        ])
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Part {
    pub shape: u8,
    pub rotation: u8,
    pub sequence: Option<u32>,
    pub generation: u64,
}
pub fn parts(
    shape: u8,
    rotation: u8,
    sequence: Option<u32>,
    generation: u64,
) -> Result<[Option<Part>; 2]> {
    category(shape)?;
    let part = |shape, rotation| {
        Some(Part {
            shape,
            rotation,
            sequence,
            generation,
        })
    };
    Ok(match shape {
        2 => [part(2, rotation + 4), part(2, (rotation + 1) & 3)],
        4 | 5 => [part(4, rotation), None],
        6 => [part(4, rotation + 4), None],
        7 => [part(4, ((rotation + 2) & 3) + 4), None],
        8 => [part(4, rotation + 4), part(4, ((rotation + 2) & 3) + 4)],
        11 => [part(10, rotation), None],
        _ => [part(shape, rotation), None],
    })
}
/// Dynamic renderables sample terrain using their own orientation; orientations
/// 4..7 do not swap dimensions, even when their low two bits are odd.
pub fn contour_origin(
    size: [u8; 2],
    rotation: u8,
    tile: [usize; 2],
    mut height: impl FnMut(usize, usize) -> Option<i32>,
) -> crate::Result<[i32; 3]> {
    let [sx, sz] = if matches!(rotation, 1 | 3) {
        [size[1], size[0]]
    } else {
        size
    };
    let [x, z] = tile;
    let x0 = x + sx as usize / 2;
    let x1 = x + (sx as usize + 1) / 2;
    let z0 = z + sz as usize / 2;
    let z1 = z + (sz as usize + 1) / 2;
    let mut sum = 0i32;
    for (x, z) in [(x0, z1), (x0, z0), (x1, z0), (x1, z1)] {
        sum = sum.wrapping_add(
            height(x, z).ok_or(crate::Error::Invalid("dynamic contour outside scene"))?,
        );
    }
    Ok([
        x as i32 * 128 + sx as i32 * 64,
        sum >> 2,
        z as i32 * 128 + sz as i32 * 64,
    ])
}

/// Packet 30 uses an independent packed scene coordinate and the active view plane.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Specific {
    pub position: [i32; 2],
    pub request: Request,
}
impl Specific {
    pub fn decode(r: &mut Reader<'_>) -> Result<Self> {
        let packed = r.transformed(Transform::Add)?;
        let sequence = r.u16()? as i32;
        let b = r.bytes(3)?;
        let coordinate = b[0] as i32 | (b[1] as i32) << 8 | (b[2] as i32) << 16;
        category(packed >> 2)?;
        Ok(Self {
            position: [
                (coordinate >> 16) + ((coordinate >> 4) & 7),
                ((coordinate >> 8) & 255) + (coordinate & 7),
            ],
            request: Request {
                at: 0,
                shape: packed >> 2,
                rotation: packed & 3,
                sequence,
            },
        })
    }
    pub fn event(
        self,
        base: [i32; 2],
        dimensions: [i32; 2],
        plane: i32,
    ) -> Option<crate::location_update::Event> {
        if (0..2).any(|axis| self.position[axis] < 0 || self.position[axis] >= dimensions[axis] - 1)
        {
            return None;
        }
        Some(crate::location_update::Event::Animate(
            [
                plane,
                base[0] + self.position[0],
                base[1] + self.position[1],
            ],
            self.request,
        ))
    }
}

/// Scene shape 11 keeps shape-10 model selection, then applies a 45-degree
/// render rotation after animation and terrain contouring (GameIgnore.ab).
pub fn scene_vertex(shape: u8, vertex: [f32; 3]) -> [f32; 3] {
    crate::model_draw::Transform::new(if shape == 11 { 256 } else { 0 }, [0; 3])
        .expect("fixed scene orientation")
        .vertex(vertex)
}

/// GameDynamicObject.as prefers a present current clock, even when it is inactive.
/// Each clock contributes its sequence id and signed opcode16 offset; asset readiness is independent.
pub fn height_offset(current: Option<(i32, i8)>, fallback: Option<(i32, i8)>) -> i32 {
    current.or(fallback).filter(|&(id, _)| id >= 0)
        .map_or(0, |(_, offset)| offset as i32)
}
