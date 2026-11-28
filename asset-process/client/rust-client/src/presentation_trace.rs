//! Explicit per-window bounded lifecycle diagnostics. Never writes pixels or addresses.
use std::{
    collections::VecDeque,
    fs::{File, OpenOptions},
    io::{self, Seek, SeekFrom, Write},
    path::Path,
    time::Instant,
};
const HEADER: &str = "frame,start_ms,network_ms,scripts_ms,world_ms,ui_ms,present_ms,logical_w,logical_h,window_w,window_h,hash,black_samples,samples,success,paint_events,erase_events,resize_events\n";
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Events {
    pub paint: u64,
    pub erase: u64,
    pub resize: u64,
}
impl Events {
    pub fn event(&mut self, message: u32) {
        let counter = match message {
            0x0f => &mut self.paint,
            0x14 => &mut self.erase,
            5 => &mut self.resize,
            _ => return,
        };
        *counter = counter.wrapping_add(1);
    }
}
pub struct Sample<'a> {
    /// Start, after network, after scripts, after world, before submit, after submit.
    pub times: [Instant; 6],
    pub logical: [i32; 2],
    pub physical: [i32; 2],
    pub pixels: &'a [u32],
    pub success: bool,
    pub events: Events,
}
pub struct Trace {
    file: File,
    epoch: Instant,
    rows: VecDeque<String>,
    count: u64,
    failed: bool,
}
impl Trace {
    pub fn frame_count(&self)->u64 {self.count}
    /// Reserve only a fresh diagnostic file; never truncate somebody else's evidence.
    pub fn new(path: &Path) -> io::Result<Self> {
        let file = OpenOptions::new().write(true).create_new(true).open(path)?;
        let mut trace = Self {
            file,
            epoch: Instant::now(),
            rows: VecDeque::new(),
            count: 0,
            failed: false,
        };
        trace.save()?;
        Ok(trace)
    }
    /// An observation/write failure disables only this diagnostic stream.
    pub fn complete(owner: &mut Option<Self>, sample: Sample<'_>) -> Option<io::Error> {
        let trace = owner.as_mut()?;
        if let Err(error) = trace.record(sample) {
            trace.failed = true;
            *owner = None;
            Some(error)
        } else {
            None
        }
    }
    fn record(&mut self, s: Sample<'_>) -> io::Result<()> {
        let invalid = || {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "nonmonotonic presentation trace time",
            )
        };
        let start = s.times[0]
            .checked_duration_since(self.epoch)
            .ok_or_else(invalid)?
            .as_secs_f64()
            * 1000.;
        let mut stages = [0.; 5];
        for (i, value) in stages.iter_mut().enumerate() {
            *value = s.times[i + 1]
                .checked_duration_since(s.times[i])
                .ok_or_else(invalid)?
                .as_secs_f64()
                * 1000.;
        }
        let (hash, black, samples) = fingerprint(s.pixels);
        self.count = self.count.wrapping_add(1);
        self.rows.push_back(format!("{},{start:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{},{},{},{},{hash:x},{black},{samples},{},{},{},{}\n",self.count,stages[0],stages[1],stages[2],stages[3],stages[4],s.logical[0],s.logical[1],s.physical[0],s.physical[1],s.success,s.events.paint,s.events.erase,s.events.resize));
        if self.rows.len() > 512 {
            self.rows.pop_front();
        }
        if self.count % 128 == 0 {
            self.save()?;
        }
        Ok(())
    }
    fn save(&mut self) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(HEADER.as_bytes())?;
        for row in &self.rows {
            self.file.write_all(row.as_bytes())?;
        }
        let end = self.file.stream_position()?;
        self.file.set_len(end)?;
        self.file.flush()
    }
}
impl Drop for Trace {
    fn drop(&mut self) {
        if !self.failed {
            let _ = self.save();
        }
    }
}
fn fingerprint(pixels: &[u32]) -> (u64, usize, usize) {
    let mut hash = 0xcbf29ce484222325u64;
    let mut black = 0;
    let mut samples = 0;
    // Preserve the source stride exactly (at most8191 samples, not exactly4096).
    for &pixel in pixels.iter().step_by((pixels.len() / 4096).max(1)) {
        hash = (hash ^ u64::from(pixel)).wrapping_mul(0x100000001b3);
        black += usize::from(pixel & 0xfefefe == 0);
        samples += 1;
    }
    (hash, black, samples)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    fn path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "client-present-trace-{}-{}.csv",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ))
    }
    fn sample(epoch: Instant, success: bool) -> Sample<'static> {
        Sample {
            times: std::array::from_fn(|i| epoch + Duration::from_millis(i as u64)),
            logical: [2, 2],
            physical: [64, 64],
            pixels: &[0, 0x010101, 0x020202, 0xffffffff],
            success,
            events: Events {
                paint: 2,
                erase: 3,
                resize: 4,
            },
        }
    }
    #[test]
    fn source_fingerprint_stride_and_near_black_rule_are_preserved() {
        assert_eq!(
            fingerprint(&[0, 0x010101, 0x020202, 0xffffffff]),
            (0x1fb9d98419e1ac55, 2, 4)
        );
        assert_eq!(fingerprint(&[]), (0xcbf29ce484222325, 0, 0));
        assert_eq!(fingerprint(&vec![0; 8191]).2, 8191);
        assert_eq!(fingerprint(&vec![0; 8192]).2, 4096);
    }
    #[test]
    fn rolling_rows_flush_cadence_drop_and_existing_output_are_bounded() {
        let path = path();
        let mut trace = Trace::new(&path).unwrap();
        let epoch = trace.epoch;
        assert!(Trace::new(&path).is_err());
        for _ in 0..127 {
            trace.record(sample(epoch, true)).unwrap();
        }
        assert_eq!(std::fs::read_to_string(&path).unwrap(), HEADER);
        trace.record(sample(epoch, false)).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text.lines().count(), 129);
        assert!(text.lines().last().unwrap().ends_with(",false,2,3,4"));
        for _ in 128..600 {
            trace.record(sample(epoch, true)).unwrap();
        }
        assert_eq!(trace.rows.len(), 512);
        drop(trace);
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text.lines().count(), 513);
        assert!(text.lines().nth(1).unwrap().starts_with("89,"));
        assert!(text.lines().last().unwrap().starts_with("600,"));
        assert!(!text.lines().next().unwrap().contains("buffer"));
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn invalid_time_and_failed_write_disable_only_the_owner() {
        let first = path();
        let second = path();
        let mut a = Some(Trace::new(&first).unwrap());
        let b = Trace::new(&second).unwrap();
        let mut s = sample(a.as_ref().unwrap().epoch, true);
        s.times[2] = s.times[0];
        assert!(Trace::complete(&mut a, s).is_some());
        assert!(a.is_none());
        assert_eq!(b.count, 0);
        let third = path();
        let mut c = Some(Trace::new(&third).unwrap());
        let epoch = c.as_ref().unwrap().epoch;
        c.as_mut().unwrap().file = File::open(&third).unwrap();
        c.as_mut().unwrap().count = 127;
        assert!(Trace::complete(&mut c, sample(epoch, true)).is_some());
        assert!(c.is_none());
        drop(b);
        for p in [first, second, third] {
            std::fs::remove_file(p).unwrap();
        }
    }
    #[test]
    fn event_counts_are_local_and_wrap_without_panicking() {
        let mut a = Events {
            paint: u64::MAX,
            ..Default::default()
        };
        let b = Events::default();
        for message in [0x0f, 0x14, 5, 99] {
            a.event(message);
        }
        assert_eq!(
            a,
            Events {
                paint: 0,
                erase: 1,
                resize: 1
            }
        );
        assert_eq!(b, Events::default());
    }
}
