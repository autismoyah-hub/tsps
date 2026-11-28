//! Item inventory, equipment and head meshes from cached model data.
use crate::{
    disk_cache::DiskCache,
    item::{Definition, Definitions},
    model::Model,
    Error, Result,
};
use std::{collections::HashMap, sync::Arc};
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct AppearanceOverride {
    pub inventory_model: Option<u32>,
    pub wear_model: [Option<u32>; 2],
    pub head_model: [Option<u32>; 2],
    pub recolor: Option<Vec<u16>>,
    pub retexture: Option<Vec<u16>>,
}
#[derive(Default)]
pub struct Models {
    raw: HashMap<u32, Arc<Model>>,
}
impl Models {
    pub fn available(&self, cache: &mut DiskCache, id: u32) -> bool {
        self.raw.contains_key(&id) || cache.group(7, id, [0; 4]).is_ok()
    }
    pub fn raw(&mut self, cache: &mut DiskCache, id: u32) -> Result<Arc<Model>> {
        if let Some(m) = self.raw.get(&id) {
            return Ok(m.clone());
        }
        let m = Arc::new(Model::decode(&cache.file(7, id, 0, [0; 4])?)?);
        if self.raw.len() >= 256 {
            self.raw.clear();
        }
        self.raw.insert(id, m.clone());
        Ok(m)
    }
    fn colors(model: &mut Model, d: &Definition, appearance: Option<&AppearanceOverride>) {
        let recolor = appearance
            .and_then(|a| a.recolor.as_ref())
            .filter(|a| a.len() == d.recolor.len());
        for (i, &(from, to)) in d.recolor.iter().enumerate() {
            model.recolor(from, recolor.map_or(to, |a| a[i]));
        }
        let retexture = appearance
            .and_then(|a| a.retexture.as_ref())
            .filter(|a| a.len() == d.retexture.len());
        for (i, &(from, to)) in d.retexture.iter().enumerate() {
            model.retexture(from, retexture.map_or(to, |a| a[i]));
        }
    }
    pub fn inventory(
        &mut self,
        cache: &mut DiskCache,
        defs: &Definitions,
        id: u32,
        quantity: i32,
        appearance: Option<&AppearanceOverride>,
    ) -> Result<Model> {
        let d = defs
            .items
            .get(&id)
            .ok_or(Error::Invalid("item definition"))?;
        let d = defs
            .items
            .get(&d.count_variant(quantity))
            .ok_or(Error::Invalid("item count variant"))?;
        let id = appearance
            .and_then(|a| a.inventory_model)
            .unwrap_or(d.model as u32);
        let mut m = (*self.raw(cache, id)?).clone();
        m.scale(d.scale);
        Self::colors(&mut m, d, appearance);
        Ok(m)
    }
    pub fn wearable(
        &mut self,
        cache: &mut DiskCache,
        d: &Definition,
        gender: usize,
        head: bool,
        appearance: Option<&AppearanceOverride>,
    ) -> Result<Option<Model>> {
        if gender > 1 {
            return Err(Error::Invalid("item body type"));
        }
        let mut ids = if head {
            vec![d.head_models[gender][0], d.head_models[gender][1]]
        } else {
            d.wear_models[gender].to_vec()
        };
        if let Some(id) = appearance.and_then(|a| {
            if head {
                a.head_model[gender]
            } else {
                a.wear_model[gender]
            }
        }) {
            ids[0] = id as i32;
        }
        if ids[0] == -1 {
            return Ok(None);
        }
        // The original only loads the third worn model when a second model exists.
        if ids[1] == -1 {
            ids.truncate(1);
        }
        let mut parts = Vec::new();
        for id in ids.into_iter().filter(|&i| i != -1) {
            parts.push((*self.raw(cache, id as u32)?).clone());
        }
        let mut m = if parts.len() == 1 {
            parts.remove(0)
        } else {
            Model::combine(&parts)?
        };
        if !head {
            m.translate([0, d.wear_offset[gender], 0]);
        }
        Self::colors(&mut m, d, appearance);
        Ok(Some(m))
    }
}

fn icon_trig(angle: i32) -> (i32, i32) {
    let value = angle as f64 * 0.0030679615;
    (
        (value.sin() * 65536.0) as i32,
        (value.cos() * 65536.0) as i32,
    )
}
fn icon_translation(model: &Model, d: &Definition, zoom: i32, sine: i32, cosine: i32) -> [i32; 2] {
    let height = model
        .vertices
        .iter()
        .map(|v| -(v[1] as i32))
        .max()
        .unwrap_or(0)
        .max(0);
    [
        d.offset[1]
            .wrapping_add(sine.wrapping_mul(zoom) >> 16)
            .wrapping_add(height / 2),
        d.offset[1].wrapping_add(cosine.wrapping_mul(zoom) >> 16),
    ]
}
/// GameModel.at/bj bucket radius and camera-space origin, before per-vertex subtraction.
pub fn icon_depth_bounds(model: &Model, d: &Definition, zoom: i32) -> [i32; 2] {
    let radius = model
        .vertices
        .iter()
        .map(|&[x, y, z]| x * x + z * z + y * y)
        .fold(0.0f32, f32::max);
    let (sine, cosine) = icon_trig(d.angles[0]);
    let [y, z] = icon_translation(model, d, zoom, sine, cosine);
    [
        (radius as f64).sqrt().ceil() as i32,
        y.wrapping_mul(sine).wrapping_add(z.wrapping_mul(cosine)) >> 16,
    ]
}
/// Camera-space vertices used by the inventory icon projection (GameModel.bj).
pub fn icon_vertices(model: &Model, d: &Definition, zoom: i32) -> Result<Vec<[f32; 3]>> {
    if d.angles.iter().any(|&a| !(0..2048).contains(&a)) {
        return Err(Error::Invalid("item icon angle"));
    }
    let (sx, cx) = icon_trig(d.angles[0]);
    let (sy, cy) = icon_trig(d.angles[1]);
    let (sz, cz) = icon_trig(d.angles[2]);
    let [translate_y, translate_z] = icon_translation(model, d, zoom, sx, cx);
    Ok(model
        .vertices
        .iter()
        .map(|v| {
            let (mut x, mut y, mut z) = (v[0] as i32, v[1] as i32, v[2] as i32);
            if d.angles[2] != 0 {
                let nx = y.wrapping_mul(sz).wrapping_add(x.wrapping_mul(cz)) >> 16;
                y = y.wrapping_mul(cz).wrapping_sub(x.wrapping_mul(sz)) >> 16;
                x = nx;
            }
            if d.angles[1] != 0 {
                let nx = z.wrapping_mul(sy).wrapping_add(x.wrapping_mul(cy)) >> 16;
                z = z.wrapping_mul(cy).wrapping_sub(x.wrapping_mul(sy)) >> 16;
                x = nx;
            }
            x = x.wrapping_add(d.offset[0]);
            y = y.wrapping_add(translate_y);
            z = z.wrapping_add(translate_z);
            let ny = y.wrapping_mul(cx).wrapping_sub(z.wrapping_mul(sx)) >> 16;
            z = y.wrapping_mul(sx).wrapping_add(z.wrapping_mul(cx)) >> 16;
            [x as f32, ny as f32, z as f32]
        })
        .collect())
}
