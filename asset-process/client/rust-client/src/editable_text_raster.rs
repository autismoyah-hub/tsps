//! Original gj selection/caret painting over zi character positions.
use crate::{
    editable_text_layout::{positions, Options},
    font::Font,
    raster::Raster,
    Error, Result,
};
#[derive(Clone, Copy, Debug)]
pub struct Paint {
    pub rect: [i32; 4],
    pub scroll: [i32; 2],
    pub cursor: usize,
    pub anchor: usize,
    pub color: u32,
    pub shadow: Option<u32>,
    pub selected: u32,
    pub background: u32,
    pub caret: bool,
    pub show_last: bool,
}
fn align(value: i32, mode: i32) -> i32 {
    match mode {
        1 => value / 2,
        2 => value,
        _ => 0,
    }
}
pub fn draw(
    raster: &mut Raster,
    font: &Font,
    text: &[u16],
    options: Options,
    paint: Paint,
) -> Result<()> {
    let points = positions(text, &font.metrics, options)?;
    let [x, y, w, h] = paint.rect;
    let origin = [
        x - paint.scroll[0] + align(w, options.align[0]),
        font.metrics.ascent + y - paint.scroll[1] + align(h, options.align[1]),
    ];
    raster.set_clip(x, y, x.wrapping_add(w), y.wrapping_add(h));
    let start = paint.cursor.min(paint.anchor);
    let end = paint.cursor.max(paint.anchor);
    for index in (start..end).filter(|_| end <= text.len()) {
        let ch = if text[index] == 160 { 32 } else { text[index] } as usize;
        let width = *font
            .metrics
            .advances
            .get(ch)
            .ok_or(Error::Invalid("editable selection character"))?;
        raster.fill_rect(
            origin[0] + points[index][0],
            origin[1] + points[index][1] - font.metrics.ascent,
            width as i32,
            font.max_ascent + font.max_descent,
            paint.background,
            256,
        );
    }
    for (begin, end, color) in [
        (0, start, paint.color),
        (start, end, paint.selected),
        (end, text.len(), paint.color),
    ] {
        for index in (begin..end).filter(|_| end <= text.len() && begin <= text.len()) {
            let mut ch = text[index];
            if ch == 10 {
                continue;
            }
            // zt.af compares the index relative to this range with its absolute end.
            if options.mask == 2
                || (options.mask == 1 && (!paint.show_last || index - begin != end - 1))
            {
                ch = 42;
            }
            if ch == 9 {
                continue;
            }
            if ch == 160 {
                ch = 32;
            }
            let g = font
                .glyphs
                .sprites
                .get(ch as usize)
                .ok_or(Error::Invalid("editable glyph"))?;
            let gx = origin[0] + points[index][0] + g.x as i32;
            let gy = origin[1] - font.metrics.ascent + points[index][1] + g.y as i32;
            if let Some(shadow) = paint.shadow {
                raster.draw_mask(&g.indices, g.width, g.height, gx + 1, gy + 1, shadow, 256)?;
            }
            raster.draw_mask(&g.indices, g.width, g.height, gx, gy, color, 256)?;
        }
    }
    if paint.caret {
        let endpoint = if paint.cursor == 0 || paint.cursor > text.len() {
            [0, 0]
        } else {
            let index = paint.cursor - 1;
            let ch = text[index];
            let width = if ch == 10 {
                0
            } else if options.mask == 2 || (options.mask != 0 && index + 1 != text.len()) {
                font.metrics.advances[42] as i32
            } else {
                let advance = font.metrics.advances[ch as usize] as i32;
                if advance == 0 {
                    font.metrics.advances[32] as i32 * if ch == 9 { 3 } else { 1 }
                } else {
                    advance
                }
            };
            [points[index][0] + width, points[index][1]]
        };
        let cx = origin[0] + endpoint[0];
        let cy = origin[1] + endpoint[1];
        let height = h.min((font.metrics.ascent as f32 * 0.9) as i32);
        crate::widget_line::draw(raster, [cx, cy], [cx, cy - height], 1, paint.color);
        if let Some(shadow) = paint.shadow {
            crate::widget_line::draw(
                raster,
                [cx + 1, cy + 1],
                [cx + 1, cy - height + 1],
                1,
                shadow,
            );
        }
    }
    Ok(())
}
