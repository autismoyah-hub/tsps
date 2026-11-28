//! Original rw scheduling, with injected time, random draws and the existing PCM backend.
use crate::{
    ambient_sound::{fade_volume, scaled_duration, Definition, Fade},
    Error, Result,
};

pub trait Backend {
    fn play(&mut self, id: u16, legacy: bool, volume: i32, loops: i32) -> Result<Option<u64>>;
    fn volume(&self, voice: u64) -> Option<i32>;
    fn set_volume(&mut self, voice: u64, volume: i32);
    fn remove(&mut self, voice: u64);
}
#[derive(Debug)]
struct Voice {
    handle: u64,
    present: bool,
    volume: i32,
    from: i32,
    target: i32,
    start: i64,
    fade: Fade,
}
impl Voice {
    fn new(handle: u64, volume: i32) -> Self {
        Self {
            handle,
            volume,
            present: true,
            from: 0,
            target: 0,
            start: 0,
            fade: Fade {
                duration: 0,
                mode: 0,
            },
        }
    }
    fn fade(&mut self, target: i32, baseline: i32, fade: &Fade, now: i64) {
        if self.present && self.target != target {
            self.from = self.volume;
            self.target = target;
            self.start = now;
            self.fade = Fade {
                mode: fade.mode,
                duration: scaled_duration(fade.duration, self.from.wrapping_sub(target), baseline),
            };
        }
    }
    fn update(&mut self, now: i64, backend: &mut impl Backend) {
        if !self.present {
            return;
        }
        self.volume = fade_volume(
            self.from,
            self.target,
            self.volume,
            now.wrapping_sub(self.start),
            &self.fade,
        );
        backend.set_volume(self.handle, self.volume);
        if backend.volume(self.handle).is_none() {
            self.present = false;
            self.volume = 0;
        }
    }
    fn adjust(
        &mut self,
        volume: i32,
        definition: &Definition,
        now: i64,
        backend: &mut impl Backend,
    ) {
        if self.volume < volume {
            self.fade(volume, volume, &definition.fade_in, now);
        } else if self.volume > volume {
            self.fade(volume, volume, &definition.fade_out, now);
        }
        self.update(now, backend);
    }
}
#[derive(Default, Debug)]
pub struct State {
    current: Option<Definition>,
    legacy: bool,
    loops: Vec<Option<Voice>>,
    random: Vec<Option<Voice>>,
    deadlines: Vec<i64>,
    retiring: Vec<Voice>,
}
impl State {
    pub fn current_id(&self) -> Option<u32> {
        self.current.as_ref().map(|d| d.id)
    }
    pub fn retiring_count(&self) -> usize {
        self.retiring.len()
    }
    pub fn legacy(&self) -> bool {
        self.legacy
    }
    pub fn loop_present(&self, index: usize) -> bool {
        self.loops.get(index).is_some_and(Option::is_some)
    }
    pub fn start(
        &mut self,
        definition: Definition,
        fade: bool,
        legacy: bool,
        now: i64,
        draw: &mut impl FnMut() -> f64,
        backend: &mut impl Backend,
    ) -> Result<()> {
        if self.current_id() == Some(definition.id) {
            return Ok(());
        }
        self.stop(fade, now, backend);
        self.legacy = legacy;
        // pn permits this data, but rw's eight-element arrays reject it.
        if definition.loops.len() > 8 {
            return Err(Error::Invalid("ambient loop capacity"));
        }
        self.loops = (0..definition.loops.len()).map(|_| None).collect();
        self.random = (0..definition.random.len()).map(|_| None).collect();
        self.deadlines = definition
            .random
            .iter()
            .map(|g| now.wrapping_add(delay(g.min_ms, g.max_ms, draw()) as i64))
            .collect();
        self.current = Some(definition);
        Ok(())
    }
    pub fn stop(&mut self, fade: bool, now: i64, backend: &mut impl Backend) {
        let Some(definition) = self.current.take() else {
            return;
        };
        for voice in self.loops.iter_mut().chain(self.random.iter_mut()) {
            if let Some(mut voice) = voice.take() {
                if fade {
                    voice.fade(0, voice.volume, &definition.fade_out, now);
                    voice.update(now, backend);
                    self.retiring.push(voice);
                } else {
                    backend.remove(voice.handle);
                }
            }
        }
    }
    pub fn tick(
        &mut self,
        volume: i32,
        now: i64,
        draw: &mut impl FnMut() -> f64,
        backend: &mut impl Backend,
    ) -> Result<()> {
        if !self.retiring.is_empty() {
            self.retiring.retain_mut(|voice| {
                if voice.volume == 0 {
                    backend.remove(voice.handle);
                    false
                } else {
                    voice.update(now, backend);
                    true
                }
            });
            return Ok(());
        }
        let Some(definition) = &self.current else {
            return Ok(());
        };
        for (slot, id) in self.loops.iter_mut().zip(&definition.loops) {
            if let Some(voice) = slot.as_mut().filter(|v| v.present) {
                voice.adjust(volume, definition, now, backend);
            } else {
                *slot = backend
                    .play(*id, self.legacy, volume, -1)?
                    .map(|h| Voice::new(h, volume));
            }
        }
        for ((slot, deadline), group) in self
            .random
            .iter_mut()
            .zip(&mut self.deadlines)
            .zip(&definition.random)
        {
            if let Some(voice) = slot.as_mut().filter(|v| v.present) {
                voice.adjust(volume, definition, now, backend);
            } else if now >= *deadline && volume > 0 {
                let index = (draw() * group.sounds.len() as f64) as usize;
                let id = *group
                    .sounds
                    .get(index)
                    .ok_or(Error::Invalid("ambient random group empty"))?;
                *slot = backend
                    .play(id, self.legacy, volume, 0)?
                    .map(|h| Voice::new(h, volume));
                if let Some(voice) = slot {
                    voice.fade(volume, volume, &definition.fade_in, now);
                    voice.update(now, backend);
                }
                *deadline = now.wrapping_add(delay(group.min_ms, group.max_ms, draw()) as i64);
            }
        }
        Ok(())
    }
}
fn delay(min: i32, max: i32, random: f64) -> i32 {
    min.wrapping_add((random * max.wrapping_sub(min) as f64) as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Reader;
    #[derive(Default)]
    struct Mixer {
        voices: std::collections::HashMap<u64, i32>,
        next: u64,
        requests: Vec<(u16, bool, i32, i32)>,
        missing: bool,
    }
    impl Backend for Mixer {
        fn play(&mut self, id: u16, legacy: bool, volume: i32, loops: i32) -> Result<Option<u64>> {
            self.requests.push((id, legacy, volume, loops));
            if self.missing {
                return Ok(None);
            }
            self.next += 1;
            self.voices.insert(self.next, volume);
            Ok(Some(self.next))
        }
        fn volume(&self, voice: u64) -> Option<i32> {
            self.voices.get(&voice).copied()
        }
        fn set_volume(&mut self, voice: u64, volume: i32) {
            if let Some(v) = self.voices.get_mut(&voice) {
                *v = volume;
            }
        }
        fn remove(&mut self, voice: u64) {
            self.voices.remove(&voice);
        }
    }
    fn snapshot(r: &mut Reader<'_>, state: &State, mixer: &Mixer) {
        let id = r.u32().unwrap() as i32;
        assert_eq!(state.current_id().map_or(-1, |v| v as i32), id);
        assert_eq!(state.legacy(), r.u8().unwrap() != 0);
        assert_eq!(state.loop_present(0), r.u8().unwrap() != 0);
        assert_eq!(state.retiring_count(), r.u32().unwrap() as usize);
        let volume = r.u32().unwrap() as i32;
        let linked = r.u8().unwrap() != 0;
        assert_eq!(mixer.volume(1).is_some(), linked);
        if linked {
            assert_eq!(mixer.volume(1), Some(volume));
        }
    }
    #[test]
    fn original_switch_and_retiring_lifecycles() {
        let mut r = Reader::new(include_bytes!(
            "../tests/fixtures/ambient-lifecycle-jvm.bin"
        ));
        assert_eq!(r.u32().unwrap(), 16);
        for _ in 0..16 {
            let volume = r.u32().unwrap() as i32;
            let primed = r.u8().unwrap() != 0;
            let fade = r.u8().unwrap() != 0;
            let same = r.u8().unwrap() != 0;
            let mut state = State::default();
            let mut mixer = Mixer::default();
            let definition = Definition {
                id: 731,
                loops: vec![7],
                ..Definition::default()
            };
            state
                .start(definition.clone(), false, false, 0, &mut || 0.5, &mut mixer)
                .unwrap();
            state.tick(volume, 0, &mut || 0.5, &mut mixer).unwrap();
            if primed {
                state.loops[0]
                    .as_mut()
                    .unwrap()
                    .fade(volume, volume, &definition.fade_in, 0);
            }
            snapshot(&mut r, &state, &mixer);
            state
                .start(
                    if same {
                        definition
                    } else {
                        Definition {
                            id: 732,
                            ..Definition::default()
                        }
                    },
                    fade,
                    true,
                    0,
                    &mut || 0.5,
                    &mut mixer,
                )
                .unwrap();
            snapshot(&mut r, &state, &mixer);
            state.tick(volume, 1000, &mut || 0.5, &mut mixer).unwrap();
            snapshot(&mut r, &state, &mixer);
            state.tick(volume, 1000, &mut || 0.5, &mut mixer).unwrap();
            snapshot(&mut r, &state, &mixer);
        }
        assert_eq!(r.remaining(), 0);
    }
    #[test]
    fn random_deadlines_muting_missing_assets_and_finished_voice_delay() {
        let mut state = State::default();
        let mut mixer = Mixer::default();
        let definition = Definition {
            id: 7,
            random: vec![crate::ambient_sound::RandomGroup {
                min_ms: 100,
                max_ms: 300,
                sounds: vec![9, 11],
            }],
            ..Definition::default()
        };
        state
            .start(definition, false, true, 10, &mut || 0.5, &mut mixer)
            .unwrap();
        state.tick(127, 209, &mut || 0.75, &mut mixer).unwrap();
        assert!(mixer.requests.is_empty());
        state.tick(0, 210, &mut || 0.75, &mut mixer).unwrap();
        assert!(mixer.requests.is_empty());
        mixer.missing = true;
        state.tick(127, 210, &mut || 0.75, &mut mixer).unwrap();
        assert_eq!(mixer.requests, [(11, true, 127, 0)]);
        mixer.missing = false;
        state.tick(127, 459, &mut || 0.0, &mut mixer).unwrap();
        assert_eq!(mixer.requests.len(), 1);
        state.tick(127, 460, &mut || 0.0, &mut mixer).unwrap();
        assert_eq!(mixer.requests[1], (9, true, 127, 0));
        mixer.remove(1);
        state.tick(127, 560, &mut || 0.0, &mut mixer).unwrap();
        assert_eq!(mixer.requests.len(), 2);
        state.tick(127, 561, &mut || 0.0, &mut mixer).unwrap();
        assert_eq!(mixer.requests.len(), 3);
    }
}
