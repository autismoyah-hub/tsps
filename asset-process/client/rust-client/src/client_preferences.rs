//! Script-controlled client display and input preferences.
use crate::{interface_script::ints, vm::Stacks, Result};
pub struct Preferences {
    /// Bootstrap policy revision (GameClient.gy); unlike accepted_eula, this is not persisted.
    pub required_eula: i32,
    pub saved: crate::saved_preferences::SavedPreferences,
    pub device: crate::device_options::DeviceOptions,
    pub follower_ops_low_priority: bool,
    pub mouse_camera: bool,
    pub camera_follow_height: i32,
    pub mouse_cross: bool,
    pub loading_messages: bool,
    pub player_names: u8,
    pub mouse_over_text: bool,
    pub render_self: bool,
    pub tap_to_drop: bool,
    pub camera_speed: [i32; 2],
    pub window_mode: i32,
    pub preferred_window_mode: i32,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            required_eula: -1,
            saved: Default::default(),
            device: Default::default(),
            follower_ops_low_priority: false,
            mouse_camera: false,
            camera_follow_height: 50,
            mouse_cross: true,
            loading_messages: true,
            player_names: 0,
            mouse_over_text: true,
            render_self: true,
            tap_to_drop: false,
            camera_speed: [12, 6],
            window_mode: 1,
            preferred_window_mode: 1,
        }
    }
}
impl Preferences {
    pub fn load_saved(&mut self, bytes: &[u8]) -> Result<()> {
        let saved = crate::saved_preferences::SavedPreferences::decode(bytes)?;
        self.device = saved.device.clone();
        self.preferred_window_mode = saved.window_mode;
        self.saved = saved;
        Ok(())
    }
    pub fn encode_saved(&mut self) -> Result<Vec<u8>> {
        self.saved.device = self.device.clone();
        self.saved.window_mode = self.preferred_window_mode;
        self.saved.encode()
    }
    pub fn execute(&mut self, op: u16, s: &mut Stacks, canvas: [i32; 2]) -> Result<bool> {
        if self.device.execute(op, s)? {
            return Ok(true);
        }
        match op {
            6512 => self.follower_ops_low_priority = s.pop_int()? == 1,
            3110 => self.mouse_camera = s.pop_int()? == 1,
            3117 | 3131 | 3133 | 5310 => {
                s.pop_int()?;
            }
            3118 => self.mouse_over_text = s.pop_int()? == 1,
            3119 => self.render_self = s.pop_int()? == 1,
            3120..=3123 => {
                let mask = 1 << (op - 3120);
                if s.pop_int()? == 1 {
                    self.player_names |= mask;
                } else {
                    self.player_names &= !mask;
                }
            }
            3124 => self.player_names = 0,
            3125 => self.mouse_cross = s.pop_int()? == 1,
            3126 => self.loading_messages = s.pop_int()? == 1,
            3127 => self.tap_to_drop = s.pop_int()? == 1,
            3128 => s.push_int(self.tap_to_drop as i32)?,
            3129 => self.camera_speed = ints(s)?,
            3130 => {
                ints::<2>(s)?;
            }
            3132 => {
                s.push_int(canvas[0])?;
                s.push_int(canvas[1])?;
            }
            3134 => {}
            5530 => self.camera_follow_height = s.pop_int()?.max(0),
            5531 => s.push_int(self.camera_follow_height)?,
            5306 => s.push_int(self.window_mode)?,
            5307 => {
                let v = s.pop_int()?;
                if v == 1 || v == 2 {
                    self.window_mode = v;
                }
            }
            5308 => s.push_int(self.preferred_window_mode)?,
            5309 => {
                let v = s.pop_int()?;
                if v == 1 || v == 2 {
                    self.preferred_window_mode = v;
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
