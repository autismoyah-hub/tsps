//! Packed cache music (nk) to MIDI, including the instrument sample-use index.
use crate::{buffer::Reader, Error, Result};
use std::collections::BTreeMap;
#[derive(Debug)]
pub struct Instrument {
    pub first_tick: i32,
    pub notes: [bool; 128],
}
#[derive(Debug)]
pub struct Track {
    pub midi: Vec<u8>,
    pub instruments: BTreeMap<i32, Instrument>,
}
fn control_stream(control: usize) -> usize {
    match control {
        0 | 32 => 14,
        1 => 4,
        33 => 11,
        7 => 5,
        39 => 12,
        10 => 6,
        42 => 13,
        99 => 16,
        98 => 17,
        101 => 18,
        100 => 19,
        64 | 65 | 120 | 121 | 123 => 0,
        _ => 9,
    }
}
fn take(bytes: &[u8], cursor: &mut usize) -> Result<i32> {
    let value = *bytes.get(*cursor).ok_or(Error::Truncated)? as i8 as i32;
    *cursor += 1;
    Ok(value)
}
fn variable(out: &mut Vec<u8>, value: u32) {
    let mut buf = [0u8; 5];
    let mut i = 4;
    buf[i] = (value & 127) as u8;
    let mut v = value >> 7;
    while v != 0 {
        i -= 1;
        buf[i] = (v as u8 & 127) | 128;
        v >>= 7;
    }
    out.extend_from_slice(&buf[i..]);
}
impl Track {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let end = bytes.len().checked_sub(3).ok_or(Error::Truncated)?;
        let tracks = bytes[end] as usize;
        let division = u16::from_be_bytes([bytes[end + 1], bytes[end + 2]]);
        let mut r = Reader::new(&bytes[..end]);
        let mut counts = [0usize; 8];
        let mut tempos = 0;
        let mut events = Vec::new();
        for _ in 0..tracks {
            loop {
                let event = r.u8()?;
                events.push(event);
                if event == 7 {
                    counts[7] += 1;
                    break;
                }
                if event == 23 {
                    tempos += 1;
                } else {
                    let kind = (event & 15) as usize;
                    if kind >= 7 {
                        return Err(Error::Invalid("packed music opcode"));
                    }
                    counts[kind] += 1;
                }
            }
        }
        let deltas = (0..events.len())
            .map(|_| r.varint())
            .collect::<Result<Vec<_>>>()?;
        let controllers = r.bytes(counts[2])?;
        let mut sizes = [0usize; 21];
        sizes[1] = counts[5];
        sizes[2] = counts[4];
        sizes[3] = counts[3];
        sizes[7] = counts[0] + counts[1] + counts[5];
        sizes[8] = counts[0];
        sizes[10] = counts[1];
        sizes[14] = counts[6];
        sizes[15] = counts[3];
        sizes[20] = tempos * 3;
        let mut control = 0usize;
        for &delta in controllers {
            control = (control + delta as usize) & 127;
            sizes[control_stream(control)] += 1;
        }
        let mut offset = end - r.remaining();
        let mut cursors = [0usize; 21];
        for (i, size) in sizes.into_iter().enumerate() {
            cursors[i] = offset;
            offset = offset
                .checked_add(size)
                .ok_or(Error::Invalid("music size overflow"))?;
        }
        if offset != end {
            return Err(Error::Invalid("packed music stream lengths"));
        }
        let mut midi = Vec::new();
        midi.extend_from_slice(b"MThd");
        midi.extend_from_slice(&6u32.to_be_bytes());
        midi.extend_from_slice(&u16::from(tracks > 1).to_be_bytes());
        midi.extend_from_slice(&(tracks as u16).to_be_bytes());
        midi.extend_from_slice(&division.to_be_bytes());
        let mut instruments = BTreeMap::new();
        let mut bank = [0i32; 16];
        let mut program = [0i32; 16];
        bank[9] = 128;
        program[9] = 128;
        let mut channel = 0usize;
        let mut note = 0i32;
        let mut on = 0i32;
        let mut off = 0i32;
        let mut pitch = 0i32;
        let mut pressure = 0i32;
        let mut poly = 0i32;
        let mut control = 0usize;
        let mut control_values = [0i32; 128];
        let mut ci = 0;
        let mut ei = 0;
        for _ in 0..tracks {
            midi.extend_from_slice(b"MTrk");
            let length = midi.len();
            midi.extend_from_slice(&[0; 4]);
            let start = midi.len();
            // nk records its initial MIDI output cursor, then accumulates delta ticks.
            let mut tick = start as i32;
            let mut previous = -1i32;
            loop {
                let event = events[ei];
                let delta = deltas[ei];
                ei += 1;
                variable(&mut midi, delta);
                tick = tick.wrapping_add(delta as i32);
                let status = event as i32 != previous;
                let kind = event & 15;
                previous = kind as i32;
                if event == 7 {
                    if status {
                        midi.push(255);
                    }
                    midi.extend_from_slice(&[47, 0]);
                    let size = (midi.len() - start) as u32;
                    midi[length..length + 4].copy_from_slice(&size.to_be_bytes());
                    break;
                }
                if event == 23 {
                    if status {
                        midi.push(255);
                    }
                    midi.extend_from_slice(&[81, 3]);
                    for _ in 0..3 {
                        midi.push(take(bytes, &mut cursors[20])? as u8);
                    }
                    continue;
                }
                channel ^= (event >> 4) as usize;
                if status {
                    midi.push([144, 128, 176, 224, 208, 160, 192][kind as usize] + channel as u8);
                }
                match kind {
                    0 => {
                        note = note.wrapping_add(take(bytes, &mut cursors[7])?);
                        on = on.wrapping_add(take(bytes, &mut cursors[8])?);
                        midi.extend_from_slice(&[(note & 127) as u8, (on & 127) as u8]);
                        if on & 127 > 0 {
                            instruments
                                .entry(program[channel])
                                .or_insert_with(|| Instrument {
                                    first_tick: tick,
                                    notes: [false; 128],
                                })
                                .notes[(note & 127) as usize] = true;
                        }
                    }
                    1 => {
                        note = note.wrapping_add(take(bytes, &mut cursors[7])?);
                        off = off.wrapping_add(take(bytes, &mut cursors[10])?);
                        midi.extend_from_slice(&[(note & 127) as u8, (off & 127) as u8]);
                    }
                    2 => {
                        control = (control + controllers[ci] as usize) & 127;
                        ci += 1;
                        let stream = control_stream(control);
                        let value = take(bytes, &mut cursors[stream])?
                            .wrapping_add(control_values[control])
                            as i8 as i32;
                        control_values[control] = value;
                        let value = value & 127;
                        midi.extend_from_slice(&[control as u8, value as u8]);
                        if control == 0 {
                            bank[channel] = (bank[channel] & !0x1fc000) + (value << 14);
                        }
                        if control == 32 {
                            bank[channel] = (bank[channel] & !0x3f80) + (value << 7);
                        }
                    }
                    3 => {
                        pitch = pitch
                            .wrapping_add(take(bytes, &mut cursors[15])?)
                            .wrapping_add(take(bytes, &mut cursors[3])? << 7);
                        midi.extend_from_slice(&[(pitch & 127) as u8, ((pitch >> 7) & 127) as u8]);
                    }
                    4 => {
                        pressure = pressure.wrapping_add(take(bytes, &mut cursors[2])?);
                        midi.push((pressure & 127) as u8);
                    }
                    5 => {
                        note = note.wrapping_add(take(bytes, &mut cursors[7])?);
                        poly = poly.wrapping_add(take(bytes, &mut cursors[1])?);
                        midi.extend_from_slice(&[(note & 127) as u8, (poly & 127) as u8]);
                    }
                    6 => {
                        let value = take(bytes, &mut cursors[14])?;
                        program[channel] = bank[channel] + value;
                        midi.push(value as u8);
                    }
                    _ => unreachable!(),
                }
            }
        }
        Ok(Self { midi, instruments })
    }
}
