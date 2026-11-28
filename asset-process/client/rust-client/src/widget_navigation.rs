use crate::{Error, Result};
#[derive(Clone, Copy)]
pub struct Selected {
    pub index: i32,
    pub parent: i32,
    pub kind: u8,
}
/// Native child-array lookup for ex.aq 217..222. Array slots may contain gaps.
pub fn target(
    opcode: u16,
    selected: Option<Selected>,
    ordinal: i32,
    children: &[(i32, i32)],
) -> Result<Option<i32>> {
    if opcode == 222 {
        return selected
            .map(|_| None)
            .ok_or(Error::Invalid("active widget"));
    }
    if opcode == 217 && selected.is_none_or(|node| node.kind != 0) {
        return Err(Error::Invalid("widget navigation container"));
    }
    let Some(node) = selected else {
        return Ok(None);
    };
    if matches!(opcode, 220 | 221) && node.kind != 0 {
        return Ok(None);
    }
    let parent = if matches!(opcode, 218 | 219) {
        node.parent
    } else {
        node.index
    };
    let candidates = children
        .iter()
        .filter(|(index, p)| *p == parent && (opcode != 217 || *index > node.index))
        .map(|(index, _)| *index);
    Ok(match opcode {
        217 => {
            let mut ordered = candidates.collect::<Vec<_>>();
            ordered.sort_unstable();
            usize::try_from(ordinal)
                .ok()
                .and_then(|n| ordered.get(n).copied())
        }
        218 | 220 => candidates.min(),
        219 | 221 => candidates.max(),
        _ => return Err(Error::Invalid("widget navigation opcode")),
    })
}

pub fn relative(
    opcode: u16,
    selected: Option<Selected>,
    allocated: bool,
    children: &[(i32, i32)],
) -> Result<(i32, Option<i32>)> {
    let Some(node) = selected else {
        return Ok((if opcode == 209 { -1 } else { 0 }, None));
    };
    if opcode == 209 {
        return Ok((node.parent, None));
    }
    if opcode == 208 {
        return Ok((
            if node.kind == 0 {
                children
                    .iter()
                    .filter(|(_, parent)| *parent == node.index)
                    .count() as i32
            } else {
                0
            },
            None,
        ));
    }
    let target = match opcode {
        204 => {
            if node.parent == -1 || children.iter().any(|(index, _)| *index == node.parent) {
                Some(node.parent)
            } else {
                None
            }
        }
        205 => Some(-1),
        206 | 207 if node.index >= 0 => {
            if !allocated {
                return Err(Error::Invalid("widget navigation child array"));
            }
            let candidates = children
                .iter()
                .filter(|(index, parent)| {
                    *parent == node.parent
                        && if opcode == 206 {
                            *index > node.index
                        } else {
                            *index < node.index
                        }
                })
                .map(|(index, _)| *index);
            if opcode == 206 {
                candidates.min()
            } else {
                candidates.max()
            }
        }
        206 | 207 => None,
        _ => return Err(Error::Invalid("widget relative opcode")),
    };
    Ok((target.is_some() as i32, target))
}
