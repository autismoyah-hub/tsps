//! Revision 239 player extension blocks, in wire execution order.
use crate::{
    actor_update::{Block, Headbar, Hit},
    buffer::{Reader, Transform},
    huffman::Huffman,
    item::Definitions,
    player_appearance, Error, Result,
};
#[derive(Clone, Debug)]
pub enum Update {
    Actor(Block),
    Actions([String; 3]),
    Reset(u8),
    MoveSpeed(i8),
    TemporarySpeed(i8),
    /// Legacy extension bytes discarded by dy.ab in revision 239.
    FaceEntity(u32),
    Appearance(player_appearance::Update),
    Chat(Chat),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Chat {
    pub color: u8,
    pub effect: u8,
    pub icon: u8,
    pub automatic: bool,
    pub text: String,
    pub pattern: Vec<u8>,
}
fn byte(r: &mut Reader<'_>, t: Transform) -> Result<u8> {
    r.transformed(t)
}
pub fn decode(r: &mut Reader<'_>, items: &Definitions, huffman: &Huffman) -> Result<Vec<Update>> {
    use Transform::{Add, Neg, Sub};
    let mut mask = r.u8()? as u32;
    if mask & 8 != 0 {
        mask |= (r.u8()? as u32) << 8;
    }
    if mask & 0x800 != 0 {
        mask |= (r.u8()? as u32) << 16;
    }
    if mask & !0x1f7fff != 0 {
        return Err(Error::Invalid("unknown player extension mask"));
    }
    let mut out = vec![];
    if mask & 0x2000 != 0 {
        out.push(Update::Actions([r.string()?, r.string()?, r.string()?]));
    }
    if mask & 0x40000 != 0 {
        let count = byte(r, Sub)?;
        let mut hits = vec![];
        for _ in 0..count {
            hits.push(Hit {
                kind: r.unsigned_smart()?,
                value: r.unsigned_smart()?,
                delay: r.unsigned_smart()?,
                limit: r.unsigned_smart()?,
            });
        }
        out.push(Update::Actor(Block::Hits(hits)));
    }
    if mask & 2 != 0 {
        out.push(Update::Reset(byte(r, Neg)?));
    }
    if mask & 1 != 0 {
        let flags = byte(r, Neg)?;
        out.push(Update::Actor(Block::Face(crate::npc_info::face_flags(
            r, flags,
        )?)));
    }
    if mask & 128 != 0 {
        let mut bytes = r.bytes(4)?.to_vec();
        let size = byte(r, Sub)? as usize;
        bytes.push(size as u8);
        bytes.extend_from_slice(r.bytes(size)?);
        out.push(Update::Actor(Block::Discarded { mask: 128, bytes }));
    }
    if mask & 0x400 != 0 {
        out.push(Update::MoveSpeed(byte(r, Neg)? as i8));
    }
    if mask & 0x20000 != 0 {
        let count = r.u8()?;
        let mut spots = vec![];
        for _ in 0..count {
            let slot = byte(r, Neg)?;
            let id = r.u16_add(false)?;
            let packed = r.u32()?;
            spots.push((slot, id, (packed >> 16) as u16, packed as u16));
        }
        out.push(Update::Actor(Block::SpotAnimations(spots)));
    }
    if mask & 16 != 0 {
        let low = r.u16_add(true)? as u32;
        out.push(Update::FaceEntity(low | ((r.u8()? as u32) << 16)));
    }
    if mask & 0x100 != 0 {
        let packed = r.u16_add(false)?;
        let color = (packed >> 8) as u8;
        let effect = packed as u8;
        let icon = byte(r, Neg)?;
        let automatic = byte(r, Add)? == 1;
        let size = byte(r, Neg)? as usize;
        let mut compressed = r.bytes(size)?.to_vec();
        compressed.reverse();
        let text = crate::chat_packet::decompress(&compressed, huffman);
        let mut pattern = vec![];
        if (13..=20).contains(&color) {
            for _ in 0..color - 12 {
                pattern.push(byte(r, Neg)?);
            }
        }
        out.push(Update::Chat(Chat {
            color,
            effect,
            icon,
            automatic,
            text,
            pattern,
        }));
    }
    if mask & 0x100000 != 0 {
        out.push(Update::Actor(Block::Transparency {
            delays: [r.u16_add(false)? as i16, r.u16()? as i16],
            alpha: [byte(r, Sub)? as i8, byte(r, Sub)? as i8],
            use_start: byte(r, Sub)? != 0,
        }));
    }
    if mask & 0x200 != 0 {
        out.push(Update::Actor(Block::Tint {
            delays: [r.u16_le()?, r.u16()?],
            hsl: [byte(r, Neg)?, r.u8()?, byte(r, Sub)?],
            weight: byte(r, Sub)?,
        }));
    }
    if mask & 0x10000 != 0 {
        let count = byte(r, Add)?;
        let mut bars = vec![];
        for _ in 0..count {
            let kind = r.unsigned_smart()?;
            let end = r.unsigned_smart()?;
            let (timing, fill) = if end == 32767 {
                (None, [0, 0])
            } else {
                let start = r.unsigned_smart()?;
                let a = byte(r, Add)?;
                let b = if end > 0 { byte(r, Neg)? } else { a };
                (Some([start, end]), [a, b])
            };
            bars.push(Headbar { kind, timing, fill });
        }
        out.push(Update::Actor(Block::Headbars(bars)));
    }
    if mask & 4 != 0 {
        out.push(Update::Actor(Block::Say(r.string()?)));
    }
    if mask & 64 != 0 {
        out.push(Update::Actor(Block::Sequence {
            id: r.u16_add(false)?,
            delay: r.u8()?,
        }));
    }
    if mask & 0x1000 != 0 {
        out.push(Update::TemporarySpeed(r.u8()? as i8));
    }
    if mask & 32 != 0 {
        let len = byte(r, Sub)? as usize;
        let bytes = r
            .bytes(len)?
            .iter()
            .map(|v| v.wrapping_sub(128))
            .collect::<Vec<_>>();
        out.push(Update::Appearance(player_appearance::decode(
            &bytes, items,
        )?));
    }
    if mask & 0x4000 != 0 {
        out.push(Update::Actor(Block::ExactMove {
            offsets: [
                r.u8()? as i8,
                byte(r, Add)? as i8,
                byte(r, Neg)? as i8,
                byte(r, Sub)? as i8,
            ],
            delays: [r.u16()?, r.u16_le()?],
            direction: r.u16_add(true)?,
        }));
    }
    if mask & 0x80000 != 0 {
        out.push(Update::Actor(Block::Freeze {
            delay: r.u16_add(true)?,
            duration: r.u16_add(true)?,
            cancel_sequence: byte(r, Add)? != 0,
        }));
    }
    Ok(out)
}
