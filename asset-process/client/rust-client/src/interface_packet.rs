//! Revision 239 interface packet payloads, matching GameClient and RSProx v239.
use crate::{
    buffer::{Reader, Transform},
    protocol::Packet,
    Error, Result,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubInterface {
    pub parent: u32,
    pub group: u16,
    pub kind: u8,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Events {
    pub widget: u32,
    pub first: i32,
    pub last: i32,
    pub masks: [u32; 2],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Update {
    OpenTop(u16),
    OpenSub(SubInterface),
    CloseSub(u32),
    MoveSub {
        source: u32,
        destination: u32,
    },
    Resync {
        root: u16,
        children: Vec<SubInterface>,
        events: Vec<Events>,
    },
    Events(Events),
    ClearInventory(u32),
    Position {
        widget: u32,
        position: [i32; 2],
    },
    Scroll {
        widget: u32,
        y: u16,
    },
    Rotate {
        widget: u32,
        speed: [i16; 2],
    },
    Text {
        widget: u32,
        text: String,
    },
    Hide {
        widget: u32,
        hidden: bool,
    },
    Angle {
        widget: u32,
        angles: [u16; 2],
        zoom: u16,
    },
    Object {
        widget: u32,
        item: i32,
        count: i32,
    },
    Color {
        widget: u32,
        packed: u16,
    },
    Animation {
        widget: u32,
        id: i16,
    },
    NpcHead {
        widget: u32,
        id: u16,
    },
    NpcHeadActive {
        widget: u32,
        index: u16,
    },
    PlayerHead(u32),
    Model {
        widget: u32,
        id: i32,
    },
    PlayerColor {
        widget: u32,
        index: u8,
        color: u8,
    },
    PlayerBody {
        widget: u32,
        body: u8,
    },
    PlayerObject {
        widget: u32,
        item: i32,
    },
    PlayerSelf {
        widget: u32,
        copy_objects: bool,
    },
}
impl Update {
    /// Group dependencies needed before applying a packet's interface mutations.
    pub fn groups(&self) -> Vec<u16> {
        let mut groups = match self {
            Self::OpenTop(group) => vec![*group],
            Self::OpenSub(sub) => vec![(sub.parent >> 16) as u16, sub.group],
            Self::Resync { root, children, .. } => std::iter::once(*root)
                .chain(
                    children
                        .iter()
                        .flat_map(|s| [(s.parent >> 16) as u16, s.group]),
                )
                .collect(),
            Self::Position { widget, .. }
            | Self::Scroll { widget, .. }
            | Self::Rotate { widget, .. }
            | Self::Text { widget, .. }
            | Self::Hide { widget, .. }
            | Self::Angle { widget, .. }
            | Self::Object { widget, .. }
            | Self::Color { widget, .. }
            | Self::Animation { widget, .. }
            | Self::NpcHead { widget, .. }
            | Self::NpcHeadActive { widget, .. }
            | Self::Model { widget, .. }
            | Self::PlayerColor { widget, .. }
            | Self::PlayerBody { widget, .. }
            | Self::PlayerObject { widget, .. }
            | Self::PlayerSelf { widget, .. }
            | Self::PlayerHead(widget) => vec![(*widget >> 16) as u16],
            _ => Vec::new(),
        };
        groups.retain(|g| *g != 65535);
        groups.sort_unstable();
        groups.dedup();
        groups
    }
}
fn slot(v: u16) -> i32 {
    if v == 65535 {
        -1
    } else {
        v as i32
    }
}
pub fn decode(packet: &Packet) -> Result<Option<Update>> {
    let mut r = Reader::new(&packet.payload);
    let u = match packet.opcode {
        7 => {
            let group = r.u16_add(true)?;
            let parent = r.u32_middle()?;
            Update::OpenSub(SubInterface {
                parent,
                group,
                kind: r.u8()?,
            })
        }
        14 => {
            let y = r.u16()? as i16;
            let x = r.u16_add(false)? as i16;
            Update::Rotate {
                widget: r.u32_middle()?,
                speed: [x, y],
            }
        }
        16 => {
            let index = r.u8()?;
            let color = r.u8()?;
            Update::PlayerColor {
                widget: r.u32()?,
                index,
                color,
            }
        }
        17 => {
            let widget = r.u32_middle()?;
            let zoom = r.u16()?;
            let angles = [r.u16()?, r.u16_add(true)?];
            Update::Angle {
                widget,
                angles,
                zoom,
            }
        }
        23 => Update::CloseSub(r.u32()?),
        25 => {
            let root = r.u16()?;
            let count = r.u16()?;
            let mut children = Vec::new();
            for _ in 0..count {
                children.push(SubInterface {
                    parent: r.u32()?,
                    group: r.u16()?,
                    kind: r.u8()?,
                });
            }
            let mut events = Vec::new();
            while r.remaining() != 0 {
                events.push(Events {
                    widget: r.u32()?,
                    first: slot(r.u16()?),
                    last: slot(r.u16()?),
                    masks: [r.u32()?, r.u32()?],
                });
            }
            Update::Resync {
                root,
                children,
                events,
            }
        }
        55 => {
            let packed = r.u16()?;
            Update::Color {
                widget: r.u32_inverse_middle()?,
                packed,
            }
        }
        59 => Update::Animation {
            widget: r.u32_le()?,
            id: r.u16_le()? as i16,
        },
        63 => Update::Hide {
            widget: r.u32_middle()?,
            hidden: r.transformed(Transform::Add)? == 1,
        },
        80 => {
            let text = r.string()?;
            Update::Text {
                widget: r.u32_inverse_middle()?,
                text,
            }
        }
        82 => Update::Scroll {
            widget: r.u32_middle()?,
            y: r.u16()?,
        },
        84 => Update::NpcHead {
            widget: r.u32()?,
            id: r.u16_add(true)?,
        },
        89 => {
            let index = r.u16_add(false)?;
            Update::NpcHeadActive {
                widget: r.u32_inverse_middle()?,
                index,
            }
        }
        94 => Update::ClearInventory(r.u32_inverse_middle()?),
        96 => Update::OpenTop(r.u16_add(false)?),
        98 => Update::PlayerObject {
            widget: r.u32_inverse_middle()?,
            item: r.u32_middle()? as i32,
        },
        100 => Update::PlayerBody {
            widget: r.u32_inverse_middle()?,
            body: r.u8()?,
        },
        102 => {
            let y = r.u16_add(true)? as i16 as i32;
            let widget = r.u32_inverse_middle()?;
            let x = r.u16_add(true)? as i16 as i32;
            Update::Position {
                widget,
                position: [x, y],
            }
        }
        106 => Update::Object {
            widget: r.u32_inverse_middle()?,
            item: slot(r.u16_add(false)?),
            count: r.u32()? as i32,
        },
        108 => {
            let last = slot(r.u16_add(false)?);
            let second = r.u32()?;
            let first_mask = r.u32_inverse_middle()?;
            let first = slot(r.u16()?);
            Update::Events(Events {
                widget: r.u32_middle()?,
                first,
                last,
                masks: [first_mask, second],
            })
        }
        120 => Update::PlayerHead(r.u32_inverse_middle()?),
        121 => {
            let copy_objects = r.transformed(Transform::Sub)? == 1;
            Update::PlayerSelf {
                widget: r.u32()?,
                copy_objects,
            }
        }
        123 => {
            let destination = r.u32_le()?;
            Update::MoveSub {
                source: r.u32()?,
                destination,
            }
        }
        142 => {
            let id = r.u32_inverse_middle()? as i32;
            Update::Model {
                widget: r.u32_le()?,
                id,
            }
        }
        _ => return Ok(None),
    };
    if r.remaining() != 0 {
        return Err(Error::Invalid("interface packet trailing bytes"));
    }
    Ok(Some(u))
}
