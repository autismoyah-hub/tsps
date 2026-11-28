//! Maya animation curves, ported from sn.yf/ty and polynomial root search.
use crate::{buffer::Reader, Error, Result};
#[derive(Clone, Debug)]
pub struct Key {
    pub time: i16,
    pub value: f32,
    pub incoming: [f32; 2],
    pub outgoing: [f32; 2],
}
#[derive(Clone, Debug)]
pub struct Curve {
    pub before: u8,
    pub after: u8,
    pub weighted: bool,
    pub keys: Vec<Key>,
}
fn polynomial(c: &[f32], x: f32) -> f32 {
    let mut value = *c.last().unwrap();
    for &v in c[..c.len() - 1].iter().rev() {
        value = v + x * value;
    }
    value
}
fn brent(c: &[f32], mut a: f32, mut b: f32) -> f32 {
    let (mut fa, mut fb) = (polynomial(c, a), polynomial(c, b));
    if fa.abs() < f32::EPSILON {
        return a;
    }
    if fb.abs() < f32::EPSILON {
        return b;
    }
    let (mut point, mut fp, mut d, mut e) = (a, fa, b - a, b - a);
    let mut reset = true;
    for _ in 0..128 {
        if reset {
            point = a;
            fp = fa;
            d = b - a;
            e = d;
            reset = false;
        }
        if fp.abs() < fb.abs() {
            a = b;
            b = point;
            point = a;
            fa = fb;
            fb = fp;
            fp = fa;
        }
        let tolerance = 2.0 * f32::EPSILON * b.abs();
        let midpoint = (point - b) * 0.5;
        if midpoint.abs() <= tolerance || fb == 0.0 {
            return b;
        }
        if e.abs() < tolerance || fa.abs() <= fb.abs() {
            d = midpoint;
            e = d;
        } else {
            let mut ratio = fb / fa;
            let (mut p, mut q) = if a == point {
                (ratio * (2.0 * midpoint), 1.0 - ratio)
            } else {
                let q = fa / fp;
                let r = fb / fp;
                (
                    ((q - r) * (q * (2.0 * midpoint)) - (b - a) * (r - 1.0)) * ratio,
                    (q - 1.0) * (r - 1.0) * (ratio - 1.0),
                )
            };
            if p > 0.0 {
                q = -q;
            } else {
                p = -p;
            }
            ratio = e;
            e = d;
            if 2.0 * p < 3.0 * midpoint * q - (tolerance * q).abs() && p < (0.5 * ratio * q).abs() {
                d = p / q;
            } else {
                d = midpoint;
                e = d;
            }
        }
        a = b;
        fa = fb;
        if d.abs() > tolerance {
            b += d;
        } else if midpoint > 0.0 {
            b += tolerance;
        } else {
            b -= tolerance;
        }
        fb = polynomial(c, b);
        if fb * (fp / fp.abs()) > 0.0 {
            reset = true;
        }
    }
    b
}
fn roots(
    input: &[f32],
    low: f32,
    include_low: bool,
    high: f32,
    include_high: bool,
) -> Option<Vec<f32>> {
    let mut degree = input.len() - 1;
    let sum = input.iter().map(|v| v.abs()).sum::<f32>();
    let tolerance = (low.abs() + high.abs()) * (degree + 1) as f32 * f32::EPSILON;
    if sum <= tolerance {
        return None;
    }
    let mut c = input.iter().map(|&v| 1.0 / sum * v).collect::<Vec<_>>();
    while degree > 0 && c[degree].abs() < tolerance {
        degree -= 1;
    }
    c.truncate(degree + 1);
    let mut out = Vec::new();
    if degree == 0 {
        return Some(out);
    }
    if degree == 1 {
        let mut root = -c[0] / c[1];
        if (if include_low {
            low < tolerance + root
        } else {
            low < root - tolerance
        }) && (if include_high {
            high > root - tolerance
        } else {
            high > tolerance + root
        }) {
            if include_low && root < low {
                root = low;
            } else if include_high && root > high {
                root = high;
            }
            out.push(root);
        }
        return Some(out);
    }
    let derivative = (1..=degree).map(|i| c[i] * i as f32).collect::<Vec<_>>();
    let Some(stationary) = roots(&derivative, low, false, high, false) else {
        return Some(out);
    };
    let (mut last, mut last_value, mut skip) = (low, polynomial(&c, low), false);
    if last_value.abs() <= tolerance && include_low {
        out.push(low);
    }
    for i in 0..=stationary.len() {
        let end = if i == stationary.len() {
            skip = false;
            high
        } else {
            stationary[i]
        };
        let value = polynomial(&c, end);
        if skip {
            skip = false;
        } else if value.abs() < tolerance {
            if i != stationary.len() || include_high {
                out.push(end);
                skip = true;
            }
        } else if (last_value < 0.0 && value > 0.0) || (last_value > 0.0 && value < 0.0) {
            let root = brent(&c, last, end);
            out.push(root);
            if out.len() > 1 {
                let n = out.len();
                if out[n - 2] >= out[n - 1] - tolerance {
                    out[n - 2] = (out[n - 2] + out[n - 1]) * 0.5;
                    out.pop();
                }
            }
        }
        last = end;
        last_value = value;
    }
    Some(out)
}
impl Curve {
    pub fn decode(r: &mut Reader<'_>) -> Result<Self> {
        let n = r.u16()? as usize;
        r.u8()?;
        let before = r.u8()?;
        let after = r.u8()?;
        let weighted = r.u8()? != 0;
        if before > 4 || after > 4 || n == 0 {
            return Err(Error::Invalid("curve header"));
        }
        let mut keys = Vec::with_capacity(n);
        for _ in 0..n {
            keys.push(Key {
                time: r.u16()? as i16,
                value: f32::from_bits(r.u32()?),
                incoming: [f32::from_bits(r.u32()?), f32::from_bits(r.u32()?)],
                outgoing: [f32::from_bits(r.u32()?), f32::from_bits(r.u32()?)],
            });
        }
        if keys.windows(2).any(|w| w[0].time > w[1].time) {
            return Err(Error::Invalid("curve key times"));
        }
        Ok(Self {
            before,
            after,
            weighted,
            keys,
        })
    }
    pub fn evaluate(&self, time: f32) -> f32 {
        if self.keys.is_empty() || !time.is_finite() {
            return 0.0;
        }
        let first = &self.keys[0];
        let last = self.keys.last().unwrap();
        if time < first.time as f32 {
            return self.extrapolate(time, true);
        }
        if time > last.time as f32 {
            return self.extrapolate(time, false);
        }
        let index = self
            .keys
            .partition_point(|k| k.time as i32 <= time as i32)
            .saturating_sub(1);
        let a = &self.keys[index];
        if a.outgoing == [0.0; 2] {
            return a.value;
        }
        if a.outgoing == [f32::MAX; 2] {
            return if time != a.time as f32 && index + 1 < self.keys.len() {
                self.keys[index + 1].value
            } else {
                a.value
            };
        }
        let Some(b) = self.keys.get(index + 1) else {
            return a.value;
        };
        let (x0, y0, x3, y3) = (a.time as f32, a.value, b.time as f32, b.value);
        let x1 = x0 + 0.33333334 * a.outgoing[0];
        let mut y1 = a.outgoing[1] * 0.33333334 + y0;
        let x2 = x3 - 0.33333334 * b.incoming[0];
        let mut y2 = y3 - b.incoming[1] * 0.33333334;
        let span = x3 - x0;
        if !self.weighted {
            let slope0 = if x1 - x0 != 0.0 {
                (y1 - y0) / (x1 - x0)
            } else {
                0.0
            };
            let slope1 = if x3 - x2 != 0.0 {
                (y3 - y2) / (x3 - x2)
            } else {
                0.0
            };
            let dy = y3 - y0;
            let inv = 1.0 / (span * span);
            let m0 = span * slope0;
            let m1 = span * slope1;
            let c0 = inv * (m0 + m1 - dy - dy) / span;
            let c1 = (dy + dy + dy - m0 - m0 - m1) * inv;
            let dt = time - x0;
            return y0 + dt * (slope0 + (c1 + dt * c0) * dt);
        }
        let (mut u, mut v) = ((x1 - x0) / span, (x2 - x0) / span);
        let linear = u == 0.33333334 && v == 0.6666667;
        let (original_u, original_v) = (u, v);
        u = u.max(0.0);
        v = v.min(1.0);
        let epsilon = f32::EPSILON;
        if u > 1.0 || v < -1.0 {
            v = 1.0 - v;
            v = v.max(0.0);
            if u > 1.0 || v > 1.0 {
                let curvature = 1.0 + u * (u - 2.0 + v) + (v - 2.0) * v;
                if curvature + epsilon > 0.0 {
                    if epsilon + u < 1.3333334 {
                        let p = u - 2.0;
                        let q = u - 1.0;
                        let root = (p * p - q * 4.0 * q).sqrt();
                        let upper = (-p + root) * 0.5;
                        if v + epsilon > upper {
                            v = upper - epsilon;
                        } else {
                            let lower = 0.5 * (-p - root);
                            if v < lower + epsilon {
                                v = lower + epsilon;
                            }
                        }
                    } else {
                        u = 1.3333334 - epsilon;
                        v = 0.33333334 - epsilon;
                    }
                }
            }
            v = 1.0 - v;
        }
        if u != original_u && original_u != 0.0 {
            y1 = y0 + (y1 - y0) * u / original_u;
        }
        if v != original_v && original_v != 0.0 {
            y2 = y3 - (y3 - y2) * (1.0 - v) / (1.0 - original_v);
        }
        let d0 = u;
        let d1 = v - u;
        let d2 = 1.0 - v;
        let d = d1 - d0;
        let xcoeff = [0.0, d0 + d0 + d0, d + d + d, d2 - d1 - d];
        let d0 = y1 - y0;
        let d1 = y2 - y1;
        let d2 = y3 - y2;
        let d = d1 - d0;
        let ycoeff = [y0, d0 + d0 + d0, d + d + d, d2 - d1 - d];
        let fraction = if time == x0 {
            0.0
        } else if time == x3 {
            1.0
        } else {
            (time - x0) / (x3 - x0)
        };
        let t = if linear {
            fraction
        } else {
            let c = [-fraction, xcoeff[1], xcoeff[2], xcoeff[3]];
            let rs = roots(&c, 0.0, true, 1.0, true).unwrap_or_default();
            if rs.len() == 1 {
                rs[0]
            } else {
                0.0
            }
        };
        t * (ycoeff[1] + (t * ycoeff[3] + ycoeff[2]) * t) + ycoeff[0]
    }
    fn extrapolate(&self, time: f32, before: bool) -> f32 {
        let a = &self.keys[0];
        let b = self.keys.last().unwrap();
        let mode = if before { self.before } else { self.after };
        if mode == 0 {
            return if before { a.value } else { b.value };
        }
        let (start, end) = (a.time as f32, b.time as f32);
        let span = end - start;
        if span == 0.0 {
            return a.value;
        }
        let ratio = if time > end {
            (time - end) / span
        } else {
            (time - start) / span
        };
        let integral = ratio as i32 as f32;
        let mut local = (ratio - integral).abs() * span;
        let cycles = (integral + 1.0).abs();
        let half = cycles / 2.0;
        let odd = half - half as i32 as f32;
        if before {
            match mode {
                4 => {
                    if odd != 0.0 {
                        local += start;
                    } else {
                        local = end - local;
                    }
                }
                2 | 3 => local = end - local,
                1 => {
                    return if a.incoming[0] != 0.0 {
                        a.value - a.incoming[1] * (start - time) / a.incoming[0]
                    } else {
                        a.value
                    }
                }
                _ => {}
            }
        } else {
            match mode {
                4 => {
                    if odd != 0.0 {
                        local = end - local;
                    } else {
                        local += start;
                    }
                }
                2 | 3 => local += start,
                1 => {
                    return if b.outgoing[0] != 0.0 {
                        b.value + b.outgoing[1] * (time - end) / b.outgoing[0]
                    } else {
                        b.value
                    }
                }
                _ => {}
            }
        }
        let mut value = self.evaluate(local.clamp(start, end));
        if mode == 3 {
            let delta = b.value - a.value;
            if before {
                value -= cycles * delta;
            } else {
                value += delta * cycles;
            }
        }
        value
    }
}
