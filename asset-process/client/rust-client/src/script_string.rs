//! Deterministic text opcodes; UTF-16 indices are used for Java compatibility.
use crate::{
    interface_script::{ints, string},
    vm::{Object, Stacks},
    Error, Result,
};
fn nullable_text(object: &Object) -> Result<Option<&str>> {
    match object {
        Object::String(text) => Ok(Some(text)),
        Object::Null => Ok(None),
        _ => Err(Error::Invalid("script string argument")),
    }
}
pub fn execute(op: u16, s: &mut Stacks) -> Result<bool> {
    execute_language(op, s, 0)
}
pub fn execute_language(op: u16, s: &mut Stacks, language: u8) -> Result<bool> {
    let text = match op {
        4100 | 4102 => {
            let a = string(s)?;
            let b = s.pop_int()?;
            if op == 4102 && b >= 0 {
                Some(format!("{a}+{b}"))
            } else {
                Some(format!("{a}{b}"))
            }
        }
        4101 => {
            let b = string(s)?;
            let a = string(s)?;
            Some(a + &b)
        }
        4103 | 4111 | 4122 => {
            let object = s.pop_object()?;
            nullable_text(&object)?; // Java casts before reserving the result slot.
            s.push_object(object)?;
            let a = nullable_text(s.objects().last().unwrap())?
                .ok_or(Error::Invalid("null script text"))?;
            let result = match op {
                4103 => a.to_lowercase(),
                4122 => a.to_uppercase(),
                _ => a.chars().map(|c| match c {
                    '<' => "<lt>".into(),
                    '>' => "<gt>".into(),
                    _ => c.to_string(),
                }).collect(),
            };
            s.pop_object()?;
            Some(result)
        }
        4104 => Some(crate::script_date::format_local(s.pop_int()?)?),
        4107 => {
            let b = s.pop_object()?;
            let a = s.pop_object()?;
            s.reserve_int()?;
            let a = nullable_text(&a)?.ok_or(Error::Invalid("null comparison text"))?;
            let b = nullable_text(&b)?.ok_or(Error::Invalid("null comparison text"))?;
            let result = crate::string_compare::compare(
                    &a.encode_utf16().collect::<Vec<_>>(),
                    &b.encode_utf16().collect::<Vec<_>>(),
                    language,
                ).signum();
            s.pop_int()?;
            s.push_int(result)?;
            None
        }
        4106 => Some(s.pop_int()?.to_string()),
        4110 => {
            let b = s.pop_object()?;
            let a = s.pop_object()?;
            nullable_text(&a)?;
            nullable_text(&b)?;
            let selected = if s.pop_int()? == 1 { a } else { b };
            s.push_object(selected)?;
            None
        }
        4112 => {
            let mut a = string(s)?;
            let value = s.pop_int()? as u16;
            let ch = char::from_u32(value as u32)
                .ok_or(Error::Unsupported("unpaired UTF16 script character"))?;
            a.push(ch);
            Some(a)
        }
        4113..=4116 => {
            let ch = s.pop_int()? as u16;
            let yes = match op {
                4113 => {
                    (32..=126).contains(&ch)
                        || (160..=255).contains(&ch)
                        || [8364, 338, 8212, 339, 376].contains(&ch)
                }
                4114 => {
                    (48..=57).contains(&ch) || (65..=90).contains(&ch) || (97..=122).contains(&ch)
                }
                4115 => (65..=90).contains(&ch) || (97..=122).contains(&ch),
                _ => (48..=57).contains(&ch),
            };
            s.push_int(yes as i32)?;
            None
        }
        4117 => {
            let n = match s.pop_object()? {
                Object::Null => 0,
                Object::String(v) => v.encode_utf16().count(),
                _ => return Err(Error::Invalid("string length argument")),
            };
            s.push_int(n as i32)?;
            None
        }
        4118 => {
            let object = s.pop_object()?;
            nullable_text(&object)?;
            let [from, to] = ints(s)?;
            s.push_object(object)?;
            let a = nullable_text(s.objects().last().unwrap())?
                .ok_or(Error::Invalid("null substring text"))?
                .encode_utf16().collect::<Vec<_>>();
            let slice = a.get(from as usize..to as usize)
                .ok_or(Error::Invalid("script substring bounds"))?;
            let result = String::from_utf16(slice)
                .map_err(|_| Error::Unsupported("unpaired UTF16 script substring"))?;
            s.pop_object()?;
            Some(result)
        }
        4119 => {
            let object = s.pop_object()?;
            let a = nullable_text(&object)?.ok_or(Error::Invalid("null tag-removal text"))?;
            let mut tag = false;
            Some(
                a.chars()
                    .filter(|&c| match c {
                        '<' => {
                            tag = true;
                            false
                        }
                        '>' => {
                            tag = false;
                            false
                        }
                        _ => !tag,
                    })
                    .collect(),
            )
        }
        4120 => {
            let object = s.pop_object()?;
            nullable_text(&object)?;
            let point = s.pop_int()?;
            s.reserve_int()?;
            let a = nullable_text(&object)?.ok_or(Error::Invalid("null character-search text"))?;
            // String.indexOf(int) searches BMP values as UTF-16 code units,
            // including surrogate halves within otherwise valid Unicode text.
            let index = if (0..=0xffff).contains(&point) {
                a.encode_utf16()
                    .position(|unit| unit == point as u16)
                    .map_or(-1, |i| i as i32)
            } else {
                char::from_u32(point as u32)
                    .and_then(|c| a.find(c))
                    .map_or(-1, |i| a[..i].encode_utf16().count() as i32)
            };
            s.pop_int()?;
            s.push_int(index)?;
            None
        }
        4121 => {
            // gp.bt removes both object slots before casts, then reserves the
            // integer destination before String.indexOf may fail on null.
            let b = s.pop_object()?;
            let a = s.pop_object()?;
            let nullable = |value| match value {
                Object::String(text) => Ok(Some(text)),
                Object::Null => Ok(None),
                _ => Err(Error::Invalid("substring search argument type")),
            };
            let a = nullable(a)?;
            let b = nullable(b)?;
            let start = s.pop_int()?.max(0) as usize;
            s.reserve_int()?;
            let a = a.ok_or(Error::Invalid("null substring search source"))?
                .encode_utf16().collect::<Vec<_>>();
            let b = b.ok_or(Error::Invalid("null substring search needle"))?
                .encode_utf16().collect::<Vec<_>>();
            let index = if b.is_empty() {
                start.min(a.len()) as i32
            } else {
                (start..=a.len().saturating_sub(b.len()))
                    .find(|&i| a.get(i..i + b.len()) == Some(b.as_slice()))
                    .map_or(-1, |i| i as i32)
            };
            s.pop_int()?;
            s.push_int(index)?;
            None
        }
        4125 => Some(s.pop_long()?.to_string()),
        4126 => {
            // gp.bt consumes the long before the grouping selector. The selector
            // only enables literal commas; it is not the separator itself.
            let value = s.pop_long()?;
            let selector = match s.pop_object()? {
                Object::String(v) => Some(v),
                Object::Null => None,
                _ => return Err(Error::Invalid("long format selector type")),
            };
            Some(if value == 0 {
                "0".into()
            } else if selector
                .ok_or(Error::Invalid("null long format selector"))?
                .is_empty()
            {
                value.to_string()
            } else if value == i64::MIN {
                // Java's negation wraps, leaving the digit loop empty.
                "-".into()
            } else {
                let digits = value.abs().to_string();
                let mut text = String::new();
                if value < 0 {
                    text.push('-');
                }
                for (i, c) in digits.chars().enumerate() {
                    if i > 0 && (digits.len() - i) % 3 == 0 {
                        text.push(',');
                    }
                    text.push(c);
                }
                text
            })
        }
        4127 => {
            let text = match s.pop_object()? {
                Object::String(v) => Some(v),
                Object::Null => None,
                _ => return Err(Error::Invalid("integer parse text type")),
            };
            s.reserve_int()?;
            if let Some(value) = text.as_deref().and_then(parse_java_int) {
                s.pop_int()?;
                s.push_int(value)?;
                s.push_int(1)?;
            } else {
                s.push_int(0)?;
                s.push_int(0)?;
            }
            None
        }
        _ => return Ok(false),
    };
    if let Some(text) = text {
        s.push_object(Object::String(text))?;
    }
    Ok(true)
}

/// gp.bt 4123/4124 select the local view player's appearance selector, not body gender.
pub fn execute_appearance(op: u16, s: &mut Stacks, appearance: Option<i8>) -> Result<bool> {
    match op {
        4123 => {
            let third = s.pop_object()?;
            let second = s.pop_object()?;
            let first = s.pop_object()?;
            if [&first, &second, &third]
                .iter()
                .any(|v| !matches!(v, Object::Null | Object::String(_)))
            {
                return Err(Error::Invalid("appearance text type"));
            }
            s.push_object(match appearance {
                Some(0) => first,
                Some(1) => second,
                _ => third,
            })?;
        }
        4124 => s.push_int(appearance.map_or(-1, i32::from))?,
        _ => return Ok(false),
    }
    Ok(true)
}

/// Integer.parseInt decimal semantics, including BMP Character.digit values.
/// These37 decimal blocks match the pinned Java21 Character table. Supplementary
/// digits arrive as surrogate pairs and are rejected by the original char parser.
pub fn parse_java_int(text: &str) -> Option<i32> {
    const ZEROS: [u16; 37] = [
        0x0030, 0x0660, 0x06f0, 0x07c0, 0x0966, 0x09e6, 0x0a66, 0x0ae6, 0x0b66, 0x0be6, 0x0c66,
        0x0ce6, 0x0d66, 0x0de6, 0x0e50, 0x0ed0, 0x0f20, 0x1040, 0x1090, 0x17e0, 0x1810, 0x1946,
        0x19d0, 0x1a80, 0x1a90, 0x1b50, 0x1bb0, 0x1c40, 0x1c50, 0xa620, 0xa8d0, 0xa900, 0xa9d0,
        0xa9f0, 0xaa50, 0xabf0, 0xff10,
    ];
    let mut chars = text.encode_utf16().peekable();
    let negative = chars.peek() == Some(&(b'-' as u16));
    if negative || chars.peek() == Some(&(b'+' as u16)) {
        chars.next();
    }
    let mut value = 0i32;
    let mut count = 0;
    for c in chars {
        let digit = ZEROS
            .iter()
            .find_map(|&zero| c.checked_sub(zero).filter(|&d| d < 10))?;
        value = value.checked_mul(10)?.checked_sub(i32::from(digit))?;
        count += 1;
    }
    if count == 0 {
        None
    } else if negative {
        Some(value)
    } else {
        value.checked_neg()
    }
}
