//! Additional revision-239 array instructions (ve.cs).
use crate::{
    script_array_storage::Element,
    vm::{Array, Object, Stacks, Value},
    Error, Result,
};
use std::{cell::RefCell, rc::Rc};
fn pop(s: &mut Stacks) -> Result<Rc<RefCell<Array>>> {
    match s.pop_object()? {
        Object::Array(a) => Ok(a),
        _ => Err(Error::Invalid("array required")),
    }
}
fn read(a: &Array, i: usize) -> Result<Element> {
    Ok(match a.data() {
        Array::Ints(v) => Element::Int(*v.get(i).ok_or(Error::Invalid("array index"))?),
        Array::Longs(v) => Element::Long(*v.get(i).ok_or(Error::Invalid("array index"))?),
        Array::Strings(v) => {
            Element::String(v.get(i).ok_or(Error::Invalid("array index"))?.clone())
        }
        _ => unreachable!(),
    })
}
fn assign(a: &mut Array, i: usize, v: Element) -> Result<()> {
    match (a.data_mut(), v) {
        (Array::Ints(a), Element::Int(v)) => {
            *a.get_mut(i).ok_or(Error::Invalid("array index"))? = v
        }
        (Array::Longs(a), Element::Long(v)) => {
            *a.get_mut(i).ok_or(Error::Invalid("array index"))? = v
        }
        (Array::Strings(a), Element::String(v)) => {
            *a.get_mut(i).ok_or(Error::Invalid("array index"))? = v
        }
        _ => return Err(Error::Invalid("array element type")),
    }
    Ok(())
}
fn value(v: Option<Value>) -> Result<Element> {
    Ok(match v {
        Some(Value::Int(v)) => Element::Int(v),
        Some(Value::Long(v)) => Element::Long(v),
        Some(Value::Object(Object::String(v))) => Element::String(Some(v)),
        None | Some(Value::Object(Object::Null)) => Element::String(None),
        _ => return Err(Error::Invalid("array element type")),
    })
}
fn push(s: &mut Stacks, v: Element) -> Result<()> {
    match v {
        Element::Null => s.push_object(Object::Null),
        Element::Int(v) => s.push_int(v),
        Element::Long(v) => s.push_long(v),
        Element::String(v) => s.push_object(v.map_or(Object::Null, Object::String)),
    }
}
fn same_type(a: &Array, b: &Array) -> bool {
    std::mem::discriminant(a.data()) == std::mem::discriminant(b.data())
}
fn range(a: &Array, start: i32, count: i32) -> (usize, usize) {
    let start = start.max(0);
    let end = start.wrapping_add(count);
    (
        start as usize,
        if count < 0 || end < 0 || end as usize > a.len() {
            a.len()
        } else {
            end as usize
        },
    )
}
fn reverse(a: &mut Array) {
    match a.data_mut() {
        Array::Ints(v) => v.reverse(),
        Array::Longs(v) => v.reverse(),
        Array::Strings(v) => v.reverse(),
        _ => unreachable!(),
    }
}
fn rotate(a: &mut Array, n: i32) {
    if a.len() < 2 {
        return;
    }
    let n = n.rem_euclid(a.len() as i32) as usize;
    match a.data_mut() {
        Array::Ints(v) => v.rotate_right(n),
        Array::Longs(v) => v.rotate_right(n),
        Array::Strings(v) => v.rotate_right(n),
        _ => unreachable!(),
    }
}
fn truncate(a: &mut Array, n: usize) {
    match a.data_mut() {
        Array::Ints(v) => v.truncate(n),
        Array::Longs(v) => v.truncate(n),
        Array::Strings(v) => v.truncate(n),
        _ => unreachable!(),
    }
}
fn extend_zero(a: &mut Array, n: usize) {
    match a.data_mut() {
        Array::Ints(v) => v.resize(n, 0),
        Array::Longs(v) => v.resize(n, 0),
        Array::Strings(v) => v.resize(n, None),
        _ => unreachable!(),
    }
}
fn copy(a: &Array, from: usize, b: &mut Array, to: usize, count: usize) -> Result<()> {
    let values = (from..from + count)
        .map(|i| read(a, i))
        .collect::<Result<Vec<_>>>()?;
    for (i, v) in values.into_iter().enumerate() {
        assign(b, to + i, v)?;
    }
    Ok(())
}
fn within(a: &mut Array, from: usize, to: usize, count: usize) -> Result<()> {
    let values = (from..from + count)
        .map(|i| read(a, i))
        .collect::<Result<Vec<_>>>()?;
    for (i, v) in values.into_iter().enumerate() {
        assign(a, to + i, v)?;
    }
    Ok(())
}
pub fn execute(op: u16, s: &mut Stacks) -> Result<bool> {
    match op {
        8008 | 8009 => {
            let a = pop(s)?;
            let a = a.borrow();
            let mut best: Option<Element> = None;
            for i in 0..a.len() {
                let v = read(&a, i)?;
                let replace = if let Some(old) = &best {
                    match (&v, old) {
                        (Element::Int(v), Element::Int(o)) => {
                            if op == 8008 {
                                v < o
                            } else {
                                v > o
                            }
                        }
                        (Element::Long(v), Element::Long(o)) => {
                            if op == 8008 {
                                v < o
                            } else {
                                v > o
                            }
                        }
                        (Element::String(None), _) => false,
                        (Element::String(Some(v)), Element::String(Some(o))) => {
                            let c = v.encode_utf16().cmp(o.encode_utf16());
                            if op == 8008 {
                                c.is_lt()
                            } else {
                                c.is_gt()
                            }
                        }
                        _ => return Err(Error::Invalid("null string extremum")),
                    }
                } else {
                    true
                };
                if replace {
                    best = Some(v)
                }
            }
            match a.data() {
                Array::Ints(_) => push(s, best.unwrap_or(Element::Int(-1)))?,
                Array::Strings(_) => {
                    let v = match best {
                        Some(Element::String(Some(v))) => v,
                        _ => String::new(),
                    };
                    s.push_object(Object::String(v))?
                }
                _ => return Err(Error::Invalid("long extremum opcode")),
            }
        }
        8010 => {
            let [start, count, kind] = crate::interface_script::ints(s)?;
            let value = value(crate::param_def::pop(s, kind)?)?;
            let a = pop(s)?;
            let mut a = a.borrow_mut();
            a.writable()?;
            let (start, end) = range(&a, start, count);
            if start > end {
                return Err(Error::Invalid("array fill range"));
            }
            let matches = matches!(
                (a.data(), &value),
                (Array::Ints(_), Element::Int(_))
                    | (Array::Longs(_), Element::Long(_))
                    | (Array::Strings(_), Element::String(_))
            );
            if !matches && !(matches!(a.data(), Array::Strings(_)) && start == end) {
                return Err(Error::Invalid("array fill type"));
            }
            for i in start..end {
                assign(&mut a, i, value.clone())?;
            }
        }
        8011 => {
            let [first, start, count] = crate::interface_script::ints(s)?;
            let a = pop(s)?;
            let mut a = a.borrow_mut();
            a.writable()?;
            let (start, end) = range(&a, start, count);
            let Array::Ints(v) = a.data_mut() else {
                return Err(Error::Invalid("array sequence type"));
            };
            let mut next = first;
            for i in start..end {
                *v.get_mut(i).ok_or(Error::Invalid("array sequence range"))? = next;
                next = next.wrapping_add(1);
            }
        }
        8012 | 8013 | 8014 => {
            let args = match op {
                8013 => [s.pop_int()?, 0],
                8014 => crate::interface_script::ints(s)?,
                _ => [0; 2],
            };
            let a = pop(s)?;
            let mut a = a.borrow_mut();
            a.writable()?;
            match op {
                8012 => reverse(&mut a),
                8013 => rotate(&mut a, args[0]),
                _ => {
                    let x = read(&a, args[0] as usize)?;
                    let y = read(&a, args[1] as usize)?;
                    assign(&mut a, args[0] as usize, y)?;
                    assign(&mut a, args[1] as usize, x)?;
                }
            }
        }
        8015 => {
            let [from, to, mut count] = crate::interface_script::ints(s)?;
            let b = pop(s)?;
            let a = pop(s)?;
            let alias = Rc::ptr_eq(&a, &b);
            {
                let a = a.borrow();
                let b = b.borrow();
                b.writable()?;
                if !same_type(&a, &b) {
                    return Err(Error::Invalid("array copy types"));
                }
                if count == -1 {
                    count = (a.len() as i32).wrapping_sub(from)
                }
                if count == 0 || (alias && from == to) {
                    return Ok(true);
                }
                if count < 0
                    || from < 0
                    || to < 0
                    || from as usize + count as usize > a.len()
                    || to as usize + count as usize > b.len()
                {
                    return Err(Error::Invalid("array copy range"));
                }
            }
            if alias {
                within(
                    &mut a.borrow_mut(),
                    from as usize,
                    to as usize,
                    count as usize,
                )?;
            } else {
                copy(
                    &a.borrow(),
                    from as usize,
                    &mut b.borrow_mut(),
                    to as usize,
                    count as usize,
                )?;
            }
        }
        8016 | 8017 => {
            let mut limit = if op == 8017 { s.pop_int()? } else { 0 };
            let a = pop(s)?;
            let a = a.borrow();
            let Array::Ints(v) = a.data() else {
                return Err(Error::Invalid("integer array required"));
            };
            let result = if op == 8016 {
                v.iter().fold(0i32, |n, &x| n.wrapping_add(x))
            } else {
                let mut index = -1;
                if limit >= 0 {
                    for (i, &x) in v.iter().enumerate() {
                        if limit < x {
                            index = i as i32;
                            break;
                        }
                        limit = limit.wrapping_sub(x);
                    }
                }
                index
            };
            s.push_int(result)?;
        }
        8018 => {
            let delimiter = s.pop_object()?;
            let text = s.pop_object()?;
            let (Object::String(text), Object::String(delimiter)) = (text, delimiter) else {
                return Err(Error::Invalid("array split strings"));
            };
            if delimiter.is_empty() {
                return Err(Error::Invalid("empty array delimiter"));
            }
            let parts: Vec<_> = text.split(&delimiter).map(|v| Some(v.to_owned())).collect();
            let capacity = parts.len();
            s.push_object(Object::Array(Rc::new(RefCell::new(Array::managed(
                Array::Strings(parts),
                Element::String(None),
                capacity,
                true,
            )))))?;
        }
        8019 => {
            let delimiter = s.pop_object()?;
            if !matches!(delimiter, Object::String(_) | Object::Null) {
                return Err(Error::Invalid("array join delimiter type"));
            }
            let a = pop(s)?;
            let a = a.borrow();
            let Array::Strings(parts) = a.data() else {
                return Err(Error::Invalid("array join type"));
            };
            let value = match parts.as_slice() {
                [] => Some(String::new()),
                [part] => part.clone(),
                _ => {
                    let Object::String(delimiter) = delimiter else {
                        return Err(Error::Invalid("null array delimiter"));
                    };
                    let parts = parts
                        .iter()
                        .map(|v| {
                            v.as_deref()
                                .ok_or(Error::Invalid("null array join element"))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    Some(parts.join(&delimiter))
                }
            };
            s.push_object(value.map_or(Object::Null, Object::String))?;
        }
        8022 => {
            let [kind, size, capacity] = crate::interface_script::ints(s)?;
            s.push_object(Object::Array(Rc::new(RefCell::new(Array::create(
                kind, size, capacity,
            )?))))?;
        }
        8023 => {
            let n = s.pop_int()?;
            let a = pop(s)?;
            if n < 0 {
                return Err(Error::Invalid("array size"));
            }
            a.borrow_mut().resize(n as usize)?;
        }
        8024 | 8025 => {
            let (index, kind) = if op == 8025 {
                let [i, k] = crate::interface_script::ints(s)?;
                (Some(i), k)
            } else {
                (None, s.pop_int()?)
            };
            let value = value(crate::param_def::pop(s, kind)?)?;
            let a = pop(s)?;
            let mut a = a.borrow_mut();
            a.writable()?;
            let n = a.len();
            let index = index.unwrap_or(n as i32);
            if index < 0 || index as usize > n {
                return Err(Error::Invalid("array insertion index"));
            }
            a.reserve_length(n + 1)?;
            extend_zero(&mut a, n + 1);
            within(
                &mut a,
                index as usize,
                index as usize + 1,
                n - index as usize,
            )?;
            let result = assign(&mut a, index as usize, value);
            if result.is_err() {
                truncate(&mut a, n)
            }
            result?;
        }
        8026 => {
            let index = s.pop_int()? as usize;
            let a = pop(s)?;
            let mut a = a.borrow_mut();
            a.writable()?;
            let value = read(&a, index)?;
            let n = a.len();
            within(&mut a, index + 1, index, n - index - 1)?;
            truncate(&mut a, n - 1);
            push(s, value)?;
        }
        8027 | 8028 => {
            let index = if op == 8028 { Some(s.pop_int()?) } else { None };
            let b = pop(s)?;
            let a = pop(s)?;
            let alias = Rc::ptr_eq(&a, &b);
            let incoming = b.borrow().data().clone();
            let mut a = a.borrow_mut();
            a.writable()?;
            if !same_type(&a, &incoming) {
                return Err(Error::Invalid("array insertion types"));
            }
            let n = a.len();
            let index = index.unwrap_or(n as i32);
            if index < 0 || index as usize > n {
                return Err(Error::Invalid("array insertion index"));
            }
            let count = incoming.len();
            if count != 0 {
                a.reserve_length(n + count)?;
                extend_zero(&mut a, n + count);
                within(
                    &mut a,
                    index as usize,
                    index as usize + count,
                    n - index as usize,
                )?;
                if alias {
                    within(&mut a, 0, index as usize, count)?;
                } else {
                    copy(&incoming, 0, &mut a, index as usize, count)?;
                }
            }
        }
        8029 => {
            let [count, index] = crate::interface_script::ints(s)?;
            let a = pop(s)?;
            let mut a = a.borrow_mut();
            a.writable()?;
            if count != 0 {
                if count < 0
                    || index < 0
                    || index as usize >= a.len()
                    || index as usize + count as usize > a.len()
                {
                    return Err(Error::Invalid("array removal range"));
                }
                let n = a.len();
                within(
                    &mut a,
                    index as usize + count as usize,
                    index as usize,
                    n - index as usize - count as usize,
                )?;
                truncate(&mut a, n - count as usize);
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}
