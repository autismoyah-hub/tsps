//! Original yu.ey/ej widget lines, including revision-239 thick diagonal coverage.
use crate::raster::Raster;
pub fn draw(r: &mut Raster, mut a: [i32; 2], mut b: [i32; 2], width: u8, color: u32) {
    let dx = b[0].wrapping_sub(a[0]);
    let dy = b[1].wrapping_sub(a[1]);
    let thick = width > 1;
    let low = if thick { width as i32 / 2 } else { 0 };
    let high = if thick { width as i32 - low } else { 0 };
    if dy == 0 {
        let left = a[0].min(b[0]);
        r.fill_rect(
            left,
            a[1].wrapping_sub(low),
            a[0].max(b[0]).wrapping_sub(left).wrapping_add(1),
            low + high + 1,
            color,
            256,
        );
        return;
    }
    if dx == 0 {
        let top = a[1].min(b[1]);
        r.fill_rect(
            a[0].wrapping_sub(low),
            top,
            low + high + 1,
            a[1].max(b[1]).wrapping_sub(top).wrapping_add(1),
            color,
            256,
        );
        return;
    }
    if thick && dx.unsigned_abs() == dy.unsigned_abs() {
        let sx = dx.signum();
        let sy = dy.signum();
        // Skip steps whose complete stroke lies outside the horizontal clip.
        // Keep the original step index for the odd-step extra pixel.
        let clip = r.clip();
        let min_x = clip[0] as i64 - width as i64 - 1;
        let max_x = clip[2] as i64 + width as i64 + 1;
        let (start, end) = if sx > 0 {
            (min_x - a[0] as i64, max_x - a[0] as i64)
        } else {
            (a[0] as i64 - max_x, a[0] as i64 - min_x)
        };
        for step in start.max(0)..=end.min(dx.unsigned_abs() as i64) {
            let x = a[0].wrapping_add((step as i32).wrapping_mul(sx));
            let y = a[1].wrapping_add((step as i32).wrapping_mul(sy));
            for offset in -low..high {
                let px = x.wrapping_sub(sy * offset);
                let py = y.wrapping_add(sx * offset);
                r.fill_rect(px, py, 1, 1, color, 256);
                if step & 1 != 0 {
                    r.fill_rect(px.wrapping_add(1), py, 1, 1, color, 256);
                }
                r.fill_rect(px.wrapping_add(sx), py, 1, 1, color, 256);
            }
        }
        return;
    }
    let axis = if thick {
        usize::from(dx.unsigned_abs() < dy.unsigned_abs())
    } else {
        if dx.wrapping_add(dy) < 0 {
            std::mem::swap(&mut a, &mut b);
        }
        usize::from(b[0].wrapping_sub(a[0]) <= b[1].wrapping_sub(a[1]))
    };
    if thick && b[axis] < a[axis] {
        std::mem::swap(&mut a, &mut b);
    }
    let other = 1 - axis;
    let major = b[axis].wrapping_sub(a[axis]);
    let numerator = b[other].wrapping_sub(a[other]).wrapping_shl(16);
    let slope = if thick {
        if numerator >= 0 {
            numerator.wrapping_add(major / 2).wrapping_div(major)
        } else {
            numerator
                .wrapping_neg()
                .wrapping_add(major / 2)
                .wrapping_div(major)
                .wrapping_neg()
        }
    } else {
        (numerator as f64 / major as f64 + 0.5).floor() as i32
    };
    let clip = r.clip();
    let start = a[axis].max(clip[axis] as i32);
    let end = b[axis].min(clip[axis + 2] as i32 - 1);
    let mut minor = a[other]
        .wrapping_shl(16)
        .wrapping_add(32768)
        .wrapping_add(slope.wrapping_mul(start.wrapping_sub(a[axis])));
    for major in start..=end {
        let minor_pixel = (minor >> 16).wrapping_sub(low);
        if axis == 0 {
            r.fill_rect(major, minor_pixel, 1, low + high + 1, color, 256);
        } else {
            r.fill_rect(minor_pixel, major, low + high + 1, 1, color, 256);
        }
        minor = minor.wrapping_add(slope);
    }
}
