//! dl.ao/aq persistent client variables. Reads retain valid entries before a corrupt tail.
use crate::{
    buffer::Reader,
    vm::{Array, Object, Value},
    writer::Writer,
    Error, Result,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub fn read(bytes: &[u8], persistent: &[bool], values: &mut HashMap<i32, Value>) -> Result<()> {
    if bytes.is_empty() {
        return Ok(());
    }
    let mut r = Reader::new(bytes);
    let version = r.u8()?;
    if version > 2 {
        return Ok(());
    }
    for _ in 0..r.u16()? {
        let id = r.u16()? as usize;
        let value = if version < 2 {
            Value::Int(r.u32()? as i32)
        } else {
            match r.u8()? {
                0 => Value::Int(r.u32()? as i32),
                1 => Value::Long(r.u64()? as i64),
                2 => Value::Object(Object::String(r.string()?)),
                5 => {
                    let len = r.u16()? as usize;
                    if len > 5000 {
                        return Err(Error::Invalid("saved varc array size"));
                    }
                    let array = match r.u8()? {
                        0 => Array::Ints(
                            (0..len)
                                .map(|_| r.u32().map(|v| v as i32))
                                .collect::<Result<_>>()?,
                        ),
                        1 => Array::Longs(
                            (0..len)
                                .map(|_| r.u64().map(|v| v as i64))
                                .collect::<Result<_>>()?,
                        ),
                        2 => Array::Strings(
                            (0..len)
                                .map(|_| r.string().map(Some))
                                .collect::<Result<_>>()?,
                        ),
                        _ => return Err(Error::Invalid("saved varc array type")),
                    };
                    Value::Object(Object::Array(Rc::new(RefCell::new(Array::managed(
                        array,
                        crate::script_array_storage::Element::Null,
                        len,
                        false,
                    )))))
                }
                _ => return Err(Error::Invalid("saved varc type")),
            }
        };
        if persistent.get(id).copied().unwrap_or(false) {
            values.insert(id as i32, value);
        }
    }
    if version < 2 {
        for _ in 0..r.u16()? {
            r.u16()?;
            r.string()?;
        }
    }
    Ok(())
}
pub fn encode(persistent: &[bool], values: &HashMap<i32, Value>) -> Result<Vec<u8>> {
    let mut entries = Vec::new();
    for (&id, value) in values {
        let keep = persistent
            .get(id as usize)
            .ok_or(Error::Invalid("saved varc id"))?;
        if *keep {
            entries.push((id, value));
        }
    }
    entries.sort_unstable_by_key(|entry| entry.0);
    let mut w = Writer::new();
    w.u8(2);
    w.u16(u16::try_from(entries.len()).map_err(|_| Error::Invalid("saved varc count"))?);
    for (id, value) in entries {
        w.u16(id as u16);
        match value {
            Value::Int(value) => {
                w.u8(0);
                w.u32(*value as u32);
            }
            Value::Long(value) => {
                w.u8(1);
                w.u64(*value as u64);
            }
            Value::Object(Object::String(value)) => {
                w.u8(2);
                w.string(value)?;
            }
            // Original allocation excludes arrays and nulls; do not silently lose them.
            _ => return Err(Error::Unsupported("saved varc object serialization")),
        }
    }
    Ok(w.into_vec())
}
