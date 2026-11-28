//! Java terrain deformation for unlit model data and already-lit rendered models.
use crate::{model::Model, Error, Result};
pub fn contour(
    model: &Model,
    origin: [i32; 3],
    limit: i32,
    height: impl Fn(usize, usize) -> Option<i32>,
) -> Result<Model> {
    let mut out = model.clone();
    deform(&mut out.vertices, origin, limit, false, height)?;
    Ok(out)
}
/// GameModel.ap uses its cylinder radius for the whole-model bounds check.
/// Geometry changes after lighting, so this returns only deformed vertices.
pub fn rendered(
    model: &Model,
    origin: [i32; 3],
    limit: i32,
    height: impl Fn(usize, usize) -> Option<i32>,
) -> Result<Vec<[f32; 3]>> {
    let mut vertices = model.vertices.clone();
    deform(&mut vertices, origin, limit, true, height)?;
    Ok(vertices)
}
fn deform(
    vertices: &mut [[f32; 3]],
    origin: [i32; 3],
    limit: i32,
    radial: bool,
    height: impl Fn(usize, usize) -> Option<i32>,
) -> Result<()> {
    if limit < 0 || vertices.is_empty() {
        return Ok(());
    }
    let mut minimum = [i32::MAX; 2];
    let mut maximum = [i32::MIN; 2];
    let mut model_height = 0;
    for vertex in vertices.iter() {
        for (axis, coordinate) in [0, 2].into_iter().enumerate() {
            minimum[axis] = minimum[axis].min(vertex[coordinate] as i32);
            maximum[axis] = maximum[axis].max(vertex[coordinate] as i32);
        }
        model_height = model_height.max((vertex[1] as i32).wrapping_neg());
    }
    if radial {
        let radius = crate::terrain_light::model_radius(vertices);
        minimum = [-radius; 2];
        maximum = [radius; 2];
    }
    let low = [minimum[0] + origin[0], minimum[1] + origin[2]];
    let high = [maximum[0] + origin[0], maximum[1] + origin[2]];
    if low.iter().any(|v| *v < 0)
        || height(
            ((high[0] + 128) >> 7) as usize,
            ((high[1] + 128) >> 7) as usize,
        )
        .is_none()
    {
        return Ok(());
    }
    let min = low.map(|v| (v >> 7) as usize);
    let max = high.map(|v| ((v + 127) >> 7) as usize);
    if [
        (min[0], min[1]),
        (max[0], min[1]),
        (min[0], max[1]),
        (max[0], max[1]),
    ]
    .into_iter()
    .all(|(x, z)| height(x, z) == Some(origin[1]))
    {
        return Ok(());
    }
    if limit > 0 && model_height == 0 {
        return Err(Error::Invalid("model contour height"));
    }
    for vertex in vertices.iter_mut() {
        let ratio = if limit == 0 {
            0
        } else {
            (vertex[1] as i32).wrapping_neg().wrapping_shl(16) / model_height
        };
        if limit != 0 && ratio >= limit {
            continue;
        }
        let [x, z] = [vertex[0] as i32 + origin[0], vertex[2] as i32 + origin[2]];
        let [tx, tz] = [x, z].map(|v| (v >> 7) as usize);
        let heights = [(tx, tz), (tx + 1, tz), (tx + 1, tz + 1), (tx, tz + 1)]
            .map(|(x, z)| height(x, z).unwrap());
        let delta = crate::tile_mesh::bilinear_height(heights, [x, z]).wrapping_sub(origin[1]);
        let delta = if limit == 0 {
            delta
        } else {
            delta.wrapping_mul(limit.wrapping_sub(ratio)) / limit
        };
        vertex[1] = (vertex[1] as i32).wrapping_add(delta) as f32;
    }
    Ok(())
}
