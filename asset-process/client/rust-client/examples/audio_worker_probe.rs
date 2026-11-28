//! Check PCM progress while the controlling/game thread does no work.
#[cfg(windows)]
fn main()->Result<(),Box<dyn std::error::Error>> {
 use gielinor_client::{audio_runtime::{Audio,Sound},audio_worker::Worker};
 use std::{sync::Arc,time::Duration};
 let mut audio=Audio::default();
 audio.play(Arc::new(Sound{rate:22050,ping_pong:false,legacy:false,samples:vec![0;22050],loop_start:0,loop_end:22050,delay:0}),-1,1);
 let worker=Worker::start(audio.mixer())?;
 std::thread::sleep(Duration::from_millis(300));let before=worker.stats();
 std::thread::sleep(Duration::from_millis(1200));let after=worker.stats();
 println!("Independent native PCM before {before:?}; after {after:?}");
 assert_eq!(after.errors,0,"native device errors");
 assert!(after.pumps>before.pumps+20,"worker did not run independently");
 let advanced=after.submitted_frames-before.submitted_frames;
 assert!((20000..=33000).contains(&advanced),"unexpected PCM progress {advanced}");
 drop(worker);println!("Independent native PCM continued through 1.2 seconds of game-thread inactivity and shut down cleanly");Ok(())
}
#[cfg(not(windows))] fn main(){}
