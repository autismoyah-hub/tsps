//! Music request decoding for server packets and CS2 opcodes.
use crate::{
    buffer::Reader, interface_script::ints, music_runtime::Music, protocol::Packet, vm::Stacks,
    Error, Result,
};
#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    Songs(Vec<i32>, [i32; 4]),
    Jingle(i32, i32),
    Stop(i32, i32),
    Swap([i32; 4]),
}
fn song(id: u16) -> i32 {
    if id == 65535 {
        -1
    } else {
        id as i32
    }
}
impl Request {
    pub fn packet(packet: &Packet) -> Result<Option<Self>> {
        let mut r = Reader::new(&packet.payload);
        let request = match packet.opcode {
            0 => Self::Stop(r.u16_add(true)? as i32, r.u16_add(true)? as i32),
            35 => {
                let start = r.u16_le()? as i32;
                let stop = r.u16()? as i32;
                let id = song(r.u16_add(true)?);
                let fade_in = r.u16()? as i32;
                let fade_out = r.u16_add(true)? as i32;
                Self::Songs(vec![id], [stop, fade_out, start, fade_in])
            }
            99 => {
                let first = song(r.u16()?);
                let start = r.u16_add(true)? as i32;
                let second = song(r.u16_le()?);
                let fade_out = r.u16()? as i32;
                let stop = r.u16_le()? as i32;
                let fade_in = r.u16_add(true)? as i32;
                Self::Songs(vec![first, second], [stop, fade_out, start, fade_in])
            }
            107 => {
                let fade_out = r.u16()? as i32;
                let start = r.u16()? as i32;
                let fade_in = r.u16_add(false)? as i32;
                let stop = r.u16()? as i32;
                Self::Swap([stop, fade_out, start, fade_in])
            }
            118 => {
                let unused = r.u24()? as i32;
                Self::Jingle(song(r.u16_add(true)?), unused)
            }
            _ => return Ok(None),
        };
        if r.remaining() != 0 {
            return Err(Error::Invalid("Music packet trailing bytes"));
        }
        Ok(Some(request))
    }
    pub fn script(opcode: u16, stacks: &mut Stacks) -> Result<Option<Self>> {
        Ok(Some(match opcode {
            3201 => {
                let [id, a, b, c, d] = ints(stacks)?;
                Self::Songs(vec![id], [a, b, c, d])
            }
            3202 => {
                let [id, unused] = ints(stacks)?;
                Self::Jingle(id, unused)
            }
            3220 => {
                let [a, b] = ints(stacks)?;
                Self::Stop(a, b)
            }
            3221 => {
                let [first, second, a, b, c, d] = ints(stacks)?;
                Self::Songs(vec![first, second], [a, b, c, d])
            }
            3222 => Self::Swap(ints(stacks)?),
            _ => return Ok(None),
        }))
    }
    pub fn apply(self, music: &mut Music, volume: i32) {
        match self {
            Self::Songs(ids, timing) => music.songs(&ids, timing, volume),
            Self::Jingle(id, _) => music.play_jingle(id, volume),
            Self::Stop(delay, duration) => music.stop(delay, duration),
            Self::Swap(timing) => music.swap(timing),
        }
    }
}
