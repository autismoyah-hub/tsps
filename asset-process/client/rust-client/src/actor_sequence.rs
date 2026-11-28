//! Persistent actor action-animation state (li.bd, qp and GameActor.dg).
use crate::{animation::Sequence, animation_runtime::Clock, Error, Result};
use std::collections::HashMap;
#[derive(Clone, Debug)]
pub struct State {
    /// GameActor.an: retained while action ticking is frozen or path-suspended.
    pub stretch: bool,
    pub id: i32,
    pub frame: i32,
    pub elapsed: u32,
    pub loops: u32,
    pub loading: i32,
    pub delay: i32,
    pub path_at_start: i32,
}
impl Default for State {
    fn default() -> Self {
        Self {
            stretch: false,
            id: -1,
            frame: -1,
            elapsed: 0,
            loops: 0,
            loading: 0,
            delay: 0,
            path_at_start: 0,
        }
    }
}
impl State {
    pub fn reset(&mut self) {
        self.id = -1;
        self.frame = -1;
    }
    pub fn set(
        &mut self,
        id: i32,
        delay: i32,
        path: i32,
        sequences: &HashMap<u32, Sequence>,
    ) -> Result<()> {
        if self.id == id && id != -1 {
            let sequence = sequences
                .get(&(id as u32))
                .ok_or(Error::Invalid("actor sequence"))?;
            match sequence.restart {
                1 => {
                    self.frame = 0;
                    self.elapsed = 0;
                    self.loops = 0;
                    self.delay = delay;
                }
                2 => self.loops = 0,
                _ => {}
            }
            return Ok(());
        }
        if id != -1 && self.id != -1 {
            let new = sequences
                .get(&(id as u32))
                .ok_or(Error::Invalid("actor sequence"))?;
            let old = sequences
                .get(&(self.id as u32))
                .ok_or(Error::Invalid("actor previous sequence"))?;
            if new.priority < old.priority {
                return Ok(());
            }
        }
        if id >= 0 && !sequences.contains_key(&(id as u32)) {
            return Err(Error::Invalid("actor sequence"));
        }
        if id >= 0 && self.id != id {
            self.loading = 0;
        }
        self.id = id;
        self.frame = 0;
        self.elapsed = 0;
        self.loops = 0;
        self.delay = delay;
        self.path_at_start = path;
        Ok(())
    }
    pub fn allows_exact_step(&self, sequences: &HashMap<u32, Sequence>) -> bool {
        if self.id < 0 || self.delay != 0 {
            return true;
        }
        sequences.get(&(self.id as u32)).is_none_or(|s| {
            s.cached_animation.is_some()
                || s.lengths
                    .get(self.frame.max(0) as usize)
                    .is_none_or(|length| self.elapsed.wrapping_add(1) > *length as u32)
        })
    }
    pub fn suspend_for_path(
        &mut self,
        sequences: &HashMap<u32, Sequence>,
        path_at_start: i32,
        exact_start: i32,
        exact_end: i32,
        cycle: i32,
    ) -> Result<bool> {
        self.path_at_start = path_at_start;
        if self.id >= 0 && self.delay <= 1 {
            let sequence = sequences
                .get(&(self.id as u32))
                .ok_or(Error::Invalid("actor sequence"))?;
            if sequence.precedence == 1
                && path_at_start > 0
                && exact_start <= cycle
                && exact_end < cycle
            {
                self.delay = 1;
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// One actor simulation tick after movement (GameClient.nc). Returns crossed frames for sound dispatch.
    pub fn tick(
        &mut self,
        sequences: &HashMap<u32, Sequence>,
        path_at_start: i32,
        exact_start: i32,
        exact_end: i32,
        cycle: i32,
        frozen: bool,
        assets_ready: bool,
    ) -> Result<Vec<usize>> {
        if self.suspend_for_path(sequences, path_at_start, exact_start, exact_end, cycle)? {
            return Ok(Vec::new());
        }
        if self.delay > 0 {
            self.delay -= 1;
            if !frozen {
                self.stretch = sequences
                    .get(&(self.id as u32))
                    .is_some_and(|s| self.id >= 0 && s.stretch);
            }
            return Ok(Vec::new());
        }
        if frozen {
            return Ok(Vec::new());
        }
        if self.id < 0 {
            self.stretch = false;
            self.loading = 0;
            return Ok(Vec::new());
        }
        let sequence = sequences
            .get(&(self.id as u32))
            .ok_or(Error::Invalid("actor sequence"))?;
        self.loading = if assets_ready {
            0
        } else {
            self.loading.wrapping_add(1)
        };
        let mut clock = self.clock();
        let (flags, events) = clock.advance(sequence, 1)?;
        self.frame = clock.frame as i32;
        self.elapsed = clock.elapsed;
        self.loops = clock.loops;
        if flags & 2 != 0 {
            self.reset();
        }
        self.stretch = self.id >= 0 && sequence.stretch;
        Ok(events)
    }
    pub fn clock(&self) -> Clock {
        Clock {
            frame: self.frame.max(0) as usize,
            elapsed: self.elapsed,
            loops: self.loops,
            stopped: self.id < 0,
        }
    }
}
