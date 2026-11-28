//! Static terrain illumination and location shadows (rl4.va / GameIgnore.ab).
use crate::{Error, Result};

/// Samples are ordered centre, west, east, south, north.
pub fn brightness(heights: [i32; 5], shadows: [i8; 5]) -> Result<i32> {
    let dx = heights[2].wrapping_sub(heights[1]);
    let dz = heights[4].wrapping_sub(heights[3]);
    let squared = dx
        .wrapping_mul(dx)
        .wrapping_add(dz.wrapping_mul(dz))
        .wrapping_add(65536);
    let length = (f64::from(squared).sqrt()) as i32;
    if length == 0 {
        return Err(Error::Invalid("terrain normal length"));
    }
    let nx = dx.wrapping_shl(8) / length;
    let ny = 65536 / length;
    let nz = dz.wrapping_shl(8) / length;
    let base = 96
        + nx.wrapping_mul(-50)
            .wrapping_add(ny.wrapping_mul(-10))
            .wrapping_add(nz.wrapping_mul(-50))
            / 213;
    let [c, w, e, s, n] = shadows.map(i32::from);
    Ok(base - ((c >> 1) + (w >> 2) + (e >> 3) + (s >> 2) + (n >> 3)))
}

pub struct Caster {
    pub shape: u8,
    pub rotation: u8,
    /// Already rotated footprint.
    pub size: [u8; 2],
    pub clipped: bool,
    pub accepted: bool,
    /// None is the Java dynamic/model-data default of 15.
    pub radius: Option<i32>,
}
pub fn stamp(grid: &mut [i8], stride: usize, x: usize, y: usize, caster: Caster) {
    if !caster.clipped || stride == 0 {
        return;
    }
    let mut set = |dx: usize, dy: usize, value: i8, maximum: bool| {
        if y + dy < stride {
            if let Some(old) = grid.get_mut((x + dx) * stride + y + dy) {
                *old = if maximum { (*old).max(value) } else { value };
            }
        }
    };
    let rotation = (caster.rotation & 3) as usize;
    match caster.shape {
        0 => {
            let points = [
                [(0, 0), (0, 1)],
                [(0, 1), (1, 1)],
                [(1, 0), (1, 1)],
                [(0, 0), (1, 0)],
            ];
            for (dx, dy) in points[rotation] {
                set(dx, dy, 50, false);
            }
        }
        1 | 3 => {
            let (dx, dy) = [(0, 1), (1, 1), (1, 0), (0, 0)][rotation];
            set(dx, dy, 50, false);
        }
        10 | 11 if caster.accepted => {
            let strength = caster.radius.map_or(15, |radius| (radius / 4).min(30)) as i8;
            for dx in 0..=usize::from(caster.size[0]) {
                for dy in 0..=usize::from(caster.size[1]) {
                    set(dx, dy, strength, true);
                }
            }
        }
        _ => {}
    }
}

pub fn model_radius(vertices: &[[f32; 3]]) -> i32 {
    let squared = vertices
        .iter()
        .fold(0.0_f32, |value, [x, _, z]| value.max(x * x + z * z));
    f64::from(squared).sqrt().ceil() as i32
}

/// Empty map rectangles use inclusive corners, but exclude the outer scene border.
/// Edge copies occur west, east, south, north in that order (rl4.fa).
pub fn fill_void(heights: &mut [i32], shadows: &mut [i8], dimensions: [usize; 2], rect: [i32; 4]) {
    let [width, height] = dimensions;
    let [x, y, w, h] = rect;
    let stride = height + 1;
    for py in y.max(0)..=y.saturating_add(h).min(height as i32 - 1) {
        for px in x.max(0)..=x.saturating_add(w).min(width as i32 - 1) {
            let index = px as usize * stride + py as usize;
            shadows[index] = 127;
            if px == x && px > 0 {
                heights[index] = heights[index - stride];
            }
            if px == x + w && px < width as i32 - 1 {
                heights[index] = heights[index + stride];
            }
            if py == y && py > 0 {
                heights[index] = heights[index - 1];
            }
            if py == y + h && py < height as i32 - 1 {
                heights[index] = heights[index + 1];
            }
        }
    }
}
