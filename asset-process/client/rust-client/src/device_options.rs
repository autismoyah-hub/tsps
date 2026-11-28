//! Revision 239 device-option CS2 operations (kb.bs).
use crate::{vm::Stacks, Error, Result};
#[derive(Clone)]
pub struct DeviceOptions {
    pub hide_username: bool,
    pub title_music_disabled: bool,
    pub display_fps: bool,
    pub extra: i32,
    pub brightness: f64,
    pub draw_distance: i32,
    pub master_volume: i32,
    pub option22: bool,
    pub roofs_hidden: bool,
    pub volumes: [i32; 3],
}
impl Default for DeviceOptions {
    fn default() -> Self {
        Self {
            hide_username: false,
            title_music_disabled: false,
            display_fps: false,
            extra: 0,
            brightness: 0.8,
            draw_distance: 25,
            master_volume: 100,
            option22: false,
            roofs_hidden: false,
            volumes: [127; 3],
        }
    }
}
impl DeviceOptions {
    /// GameScene.cq clamps the preference, then fv.as scales it to fine units.
    pub fn far_clip(&self) -> i32 { self.draw_distance.clamp(25, 90) * 210 }
    pub fn contains(id: i32) -> bool {
        matches!(id, 2 | 3 | 4 | 5 | 6 | 14 | 19 | 22)
    }
    pub fn get(&self, id: i32) -> Result<i32> {
        Ok(match id {
            2 => self.hide_username as i32,
            3 => self.title_music_disabled as i32,
            4 => self.display_fps as i32,
            5 => self.extra,
            6 => 100 - (200.0f32 * (self.brightness as f32 - 0.5) + 0.5).floor() as i32,
            14 => self.draw_distance,
            19 => ((self.master_volume as f32 / 100.0) * 100.0) as i32,
            22 => self.option22 as i32,
            _ => return Err(Error::Invalid("unknown device option")),
        })
    }
    pub fn set(&mut self, id: i32, value: i32) -> Result<()> {
        match id {
            2 => self.hide_username = value == 1,
            3 => self.title_music_disabled = value == 1,
            4 => self.display_fps = value == 1,
            5 => self.extra = value.max(0),
            6 => self.brightness = (0.5f32 + (100 - value.clamp(0, 100)) as f32 / 200.0) as f64,
            14 => self.draw_distance = value,
            19 => self.master_volume = value,
            22 => self.option22 = value == 1,
            _ => return Err(Error::Invalid("unknown device option")),
        }
        Ok(())
    }
    pub fn execute(&mut self, op: u16, s: &mut Stacks) -> Result<bool> {
        match op {
            3213 => {
                let value = s.pop_int()?;
                let id = s.pop_int()?;
                self.set_game(id, value)?;
            }
            3215 => {
                let id = s.pop_int()?;
                s.push_int(self.get_game(id)?)?;
            }
            3218 => {
                let id = s.pop_int()?;
                s.push_int(matches!(id, 1 | 7 | 8 | 9) as i32)?;
            }
            3219 => {
                let id = s.pop_int()?;
                let hi = match id {
                    1 => 1,
                    7 | 8 | 9 => 100,
                    _ => return Err(Error::Invalid("unknown game option")),
                };
                s.push_int(0)?;
                s.push_int(hi)?;
            }
            3209 => {
                let value = s.pop_int()?;
                let id = s.pop_int()?;
                if Self::contains(id) {
                    self.set(id, value)?;
                } else {
                    self.set_game(id, value)?;
                }
            }
            3210 => {
                let id = s.pop_int()?;
                s.push_int(if Self::contains(id) {
                    self.get(id)?
                } else {
                    self.get_game(id)?
                })?;
            }
            3203 | 3205 | 3207 => {
                let value = s.pop_int()?;
                self.set_game(7 + (op as i32 - 3203) / 2, value)?;
            }
            3204 | 3206 | 3208 => s.push_int(self.get_game(7 + (op as i32 - 3204) / 2)?)?,
            3212 => {
                let value = s.pop_int()?;
                let id = s.pop_int()?;
                self.set(id, value)?;
            }
            3214 => {
                let id = s.pop_int()?;
                s.push_int(self.get(id)?)?;
            }
            3216 => {
                let id = s.pop_int()?;
                s.push_int(Self::contains(id) as i32)?;
            }
            3217 => {
                let id = s.pop_int()?;
                let (lo, hi) = match id {
                    2 | 3 | 4 => (0, 1),
                    5 => (0, i32::MAX),
                    6 => (0, 100),
                    14 => (25, 90),
                    _ => return Err(Error::Invalid("device option has no range")),
                };
                s.push_int(lo)?;
                s.push_int(hi)?;
            }
            3181 => {
                let value = s.pop_int()?;
                self.set(6, value)?;
            }
            3182 => s.push_int(self.get(6)?)?,
            3211 => {}
            _ => return Ok(false),
        }
        Ok(true)
    }
    pub fn set_game(&mut self, id: i32, value: i32) -> Result<()> {
        match id {
            1 => self.roofs_hidden = value == 1,
            7..=9 => self.volumes[(id - 7) as usize] = value,
            _ => return Err(Error::Invalid("unknown game option")),
        }
        Ok(())
    }
    pub fn get_game(&self, id: i32) -> Result<i32> {
        if id == 1 {
            return Ok(self.roofs_hidden as i32);
        }
        if !(7..=9).contains(&id) {
            return Err(Error::Invalid("unknown game option"));
        }
        let table: &[i32] = if id == 7 {
            &crate::volume_tables::MUSIC
        } else {
            &crate::volume_tables::EFFECTS
        };
        let scaled = (self.master_volume as f32 / 100.0)
            * (self.volumes[(id - 7) as usize] as f32 / 100.0)
            * (table.len() - 1) as f32;
        let index = ((scaled + 0.5).floor() as i32).clamp(0, (table.len() - 1) as i32);
        Ok(table[index as usize])
    }
}
