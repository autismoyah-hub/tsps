//! Original GameWorldMapRenderer.ie flashing positions on the overview image.
use crate::raster::Raster;
pub fn draw(
    raster: &mut Raster,
    rect: [i32; 4],
    base: [i32; 2],
    extent: [i32; 2],
    points: impl IntoIterator<Item = [i32; 2]>,
    cycle: i32,
    period: i32,
) {
    if cycle <= 0 || period <= 0 || cycle % period >= period / 2 || extent.iter().any(|&n| n == 0) {
        return;
    }
    for point in points {
        let x = rect[0].wrapping_add(
            point[0]
                .wrapping_sub(base[0])
                .wrapping_mul(rect[2])
                .wrapping_div(extent[0]),
        );
        let y = rect[1].wrapping_add(
            rect[3].wrapping_sub(
                rect[3]
                    .wrapping_mul(point[1].wrapping_sub(base[1]))
                    .wrapping_div(extent[1]),
            ),
        );
        // Radius-two midpoint circle: five exact horizontal spans.
        for (dy, width) in [(-2, 3), (-1, 5), (0, 5), (1, 5), (2, 3)] {
            raster.fill_rect(
                x.wrapping_sub(width / 2),
                y.wrapping_add(dy),
                width,
                1,
                0xffff00,
                256,
            );
        }
    }
}

pub struct Markers {
    pub base: [i32; 2],
    pub extent: [i32; 2],
    pub points: Vec<[i32; 2]>,
    pub cycle: i32,
    pub period: i32,
}
impl Markers {
    pub fn draw(&self, raster: &mut Raster, rect: [i32; 4]) {
        draw(
            raster,
            rect,
            self.base,
            self.extent,
            self.points.iter().copied(),
            self.cycle,
            self.period,
        );
    }
}
