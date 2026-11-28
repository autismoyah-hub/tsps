//! Dialog input responses from GameClanChannel.bc (3104..3106).
use crate::{protocol::Packet, writer::Writer, Error, Result};
pub fn response(opcode:u16,text:&str)->Result<Packet> {
    let mut writer=Writer::new();
    let opcode=match opcode {
        3104=>{writer.u32(text.parse::<i32>().unwrap_or(0) as u32);75},
        3105|3106=>{writer.string(text)?;if opcode==3105 {26}else{64}},
        _=>return Err(Error::Invalid("dialog response opcode")),
    };
    Ok(Packet{opcode,payload:writer.into_vec()})
}
