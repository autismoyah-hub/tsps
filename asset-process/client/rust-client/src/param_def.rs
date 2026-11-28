//! Parameter metadata and typed defaults (pm constructor / yt base-type map).
use crate::{
    buffer::Reader,
    vm::{Object, Stacks, Value},
    Error, Result,
};
#[derive(Clone, Debug)]
pub struct Definition {
    pub kind: u8,
    pub default_int: i32,
    pub default: Value,
    pub auto_disable: bool,
}
impl Definition {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let (mut kind, mut int, mut long, mut text, mut auto) = (0, 0, 0, None, true);
        loop {
            match r.u8()? {
                0 => break,
                1 => kind = if r.u8()? == b's' { 36 } else { 0 },
                2 => int = r.u32()? as i32,
                4 => auto = false,
                5 => text = Some(r.string()?),
                7 => long = r.u64()? as i64,
                8 => kind = r.u8()?,
                _ => {}
            }
        }
        let default = match kind {
            36 => Value::Object(text.map_or(Object::Null, Object::String)),
            49 | 56 | 71 | 110 | 115 | 116 | 213 => Value::Long(long),
            0..=122 if kind != 50 => Value::Int(int),
            209 | 212 | 214 => Value::Int(int),
            _ => return Err(Error::Invalid("parameter base type")),
        };
        Ok(Self {
            kind,
            default_int: int,
            default,
            auto_disable: auto,
        })
    }
}
pub fn pop(s: &mut Stacks, kind: i32) -> Result<Option<Value>> {
    Ok(match kind {
        -1 => None,
        0 => Some(Value::Int(s.pop_int()?)),
        1 => Some(Value::Long(s.pop_long()?)),
        2 => Some(Value::Object(s.pop_object()?)),
        _ => return Err(Error::Invalid("parameter stack type")),
    })
}
pub fn equal(a: &Value, b: &Option<Value>) -> bool {
    match (a, b) {
        (Value::Int(a), Some(Value::Int(b))) => a == b,
        (Value::Long(a), Some(Value::Long(b))) => a == b,
        (Value::Object(Object::String(a)), Some(Value::Object(Object::String(b)))) => a == b,
        (Value::Object(Object::Null), Some(Value::Object(Object::Null))) => true,
        (Value::Object(Object::Array(a)), Some(Value::Object(Object::Array(b)))) => {
            std::rc::Rc::ptr_eq(a, b)
        }
        _ => false,
    }
}
pub fn push(s: &mut Stacks, v: Value) -> Result<()> {
    match v {
        Value::Int(v) => s.push_int(v),
        Value::Long(v) => s.push_long(v),
        Value::Object(v) => s.push_object(v),
    }
}
