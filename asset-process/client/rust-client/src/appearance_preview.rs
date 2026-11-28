//! ga.aa character-design widget overrides, shared across interface groups.
use crate::widget::Widget;
#[derive(Default)]
pub struct Preview {
    sprites: Option<[i32; 2]>,
}
impl Preview {
    /// Returns the model-type-5 selector (false: designer, true: local actor).
    pub fn update(&mut self, w: &mut Widget, gender: u8, cycle: i32) -> Option<bool> {
        match w.content {
            324 | 325 => {
                if self.sprites.is_none_or(|sprites| sprites[0] == -1) {
                    self.sprites = Some(w.sprites);
                }
                let sprites = self.sprites.unwrap();
                w.sprites[0] = sprites[usize::from((w.content == 324) == (gender != 1))];
                None
            }
            327 | 328 => {
                w.model_angles[0] = 150;
                w.model_angles[1] = (((cycle as f64 / 40.0).sin() * 256.0) as i32 & 2047) as u16;
                Some(w.content == 328)
            }
            _ => None,
        }
    }
}
