//! Database row/table definitions (GameDBRowConfig / yi / sb.aa).
use crate::{
    buffer::Reader,
    vm::{Object, Value},
    Error, Result,
};
#[derive(Clone, Debug, Default)]
pub struct Column {
    pub types: Vec<u16>,
    pub values: Option<Vec<Value>>,
}
#[derive(Clone, Debug)]
pub struct Row {
    pub table: i32,
    pub columns: Vec<Option<Column>>,
}
#[derive(Clone, Debug, Default)]
pub struct Table {
    pub columns: Vec<Option<Column>>,
}
pub fn base_type(kind: u16) -> Result<u8> {
    Ok(match kind {
        36 => 2,
        49 | 56 | 71 | 110 | 115 | 116 | 213 => 1,
        0..=122 if kind != 50 => 0,
        209 | 212 | 214 => 0,
        _ => return Err(Error::Invalid("database value type")),
    })
}
fn types(r: &mut Reader) -> Result<Vec<u16>> {
    let count = r.u8()?;
    let mut values = Vec::new();
    for _ in 0..count {
        values.push(r.unsigned_smart()?);
    }
    Ok(values)
}
fn values(r: &mut Reader, kinds: &[u16]) -> Result<Vec<Value>> {
    let count = (r.unsigned_smart()? as usize)
        .checked_mul(kinds.len())
        .ok_or(Error::Invalid("database tuple size"))?;
    if count > r.remaining() {
        return Err(Error::Truncated);
    }
    let mut values = Vec::with_capacity(count);
    for _ in 0..count / kinds.len().max(1) {
        for &kind in kinds {
            values.push(match base_type(kind)? {
                0 => Value::Int(r.u32()? as i32),
                1 => Value::Long(r.u64()? as i64),
                _ => Value::Object(Object::String(r.string()?)),
            });
        }
    }
    Ok(values)
}
impl Row {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let mut row = Self {
            table: -1,
            columns: Vec::new(),
        };
        loop {
            match r.u8()? {
                0 => break,
                3 => {
                    row.columns = vec![None; r.u8()? as usize];
                    loop {
                        let index = r.u8()?;
                        if index == 255 {
                            break;
                        }
                        let types = types(&mut r)?;
                        let values = Some(values(&mut r, &types)?);
                        *row.columns
                            .get_mut(index as usize)
                            .ok_or(Error::Invalid("database row column"))? =
                            Some(Column { types, values });
                    }
                }
                4 => row.table = r.varint_le()? as i32,
                _ => return Err(Error::Unsupported("database row opcode")),
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("database row trailing bytes"));
        }
        Ok(row)
    }
}
impl Table {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let mut table = Self::default();
        loop {
            match r.u8()? {
                0 => break,
                1 => {
                    table.columns = vec![None; r.u8()? as usize];
                    loop {
                        let tag = r.u8()?;
                        if tag == 255 {
                            break;
                        }
                        let types = types(&mut r)?;
                        let values = if tag & 128 != 0 {
                            Some(values(&mut r, &types)?)
                        } else {
                            None
                        };
                        *table
                            .columns
                            .get_mut((tag & 127) as usize)
                            .ok_or(Error::Invalid("database table column"))? =
                            Some(Column { types, values });
                    }
                }
                _ => return Err(Error::Unsupported("database table opcode")),
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("database table trailing bytes"));
        }
        Ok(table)
    }
}
