//! Arithmetic script opcodes from cx.bj, including Java overflow and shifts.
use crate::{
    vm::{Object, Stacks},
    Error, Result,
};
fn pair(s: &mut Stacks) -> Result<(i32, i32)> {
    let b = s.pop_int()?;
    Ok((s.pop_int()?, b))
}
fn long_pair(s: &mut Stacks) -> Result<(i64, i64)> {
    let b = s.pop_long()?;
    Ok((s.pop_long()?, b))
}
fn div(a: i32, b: i32) -> Result<i32> {
    if b == 0 {
        Err(Error::Invalid("integer division by zero"))
    } else {
        Ok(a.wrapping_div(b))
    }
}
fn long_div(a: i64, b: i64) -> Result<i64> {
    if b == 0 {
        Err(Error::Invalid("long division by zero"))
    } else {
        Ok(a.wrapping_div(b))
    }
}
fn mask(bits: i32) -> Result<i64> {
    match bits {
        0..=63 => Ok((1u64 << bits).wrapping_sub(1) as i64),
        64 => Ok(-1),
        _ => Err(Error::Invalid("bit mask width")),
    }
}
/// Returns false for an opcode outside this handler. Random is supplied by the
/// client host so seeded replay and normal game randomness share this code.
pub fn execute(op: u16, s: &mut Stacks, mut random: impl FnMut() -> f64) -> Result<bool> {
    match op {
        4000..=4003 | 4007..=4017 | 4026 | 4034 => {
            let (a, b) = pair(s)?;
            let value = match op {
                4000 => a.wrapping_add(b),
                4001 => a.wrapping_sub(b),
                4002 => a.wrapping_mul(b),
                4003 => div(a, b)?,
                4007 => a.wrapping_add(div(a.wrapping_mul(b), 100)?),
                4008 => {
                    if (0..=31).contains(&b) {
                        a | 1i32.wrapping_shl(b as u32)
                    } else {
                        a
                    }
                }
                4009 => {
                    if (0..=31).contains(&b) {
                        a & !1i32.wrapping_shl(b as u32)
                    } else {
                        a
                    }
                }
                4010 => ((0..=31).contains(&b) && (a & 1i32.wrapping_shl(b as u32)) != 0) as i32,
                4011 => {
                    if b == 0 {
                        return Err(Error::Invalid("integer remainder by zero"));
                    }
                    a.wrapping_rem(b)
                }
                4012 => {
                    if a == 0 {
                        0
                    } else {
                        (a as f64).powf(b as f64) as i32
                    }
                }
                4013 => {
                    if a == 0 {
                        0
                    } else {
                        match b {
                            0 => i32::MAX,
                            1 => a,
                            2 => (a as f64).sqrt() as i32,
                            3 => (a as f64).cbrt() as i32,
                            4 => (a as f64).sqrt().sqrt() as i32,
                            _ => (a as f64).powf(1.0 / b as f64) as i32,
                        }
                    }
                }
                4014 => a & b,
                4015 => a | b,
                4016 => a.min(b),
                4017 => a.max(b),
                4026 => a ^ 1i32.wrapping_shl(b as u32),
                4034 => {
                    (((a as f64).atan2(b as f64) * 2607.5945876176133 + 0.5).floor() as i64 as i32)
                        & 16383
                }
                _ => unreachable!(),
            };
            s.push_int(value)?;
        }
        4004 | 4005 => {
            let n = s.pop_int()?;
            let value = random();
            if !(0.0..1.0).contains(&value) {
                return Err(Error::Invalid("script random range"));
            }
            s.push_int((value * if op == 4005 { n.wrapping_add(1) } else { n } as f64) as i32)?;
        }
        4006 => {
            let x = s.pop_int()?;
            let x1 = s.pop_int()?;
            let x0 = s.pop_int()?;
            let y1 = s.pop_int()?;
            let y0 = s.pop_int()?;
            s.push_int(y0.wrapping_add(div(
                y1.wrapping_sub(y0).wrapping_mul(x.wrapping_sub(x0)),
                x1.wrapping_sub(x0),
            )?))?;
        }
        4018 => {
            let c = s.pop_int()? as i64;
            let b = s.pop_int()? as i64;
            let a = s.pop_int()? as i64;
            s.push_int(long_div(a * c, b)? as i32)?;
        }
        4025 => {
            let a = s.pop_int()?;
            s.push_int(a.count_ones() as i32)?;
        }
        4027..=4030 => {
            let end = s.pop_int()?;
            let start = s.pop_int()?;
            let replacement = if op == 4030 { Some(s.pop_int()?) } else { None };
            let a = s.pop_int()?;
            let value = if !(0..=31).contains(&start) || !(0..=31).contains(&end) {
                a
            } else if op == 4029 {
                (a.wrapping_shl((31 - end) as u32) as u32).wrapping_shr((31 - end + start) as u32)
                    as i32
            } else {
                let bits = mask(end - start + 1)? as i32;
                let field = bits.wrapping_shl(start as u32);
                match op {
                    4027 => a | field,
                    4028 => a & !field,
                    _ => (a & !field) | replacement.unwrap().min(bits).wrapping_shl(start as u32),
                }
            };
            s.push_int(value)?;
        }
        4032 | 4033 => {
            let angle = (s.pop_int()? & 16383) as f64 * 3.834951969714103E-4;
            s.push_int((16384.0 * if op == 4032 { angle.sin() } else { angle.cos() }) as i32)?;
        }
        4035 => {
            let a = s.pop_int()?;
            s.push_int(a.wrapping_abs())?;
        }
        4036 => {
            let Object::String(value) = s.pop_object()? else {
                return Err(Error::Invalid("parse integer string"));
            };
            let valid = !value.is_empty()
                && value
                    .bytes()
                    .enumerate()
                    .all(|(i, b)| b.is_ascii_digit() || (i == 0 && (b == b'+' || b == b'-')));
            s.push_int(if valid {
                value.parse::<i32>().unwrap_or(-1)
            } else {
                -1
            })?;
        }
        4037..=4042 | 4055..=4057 => {
            let (a, b) = long_pair(s)?;
            let value = match op {
                4037 => a
                    .checked_add(b)
                    .ok_or(Error::Invalid("long addition overflow"))?,
                4038 => a
                    .checked_sub(b)
                    .ok_or(Error::Invalid("long subtraction overflow"))?,
                4039 => {
                    let value = a.wrapping_mul(b);
                    if b != 0 && a != value.wrapping_div(b) {
                        return Err(Error::Invalid("long multiplication overflow"));
                    }
                    value
                }
                4040 => long_div(a, b)?,
                4041 => a.min(b),
                4042 => a.max(b),
                4055 => {
                    if b == 0 {
                        return Err(Error::Invalid("long remainder by zero"));
                    }
                    a.wrapping_rem(b)
                }
                4056 => a & b,
                4057 => a | b,
                _ => unreachable!(),
            };
            s.push_long(value)?;
        }
        4043 => {
            let c = s.pop_long()? as i128;
            let b = s.pop_long()? as i128;
            let a = s.pop_long()? as i128;
            if b == 0 {
                return Err(Error::Invalid("long scale by zero"));
            }
            s.push_long(
                i64::try_from(a * c / b).map_err(|_| Error::Invalid("long scale overflow"))?,
            )?;
        }
        4044 => {
            let a = s.pop_int()?;
            s.push_long(a as i64)?;
        }
        4047..=4049 | 4051 => {
            let shift = s.pop_int()? as u32;
            let a = s.pop_long()?;
            let bit = 1i64.wrapping_shl(shift);
            if op == 4049 {
                s.push_int((a & bit != 0) as i32)?;
            } else {
                s.push_long(match op {
                    4047 => a | bit,
                    4048 => a & !bit,
                    _ => a ^ bit,
                })?;
            }
        }
        4050 => {
            let a = s.pop_long()?;
            s.push_int(a.count_ones() as i32)?;
        }
        4052..=4054 => {
            let end = s.pop_int()?.clamp(0, 63);
            let start = s.pop_int()?.clamp(0, 63);
            let a = s.pop_long()?;
            if end < start {
                return Err(Error::Invalid("long bit range"));
            }
            let bits = mask(end - start + 1)?.wrapping_shl(start as u32);
            s.push_long(match op {
                4052 => a | bits,
                4053 => a & !bits,
                _ => (a.wrapping_shl((63 - end) as u32) as u64)
                    .wrapping_shr((63 - end + start) as u32) as i64,
            })?;
        }
        4058 => {
            let a = s.pop_long()?;
            s.push_long(!a)?;
        }
        4059 => {
            let end = s.pop_int()?;
            let start = s.pop_int()?;
            let (a, b) = long_pair(s)?;
            let bits = mask(end.wrapping_sub(start).wrapping_add(1))?;
            s.push_long(
                (a & !bits.wrapping_shl(start as u32)) | b.min(bits).wrapping_shl(start as u32),
            )?;
        }
        4060 => {
            let (a, b) = pair(s)?;
            s.push_long(((a as i64) << 32) | (b as u32 as i64))?;
        }
        4061 => {
            let a = s.pop_long()?;
            s.push_int((a >> 32) as i32)?;
            s.push_int(a as i32)?;
        }
        _ => return Ok(false),
    };
    Ok(true)
}
