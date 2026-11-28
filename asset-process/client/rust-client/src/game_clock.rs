//! Pinned engine mj.sd's 20 ms simulation clock, independent of painting.
#[derive(Clone, Copy, Debug, Default)]
pub struct Clock {
    pub next_ns: i64,
    pub last_ns: i64,
}
#[derive(Clone, Copy, Debug)]
pub struct Step {
    pub ticks: u32,
    pub interpolation: f64,
}
impl Clock {
    pub fn advance(&mut self, now_ns: i64) -> Step {
        const PERIOD: i64 = 20_000_000;
        let mut ticks = 0;
        while ticks < 10 && self.next_ns < now_ns {
            ticks += 1;
            self.last_ns = self.next_ns;
            self.next_ns = self.next_ns.wrapping_add(PERIOD);
        }
        if self.next_ns < now_ns {
            self.next_ns = now_ns;
        }
        Step {
            ticks,
            interpolation: now_ns.wrapping_sub(self.last_ns).min(PERIOD) as f64 / PERIOD as f64,
        }
    }
}
