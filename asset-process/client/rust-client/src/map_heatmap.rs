//! GameRenderOverview.ah heatmap cache and compositing.
use crate::{heatmap::Heatmap, Error, Result};
#[derive(Default)]
pub struct Overlay {
    pub origin: [i32; 2],
    pub scale: i32,
    pub cycle: i32,
    size: [usize; 2],
    pixels: Vec<u32>,
    pub renders: u64,
}
impl Overlay {
    pub fn paint(
        &mut self,
        heat: &mut Heatmap,
        target: &mut [u32],
        surface: [usize; 2],
        clip: [usize; 4],
        rect: [i32; 4],
        center: [i32; 2],
        tiles: [i32; 2],
        scale: i32,
        cycle: i32,
    ) -> Result<()> {
        if !(1..=8).contains(&scale)
            || rect[2] <= 0
            || rect[3] <= 0
            || rect[2] > 3584
            || rect[3] > 3584
            || surface[0].checked_mul(surface[1]) != Some(target.len())
        {
            return Err(Error::Invalid("map heatmap dimensions"));
        }
        let margin = 512 / (scale * 2);
        let size = [rect[2] as usize + 512, rect[3] as usize + 512];
        let origin = [
            center[0].wrapping_sub(tiles[0] / 2).wrapping_sub(margin),
            center[1].wrapping_sub(tiles[1] / 2).wrapping_sub(margin),
        ];
        let offset = |cached: [i32; 2]| {
            [
                rect[0].wrapping_sub(
                    scale.wrapping_mul(origin[0].wrapping_add(margin).wrapping_sub(cached[0])),
                ),
                rect[1].wrapping_sub(
                    scale.wrapping_mul(margin.wrapping_sub(origin[1].wrapping_sub(cached[1]))),
                ),
            ]
        };
        let mut at = offset(self.origin);
        if self.pixels.is_empty()
            || self.size != size
            || self.scale != scale
            || self.cycle != cycle
            || at[0] > 0
            || at[1] > 0
            || (at[0] as i64 + size[0] as i64) < rect[2] as i64
            || (at[1] as i64 + size[1] as i64) < rect[3] as i64
        {
            self.origin = origin;
            self.size = size;
            self.scale = scale;
            self.cycle = cycle;
            self.pixels = heat.render(origin, size, scale as f32)?;
            self.renders += 1;
            at = offset(self.origin);
        }
        let [left, top, right, bottom] = clip;
        let right = right.min(surface[0]);
        let bottom = bottom.min(surface[1]);
        for y in
            (rect[1] as i64).max(top as i64)..(rect[1] as i64 + rect[3] as i64).min(bottom as i64)
        {
            for x in (rect[0] as i64).max(left as i64)
                ..(rect[0] as i64 + rect[2] as i64).min(right as i64)
            {
                let pixel = &mut target[y as usize * surface[0] + x as usize];
                *pixel = (((*pixel & 0xff00ff) * 128 >> 8) & 0xff00ff)
                    + (((*pixel & 0xff00) * 128 >> 8) & 0xff00);
            }
        }
        for y in (at[1] as i64).max(top as i64)..(at[1] as i64 + size[1] as i64).min(bottom as i64)
        {
            for x in
                (at[0] as i64).max(left as i64)..(at[0] as i64 + size[0] as i64).min(right as i64)
            {
                let source = self.pixels
                    [(y - at[1] as i64) as usize * size[0] + (x - at[0] as i64) as usize];
                let target = &mut target[y as usize * surface[0] + x as usize];
                let rb = (source & 0xff00ff) * 192;
                let source = ((rb & 0xff00ff00)
                    + (source.wrapping_mul(192).wrapping_sub(rb) & 0xff0000))
                    >> 8;
                let sum = source.wrapping_add(*target);
                let rb = (source & 0xff00ff) + (*target & 0xff00ff);
                let carry = (rb & 0x1000100) + (sum.wrapping_sub(rb) & 0x10000);
                *target = sum.wrapping_sub(carry) | carry.wrapping_sub(carry >> 8);
            }
        }
        Ok(())
    }
}
