//! Initial world action encoders, following original GameClient/GameItemContainer writes.
use crate::{buffer::Transform, protocol::Packet, writer::Writer, Error, Result};
pub fn world_entity(operation: u8, id: u16, control: bool) -> Result<Packet> {
    let mut w = Writer::new();
    let opcode = match operation {
        1 => {
            w.u16_le(id);
            w.transformed(control as u8, Transform::Sub);
            14
        }
        2 => {
            w.u16_add(id, false);
            w.transformed(control as u8, Transform::Add);
            80
        }
        3 => {
            w.u16(id);
            w.transformed(control as u8, Transform::Sub);
            100
        }
        4 => {
            w.u16(id);
            w.u8(control as u8);
            83
        }
        5 => {
            w.transformed(control as u8, Transform::Add);
            w.u16_add(id, false);
            106
        }
        6 => {
            w.u16_add(id, true);
            98
        }
        _ => return Err(Error::Invalid("world entity operation index")),
    };
    Ok(Packet {
        opcode,
        payload: w.into_vec(),
    })
}
pub fn walk(x: u16, z: u16, keys: u8) -> Result<Packet> {
    if x >= 16384 || z >= 16384 || keys > 2 {
        return Err(Error::Invalid("walk target/modifiers"));
    }
    let mut w = Writer::new();
    w.u16_le(x);
    w.u16_le(z);
    w.transformed(keys, Transform::Sub);
    Ok(Packet {
        opcode: 114,
        payload: w.into_vec(),
    })
}
pub fn npc_first(index: u16, subop: u8, control: bool) -> Packet {
    let mut w = Writer::new();
    w.u16_le(index);
    w.transformed(subop, Transform::Neg);
    w.transformed(u8::from(control), Transform::Sub);
    Packet {
        opcode: 102,
        payload: w.into_vec(),
    }
}
pub fn npc(operation: u8, index: u16, sub: u8, control: bool) -> Result<Packet> {
    let mut w = Writer::new();
    let ctrl = u8::from(control);
    let opcode = match operation {
        1 => return Ok(npc_first(index, sub, control)),
        2 => {
            w.u16_add(index, true);
            w.transformed(sub, Transform::Neg);
            w.transformed(ctrl, Transform::Sub);
            13
        }
        3 => {
            w.transformed(ctrl, Transform::Neg);
            w.transformed(sub, Transform::Sub);
            w.u16_add(index, false);
            19
        }
        4 => {
            w.transformed(sub, Transform::Add);
            w.u16(index);
            w.transformed(ctrl, Transform::Neg);
            45
        }
        5 => {
            w.transformed(sub, Transform::Add);
            w.transformed(ctrl, Transform::Neg);
            w.u16_add(index, false);
            85
        }
        _ => return Err(Error::Invalid("NPC operation index")),
    };
    Ok(Packet {
        opcode,
        payload: w.into_vec(),
    })
}
pub fn player(operation: u8, index: u16, control: bool) -> Result<Packet> {
    let mut w = Writer::new();
    let ctrl = u8::from(control);
    let opcode = match operation {
        1 => {
            w.transformed(ctrl, Transform::Sub);
            w.u16_le(index);
            93
        }
        2 => {
            w.u16_le(index);
            w.transformed(ctrl, Transform::Neg);
            3
        }
        3 => {
            w.transformed(ctrl, Transform::Neg);
            w.u16_add(index, true);
            92
        }
        4 => {
            w.u16(index);
            w.u8(ctrl);
            37
        }
        5 => {
            w.transformed(ctrl, Transform::Sub);
            w.u16(index);
            57
        }
        6 => {
            w.u16_add(index, true);
            w.transformed(ctrl, Transform::Sub);
            70
        }
        7 => {
            w.u8(ctrl);
            w.u16(index);
            22
        }
        8 => {
            w.u16(index);
            w.transformed(ctrl, Transform::Neg);
            53
        }
        _ => return Err(Error::Invalid("player operation index")),
    };
    Ok(Packet {
        opcode,
        payload: w.into_vec(),
    })
}
pub fn examine(kind: u8, id: u16, tile: [u16; 2]) -> Result<Packet> {
    let mut w = Writer::new();
    let opcode = match kind {
        0 => {
            w.u16(id);
            0
        }
        1 => {
            w.u16_le(id);
            78
        }
        2 => {
            w.u16_add(tile[1], false);
            w.u16_le(id);
            w.u16_le(tile[0]);
            58
        }
        _ => return Err(Error::Invalid("examine entity kind")),
    };
    Ok(Packet {
        opcode,
        payload: w.into_vec(),
    })
}
pub fn object_first(id: u16, x: u16, y: u16, sub: u8, control: bool) -> Result<Packet> {
    object(1, id, x, y, sub, control)
}
/// GameClient.wt: the five location operations.
pub fn object(op: u8, id: u16, x: u16, y: u16, sub: u8, control: bool) -> Result<Packet> {
    if x >= 16384 || y >= 16384 {
        return Err(Error::Invalid("object target"));
    }
    let mut w = Writer::new();
    let ctrl = u8::from(control);
    let opcode = match op {
        1 => {
            w.u16_add(y, false);
            w.u16_le(x);
            w.transformed(ctrl, Transform::Add);
            w.transformed(sub, Transform::Add);
            w.u16(id);
            73
        }
        2 => {
            w.u16_add(y, true);
            w.u16_add(id, true);
            w.u16(x);
            w.transformed(sub, Transform::Neg);
            w.u8(ctrl);
            103
        }
        3 => {
            w.u16_add(id, false);
            w.transformed(sub, Transform::Sub);
            w.u8(ctrl);
            w.u16_add(y, true);
            w.u16_add(x, false);
            104
        }
        4 => {
            w.transformed(ctrl, Transform::Add);
            w.u16_le(x);
            w.u16_add(id, true);
            w.transformed(sub, Transform::Neg);
            w.u16(y);
            15
        }
        5 => {
            w.u16_add(y, true);
            w.u16_le(x);
            w.transformed(ctrl, Transform::Add);
            w.transformed(sub, Transform::Neg);
            w.u16(id);
            116
        }
        _ => return Err(Error::Invalid("object operation index")),
    };
    Ok(Packet {
        opcode,
        payload: w.into_vec(),
    })
}
/// The minimap click has its own packet (nw.bi), including the original input trailer.
pub fn minimap_walk(
    target: [i32; 2],
    keys: u8,
    click: [i32; 2],
    angle: u16,
    local_fine: [i32; 2],
) -> Result<Packet> {
    if keys > 2 || angle >= 16384 {
        return Err(Error::Invalid("minimap walk arguments"));
    }
    let mut w = Writer::new();
    w.u16_le(target[0] as u16);
    w.u16_le(target[1] as u16);
    w.transformed(keys, Transform::Sub);
    w.u8(click[0] as u8);
    w.u8(click[1] as u8);
    w.u16(angle / 8);
    for b in [57, 0, 0, 89] {
        w.u8(b);
    }
    w.u16(local_fine[0] as u16);
    w.u16(local_fine[1] as u16);
    w.u8(63);
    Ok(Packet {
        opcode: 67,
        payload: w.into_vec(),
    })
}
/// GameClient.vf: the five ground-item operations (one-based operation index).
pub fn ground_item(op: u8, id: u16, x: u16, y: u16, sub: u8, control: bool) -> Result<Packet> {
    if x >= 16384 || y >= 16384 {
        return Err(Error::Invalid("ground action coordinate"));
    }
    let mut w = Writer::new();
    let ctrl = u8::from(control);
    let opcode = match op {
        1 => {
            w.u16_add(id, false);
            w.transformed(sub, Transform::Sub);
            w.u16_add(y, false);
            w.transformed(ctrl, Transform::Sub);
            w.u16(x);
            46
        }
        2 => {
            w.transformed(ctrl, Transform::Add);
            w.transformed(sub, Transform::Sub);
            w.u16(y);
            w.u16_le(id);
            w.u16_add(x, true);
            74
        }
        3 => {
            w.transformed(ctrl, Transform::Add);
            w.u16_le(x);
            w.u16_add(id, true);
            w.transformed(sub, Transform::Add);
            w.u16_add(y, false);
            7
        }
        4 => {
            w.transformed(sub, Transform::Sub);
            w.u16(id);
            w.u16(x);
            w.u16_le(y);
            w.transformed(ctrl, Transform::Neg);
            31
        }
        5 => {
            w.transformed(sub, Transform::Neg);
            w.u16_add(x, false);
            w.u16(id);
            w.u16_add(y, true);
            w.transformed(ctrl, Transform::Add);
            43
        }
        _ => return Err(Error::Invalid("ground operation index")),
    };
    Ok(Packet {
        opcode,
        payload: w.into_vec(),
    })
}
