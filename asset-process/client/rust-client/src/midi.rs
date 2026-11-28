//! Multi-track MIDI cursor and tempo clock, matching nd's event scheduling.
use crate::{buffer::Reader, Error, Result};
struct Cursor<'a> {
    data: &'a [u8],
    reader: Reader<'a>,
    tick: i32,
    status: u8,
    done: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub track: usize,
    pub tick: i32,
    /// Microseconds multiplied by ticks per quarter note.
    pub time: i64,
    /// 0=sysex, 1=end, 2=tempo, 3=other meta, otherwise packed MIDI message.
    pub message: u32,
}
pub struct Midi<'a> {
    pub division: u16,
    tempo: i32,
    offset: i64,
    tracks: Vec<Cursor<'a>>,
}
impl<'a> Midi<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        if r.bytes(4)? != b"MThd" {
            return Err(Error::Invalid("MIDI header"));
        }
        let n = r.u32()? as usize;
        let mut header = Reader::new(r.bytes(n)?);
        let _format = header.u16()?;
        let count = header.u16()? as usize;
        let division = header.u16()?;
        if division == 0 {
            return Err(Error::Invalid("MIDI division"));
        }
        let mut tracks = Vec::with_capacity(count);
        while tracks.len() < count {
            let tag = r.bytes(4)?;
            let len = r.u32()? as usize;
            let data = r.bytes(len)?;
            if tag == b"MTrk" {
                let mut reader = Reader::new(data);
                let tick = reader.varint()? as i32;
                tracks.push(Cursor {
                    data,
                    reader,
                    tick,
                    status: 0,
                    done: false,
                });
            }
        }
        Ok(Self {
            division,
            tempo: 500000,
            offset: 0,
            tracks,
        })
    }
    pub fn reset(&mut self, time: i64) -> Result<()> {
        self.offset = time;
        for track in &mut self.tracks {
            track.reader = Reader::new(track.data);
            track.tick = track.reader.varint()? as i32;
            track.status = 0;
            track.done = false;
        }
        Ok(())
    }
    pub fn next_event(&mut self) -> Result<Option<Event>> {
        let Some(index) = self
            .tracks
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.done)
            .min_by_key(|(i, t)| (t.tick, *i))
            .map(|(i, _)| i)
        else {
            return Ok(None);
        };
        let track = &mut self.tracks[index];
        let tick = track.tick;
        let saved = track.reader.clone();
        let byte = track.reader.u8()?;
        if byte & 128 != 0 {
            track.status = byte;
        } else {
            track.reader = saved;
        }
        if track.status < 128 {
            return Err(Error::Invalid("MIDI running status"));
        }
        let mut status = track.status;
        let message = if matches!(status, 240 | 247) {
            let length = track.reader.varint()? as usize;
            let saved = track.reader.clone();
            let embedded = if status == 247 && length > 0 {
                Some(track.reader.u8()?)
            } else {
                None
            };
            if embedded.is_some_and(|v| matches!(v,241..=243|246|248|250..=252|254)) {
                status = embedded.unwrap();
                track.status = status;
                message(&mut track.reader, status)?.0
            } else {
                track.reader = saved;
                track.reader.bytes(length)?;
                0
            }
        } else {
            let (message, tempo) = message(&mut track.reader, status)?;
            if let Some(tempo) = tempo {
                self.offset = self.offset.wrapping_add(
                    (tick as i64).wrapping_mul(self.tempo.wrapping_sub(tempo) as i64),
                );
                self.tempo = tempo;
            }
            message
        };
        let time = self
            .offset
            .wrapping_add((tick as i64).wrapping_mul(self.tempo as i64));
        if message == 1 {
            track.done = true;
        } else {
            track.tick = track.tick.wrapping_add(track.reader.varint()? as i32);
        }
        Ok(Some(Event {
            track: index,
            tick,
            time,
            message,
        }))
    }
}
fn message(r: &mut Reader<'_>, status: u8) -> Result<(u32, Option<i32>)> {
    if status == 255 {
        let kind = r.u8()?;
        let len = r.varint()? as usize;
        let bytes = r.bytes(len)?;
        return Ok(match kind {
            47 => (1, None),
            81 => {
                let mut tempo = Reader::new(bytes);
                (2, Some(tempo.u24()? as i32))
            }
            _ => (3, None),
        });
    }
    let size = match status {
        128..=191 | 224..=239 | 242 => 2,
        192..=223 | 241 | 243 => 1,
        _ => 0,
    };
    let mut message = status as u32;
    if size >= 1 {
        message |= (r.u8()? as u32) << 8;
    }
    if size >= 2 {
        message |= (r.u8()? as u32) << 16;
    }
    Ok((message, None))
}
