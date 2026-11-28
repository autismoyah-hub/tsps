//! Interface model positioning from ga and GameModel.bj.
use crate::{model::Model, Error, Result};
pub fn vertices(
    model: &Model,
    angles: [i32; 3],
    offset: [i32; 2],
    zoom: i32,
    center_height: bool,
) -> Result<Vec<[f32; 3]>> {
    if angles.iter().any(|&a| !(0..2048).contains(&a)) {
        return Err(Error::Invalid("interface model angle"));
    }
    let trig = |a: i32| {
        let v = a as f64 * 0.0030679615;
        ((v.sin() * 65536.0) as i32, (v.cos() * 65536.0) as i32)
    };
    let (sx, cx) = trig(angles[0]);
    let (sy, cy) = trig(angles[1]);
    let (sz, cz) = trig(angles[2]);
    let [ty, tz] = translation(model, offset, zoom, center_height, sx, cx);
    Ok(model
        .vertices
        .iter()
        .map(|v| {
            let (mut x, mut y, mut z) = (v[0] as i32, v[1] as i32, v[2] as i32);
            if angles[2] != 0 {
                let nx = y.wrapping_mul(sz).wrapping_add(x.wrapping_mul(cz)) >> 16;
                y = y.wrapping_mul(cz).wrapping_sub(x.wrapping_mul(sz)) >> 16;
                x = nx;
            }
            if angles[1] != 0 {
                let nx = z.wrapping_mul(sy).wrapping_add(x.wrapping_mul(cy)) >> 16;
                z = z.wrapping_mul(cy).wrapping_sub(x.wrapping_mul(sy)) >> 16;
                x = nx;
            }
            x = x.wrapping_add(offset[0]);
            y = y.wrapping_add(ty);
            z = z.wrapping_add(tz);
            let ny = y.wrapping_mul(cx).wrapping_sub(z.wrapping_mul(sx)) >> 16;
            z = y.wrapping_mul(sx).wrapping_add(z.wrapping_mul(cx)) >> 16;
            [x as f32, ny as f32, z as f32]
        })
        .collect())
}

fn translation(
    model: &Model,
    offset: [i32; 2],
    zoom: i32,
    center_height: bool,
    sx: i32,
    cx: i32,
) -> [i32; 2] {
    let height = if center_height {
        model
            .vertices
            .iter()
            .map(|v| -(v[1] as i32))
            .max()
            .unwrap_or(0)
            .max(0)
            / 2
    } else {
        0
    };
    let ty = (sx.wrapping_mul(zoom) >> 16)
        .wrapping_add(offset[1])
        .wrapping_add(height);
    let tz = (cx.wrapping_mul(zoom) >> 16).wrapping_add(offset[1]);
    [ty, tz]
}
/// Camera-space draw origin used before integer depth-bucket averaging.
pub fn depth_origin(
    model: &Model,
    pitch: i32,
    offset: [i32; 2],
    zoom: i32,
    center_height: bool,
) -> i32 {
    let angle = pitch as f64 * 0.0030679615;
    let (sx, cx) = ((angle.sin() * 65536.) as i32, (angle.cos() * 65536.) as i32);
    let [ty, tz] = translation(model, offset, zoom, center_height, sx, cx);
    ty.wrapping_mul(sx).wrapping_add(tz.wrapping_mul(cx)) >> 16
}
