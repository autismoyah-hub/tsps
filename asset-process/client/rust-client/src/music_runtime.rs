//! Revision 239 music request queue and transition tasks.
use crate::{
    disk_cache::DiskCache,
    music_player::Player,
    music_samples::Samples,
    music_track::Track,
    music_transition::{Delay, Fade},
    Error, Result,
};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, Mutex},
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Song {
    pub archive: u8,
    pub id: u32,
    pub volume: i32,
    pub looping: bool,
}
struct Request {
    song: Song,
    current: f32,
    slot: Option<usize>,
    track: Option<Track>,
    started: bool,
    marked: bool,
    fading: bool,
}
struct Task {
    kind: Kind,
    next: Option<Box<Task>>,
}
enum Kind {
    Add(Vec<u64>),
    Load,
    Start,
    Fade(Option<u64>, Fade),
    Delay(Delay),
    Clear(Vec<u64>),
    Swap,
    Concurrent(Vec<Task>),
}
impl Task {
    fn new(kind: Kind) -> Self {
        Self { kind, next: None }
    }
    fn then(mut self, next: Task) -> Self {
        self.next = Some(Box::new(next));
        self
    }
    fn delay(duration: i32, next: Task) -> Self {
        Self::new(Kind::Delay(Delay {
            elapsed: 0,
            duration,
        }))
        .then(next)
    }
    fn fade(id: Option<u64>, duration: i32, fade_in: bool) -> Self {
        Self::new(Kind::Fade(id, Fade { duration, fade_in }))
    }
    fn tick(&mut self, music: &mut Music, cache: &mut DiskCache) -> Result<bool> {
        Ok(match &mut self.kind {
            Kind::Add(ids) => {
                for &id in ids.iter() {
                    music.assign(id);
                }
                true
            }
            Kind::Load => {
                match music.load(cache) {
                    Ok(()) => true,
                    // vu.ag retains LoadSongTask while archive data is unavailable.
                    Err(Error::PendingAsset { .. }) => false,
                    Err(error) => return Err(error),
                }
            }
            Kind::Start => {
                for &id in &music.active {
                    let request = music.requests.get_mut(&id).unwrap();
                    if !request.started {
                        if let Some(slot) = request.slot {
                            let mut players = music.players.lock().expect("music mixer lock");
                            if let Some(track) = request.track.take() {
                                players[slot].install(&track.midi, request.song.looping)?;
                            }
                            players[slot].synth.master = 0;
                            request.started = true;
                        }
                    }
                }
                true
            }
            Kind::Fade(id, fade) => {
                if let Some(request) = id.and_then(|id| music.requests.get_mut(&id)) {
                    if let Some(slot) = request.slot {
                        let done = fade.tick(
                            &mut request.current,
                            request.song.volume,
                            music.players.lock().expect("music mixer lock")[slot].playing(),
                        );
                        request.fading = !done;
                        if !done {
                            music.players.lock().expect("music mixer lock")[slot].synth.master = request.current as i32;
                        }
                        done
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            Kind::Delay(delay) => delay.tick(),
            Kind::Clear(ids) => {
                for &id in ids.iter() {
                    music.clear(id);
                }
                true
            }
            Kind::Swap => {
                if music.active.len() > 1
                    && music.active.iter().take(2).all(|id| {
                        music.requests[id]
                            .slot
                            .is_some_and(|s| music.players.lock().expect("music mixer lock")[s].playing())
                    })
                {
                    music.active.swap(0, 1);
                }
                true
            }
            Kind::Concurrent(tasks) => {
                let mut index = 0;
                while index < tasks.len() {
                    if tasks[index].tick(music, cache)? {
                        let mut task = tasks.remove(index);
                        if let Some(next) = task.next.take() {
                            tasks.push(*next);
                        }
                    } else {
                        index += 1;
                    }
                }
                tasks.is_empty()
            }
        })
    }
}
/// PCM streams shared with the output worker; cache/task state stays on the game thread.
#[derive(Clone)]
pub struct Mixer {
    players: Arc<Mutex<[Player; 3]>>,
}
impl Mixer {
    pub fn mix(&self, output: &mut [i32], stereo: bool) {
        for player in self.players.lock().expect("music mixer lock").iter_mut() {
            player.mix(output, stereo);
        }
    }
}
pub struct Music {
    players: Arc<Mutex<[Player; 3]>>,
    users: [i32; 3],
    samples: Samples,
    requests: BTreeMap<u64, Request>,
    active: Vec<u64>,
    remembered: Vec<Song>,
    tasks: VecDeque<Task>,
    next_id: u64,
    timings: [i32; 4],
    pub jingle: bool,
}
impl Default for Music {
    fn default() -> Self {
        let players = std::array::from_fn(|_| {
            let mut p = Player::empty(22050);
            p.synth.set_program(9, 128);
            p
        });
        Self {
            players: Arc::new(Mutex::new(players)),
            users: [0; 3],
            samples: Samples::default(),
            requests: BTreeMap::new(),
            active: Vec::new(),
            remembered: Vec::new(),
            tasks: VecDeque::new(),
            next_id: 0,
            timings: [0; 4],
            jingle: false,
        }
    }
}
impl Music {
    pub fn title(&mut self, cache: &mut DiskCache, disabled: bool) -> Result<()> {
        self.stop(0, 0);
        if !disabled {
            let id = cache
                .reference(6)?
                .groups
                .iter()
                .find(|g| g.name_hash == Some(crate::disk_cache::name_hash("scape main")))
                .ok_or(Error::Invalid("Title music track missing"))?
                .id;
            self.schedule(
                vec![Song {
                    archive: 6,
                    id,
                    volume: 255,
                    looping: false,
                }],
                [0, 0, 0, 100],
                false,
            );
        }
        Ok(())
    }
    pub fn volume_changed(&mut self, volume: i32) {
        if volume == 0 {
            self.stop(0, 0);
            self.jingle = false;
            return;
        }
        for song in &mut self.remembered {
            song.volume = volume;
        }
        for id in &self.active {
            self.requests.get_mut(id).unwrap().song.volume = volume;
        }
        if let Some(id) = self.active.first() {
            let request = self.requests.get_mut(id).unwrap();
            if let Some(slot) = request.slot {
                if self.players.lock().expect("music mixer lock")[slot].playing() && !request.fading {
                    self.players.lock().expect("music mixer lock")[slot].synth.master = volume;
                    request.current = volume as f32;
                }
            }
        }
    }
    pub fn active(&self) -> bool {
        self.players.lock().expect("music mixer lock").iter().any(Player::active) || !self.tasks.is_empty()
    }
    fn waiting_for_music(&self) -> bool {
        !self.tasks.is_empty()
            || self
                .active
                .first()
                .and_then(|id| self.requests[id].slot)
                .is_some_and(|slot| self.players.lock().expect("music mixer lock")[slot].playing())
    }
    pub fn pending_tasks(&self) -> usize {
        self.tasks.len()
    }
    fn clear(&mut self, id: u64) {
        if !self.active.contains(&id) {
            return;
        }
        if let Some(slot) = self.requests[&id].slot {
            if self.users[slot] > 0 {
                self.users[slot] -= 1;
            }
            if self.users[slot] == 0 {
                self.players.lock().expect("music mixer lock")[slot].stop();
                self.players.lock().expect("music mixer lock")[slot].synth.instruments.clear();
                self.players.lock().expect("music mixer lock")[slot].synth.master = 0;
            }
        }
        self.active.retain(|&active| active != id);
    }
    fn clear_all(&mut self) {
        for &id in &self.active {
            if let Some(slot) = self.requests[&id].slot {
                self.players.lock().expect("music mixer lock")[slot].stop();
                self.players.lock().expect("music mixer lock")[slot].synth.instruments.clear();
                self.players.lock().expect("music mixer lock")[slot].synth.master = 0;
                self.users[slot] = 0;
            }
        }
        self.active.clear();
    }
    fn assign(&mut self, id: u64) {
        let mut players = self.players.lock().expect("music mixer lock");
        let mut selected = 0;
        for slot in 1..3 {
            if self.users[selected] > self.users[slot]
                || players[slot].synth.master == 0 && players[slot].playing()
            {
                selected = slot;
            }
        }
        self.users[selected] += 1;
        if players[selected].synth.master == 0 && players[selected].playing() {
            players[selected].stop();
            players[selected].synth.instruments.clear();
            players[selected].synth.master = 0;
        }
        self.requests.get_mut(&id).unwrap().slot = Some(selected);
        self.active.push(id);
    }
    fn load(&mut self, cache: &mut DiskCache) -> Result<()> {
        for &id in &self.active {
            let request = self.requests.get_mut(&id).unwrap();
            if request.started || request.track.is_some() {
                continue;
            }
            let slot = request
                .slot
                .ok_or(Error::Invalid("Music request has no stream"))?;
            if self.users[slot] > 1 && !self.players.lock().expect("music mixer lock")[slot].synth.instruments.is_empty() {
                return Err(Error::Invalid(
                    "Music stream already loading another request",
                ));
            }
            let track =
                Track::decode(&cache.file(request.song.archive, request.song.id, 0, [0; 4])?)?;
            for (&patch, notes) in &track.instruments {
                let mut instrument = self.samples.instrument(cache, patch as u32, &notes.notes)?;
                if let Some(previous) = self.players.lock().expect("music mixer lock")[slot].synth.instruments.get(&patch) {
                    for key in 0..128 {
                        if instrument.samples[key].is_none() {
                            instrument.samples[key] = previous.samples[key].clone();
                        }
                    }
                }
                self.players.lock().expect("music mixer lock")[slot]
                    .synth
                    .instruments
                    .insert(patch, Arc::new(instrument));
            }
            request.track = Some(track);
        }
        Ok(())
    }
    pub fn schedule(&mut self, songs: Vec<Song>, timings: [i32; 4], jingle: bool) {
        if songs.is_empty() {
            return;
        }
        self.tasks.clear();
        if jingle {
            self.clear_all();
        } else {
            let old = self.active.clone();
            for id in old {
                if self.requests[&id].marked {
                    let slot = self.requests[&id].slot;
                    self.clear(id);
                    // ii.ab(false) resets an interrupted request's player even
                    // when another request still shares that player.
                    if let Some(slot) = slot {
                        self.players.lock().expect("music mixer lock")[slot].stop();
                        self.players.lock().expect("music mixer lock")[slot].synth.instruments.clear();
                        self.players.lock().expect("music mixer lock")[slot].synth.master = 0;
                    }
                } else {
                    self.requests.get_mut(&id).unwrap().marked = true;
                }
            }
            self.remembered.clear();
        }
        let mut pending = Vec::new();
        for song in songs {
            if !jingle {
                self.remembered.push(song.clone());
            }
            let id = self.next_id;
            self.next_id += 1;
            self.requests.insert(
                id,
                Request {
                    song,
                    current: 0.0,
                    slot: None,
                    track: None,
                    started: false,
                    marked: false,
                    fading: false,
                },
            );
            pending.push(id);
        }
        self.timings = timings;
        let fade_in =
            Task::new(Kind::Start).then(Task::fade(pending.first().copied(), timings[3], true));
        self.tasks.push_back(Task::new(Kind::Add(pending)));
        self.tasks.push_back(Task::new(Kind::Load));
        if self.active.is_empty() {
            self.tasks.push_back(Task::new(Kind::Delay(Delay {
                elapsed: 0,
                duration: timings[2],
            })));
            self.tasks
                .push_back(Task::new(Kind::Concurrent(vec![fade_in])));
        } else {
            let fade_out = Task::fade(self.active.first().copied(), timings[1], false)
                .then(Task::new(Kind::Clear(self.active.clone())));
            self.tasks.push_back(Task::new(Kind::Concurrent(vec![
                Task::delay(timings[2], Task::new(Kind::Concurrent(vec![fade_in]))),
                Task::delay(timings[0], fade_out),
            ])));
        }
    }
    pub fn stop(&mut self, delay: i32, duration: i32) {
        self.timings = [delay, duration, 0, 0];
        self.remembered.clear();
        self.tasks.clear();
        if self.active.is_empty() || (delay == 0 && duration == 0) {
            self.clear_all();
            return;
        }
        self.tasks.push_back(Task::new(Kind::Delay(Delay {
            elapsed: 0,
            duration: delay,
        })));
        self.tasks
            .push_back(Task::fade(self.active.first().copied(), duration, false));
        self.tasks
            .push_back(Task::new(Kind::Clear(self.active.clone())));
    }
    pub fn swap(&mut self, timings: [i32; 4]) {
        if self.active.len() < 2
            || !self.active.iter().take(2).all(|id| {
                self.requests[id]
                    .slot
                    .is_some_and(|slot| self.players.lock().expect("music mixer lock")[slot].playing())
            })
        {
            return;
        }
        self.timings = timings;
        self.tasks.push_back(Task::new(Kind::Swap));
        self.tasks.push_back(Task::new(Kind::Concurrent(vec![
            Task::delay(
                timings[2],
                Task::fade(Some(self.active[1]), timings[3], true),
            ),
            Task::delay(
                timings[0],
                Task::fade(Some(self.active[0]), timings[1], false),
            ),
        ])));
        if self.remembered.len() > 1 {
            let previous = self.remembered[0].clone();
            self.remembered[0] = self.requests[&self.active[1]].song.clone();
            self.remembered[1] = previous;
        }
    }
    pub fn songs(&mut self, ids: &[i32], timings: [i32; 4], volume: i32) {
        let Some(&first) = ids.first() else { return };
        if first == -1 {
            if !self.jingle {
                self.stop(0, 0);
            }
            return;
        }
        if volume == 0
            || self
                .remembered
                .first()
                .is_some_and(|song| song.id == first as u32)
        {
            return;
        }
        let songs = ids
            .iter()
            .filter(|&&id| id >= 0)
            .map(|&id| Song {
                archive: 6,
                id: id as u32,
                volume,
                looping: false,
            })
            .collect::<Vec<_>>();
        if self.jingle {
            self.remembered = songs;
            self.timings = timings;
        } else {
            self.schedule(songs, timings, false);
        }
    }
    pub fn play_jingle(&mut self, id: i32, volume: i32) {
        if volume == 0 || id == -1 {
            return;
        }
        self.schedule(
            vec![Song {
                archive: 11,
                id: id as u32,
                volume,
                looping: false,
            }],
            [0; 4],
            true,
        );
        self.jingle = true;
    }
    pub fn tick(&mut self, cache: &mut DiskCache) -> Result<()> {
        if let Some(mut task) = self.tasks.pop_front() {
            match task.tick(self, cache) {
                Ok(true) => {
                    if let Some(next) = task.next.take() {
                        self.tasks.push_front(*next);
                    }
                }
                Ok(false) => self.tasks.push_front(task),
                Err(error) => {
                    self.tasks.clear();
                    self.clear_all();
                    return Err(error);
                }
            }
        }
        if self.jingle && !self.waiting_for_music() {
            self.jingle = false;
            if !self.remembered.is_empty() {
                self.schedule(self.remembered.clone(), self.timings, false);
            }
        }
        // Only task/live request references need to survive; history is stored as songs.
        if self.tasks.is_empty() {
            self.requests.retain(|id, _| self.active.contains(id));
        }
        Ok(())
    }
    pub fn mixer(&self) -> Mixer { Mixer { players: self.players.clone() } }
    pub fn mix(&mut self, output: &mut [i32], stereo: bool) {
        self.mixer().mix(output, stereo);
    }
}

#[cfg(all(test, feature = "offscreen"))]
impl Music {
    /// #233 test-only: installs an authored standard MIDI file on one player slot, so the
    /// null-device tests can advance a finite song without cache data.
    pub(crate) fn install_authored(&mut self, slot: usize, midi: &[u8], looping: bool) -> Result<()> {
        self.players.lock().expect("music mixer lock")[slot].install(midi, looping)
    }
    pub(crate) fn playing_slot(&self, slot: usize) -> bool {
        self.players.lock().expect("music mixer lock")[slot].playing()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Reader;
    #[test]
    fn original_active_request_readiness() {
        let mut r = Reader::new(include_bytes!("../tests/fixtures/music-activity-jvm.bin"));
        let count = r.u32().unwrap();
        let n = r.u32().unwrap() as usize;
        let midi = r.bytes(n).unwrap();
        for _ in 0..count {
            let mask = r.u32().unwrap();
            let expected = r.u8().unwrap() != 0;
            let mut music = Music::default();
            if mask & 1 != 0 {
                music.tasks.push_back(Task::new(Kind::Start));
            }
            for slot in 0..2 {
                if mask & (2 << slot) == 0 {
                    continue;
                }
                let present = mask & 32 == 0 || slot != 0;
                if present && mask & (8 << slot) != 0 {
                    music.players.lock().expect("music mixer lock")[slot].install(midi, false).unwrap();
                }
                let id = slot as u64;
                music.active.push(id);
                music.requests.insert(
                    id,
                    Request {
                        song: Song {
                            archive: 6,
                            id: slot as u32,
                            volume: 255,
                            looping: false,
                        },
                        current: 0.0,
                        slot: present.then_some(slot),
                        track: None,
                        started: true,
                        marked: false,
                        fading: false,
                    },
                );
            }
            assert_eq!(music.waiting_for_music(), expected, "mask {mask}");
        }
        assert_eq!(r.remaining(), 0);
    }
    #[test]
    fn original_music_request_queue_cycles() {
        let mut r = Reader::new(include_bytes!("../tests/fixtures/music-queue-jvm.bin"));
        let frames = r.u32().unwrap();
        let n = r.u32().unwrap() as usize;
        let midi = r.bytes(n).unwrap().to_vec();
        let root =
            std::env::temp_dir().join(format!("gielinor-music-queue-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("main_file_cache.dat2"), []).unwrap();
        let mut cache = DiskCache::open(&root, 1024).unwrap();
        let mut music = Music::default();
        for frame in 0..frames {
            let operation = r.u32().unwrap();
            let timing = std::array::from_fn(|_| r.u32().unwrap() as i32);
            match operation {
                1 | 4 => {
                    let songs = (0..r.u32().unwrap())
                        .map(|_| Song {
                            archive: 6,
                            id: r.u32().unwrap(),
                            volume: r.u32().unwrap() as i32,
                            looping: false,
                        })
                        .collect();
                    music.schedule(songs, timing, operation == 4);
                }
                2 => music.stop(timing[0], timing[1]),
                3 => music.swap(timing),
                _ => {}
            }
            // JVM oracle injects already-decoded tracks as well, isolating queue timing
            // from asynchronous archive loading and thread scheduling.
            for request in music.requests.values_mut() {
                if !request.started && request.track.is_none() {
                    request.track = Some(Track {
                        midi: midi.clone(),
                        instruments: BTreeMap::new(),
                    });
                }
            }
            music.tick(&mut cache).unwrap();
            assert_eq!(
                music.tasks.len(),
                r.u32().unwrap() as usize,
                "frame {frame} tasks"
            );
            assert_eq!(
                music.active.len(),
                r.u32().unwrap() as usize,
                "frame {frame} active count"
            );
            for &id in &music.active {
                let request = &music.requests[&id];
                assert_eq!(request.song.id, r.u32().unwrap(), "frame {frame} song");
                assert_eq!(
                    request.current.to_bits(),
                    r.u32().unwrap(),
                    "frame {frame} fade"
                );
                assert_eq!(
                    request.slot.unwrap(),
                    r.u32().unwrap() as usize,
                    "frame {frame} slot"
                );
                assert_eq!(
                    request.started,
                    r.u8().unwrap() != 0,
                    "frame {frame} started"
                );
                assert_eq!(request.marked, r.u8().unwrap() != 0, "frame {frame} marked");
                assert_eq!(request.fading, r.u8().unwrap() != 0, "frame {frame} fading");
            }
            for slot in 0..3 {
                assert_eq!(
                    music.users[slot],
                    r.u32().unwrap() as i32,
                    "frame {frame} slot {slot} users"
                );
                assert_eq!(
                    music.players.lock().expect("music mixer lock")[slot].synth.master,
                    r.u32().unwrap() as i32,
                    "frame {frame} slot {slot} volume"
                );
                assert_eq!(
                    music.players.lock().expect("music mixer lock")[slot].playing(),
                    r.u8().unwrap() != 0,
                    "frame {frame} slot {slot} playing"
                );
            }
            assert_eq!(
                music.remembered.len(),
                r.u32().unwrap() as usize,
                "frame {frame} remembered"
            );
            for song in &music.remembered {
                assert_eq!(song.id, r.u32().unwrap(), "frame {frame} remembered song");
            }
        }
        assert_eq!(r.remaining(), 0);
        drop(cache);
        std::fs::remove_file(root.join("main_file_cache.dat2")).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
