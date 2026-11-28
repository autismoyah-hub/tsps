//! NPC body and chathead assembly. Lighting precedes animation and body resizing.
use crate::{
    disk_cache::DiskCache, item_model::Models, model::Model, npc::Definition, Error, Result,
};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Debug, Default)]
pub struct Overrides {
    pub models: Option<Vec<u32>>,
    pub recolor: Option<Vec<u16>>,
    pub retexture: Option<Vec<u16>>,
    pub use_player: bool,
}
pub fn resolve<'a>(
    defs: &'a HashMap<u32, Definition>,
    mut id: u32,
    mut variable: impl FnMut(bool, u16) -> Result<i32>,
) -> Result<Option<&'a Definition>> {
    let mut seen = HashSet::new();
    loop {
        if !seen.insert(id) {
            return Err(Error::Invalid("NPC transform cycle"));
        }
        let d = defs
            .get(&id)
            .ok_or(Error::Invalid("NPC definition missing"))?;
        if d.transforms.is_none() {
            return Ok(Some(d));
        }
        let bit = if let Some(n) = d.transform_varbit {
            variable(true, n)?
        } else {
            -1
        };
        let varp = if d.transform_varbit.is_none() {
            if let Some(n) = d.transform_varp {
                variable(false, n)?
            } else {
                -1
            }
        } else {
            -1
        };
        let Some(next) = d.transformed_id(bit, varp) else {
            return Ok(None);
        };
        id = next;
    }
}
pub fn mesh(
    cache: &mut DiskCache,
    models: &mut Models,
    d: &Definition,
    head: bool,
    overrides: Option<&Overrides>,
) -> Result<Option<Model>> {
    if overrides.is_some_and(|v| v.use_player) {
        return Err(Error::Unsupported("NPC player appearance override"));
    }
    let ids = overrides
        .and_then(|v| v.models.as_ref())
        .unwrap_or(if head { &d.head_models } else { &d.models });
    if ids.is_empty() {
        return Ok(None);
    }
    // GameNPCComposition.be asks for every part before constructing any model.
    let mut available = true;
    for &id in ids {
        if id != u32::MAX {
            available &= models.available(cache, id);
        }
    }
    if !available {
        return Ok(None);
    }
    let mut parts = Vec::new();
    for &id in ids {
        if id != u32::MAX {
            parts.push((*models.raw(cache, id)?).clone());
        }
    }
    if parts.is_empty() {
        return Ok(None);
    }
    let mut model = if parts.len() == 1 {
        parts.remove(0)
    } else {
        Model::combine(&parts)?
    };
    let recolor = overrides.and_then(|v| v.recolor.as_ref());
    let retexture = overrides.and_then(|v| v.retexture.as_ref());
    if recolor.is_some_and(|v| v.len() < d.recolor.len())
        || retexture.is_some_and(|v| v.len() < d.retexture.len())
    {
        return Err(Error::Invalid("NPC color override length"));
    }
    for (i, &(from, to)) in d.recolor.iter().enumerate() {
        model.recolor(from, recolor.map_or(to, |v| v[i]));
    }
    for (i, &(from, to)) in d.retexture.iter().enumerate() {
        model.retexture(from, retexture.map_or(to, |v| v[i]));
    }
    Ok(Some(model))
}
pub fn resize(model: &mut Model, d: &Definition) {
    model.scale([d.scale[0] as i32, d.scale[1] as i32, d.scale[0] as i32]);
}
