//! GameGraphicsObject.an2929 / pz.fq argument-bearing widget operation.
use crate::{
    disk_cache::DiskCache,
    interface_state::Interfaces,
    protocol::Packet,
    vm::{Array, Object, Stacks, Value},
    writer::Writer,
    Error, Result,
};
use crate::outbound::Queue;

pub fn execute(
    opcode: u16,
    stacks: &mut Stacks,
    cache: &mut DiskCache,
    interfaces: &mut Interfaces,
    outbound: &mut Queue,
) -> Result<bool> {
    if opcode != 2929 {
        return Ok(false);
    }
    let Object::String(format) = stacks.pop_object()? else {
        return Err(Error::Invalid("widget operation argument signature"));
    };
    let mut arguments = Vec::new();
    // Java's format indexes UTF-16 units. Every non-'i' unit consumes an object.
    for kind in format.encode_utf16().collect::<Vec<_>>().into_iter().rev() {
        arguments.push(if kind == b'i' as u16 {
            Value::Int(stacks.pop_int()?)
        } else {
            Value::Object(stacks.pop_object()?)
        });
    }
    arguments.reverse();
    let [action, widget, child] = crate::interface_script::ints(stacks)?;
    interfaces.load_group(cache, (widget as u32 >> 16) as u16)?;
    let id = interfaces
        .checked_script_child(widget as u32, child)?
        .ok_or(Error::Invalid("widget argument operation target"))?;
    let item = interfaces.runtime[&id].item;
    if interfaces.interaction_flags(id) & (1 << 23) == 0 {
        return Ok(true);
    }
    match packet(widget as u32, child, item, action, &arguments) {
        Ok(packet) => outbound.push_back(packet),
        Err(error) => {
            // pz.fq has already allocated/encrypted the opcode through xt.ag.
            outbound.discard_cipher();
            return Err(error);
        }
    }
    Ok(true)
}

fn signed_leb128(writer: &mut Writer, value: i32) {
    let mut bits = (value.wrapping_shl(1) ^ (value >> 31)) as u32;
    while bits > 127 {
        writer.u8(bits as u8 | 128);
        bits >>= 7;
    }
    writer.u8(bits as u8);
}
fn packet(widget: u32, child: i32, item: i32, action: i32, arguments: &[Value]) -> Result<Packet> {
    let mut writer = Writer::new();
    writer.u16_add(child as u16, true);
    writer.u16(item as u16);
    writer.u32_le(widget);
    writer.u32_le(action as u32);
    for argument in arguments {
        match argument {
            Value::Int(value) => signed_leb128(&mut writer, *value),
            Value::Object(Object::String(value)) => writer.string(value)?,
            Value::Object(Object::Null) => writer.u8(0),
            Value::Object(Object::Array(array)) => {
                let array = array.borrow();
                writer.unsigned_smart(
                    u16::try_from(array.len())
                        .map_err(|_| Error::Invalid("widget argument array length"))?,
                )?;
                match array.data() {
                    Array::Ints(values) => {
                        for &value in values {
                            signed_leb128(&mut writer, value);
                        }
                    }
                    Array::Strings(values) => {
                        for value in values {
                            writer.string(
                                value
                                    .as_deref()
                                    .ok_or(Error::Invalid("null widget argument array string"))?,
                            )?;
                        }
                    }
                    _ => return Err(Error::Invalid("widget argument array type")),
                }
            }
            // pz.fq emits nothing for other boxed values.
            Value::Long(_) => {}
        }
    }
    // xt.ag allocates10000 bytes including opcode and the two-byte length prefix.
    if writer.len() > 9997 {
        return Err(Error::Invalid("widget argument packet capacity"));
    }
    Ok(Packet {
        opcode: 56,
        payload: writer.into_vec(),
    })
}
