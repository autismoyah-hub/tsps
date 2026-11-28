//! GameModel.by draw-time quantization and floating-point orientation.
use crate::{Error, Result};

pub struct Transform {
    angle: u16,
    sin: f32,
    cos: f32,
    position: [i32; 3],
}
impl Transform {
    pub fn new(angle: u16, position: [i32; 3]) -> Result<Self> {
        if angle >= 2048 {
            return Err(Error::Invalid("model draw angle"));
        }
        let radians = angle as f64 * 0.0030679615;
        Ok(Self {
            angle,
            sin: radians.sin() as f32,
            cos: radians.cos() as f32,
            position,
        })
    }
    /// Model editing has different rounding: drawing truncates all three input
    /// coordinates, then retains fractional orientation results.
    pub fn vertex(&self, input: [f32; 3]) -> [f32; 3] {
        let [mut x, y, mut z] = input.map(|v| v as i32 as f32);
        if self.angle != 0 {
            let rotated = z * self.sin + x * self.cos;
            z = z * self.cos - x * self.sin;
            x = rotated;
        }
        [
            x + self.position[0] as f32,
            y + self.position[1] as f32,
            z + self.position[2] as f32,
        ]
    }
}
pub fn vertices(input: &[[f32; 3]], angle: u16, position: [i32; 3]) -> Result<Vec<[f32; 3]>> {
    let transform = Transform::new(angle, position)?;
    Ok(input.iter().map(|v| transform.vertex(*v)).collect())
}
