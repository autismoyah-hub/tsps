//! Cache database indexes and query cursor, recovered from ye and GameScript.cc.
use crate::{
    buffer::Reader, disk_cache::DiskCache, interface_script::string, vm::Stacks, Error, Result,
};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Int(i32),
    Long(i64),
    String(String),
}
#[derive(Default)]
pub struct Index {
    columns: Vec<HashMap<Key, Vec<i32>>>,
}
impl Index {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let count = r.varint_le()? as usize;
        if count > r.remaining() {
            return Err(Error::Truncated);
        }
        let mut columns = Vec::with_capacity(count);
        for _ in 0..count {
            let kind = r.u8()?;
            let entries = r.varint_le()? as usize;
            if entries > r.remaining() {
                return Err(Error::Truncated);
            }
            let mut map = HashMap::new();
            for _ in 0..entries {
                let key = match kind {
                    0 => Key::Int(r.u32()? as i32),
                    1 => Key::Long(r.u64()? as i64),
                    2 => Key::String(r.string()?),
                    _ => return Err(Error::Unsupported("database index key type")),
                };
                let count = r.varint_le()? as usize;
                if count > r.remaining() {
                    return Err(Error::Truncated);
                }
                let mut rows = Vec::with_capacity(count);
                for _ in 0..count {
                    rows.push(r.varint_le()? as i32);
                }
                map.insert(key, rows);
            }
            columns.push(map);
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("database index trailing bytes"));
        }
        Ok(Self { columns })
    }
    pub fn query(&self, key: &Key, column: i32) -> Result<Option<&[i32]>> {
        Ok(self
            .columns
            .get(column.max(0) as usize)
            .ok_or(Error::Invalid("database index column"))?
            .get(key)
            .map(Vec::as_slice))
    }
}
#[derive(Default)]
pub struct Database {
    row_files: Option<HashMap<u32, Vec<u8>>>,
    table_files: Option<HashMap<u32, Vec<u8>>>,
    definitions: HashMap<u32, crate::database_defs::Row>,
    tables: HashMap<u32, crate::database_defs::Table>,
    indexes: HashMap<(u32, u32), Index>,
    rows: Option<Vec<i32>>,
    cursor: usize,
    iterator_rows: Option<Vec<i32>>,
    table: Option<u32>,
}
impl Database {
    fn index(&mut self, cache: &mut DiskCache, table: u32, file: u32) -> Result<&Index> {
        if !self.indexes.contains_key(&(table, file)) {
            let index = Index::decode(&cache.file(21, table, file, [0; 4])?)?;
            self.indexes.insert((table, file), index);
        }
        Ok(&self.indexes[&(table, file)])
    }
    fn row(&mut self, cache: &mut DiskCache, id: u32) -> Result<&crate::database_defs::Row> {
        if !self.definitions.contains_key(&id) {
            let bytes = definition_file(cache, &mut self.row_files, 38, id)?;
            self.definitions
                .insert(id, crate::database_defs::Row::decode(&bytes)?);
        }
        Ok(&self.definitions[&id])
    }
    fn table_definition(
        &mut self,
        cache: &mut DiskCache,
        id: u32,
    ) -> Result<&crate::database_defs::Table> {
        if !self.tables.contains_key(&id) {
            let bytes = definition_file(cache, &mut self.table_files, 39, id)?;
            self.tables
                .insert(id, crate::database_defs::Table::decode(&bytes)?);
        }
        Ok(&self.tables[&id])
    }
    pub fn execute(&mut self, op: u16, s: &mut Stacks, cache: &mut DiskCache) -> Result<bool> {
        match op {
            7505 => {
                let id = s.pop_int()? as u32;
                s.push_int(self.row(cache, id)?.table)?;
            }
            7502 | 7503 => {
                let tuple = if op == 7502 { s.pop_int()? } else { 0 };
                let descriptor = s.pop_int()? as u32;
                let row_id = s.pop_int()?;
                let column = ((descriptor >> 4) & 255) as usize;
                let field = (descriptor & 15) as i32 - 1;
                let definition = self
                    .table_definition(cache, descriptor >> 12)?
                    .columns
                    .get(column)
                    .and_then(Option::as_ref)
                    .ok_or(Error::Invalid("undefined database column"))?
                    .clone();
                let types = &definition.types;
                if op == 7502 && row_id == -1 {
                    // The reference ignores the tuple selector for the missing-row sentinel.
                    for &kind in types {
                        crate::param_def::push(
                            s,
                            match crate::database_defs::base_type(kind)? {
                                2 => crate::vm::Value::Object(crate::vm::Object::String(
                                    String::new(),
                                )),
                                1 => crate::vm::Value::Long(-1),
                                _ => crate::vm::Value::Int(if kind <= 1 { 0 } else { -1 }),
                            },
                        )?;
                    }
                    return Ok(true);
                }
                let values = self
                    .row(cache, row_id as u32)?
                    .columns
                    .get(column)
                    .and_then(Option::as_ref)
                    .and_then(|c| c.values.as_ref())
                    .cloned()
                    .or(definition.values);
                if op == 7503 {
                    let count = match values {
                        Some(v) => v
                            .len()
                            .checked_div(types.len())
                            .ok_or(Error::Invalid("zero database tuple width"))?,
                        None => 0,
                    };
                    s.push_int(count as i32)?;
                } else {
                    let range = if field < 0 {
                        0..types.len()
                    } else {
                        if field as usize >= types.len() {
                            return Err(Error::Invalid("database tuple field"));
                        }
                        field as usize..field as usize + 1
                    };
                    if let Some(values) = values {
                        if types.is_empty()
                            || tuple < 0
                            || tuple as usize >= values.len() / types.len()
                        {
                            return Err(Error::Invalid("database tuple index"));
                        }
                        for field in range {
                            let value = values[tuple as usize * types.len() + field].clone();
                            // This revision's non-string branch casts to Integer, including long fields.
                            if matches!(value, crate::vm::Value::Long(_)) {
                                return Err(Error::Invalid("database field requires integer"));
                            }
                            crate::param_def::push(s, value)?;
                        }
                    } else {
                        for field in range {
                            let kind = types[field];
                            if crate::database_defs::base_type(kind)? == 2 {
                                s.push_object(crate::vm::Object::String(String::new()))?;
                            } else {
                                s.push_int(if kind <= 1 { 0 } else { -1 })?;
                            }
                        }
                    }
                }
            }

            7500 | 7507 => {
                let kind = s.pop_int()?;
                let key = match kind {
                    0 => Some(Key::Int(s.pop_int()?)),
                    1 => Some(Key::Long(s.pop_long()?)),
                    2 => Some(Key::String(string(s)?)),
                    -1 => None,
                    _ => return Err(Error::Unsupported("database query stack type")),
                };
                let descriptor = s.pop_int()? as u32;
                let table = descriptor >> 12;
                let file = ((descriptor >> 4) & 255) + 1;
                let column = (descriptor & 15) as i32 - 1;
                let index = self.index(cache, table, file)?;
                let rows = match key {
                    Some(key) => index.query(&key, column)?.map(<[i32]>::to_vec),
                    None => None,
                };
                if op == 7507 {
                    if self.table != Some(table) {
                        return Err(Error::Invalid("database filter table mismatch"));
                    }
                    let current = self
                        .rows
                        .as_mut()
                        .ok_or(Error::Invalid("database filter without query"))?;
                    let matching: HashSet<i32> = rows.unwrap_or_default().into_iter().collect();
                    current.retain(|v| matching.contains(v));
                } else {
                    self.table = rows.as_ref().map(|_| table);
                    self.rows = rows;
                }
                self.cursor = 0;
                self.iterator_rows = self.rows.clone();
                s.push_int(self.rows.as_ref().map_or(0, |v| v.len() as i32))?;
            }
            7501 => {
                let value = self
                    .iterator_rows
                    .as_ref()
                    .and_then(|v| v.get(self.cursor))
                    .copied();
                if value.is_some() {
                    self.cursor += 1;
                }
                s.push_int(value.unwrap_or(-1))?;
            }
            7504 => {
                let table = s.pop_int()? as u32;
                let rows = self
                    .index(cache, table, 0)?
                    .query(&Key::Int(0), 0)?
                    .map(<[i32]>::to_vec);
                if rows.is_some() {
                    self.table = Some(table);
                    self.cursor = 0;
                    self.iterator_rows = rows.clone();
                }
                self.rows = rows;
                s.push_int(self.rows.as_ref().map_or(0, |v| v.len() as i32))?;
            }
            7506 => {
                let index = s.pop_int()?;
                s.push_int(
                    self.rows
                        .as_ref()
                        .and_then(|v| v.get(index as usize))
                        .copied()
                        .unwrap_or(-1),
                )?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

fn definition_file(
    cache: &mut DiskCache,
    files: &mut Option<HashMap<u32, Vec<u8>>>,
    group: u32,
    id: u32,
) -> Result<Vec<u8>> {
    if files.is_none() {
        let exists = cache.reference(2)?.groups.iter().any(|g| g.id == group);
        *files = Some(if exists {
            cache.group(2, group, [0; 4])?.into_iter().collect()
        } else {
            HashMap::new()
        });
    }
    Ok(files
        .as_ref()
        .unwrap()
        .get(&id)
        .cloned()
        .unwrap_or_else(|| vec![0]))
}

#[cfg(test)]
#[path = "database_tests.rs"]
mod tests;
