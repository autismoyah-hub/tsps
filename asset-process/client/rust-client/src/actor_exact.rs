//! Forced actor movement interpolation (the first two branches of GameActor.jc).
use crate::actor_path::Path;
#[derive(Clone, Debug)]
pub struct Exact {
    pub start: i32,
    pub end: i32,
    pub from: [i32; 2],
    pub to: [i32; 2],
    pub direction: u16,
}
impl Exact {
    /// Returns whether forced movement owns this tick. The caller supplies the action-frame gate.
    pub fn tick(&self, path: &mut Path, size: i32, cycle: i32, interpolate: bool) -> bool {
        let from = self
            .from
            .map(|v| v.wrapping_mul(128).wrapping_add(size.wrapping_mul(64)));
        if self.start >= cycle {
            let remaining = self.start.wrapping_sub(cycle).max(1);
            for axis in 0..2 {
                path.position[axis] = path.position[axis]
                    .wrapping_add(from[axis].wrapping_sub(path.position[axis]) / remaining);
            }
            path.fine = path.position.map(|v| v as f32);
            path.stalled = 0;
            true
        } else if self.end >= cycle {
            if interpolate || self.end == cycle {
                let duration = self.end.wrapping_sub(self.start);
                let elapsed = cycle.wrapping_sub(self.start);
                let to = self
                    .to
                    .map(|v| v.wrapping_mul(128).wrapping_add(size.wrapping_mul(64)));
                for axis in 0..2 {
                    path.position[axis] = elapsed
                        .wrapping_mul(to[axis])
                        .wrapping_add(from[axis].wrapping_mul(duration.wrapping_sub(elapsed)))
                        / duration;
                }
                path.fine = path.position.map(|v| v as f32);
            }
            path.stalled = 0;
            true
        } else {
            false
        }
    }
}
