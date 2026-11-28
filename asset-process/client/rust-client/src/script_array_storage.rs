//! Mutable script-array metadata from sl, including defaults and growth limits.
use crate::{vm::Array, Error, Result};
#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Null,
    Int(i32),
    Long(i64),
    String(Option<String>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Storage {
    pub values: Array,
    pub default: Element,
    pub capacity: usize,
    pub mutable: bool,
}
impl Array {
    pub fn managed(values: Array, default: Element, capacity: usize, mutable: bool) -> Self {
        Self::Managed(Box::new(Storage {
            values,
            default,
            capacity,
            mutable,
        }))
    }
    pub fn data(&self) -> &Array {
        match self {
            Self::Managed(s) => s.values.data(),
            _ => self,
        }
    }
    pub fn data_mut(&mut self) -> &mut Array {
        match self {
            Self::Managed(s) => s.values.data_mut(),
            _ => self,
        }
    }
    pub fn len(&self) -> usize {
        match self.data() {
            Self::Ints(v) => v.len(),
            Self::Longs(v) => v.len(),
            Self::Strings(v) => v.len(),
            _ => unreachable!(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn writable(&self) -> Result<()> {
        if matches!(self,Self::Managed(s) if !s.mutable) {
            Err(Error::Invalid("immutable script array"))
        } else {
            Ok(())
        }
    }
    pub fn capacity(&self) -> usize {
        match self {
            Self::Managed(s) => s.capacity,
            _ => self.len(),
        }
    }
    pub fn default_value(&self) -> Element {
        match self {
            Self::Managed(s) => s.default.clone(),
            Self::Ints(_) => Element::Int(0),
            Self::Longs(_) => Element::Long(0),
            Self::Strings(_) => Element::String(None),
        }
    }
    pub fn reserve_length(&mut self, size: usize) -> Result<()> {
        self.writable()?;
        if size > 5000 {
            return Err(Error::Invalid("script array size"));
        }
        if let Self::Managed(s) = self {
            if size > s.capacity {
                s.capacity = size.max((s.capacity + s.capacity / 2).min(5000));
            }
        }
        Ok(())
    }
    pub fn resize(&mut self, size: usize) -> Result<()> {
        self.reserve_length(size)?;
        let default = self.default_value();
        match (self.data_mut(), default) {
            (Self::Ints(v), Element::Int(x)) => v.resize(size, x),
            (Self::Longs(v), Element::Long(x)) => v.resize(size, x),
            (Self::Strings(v), Element::String(x)) => v.resize(size, x),
            _ => return Err(Error::Invalid("script array default type")),
        }
        Ok(())
    }
    pub fn create(kind: i32, size: i32, capacity: i32) -> Result<Self> {
        let capacity = capacity.max(size);
        if !(0..=5000).contains(&size) || !(0..=5000).contains(&capacity) {
            return Err(Error::Invalid("script array size"));
        }
        let (values, default) = if kind == 115 {
            (
                Self::Strings(vec![Some(String::new()); size as usize]),
                Element::String(Some(String::new())),
            )
        } else {
            let value = if kind == 105 || kind == 49 { 0 } else { -1 };
            (Self::Ints(vec![value; size as usize]), Element::Int(value))
        };
        Ok(Self::managed(values, default, capacity as usize, true))
    }
}
