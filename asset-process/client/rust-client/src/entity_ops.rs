//! Conditional operation labels and child operations (GameEntityOps/oe/oi).
use crate::{
    object::Operation,
    script_runtime::{Source, Variables},
    Result,
};
pub fn label<'a>(
    operations: &'a [Operation],
    index: u8,
    sub: Option<u8>,
    variables: &Variables,
    source: &mut impl Source,
) -> Result<Option<&'a str>> {
    let start = operations
        .iter()
        .rposition(|op| op.index == index && op.sub.is_none() && op.condition.is_none())
        .unwrap_or(0);
    let operations = &operations[start..];
    let start = operations
        .iter()
        .rposition(|op| op.index == index && op.sub == sub && op.condition.is_none())
        .unwrap_or(0);
    let operations = &operations[start..];
    let mut base = None;
    for op in operations.iter().filter(|op| op.index == index) {
        if let Some((a, b, c, min, max)) = op.condition {
            let selector = match (sub, c) {
                (None, None) => Some((a, b)),
                (Some(sub), Some(bit)) if a == sub as u16 => Some((b, bit)),
                _ => None,
            };
            if let Some((varp, bit)) = selector {
                let value = if bit == u16::MAX {
                    variables.player(varp)
                } else {
                    let def = source.varbit(bit as i32)?;
                    def.get(variables.player(def.base))
                };
                if value >= min && value <= max {
                    return Ok((!op.text.is_empty()).then_some(op.text.as_str()));
                }
            }
        } else if op.sub == sub {
            base = (!op.text.is_empty()).then_some(op.text.as_str());
        }
    }
    Ok(base)
}
pub fn children(
    operations: &[Operation],
    index: u8,
    variables: &Variables,
    source: &mut impl Source,
) -> Result<Vec<(u8, String)>> {
    let start = operations
        .iter()
        .rposition(|op| op.index == index && op.sub.is_none() && op.condition.is_none())
        .unwrap_or(0);
    let operations = &operations[start..];
    let mut indices = operations
        .iter()
        .filter(|op| op.index == index)
        .filter_map(|op| {
            op.sub.or_else(|| {
                op.condition
                    .and_then(|(sub, _, bit, _, _)| bit.and_then(|_| u8::try_from(sub).ok()))
            })
        })
        .collect::<Vec<_>>();
    indices.sort_unstable();
    indices.dedup();
    let mut out = Vec::new();
    for sub in indices.into_iter().rev() {
        if let Some(text) = label(operations, index, Some(sub), variables, source)? {
            out.push((sub, text.to_owned()));
        }
    }
    Ok(out)
}
