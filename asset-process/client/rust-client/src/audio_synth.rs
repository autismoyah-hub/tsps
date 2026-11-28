//! Cache sound-effect instruments and native 22,050 Hz synthesis.
use crate::{
    audio_envelope::{Cursor, Envelope},
    audio_filter::Filter,
    buffer::Reader,
    Error, Result,
};
use std::sync::OnceLock;
#[derive(Clone, Debug)]
pub struct Instrument {
    pitch: Envelope,
    volume: Envelope,
    pitch_mod: Option<(Envelope, Envelope)>,
    volume_mod: Option<(Envelope, Envelope)>,
    gate: Option<(Envelope, Envelope)>,
    oscillators: Vec<(i32, i32, i32)>,
    delay: i32,
    decay: i32,
    pub duration: i32,
    pub offset: i32,
    filter: Filter,
    filter_envelope: Envelope,
}
fn optional(r: &mut Reader<'_>) -> Result<Option<(Envelope, Envelope)>> {
    if r.clone().u8()? == 0 {
        r.u8()?;
        Ok(None)
    } else {
        Ok(Some((Envelope::decode(r)?, Envelope::decode(r)?)))
    }
}
fn tables() -> &'static (Vec<i32>, Vec<i32>) {
    static TABLES: OnceLock<(Vec<i32>, Vec<i32>)> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut seed = 25214903917u64;
        let noise = (0..32768)
            .map(|_| {
                seed = seed.wrapping_mul(25214903917).wrapping_add(11) & ((1 << 48) - 1);
                ((seed >> 16) as i32 & 2) - 1
            })
            .collect();
        let sine = (0..32768)
            .map(|i| (libm::sin(i as f64 / 5215.1903) * 16384.0) as i32)
            .collect();
        (noise, sine)
    })
}
fn wave(phase: i32, amplitude: i32, form: u8) -> i32 {
    match form {
        1 => {
            if phase & 32767 < 16384 {
                amplitude
            } else {
                amplitude.wrapping_neg()
            }
        }
        2 => tables().1[(phase & 32767) as usize].wrapping_mul(amplitude) >> 14,
        3 => ((phase & 32767).wrapping_mul(amplitude) >> 14).wrapping_sub(amplitude),
        4 => tables().0[((phase / 2607) & 32767) as usize].wrapping_mul(amplitude),
        _ => 0,
    }
}
impl Instrument {
    pub fn decode(r: &mut Reader<'_>) -> Result<Self> {
        let pitch = Envelope::decode(r)?;
        let volume = Envelope::decode(r)?;
        let pitch_mod = optional(r)?;
        let volume_mod = optional(r)?;
        let gate = optional(r)?;
        let mut oscillators = Vec::new();
        for _ in 0..10 {
            let amplitude = r.unsigned_smart()?;
            if amplitude == 0 {
                break;
            }
            if oscillators.len() == 5 {
                return Err(Error::Invalid("sound oscillator count"));
            }
            oscillators.push((
                amplitude as i32,
                r.signed_smart()? as i32,
                r.unsigned_smart()? as i32,
            ));
        }
        let delay = r.unsigned_smart()? as i32;
        let decay = r.unsigned_smart()? as i32;
        let duration = r.u16()? as i32;
        let offset = r.u16()? as i32;
        let mut filter_envelope = Envelope::default();
        let filter = Filter::decode(r, &mut filter_envelope)?;
        Ok(Self {
            pitch,
            volume,
            pitch_mod,
            volume_mod,
            gate,
            oscillators,
            delay,
            decay,
            duration,
            offset,
            filter,
            filter_envelope,
        })
    }
    pub fn synthesize(&self, sample_rate: i32) -> Result<Vec<i32>> {
        if !(1..=192000).contains(&sample_rate) {
            return Err(Error::Invalid("sound sample rate"));
        }
        let count = (self.duration as i64 * sample_rate as i64 / 1000) as usize;
        if count > 220500 {
            return Err(Error::Invalid("instrument sample limit"));
        }
        let mut out = vec![0i32; count];
        if self.duration < 10 {
            return Ok(out);
        }
        let ratio = count as f64 / self.duration as f64;
        let (mut pitch, mut volume) = (Cursor::default(), Cursor::default());
        let mut pc = [Cursor::default(), Cursor::default()];
        let mut vc = [Cursor::default(), Cursor::default()];
        let mut phases = [0i32; 5];
        let (mut pitch_phase, mut volume_phase) = (0i32, 0i32);
        let mod_steps = |m: &Option<(Envelope, Envelope)>| {
            m.as_ref().map_or([0; 2], |(e, _)| {
                [
                    ((e.end.wrapping_sub(e.start)) as f64 * 32.768 / ratio) as i32,
                    (e.start as f64 * 32.768 / ratio) as i32,
                ]
            })
        };
        let pm = mod_steps(&self.pitch_mod);
        let vm = mod_steps(&self.volume_mod);
        let oscillators = self
            .oscillators
            .iter()
            .map(|&(volume, semitone, delay)| {
                (
                    (volume << 14) / 100,
                    (delay as f64 * ratio) as usize,
                    (self.pitch.end.wrapping_sub(self.pitch.start) as f64
                        * 32.768
                        * libm::pow(1.0057929410678534, semitone as f64)
                        / ratio) as i32,
                    (self.pitch.start as f64 * 32.768 / ratio) as i32,
                )
            })
            .collect::<Vec<_>>();
        for sample in 0..count {
            let mut p = pitch.next(&self.pitch, count as i32);
            let mut v = volume.next(&self.volume, count as i32);
            if let Some((rate, amplitude)) = &self.pitch_mod {
                let r = pc[0].next(rate, count as i32);
                let a = pc[1].next(amplitude, count as i32);
                p = p.wrapping_add(wave(pitch_phase, a, rate.form) >> 1);
                pitch_phase =
                    pitch_phase.wrapping_add((r.wrapping_mul(pm[0]) >> 16).wrapping_add(pm[1]));
            }
            if let Some((rate, amplitude)) = &self.volume_mod {
                let r = vc[0].next(rate, count as i32);
                let a = vc[1].next(amplitude, count as i32);
                v = v.wrapping_mul((wave(volume_phase, a, rate.form) >> 1).wrapping_add(32768))
                    >> 15;
                volume_phase =
                    volume_phase.wrapping_add((r.wrapping_mul(vm[0]) >> 16).wrapping_add(vm[1]));
            }
            for (i, &(amplitude, delay, step, base)) in oscillators.iter().enumerate() {
                let index = sample + delay;
                if index < count {
                    out[index] = out[index].wrapping_add(wave(
                        phases[i],
                        v.wrapping_mul(amplitude) >> 15,
                        self.pitch.form,
                    ));
                    phases[i] = phases[i]
                        .wrapping_add(p.wrapping_mul(step) >> 16)
                        .wrapping_add(base);
                }
            }
        }
        if let Some((release, attack)) = &self.gate {
            let mut a = Cursor::default();
            let mut b = Cursor::default();
            let mut time = 0i32;
            let mut muted = true;
            for value in &mut out {
                let x = a.next(release, count as i32);
                let y = b.next(attack, count as i32);
                let threshold = release.start.wrapping_add(
                    release
                        .end
                        .wrapping_sub(release.start)
                        .wrapping_mul(if muted { x } else { y })
                        >> 8,
                );
                time = time.wrapping_add(256);
                if time >= threshold {
                    time = 0;
                    muted = !muted;
                }
                if muted {
                    *value = 0;
                }
            }
        }
        if self.delay > 0 && self.decay > 0 {
            let delay = (self.delay as f64 * ratio) as usize;
            for i in delay..count {
                out[i] = out[i].wrapping_add(out[i - delay].wrapping_mul(self.decay) / 100);
            }
        }
        if self.filter.pairs != [0, 0] {
            let mut envelope = Cursor::default();
            let mut t = envelope.next(&self.filter_envelope, count as i32 + 1);
            let mut coefficients = self.filter.coefficients(t as f32 / 65536.0, sample_rate);
            let [forward, back] = coefficients.counts;
            if count >= forward + back {
                for i in 0..count {
                    if i != 0 && i % 128 == 0 && i < count - forward {
                        coefficients = self.filter.coefficients(t as f32 / 65536.0, sample_rate);
                    }
                    let mul = |a: i32, b: i32| ((a as i64 * b as i64) >> 16) as i32;
                    let mut value = if i + forward < count {
                        mul(out[i + forward], coefficients.gain)
                    } else {
                        0
                    };
                    for j in (i + forward).saturating_sub(count)..forward {
                        value = value
                            .wrapping_add(mul(out[i + forward - 1 - j], coefficients.values[0][j]));
                    }
                    for j in 0..back.min(i) {
                        value = value.wrapping_sub(mul(out[i - 1 - j], coefficients.values[1][j]));
                    }
                    out[i] = value;
                    t = envelope.next(&self.filter_envelope, count as i32 + 1);
                }
            }
        }
        for value in &mut out {
            *value = (*value).clamp(-32768, 32767);
        }
        Ok(out)
    }
}
#[derive(Clone, Debug)]
pub struct Effect {
    pub instruments: Vec<Instrument>,
    pub loop_start: i32,
    pub loop_end: i32,
}
impl Effect {
    pub fn trim_delay(&mut self) -> i32 {
        let mut delay = self
            .instruments
            .iter()
            .map(|i| i.offset / 20)
            .min()
            .unwrap_or(9999999);
        if self.loop_start < self.loop_end {
            delay = delay.min(self.loop_start / 20);
        }
        if delay == 9999999 || delay == 0 {
            return 0;
        }
        for i in &mut self.instruments {
            i.offset -= delay * 20;
        }
        if self.loop_start < self.loop_end {
            self.loop_start -= delay * 20;
            self.loop_end -= delay * 20;
        }
        delay
    }
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let mut instruments = Vec::new();
        for _ in 0..10 {
            if r.clone().u8()? == 0 {
                r.u8()?;
            } else {
                instruments.push(Instrument::decode(&mut r)?);
            }
        }
        let loop_start = r.u16()? as i32;
        let loop_end = r.u16()? as i32;
        if r.remaining() != 0 {
            return Err(Error::Invalid("sound effect trailing data"));
        }
        Ok(Self {
            instruments,
            loop_start,
            loop_end,
        })
    }
    pub fn synthesize(&self, legacy_eight_bit: bool) -> Result<Vec<i16>> {
        let length = self
            .instruments
            .iter()
            .map(|i| i.duration + i.offset)
            .max()
            .unwrap_or(0) as usize
            * 22050
            / 1000;
        let mut out = vec![0i16; length];
        for instrument in &self.instruments {
            let offset = instrument.offset as usize * 22050 / 1000;
            for (i, sample) in instrument.synthesize(22050)?.into_iter().enumerate() {
                let value = out[i + offset] as i32
                    + if legacy_eight_bit {
                        sample >> 8
                    } else {
                        sample
                    };
                out[i + offset] = value.clamp(
                    if legacy_eight_bit { -128 } else { -32768 },
                    if legacy_eight_bit { 127 } else { 32767 },
                ) as i16;
            }
        }
        Ok(out)
    }
}
