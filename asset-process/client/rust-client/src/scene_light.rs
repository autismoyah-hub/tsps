//! Bake static model-data lighting before bridge lowering and runtime replacements.
use crate::{model::Model, Result};
use std::{collections::HashMap, sync::Arc};
pub(crate) struct Baked {
    pub model: Arc<Model>,
    pub colors: Vec<[i32; 3]>,
}
pub(crate) fn bake_static(
    scene: &crate::scene::Scene,
    cache: &mut crate::disk_cache::DiskCache,
    definitions: &HashMap<u32, crate::object::Definition>,
    models: &mut crate::object_model::Models,
) -> Result<HashMap<(usize, usize), Baked>> {
    use crate::scene_normal::{Kind, Object};
    let renderer=scene.definition_renderer();
    let mut prepared = Vec::new();
    for (index, object) in scene
        .initial_objects()
        .iter()
        .enumerate()
        .filter(|(_, o)| !o.runtime && !o.rejected && (o.shape <= 3 || o.shape >= 9))
    {
        let definition = renderer.get(object.plane as usize,[object.x as i32,object.y as i32],&definitions[&object.id]);
        if !definition.non_flat_shading
            || definition.animation.is_some()
            || definition.transforms.is_some()
        {
            continue;
        }
        let [sx, sz] = definition.dimensions(object.rotation).map(usize::from);
        let [plane, x, z] = [object.plane as usize, object.x as usize, object.y as usize];
        let [x1, x2] = if x + sx <= scene.width {
            [x + sx / 2, x + (sx + 1) / 2]
        } else {
            [x, x + 1]
        };
        let [z1, z2] = if z + sz <= scene.height {
            [z + sz / 2, z + (sz + 1) / 2]
        } else {
            [z, z + 1]
        };
        let base = (scene.height(plane, x1, z1).unwrap()
            + scene.height(plane, x2, z1).unwrap()
            + scene.height(plane, x1, z2).unwrap()
            + scene.height(plane, x2, z2).unwrap())
            >> 2;
        let size = if matches!(object.shape, 10 | 11) {
            [sx, sz]
        } else {
            [1, 1]
        };
        for (part, specification) in
            crate::location_animation::parts(object.shape, object.rotation, None, 0)?
                .into_iter()
                .enumerate()
        {
            let Some(specification) = specification else {
                continue;
            };
            let Some(model) = models.object(
                cache,
                &definition,
                specification.shape,
                specification.rotation,
            )?
            else {
                continue;
            };
            let model = crate::model_contour::contour(
                &model,
                [
                    x as i32 * 128 + sx as i32 * 64,
                    base,
                    z as i32 * 128 + sz as i32 * 64,
                ],
                definition.contour,
                |x, z| scene.height(plane, x, z),
            )?;
            let kind = if object.shape <= 3 {
                Kind::Wall(part as u8)
            } else if object.shape == 22 {
                Kind::Ground
            } else {
                Kind::Game
            };
            prepared.push((
                (index, part),
                [plane, x, z],
                size,
                kind,
                model,
                64 + definition.ambient as i32,
                768 + definition.contrast,
            ));
        }
    }
    let objects: Vec<_> = prepared
        .iter()
        .map(|(_, tile, size, kind, model, ambient, contrast)| Object {
            tile: *tile,
            size: *size,
            kind: *kind,
            model,
            ambient: *ambient,
            contrast: *contrast,
        })
        .collect();
    let colors =
        crate::scene_normal::lighting(&objects, [scene.width, scene.height], |[plane, x, z]| {
            (scene.height(plane, x, z).unwrap()
                + scene.height(plane, x + 1, z).unwrap()
                + scene.height(plane, x, z + 1).unwrap()
                + scene.height(plane, x + 1, z + 1).unwrap())
                >> 2
        })?;
    Ok(prepared
        .into_iter()
        .zip(colors)
        .map(|((key, _, _, _, model, _, _), colors)| {
            (
                key,
                Baked {
                    model: Arc::new(model),
                    colors,
                },
            )
        })
        .collect())
}
pub struct Ground<'a> {
    pub tile: [usize; 3],
    pub elevation: i32,
    pub model: &'a Model,
    pub ambient: i32,
    pub contrast: i32,
}
pub fn ground_lighting(objects: &[Ground<'_>]) -> Result<Vec<Vec<[i32; 3]>>> {
    let elevations: HashMap<_, _> = objects.iter().map(|o| (o.tile, o.elevation)).collect();
    let dimensions = objects.iter().fold([0, 0], |size, o| {
        [size[0].max(o.tile[1] + 1), size[1].max(o.tile[2] + 1)]
    });
    let placements: Vec<_> = objects
        .iter()
        .map(|o| crate::scene_normal::Object {
            tile: o.tile,
            size: [1, 1],
            kind: crate::scene_normal::Kind::Ground,
            model: o.model,
            ambient: o.ambient,
            contrast: o.contrast,
        })
        .collect();
    crate::scene_normal::lighting(&placements, dimensions, |tile| elevations[&tile])
}
