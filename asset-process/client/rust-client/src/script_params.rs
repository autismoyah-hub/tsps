//! Typed NPC/object/item/struct/inventory parameter queries (6513..6516, 6528).
use crate::{
    buffer::Reader,
    disk_cache::DiskCache,
    interface_script::ints,
    item::Definitions,
    npc,
    object::{self, Param},
    param_def,
    vm::{Object, Stacks, Value},
    Error, Result,
};
use std::collections::HashMap;
#[derive(Default)]
pub struct Parameters {
    metadata: HashMap<u32, param_def::Definition>,
    inventories: HashMap<u32, crate::inventory_def::Definition>,
    structs: HashMap<u32, HashMap<u32, Param>>,
    objects: HashMap<u32, object::Definition>,
}
pub fn decode_struct(bytes: &[u8]) -> Result<HashMap<u32, Param>> {
    let mut r = Reader::new(bytes);
    let mut params = HashMap::new();
    loop {
        match r.u8()? {
            0 => break,
            249 => {
                for _ in 0..r.u8()? {
                    let kind = r.u8()?;
                    let key = r.u24()?;
                    let value = match kind {
                        1 => Param::String(r.string()?),
                        2 => Param::Long(r.u64()? as i64),
                        _ => Param::Int(r.u32()? as i32),
                    };
                    params.entry(key).or_insert(value);
                }
            }
            _ => {}
        }
    }
    if r.remaining() != 0 {
        return Err(Error::Invalid("struct trailing bytes"));
    }
    Ok(params)
}
pub fn query(params: &HashMap<u32, Param>, id: u32, default: &Value) -> Result<Value> {
    Ok(match params.get(&id) {
        None => default.clone(),
        Some(Param::Int(n)) if matches!(default, Value::Int(_)) => Value::Int(*n),
        Some(Param::Long(n)) if matches!(default, Value::Long(_)) => Value::Long(*n),
        Some(Param::String(s)) if matches!(default, Value::Object(_)) => {
            Value::Object(Object::String(s.clone()))
        }
        _ => return Err(Error::Invalid("parameter value type")),
    })
}
impl Parameters {
    pub fn execute(
        &mut self,
        op: u16,
        s: &mut Stacks,
        cache: &mut DiskCache,
        items: &Definitions,
        npcs: &HashMap<u32, npc::Definition>,
    ) -> Result<bool> {
        if !(6513..=6516).contains(&op) && op != 6528 {
            return Ok(false);
        }
        let [id, param] = ints(s)?;
        let (id, param) = (id as u32, param as u32);
        if !self.metadata.contains_key(&param) {
            let bytes = optional(cache, 11, param)?;
            self.metadata
                .insert(param, param_def::Definition::decode(&bytes)?);
        }
        let default = &self.metadata[&param].default;
        let value = match op {
            6528 => {
                if !self.inventories.contains_key(&id) {
                    self.inventories
                        .insert(id, crate::inventory_def::Definition::load(cache, id)?);
                }
                self.inventories[&id].query(param, &self.metadata[&param])?
            }

            6513 => npcs
                .get(&id)
                .map_or_else(|| Ok(default.clone()), |d| query(&d.params, param, default))?,
            6515 => items
                .items
                .get(&id)
                .map_or_else(|| Ok(default.clone()), |d| query(&d.params, param, default))?,
            6514 => {
                if !self.objects.contains_key(&id) {
                    let bytes = optional(cache, 6, id)?;
                    self.objects
                        .insert(id, object::Definition::decode_local(id, &bytes)?);
                }
                query(&self.objects[&id].params, param, default)?
            }
            _ => {
                if !self.structs.contains_key(&id) {
                    self.structs
                        .insert(id, decode_struct(&optional(cache, 34, id)?)?);
                }
                query(&self.structs[&id], param, default)?
            }
        };
        param_def::push(s, value)?;
        Ok(true)
    }
}
fn optional(cache: &mut DiskCache, group: u32, id: u32) -> Result<Vec<u8>> {
    let exists = cache
        .reference(2)?
        .groups
        .iter()
        .find(|g| g.id == group)
        .is_some_and(|g| g.files.iter().any(|f| f.id == id));
    if exists {
        cache.file(2, group, id, [0; 4])
    } else {
        Ok(vec![0])
    }
}
