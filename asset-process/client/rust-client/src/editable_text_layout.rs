//! Per-character editable text layout (zi.bb); markup is literal text.
use crate::{font::Metrics, Error, Result};
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub width: i32,
    pub max_lines: i32,
    pub wrap: i32,
    pub mask: i32,
    pub align: [i32; 2],
    pub line_height: i32,
}
fn aligned(value: i32, alignment: i32) -> i32 {
    match alignment {
        1 => value / 2,
        2 => value,
        _ => 0,
    }
}
pub fn positions(text: &[u16], font: &Metrics, options: Options) -> Result<Vec<[i32; 2]>> {
    let widths = text
        .iter()
        .map(|&ch| glyph_width(ch, font, options.mask))
        .collect::<Result<Vec<_>>>()?;
    let mut positions = vec![[0; 2]; text.len()];
    if text.is_empty() {
        return Ok(positions);
    }
    let (mut start, mut line_width, mut y, mut lines) = (0usize, 0i32, 0i32, 0i32);
    for index in 0..=text.len() {
        let end = index == text.len();
        let advance = widths.get(index).copied().unwrap_or(0);
        let newline = !end && text[index] == 10;
        let overflow =
            !end && options.max_lines > 1 && line_width.wrapping_add(advance) > options.width;
        if end || newline || overflow {
            let mut stop = index;
            let mut carry = 0i32;
            if overflow && options.wrap == 1 {
                let mut scan = index;
                let mut trailing = 0i32;
                while scan > start {
                    if scan < index {
                        trailing = trailing.wrapping_add(widths[scan]);
                    }
                    if text[scan] == 32 || text[scan] == 10 {
                        stop = scan;
                        line_width = line_width.wrapping_sub(trailing);
                        carry = trailing;
                        break;
                    }
                    scan -= 1;
                }
            }
            let mut x = -aligned(line_width, options.align[0]);
            for glyph in start..stop {
                positions[glyph] = [x, y];
                x = x.wrapping_add(widths[glyph]);
            }
            start = stop;
            line_width = carry;
            y = y.wrapping_add(options.line_height);
            lines += 1;
        }
        line_width = line_width.wrapping_add(advance);
    }
    let shift = aligned(lines.wrapping_mul(options.line_height), options.align[1]);
    for point in &mut positions {
        point[1] = point[1].wrapping_sub(shift);
    }
    Ok(positions)
}
fn glyph_width(ch: u16, font: &Metrics, mask: i32) -> Result<i32> {
    if ch == 10 {
        return Ok(0);
    }
    if mask != 0 {
        return Ok(font.advances[42] as i32);
    }
    let advance = *font
        .advances
        .get(ch as usize)
        .ok_or(Error::Invalid("editable text font character"))? as i32;
    Ok(if advance == 0 {
        font.advances[32] as i32 * if ch == 9 { 3 } else { 1 }
    } else {
        advance
    })
}

/// Original zi.ad chooses the closest glyph endpoint on the requested visual line.
pub fn vertical(
    text: &[u16],
    font: &Metrics,
    options: Options,
    cursor: i32,
    delta: i32,
) -> Result<i32> {
    if text.is_empty() || cursor > text.len() as i32 {
        return Ok(0);
    }
    if cursor < 0 {
        return Err(Error::Invalid("negative editable vertical cursor"));
    }
    let points = positions(text, font, options)?;
    let direction = if delta > 0 { 1 } else { -1 };
    let lines = delta.wrapping_abs();
    let (x, mut y) = if cursor > 0 {
        let i = cursor as usize - 1;
        (
            points[i][0].wrapping_add(glyph_width(text[i], font, options.mask)?),
            points[i][1],
        )
    } else if direction < 0 {
        return Ok(0);
    } else {
        (0, 0)
    };
    let mut crossed = 0;
    let mut best = cursor;
    let mut distance = 16777215;
    let end = if direction > 0 {
        text.len() as i32 + 1
    } else {
        0
    };
    let mut next = cursor + direction;
    while next != end {
        let i = next as usize - 1;
        if points[i][1] != y {
            crossed += 1;
            y = points[i][1];
            if crossed > lines {
                return Ok(best);
            }
        }
        if crossed == lines {
            let dx = points[i][0]
                .wrapping_add(glyph_width(text[i], font, options.mask)?)
                .wrapping_sub(x)
                .wrapping_abs();
            if dx >= distance {
                return Ok(best);
            }
            best = next;
            distance = dx;
        }
        next += direction;
    }
    if direction > 0 {
        return Ok(text.len() as i32);
    }
    if y != 0 {
        crossed += 1;
    }
    if distance != 16777215 && !(crossed == lines && x < distance) {
        Ok(best)
    } else {
        Ok(0)
    }
}
/// Original zi.au pointer-to-cursor mapping, including inclusive line edges.
pub fn hit_test(text: &[u16], font: &Metrics, options: Options, point: [i32; 2]) -> Result<usize> {
    let [x, y] = point;
    if options.max_lines > 1 && x > options.width {
        return Ok(text.len());
    }
    let points = positions(text, font, options)?;
    for (i, &[gx, gy]) in points.iter().enumerate() {
        if y <= gy.wrapping_add(options.line_height) {
            if y < gy {
                break;
            }
            if x < gx {
                return Ok(i.saturating_sub(1));
            }
            if i + 1 < points.len() && points[i + 1][1] != gy {
                if x < gx.wrapping_add(glyph_width(text[i], font, options.mask)?) {
                    return Ok(i);
                }
                return Ok(i + 1);
            }
        }
    }
    if let Some(&[gx, gy]) = points.last() {
        if x >= gx
            && x <= gx.wrapping_add(glyph_width(*text.last().unwrap(), font, options.mask)?)
            && y >= gy
            && y <= gy.wrapping_add(options.line_height)
        {
            return Ok(text.len() - 1);
        }
    }
    Ok(text.len())
}
