//! Typed event argument substitution from the revision 239 interpreter.
use crate::{
    vm::{Object, Value},
    Error, Result,
};
#[derive(Clone, Debug, Default)]
pub struct Context {
    pub resize_depth: i32,
    pub mouse: [i32; 2],
    pub widget: Option<(u32, i32)>,
    pub drag_target: Option<(u32, i32)>,
    pub operation: i32,
    pub key_code: i32,
    pub key_char: i32,
    pub extra: i32,
    pub operation_base: Option<String>,
}
pub struct Arguments {
    pub script: i32,
    pub ints: Vec<i32>,
    pub objects: Vec<Object>,
    pub longs: Vec<i64>,
}
pub fn bind(values: &[Value], context: &Context) -> Result<Arguments> {
    let Some(Value::Int(script)) = values.first() else {
        return Err(Error::Invalid("event script id"));
    };
    let mut result = Arguments {
        script: *script,
        ints: vec![],
        objects: vec![],
        longs: vec![],
    };
    for value in &values[1..] {
        match value {
            Value::Int(v) => result.ints.push(match *v {
                -2147483647 => context.mouse[0],
                -2147483646 => context.mouse[1],
                -2147483645 => context.widget.map_or(-1, |w| w.0 as i32),
                -2147483644 => context.operation,
                -2147483643 => context.widget.map_or(-1, |w| w.1),
                -2147483642 => context.drag_target.map_or(-1, |w| w.0 as i32),
                -2147483641 => context.drag_target.map_or(-1, |w| w.1),
                -2147483640 => context.key_code,
                -2147483639 => context.key_char,
                -2147483638 => context.extra,
                _ => *v,
            }),
            Value::Long(v) => result.longs.push(*v),
            Value::Object(Object::String(v)) if v == "event_opbase" => result.objects.push(
                context
                    .operation_base
                    .clone()
                    .map_or(Object::Null, Object::String),
            ),
            Value::Object(v) => result.objects.push(v.clone()),
        }
    }
    Ok(result)
}
