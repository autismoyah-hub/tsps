//! Scheduled actor animation freezes from GameActor.dh and GameClient.nc.
use crate::actor_sequence::State;
#[derive(Clone, Debug)]
pub struct Interval {
    pub start: i32,
    pub end: i32,
    pub cancel: bool,
    pub last: i32,
}
impl Interval {
    fn active(&self, cycle: i32) -> bool {
        cycle >= self.start && cycle < self.end
    }
    fn expired(&self, cycle: i32) -> bool {
        !self.active(cycle) && cycle >= self.end
    }
}
#[derive(Clone, Debug, Default)]
pub struct Freezes {
    pub intervals: Vec<Interval>,
    pub active: bool,
    pub display: Option<State>,
    pub display_pose: Option<State>,
}
impl Freezes {
    pub fn clear(&mut self) {
        self.intervals.clear();
        self.display = None;
        self.display_pose = None;
    }
    pub fn apply(
        &mut self,
        delay: u16,
        duration: u16,
        cancel: bool,
        cycle: i32,
        sequence: &mut State,
    ) {
        if delay == 65535 || duration == 65535 {
            self.clear();
            if cancel {
                sequence.reset();
            }
        } else {
            self.add(delay as i32, duration as i32, cancel, cycle);
        }
    }
    pub fn add(&mut self, delay: i32, duration: i32, cancel: bool, cycle: i32) {
        let start = cycle.wrapping_add(delay);
        let end = start.wrapping_add(duration);
        if let Some(interval) = self.intervals.iter_mut().find(|v| v.expired(cycle)) {
            interval.start = start;
            interval.end = end;
            interval.cancel = cancel;
        } else {
            self.intervals.push(Interval {
                start,
                end,
                cancel,
                last: 0,
            });
        }
    }
    pub fn tick(&mut self, cycle: i32, sequence: &mut State) -> bool {
        let mut entered = false;
        let mut active = false;
        for interval in &mut self.intervals {
            if interval.start > interval.last && interval.start <= cycle {
                if !entered {
                    entered = true;
                    if !self.active {
                        self.display =
                            (sequence.id >= 0 && sequence.delay == 0).then(|| sequence.clone());
                    }
                }
                if interval.cancel {
                    sequence.reset();
                }
            }
            active |= interval.active(cycle);
            interval.last = cycle;
        }
        if !active && self.active {
            self.display = None;
            self.display_pose = None;
        }
        self.active = active;
        active
    }
    pub fn tick_pose(&mut self, cycle: i32, sequence: &mut State, pose: &State) -> bool {
        let entering = !self.active
            && self
                .intervals
                .iter()
                .any(|v| v.start > v.last && v.start <= cycle);
        if entering {
            self.display_pose = (pose.id >= 0).then(|| pose.clone());
        }
        self.tick(cycle, sequence)
    }
    pub fn displayed_pose<'a>(&'a self, pose: &'a State) -> Option<&'a State> {
        if self.active {
            self.display_pose.as_ref()
        } else {
            (pose.id >= 0).then_some(pose)
        }
    }
    pub fn displayed<'a>(&'a self, sequence: &'a State) -> Option<&'a State> {
        if self.active {
            self.display.as_ref()
        } else {
            (sequence.id >= 0 && sequence.delay == 0).then_some(sequence)
        }
    }
}
