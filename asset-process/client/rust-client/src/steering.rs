//! Sixteen-way steering input used by the revision-239 minimap control mode.
use crate::{protocol::Packet, Error, Result};
#[derive(Clone, Copy, Debug, Default)]
pub struct Steering {
    pub direction: u8,
    pub remaining: u8,
    /// Current root-scene submissions, refreshed before each world paint.
    pub scene: [Option<crate::steering_scene::Placement>; 2],
}
impl Steering {
    pub fn request(&mut self, direction: u8) -> Result<Packet> {
        if direction >= 16 {
            return Err(Error::Invalid("steering direction"));
        }
        self.direction = direction;
        self.remaining = 30;
        Ok(Packet {
            opcode: 44,
            payload: vec![direction],
        })
    }
    pub fn tick(&mut self) {
        self.remaining = self.remaining.saturating_sub(1);
    }
}
/// The injected GameWorldMapIcon.av uses a float angle and GameClient.oe's float rounding.
pub fn minimap_direction(centered: [i32; 2], camera: u16) -> u8 {
    let angle = (centered[1] as f64).atan2(centered[0] as f64) as f32;
    let heading = (angle * 325.9493f32 + 0.5).floor() as i32;
    (((heading - 512 - (camera as i32 >> 3) + 64) & 2047) / 128) as u8
}
pub fn minimap_click(
    mask: &crate::minimap::Mask,
    press: [i32; 2],
    pointer: [i32; 2],
    camera: u16,
) -> Option<u8> {
    mask.contains(press).then(|| {
        minimap_direction(
            [
                pointer[0] - mask.width as i32 / 2,
                pointer[1] - mask.starts.len() as i32 / 2,
            ],
            camera,
        )
    })
}

/// GameClient.hc / GameSpritePixels.il, including odd-framebuffer source correction.
pub fn draw_arrow(
    sprite: &crate::sprite_pixels::Pixels,
    raster: &mut crate::raster::Raster,
    centre: [i32; 2],
    camera: u16,
    direction: u16,
) {
    let angle = (direction as i32 + (camera as i32 >> 3)) & 2047;
    let theta = angle as f64 * std::f64::consts::TAU / 2048.0;
    let sin = (theta.sin() * 65536.0) as i32;
    let cos = (theta.cos() * 65536.0) as i32;
    let [w, h] = sprite.size.map(|v| v as i32);
    let half = w.max(h) / 2;
    let x = centre[0] - (sin * 30 >> 16) - half;
    let y = centre[1] + (cos * 30 >> 16) - half;
    let rotation = angle as f64 * 0.0030679615;
    let mut source = [w / 2, h / 2];
    if rotation < std::f64::consts::FRAC_PI_2 || rotation > std::f64::consts::PI * 1.5 {
        source[0] += (raster.width() % 2) as i32;
        source[1] += (raster.height() % 2) as i32;
    }
    let sin = (rotation.sin() * 65536.0) as i32;
    let cos = (rotation.cos() * 65536.0) as i32;
    let mut sx = (source[0] << 16) + (-h / 2) * sin + (-w / 2) * cos;
    let mut sy = (source[1] << 16) + (-h / 2) * cos - (-w / 2) * sin;
    for row in 0..h {
        let (mut u, mut v) = (sx, sy);
        for col in 0..w {
            let (px, py) = (u >> 16, v >> 16);
            if px >= 0 && px < w && py >= 0 && py < h {
                let color = sprite.pixels[py as usize * sprite.size[0] + px as usize];
                if color != 0 {
                    // This original routine uses flat array addressing, without the raster clip.
                    let index = x + col + (y + row) * raster.width() as i32;
                    if index < 0 || index as usize >= raster.pixels().len() {
                        return;
                    }
                    raster.blend(
                        index as usize % raster.width(),
                        index as usize / raster.width(),
                        color,
                        256,
                    );
                }
            }
            u += cos;
            v -= sin;
        }
        sx += sin;
        sy += cos;
    }
}
