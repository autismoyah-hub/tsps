//! Original region icon placement, sprite alignment, and hit bounds.
use crate::{raster::Raster, sprite_pixels::Pixels, Error, Result};
pub fn position(tile: [i32; 2], origin: [i32; 2], region_size: i32, geography: bool) -> [i32; 2] {
    let scale = region_size as f32 / 64.0;
    if geography {
        let half = scale / 2.0;
        [
            (origin[0] as f32 + scale * tile[0] as f32 - half) as i32,
            (origin[1].wrapping_add(region_size) as f32 - scale * tile[1] as f32 - half) as i32,
        ]
    } else {
        [
            (origin[0] as f32 + scale * (tile[0] % 64) as f32) as i32,
            (origin[1] as f32 + scale * (63 - tile[1] % 64) as f32) as i32,
        ]
    }
}
fn alignments(alignment: [Option<u8>; 2]) -> Result<[u8; 2]> {
    match alignment {
        [Some(x @ 0..=2), Some(y @ 0..=2)] => Ok([x, y]),
        _ => Err(Error::Invalid("map icon alignment")),
    }
}
pub fn draw(
    image: &Pixels,
    raster: &mut Raster,
    center: [i32; 2],
    alignment: [Option<u8>; 2],
) -> Result<()> {
    let [ax, ay] = alignments(alignment)?;
    let [w, h] = image.size.map(|v| v as i32);
    let dx = match ax {
        0 => 0,
        1 => -w / 2,
        _ => -w,
    };
    let dy = match ay {
        0 => 0,
        1 => -h / 2,
        _ => -h,
    };
    image.draw(
        raster,
        center[0].wrapping_add(dx),
        center[1].wrapping_add(dy),
        256,
    )
}
/// Java hit bounds intentionally differ from drawing alignment at the edges.
pub fn sprite_hit(
    size: [i32; 2],
    center: [i32; 2],
    point: [i32; 2],
    alignment: [Option<u8>; 2],
) -> Result<bool> {
    let [ax, ay] = alignments(alignment)?;
    let [w, h] = size;
    let [cx, cy] = center;
    let [x, y] = point;
    let horizontal = match ax {
        0 => x > cx.wrapping_sub(w) && x <= cx,
        1 => x >= cx.wrapping_sub(w / 2) && x <= cx.wrapping_add(w / 2),
        _ => x >= cx && x < cx.wrapping_add(w),
    };
    let vertical = match ay {
        0 => y > cy.wrapping_sub(h) && y <= cy,
        1 => y >= cy.wrapping_sub(h / 2) && y <= cy.wrapping_add(h / 2),
        _ => y >= cy && y < cy.wrapping_add(h),
    };
    Ok(horizontal && vertical)
}
