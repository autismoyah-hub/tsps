//! Map floor tile raster composition, including the original 255-based pattern alpha.
use crate::{map_geography::Tile, map_mask::Masks, Error, Result};
fn pattern(pixels: &mut [u32], mask: &[u8], background: u32, foreground: u32) {
    let opaque = background >> 24 == 255 && foreground >> 24 == 255;
    for (dst, &bit) in pixels.iter_mut().zip(mask) {
        let color = if bit == 0 { background } else { foreground };
        if opaque {
            *dst = color;
        } else {
            let a = color >> 24;
            let inv = 255 - a;
            let rb = (color & 0xff00ff)
                .wrapping_mul(a)
                .wrapping_add((*dst & 0xff00ff).wrapping_mul(inv))
                & 0xff00ff00;
            let g = (color & 0xff00)
                .wrapping_mul(a)
                .wrapping_add((*dst & 0xff00).wrapping_mul(inv))
                & 0xff0000;
            *dst = (rb.wrapping_add(g) as i32 >> 8) as u32;
        }
    }
}
pub fn paint(
    pixels: &mut [u32],
    tile: &Tile,
    masks: &Masks,
    background: u32,
    ground: u32,
    colors: &[u32],
) -> Result<()> {
    if pixels.len() != masks.size * masks.size || colors.len() != tile.layers.len() {
        return Err(Error::Invalid("map floor tile dimensions"));
    }
    let base = tile.layers.first();
    let under = if tile.underlay == 0 {
        background
    } else {
        ground
    };
    match base {
        Some(layer) if layer.overlay != 0 => {
            if layer.shape == 0 && layer.overlay as i16 > 0 {
                pixels.fill(colors[0] | 0xff000000);
            } else if layer.shape != 0 {
                pattern(
                    pixels,
                    masks.get(layer.shape, layer.rotation)?,
                    under,
                    colors[0],
                );
            }
        }
        _ => pixels.fill(under | 0xff000000),
    }
    for (layer, &color) in tile.layers.iter().zip(colors).skip(1) {
        if layer.overlay as i16 > 0 {
            if layer.shape == 0 {
                pixels.fill(color | 0xff000000);
            } else {
                pattern(pixels, masks.get(layer.shape, layer.rotation)?, 0, color);
            }
        }
    }
    Ok(())
}
