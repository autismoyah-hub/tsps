//! Native event buffering matching tg and fz, independent of render frames.
#[derive(Debug)]
pub struct Input {
    pub held: [bool; 112],
    pub idle_age: i32,
    pending: Vec<i32>,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            held: [false; 112],
            idle_age: 0,
            pending: Vec::new(),
        }
    }
}
impl Input {
    pub fn key(&mut self, key: i32, down: bool) {
        if !(0..112).contains(&key) {
            return;
        }
        let held = &mut self.held[key as usize];
        if down {
            if !*held {
                self.idle_age = 0;
            }
            if self.pending.len() < 128 {
                self.pending.push(key);
            }
        }
        *held = down;
    }
    pub fn lose_focus(&mut self) {
        self.held.fill(false);
    }
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }
    pub fn advance(&mut self, ticks: u32) -> Option<(i32, Vec<i32>)> {
        if ticks == 0 {
            return None;
        }
        // Events belong to the first elapsed game cycle; later cycles have no presses.
        let age = self.idle_age.wrapping_add(1);
        self.idle_age = self.idle_age.wrapping_add(ticks as i32);
        Some((age, std::mem::take(&mut self.pending)))
    }
}
