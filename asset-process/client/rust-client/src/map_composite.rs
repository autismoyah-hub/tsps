//! Composite-map region/chunk references and static icons (hk.da/dt).
use crate::{buffer::Reader, Error, Result};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Region {
    pub plane: u8,
    pub planes: u8,
    pub source: [u16; 2],
    pub destination: [u16; 2],
}
impl Region {
    fn read(r: &mut Reader<'_>) -> Result<Self> {
        Ok(Self {
            plane: r.u8()?,
            planes: r.u8()?,
            source: [r.u16()?, r.u16()?],
            destination: [r.u16()?, r.u16()?],
        })
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chunk {
    pub region: Region,
    pub source: [u8; 2],
    pub destination: [u8; 2],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Icon {
    pub element: i32,
    pub coord: [i32; 3],
}
#[derive(Default, Debug)]
pub struct Composite {
    pub regions: std::collections::BTreeMap<[u16; 2], Region>,
    pub chunks: Vec<Chunk>,
    pub icons: Vec<Icon>,
}
fn element(r: &mut Reader<'_>) -> Result<i32> {
    let head = r.u8()?;
    if head & 128 != 0 {
        Ok((((head as u32 & 127) << 24) | r.u24()?) as i32)
    } else {
        let value = ((head as u16) << 8) | r.u8()? as u16;
        Ok(if value == 32767 { -1 } else { value as i32 })
    }
}
impl Composite {
    pub fn decode(bytes: &[u8], members: bool) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let mut value = Self::default();
        for _ in 0..r.u16()? {
            if r.u8()? != 0 {
                return Err(Error::Invalid("map region tag"));
            }
            let region = Region::read(&mut r)?;
            value.regions.entry(region.destination).or_insert(region);
        }
        for _ in 0..r.u16()? {
            if r.u8()? != 1 {
                return Err(Error::Invalid("map chunk tag"));
            }
            let plane = r.u8()?;
            let planes = r.u8()?;
            let source_region = [r.u16()?, r.u16()?];
            let source = [r.u8()?, r.u8()?];
            let destination_region = [r.u16()?, r.u16()?];
            let destination = [r.u8()?, r.u8()?];
            value.chunks.push(Chunk {
                region: Region {
                    plane,
                    planes,
                    source: source_region,
                    destination: destination_region,
                },
                source,
                destination,
            });
        }
        for _ in 0..r.u16()? {
            let element = element(&mut r)?;
            let packed = r.u32()? as i32;
            let restricted = r.u8()? == 1;
            if members || !restricted {
                value.icons.push(Icon {
                    element,
                    coord: if packed == -1 {
                        [-1, 0, 0]
                    } else {
                        [(packed >> 28) & 3, (packed >> 14) & 16383, packed & 16383]
                    },
                });
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("map composite trailing bytes"));
        }
        Ok(value)
    }
}
