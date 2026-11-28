//! Revision 239 selected-spell state and outbound target packets.
use crate::{buffer::Transform, protocol::Packet, writer::Writer, Error, Result};
#[derive(Clone, Debug)]
pub struct Selection {
    pub widget: u32,
    pub logical: u32,
    pub child: i32,
    pub item: i32,
    pub mask: u8,
    pub option: String,
    pub target: String,
}
#[derive(Clone, Copy)]
pub struct Source {
    pub widget: u32,
    pub child: u16,
    pub item: u16,
}
impl Selection {
    pub fn from_widget(
        id: u32,
        logical: u32,
        child: i32,
        item: i32,
        flags: u32,
        widget: &crate::widget::Widget,
    ) -> Self {
        let mask = ((flags >> 11) & 63) as u8;
        Self {
            widget: id,
            logical,
            child,
            item,
            mask,
            option: if mask == 0 || !crate::menu_label::present(&widget.spell_action) {
                "null".into()
            } else {
                widget.spell_action.clone()
            },
            target: if widget.if3 {
                format!("{}<col=ffffff>", widget.data_text)
            } else {
                format!("<col=ff00>{}<col=ffffff>", widget.spell_name)
            },
        }
    }
    pub fn source(&self) -> Source {
        Source {
            widget: self.logical,
            child: self.child as u16,
            item: self.item as u16,
        }
    }
}
pub fn world(kind: u8, id: u16, tile: [u16; 2], control: bool, source: Source) -> Result<Packet> {
    let mut w = Writer::new();
    let opcode = match kind {
        0 => {
            w.transformed(control as u8, Transform::Neg);
            w.u16_add(source.child, false);
            w.u16_le(id);
            w.u32_le(source.widget);
            w.u16_add(tile[1], true);
            w.u16_le(tile[0]);
            w.u16_le(source.item);
            62
        }
        1 => {
            w.transformed(control as u8, Transform::Neg);
            w.u16(source.child);
            w.u16_add(id, false);
            w.u16(source.item);
            w.u32(source.widget);
            86
        }
        2 => {
            w.u16_add(source.child, false);
            w.transformed(control as u8, Transform::Add);
            w.u16_add(source.item, true);
            w.u32_inverse_middle(source.widget);
            w.u16(id);
            79
        }
        3 => {
            w.u16_le(source.item);
            w.u16_add(id, false);
            w.u16(tile[0]);
            w.u32_inverse_middle(source.widget);
            w.u8(control as u8);
            w.u16_le(source.child);
            w.u16(tile[1]);
            76
        }
        _ => return Err(Error::Invalid("spell target kind")),
    };
    Ok(Packet {
        opcode,
        payload: w.into_vec(),
    })
}
pub fn interface(source: Source, widget: u32, child: u16, item: u16) -> Packet {
    let mut w = Writer::new();
    w.u32(widget);
    w.u16_add(source.child, false);
    w.u16_add(item, true);
    w.u16_le(child);
    w.u16_add(source.item, false);
    w.u32_inverse_middle(source.widget);
    Packet {
        opcode: 27,
        payload: w.into_vec(),
    }
}

impl Selection {
    pub fn world_entry(
        &self,
        kind: u8,
        id: u16,
        tile: [u16; 2],
        target: &str,
    ) -> Option<crate::context_menu::Entry<crate::context_menu::Action>> {
        let bit = match kind {
            0 => 4,
            1 => 2,
            2 => 8,
            3 => 1,
            _ => 0,
        };
        (self.mask & bit != 0).then(|| crate::context_menu::Entry {
            option: self.option.clone(),
            target: format!("{} -> {}", self.target, target),
            action: crate::context_menu::Action::CastWorld { kind, id, tile },
            submenu: false,
            children: Vec::new(),
        })
    }
}
