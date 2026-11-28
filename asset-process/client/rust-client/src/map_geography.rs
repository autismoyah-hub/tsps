//! Revision 239 world-map geography tiles (hy.qh).
use crate::{buffer::Reader, Error, Result};
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Decoration {
    pub id: i32,
    pub shape: u8,
    pub rotation: u8,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Layer {
    pub overlay: u16,
    pub shape: u8,
    pub rotation: u8,
    pub decorations: Vec<Decoration>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tile {
    pub underlay: u16,
    pub layers: Vec<Layer>,
}
impl Tile {
    pub fn read(r: &mut Reader<'_>, planes: u8) -> Result<Self> {
        let planes = planes.min(4) as usize;
        let flags = r.u8()?;
        let mut tile = Self {
            underlay: 0,
            layers: vec![Layer::default(); planes],
        };
        if flags == 0 {
            return Ok(tile);
        }
        if flags & 1 != 0 {
            if flags & 2 != 0 {
                let overlay = r.u16()?;
                tile.layers
                    .get_mut(0)
                    .ok_or(Error::Invalid("map overlay plane"))?
                    .overlay = overlay;
            }
            tile.underlay = r.u16()?;
            return Ok(tile);
        }
        tile.underlay = r.u16()?;
        if flags & 2 != 0 {
            for p in 0..r.u8()? as usize {
                let overlay = r.u16()?;
                if overlay != 0 {
                    let layer = tile
                        .layers
                        .get_mut(p)
                        .ok_or(Error::Invalid("map overlay plane"))?;
                    layer.overlay = overlay;
                    let info = r.u8()?;
                    layer.shape = info >> 2;
                    layer.rotation = info & 3;
                }
            }
        }
        if flags & 4 != 0 {
            for p in 0..((flags & 24) >> 3) as usize + 1 {
                let count = r.u8()?;
                if count != 0 {
                    let layer = tile
                        .layers
                        .get_mut(p)
                        .ok_or(Error::Invalid("map decoration plane"))?;
                    for _ in 0..count {
                        let id = r.u32()? as i32;
                        let info = r.u8()?;
                        layer.decorations.push(Decoration {
                            id,
                            shape: info >> 2,
                            rotation: info & 3,
                        });
                    }
                }
            }
        }
        Ok(tile)
    }
}
#[derive(Debug)]
pub struct Geography {
    pub tiles: Vec<Tile>,
}
impl Geography {
    /// Decode a complete region or the ordered chunk payloads for that destination.
    pub fn decode(
        bytes: &[u8],
        composite: &crate::map_composite::Composite,
        destination: [u16; 2],
    ) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let mut value = Self {
            tiles: vec![Tile::default(); 4096],
        };
        if let Some(region) = composite.regions.get(&destination) {
            for x in 0..64 {
                for y in 0..64 {
                    value.tiles[x * 64 + y] = Tile::read(&mut r, region.planes)?;
                }
            }
        } else {
            for chunk in composite
                .chunks
                .iter()
                .filter(|c| c.region.destination == destination)
            {
                if chunk.destination.iter().any(|&v| v >= 8) {
                    return Err(Error::Invalid("map geography chunk position"));
                }
                for x in 0..8 {
                    for y in 0..8 {
                        let px = chunk.destination[0] as usize * 8 + x;
                        let py = chunk.destination[1] as usize * 8 + y;
                        value.tiles[px * 64 + py] = Tile::read(&mut r, chunk.region.planes)?;
                    }
                }
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("map geography trailing bytes"));
        }
        Ok(value)
    }
}
