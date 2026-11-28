//! Gameplay receive deadline; transport buffering alone is not packet progress.
#[derive(Default, Debug)]
pub struct ReceiveTimeout {
    cycles: u32,
    progress: u64,
}
impl ReceiveTimeout {
    pub fn observe(&mut self, progress: u64) {
        if self.progress != progress {
            self.progress = progress;
            self.cycles = 0;
        }
    }
    pub fn advance(&mut self, ticks: u32) -> bool {
        self.cycles = self.cycles.saturating_add(ticks).min(751);
        self.cycles > 750
    }
    pub fn cycles(&self) -> u32 {
        self.cycles
    }
}
