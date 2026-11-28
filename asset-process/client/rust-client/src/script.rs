//! Revision 239 client-script decoding, from sz.aa([BI)Lbk;.
//! Revision 239 has long locals/arguments and a 16-byte fixed footer.
use crate::{buffer::Reader, Error, Result};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    Int(i32),
    String(String),
    Long(i64),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instruction {
    pub opcode: u16,
    pub operand: Operand,
}
#[derive(Debug)]
pub struct Script {
    pub name: Option<String>,
    pub local_ints: u16,
    pub local_strings: u16,
    pub local_longs: u16,
    pub int_arguments: u16,
    pub string_arguments: u16,
    pub long_arguments: u16,
    pub instructions: Vec<Instruction>,
    pub switches: Vec<BTreeMap<i32, i32>>,
}
impl Script {
    pub fn decode(data: &[u8], max_instructions: usize) -> Result<Self> {
        let end = data.len().checked_sub(2).ok_or(Error::Truncated)?;
        let switch_size = Reader::new(&data[end..]).u16()? as usize;
        let footer = end.checked_sub(switch_size + 16).ok_or(Error::Truncated)?;
        let mut r = Reader::new(&data[footer..end]);
        let count = r.u32()? as usize;
        if count > max_instructions || count > footer / 3 {
            return Err(Error::Invalid("script instruction count"));
        }
        let local_ints = r.u16()?;
        let local_strings = r.u16()?;
        let local_longs = r.u16()?;
        let int_arguments = r.u16()?;
        let string_arguments = r.u16()?;
        let long_arguments = r.u16()?;
        if int_arguments > local_ints
            || string_arguments > local_strings
            || long_arguments > local_longs
        {
            return Err(Error::Invalid("script arguments exceed locals"));
        }
        let mut switches = Vec::new();
        for _ in 0..r.u8()? {
            let mut table = BTreeMap::new();
            for _ in 0..r.u16()? {
                let key = r.u32()? as i32;
                let jump = r.u32()? as i32;
                if table.insert(key, jump).is_some() {
                    return Err(Error::Invalid("duplicate script switch key"));
                }
            }
            switches.push(table);
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("script footer trailing bytes"));
        }
        let mut r = Reader::new(&data[..footer]);
        let name = r.string()?;
        let name = if name.is_empty() { None } else { Some(name) };
        let mut instructions = Vec::with_capacity(count);
        for _ in 0..count {
            let opcode = r.u16()?;
            let operand = match opcode {
                3 => Operand::String(r.string()?),
                61 => Operand::Long(r.u64()? as i64),
                21 | 38 | 39 | 62 | 63 => Operand::Int(r.u8()? as i32),
                0..=99 => Operand::Int(r.u32()? as i32),
                _ => Operand::Int(r.u8()? as i32),
            };
            instructions.push(Instruction { opcode, operand });
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("script instruction trailing bytes"));
        }
        Ok(Self {
            name,
            local_ints,
            local_strings,
            local_longs,
            int_arguments,
            string_arguments,
            long_arguments,
            instructions,
            switches,
        })
    }
}
