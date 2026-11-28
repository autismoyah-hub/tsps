//! Original GameModel.go and fy.cc/cf alpha dispatch.
pub fn normalize(encoded: u8) -> u8 {
    if encoded == 255 {
        253
    } else {
        encoded
    }
}
/// Foreground and background products are truncated separately.
pub fn blend(foreground: u32, background: u32, alpha: u8) -> u32 {
    let a = alpha as u32;
    let b = 256 - a;
    let channel = |shift: u32| {
        (((foreground >> shift) & 255u32) * b >> 8) + (((background >> shift) & 255u32) * a >> 8)
    };
    0xff000000 | channel(16) << 16 | channel(8) << 8 | channel(0)
}
/// Flat alpha254 shifts each scanline one pixel left, without writing depth.
pub fn shift_span(
    pixels: &mut [u32],
    depths: &[f32],
    row: usize,
    width: usize,
    start: i32,
    end: i32,
    initial: f32,
    step: f32,
    mut hit: impl FnMut(usize),
) {
    let start = start.max(0) as usize;
    let end = end.min(width as i32).max(0) as usize;
    if start >= end || start == 0 || end > width.saturating_sub(1) {
        return;
    }
    let mut z = initial + step * start as f32;
    for x in start..end {
        let target = row + x - 1;
        if (depths[target] - z).is_sign_negative() {
            pixels[target] = pixels[row + x];
            hit(target);
        }
        z += step;
    }
}
