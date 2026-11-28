//! Cache Vorbis packets adapted to the standard mono Vorbis headers.
use crate::{buffer::Reader, Error, Result};
pub struct Sample {
    pub rate: u32,
    pub samples: Vec<i16>,
    pub loop_start: usize,
    pub loop_end: usize,
    pub ping_pong: bool,
}
pub fn decode_embedded(data: &[u8]) -> Result<Sample> {
    let mut r = Reader::new(data);
    let length = r.u32()? as usize;
    let setup = r.bytes(length)?;
    decode(setup, r.bytes(r.remaining())?)
}
pub fn decode(setup: &[u8], data: &[u8]) -> Result<Sample> {
    let mut r = Reader::new(data);
    let rate = r.u32()?;
    let count = r.u32()? as usize;
    let loop_start = r.u32()? as usize;
    let end = r.u32()? as i32;
    let ping_pong = end < 0;
    let loop_end = if ping_pong { !end } else { end } as usize;
    let packets = r.u32()? as usize;
    if rate == 0 || rate > 192000 || count > 64 << 20 || packets > 1 << 20 {
        return Err(Error::Invalid("Vorbis cache sample dimensions"));
    }
    let blocks = *setup.first().ok_or(Error::Truncated)?;
    let mut identification = vec![1, b'v', b'o', b'r', b'b', b'i', b's'];
    identification.extend(0u32.to_le_bytes());
    identification.push(1);
    identification.extend(rate.to_le_bytes());
    identification.extend([0; 12]);
    identification.push(blocks);
    identification.push(1);
    let ident = lewton::header::read_header_ident(&identification)
        .map_err(|e| Error::Io(format!("Vorbis identification: {e:?}")))?;
    let mut header = vec![5, b'v', b'o', b'r', b'b', b'i', b's'];
    header.extend_from_slice(&setup[1..]);
    let setup =
        lewton::header::read_header_setup(&header, 1, (ident.blocksize_0, ident.blocksize_1))
            .map_err(|e| Error::Io(format!("Vorbis setup: {e:?}")))?;
    let mut previous = lewton::audio::PreviousWindowRight::new();
    let mut samples = Vec::with_capacity(count);
    for _ in 0..packets {
        let mut length = 0usize;
        loop {
            let n = r.u8()? as usize;
            length = length
                .checked_add(n)
                .ok_or(Error::Invalid("Vorbis packet length"))?;
            if n != 255 {
                break;
            }
        }
        let packet = r.bytes(length)?;
        let decoded: Vec<Vec<i16>> =
            lewton::audio::read_audio_packet_generic(&ident, &setup, packet, &mut previous)
                .map_err(|e| Error::Io(format!("Vorbis audio packet: {e:?}")))?;
        if let Some(mono) = decoded.first() {
            let remaining = count - samples.len();
            samples.extend(mono.iter().copied().take(remaining));
        }
    }
    if r.remaining() != 0 {
        return Err(Error::Invalid("Vorbis trailing data"));
    }
    samples.resize(count, 0);
    Ok(Sample {
        rate,
        samples,
        loop_start,
        loop_end,
        ping_pong,
    })
}
