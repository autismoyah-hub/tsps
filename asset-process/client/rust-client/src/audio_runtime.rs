//! Delayed effect requests and loop-region PCM mixing.
use crate::{
    audio_synth::Effect, device_options::DeviceOptions, disk_cache::DiskCache,
    interface_script::ints, vm::Stacks, Result,
};
use std::{collections::HashMap, sync::{Arc, Mutex}};
pub struct Sound {
    pub rate: u32,
    pub ping_pong: bool,
    pub legacy: bool,
    pub samples: Vec<i16>,
    pub loop_start: usize,
    pub loop_end: usize,
    pub delay: i32,
}
struct Pending {
    id: u32,
    loops: i32,
    delay: i32,
    sound: Option<Arc<Sound>>,
    area: Option<crate::area_sound::Request>,
    world: Option<u16>,
    cross_world: bool,
}
struct Voice {
    handle: Option<u64>,
    sound: Arc<Sound>,
    position: i64,
    step: i64,
    loops: i32,
    volume: i32,
}
#[derive(Default)]
pub struct Audio {
    pub ambient: crate::ambient_runtime::State,
    next_voice: u64,
    ambient_epoch: Option<std::time::Instant>,
    pub music: crate::music_runtime::Music,
    last_music_volume: Option<i32>,
    pending: Vec<Pending>,
    voices: Arc<Mutex<Vec<Voice>>>,
    cache: HashMap<(u32, bool), Arc<Sound>>,
    listener: [i32; 2],
}
impl Audio {
    /// GameGameEngine.ay retires ambient streams before returning to the title.
    pub fn prepare_logout(&mut self) {
        let now = self.ambient_now();
        self.prepare_logout_at(now);
    }
    fn prepare_logout_at(&mut self, now: i64) {
        let mut state = std::mem::take(&mut self.ambient);
        state.stop(true, now, &mut AmbientBackend { audio: self, cache: None });
        self.ambient = state;
        self.pending.clear();
        self.voices.lock().expect("effect mixer lock").retain(|v| v.handle.is_some());
        self.music = Default::default();
        self.last_music_volume = None;
    }
    fn ambient_now(&mut self) -> i64 {
        self.ambient_epoch.get_or_insert_with(std::time::Instant::now).elapsed().as_millis() as i64
    }
    pub fn ambient_start(&mut self, definition: crate::ambient_sound::Definition, fade: bool, legacy: bool, cache: &mut DiskCache) -> Result<()> {
        let now = self.ambient_now();
        let mut state = std::mem::take(&mut self.ambient);
        let result = state.start(definition, fade, legacy, now, &mut ambient_random, &mut AmbientBackend { audio: self, cache: Some(cache) });
        self.ambient = state;
        result
    }
    pub fn ambient_stop(&mut self, fade: bool) {
        let now = self.ambient_now();
        let mut state = std::mem::take(&mut self.ambient);
        state.stop(fade, now, &mut AmbientBackend { audio: self, cache: None });
        self.ambient = state;
    }
    fn tick_ambient(&mut self, cache: &mut DiskCache, options: &DeviceOptions) -> Result<()> {
        let now = self.ambient_now();
        let volume = options.get_game(8)?;
        let mut state = std::mem::take(&mut self.ambient);
        let result = state.tick(volume, now, &mut ambient_random, &mut AmbientBackend { audio: self, cache: Some(cache) });
        self.ambient = state;
        result
    }
    pub fn play(&mut self, sound: Arc<Sound>, loops: i32, volume: i32) {
        let loops = if sound.loop_start < sound.loop_end && sound.loop_end <= sound.samples.len() {
            loops
        } else {
            0
        };
        let step = (sound.rate as i64 * 256 / 22050).max(1);
        self.voices.lock().expect("effect mixer lock").push(Voice {
            handle: None,
            sound,
            position: 0,
            step,
            loops,
            volume,
        });
    }
    /// Read-only enqueue identities for runtime diagnostics and parity hosts.
    pub fn pending_effects(&self) -> impl Iterator<Item = (u32, Option<u16>)> + '_ {
        self.pending.iter().map(|pending| (pending.id, pending.world))
    }
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
    pub fn active(&self) -> bool {
        !self.voices.lock().expect("effect mixer lock").is_empty() || self.music.active()
    }
    pub fn request(&mut self, id: u32, loops: i32, delay: i32, volume: i32) {
        if volume != 0 && loops != 0 && self.pending.len() < 50 {
            self.pending.push(Pending {
                id,
                loops,
                delay,
                sound: None,
                area: None,
                world: None,
                cross_world: false,
            });
        }
    }
    pub fn set_listener(&mut self, fine: [i32; 2]) {
        self.listener = fine;
    }
    pub fn request_area(
        &mut self,
        request: crate::area_sound::Request,
        options: &DeviceOptions,
    ) -> Result<()> {
        self.request_area_from(request, options, None)
    }
    pub fn request_world_area(
        &mut self,
        world: u16,
        request: crate::area_sound::Request,
        options: &DeviceOptions,
    ) -> Result<()> {
        self.request_area_from(request, options, Some(world))
    }
    fn request_area_from(
        &mut self,
        request: crate::area_sound::Request,
        options: &DeviceOptions,
        world: Option<u16>,
    ) -> Result<()> {
        if request.admitted(
            options.get_game(8)?,
            options.get_game(9)?,
            self.pending.len(),
        ) {
            self.pending.push(Pending {
                id: request.id as u32,
                loops: request.loops as i32,
                delay: request.delay as i32,
                sound: None,
                area: Some(request),
                world,
                cross_world: false,
            });
        }
        Ok(())
    }
    pub fn request_sequence(&mut self, request: crate::area_sound::Request, options: &DeviceOptions) -> Result<()> {
        self.request_sequence_from(None, false, request, options)
    }
    /// zy.ag / rr.co retain local coordinates, view identity and sequence flag19.
    /// Placement and cross-view audibility are resolved when the sound plays.
    pub fn request_world_sequence(&mut self, world: u16, cross_world: bool, request: crate::area_sound::Request, options: &DeviceOptions) -> Result<()> {
        self.request_sequence_from(Some(world), cross_world, request, options)
    }
    fn request_sequence_from(
        &mut self,
        world: Option<u16>,
        cross_world: bool,
        request: crate::area_sound::Request,
        options: &DeviceOptions,
    ) -> Result<()> {
        if crate::sequence_sound::admitted(
            &request,
            options.get_game(8)?,
            options.get_game(9)?,
            self.pending.len(),
        ) {
            self.pending.push(Pending {
                id: request.id as u32,
                loops: request.loops as i32,
                delay: 0,
                sound: None,
                area: Some(request),
                world,
                cross_world,
            });
        }
        Ok(())
    }
    pub fn execute(
        &mut self,
        opcode: u16,
        s: &mut Stacks,
        options: &DeviceOptions,
    ) -> Result<bool> {
        if let Some(request) = crate::music_request::Request::script(opcode, s)? {
            request.apply(&mut self.music, options.get_game(7)?);
            return Ok(true);
        }
        if opcode != 3200 {
            return Ok(false);
        }
        let [id, loops, delay] = ints(s)?;
        self.request(id as u32, loops, delay, options.get_game(8)?);
        Ok(true)
    }
    pub fn tick(&mut self, cache: &mut DiskCache, options: &DeviceOptions) -> Result<()> {
        self.tick_in_worlds(cache, options, None)
    }
    pub fn tick_in_worlds(
        &mut self,
        cache: &mut DiskCache,
        options: &DeviceOptions,
        worlds: Option<&crate::world_audio::Context<'_>>,
    ) -> Result<()> {
        let music_volume = options.get_game(7)?;
        if self
            .last_music_volume
            .is_some_and(|old| old != music_volume)
        {
            self.music.volume_changed(music_volume);
        }
        self.last_music_volume = Some(music_volume);
        self.music.tick(cache)?;
        self.tick_ambient(cache, options)?;
        let volume = options.get_game(8)?;
        let area_volume = options.get_game(9)?;
        for pending in &mut self.pending {
            let volume = pending
                .area
                .as_ref()
                .map_or(volume, |area| match pending.world {
                    Some(world) => worlds.map_or(0, |context| {
                        context.volume(area, world, self.listener, volume, area_volume, pending.cross_world)
                    }),
                    None => area.volume(self.listener, volume, area_volume),
                });
            pending.delay = pending.delay.wrapping_sub(1);
            if pending.delay < -10 {
                continue;
            }
            if pending.sound.is_none() {
                let key = (pending.id, options.option22);
                let sound = if let Some(sound) = self.cache.get(&key) {
                    sound.clone()
                } else {
                    let Some(group) = cache
                        .reference(4)?
                        .groups
                        .iter()
                        .find(|g| g.id == pending.id)
                    else {
                        continue;
                    };
                    let file = u32::from(group.files.iter().any(|f| f.id == 1));
                    let bytes = match cache.file(4, pending.id, file, [0; 4]) {
                        Ok(bytes) => bytes,
                        Err(crate::Error::PendingAsset { .. }) => continue,
                        Err(error) => return Err(error),
                    };
                    let sound = if file == 1 {
                        let sample = crate::audio_vorbis::decode_embedded(
                            &bytes,
                        )?;
                        Arc::new(Sound {
                            rate: sample.rate,
                            ping_pong: sample.ping_pong,
                            legacy: false,
                            delay: if sample.loop_start < sample.loop_end {
                                (sample.loop_start / 20) as i32
                            } else {
                                0
                            },
                            samples: sample.samples,
                            loop_start: sample.loop_start,
                            loop_end: sample.loop_end,
                        })
                    } else {
                        let mut effect = Effect::decode(&bytes)?;
                        let delay = effect.trim_delay();
                        Arc::new(Sound {
                            rate: 22050,
                            ping_pong: false,
                            legacy: options.option22,
                            samples: effect.synthesize(options.option22)?,
                            loop_start: effect.loop_start as usize * 22050 / 1000,
                            loop_end: effect.loop_end as usize * 22050 / 1000,
                            delay,
                        })
                    };
                    if self.cache.len() >= 128 {
                        self.cache.clear();
                    }
                    self.cache.insert(key, sound.clone());
                    sound
                };
                pending.delay = pending.delay.wrapping_add(sound.delay);
                pending.sound = Some(sound);
            }
            if pending.delay < 0 {
                if volume > 0 {
                    let sound = pending.sound.as_ref().unwrap().clone();
                    let loops = if sound.loop_start < sound.loop_end
                        && sound.loop_end <= sound.samples.len()
                    {
                        pending.loops.wrapping_sub(1)
                    } else {
                        0
                    };
                    self.voices.lock().expect("effect mixer lock").push(Voice {
                        handle: None,
                        step: (sound.rate as i64 * 256 / 22050).max(1),
                        sound,
                        position: 0,
                        loops,
                        volume,
                    });
                }
                pending.delay = -100;
            }
        }
        self.pending.retain(|p| p.delay >= -10);
        Ok(())
    }
    pub fn mix(&mut self, count: usize) -> Vec<i16> {
        self.mix_values(count)
            .into_iter()
            .map(|v| (v >> 8).clamp(-32768, 32767) as i16)
            .collect()
    }
    pub fn mix_stereo(&mut self, count: usize) -> Vec<i16> {
        self.mixer().mix_stereo(count)
    }
    fn mix_values(&mut self, count: usize) -> Vec<i64> {
        mix_voices(&mut self.voices.lock().expect("effect mixer lock"), count)
    }
    pub fn mixer(&self) -> Mixer {
        Mixer { voices: self.voices.clone(), music: self.music.mixer() }
    }

}
fn ambient_random() -> f64 {
    // Java Random's 48-bit LCG and 53-bit double construction; each client owns its stream.
    thread_local! { static SEED: std::cell::Cell<u64> = std::cell::Cell::new(crate::market::now() as u64 ^ 0x5deece66d); }
    SEED.with(|seed| {
        let a = seed.get().wrapping_mul(0x5deece66d).wrapping_add(11) & ((1 << 48) - 1);
        let b = a.wrapping_mul(0x5deece66d).wrapping_add(11) & ((1 << 48) - 1);
        seed.set(b);
        (((a >> 22) << 27) + (b >> 21)) as f64 / (1u64 << 53) as f64
    })
}
struct AmbientBackend<'a> { audio: &'a mut Audio, cache: Option<&'a mut DiskCache> }
impl crate::ambient_runtime::Backend for AmbientBackend<'_> {
    fn play(&mut self, id: u16, legacy: bool, volume: i32, loops: i32) -> Result<Option<u64>> {
        let cache = self.cache.as_deref_mut().ok_or(crate::Error::Invalid("ambient control cannot load samples"))?;
        let Some(group) = cache.reference(4)?.groups.iter().find(|g| g.id == id as u32) else { return Ok(None); };
        // GameIndexDataBase.co is the sparse file-array capacity, not populated count.
        let Some(file) = group.files.iter().map(|f| f.id).max().filter(|f| *f <= 1) else { return Ok(None); };
        let bytes = match cache.file(4, id as u32, file as u32, [0; 4]) {
            Ok(bytes) => bytes,
            Err(crate::Error::PendingAsset { .. }) => return Ok(None),
            Err(error) => return Err(error),
        };
        let sound = if file == 1 {
            let sample = crate::audio_vorbis::decode_embedded(&bytes)?;
            Sound { rate: sample.rate, ping_pong: sample.ping_pong, legacy: false, delay: 0, samples: sample.samples, loop_start: sample.loop_start, loop_end: sample.loop_end }
        } else {
            // rw plays the untrimmed sample, unlike delayed SYNTH_SOUND effects.
            let effect = Effect::decode(&bytes)?;
            Sound { rate: 22050, ping_pong: false, legacy, delay: 0, samples: effect.synthesize(legacy)?, loop_start: effect.loop_start as usize * 22050 / 1000, loop_end: effect.loop_end as usize * 22050 / 1000 }
        };
        if sound.samples.is_empty() { return Ok(None); }
        self.audio.next_voice = self.audio.next_voice.wrapping_add(1);
        let handle = self.audio.next_voice;
        let loops = if sound.loop_start < sound.loop_end && sound.loop_end <= sound.samples.len() { loops } else { 0 };
        self.audio.voices.lock().expect("effect mixer lock").push(Voice {
            handle: Some(handle), step: (sound.rate as i64 * 256 / 22050).max(1), sound: Arc::new(sound), position: 0, loops, volume,
        });
        Ok(Some(handle))
    }
    fn volume(&self, handle: u64) -> Option<i32> {
        self.audio.voices.lock().expect("effect mixer lock").iter().find(|v| v.handle == Some(handle)).map(|v| v.volume)
    }
    fn set_volume(&mut self, handle: u64, volume: i32) {
        if let Some(voice) = self.audio.voices.lock().expect("effect mixer lock").iter_mut().find(|v| v.handle == Some(handle)) { voice.volume = volume; }
    }
    fn remove(&mut self, handle: u64) {
        self.audio.voices.lock().expect("effect mixer lock").retain(|v| v.handle != Some(handle));
    }
}
/// Cloneable PCM-only handle. It performs no cache access or request preparation.
#[derive(Clone)]
pub struct Mixer {
    voices: Arc<Mutex<Vec<Voice>>>,
    music: crate::music_runtime::Mixer,
}
impl Mixer {
    pub fn mix_stereo(&self, count: usize) -> Vec<i16> {
        let effects = mix_voices(&mut self.voices.lock().expect("effect mixer lock"), count);
        let mut music = vec![0i32; count * 2];
        self.music.mix(&mut music, true);
        music.into_iter().enumerate()
            .map(|(i, v)| ((v as i64 + effects[i / 2]) >> 8).clamp(-32768, 32767) as i16)
            .collect()
    }
}
fn mix_voices(voices: &mut Vec<Voice>, count: usize) -> Vec<i64> {
        let mut result = vec![0i64; count];
        for voice in voices.iter_mut() {
            for sample in &mut result {
                let start = voice.sound.loop_start as i64 * 256;
                let end = voice.sound.loop_end as i64 * 256;
                while voice.loops != 0
                    && ((voice.step > 0 && voice.position >= end)
                        || (voice.step < 0 && voice.position < start))
                {
                    if voice.sound.ping_pong {
                        let boundary = if voice.step > 0 { end } else { start };
                        voice.position = 2 * boundary - 1 - voice.position;
                        voice.step = -voice.step;
                    } else if voice.step > 0 {
                        voice.position -= end - start;
                    } else {
                        voice.position += end - start;
                    }
                    if voice.loops > 0 {
                        voice.loops -= 1;
                    }
                }
                if voice.position < 0 || voice.position >= voice.sound.samples.len() as i64 * 256 {
                    break;
                }
                let index = (voice.position >> 8) as usize;
                let fraction = voice.position & 255;
                let current = voice.sound.samples[index] as i64;
                let value = if fraction == 0 && voice.step.abs() == 256 {
                    current << 8
                } else if voice.step >= 0 {
                    let next = if voice.loops != 0 && index + 1 >= voice.sound.loop_end {
                        if voice.sound.ping_pong {
                            current
                        } else {
                            voice.sound.samples[voice.sound.loop_start] as i64
                        }
                    } else {
                        voice.sound.samples.get(index + 1).copied().unwrap_or(0) as i64
                    };
                    (current << 8) + (next - current) * fraction
                } else {
                    let previous = if voice.loops != 0 && index <= voice.sound.loop_start {
                        if voice.sound.ping_pong {
                            current
                        } else {
                            voice.sound.samples[voice.sound.loop_end - 1] as i64
                        }
                    } else if index == 0 {
                        0
                    } else {
                        voice.sound.samples[index - 1] as i64
                    };
                    (previous << 8) + (current - previous) * fraction
                };
                // The original stereo fast path scales legacy bytes; its
                // interpolating path truncates to a sample before applying gain.
                let legacy_scale = if fraction == 0 && voice.step == 256 && voice.sound.legacy {
                    256
                } else {
                    1
                };
                *sample += (value >> 8) * voice.volume as i64 * legacy_scale;
                voice.position += voice.step;
            }
        }
        voices.retain(|v| {
            (v.position >= 0 && v.position < v.sound.samples.len() as i64 * 256) || v.loops != 0
        });
        result
    }
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn logout_keeps_pcm_handle_until_ambient_fade_finishes() {
        use crate::ambient_runtime::Backend;
        struct Prepared<'a>(&'a mut Audio);
        impl Backend for Prepared<'_> {
            fn play(&mut self, _: u16, _: bool, volume: i32, loops: i32) -> Result<Option<u64>> {
                self.0.play(Arc::new(Sound { rate: 22050, ping_pong: false, legacy: false, samples: vec![256; 4], loop_start: 0, loop_end: 4, delay: 0 }), loops, volume);
                self.0.voices.lock().unwrap().last_mut().unwrap().handle = Some(1);
                Ok(Some(1))
            }
            fn volume(&self, handle: u64) -> Option<i32> {
                self.0.voices.lock().unwrap().iter().find(|v| v.handle == Some(handle)).map(|v| v.volume)
            }
            fn set_volume(&mut self, handle: u64, volume: i32) {
                AmbientBackend { audio: self.0, cache: None }.set_volume(handle, volume);
            }
            fn remove(&mut self, handle: u64) {
                AmbientBackend { audio: self.0, cache: None }.remove(handle);
            }
        }
        let mut audio = Audio::default();
        let mut state = crate::ambient_runtime::State::default();
        state.start(crate::ambient_sound::Definition { id: 1, loops: vec![7], ..Default::default() }, false, false, 0, &mut || 0.0, &mut Prepared(&mut audio)).unwrap();
        state.tick(64, 0, &mut || 0.0, &mut Prepared(&mut audio)).unwrap();
        state.tick(127, 0, &mut || 0.0, &mut Prepared(&mut audio)).unwrap();
        audio.ambient = state;
        audio.request(9, 1, 0, 127);
        audio.play(Arc::new(Sound { rate: 22050, ping_pong: false, legacy: false, samples: vec![256; 4], loop_start: 0, loop_end: 4, delay: 0 }), -1, 127);
        let mixer = audio.mixer();
        audio.prepare_logout_at(0);
        assert_eq!(audio.ambient.current_id(), None);
        assert_eq!(audio.ambient.retiring_count(), 1);
        assert_eq!(audio.pending_count(), 0);
        assert_eq!(audio.voices.lock().unwrap().len(), 1);
        assert_eq!(mixer.mix_stereo(1), [127, 127]);
        let mut state = std::mem::take(&mut audio.ambient);
        state.tick(127, 150, &mut || 0.0, &mut Prepared(&mut audio)).unwrap();
        assert_eq!(mixer.mix_stereo(1), [63, 63]);
        state.tick(127, 300, &mut || 0.0, &mut Prepared(&mut audio)).unwrap();
        assert_eq!(mixer.mix_stereo(1), [0, 0]);
        assert_eq!(state.retiring_count(), 1);
        state.tick(127, 301, &mut || 0.0, &mut Prepared(&mut audio)).unwrap();
        assert_eq!(state.retiring_count(), 0);
        assert!(!audio.active());
    }
    #[test]
    fn loops_region_then_tail_and_preserves_mixer_precision() {
        let sound = Arc::new(Sound {
            rate: 22050,
            ping_pong: false,
            legacy: false,
            samples: vec![256, 512, 768, 1024],
            loop_start: 1,
            loop_end: 3,
            delay: 0,
        });
        let mut audio = Audio::default();
        audio.voices.lock().expect("effect mixer lock").push(Voice {
            handle: None,
            sound,
            position: 0,
            step: 256,
            loops: 2,
            volume: 1,
        });
        assert_eq!(audio.mix(10), [1, 2, 3, 2, 3, 2, 3, 4, 0, 0]);
        assert!(!audio.active());
    }
    #[test]
    fn delayed_queue_has_original_capacity() {
        let mut audio = Audio::default();
        audio.request(1, 0, 0, 127);
        audio.request(1, 1, 0, 0);
        assert_eq!(audio.pending_count(), 0);
        for _ in 0..60 {
            audio.request(1, 1, 0, 127);
        }
        assert_eq!(audio.pending_count(), 50);
    }
}

#[cfg(test)]
#[path = "../tests/support/sequence_world_sound.rs"]
mod sequence_world_tests;
