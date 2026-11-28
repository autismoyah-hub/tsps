//! Original fy.ck eight-pixel textured scanline.
fn project(mapping: [i32; 3]) -> [i32; 2] {
    let divisor = mapping[2] >> 14;
    if divisor == 0 {
        [0, 0]
    } else {
        [
            mapping[0].wrapping_div(divisor).clamp(0, 16256),
            mapping[1].wrapping_div(divisor),
        ]
    }
}
fn modulate(color: u32, light: i32) -> u32 {
    let rb = (color & 0xff00ff).wrapping_mul(light as u32) & 0xff00ff00;
    let g = (color & 0xff00).wrapping_mul(light as u32) & 0xff0000;
    ((rb.wrapping_add(g) as i32) >> 8) as u32
}
#[derive(Clone, Copy)]
pub struct Span {
    pub row: usize,
    pub width: usize,
    pub center: i32,
    pub start: i32,
    pub end: i32,
    pub shade: i32,
    pub shade_step: i32,
    pub initial: f32,
    pub depth_step: f32,
    pub mapping: [i32; 3],
    pub increments: [i32; 3],
    pub alpha: u8,
    pub opaque: bool,
}
impl Span {
    pub fn draw_masked(
        self,
        pixels: &mut [u32],
        depths: &mut [f32],
        texture: &[u32],
        write_depth: bool,
        color_mask: u32,
        write_color: bool,
        hit: impl FnMut(usize),
    ) {
        if write_color {
            self.draw_impl::<true, true>(pixels, depths, texture, write_depth, color_mask, hit);
        } else {
            self.draw_impl::<true, false>(pixels, depths, texture, write_depth, color_mask, hit);
        }
    }

    pub fn draw(
        self,
        pixels: &mut [u32],
        depths: &mut [f32],
        texture: &[u32],
        write_depth: bool,
        color_mask: u32,
        hit: impl FnMut(usize),
    ) {
        self.draw_impl::<true, true>(pixels, depths, texture, write_depth, color_mask, hit);
    }
    pub fn draw_legacy(self, pixels: &mut [u32], texture: &[u32], color_mask: u32) {
        self.draw_impl::<false, true>(pixels, &mut [], texture, false, color_mask, |_| {});
    }

    fn draw_impl<const DEPTH: bool, const COLOR: bool>(
        self,
        pixels: &mut [u32],
        depths: &mut [f32],
        texture: &[u32],
        write_depth: bool,
        color_mask: u32,
        mut hit: impl FnMut(usize),
    ) {
        let Self {
            row,
            width,
            center,
            start,
            end,
            mut shade,
            shade_step,
            initial,
            depth_step,
            mut mapping,
            increments,
            alpha,
            opaque,
        } = self;
        let start = start.max(0);
        let end = end.min(width as i32);
        if start >= end {
            return;
        }
        shade = shade.wrapping_add(shade_step.wrapping_mul(start));
        let mut z = initial + depth_step * start as f32;
        for i in 0..3 {
            mapping[i] = mapping[i]
                .wrapping_add((increments[i] >> 3).wrapping_mul(start.wrapping_sub(center)));
        }
        let mut from = project(mapping);
        for i in 0..3 {
            mapping[i] = mapping[i].wrapping_add(increments[i]);
        }
        let mut to = project(mapping);
        let mut x = start;
        // The original alpha branch computes this once, even as shade advances below.
        let alpha_light = (shade >> 8).wrapping_mul(256 - alpha as i32) >> 8;
        while x < end {
            let mut packed = from[0].wrapping_shl(18).wrapping_add(from[1]);
            let delta = (to[0].wrapping_sub(from[0]) >> 3)
                .wrapping_shl(18)
                .wrapping_add(to[1].wrapping_sub(from[1]) >> 3);
            let count = (end - x).min(8);
            let light = if alpha == 0 { shade >> 8 } else { alpha_light };
            for _ in 0..count {
                let texel = texture[((packed as u32 & 16256) + (packed as u32 >> 25)) as usize];
                let index = row + x as usize;
                if (opaque || texel != 0) && (!DEPTH || (depths[index] - z).is_sign_negative()) {
                    let color = modulate(texel, light);
                    if alpha == 0 {
                        if COLOR {
                            pixels[index] = color | color_mask;
                        }
                        if DEPTH && write_depth {
                            depths[index] = z;
                        }
                    } else if COLOR {
                        pixels[index] =
                            color.wrapping_add(modulate(pixels[index], alpha as i32)) | color_mask;
                    }
                    hit(index);
                }
                x += 1;
                packed = packed.wrapping_add(delta);
                z += depth_step;
            }
            from = to;
            for i in 0..3 {
                mapping[i] = mapping[i].wrapping_add(increments[i]);
            }
            to = project(mapping);
            shade = shade.wrapping_add(shade_step.wrapping_shl(3));
        }
    }
}

impl Span {
    pub fn draw_terrain(
        self,
        pixels: &mut [u32],
        depths: &mut [f32],
        texture: &[u32],
        write_depth: bool,
        color_mask: u32,
        mut hit: impl FnMut(usize),
    ) {
        let Self {
            row,
            width,
            center,
            start,
            end,
            mut shade,
            shade_step,
            initial,
            depth_step,
            mut mapping,
            increments,
            alpha: _,
            opaque,
        } = self;

        let start = start.max(0);
        let end = end.min(width as i32);
        if start >= end {
            return;
        }
        shade = shade.wrapping_add(shade_step.wrapping_mul(start));
        let mut z = initial + depth_step * start as f32;
        let count = end - start;
        for i in 0..3 {
            mapping[i] =
                mapping[i].wrapping_add(increments[i].wrapping_mul(start.wrapping_sub(center)));
        }
        let from = project_terrain(mapping);
        for i in 0..3 {
            mapping[i] = mapping[i].wrapping_add(increments[i].wrapping_mul(count));
        }
        let to = project_terrain(mapping);
        let mut packed = from[0].wrapping_shl(18).wrapping_add(from[1]);
        let delta = to[0]
            .wrapping_sub(from[0])
            .wrapping_div(count)
            .wrapping_shl(18)
            .wrapping_add(to[1].wrapping_sub(from[1]).wrapping_div(count));
        let mut x = start;
        while x < end {
            let light = shade >> 8;
            for _ in 0..(end - x).min(8) {
                let texel = texture[((packed as u32 & 16256) + (packed as u32 >> 25)) as usize];
                let index = row + x as usize;
                if (opaque || texel != 0) && (depths[index] - z).is_sign_negative() {
                    pixels[index] = modulate(texel, light) | color_mask;
                    if write_depth {
                        depths[index] = z;
                    }
                    hit(index);
                }
                x += 1;
                packed = packed.wrapping_add(delta);
                z += depth_step;
            }
            shade = shade.wrapping_add(shade_step.wrapping_shl(3));
        }
    }
}

fn project_terrain(mapping: [i32; 3]) -> [i32; 2] {
    let divisor = mapping[2] >> 14;
    if divisor == 0 {
        [0, 0]
    } else {
        [
            mapping[0].wrapping_div(divisor),
            mapping[1].wrapping_div(divisor),
        ]
    }
}
