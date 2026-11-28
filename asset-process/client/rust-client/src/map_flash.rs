//! World-map flashing sprite and two pulse circles (GameWorldMapRegion.an).
use crate::{raster::Raster, sprite_pixels::Pixels, Result};
fn span(raster: &mut Raster, x: i32, y: i32, width: i32, color: u32, alpha: u16) {
    let clip = raster.clip();
    if y < clip[1] as i32 || y >= clip[3] as i32 {
        return;
    }
    let left = x.max(clip[0] as i32);
    let right = (x + width).min(clip[2] as i32);
    let row = y as usize * raster.width();
    let a = alpha as u32;
    let b = 256 - a;
    for x in left..right {
        let target = &mut raster.pixels_mut()[row + x as usize];
        if alpha == 256 {
            *target = color | 0xff000000;
        } else {
            let rb = ((color & 0xff00ff) * a + (*target & 0xff00ff) * b) & 0xff00ff00;
            let g = ((color & 0xff00) * a + (*target & 0xff00) * b) & 0xff0000;
            *target = (rb | g) >> 8;
        }
    }
}
fn circle(raster: &mut Raster, cx: i32, mut cy: i32, radius: i32, color: u32, alpha: u16) {
    let clip = raster.clip();
    let mut row = (cy - radius).max(clip[1] as i32);
    let end = (cy + radius + 1).min(clip[3] as i32);
    let square = radius * radius;
    let mut dx = 0;
    let mut dy = cy - row;
    let mut a = dy * dy;
    let mut b = a - dy;
    cy = cy.min(end);
    while row < cy {
        while b <= square || a <= square {
            a += dx + dx;
            b += dx + dx + 1;
            dx += 1;
        }
        span(raster, cx - dx + 1, row, 2 * dx - 1, color, alpha);
        row += 1;
        a -= dy + dy - 1;
        dy -= 1;
        b -= dy + dy;
    }
    dx = radius;
    dy = -dy;
    b = dy * dy + square;
    a = b - dx;
    b -= dy;
    while row < end {
        while b > square && a > square {
            b -= dx + dx - 1;
            dx -= 1;
            a -= dx + dx;
        }
        span(raster, cx - dx, row, 2 * dx + 1, color, alpha);
        row += 1;
        b += dy + dy;
        a += dy + dy + 1;
        dy += 1;
    }
}
pub fn draw(
    image: &Pixels,
    raster: &mut Raster,
    center: [i32; 2],
    cycle: i32,
    period: i32,
) -> Result<()> {
    image.draw(
        raster,
        center[0] - image.size[0] as i32 / 2,
        center[1] - image.size[1] as i32 / 2,
        256,
    )?;
    if period > 0 && cycle % period < period / 2 {
        circle(raster, center[0], center[1], 15, 0xffff00, 128);
        circle(raster, center[0], center[1], 7, 0xffffff, 256);
    }
    Ok(())
}
