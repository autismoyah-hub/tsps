//! Patch and sample loading for MIDI instruments.
use crate::{
    audio_runtime::Sound, audio_synth::Effect, audio_vorbis, disk_cache::DiskCache,
    music_patch::Patch, Error, Result,
};
use std::{collections::HashMap, sync::Arc};
pub struct Instrument {
    pub patch: Patch,
    pub samples: [Option<Arc<Sound>>; 128],
}
#[derive(Default)]
pub struct Samples {
    decoded: HashMap<u32, Arc<Sound>>,
    setup: Option<Vec<u8>>,
}
impl Samples {
    pub fn load(&mut self, cache: &mut DiskCache, code: u32) -> Result<Option<Arc<Sound>>> {
        if code == 0 {
            return Ok(None);
        }
        // Bit one is the instrument's loop flag, not a different sample.
        let key = (code - 1) & !2;
        if let Some(sound) = self.decoded.get(&key) {
            return Ok(Some(sound.clone()));
        }
        let id = key >> 2;
        let vorbis = key & 1 != 0;
        let archive = if vorbis { 14 } else { 4 };
        let reference = cache.reference(archive)?;
        let (group, file) = if reference.groups.len() == 1 {
            (reference.groups[0].id, id)
        } else {
            let group = reference
                .groups
                .iter()
                .find(|g| g.id == id)
                .ok_or_else(|| Error::Io(format!("Missing instrument sample {archive}:{id}")))?;
            (
                id,
                if !vorbis && group.files.iter().any(|f| f.id == 1) {
                    1
                } else {
                    0
                },
            )
        };
        let data = cache.file(archive, group, file, [0; 4])?;
        let sound = if vorbis || file == 1 {
            let sample = if vorbis {
                if self.setup.is_none() {
                    self.setup = Some(cache.file(14, 0, 0, [0; 4])?);
                }
                audio_vorbis::decode(self.setup.as_ref().unwrap(), &data)?
            } else {
                audio_vorbis::decode_embedded(&data)?
            };
            Sound {
                rate: sample.rate,
                ping_pong: sample.ping_pong,
                legacy: false,
                samples: sample.samples,
                loop_start: sample.loop_start,
                loop_end: sample.loop_end,
                delay: 0,
            }
        } else {
            let effect = Effect::decode(&data)?;
            Sound {
                rate: 22050,
                ping_pong: false,
                legacy: false,
                samples: effect.synthesize(false)?,
                loop_start: effect.loop_start as usize * 22050 / 1000,
                loop_end: effect.loop_end as usize * 22050 / 1000,
                delay: 0,
            }
        };
        let sound = Arc::new(sound);
        self.decoded.insert(key, sound.clone());
        Ok(Some(sound))
    }
    pub fn instrument(
        &mut self,
        cache: &mut DiskCache,
        id: u32,
        notes: &[bool; 128],
    ) -> Result<Instrument> {
        let patch = Patch::decode(&cache.file(15, id, 0, [0; 4])?)?;
        let mut samples = std::array::from_fn(|_| None);
        for key in 0..128 {
            if notes[key] {
                samples[key] = self.load(cache, patch.samples[key])?;
            }
        }
        Ok(Instrument { patch, samples })
    }
}
