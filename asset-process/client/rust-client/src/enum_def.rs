//! Revision 239 enum definitions, including long values.
use crate::{
    buffer::Reader,
    script_array_storage::Element,
    vm::{Array, Object, Value},
    Error, Result,
};
use std::{cell::RefCell, rc::Rc};
pub struct Definition {
    pub input: u8,
    pub output: u8,
    pub default_int: i32,
    pub default_long: i64,
    pub default_string: String,
    pub entries: Vec<(i32, Value)>,
    keys: Option<Vec<i32>>,
    ints: Option<Vec<i32>>,
    longs: Option<Vec<i64>>,
    strings: Option<Vec<Option<String>>>,
    arrays: RefCell<[Option<Rc<RefCell<Array>>>; 2]>,
}
impl Definition {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut d = Self {
            input: 0,
            output: 0,
            default_int: 0,
            default_long: 0,
            default_string: "null".into(),
            entries: vec![],
            keys: None,
            ints: None,
            longs: None,
            strings: None,
            arrays: RefCell::new([None, None]),
        };
        let mut r = Reader::new(bytes);
        loop {
            match r.u8()? {
                0 => break,
                1 => d.input = r.u8()?,
                2 => d.output = r.u8()?,
                3 => d.default_string = r.string()?,
                4 => d.default_int = r.u32()? as i32,
                op @ 5..=7 => {
                    d.entries.clear();
                    for _ in 0..r.u16()? {
                        let key = r.u32()? as i32;
                        let value = match op {
                            5 => Value::Object(Object::String(r.string()?)),
                            6 => Value::Int(r.u32()? as i32),
                            _ => Value::Long(r.u64()? as i64),
                        };
                        d.entries.push((key, value));
                    }
                    d.keys = Some(d.entries.iter().map(|(k, _)| *k).collect());
                    match op {
                        5 => {
                            d.strings = Some(
                                d.entries
                                    .iter()
                                    .map(|(_, v)| match v {
                                        Value::Object(Object::String(v)) => Some(v.clone()),
                                        _ => unreachable!(),
                                    })
                                    .collect(),
                            )
                        }
                        6 => {
                            d.ints = Some(
                                d.entries
                                    .iter()
                                    .map(|(_, v)| match v {
                                        Value::Int(v) => *v,
                                        _ => unreachable!(),
                                    })
                                    .collect(),
                            )
                        }
                        _ => {
                            d.longs = Some(
                                d.entries
                                    .iter()
                                    .map(|(_, v)| match v {
                                        Value::Long(v) => *v,
                                        _ => unreachable!(),
                                    })
                                    .collect(),
                            )
                        }
                    }
                }
                8 => d.default_long = r.u64()? as i64,
                _ => {}
            }
        }
        Ok(d)
    }
    pub fn array(&self, values: bool, kind: i32) -> Result<Rc<RefCell<Array>>> {
        let expected = if values { self.output } else { self.input };
        if kind as u16 != expected as u16 {
            return Err(Error::Invalid("enum array type"));
        }
        let index = values as usize;
        let mut cached = self.arrays.borrow_mut();
        if let Some(a) = &cached[index] {
            return Ok(a.clone());
        }
        let missing = Error::Invalid("missing enum array");
        let data = if !values {
            Array::Ints(self.keys.clone().ok_or(missing)?)
        } else {
            match self.output {
                b's' => Array::Strings(self.strings.clone().ok_or(missing)?),
                207 => Array::Longs(self.longs.clone().ok_or(missing)?),
                _ => Array::Ints(self.ints.clone().ok_or(missing)?),
            }
        };
        let length = data.len();
        let array = Rc::new(RefCell::new(Array::managed(
            data,
            Element::Null,
            length,
            false,
        )));
        cached[index] = Some(array.clone());
        Ok(array)
    }
    pub fn get(&self, key: i32) -> Value {
        self.entries
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| match self.output {
                b's' => Value::Object(Object::String(self.default_string.clone())),
                207 => Value::Long(self.default_long),
                _ => Value::Int(self.default_int),
            })
    }
}
