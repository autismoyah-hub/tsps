//! Typed clan profile variables, cache group 47 (te/tq, ti and wg).
use crate::{
    buffer::Reader,
    disk_cache::DiskCache,
    vm::{Object, Value},
    Error, Result,
};
use std::collections::HashMap;
#[derive(Clone, Copy, Debug, Default)]
pub struct Definition {
    pub kind: Option<u8>,
}
fn circumfixed(r: &mut Reader<'_>) -> Result<String> {
    if r.u8()? != 0 {
        return Err(Error::Invalid("profile string prefix"));
    }
    r.string()
}
impl Definition {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let mut result = Self::default();
        loop {
            match r.u8()? {
                0 => break,
                1 => result.kind = Some(crate::database_defs::base_type(r.u8()? as u16)?),
                2 => {
                    r.u8()?;
                }
                3 => return Err(Error::Invalid("unrecognized profile encoding key")),
                10 => {
                    circumfixed(&mut r)?;
                }
                _ => {} // te's extension decoder is intentionally empty.
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("profile definition trailing bytes"));
        }
        Ok(result)
    }
    pub fn default_value(self) -> Option<Value> {
        match self.kind {
            Some(0) => Some(Value::Int(0)),
            Some(1) => Some(Value::Long(-1)),
            Some(2) => Some(Value::Object(Object::String(String::new()))),
            _ => None,
        }
    }
    pub fn decode_value(self, bytes: &[u8]) -> Result<Value> {
        let mut r = Reader::new(bytes);
        let value = match self.kind {
            Some(0) => Value::Int(r.u32()? as i32),
            Some(1) => Value::Long(r.u64()? as i64),
            Some(2) => Value::Object(Object::String(circumfixed(&mut r)?)),
            _ => return Err(Error::Invalid("missing profile variable type")),
        };
        if r.remaining() != 0 {
            return Err(Error::Invalid("profile value trailing bytes"));
        }
        Ok(value)
    }
}
#[derive(Default)]
pub struct Profile {
    pub present: bool,
    pub values: HashMap<i32, Value>,
    definitions: Option<HashMap<u32, Vec<u8>>>,
}
impl Profile {
    pub fn reset(&mut self, present: bool) {
        self.present = present;
        self.values.clear();
    }
    fn definition(&mut self, id: i32, cache: &mut DiskCache) -> Result<Definition> {
        if self.definitions.is_none() {
            let mut definitions = HashMap::new();
            if cache.reference(2)?.groups.iter().any(|g| g.id == 47) {
                for (id, raw) in cache.group(2, 47, [0; 4])? {
                    definitions.insert(id, raw);
                }
            }
            self.definitions = Some(definitions);
        }
        match self.definitions.as_ref().unwrap().get(&(id as u32)) {
            Some(raw) => Definition::decode(raw),
            None => Ok(Definition::default()),
        }
    }
    pub fn update(&mut self, payload: &[u8], cache: &mut DiskCache) -> Result<i32> {
        let mut r = Reader::new(payload);
        let id = r.u16()? as i32;
        let definition = self.definition(id, cache)?;
        let value = definition.decode_value(r.bytes(r.remaining())?)?;
        self.present = true;
        self.values.insert(id, value);
        Ok(id)
    }
    pub fn read(&mut self, id: i32, cache: &mut DiskCache) -> Result<i32> {
        if !self.present {
            return Err(Error::Invalid("clan profile absent"));
        }
        let value = if let Some(value) = self.values.get(&id) {
            Some(value.clone())
        } else {
            self.definition(id, cache)?.default_value()
        };
        match value {
            Some(Value::Int(value)) => Ok(value),
            _ => Err(Error::Invalid("clan profile variable is not an integer")),
        }
    }
}
