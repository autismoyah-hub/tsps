//! The lenient JSON dialect used by the revision's vz content builder.
//! Strings retain UTF-16 units, including escaped isolated surrogates.
use std::collections::BTreeMap;
/// JSON lookup retains sorted keys while recording insertion order for Java HashMap iteration.
#[derive(Clone, Debug, Default)]
pub struct Object {
    values: BTreeMap<Vec<u16>, Value>,
    insertion: Vec<Vec<u16>>,
}
impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
impl Object {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn insert(&mut self, key: Vec<u16>, value: Value) -> Option<Value> {
        if !self.values.contains_key(&key) {
            self.insertion.push(key.clone());
        }
        self.values.insert(key, value)
    }
    /// Stable list-bucket iteration used by ordinary JSONObject objects. Java's
    /// treeified collision bins require separate reconciliation for adversarial key sets.
    pub fn java_keys(&self) -> Vec<&Vec<u16>> {
        let mut capacity = 16usize;
        while self.values.len() > capacity * 3 / 4 {
            capacity *= 2;
        }
        let mut keys = self.insertion.iter().collect::<Vec<_>>();
        keys.sort_by_key(|key| {
            let hash = key.iter().fold(0u32, |hash, &unit| {
                hash.wrapping_mul(31).wrapping_add(u32::from(unit))
            });
            (hash ^ (hash >> 16)) as usize & (capacity - 1)
        });
        keys
    }
}
impl std::ops::Deref for Object {
    type Target = BTreeMap<Vec<u16>, Value>;
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
impl<'a> IntoIterator for &'a Object {
    type Item = (&'a Vec<u16>, &'a Value);
    type IntoIter = std::collections::btree_map::Iter<'a, Vec<u16>, Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}
impl<const N: usize> From<[(Vec<u16>, Value); N]> for Object {
    fn from(entries: [(Vec<u16>, Value); N]) -> Self {
        let mut object = Self::new();
        for (key, value) in entries {
            object.insert(key, value);
        }
        object
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(Vec<u16>),
    Array(Vec<Value>),
    Object(Object),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Syntax,
    Runtime,
}
type Result<T> = std::result::Result<T, Failure>;
impl Value {
    pub fn number(&self) -> Option<f64> {
        match self {
            Self::Integer(v) => Some(*v as f64),
            Self::Float(v) => Some(*v),
            Self::String(v) => double(&String::from_utf16(v).ok()?),
            _ => None,
        }
    }
    pub fn int(&self) -> Option<i32> {
        match self {
            Self::Integer(v) => Some(*v as i32),
            Self::Float(v) => Some(*v as i32),
            // This pinned JSONObject.getInt routes strings through getDouble,
            // then Java's saturating double-to-int conversion.
            Self::String(v) => double(&String::from_utf16(v).ok()?).map(|v| v as i32),
            _ => None,
        }
    }
    fn key(self) -> Result<Vec<u16>> {
        Ok(match self {
            Self::String(v) => v,
            Self::Null => "null".encode_utf16().collect(),
            Self::Bool(v) => v.to_string().encode_utf16().collect(),
            Self::Integer(v) => v.to_string().encode_utf16().collect(),
            // Non-string structured/float keys require Java serialization rules.
            _ => return Err(Failure::Syntax),
        })
    }
}
pub fn content(text: &str) -> Result<Object> {
    match text.as_bytes().first() {
        None => return Err(Failure::Runtime),
        Some(b'{' | b'[') => {}
        _ => return Err(Failure::Syntax),
    }
    let mut parser = Parser {
        units: text.encode_utf16().collect(),
        pos: 0,
    };
    match parser.value(0)? {
        Value::Object(v) => Ok(v),
        value @ Value::Array(_) => Ok(Object::from([(
            "arrayValues".encode_utf16().collect(),
            value,
        )])),
        _ => Err(Failure::Syntax),
    }
}
struct Parser {
    units: Vec<u16>,
    pos: usize,
}
impl Parser {
    fn next(&mut self) -> u16 {
        let c = self.units.get(self.pos).copied().unwrap_or(0);
        if self.pos < self.units.len() {
            self.pos += 1;
        }
        c
    }
    fn peek_clean(&mut self) -> u16 {
        while self.units.get(self.pos).is_some_and(|c| *c > 0 && *c <= 32) {
            self.pos += 1;
        }
        self.units.get(self.pos).copied().unwrap_or(0)
    }
    fn clean(&mut self) -> u16 {
        self.peek_clean();
        self.next()
    }
    fn value(&mut self, depth: usize) -> Result<Value> {
        if depth > 256 {
            return Err(Failure::Syntax);
        }
        match self.clean() {
            quote @ (34 | 39) => Ok(Value::String(self.string(quote)?)),
            123 => self.object(depth + 1),
            open @ (91 | 40) => self.array(open, depth + 1),
            0 => Err(Failure::Syntax),
            first => {
                self.pos -= 1;
                let start = self.pos;
                while let Some(&c) = self.units.get(self.pos) {
                    if c < 32 || ",:]}/\\\"[{;=#".encode_utf16().any(|v| v == c) {
                        break;
                    }
                    self.pos += 1;
                }
                let mut token = &self.units[start..self.pos];
                while token.last().is_some_and(|c| *c <= 32) {
                    token = &token[..token.len() - 1];
                }
                if token.is_empty() || first < 32 {
                    return Err(Failure::Syntax);
                }
                Ok(scalar(token))
            }
        }
    }
    fn string(&mut self, quote: u16) -> Result<Vec<u16>> {
        let mut out = Vec::new();
        loop {
            match self.next() {
                0 | 10 | 13 => return Err(Failure::Syntax),
                c if c == quote => return Ok(out),
                92 => {
                    let c = self.next();
                    out.push(match c {
                        98 => 8,
                        116 => 9,
                        110 => 10,
                        102 => 12,
                        114 => 13,
                        117 | 120 => {
                            let len = if c == 117 { 4 } else { 2 };
                            let end = self.pos + len;
                            let digits = self.units.get(self.pos..end).ok_or(Failure::Syntax)?;
                            if digits.contains(&0) {
                                return Err(Failure::Syntax);
                            }
                            let value = radix(digits, 16).ok_or(Failure::Runtime)?;
                            self.pos = end;
                            value as u16
                        }
                        other => other,
                    });
                }
                c => out.push(c),
            }
        }
    }
    fn object(&mut self, depth: usize) -> Result<Value> {
        let mut values = Object::new();
        if self.peek_clean() == 125 {
            self.next();
            return Ok(Value::Object(values));
        }
        loop {
            let key = self.value(depth)?.key()?;
            match self.clean() {
                58 => {}
                61 => {
                    if self.units.get(self.pos) == Some(&62) {
                        self.pos += 1;
                    }
                }
                _ => return Err(Failure::Syntax),
            }
            let value = self.value(depth)?;
            if matches!(&value, Value::Float(v) if !v.is_finite()) || values.contains_key(&key) {
                return Err(Failure::Syntax);
            }
            values.insert(key, value);
            match self.clean() {
                125 => return Ok(Value::Object(values)),
                44 | 59 => {
                    if self.peek_clean() == 125 {
                        self.next();
                        return Ok(Value::Object(values));
                    }
                }
                _ => return Err(Failure::Syntax),
            }
        }
    }
    fn array(&mut self, open: u16, depth: usize) -> Result<Value> {
        let close = if open == 91 { 93 } else { 41 };
        let mut values = Vec::new();
        // The pinned implementation tests ']' here even for '(' arrays.
        if self.peek_clean() == 93 {
            self.next();
            return Ok(Value::Array(values));
        }
        loop {
            values.push(if self.peek_clean() == 44 {
                Value::Null
            } else {
                self.value(depth)?
            });
            match self.clean() {
                c @ (41 | 93) if c == close => return Ok(Value::Array(values)),
                44 | 59 => {
                    if self.peek_clean() == 93 {
                        self.next();
                        return Ok(Value::Array(values));
                    }
                }
                _ => return Err(Failure::Syntax),
            }
        }
    }
}
fn radix(units: &[u16], base: u32) -> Option<i64> {
    let mut iter = units.iter().copied().peekable();
    let negative = iter.peek() == Some(&45);
    if negative || iter.peek() == Some(&43) {
        iter.next();
    }
    let mut value = 0i64;
    let mut count = 0;
    for unit in iter {
        let digit = match unit {
            65..=90 => (unit - 65 + 10) as i32,
            97..=122 => (unit - 97 + 10) as i32,
            0xff21..=0xff3a => (unit - 0xff21 + 10) as i32,
            0xff41..=0xff5a => (unit - 0xff41 + 10) as i32,
            _ => crate::script_string::parse_java_int(&String::from_utf16(&[unit]).ok()?)?,
        };
        if digit < 0 || digit >= base as i32 {
            return None;
        }
        value = value.checked_mul(base as i64)?.checked_sub(digit as i64)?;
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
fn scalar(units: &[u16]) -> Value {
    let Ok(text) = String::from_utf16(units) else {
        return Value::String(units.to_vec());
    };
    if text.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if text.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }
    if text.eq_ignore_ascii_case("null") {
        return Value::Null;
    }
    if matches!(units[0], 48..=57 | 46 | 45 | 43) {
        if units[0] == 48 {
            let octal_or_hex = if units.len() > 2 && matches!(units[1], 120 | 88) {
                radix(&units[2..], 16)
            } else {
                radix(units, 8)
            };
            if let Some(v) = octal_or_hex.filter(|v| i32::try_from(*v).is_ok()) {
                return Value::Integer(v);
            }
        }
        if let Some(v) = radix(units, 10) {
            return Value::Integer(v);
        }
        if let Some(v) = double(&text) {
            return Value::Float(v);
        }
    }
    Value::String(units.to_vec())
}
fn double(text: &str) -> Option<f64> {
    let text = text.trim_matches(|c: char| c <= '\u{20}');
    match text {
        "NaN" | "+NaN" | "-NaN" => return Some(f64::NAN),
        "Infinity" | "+Infinity" => return Some(f64::INFINITY),
        "-Infinity" => return Some(f64::NEG_INFINITY),
        _ => {}
    }
    let text = text.strip_suffix(['f', 'F', 'd', 'D']).unwrap_or(text);
    let unsigned = text.strip_prefix(['+', '-']).unwrap_or(text);
    if let Some(hex) = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        let split = hex.find(['p', 'P'])?;
        let mut exponent = hex[split + 1..].parse::<i32>().ok()?;
        let mut value = 0.0;
        let mut fraction = false;
        let mut digits = 0;
        for c in hex[..split].chars() {
            if c == '.' && !fraction {
                fraction = true;
                continue;
            }
            value = value * 16.0 + c.to_digit(16)? as f64;
            if fraction {
                exponent = exponent.checked_sub(4)?;
            }
            digits += 1;
        }
        if digits == 0 {
            return None;
        }
        let value = value * 2.0f64.powi(exponent);
        return Some(if text.starts_with('-') { -value } else { value });
    }
    if !text
        .bytes()
        .all(|c| c.is_ascii_digit() || b"+-.eE".contains(&c))
    {
        return None;
    }
    text.parse().ok()
}
