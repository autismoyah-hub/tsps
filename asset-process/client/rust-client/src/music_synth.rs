//! MIDI note allocation, controllers and sample-voice mixing.
use crate::{
    audio_stream::Stream,
    music_channel::{Actions, Channel},
    music_samples::Instrument,
    music_voice::Voice,
};
use std::{collections::BTreeMap, sync::Arc};
struct Note {
    id: u64,
    channel: usize,
    group: i8,
    envelope: usize,
    instrument: Arc<Instrument>,
    voice: Voice,
    stream: Option<Stream>,
    remaining: usize,
    phase: i32,
}
pub struct Synth {
    pub channels: [Channel; 16],
    pub instruments: BTreeMap<i32, Arc<Instrument>>,
    pub master: i32,
    pub rate: i32,
    keys: Box<[[Option<u64>; 128]; 16]>,
    groups: Box<[[Option<u64>; 128]; 16]>,
    notes: Vec<Note>,
    tails: Vec<Stream>,
    next_id: u64,
}
impl Synth {
    pub fn new(rate: i32) -> Self {
        assert!(rate >= 100);
        Self {
            channels: [Channel::default(); 16],
            instruments: BTreeMap::new(),
            master: 256,
            rate,
            keys: Box::new([[None; 128]; 16]),
            groups: Box::new([[None; 128]; 16]),
            notes: Vec::new(),
            tails: Vec::new(),
            next_id: 0,
        }
    }
    pub fn active(&self) -> bool {
        self.notes.iter().any(|n| n.stream.is_some()) || !self.tails.is_empty()
    }
    pub fn set_program(&mut self, channel: usize, program: i32) {
        self.channels[channel].default_program = program;
        self.channels[channel].bank = program & !127;
        if self.channels[channel].program != program {
            self.groups[channel] = [None; 128];
        }
        self.channels[channel].program = program;
    }
    pub fn message(&mut self, message: u32) {
        let channel = (message & 15) as usize;
        let key = (message >> 8 & 127) as usize;
        let value = (message >> 16 & 127) as u8;
        match message & 240 {
            128 => self.note_off(channel, key),
            144 => {
                if value == 0 {
                    self.note_off(channel, key)
                } else {
                    self.note_on(channel, key, value)
                }
            }
            176 => {
                let actions = self.channels[channel].control(key as u8, value);
                self.actions(channel, actions);
            }
            192 => {
                let actions = self.channels[channel].program_change(key as u8);
                self.actions(channel, actions);
            }
            224 => {
                self.channels[channel].bend = ((message >> 9 & 16256) + (message >> 8 & 127)) as i32
            }
            _ if message & 255 == 255 => self.reset(),
            _ => {}
        }
    }
    pub fn reset(&mut self) {
        for channel in 0..16 {
            let actions = self.channels[channel].reset();
            self.actions(channel, actions);
        }
    }
    fn actions(&mut self, channel: usize, actions: Actions) {
        if actions.clear_groups {
            self.groups[channel] = [None; 128];
        }
        for note in &mut self.notes {
            if note.channel != channel {
                continue;
            }
            if actions.reset_retrigger {
                note.phase = 0;
            }
            if actions.release_portamento
                && self.keys[channel][note.voice.key as usize].is_none()
                && note.voice.release_time < 0
            {
                note.voice.release_time = 0;
            }
            if actions.release_notes && note.voice.release_time < 0 {
                self.keys[channel][note.voice.key as usize] = None;
                note.voice.release_time = 0;
            }
            if actions.silence {
                if let Some(mut stream) = note.stream.take() {
                    stream.fade(self.rate / 100);
                    if stream.ramp > 0 {
                        self.tails.push(stream);
                    }
                }
                if note.voice.release_time < 0 {
                    self.keys[channel][note.voice.key as usize] = None;
                }
            }
        }
        if actions.silence {
            self.notes.retain(|n| n.channel != channel);
        }
    }
    fn note_off(&mut self, channel: usize, key: usize) {
        let Some(id) = self.keys[channel][key].take() else {
            return;
        };
        let release = self.channels[channel].flags & 2 == 0
            || self
                .notes
                .iter()
                .any(|n| n.id != id && n.channel == channel && n.voice.release_time < 0);
        if release {
            if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                note.voice.release_time = 0;
            }
        }
    }
    fn note_on(&mut self, channel: usize, key: usize, velocity: u8) {
        self.note_off(channel, key);
        if self.channels[channel].flags & 2 != 0 {
            if let Some(note) = self
                .notes
                .iter_mut()
                .rev()
                .find(|n| n.channel == channel && n.voice.release_time < 0)
            {
                self.keys[channel][note.voice.key as usize] = None;
                self.keys[channel][key] = Some(note.id);
                note.voice.retarget(key as u8);
                return;
            }
        }
        let Some(instrument) = self
            .instruments
            .get(&self.channels[channel].program)
            .cloned()
        else {
            return;
        };
        let Some(sound) = instrument.samples[key].clone() else {
            return;
        };
        if sound.samples.is_empty() {
            return;
        }
        let patch = &instrument.patch;
        let Some(envelope) = patch.envelope[key].map(|index| &patch.envelopes[index]) else {
            return;
        };
        let voice = Voice::new(
            key as u8,
            velocity,
            patch.volume[key],
            patch.gain,
            patch.pan[key],
            patch.tuning[key],
        );
        let controls = &self.channels[channel];
        let mut stream = Stream::new(
            sound.clone(),
            voice.rate(controls, envelope, sound.rate as i32, self.rate),
            if controls.sample_offset == 0 {
                voice.volume(controls, envelope, self.master)
            } else {
                0
            },
            voice.stereo_pan(controls),
        );
        let looping = patch.tuning[key] < 0;
        if controls.sample_offset != 0 {
            sample_offset(&mut stream, controls.sample_offset, looping);
        }
        if looping {
            stream.loops = -1;
        }
        let id = self.next_id;
        self.next_id += 1;
        let group = patch.group[key];
        if group >= 0 {
            if let Some(previous) = self.groups[channel][group as usize] {
                if let Some(note) = self
                    .notes
                    .iter_mut()
                    .find(|n| n.id == previous && n.voice.release_time < 0)
                {
                    self.keys[channel][note.voice.key as usize] = None;
                    note.voice.release_time = 0;
                }
            }
            self.groups[channel][group as usize] = Some(id);
        }
        self.keys[channel][key] = Some(id);
        let envelope = patch.envelope[key].unwrap();
        self.notes.push(Note {
            id,
            channel,
            group,
            envelope,
            instrument,
            voice,
            stream: Some(stream),
            remaining: 0,
            phase: 0,
        });
    }
    pub fn mix(&mut self, output: &mut [i32], stereo: bool) {
        let width = if stereo { 2 } else { 1 };
        for stream in &mut self.tails {
            stream.mix(output, stereo);
        }
        self.tails.retain(|s| !s.finished);
        for note in &mut self.notes {
            if note.stream.is_none() {
                continue;
            }
            let channel = &self.channels[note.channel];
            let instrument = note.instrument.clone();
            let patch = &instrument.patch;
            // A portamento note retains its original instrument envelope.
            let envelope = &patch.envelopes[note.envelope];
            let mut offset = 0;
            while offset < output.len() {
                if note.remaining == 0 {
                    mix_note(
                        note,
                        channel,
                        self.rate,
                        &mut self.tails,
                        &mut output[offset..],
                        0,
                        stereo,
                    );
                    let stream = note.stream.as_mut().unwrap();
                    if note.voice.release_time >= 0 && stream.out_of_bounds() {
                        note.stream = None;
                        break;
                    }
                    note.remaining = (self.rate / 100) as usize;
                    let held = note.group >= 0
                        && self.groups[note.channel][note.group as usize] == Some(note.id);
                    let (rate, ended) = note.voice.begin_tick(
                        channel,
                        envelope,
                        stream.sound.rate as i32,
                        self.rate,
                        held,
                    );
                    stream.set_rate(rate);
                    if ended {
                        stream.fade(self.rate / 100);
                        stream.mix(&mut output[offset..], stereo);
                        let stream = note.stream.take().unwrap();
                        if stream.ramp > 0 {
                            self.tails.push(stream);
                        }
                        break;
                    }
                    stream.ramp_to(
                        self.rate / 100,
                        note.voice.volume(channel, envelope, self.master),
                        note.voice.stereo_pan(channel),
                    );
                }
                let count = note.remaining.min((output.len() - offset) / width);
                mix_note(
                    note,
                    channel,
                    self.rate,
                    &mut self.tails,
                    &mut output[offset..],
                    count,
                    stereo,
                );
                note.remaining -= count;
                offset += count * width;
            }
        }
        self.notes.retain(|note| {
            if note.stream.is_none() && note.voice.release_time >= 0 {
                if note.group > 0 && self.groups[note.channel][note.group as usize] == Some(note.id)
                {
                    self.groups[note.channel][note.group as usize] = None;
                }
                false
            } else {
                true
            }
        });
    }
}
fn sample_offset(stream: &mut Stream, offset: i32, looping: bool) {
    let length = stream.sound.samples.len() as i64;
    let position = if looping && stream.sound.ping_pong {
        let position = ((length * 2 - stream.sound.loop_start as i64) * offset as i64) >> 6;
        if position >= length << 8 {
            stream.reverse();
            (length << 9) - 1 - position
        } else {
            position
        }
    } else {
        length * offset as i64 >> 6
    };
    stream.seek(position as i32);
}
fn mix_note(
    note: &mut Note,
    channel: &Channel,
    rate: i32,
    tails: &mut Vec<Stream>,
    output: &mut [i32],
    mut count: usize,
    stereo: bool,
) {
    let width = if stereo { 2 } else { 1 };
    let mut offset = 0;
    if channel.flags & 4 != 0 && note.voice.release_time < 0 {
        let increment = channel.retrigger_rate / rate;
        if increment > 0 {
            loop {
                let until = ((1048575 + increment - note.phase) / increment) as usize;
                if until > count {
                    note.phase += count as i32 * increment;
                    break;
                }
                let stream = note.stream.as_mut().unwrap();
                stream.mix(&mut output[offset..offset + until * width], stereo);
                offset += until * width;
                count -= until;
                note.phase += increment * until as i32 - 1048576;
                let fade = (rate / 100).min(262144 / increment);
                let mut old = note.stream.take().unwrap();
                let volume = old.volume();
                let mut fresh = Stream::new(
                    old.sound.clone(),
                    old.step.abs(),
                    if channel.sample_offset == 0 {
                        volume
                    } else {
                        0
                    },
                    old.pan,
                );
                let looping = note.instrument.patch.tuning[note.voice.key as usize] < 0;
                if channel.sample_offset != 0 {
                    sample_offset(&mut fresh, channel.sample_offset, looping);
                    fresh.ramp_to(fade, volume, old.pan);
                }
                if looping {
                    fresh.loops = -1;
                }
                old.fade(fade);
                old.mix(&mut output[offset..], stereo);
                if old.ramp > 0 {
                    tails.push(old);
                }
                note.stream = Some(fresh);
            }
        }
    }
    note.stream
        .as_mut()
        .unwrap()
        .mix(&mut output[offset..offset + count * width], stereo);
}
