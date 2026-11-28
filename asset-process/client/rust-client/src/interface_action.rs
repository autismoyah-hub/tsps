//! Revision 239 interface operation packet payloads.
use crate::{protocol::Packet, writer::Writer, Error, Result};
pub fn operation(
    widget: u32,
    child: i32,
    item: i32,
    operation: u8,
    sub_operation: Option<u8>,
) -> Result<Packet> {
    if !(1..=32).contains(&operation) {
        return Err(Error::Invalid("interface operation index"));
    }
    let mut w = Writer::new();
    w.u32(widget);
    w.u16(child as u16);
    w.u16(item as u16);
    w.u8(operation);
    if let Some(sub) = sub_operation {
        w.u8(sub);
    }
    Ok(Packet {
        opcode: if sub_operation.is_some() { 40 } else { 47 },
        payload: w.into_vec(),
    })
}

pub fn button(widget: u32) -> Packet {
    let mut w = Writer::new();
    w.u32(widget);
    Packet {
        opcode: 51,
        payload: w.into_vec(),
    }
}
pub fn resume(widget: u32, child: i32) -> Packet {
    let mut w = Writer::new();
    w.u32_middle(widget);
    w.u16(child as u16);
    Packet {
        opcode: 115,
        payload: w.into_vec(),
    }
}

pub fn drag(source: (u32, i32, i32), target: (u32, i32, i32)) -> Packet {
    let mut w = Writer::new();
    w.u32_le(source.0);
    w.u16_le(source.2 as u16);
    w.u16_add(source.1 as u16, true);
    w.u32(target.0);
    w.u16_add(target.2 as u16, false);
    w.u16_add(target.1 as u16, false);
    Packet {
        opcode: 48,
        payload: w.into_vec(),
    }
}
