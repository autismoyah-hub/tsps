//! Original nb inventory definition: capacity and typed parameters.
use crate::{buffer::Reader, disk_cache::DiskCache, object::Param, Result};
use std::collections::HashMap;
#[derive(Default, Debug)]
pub struct Definition {
    pub capacity: i32,
    pub params: HashMap<u32, Param>,
}
impl Definition {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let mut d = Self::default();
        loop {
            match r.u8()? {
                0 => return Ok(d),
                2 => d.capacity = r.u16()? as i32,
                249 => {
                    for _ in 0..r.u8()? {
                        let kind = r.u8()?;
                        let id = r.u24()?;
                        let value = match kind {
                            1 => Param::String(r.string()?),
                            2 => Param::Long(r.u64()? as i64),
                            _ => Param::Int(r.u32()? as i32),
                        };
                        // bn.ag appends hash nodes; lookup returns the first matching entry.
                        d.params.entry(id).or_insert(value);
                    }
                }
                _ => {}
            }
        }
    }
    pub fn load(cache: &mut DiskCache, id: u32) -> Result<Self> {
        let exists = cache
            .reference(2)?
            .groups
            .iter()
            .find(|g| g.id == 5)
            .is_some_and(|g| g.files.iter().any(|f| f.id == id));
        if exists {
            Self::decode(&cache.file(2, 5, id, [0; 4])?)
        } else {
            Ok(Self::default())
        }
    }
    /// Opcode6528 tests only string versus integer, even for long parameter metadata.
    pub fn query(
        &self,
        id: u32,
        metadata: &crate::param_def::Definition,
    ) -> Result<crate::vm::Value> {
        let default = if matches!(metadata.default, crate::vm::Value::Object(_)) {
            metadata.default.clone()
        } else {
            crate::vm::Value::Int(metadata.default_int)
        };
        crate::script_params::query(&self.params, id, &default)
    }
}
