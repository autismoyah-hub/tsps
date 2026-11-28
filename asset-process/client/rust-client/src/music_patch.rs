//! Instrument sample maps, envelopes and key scaling (ml / nz).
use crate::{buffer::Reader, Error, Result};
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Envelope {
    pub volume: Vec<i8>,
    pub release: Vec<i8>,
    pub decay: i32,
    pub volume_pitch: i32,
    pub release_pitch: i32,
    pub decay_pitch: i32,
    pub vibrato_rate: i32,
    pub vibrato_depth: i32,
    pub vibrato_delay: i32,
}
#[derive(Debug)]
pub struct Patch {
    pub tuning: [i16; 128],
    pub volume: [i8; 128],
    pub pan: [u8; 128],
    pub group: [i8; 128],
    pub samples: [u32; 128],
    pub envelopes: Vec<Envelope>,
    pub envelope: [Option<usize>; 128],
    pub gain: i32,
}
fn runs(r: &mut Reader<'_>) -> Result<Vec<i8>> {
    let mut out = Vec::new();
    loop {
        let v = r.u8()?;
        if v == 0 {
            break;
        }
        out.push(v as i8);
    }
    Ok(out)
}
fn run(data: &[i8], index: &mut usize) -> i32 {
    if let Some(&v) = data.get(*index) {
        *index += 1;
        v as i32
    } else {
        -1
    }
}
fn times(r: &mut Reader<'_>, points: &mut [i8], first: bool) -> Result<()> {
    if points.is_empty() {
        return Ok(());
    }
    let mut time = if first { r.u8()? as i32 } else { 0 };
    points[0] = time as i8;
    for i in (2..points.len()).step_by(2) {
        time += 1 + r.u8()? as i32;
        points[i] = time as i8;
    }
    Ok(())
}
fn curve(points: &[i8], mut apply: impl FnMut(usize, i32), double: bool) -> Result<()> {
    if points.is_empty() {
        return Ok(());
    }
    let scale = if double { 2 } else { 1 };
    let mut x = points[0] as i32;
    let mut value = points[1] as i32 * scale;
    if !(0..128).contains(&x) {
        return Err(Error::Invalid("patch curve key"));
    }
    for i in 0..x {
        apply(i as usize, value);
    }
    for pair in points[2..].chunks_exact(2) {
        let next = pair[0] as i32;
        let target = pair[1] as i32 * scale;
        if next <= x || next >= 128 {
            return Err(Error::Invalid("patch curve order"));
        }
        let width = next - x;
        let mut numerator = width / 2 + value * width;
        for i in x..next {
            let sign = (numerator as u32 >> 31) as i32;
            let v = (numerator + sign) / width - sign;
            apply(i as usize, v);
            numerator += target - value;
        }
        x = next;
        value = target;
    }
    for i in x..128 {
        apply(i as usize, value);
    }
    Ok(())
}
impl Patch {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let groups = runs(&mut r)?;
        let group_values = r.bytes(groups.len() + 1)?;
        let pans = runs(&mut r)?;
        let pan_values = r.bytes(pans.len() + 1)?;
        let regions = runs(&mut r)?;
        let mut mapping = vec![0usize; regions.len() + 1];
        let mut count = mapping.len();
        if count > 1 {
            mapping[1] = 1;
            let mut previous = 1;
            count = 2;
            for value in &mut mapping[2..] {
                let n = r.u8()? as usize;
                if n == 0 {
                    previous = count;
                    count += 1;
                } else {
                    previous = if n <= previous { n - 1 } else { n };
                }
                *value = previous;
            }
        }
        let mut envelopes = vec![Envelope::default(); count];
        for e in &mut envelopes {
            e.volume = vec![0; r.u8()? as usize * 2];
            let size = r.u8()? as usize;
            if size > 0 {
                e.release = vec![0; size * 2 + 2];
                e.release[1] = 64;
            }
        }
        let mut volume_curve = vec![0i8; r.u8()? as usize * 2];
        let mut pan_curve = vec![0i8; r.u8()? as usize * 2];
        let sample_runs = runs(&mut r)?;
        let mut patch = Self {
            tuning: [0; 128],
            volume: [0; 128],
            pan: [0; 128],
            group: [0; 128],
            samples: [0; 128],
            envelopes,
            envelope: [None; 128],
            gain: 0,
        };
        let mut sum = 0i32;
        for v in &mut patch.tuning {
            sum += r.u8()? as i32;
            *v = sum as i16;
        }
        sum = 0;
        for v in &mut patch.tuning {
            sum += r.u8()? as i32;
            *v = v.wrapping_add((sum << 8) as i16);
        }
        let (mut remaining, mut index, mut sample) = (0, 0, 0u32);
        for i in 0..128 {
            if remaining == 0 {
                remaining = run(&sample_runs, &mut index);
                sample = r.varint()?;
            }
            patch.tuning[i] =
                patch.tuning[i].wrapping_add(((sample.wrapping_sub(1) & 2) << 14) as i16);
            patch.samples[i] = sample;
            remaining -= 1;
        }
        let (mut remaining, mut index, mut cursor, mut group) = (0, 0, 0, 0i8);
        for i in 0..128 {
            if patch.samples[i] != 0 {
                if remaining == 0 {
                    remaining = run(&groups, &mut index);
                    group =
                        (*group_values.get(cursor).ok_or(Error::Truncated)? as i8).wrapping_sub(1);
                    cursor += 1;
                }
                patch.group[i] = group;
                remaining -= 1;
            }
        }
        let (mut remaining, mut index, mut cursor, mut pan) = (0, 0, 0, 0u8);
        for i in 0..128 {
            if patch.samples[i] != 0 {
                if remaining == 0 {
                    remaining = run(&pans, &mut index);
                    pan = (((*pan_values.get(cursor).ok_or(Error::Truncated)? as i8 as i32) + 16)
                        << 2) as u8;
                    cursor += 1;
                }
                patch.pan[i] = pan;
                remaining -= 1;
            }
        }
        let (mut remaining, mut index, mut envelope) = (0, 0, 0);
        for i in 0..128 {
            if patch.samples[i] != 0 {
                if remaining == 0 {
                    envelope = *mapping
                        .get(index)
                        .ok_or(Error::Invalid("patch envelope map"))?;
                    remaining = run(&regions, &mut index);
                }
                if envelope >= count {
                    return Err(Error::Invalid("patch envelope index"));
                }
                patch.envelope[i] = Some(envelope);
                remaining -= 1;
            }
        }
        let (mut remaining, mut index, mut volume) = (0, 0, 0i8);
        for i in 0..128 {
            if remaining == 0 {
                remaining = run(&sample_runs, &mut index);
                if patch.samples[i] > 0 {
                    volume = r.u8()?.wrapping_add(1) as i8;
                }
            }
            patch.volume[i] = volume;
            remaining -= 1;
        }
        patch.gain = r.u8()? as i32 + 1;
        for e in &mut patch.envelopes {
            for i in (1..e.volume.len()).step_by(2) {
                e.volume[i] = r.u8()? as i8;
            }
            for i in (3..e.release.len().saturating_sub(2)).step_by(2) {
                e.release[i] = r.u8()? as i8;
            }
        }
        for points in [&mut volume_curve, &mut pan_curve] {
            for i in (1..points.len()).step_by(2) {
                points[i] = r.u8()? as i8;
            }
        }
        for e in &mut patch.envelopes {
            times(&mut r, &mut e.release, false)?;
        }
        for e in &mut patch.envelopes {
            times(&mut r, &mut e.volume, false)?;
        }
        times(&mut r, &mut volume_curve, true)?;
        curve(
            &volume_curve,
            |i, value| patch.volume[i] = ((patch.volume[i] as i32 * value + 32) >> 6) as i8,
            false,
        )?;
        times(&mut r, &mut pan_curve, true)?;
        curve(
            &pan_curve,
            |i, value| patch.pan[i] = (patch.pan[i] as i32 + value).clamp(0, 128) as u8,
            true,
        )?;
        for e in &mut patch.envelopes {
            e.decay = r.u8()? as i32;
        }
        for e in &mut patch.envelopes {
            if !e.volume.is_empty() {
                e.volume_pitch = r.u8()? as i32;
            }
            if !e.release.is_empty() {
                e.release_pitch = r.u8()? as i32;
            }
            if e.decay > 0 {
                e.decay_pitch = r.u8()? as i32;
            }
        }
        for e in &mut patch.envelopes {
            e.vibrato_rate = r.u8()? as i32;
        }
        for e in &mut patch.envelopes {
            if e.vibrato_rate > 0 {
                e.vibrato_depth = r.u8()? as i32;
            }
        }
        for e in &mut patch.envelopes {
            if e.vibrato_depth > 0 {
                e.vibrato_delay = r.u8()? as i32;
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("patch trailing bytes"));
        }
        Ok(patch)
    }
}
