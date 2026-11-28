//! Legacy ga.ae tooltip layout and uj dwell clock.
use crate::{
    font::{Font, Style},
    raster::Raster,
    Result,
};

#[derive(Default, Debug, Clone)]
pub struct State {
    target: Option<(u32, u64)>,
    age: u8,
}
impl State {
    /// ps.as advances the previous target before ps.ap selects a hovered widget.
    pub fn tick(&mut self, hovered: Option<(u32, u64)>) {
        if self.target.is_some() {
            self.age = (self.age + 1).min(50);
        } else {
            self.age = self.age.saturating_sub(1);
        }
        if let Some(target) = hovered {
            self.target = Some(target);
        }
    }
    pub fn visible(&self, id: u32, instance: u64) -> bool {
        self.age == 50 && self.target == Some((id, instance))
    }
}

pub fn draw(
    r: &mut Raster,
    font: &Font,
    text: &str,
    rect: [i32; 4],
    parent: [i32; 4],
) -> Result<()> {
    draw_with_icons(r, font, text, rect, parent, &[])
}
pub fn draw_with_icons(
    r: &mut Raster,
    font: &Font,
    text: &str,
    rect: [i32; 4],
    parent: [i32; 4],
    icons: &[crate::sprite::SpriteSheet],
) -> Result<()> {
    let lines: Vec<_> = text.split_terminator("<br>").collect();
    let width = lines
        .iter()
        .map(|line| font.metrics.width(line, icons))
        .max()
        .unwrap_or(0)
        + 6;
    let height = lines.len() as i32 * (font.metrics.ascent + 1) + 7;
    let [x, y, w, h] = rect;
    let mut left = (x + w - 5 - width).max(x + 5);
    let mut top = y + h + 5;
    if left + width > parent[2] {
        left = parent[2] - width;
    }
    if top + height > parent[3] {
        top = parent[3] - height;
    }
    r.fill_rect(left, top, width, height, 0xffffa0, 256);
    r.fill_rect(left, top, width, 1, 0, 256);
    r.fill_rect(left, top + height - 1, width, 1, 0, 256);
    r.fill_rect(left, top, 1, height, 0, 256);
    r.fill_rect(left + width - 1, top, 1, height, 0, 256);
    for (i, line) in lines.iter().enumerate() {
        font.draw(
            r,
            line,
            left + 3,
            top + 2 + font.metrics.ascent + i as i32 * (font.metrics.ascent + 1),
            Style {
                color: 0,
                shadow: None,
                alpha: 256,
            },
            icons,
        )?;
    }
    Ok(())
}
