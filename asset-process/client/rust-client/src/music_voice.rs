//! Per-note pitch, gain, and envelope clocks of the MIDI synthesizer.
use crate::{music_channel::Channel, music_patch::Envelope};

#[derive(Clone, Debug)]
pub struct Voice {
    pub key: i32,
    pub gain: i32,
    pub pan: i32,
    pub pitch: i32,
    pub glide_delta: i32,
    pub glide: i32,
    pub decay: i32,
    pub volume_time: i32,
    pub volume_index: usize,
    pub release_time: i32,
    pub release_index: usize,
    pub age: i32,
    pub vibrato_phase: i32,
}

impl Voice {
    pub fn new(key: u8, velocity: u8, volume: i8, patch_gain: i32, pan: u8, tuning: i16) -> Self {
        let gain = (volume as i32)
            .wrapping_mul(velocity as i32)
            .wrapping_mul(velocity as i32)
            .wrapping_mul(patch_gain)
            .wrapping_add(1024)
            >> 11;
        Self {
            key: key as i32,
            gain,
            pan: pan as i32,
            pitch: ((key as i32) << 8) - (tuning as i32 & 32767),
            glide_delta: 0,
            glide: 0,
            decay: 0,
            volume_time: 0,
            volume_index: 0,
            release_time: -1,
            release_index: 0,
            age: 0,
            vibrato_phase: 0,
        }
    }

    pub fn retarget(&mut self, key: u8) {
        let current = self.pitch.wrapping_add(self.glide_pitch());
        self.pitch = self.pitch.wrapping_add(((key as i32) - self.key) << 8);
        self.glide_delta = current.wrapping_sub(self.pitch);
        self.glide = 4096;
        self.key = key as i32;
    }

    fn glide_pitch(&self) -> i32 {
        self.glide.wrapping_mul(self.glide_delta) >> 12
    }

    pub fn rate(
        &self,
        channel: &Channel,
        envelope: &Envelope,
        sample_rate: i32,
        output_rate: i32,
    ) -> i32 {
        let mut pitch = self
            .pitch
            .wrapping_add(self.glide_pitch())
            .wrapping_add(channel.bend_range.wrapping_mul(channel.bend - 8192) >> 12);
        if envelope.vibrato_rate > 0 && (envelope.vibrato_depth > 0 || channel.modulation > 0) {
            let mut depth = envelope.vibrato_depth << 2;
            let delay = envelope.vibrato_delay << 1;
            if self.age < delay {
                depth = self.age.wrapping_mul(depth) / delay;
            }
            depth += channel.modulation >> 7;
            let sine = ((self.vibrato_phase & 511) as f64 * 0.01227184630308513).sin();
            pitch = pitch.wrapping_add((depth as f64 * sine) as i32);
        }
        ((sample_rate.wrapping_mul(256) as f64 * 2.0f64.powf(3.255208333333333e-4 * pitch as f64)
            / output_rate as f64
            + 0.5) as i32)
            .max(1)
    }

    pub fn volume(&self, channel: &Channel, envelope: &Envelope, master: i32) -> i32 {
        let mut gain = channel
            .expression
            .wrapping_mul(channel.volume)
            .wrapping_add(4096)
            >> 13;
        gain = gain.wrapping_mul(gain).wrapping_add(16384) >> 15;
        gain = self.gain.wrapping_mul(gain).wrapping_add(16384) >> 15;
        gain = gain.wrapping_mul(master).wrapping_add(128) >> 8;
        if envelope.decay > 0 {
            gain = (gain as f64
                * 0.5f64.powf(envelope.decay as f64 * (self.decay as f64 * 1.953125e-5))
                + 0.5) as i32;
        }
        if !envelope.volume.is_empty() {
            gain = gain
                .wrapping_mul(curve(&envelope.volume, self.volume_index, self.volume_time))
                .wrapping_add(32)
                >> 6;
        }
        if self.release_time > 0 && !envelope.release.is_empty() {
            gain = gain
                .wrapping_mul(curve(
                    &envelope.release,
                    self.release_index,
                    self.release_time,
                ))
                .wrapping_add(32)
                >> 6;
        }
        gain
    }

    pub fn stereo_pan(&self, channel: &Channel) -> i32 {
        if channel.pan < 8192 {
            (32 + self.pan * channel.pan) >> 6
        } else {
            16384 - (((128 - self.pan) * (16384 - channel.pan) + 32) >> 6)
        }
    }

    /// Starts a 10 ms control interval. Pitch is sampled before envelope/vibrato advancement.
    pub fn begin_tick(
        &mut self,
        channel: &Channel,
        envelope: &Envelope,
        sample_rate: i32,
        output_rate: i32,
        group_held: bool,
    ) -> (i32, bool) {
        if self.glide > 0 {
            self.glide = (self.glide
                - (16.0 * 2.0f64.powf(4.921259842519685e-4 * channel.portamento as f64) + 0.5)
                    as i32)
                .max(0);
        }
        let rate = self.rate(channel, envelope, sample_rate, output_rate);
        self.age = self.age.wrapping_add(1);
        self.vibrato_phase = self.vibrato_phase.wrapping_add(envelope.vibrato_rate);
        let scale =
            5.086263020833333e-6 * (((self.key - 60) << 8).wrapping_add(self.glide_pitch())) as f64;
        if envelope.decay > 0 {
            self.decay = self
                .decay
                .wrapping_add(increment(envelope.decay_pitch, scale));
        }
        let mut ended = false;
        if !envelope.volume.is_empty() {
            self.volume_time = self
                .volume_time
                .wrapping_add(increment(envelope.volume_pitch, scale));
            advance_curve(&envelope.volume, &mut self.volume_index, self.volume_time);
            ended = self.volume_index == envelope.volume.len() - 2
                && envelope.volume[self.volume_index + 1] == 0;
        }
        if self.release_time >= 0
            && !envelope.release.is_empty()
            && channel.flags & 1 == 0
            && !group_held
        {
            self.release_time = self
                .release_time
                .wrapping_add(increment(envelope.release_pitch, scale));
            advance_curve(
                &envelope.release,
                &mut self.release_index,
                self.release_time,
            );
            ended |= self.release_index == envelope.release.len() - 2;
        }
        (rate, ended)
    }
}

fn increment(pitch: i32, scale: f64) -> i32 {
    if pitch > 0 {
        (128.0 * 2.0f64.powf(pitch as f64 * scale) + 0.5) as i32
    } else {
        128
    }
}
fn advance_curve(curve: &[i8], index: &mut usize, time: i32) {
    while *index < curve.len() - 2 && time > (curve[*index + 2] as u8 as i32) << 8 {
        *index += 2;
    }
}
fn curve(curve: &[i8], index: usize, time: i32) -> i32 {
    let mut value = curve[index + 1] as i32;
    if index < curve.len() - 2 {
        let from = (curve[index] as u8 as i32) << 8;
        let to = (curve[index + 2] as u8 as i32) << 8;
        value += (time - from).wrapping_mul(curve[index + 3] as i32 - value) / (to - from);
    }
    value
}
