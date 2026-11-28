//! Independent native PCM worker, using the original device-occupancy pump policy.
use crate::{audio_pump::{Device, Pump, BLOCK},audio_runtime::Mixer,Result};
use std::{sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}},thread::{self,JoinHandle},time::{Duration,Instant}};
#[derive(Clone,Copy,Debug,Default)]
pub struct Stats {pub pumps:u64,pub mixed_frames:u64,pub submitted_frames:u64,pub skipped_frames:u64,pub errors:u64}
pub struct Worker {
 mixer:Arc<Mutex<Mixer>>,
 stop:Arc<AtomicBool>,
 stats:Arc<Mutex<Stats>>,
 thread:Option<JoinHandle<()>>,
}
struct OutputDevice {mixer:Mixer,output:Option<crate::audio_output::Output>,stats:Stats}
impl Device for OutputDevice {
 fn queued(&mut self)->Result<i32>{self.output.as_mut().ok_or(crate::Error::Invalid("audio device closed"))?.queued_frames()}
 #[cfg(not(feature = "offscreen"))]
 fn open(&mut self,_capacity:i32)->Result<()>{self.output=Some(crate::audio_output::Output::open_stereo()?);Ok(())}
 #[cfg(feature = "offscreen")]
 fn open(&mut self,capacity:i32)->Result<()>{self.output=Some(crate::audio_output::Output::open_null(capacity)?);Ok(())}
 fn close(&mut self){self.output=None;}
 fn write_block(&mut self)->Result<()> {
  let samples=self.mixer.mix_stereo(BLOCK as usize);self.stats.mixed_frames+=BLOCK as u64;
  self.output.as_mut().ok_or(crate::Error::Invalid("audio device closed"))?.write(samples)?;
  self.stats.submitted_frames+=BLOCK as u64;Ok(())
 }
 fn skip(&mut self,frames:i32){let _=self.mixer.mix_stereo(frames as usize);self.stats.skipped_frames+=frames as u64;}
}
impl Worker {
 pub fn start(mixer:Mixer)->Result<Self>{
  let shared=Arc::new(Mutex::new(mixer));let stop=Arc::new(AtomicBool::new(false));let stats=Arc::new(Mutex::new(Stats::default()));
  let source=shared.clone();let stopping=stop.clone();let result=stats.clone();
  let thread=thread::Builder::new().name("rust-client-pcm".into()).spawn(move||{
   let epoch=Instant::now();
   let mut device=OutputDevice{mixer:source.lock().expect("PCM source lock").clone(),output:None,stats:Stats::default()};
   // GameClient creates channel zero with a 2048-frame target; oh.ae starts capacity at 4096.
   let mut pump=Pump{capacity:4096,target:2048,adjustment:0,previous_queued:0,drained:0,previous_drained:0,retry_at:1,check_at:0,mixed_at:1,fresh:true};
   while !stopping.load(Ordering::Acquire) {
    let start=Instant::now();device.mixer=source.lock().expect("PCM source lock").clone();
    let now=epoch.elapsed().as_millis().min(i64::MAX as u128-1) as i64+1;
    if let Some(error)=pump.step(now,22050,&mut device){device.stats.errors+=1;eprintln!("Audio pump: {error}; retrying after two seconds");}
    device.stats.pumps+=1;*result.lock().expect("PCM statistics lock")=device.stats;
    thread::park_timeout(Duration::from_millis(10).saturating_sub(start.elapsed()));
   }
  })?;
  Ok(Self{mixer:shared,stop,stats,thread:Some(thread)})
 }
 /// Login replaces effect state while preserving the music streams.
 pub fn set_mixer(&self,mixer:Mixer){*self.mixer.lock().expect("PCM source lock")=mixer;}
 pub fn stats(&self)->Stats{*self.stats.lock().expect("PCM statistics lock")}
}
impl Drop for Worker {
 fn drop(&mut self){self.stop.store(true,Ordering::Release);if let Some(thread)=self.thread.take(){thread.thread().unpark();if thread.join().is_err(){eprintln!("Audio worker terminated unexpectedly");}}}
}
#[cfg(all(test, feature = "offscreen"))]
mod null_device_tests {
    //! #233: the unchanged Pump and OutputDevice over the null device, on an injected clock in
    //! 10 ms steps (the worker's period). Shared effect and music state advance only through
    //! `write_block`/`skip` mixing, as with the real device.
    use super::*;
    use crate::audio_runtime::{Audio, Sound};
    struct Run { device: OutputDevice, pump: Pump }
    impl Run {
        fn new(mixer: Mixer) -> Self {
            // The same Pump the worker starts with.
            let pump = Pump{capacity:4096,target:2048,adjustment:0,previous_queued:0,drained:0,previous_drained:0,retry_at:1,check_at:0,mixed_at:1,fresh:true};
            Self { device: OutputDevice { mixer, output: None, stats: Stats::default() }, pump }
        }
        /// One worker step at `ms` milliseconds; the worker passes elapsed milliseconds + 1.
        fn step(&mut self, ms: u64) -> u64 {
            crate::audio_output::set_test_now(Duration::from_millis(ms));
            let before = self.mixed();
            assert!(self.pump.step(ms as i64 + 1, 22050, &mut self.device).is_none(), "pump error at {ms} ms");
            let queue = self.device.output.as_ref().expect("device open").queue();
            assert!(queue.queued() <= queue.capacity(), "queue within capacity at {ms} ms");
            self.mixed() - before
        }
        fn mixed(&self) -> u64 { self.device.stats.mixed_frames + self.device.stats.skipped_frames }
    }
    /// Frames a real device drains in `ms` milliseconds at 22050 Hz.
    fn nominal(ms: u64) -> u64 { ms * 22050 / 1000 }

    #[test]
    fn pump_with_null_device_keeps_queue_within_capacity_and_nominal_rate() {
        let mut run = Run::new(Audio::default().mixer());
        for n in 0..=1000u64 {
            run.step(n * 10);
            let mixed = run.mixed();
            let capacity = run.device.output.as_ref().unwrap().queue().capacity();
            assert!(mixed >= nominal(n * 10) && mixed <= nominal(n * 10) + capacity, "{mixed} frames at {} ms", n * 10);
        }
        assert_eq!((run.device.stats.skipped_frames, run.device.stats.errors), (0, 0));
        assert_eq!(run.device.stats.submitted_frames, run.device.stats.mixed_frames);
    }

    #[test]
    fn pump_with_null_device_resumes_after_a_stall_without_a_catch_up_burst() {
        let mut run = Run::new(Audio::default().mixer());
        for n in 0..=100u64 { run.step(n * 10); }
        // A 3 s stall (worker descheduled), below the 5 s skip threshold: the queue empties.
        let resume = 1000 + 3000;
        let refill = run.step(resume);
        let target = (run.pump.target + run.pump.adjustment) as u64;
        assert!(refill <= target + BLOCK as u64, "one refill to the target, not a catch-up: {refill}");
        let mut window = 0;
        for n in 1..=100u64 {
            let mixed = run.step(resume + n * 10);
            assert!(mixed <= 2 * BLOCK as u64, "{mixed} frames in one 10 ms step after the stall");
            window += mixed;
        }
        let capacity = run.device.output.as_ref().unwrap().queue().capacity();
        assert!(window <= nominal(1000) + capacity, "{window} frames in the second after the stall");
    }

    #[test]
    fn pump_with_null_device_mixes_a_finite_effect_voice_to_completion() {
        let mut audio = Audio::default();
        // 0.5 s at the output rate, so one voice step is one output frame.
        let length = 11025;
        audio.play(std::sync::Arc::new(Sound {
            rate: 22050, ping_pong: false, legacy: false, samples: vec![4096; length], loop_start: 0, loop_end: 0, delay: 0,
        }), 0, 128);
        assert!(audio.active());
        let mut run = Run::new(audio.mixer());
        let mut finished = None;
        for n in 0..=200u64 {
            run.step(n * 10);
            // mix_voices prunes the voice in the mix that reaches its end, and only there.
            assert_eq!(audio.active(), run.mixed() < length as u64, "at {} ms after {} frames", n * 10, run.mixed());
            if !audio.active() { finished = Some(n * 10); break; }
        }
        let finished = finished.expect("the voice finished within 2 s");
        // Never ahead of the nominal drain by more than the device queue, never behind it.
        let capacity = run.device.output.as_ref().unwrap().queue().capacity();
        assert!(nominal(finished) + capacity >= length as u64 && finished <= 510, "finished at {finished} ms");
    }

    /// An authored format-0 MIDI file, not cache data: note on at tick 0, note off and
    /// end-of-track at tick 96 (one quarter note at the default 500,000 us tempo = 0.5 s).
    const SONG: [u8; 34] = [
        b'M', b'T', b'h', b'd', 0, 0, 0, 6, 0, 0, 0, 1, 0, 96,
        b'M', b'T', b'r', b'k', 0, 0, 0, 12,
        0x00, 0x90, 0x3c, 0x40,
        0x60, 0x80, 0x3c, 0x40,
        0x00, 0xff, 0x2f, 0x00,
    ];

    #[test]
    fn pump_with_null_device_advances_an_authored_finite_song_to_not_playing() {
        let mut audio = Audio::default();
        audio.music.install_authored(0, &SONG, false).unwrap();
        assert!(audio.music.playing_slot(0) && audio.active());
        // The player's step is 96 * 10^6 / 22050 = 4353 time units per frame; end-of-track
        // is due at 96 * 500,000 = 48,000,000, so it is reached in the mix that crosses frame
        // ceil(48,000,000 / 4353) = 11,027.
        let end = 11027;
        let mut run = Run::new(audio.mixer());
        let mut finished = None;
        for n in 0..=200u64 {
            run.step(n * 10);
            assert_eq!(audio.music.playing_slot(0), run.mixed() < end, "at {} ms after {} frames", n * 10, run.mixed());
            if !audio.music.playing_slot(0) { finished = Some(n * 10); break; }
        }
        let finished = finished.expect("the song ended within 2 s");
        let capacity = run.device.output.as_ref().unwrap().queue().capacity();
        assert!(nominal(finished) + capacity >= end && finished <= 510, "ended at {finished} ms");
        assert!(!audio.active(), "no instrument was loaded, so nothing keeps sounding");
    }
}
