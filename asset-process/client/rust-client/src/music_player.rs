//! Sample-accurate MIDI scheduling over the native instrument synthesizer.
use crate::{
    midi::{Event, Midi},
    music_synth::Synth,
    Error, Result,
};
pub struct Player {
    pub synth: Synth,
    first: Vec<Event>,
    repeat: Vec<Event>,
    repeated: bool,
    cursor: usize,
    clock: i64,
    origin: i64,
    step: i64,
    looping: bool,
    playing: bool,
}
impl Player {
    pub fn new(data: &[u8], rate: i32, looping: bool) -> Result<Self> {
        if rate < 100 {
            return Err(Error::Invalid("Music output rate"));
        }
        let mut midi = Midi::new(data)?;
        let step = (midi.division as i32).wrapping_mul(1_000_000) / rate;
        if step <= 0 {
            return Err(Error::Invalid("Music clock rate"));
        }
        let mut first = Vec::new();
        while let Some(event) = midi.next_event()? {
            first.push(event);
        }
        midi.reset(0)?;
        let mut repeat = Vec::new();
        if looping {
            while let Some(event) = midi.next_event()? {
                repeat.push(event);
            }
        }
        let playing = !first.is_empty();
        Ok(Self {
            synth: Synth::new(rate),
            first,
            repeat,
            repeated: false,
            cursor: 0,
            clock: 0,
            origin: 0,
            step: step as i64,
            looping,
            playing,
        })
    }
    pub fn empty(rate: i32) -> Self {
        Self {
            synth: Synth::new(rate),
            first: Vec::new(),
            repeat: Vec::new(),
            repeated: false,
            cursor: 0,
            clock: 0,
            origin: 0,
            step: 1,
            looping: false,
            playing: false,
        }
    }
    pub fn install(&mut self, data: &[u8], looping: bool) -> Result<()> {
        let mut next = Self::new(data, self.synth.rate, looping)?;
        std::mem::swap(&mut next.synth, &mut self.synth);
        next.synth.reset();
        *self = next;
        Ok(())
    }
    pub fn playing(&self) -> bool {
        self.playing
    }
    pub fn active(&self) -> bool {
        self.playing || self.synth.active()
    }
    pub fn stop(&mut self) {
        self.playing = false;
        self.synth.reset();
    }
    pub fn mix(&mut self, output: &mut [i32], stereo: bool) {
        let width = if stereo { 2 } else { 1 };
        let mut offset = 0;
        while self.playing && offset < output.len() {
            let events = if self.repeated {
                &self.repeat
            } else {
                &self.first
            };
            let event = events[self.cursor];
            let due = event.time + self.origin;
            let remaining = (output.len() - offset) / width;
            let end = self.clock + self.step * remaining as i64;
            // Events exactly at the buffer boundary belong to the next call.
            if due >= end {
                self.clock = end;
                break;
            }
            let count = ((due - self.clock + self.step - 1) / self.step).max(0) as usize;
            self.synth
                .mix(&mut output[offset..offset + count * width], stereo);
            offset += count * width;
            self.clock += count as i64 * self.step;
            if event.message & 128 != 0 {
                self.synth.message(event.message);
            }
            self.cursor += 1;
            if self.cursor == events.len() {
                if self.looping && event.tick != 0 && !self.repeat.is_empty() {
                    self.origin = due;
                    self.repeated = true;
                    self.cursor = 0;
                } else {
                    self.stop();
                }
            }
        }
        self.synth.mix(&mut output[offset..], stereo);
    }
}
