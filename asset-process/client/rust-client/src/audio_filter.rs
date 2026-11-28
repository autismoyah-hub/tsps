//! Cascaded two-pole sound filter coefficients from bi.
use crate::{audio_envelope::Envelope, buffer::Reader, Error, Result};
#[derive(Clone, Debug, Default)]
pub struct Filter {
    pub pairs: [usize; 2],
    gain: [u16; 2],
    frequency: [[[u16; 4]; 2]; 2],
    attenuation: [[[u16; 4]; 2]; 2],
}
pub struct Coefficients {
    pub gain: i32,
    pub counts: [usize; 2],
    pub values: [[i32; 8]; 2],
}
impl Filter {
    pub fn decode(r: &mut Reader<'_>, envelope: &mut Envelope) -> Result<Self> {
        let packed = r.u8()?;
        let mut f = Self {
            pairs: [(packed >> 4) as usize, (packed & 15) as usize],
            ..Self::default()
        };
        if f.pairs.iter().any(|&n| n > 4) {
            return Err(Error::Invalid("sound filter pair count"));
        }
        if packed == 0 {
            return Ok(f);
        }
        f.gain = [r.u16()?, r.u16()?];
        let flags = r.u8()?;
        for channel in 0..2 {
            for pair in 0..f.pairs[channel] {
                f.frequency[channel][0][pair] = r.u16()?;
                f.attenuation[channel][0][pair] = r.u16()?;
            }
        }
        for channel in 0..2 {
            for pair in 0..f.pairs[channel] {
                if flags & (1 << (channel * 4 + pair)) != 0 {
                    f.frequency[channel][1][pair] = r.u16()?;
                    f.attenuation[channel][1][pair] = r.u16()?;
                } else {
                    f.frequency[channel][1][pair] = f.frequency[channel][0][pair];
                    f.attenuation[channel][1][pair] = f.attenuation[channel][0][pair];
                }
            }
        }
        if flags != 0 || f.gain[0] != f.gain[1] {
            envelope.decode_segments(r)?;
        }
        Ok(f)
    }
    pub fn coefficients(&self, t: f32, sample_rate: i32) -> Coefficients {
        let gain_value = (self.gain[0] as f32
            + (self.gain[1] as i32 - self.gain[0] as i32) as f32 * t)
            * 0.0030517578f32;
        let gain = libm::pow(0.1, (gain_value / 20.0) as f64) as f32;
        let mut values = [[0; 8]; 2];
        for channel in 0..2 {
            let mut coefficients = [0.0f32; 8];
            for pair in 0..self.pairs[channel] {
                let attenuation = (self.attenuation[channel][0][pair] as f32
                    + t * (self.attenuation[channel][1][pair] as i32
                        - self.attenuation[channel][0][pair] as i32)
                        as f32)
                    * 0.0015258789f32;
                let radius = 1.0 - libm::pow(10.0, (-attenuation / 20.0) as f64) as f32;
                let frequency = (self.frequency[channel][0][pair] as f32
                    + t * (self.frequency[channel][1][pair] as i32
                        - self.frequency[channel][0][pair] as i32)
                        as f32)
                    * 0.00012207031f32;
                let angle = (32.703197f32 * libm::pow(2.0, frequency as f64) as f32)
                    * std::f32::consts::PI
                    / (sample_rate / 2) as f32;
                let a = -2.0 * radius * libm::cos(angle as f64) as f32;
                let b = radius * radius;
                if pair == 0 {
                    coefficients[0] = a;
                    coefficients[1] = b;
                } else {
                    coefficients[pair * 2 + 1] = coefficients[pair * 2 - 1] * b;
                    coefficients[pair * 2] =
                        coefficients[pair * 2 - 1] * a + coefficients[pair * 2 - 2] * b;
                    for i in (2..=pair * 2 - 1).rev() {
                        coefficients[i] += coefficients[i - 1] * a + coefficients[i - 2] * b;
                    }
                    coefficients[1] += coefficients[0] * a + b;
                    coefficients[0] += a;
                }
            }
            for i in 0..self.pairs[channel] * 2 {
                if channel == 0 {
                    coefficients[i] *= gain;
                }
                values[channel][i] = (coefficients[i] * 65536.0) as i32;
            }
        }
        Coefficients {
            gain: (gain * 65536.0) as i32,
            counts: self.pairs.map(|n| n * 2),
            values,
        }
    }
}
