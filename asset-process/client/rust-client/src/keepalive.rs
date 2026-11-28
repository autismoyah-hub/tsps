//! Original packet-writer idle count, measured in client cycles rather than time.
#[derive(Default, Debug, Clone, Copy)]
pub struct Keepalive {
    cycles: i32,
}
impl Keepalive {
    pub fn cycles(&self) -> i32 {
        self.cycles
    }
    pub fn written(&mut self) {
        self.cycles = 0;
    }
    /// Active gameplay checks strictly greater than 50; a successful flush resets it.
    pub fn gameplay_tick(&mut self) -> bool {
        self.cycles = self.cycles.wrapping_add(1);
        self.cycles > 50
    }
    /// Loading's helper resets before attempting its forced or 50-call flush.
    pub fn loading_tick(&mut self, force: bool) -> bool {
        self.cycles = self.cycles.wrapping_add(1);
        if self.cycles >= 50 || force {
            self.written();
            true
        } else {
            false
        }
    }
}
