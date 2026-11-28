//! World-map overlay color selection (ba.ag).
use crate::{
    floor::{pack_hsl, Overlay},
    Error, Result,
};
pub fn overlay_color(
    definition: Option<&Overlay>,
    background: u32,
    texture_average: i32,
    palette: &[u32],
) -> Result<u32> {
    let Some(def) = definition else {
        return Ok(background);
    };
    let color = if let Some(hsl) = def.secondary {
        pack_hsl(hsl.hue, hsl.saturation, hsl.lightness)
    } else if def.texture.is_some() {
        texture_average
    } else if def.rgb == 0xff00ff {
        return Ok(background);
    } else {
        pack_hsl(
            def.primary.hue,
            def.primary.saturation,
            def.primary.lightness,
        )
    };
    let index = match color {
        -2 => 12345678,
        -1 => 31,
        _ => (color & 65408) + ((color & 127) * 96 / 128).clamp(2, 126),
    };
    palette
        .get(index as usize)
        .map(|v| v | 0xff000000)
        .ok_or(Error::Invalid("map overlay palette index"))
}
