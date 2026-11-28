//! World-map label sizing and zoom thresholds (GameWorldMapRegion.bm/bv).
use crate::{
    font::{Font, Style},
    raster::Raster,
    Result,
};
#[derive(Debug)]
pub struct Label {
    pub text: String,
    pub size: [i32; 2],
    pub text_size: u8,
}
impl Label {
    pub fn new(
        text: &str,
        text_size: u8,
        font: &Font,
        small_font_present: bool,
    ) -> Result<Option<Self>> {
        if !small_font_present || text_size > 2 {
            return Ok(None);
        }
        let lines = font.metrics.wrap(text, None, &[])?;
        let width = lines
            .iter()
            .map(|line| font.metrics.width(line, &[]))
            .max()
            .unwrap_or(0);
        let height = (lines.len() as i32).wrapping_mul(font.metrics.ascent) / 2;
        Ok(Some(Self {
            text: text.into(),
            size: [width, height],
            text_size,
        }))
    }
    pub fn draw(
        &self,
        font: &Font,
        raster: &mut Raster,
        point: [i32; 2],
        zoom: f32,
        color: u32,
    ) -> Result<()> {
        let threshold = match self.text_size {
            0 => 3.0,
            1 => 2.0,
            _ => 0.0,
        };
        if zoom >= threshold {
            font.draw_paragraph(
                raster,
                &self.text,
                [
                    point[0].wrapping_sub(self.size[0] / 2),
                    point[1],
                    self.size[0],
                    self.size[1],
                ],
                Style {
                    color: color | 0xff000000,
                    shadow: Some(0),
                    alpha: 256,
                },
                [1, 0],
                font.metrics.ascent / 2,
                &[],
            )?;
        }
        Ok(())
    }
    pub fn hit(&self, center: [i32; 2], point: [i32; 2]) -> bool {
        point[0] >= center[0].wrapping_sub(self.size[0] / 2)
            && point[0] <= center[0].wrapping_add(self.size[0] / 2)
            && point[1] >= center[1]
            && point[1] <= center[1].wrapping_add(self.size[1])
    }
}
