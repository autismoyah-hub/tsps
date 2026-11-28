//! Archive 22 cached skeletal animations, matching sn.nm's channel layout.
use crate::{animation::Skeleton, buffer::Reader, curve::Curve, Error, Result};
#[derive(Clone, Debug)]
pub struct Channel {
    pub kind: u8,
    pub target: usize,
    pub component: usize,
    pub curve: Curve,
}
#[derive(Clone, Debug)]
pub struct CachedAnimation {
    pub version: u8,
    pub skeleton: u16,
    pub range: [u16; 2],
    pub pose: u8,
    pub channels: Vec<Channel>,
    pub has_alpha: bool,
}
impl CachedAnimation {
    pub fn decode(data: &[u8], skeleton: &Skeleton) -> Result<Self> {
        let mut r = Reader::new(data);
        let version = r.u8()?;
        let id = r.u16()?;
        if id != skeleton.id {
            return Err(Error::Invalid("cached animation skeleton"));
        }
        let range = [r.u16()?, r.u16()?];
        let pose = r.u8()?;
        if pose >= skeleton.poses {
            return Err(Error::Invalid("cached animation pose"));
        }
        let count = r.u16()?;
        let mut channels = Vec::new();
        let mut has_alpha = false;
        for _ in 0..count {
            let kind = r.u8()?;
            let target = r.signed_smart()?;
            let channel = r.u8()?;
            let component = match channel {
                1..=9 => channel - 1,
                10..=15 => channel - 10,
                16 => 0,
                _ => return Err(Error::Invalid("cached animation channel")),
            } as usize;
            let curve = Curve::decode(&mut r)?;
            let max = if kind == 1 {
                skeleton.bones.len()
            } else {
                skeleton.labels.len()
            };
            if target < 0 || target as usize >= max {
                return Err(Error::Invalid("cached animation target"));
            }
            has_alpha |= kind == 4;
            if component < if kind == 1 { 9 } else { 1 } {
                channels.push(Channel {
                    kind,
                    target: target as usize,
                    component,
                    curve,
                });
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("cached animation trailing bytes"));
        }
        // sn.nm stores one index per slot; a later channel replaces the earlier one.
        let mut slots = std::collections::HashSet::new();
        channels.reverse();
        channels.retain(|c| slots.insert((c.kind == 1, c.target, c.component)));
        channels.reverse();
        Ok(Self {
            version,
            skeleton: id,
            range,
            pose,
            channels,
            has_alpha,
        })
    }
}
