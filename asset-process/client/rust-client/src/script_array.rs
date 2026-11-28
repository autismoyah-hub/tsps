//! Revision 239 mutable-array sorting (ve.cs / pr.ab).
use crate::{
    vm::{Array, Object, Stacks},
    Error, Result,
};
use std::{cell::RefCell, rc::Rc};
fn array(value: Object) -> Result<Rc<RefCell<Array>>> {
    match value {
        Object::Array(a) => Ok(a),
        _ => Err(Error::Invalid("expected mutable script array")),
    }
}
fn len(a: &Array) -> usize {
    a.len()
}
fn swap(a: &mut Array, x: usize, y: usize) {
    match a.data_mut() {
        Array::Ints(v) => v.swap(x, y),
        Array::Longs(v) => v.swap(x, y),
        Array::Managed(_) => unreachable!(),
        Array::Strings(v) => v.swap(x, y),
    }
}
fn before(a: &Array, i: usize, pivot: usize) -> bool {
    match a.data() {
        Array::Managed(_) => unreachable!(),
        Array::Longs(v) => {
            v[i] < v[pivot]
                + if v[pivot] == i64::MAX {
                    0
                } else {
                    (i & 1) as i64
                }
        }
        Array::Ints(v) => {
            v[i] < v[pivot]
                + if v[pivot] == i32::MAX {
                    0
                } else {
                    (i & 1) as i32
                }
        }
        Array::Strings(v) => match (&v[i], &v[pivot]) {
            (_, None) => true,
            (None, Some(_)) => false,
            (Some(x), Some(y)) => {
                let order = x.encode_utf16().cmp(y.encode_utf16());
                order.is_lt() || (order.is_eq() && i & 1 != 0)
            }
        },
    }
}
pub fn sort(keys: &mut Array, mut associated: Option<&mut Array>) -> Result<()> {
    let count = len(keys);
    if count <= 1 {
        return Ok(());
    }
    keys.writable()?;
    if let Some(a) = associated.as_ref() {
        a.writable()?;
    }
    if associated.as_ref().is_some_and(|a| len(a) < count) {
        return Err(Error::Invalid("associated array shorter than keys"));
    }
    if associated.is_none() {
        match keys.data_mut() {
            Array::Strings(v) => {
                return crate::java_sort::try_sort(v, |a, b| {
                    let (Some(a), Some(b)) = (a, b) else {
                        return Err(Error::Invalid("null string in unpaired array sort"));
                    };
                    Ok(a.encode_utf16().cmp(b.encode_utf16()))
                })
            }
            Array::Longs(v) => {
                v.sort();
                return Ok(());
            }
            _ => {}
        }
    }
    // Iterative form of the original midpoint-pivot quicksort. Preserve tie permutations.
    let mut ranges = vec![(0, count - 1)];
    while let Some((lo, hi)) = ranges.pop() {
        if lo >= hi {
            continue;
        }
        let mid = (lo + hi) / 2;
        swap(keys, mid, hi);
        if let Some(a) = associated.as_deref_mut() {
            swap(a, mid, hi);
        }
        let mut store = lo;
        for i in lo..hi {
            if before(keys, i, hi) {
                swap(keys, i, store);
                if let Some(a) = associated.as_deref_mut() {
                    swap(a, i, store);
                }
                store += 1;
            }
        }
        swap(keys, hi, store);
        if let Some(a) = associated.as_deref_mut() {
            swap(a, hi, store);
        }
        if store + 1 < hi {
            ranges.push((store + 1, hi));
        }
        if store > lo {
            ranges.push((lo, store - 1));
        }
    }
    Ok(())
}
pub fn search(
    a: Option<&Array>,
    value: &Option<crate::vm::Value>,
    start: i32,
    end: i32,
    op: u16,
) -> Result<i32> {
    let Some(a) = a else {
        return Ok(if op == 8007 { 0 } else { -1 });
    };
    let length = len(a);
    let start = start.max(0) as usize;
    let end = if end < 0 || end as usize > length {
        length
    } else {
        end as usize
    };
    if matches!(a.data(), Array::Ints(_)) && !matches!(value, Some(crate::vm::Value::Int(_))) {
        return Err(Error::Invalid("integer array search value"));
    }
    if matches!(a.data(), Array::Longs(_)) && !matches!(value, Some(crate::vm::Value::Long(_))) {
        return Err(Error::Invalid("long array search value"));
    }
    let mut result = if op == 8007 { 0 } else { -1 };
    for i in start..end {
        let matched = match a.data() {
            Array::Managed(_) => unreachable!(),
            Array::Longs(v) => matches!(value,Some(crate::vm::Value::Long(n)) if v[i]==*n),
            Array::Ints(v) => matches!(value,Some(crate::vm::Value::Int(n)) if v[i]==*n),
            Array::Strings(v) => match (&v[i], value) {
                (None, None) | (None, Some(crate::vm::Value::Object(Object::Null))) => true,
                (Some(a), Some(crate::vm::Value::Object(Object::String(b)))) => a == b,
                _ => false,
            },
        };
        if matched {
            if op == 8007 {
                result += 1;
            } else {
                result = i as i32;
                if op == 8005 {
                    break;
                }
            }
        }
    }
    Ok(result)
}
pub fn execute(op: u16, s: &mut Stacks) -> Result<bool> {
    if crate::script_array_ops::execute(op, s)? {
        return Ok(true);
    }
    match op {
        8005..=8007 => {
            let [start, end, kind] = crate::interface_script::ints(s)?;
            let value = crate::param_def::pop(s, kind)?;
            let a = match s.pop_object()? {
                Object::Null => None,
                v => Some(array(v)?),
            };
            let result = if let Some(a) = a {
                search(Some(&a.borrow()), &value, start, end, op)?
            } else {
                search(None, &value, start, end, op)?
            };
            s.push_int(result)?;
        }
        8000 => {
            let second = s.pop_object()?;
            let first = array(s.pop_object()?)?;
            let second = match second {
                Object::Null => None,
                v => Some(array(v)?),
            };
            if second.as_ref().is_some_and(|a| Rc::ptr_eq(a, &first)) {
                sort_alias(&mut first.borrow_mut())?;
                return Ok(true);
            }
            let mut keys = first.borrow_mut();
            if let Some(second) = second {
                sort(&mut keys, Some(&mut second.borrow_mut()))?;
            } else {
                sort(&mut keys, None)?;
            }
        }
        8001 => {
            let [high, low] = crate::interface_script::ints(s)?;
            let a = array(s.pop_object()?)?;
            shuffle(&mut a.borrow_mut(), high, low)?;
        }
        8004 => {
            let b = s.pop_object()?;
            let a = s.pop_object()?;
            let a = match a {
                Object::Null => None,
                v => Some(array(v)?),
            };
            let b = match b {
                Object::Null => None,
                v => Some(array(v)?),
            };
            let value = if a
                .as_ref()
                .zip(b.as_ref())
                .is_some_and(|(a, b)| Rc::ptr_eq(a, b))
            {
                0
            } else {
                compare(
                    a.as_ref().map(|a| a.borrow()).as_deref(),
                    b.as_ref().map(|a| a.borrow()).as_deref(),
                )?
            };
            s.push_int(value)?;
        }
        8002 => {
            let value = s.pop_object()?;
            s.push_int(i32::from(matches!(value, Object::Null)))?;
        }
        8003 => {
            let value = s.pop_object()?;
            let length = match value {
                Object::Null => 0,
                value => len(&array(value)?.borrow()),
            };
            s.push_int(length as i32)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

pub fn shuffle(a: &mut Array, mut high: i32, mut low: i32) -> Result<()> {
    if len(a) <= 1 {
        return Ok(());
    }
    if high == 0 && low == 0 {
        let seed = crate::login::random_seed()?;
        high = (seed[0] % 2147483647) as i32;
        low = (seed[1] % 2147483647) as i32;
    }
    // Java sign-extends the low int before OR, including negative low seeds.
    a.writable()?;
    let seed = ((high as i64) << 32) | (low as i64);
    let mut rng = (seed as u64 ^ 0x5deece66d) & ((1 << 48) - 1);
    for i in (1..len(a)).rev() {
        let bound = (i + 1) as i32;
        let index = loop {
            rng = rng.wrapping_mul(0x5deece66d).wrapping_add(11) & ((1 << 48) - 1);
            let bits = (rng >> 17) as i32;
            if bound & (bound - 1) == 0 {
                break ((bound as i64 * bits as i64) >> 31) as usize;
            }
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                break value as usize;
            }
        };
        swap(a, i, index);
    }
    Ok(())
}
pub fn compare(a: Option<&Array>, b: Option<&Array>) -> Result<i32> {
    use std::cmp::Ordering;
    let (a, b) = match (a, b) {
        (None, None) => return Ok(0),
        (None, _) => return Ok(-1),
        (_, None) => return Ok(1),
        (Some(a), Some(b)) => (a, b),
    };
    let order = match (a.data(), b.data()) {
        (Array::Ints(a), Array::Ints(b)) => a.cmp(b),
        (Array::Longs(a), Array::Longs(b)) => a.cmp(b),
        (Array::Strings(a), Array::Strings(b)) => {
            let mut order = Ordering::Equal;
            for (a, b) in a.iter().zip(b) {
                let (Some(a), Some(b)) = (a, b) else {
                    return Err(Error::Invalid("null array comparison element"));
                };
                order = a.encode_utf16().cmp(b.encode_utf16());
                if order != Ordering::Equal {
                    break;
                }
            }
            if order == Ordering::Equal {
                a.len().cmp(&b.len())
            } else {
                order
            }
        }
        _ => return Err(Error::Invalid("array comparison types")),
    };
    Ok(match order {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    })
}
pub fn sort_alias(a: &mut Array) -> Result<()> {
    let count = len(a);
    if count <= 1 {
        return Ok(());
    }
    a.writable()?;
    // The original saves the key pivot before its first swap, and the associated
    // pivot after it. With one shared array those pivots differ.
    let mut ranges = vec![(0, count - 1)];
    while let Some((lo, hi)) = ranges.pop() {
        if lo >= hi {
            continue;
        }
        let mid = (lo + hi) / 2;
        let pivot = get(a, mid);
        swap(a, mid, hi);
        let paired = get(a, mid);
        swap(a, mid, hi);
        let mut store = lo;
        for i in lo..hi {
            if before_cell(a, i, &pivot) {
                swap(a, i, store);
                swap(a, i, store);
                store += 1;
            }
        }
        let value = get(a, store);
        set(a, hi, value);
        set(a, store, pivot);
        let value = get(a, store);
        set(a, hi, value);
        set(a, store, paired);
        if store + 1 < hi {
            ranges.push((store + 1, hi))
        }
        if store > lo {
            ranges.push((lo, store - 1))
        }
    }
    Ok(())
}
#[derive(Clone)]
enum Cell {
    Int(i32),
    Long(i64),
    String(Option<String>),
}
fn get(a: &Array, i: usize) -> Cell {
    match a.data() {
        Array::Managed(_) => unreachable!(),
        Array::Ints(v) => Cell::Int(v[i]),
        Array::Longs(v) => Cell::Long(v[i]),
        Array::Strings(v) => Cell::String(v[i].clone()),
    }
}
fn set(a: &mut Array, i: usize, c: Cell) {
    match (a.data_mut(), c) {
        (Array::Ints(v), Cell::Int(c)) => v[i] = c,
        (Array::Longs(v), Cell::Long(c)) => v[i] = c,
        (Array::Strings(v), Cell::String(c)) => v[i] = c,
        _ => unreachable!(),
    }
}
fn before_cell(a: &Array, i: usize, pivot: &Cell) -> bool {
    match (get(a, i), pivot) {
        (Cell::Int(a), Cell::Int(b)) => a < *b + if *b == i32::MAX { 0 } else { (i & 1) as i32 },
        (Cell::Long(a), Cell::Long(b)) => a < *b + if *b == i64::MAX { 0 } else { (i & 1) as i64 },
        (Cell::String(a), Cell::String(b)) => match (a, b) {
            (_, None) => true,
            (None, Some(_)) => false,
            (Some(a), Some(b)) => {
                let order = a.encode_utf16().cmp(b.encode_utf16());
                order.is_lt() || (order.is_eq() && i & 1 != 0)
            }
        },
        _ => unreachable!(),
    }
}
