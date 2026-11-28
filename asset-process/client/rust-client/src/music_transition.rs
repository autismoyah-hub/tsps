//! Per-client-cycle music fade and delay task clocks (wq, vn, va).
#[derive(Clone, Debug)]
pub struct Fade {
    pub duration: i32,
    pub fade_in: bool,
}
impl Fade {
    /// Returns true on task completion. The cycle reaching the target still
    /// returns false, exactly as the original asynchronous music task does.
    pub fn tick(&self, current: &mut f32, target: i32, playing: bool) -> bool {
        if playing
            && if self.fade_in {
                *current < target as f32
            } else {
                *current > 0.0
            }
        {
            let step = if self.duration == 0 {
                0.0
            } else {
                target as f32 / self.duration as f32
            };
            let step = if step == 0.0 { target as f32 } else { step };
            if self.fade_in {
                *current += step;
                if *current > target as f32 {
                    *current = target as f32;
                }
            } else {
                *current -= step;
                if *current < 0.0 {
                    *current = 0.0;
                }
            }
            false
        } else {
            true
        }
    }
}
#[derive(Clone, Debug)]
pub struct Delay {
    pub elapsed: i64,
    pub duration: i32,
}
impl Delay {
    pub fn tick(&mut self) -> bool {
        if self.elapsed < self.duration as i64 {
            self.elapsed = self.elapsed.wrapping_add(1);
            false
        } else {
            true
        }
    }
}
