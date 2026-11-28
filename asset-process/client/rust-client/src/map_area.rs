//! Revision239 GameWorldMapData and its four translation-section formats.
use crate::{buffer::Reader, Error, Result};
#[derive(Clone, Debug)]
pub struct Section {
    pub plane: i32,
    pub planes: i32,
    pub source: [i32; 4],
    pub destination: [i32; 4],
    bounds: [i32; 4],
}
fn contains(rect: [i32; 4], x: i32, y: i32) -> bool {
    x >= rect[0] && y >= rect[1] && x <= rect[2] && y <= rect[3]
}
impl Section {
    fn decode(r: &mut Reader<'_>) -> Result<Self> {
        let kind = r.u8()?;
        let plane = r.u8()? as i32;
        let planes = r.u8()? as i32;
        let (source, destination, bounds) = match kind {
            0 => {
                let mut v = [0; 8];
                for x in &mut v {
                    *x = r.u16()? as i32;
                }
                (
                    [v[0] * 64, v[1] * 64, v[2] * 64 + 63, v[3] * 64 + 63],
                    [v[4] * 64, v[5] * 64, v[6] * 64 + 63, v[7] * 64 + 63],
                    [v[4], v[5], v[6], v[7]],
                )
            }
            1 => {
                let mut v = [0; 4];
                for x in &mut v {
                    *x = r.u16()? as i32;
                }
                (
                    [v[0] * 64, v[1] * 64, v[0] * 64 + 63, v[1] * 64 + 63],
                    [v[2] * 64, v[3] * 64, v[2] * 64 + 63, v[3] * 64 + 63],
                    [v[2], v[3], v[2], v[3]],
                )
            }
            2 | 3 => {
                let mut regions = [0; 4];
                let mut low = [0; 4];
                let mut high = [0; 4];
                for i in 0..4 {
                    regions[i] = r.u16()? as i32;
                    low[i] = r.u8()? as i32;
                    high[i] = if kind == 2 { r.u8()? as i32 } else { low[i] };
                }
                (
                    [
                        regions[0] * 64 + low[0] * 8,
                        regions[1] * 64 + low[1] * 8,
                        regions[0] * 64 + high[0] * 8 + 7,
                        regions[1] * 64 + high[1] * 8 + 7,
                    ],
                    [
                        regions[2] * 64 + low[2] * 8,
                        regions[3] * 64 + low[3] * 8,
                        regions[2] * 64 + high[2] * 8 + 7,
                        regions[3] * 64 + high[3] * 8 + 7,
                    ],
                    [regions[2], regions[3], regions[2], regions[3]],
                )
            }
            _ => return Err(Error::Invalid("world map section kind")),
        };
        Ok(Self {
            plane,
            planes,
            source,
            destination,
            bounds,
        })
    }
    pub fn contains_coord(&self, plane: i32, x: i32, y: i32) -> bool {
        plane >= self.plane && plane < self.plane + self.planes && contains(self.source, x, y)
    }
    pub fn contains_position(&self, x: i32, y: i32) -> bool {
        contains(self.destination, x, y)
    }
}
#[derive(Clone, Debug)]
pub struct Area {
    pub id: i32,
    pub internal_name: String,
    pub name: String,
    pub origin: [i32; 3],
    pub background: i32,
    pub extra: i32,
    pub main: bool,
    pub zoom: u8,
    pub bounds: [i32; 4],
    pub sections: Vec<Section>,
}
impl Area {
    pub fn decode(id: i32, bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let internal_name = r.string()?;
        let name = r.string()?;
        let packed = r.u32()? as i32;
        let origin = if packed == -1 {
            [-1, 0, 0]
        } else {
            [(packed >> 28) & 3, (packed >> 14) & 16383, packed & 16383]
        };
        let background = r.u32()? as i32;
        let extra = r.u32()? as i32;
        r.u8()?;
        let main = r.u8()? == 1;
        let zoom = r.u8()?;
        let count = r.u8()?;
        let mut sections = Vec::with_capacity(count as usize);
        let mut bounds = [i32::MAX, i32::MAX, 0, 0];
        for _ in 0..count {
            let section = Section::decode(&mut r)?;
            for i in 0..2 {
                bounds[i] = bounds[i].min(section.bounds[i]);
                bounds[i + 2] = bounds[i + 2].max(section.bounds[i + 2]);
            }
            sections.push(section);
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("world map area trailing bytes"));
        }
        Ok(Self {
            id,
            internal_name,
            name,
            origin,
            background,
            extra,
            main,
            zoom,
            bounds,
            sections,
        })
    }
    pub fn contains_coord(&self, plane: i32, x: i32, y: i32) -> bool {
        self.sections.iter().any(|s| s.contains_coord(plane, x, y))
    }
    pub fn contains_position(&self, x: i32, y: i32) -> bool {
        contains(self.bounds, x / 64, y / 64)
            && self.sections.iter().any(|s| s.contains_position(x, y))
    }
    pub fn position(&self, plane: i32, x: i32, y: i32) -> Option<[i32; 2]> {
        self.sections
            .iter()
            .find(|s| s.contains_coord(plane, x, y))
            .map(|s| {
                [
                    x + s.destination[0] - s.source[0],
                    y + s.destination[1] - s.source[1],
                ]
            })
    }
    pub fn coord(&self, x: i32, y: i32) -> Option<[i32; 3]> {
        self.sections
            .iter()
            .find(|s| s.contains_position(x, y))
            .map(|s| {
                [
                    s.plane,
                    x + s.source[0] - s.destination[0],
                    y + s.source[1] - s.destination[1],
                ]
            })
    }
}

/// The239 archive uses fixed group IDs (hp.ag), not group-name hashes.
pub fn load_all(
    cache: &mut crate::disk_cache::DiskCache,
) -> Result<std::collections::BTreeMap<i32, Area>> {
    cache
        .group(19, 0, [0; 4])?
        .into_iter()
        .map(|(id, raw)| Area::decode(id as i32, &raw).map(|a| (id as i32, a)))
        .collect()
}
pub fn is_metadata_query(op: u16) -> bool {
    matches!(op, 6601 | 6611..=6614 | 6621)
}
pub fn metadata_query(
    op: u16,
    stack: &mut crate::vm::Stacks,
    areas: &std::collections::BTreeMap<i32, Area>,
) -> Result<bool> {
    if !is_metadata_query(op) {
        return Ok(false);
    }
    let coord = if op == 6621 {
        Some(stack.pop_int()?)
    } else {
        None
    };
    let id = stack.pop_int()?;
    let area = areas.get(&id);
    match op {
        6601 => stack.push_object(crate::vm::Object::String(
            area.map_or_else(String::new, |a| a.name.clone()),
        ))?,
        6611 => stack.push_int(area.map_or(0, |a| {
            let [p, x, y] = a.origin;
            if p < 0 {
                -1
            } else {
                p << 28 | x << 14 | y
            }
        }))?,
        6612 => {
            for i in 0..2 {
                stack.push_int(area.map_or(0, |a| {
                    a.bounds[i + 2]
                        .wrapping_sub(a.bounds[i])
                        .wrapping_add(1)
                        .wrapping_mul(64)
                }))?;
            }
        }
        6613 => {
            for i in 0..4 {
                stack.push_int(area.map_or(0, |a| {
                    a.bounds[i]
                        .wrapping_mul(64)
                        .wrapping_add(if i >= 2 { 63 } else { 0 })
                }))?;
            }
        }
        6614 => stack.push_int(area.map_or(-1, |a| a.zoom as i32))?,
        6621 => {
            let c = coord.unwrap();
            let [p, x, y] = if c == -1 {
                [-1, 0, 0]
            } else {
                [(c >> 28) & 3, (c >> 14) & 16383, c & 16383]
            };
            stack.push_int(i32::from(area.is_some_and(|a| a.contains_coord(p, x, y))))?;
        }
        _ => unreachable!(),
    }
    Ok(true)
}
