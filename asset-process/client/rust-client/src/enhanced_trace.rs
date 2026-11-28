//! Opt-in, bounded observations of successful Enhanced world draws.
//! Row numbers are world draws, not window presentations. Render time includes readback.
use crate::enhanced::Stats;
use std::{collections::VecDeque, fs::{File, OpenOptions}, io::{self, Seek, SeekFrom, Write}, path::Path};

pub struct Sample<'a> {
    pub cycle: i32,
    pub eye: [f32; 3],
    pub focus: [f32; 3],
    pub child_casters: usize,
    pub stats: &'a Stats,
    pub submission: Option<crate::render3d::SubmissionProfile>,
}
pub struct Trace {
    file: File,
    rows: VecDeque<String>,
    count: u64,
    failed: bool,
}
impl Trace {
    pub fn new(path: &Path) -> io::Result<Self> {
        Ok(Self { file: OpenOptions::new().write(true).create_new(true).open(path)?, rows: VecDeque::new(), count: 0, failed: false })
    }
    /// Failure disables only diagnostics, retaining the last saved evidence.
    pub fn complete(owner: &mut Option<Self>, sample: Sample<'_>) -> Option<io::Error> {
        let trace = owner.as_mut()?;
        if let Err(error) = trace.record(sample) {
            trace.failed = true;
            *owner = None;
            Some(error)
        } else { None }
    }
    fn record(&mut self, s: Sample<'_>) -> io::Result<()> {
        let stats = s.stats;
        if s.submission.is_some_and(|p|p.stage_ms.iter().any(|x|!x.is_finite() || *x<0.)) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,"invalid Classic submission timing"));
        }
        let submission=s.submission.map_or_else(||"null".to_string(),|p|format!(
            "{{\"stage_ms\":{:?},\"model_groups\":{},\"terrain_groups\":{}}}",p.stage_ms,p.model_groups,p.terrain_groups));
        if !s.eye.iter().chain(s.focus.iter()).all(|x| x.is_finite()) ||
            ![Some(stats.prepare_ms), Some(stats.render_ms), Some(stats.readback_ms), Some(stats.init_ms), stats.gpu_ms]
                .into_iter().flatten().all(|x| x.is_finite() && x >= 0.) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid Enhanced trace observation"));
        }
        self.count = self.count.checked_add(1).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Enhanced trace row overflow"))?;
        self.rows.push_back(format!("{{\"row\":{},\"cycle\":{},\"eye\":{:?},\"focus\":{:?},\"child_casters\":{},\"classic_submission\":{},\"renderer\":{}}}\n", self.count, s.cycle, s.eye, s.focus, s.child_casters, submission, stats.json()));
        if self.rows.len() > 512 { self.rows.pop_front(); }
        if self.count % 128 == 0 { self.save()?; }
        Ok(())
    }
    fn save(&mut self) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(0))?;
        for row in &self.rows { self.file.write_all(row.as_bytes())?; }
        let end = self.file.stream_position()?;
        self.file.set_len(end)?;
        self.file.flush()
    }
}
impl Drop for Trace {
    fn drop(&mut self) { if !self.failed { let _ = self.save(); } }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SERIAL: AtomicUsize = AtomicUsize::new(0);
    fn path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("client-enhanced-trace-{}-{}.jsonl",std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed)))
    }
    fn sample(stats: &Stats) -> Sample<'_> {
        Sample { cycle: 123, eye: [1., 2., 3.], focus: [4., 5., 6.], child_casters: 7, stats, submission:Some(crate::render3d::SubmissionProfile {stage_ms:[1.,2.,3.,4.],model_groups:5,terrain_groups:6}) }
    }
    #[test]
    fn bounded_rows_flush_drop_and_fresh_file() {
        let p = path();
        let mut t = Trace::new(&p).unwrap();
        let stats = Stats { adapter: "quote\" slash\\ newline\n control\u{1}".into(), ..Stats::default() };
        for _ in 0..127 { t.record(sample(&stats)).unwrap(); }
        assert_eq!(std::fs::read(&p).unwrap(), b"");
        t.record(sample(&stats)).unwrap();
        let saved = std::fs::read_to_string(&p).unwrap();
        assert_eq!(saved.lines().count(), 128);
        assert!(saved.contains("control\\u0001"));
        assert!(saved.contains("\"classic_submission\":{\"stage_ms\":[1.0, 2.0, 3.0, 4.0],\"model_groups\":5,\"terrain_groups\":6}"));
        assert!(Trace::new(&p).is_err());
        assert_eq!(std::fs::read_to_string(&p).unwrap(), saved);
        for _ in 128..600 { t.record(sample(&stats)).unwrap(); }
        assert_eq!(t.rows.len(), 512);
        drop(t);
        let saved = std::fs::read_to_string(&p).unwrap();
        assert_eq!(saved.lines().count(), 512);
        assert!(saved.lines().next().unwrap().starts_with("{\"row\":89,"));
        assert!(saved.lines().last().unwrap().starts_with("{\"row\":600,"));
        std::fs::remove_file(p).unwrap();
    }
    #[test]
    fn invalid_observations_and_write_failure_disable_only_diagnostics() {
        for kind in 0..6 {
            let p = path();
            let mut owner = Some(Trace::new(&p).unwrap());
            let mut stats = Stats::default();
            if kind == 1 { stats.gpu_ms = Some(f64::NAN); }
            if kind == 2 { stats.readback_ms = -1.; }
            let mut s = sample(&stats);
            if kind == 0 { s.eye[0] = f32::INFINITY; }
            if kind == 4 { s.submission.as_mut().unwrap().stage_ms[0]=f64::NAN; }
            if kind == 5 { s.submission.as_mut().unwrap().stage_ms[3]=-1.; }
            if kind == 3 {
                owner.as_mut().unwrap().file = File::open(&p).unwrap();
                owner.as_mut().unwrap().count = 127;
            }
            assert!(Trace::complete(&mut owner, s).is_some());
            assert!(owner.is_none());
            assert!(Trace::complete(&mut owner, sample(&Stats::default())).is_none());
            assert_eq!(std::fs::read(&p).unwrap(), b"");
            std::fs::remove_file(p).unwrap();
        }
    }
}
