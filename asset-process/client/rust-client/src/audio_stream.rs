//! Original fixed-point sample stream, including loop reflections and gain ramps.
use crate::audio_runtime::Sound;
use std::sync::Arc;

pub struct Stream {
    pub sound: Arc<Sound>,
    pub position: i32,
    pub step: i32,
    pub loops: i32,
    pub target: i32,
    pub pan: i32,
    pub gain: [i32; 3],
    pub ramp: i32,
    pub delta: [i32; 3],
    pub finished: bool,
}
fn gains(volume: i32, pan: i32) -> [i32; 3] {
    if pan < 0 {
        [volume, volume, -volume]
    } else {
        [
            volume,
            (volume as f64 * (((16384 - pan) as f32 * 1.2207031e-4f32) as f64).sqrt() + 0.5) as i32,
            (volume as f64 * ((pan as f32 * 1.2207031e-4f32) as f64).sqrt() + 0.5) as i32,
        ]
    }
}
impl Stream {
    pub fn new(sound: Arc<Sound>, step: i32, volume: i32, pan: i32) -> Self {
        Self {
            sound,
            position: 0,
            step,
            loops: 0,
            target: volume,
            pan,
            gain: gains(volume, pan),
            ramp: 0,
            delta: [0; 3],
            finished: false,
        }
    }
    pub fn seek(&mut self, position: i32) {
        self.position = position.clamp(-1, (self.sound.samples.len() as i32) << 8);
    }
    pub fn set_rate(&mut self, rate: i32) {
        self.step = if self.step < 0 { -rate } else { rate };
    }
    pub fn reverse(&mut self) {
        self.step = -self.step.abs();
    }
    pub fn out_of_bounds(&self) -> bool {
        self.position < 0 || self.position >= (self.sound.samples.len() as i32) << 8
    }
    pub fn volume(&self) -> i32 {
        if self.target == i32::MIN {
            0
        } else {
            self.target
        }
    }
    pub fn set_gain(&mut self, volume: i32, pan: i32) {
        self.target = volume;
        self.pan = pan;
        self.ramp = 0;
        self.gain = gains(volume, pan);
    }
    pub fn ramp_to(&mut self, mut frames: i32, volume: i32, pan: i32) {
        if frames == 0 {
            self.set_gain(volume, pan);
            return;
        }
        let target = gains(volume, pan);
        if self.gain[1..] == target[1..] {
            self.ramp = 0;
            return;
        }
        let maximum = (0..3)
            .map(|i| (target[i] - self.gain[i]).abs())
            .max()
            .unwrap();
        frames = frames.min(maximum);
        self.ramp = frames;
        self.target = volume;
        self.pan = pan;
        for i in 0..3 {
            self.delta[i] = (target[i] - self.gain[i]) / frames;
        }
    }
    pub fn fade(&mut self, mut frames: i32) {
        if frames == 0 {
            self.set_gain(0, self.pan);
            self.finished = true;
            return;
        }
        if self.gain[1] == 0 && self.gain[2] == 0 {
            self.ramp = 0;
            self.target = 0;
            self.gain[0] = 0;
            self.finished = true;
            return;
        }
        frames = frames.min(self.gain.iter().map(|g| g.abs()).max().unwrap());
        self.ramp = frames;
        self.target = i32::MIN;
        for i in 0..3 {
            self.delta[i] = -self.gain[i] / frames;
        }
    }
    fn cancel_ramp(&mut self) {
        if self.ramp != 0 {
            if self.target == i32::MIN {
                self.target = 0;
            }
            self.ramp = 0;
            self.gain = gains(self.target, self.pan);
        }
    }
    fn converge(&mut self) {
        let target = if self.target == i32::MIN {
            [0; 3]
        } else {
            gains(self.target, self.pan)
        };
        if self.gain == target {
            if self.target == i32::MIN {
                self.target = 0;
                self.gain = [0; 3];
                self.finished = true;
            } else {
                self.gain = gains(self.target, self.pan);
            }
        } else {
            self.ramp = 0;
            for i in 0..3 {
                let difference = target[i] - self.gain[i];
                self.delta[i] = difference.signum();
                if difference != 0 && (self.ramp == 0 || self.ramp > difference.abs()) {
                    self.ramp = difference.abs();
                }
            }
        }
    }
    fn prepare(&mut self) -> bool {
        if self.finished {
            if self.out_of_bounds() {
                self.cancel_ramp();
            }
            return false;
        }
        let end = (self.sound.samples.len() as i32) << 8;
        if self.sound.loop_end <= self.sound.loop_start {
            self.loops = 0;
        }
        if self.position < 0 {
            if self.step <= 0 {
                self.cancel_ramp();
                self.finished = true;
                return false;
            }
            self.position = 0;
        }
        if self.position >= end {
            if self.step >= 0 {
                self.cancel_ramp();
                self.finished = true;
                return false;
            }
            self.position = end - 1;
        }
        true
    }
    fn boundary(&mut self) {
        let start = (self.sound.loop_start as i32) << 8;
        let end = (self.sound.loop_end as i32) << 8;
        while self.loops != 0
            && ((self.step < 0 && self.position < start)
                || (self.step >= 0 && self.position >= end))
        {
            if self.sound.ping_pong {
                let boundary = if self.step < 0 { start } else { end };
                self.position = boundary + boundary - 1 - self.position;
                self.step = -self.step;
                if self.loops > 0 {
                    self.loops -= 1;
                }
            } else {
                let count = if self.step < 0 {
                    (end - 1 - self.position) / (end - start)
                } else {
                    (self.position - start) / (end - start)
                };
                let count = if self.loops < 0 {
                    count
                } else {
                    count.min(self.loops)
                };
                if self.step < 0 {
                    self.position += (end - start) * count;
                } else {
                    self.position -= (end - start) * count;
                }
                if self.loops > 0 {
                    self.loops -= count;
                }
            }
        }
        if self.loops == 0 {
            let length = (self.sound.samples.len() as i32) << 8;
            if (self.step < 0 && self.position < 0) || (self.step >= 0 && self.position >= length) {
                self.position = if self.step < 0 { -1 } else { length };
                self.cancel_ramp();
                self.finished = true;
            }
        }
    }
    pub fn mix(&mut self, output: &mut [i32], stereo: bool) {
        if !self.prepare() {
            return;
        }
        if self.target == 0 && self.ramp == 0 {
            self.skip(output.len() / if stereo { 2 } else { 1 });
            return;
        }
        // The original aligned ramp helpers update the unused gain lanes even
        // when an offset starts beyond the active loop boundary and consumes
        // no samples. Retrigger offsets can reach this state legitimately.
        if self.ramp > 0 && self.loops != 0 && self.position & 255 == 0 {
            let frames = if self.step == 256 {
                self.sound.loop_end as i32 - (self.position >> 8)
            } else if self.step == -256 {
                (self.position >> 8) - self.sound.loop_start as i32 + 1
            } else {
                0
            };
            if frames < 0 {
                if stereo {
                    self.gain[0] = self.gain[0].wrapping_add(self.delta[0].wrapping_mul(frames));
                } else {
                    for i in 1..3 {
                        self.gain[i] =
                            self.gain[i].wrapping_add(self.delta[i].wrapping_mul(frames));
                    }
                }
            }
        }
        self.boundary();
        let width = if stereo { 2 } else { 1 };
        for frame in output.chunks_exact_mut(width) {
            self.boundary();
            if self.finished {
                break;
            }
            let index = (self.position >> 8) as usize;
            let fraction = self.position & 255;
            let aligned = (self.step == 256 || self.step == -256) && fraction == 0;
            let raw = self.sound.samples[index] as i32;
            let value = if aligned {
                raw << 8
            } else if self.step >= 0 {
                let boundary = if self.loops != 0 {
                    self.sound.loop_end
                } else {
                    self.sound.samples.len()
                };
                let next = if index + 1 < boundary {
                    self.sound.samples[index + 1] as i32
                } else if self.loops == 0 {
                    0
                } else if self.sound.ping_pong {
                    self.sound.samples[boundary - 1] as i32
                } else {
                    self.sound.samples[self.sound.loop_start] as i32
                };
                (raw << 8) + (next - raw) * fraction
            } else {
                let boundary = if self.loops != 0 {
                    self.sound.loop_start
                } else {
                    0
                };
                let previous = if index > boundary {
                    self.sound.samples[index - 1] as i32
                } else if self.loops == 0 {
                    0
                } else if self.sound.ping_pong {
                    self.sound.samples[boundary] as i32
                } else {
                    self.sound.samples[self.sound.loop_end - 1] as i32
                };
                (previous << 8) + (raw - previous) * fraction
            };
            if stereo {
                for side in 0..2 {
                    let amount =
                        if aligned && self.ramp == 0 && self.step == 256 && self.sound.legacy {
                            raw.wrapping_mul(self.gain[side + 1]) << 2
                        } else {
                            (value >> 8).wrapping_mul(self.gain[side + 1]) >> 6
                        };
                    frame[side] = frame[side].wrapping_add(amount);
                }
            } else {
                frame[0] = frame[0].wrapping_add(if aligned {
                    raw.wrapping_mul(self.gain[0]) << 2
                } else {
                    value.wrapping_mul(self.gain[0]) >> 6
                });
            }
            self.position = self.position.wrapping_add(self.step);
            if self.ramp > 0 {
                for i in 0..3 {
                    self.gain[i] = self.gain[i].wrapping_add(self.delta[i]);
                }
                self.ramp -= 1;
                if self.ramp == 0 {
                    self.converge();
                }
            }
            self.boundary();
            if self.finished {
                break;
            }
        }
    }
    pub fn skip(&mut self, mut frames: usize) {
        if self.finished {
            return;
        }
        if self.ramp > 0 {
            if frames >= self.ramp as usize {
                if self.target == i32::MIN {
                    self.target = 0;
                    self.gain = [0; 3];
                    self.finished = true;
                    frames = self.ramp as usize;
                }
                self.ramp = 0;
                self.gain = gains(self.target, self.pan);
            } else {
                for i in 0..3 {
                    self.gain[i] =
                        self.gain[i].wrapping_add(self.delta[i].wrapping_mul(frames as i32));
                }
                self.ramp -= frames as i32;
            }
        }
        // A completed fade still advances by the number of frames consumed by the fade.
        let faded = self.finished;
        self.finished = false;
        if self.prepare() {
            self.position = self
                .position
                .wrapping_add(self.step.wrapping_mul(frames as i32));
            self.boundary();
        }
        self.finished |= faded;
    }
}
