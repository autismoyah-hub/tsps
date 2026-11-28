//! Explicit per-client sampling, adapted from experimental ui_profile.rs.
//! Observes existing work without changing ticks, listeners or layout.
use std::{
    io::{self, Write},
    path::PathBuf,
    time::Instant,
};

pub struct Profile {
    path: PathBuf,
    count: u64,
}

/// Explicit diagnostic controls; ordinary clients use the inert default.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Benchmark {
    orbit: Option<i32>,
    zoom: Option<[i16; 2]>,
    window: Option<[i32; 2]>,
    zero_area_frames: Option<u32>,
    sample_frames: Option<u32>,
    warmup_frames: Option<u32>,
    headless: bool,
    scene_filter: Option<crate::benchmark_scene::Mode>,
}
impl Benchmark {
    /// Separate benchmark flags from renderer flags before any content/network I/O.
    pub fn from_options(options:&[String])->Result<(Self,Vec<String>),String> {
        let mut result=Self::default();let mut renderer=Vec::new();
        for option in options {
            if option=="--headless" {
                if result.headless {return Err("Repeated --headless".into());}
                result.headless=true;continue;
            }
            let Some((name,value))=option.split_once('=') else {renderer.push(option.clone());continue;};
            match name {
                "--scene-filter" => {
                    if result.scene_filter.is_some(){return Err("Repeated --scene-filter".into());}
                    let mode=crate::benchmark_scene::Mode::parse(value).ok_or("Invalid --scene-filter")?;
                    #[cfg(not(feature="custom-assets"))]
                    if mode!=crate::benchmark_scene::Mode::Full {return Err("Scene exclusions require custom-assets".into());}
                    result.scene_filter=Some(mode);
                }
                "--sample-frames" | "--warmup-frames" => {
                    let target=if name=="--sample-frames" {&mut result.sample_frames}else{&mut result.warmup_frames};
                    if target.is_some(){return Err(format!("Repeated {name}"));}
                    let value=value.parse::<u32>().map_err(|_|format!("Invalid {name}"))?;
                    let bounds=if name=="--sample-frames" {60..=480}else{1..=4096};
                    if !bounds.contains(&value){return Err(format!("{name} outside bounds"));}
                    *target=Some(value);
                }
                "--zero-area-frames" => {
                    if result.zero_area_frames.is_some(){return Err("Repeated --zero-area-frames".into());}
                    let count=value.parse::<u32>().map_err(|_|"Invalid --zero-area-frames")?;
                    if !(1..=120).contains(&count){return Err("--zero-area-frames outside1..120".into());}
                    result.zero_area_frames=Some(count);
                }
                "--orbit-step" => {
                    if result.orbit.is_some(){return Err("Repeated --orbit-step".into());}
                    let step=value.parse::<i32>().map_err(|_|"Invalid --orbit-step")?;
                    if !(-512..=512).contains(&step){return Err("--orbit-step outside -512..512".into());}
                    result.orbit=Some(step);
                }
                "--zoom" => {
                    if result.zoom.is_some(){return Err("Repeated --zoom".into());}
                    let (a,b)=value.split_once(',').ok_or("--zoom requires X,Y")?;
                    let pair=[a.parse::<i16>().map_err(|_|"Invalid --zoom")?,b.parse::<i16>().map_err(|_|"Invalid --zoom")?];
                    if pair.iter().any(|v|!(128..=512).contains(v)){return Err("--zoom outside 128..512".into());}
                    result.zoom=Some(pair);
                }
                "--window" => {
                    if result.window.is_some(){return Err("Repeated --window".into());}
                    let (w,h)=value.split_once('x').ok_or("--window requires WIDTHxHEIGHT")?;
                    let size=[w.parse::<i32>().map_err(|_|"Invalid --window")?,h.parse::<i32>().map_err(|_|"Invalid --window")?];
                    if !(640..=3840).contains(&size[0]) || !(480..=2160).contains(&size[1]){return Err("--window outside 640x480..3840x2160".into());}
                    result.window=Some(size);
                }
                _=>renderer.push(option.clone()),
            }
        }
        if result.warmup_frames.is_some() && result.sample_frames.is_none() {return Err("--warmup-frames requires --sample-frames".into());}
        if result.sample_frames.is_some() && result.zero_area_frames.is_some() {return Err("steady-state sampling excludes zero-area probes".into());}
        if result.scene_filter()!=crate::benchmark_scene::Mode::Full && (!result.headless || result.sample_frames.is_none()) {
            return Err("Scene exclusions require bounded headless sampling".into());
        }
        Ok((result,renderer))
    }
    pub fn headless(&self)->bool {self.headless}
    pub fn scene_filter(&self)->crate::benchmark_scene::Mode {self.scene_filter.unwrap_or_default()}
    pub fn run(&self)->Option<crate::benchmark_run::Run> {self.sample_frames.map(|n|crate::benchmark_run::Run::new(self.warmup_frames.unwrap_or(64),n))}
    pub fn sample_yaw(&self,index:u32)->i32 {self.orbit.unwrap_or(0).wrapping_mul(index as i32)&16383}
    pub fn yaw(&self,current:i32)->i32 {self.orbit.map_or(current,|step|current.wrapping_add(step)&16383)}
    pub fn zoom(&self)->Option<[i16;2]> {self.zoom}
    pub fn window(&self)->Option<[i32;2]> {self.window}
    pub fn zero_area_frames(&self)->Option<u32> {self.zero_area_frames}
}

impl Profile {
    /// A failed diagnostic write disables only this stream, never gameplay.
    pub fn complete(owner:&mut Option<Self>,sample:Option<Sample>,work:u64)->Option<io::Error> {
        match Sample::finish(sample,work) {Ok(())=>None,Err(error)=>{*owner=None;Some(error)}}
    }

    pub fn new(path: PathBuf) -> Self {
        Self { path, count: 0 }
    }

    /// The first cycle and every 64th subsequent cycle, matching the experiment.
    pub fn begin(&mut self, cycle: i32, widgets: usize) -> Option<Sample> {
        let capture = self.count % 64 == 0;
        self.count = self.count.wrapping_add(1);
        if !capture {
            return None;
        }
        let now = Instant::now();
        Some(Sample {
            start: now,
            last: now,
            stages: [0.; 6],
            next: 0,
            cycle,
            widgets,
            path: self.path.clone(),
        })
    }
}

pub struct Sample {
    start: Instant,
    last: Instant,
    stages: [f64; 6],
    next: usize,
    cycle: i32,
    widgets: usize,
    path: PathBuf,
}

impl Sample {
    pub fn mark(sample: &mut Option<Self>) {
        if let Some(s) = sample {
            let now = Instant::now();
            if s.next < s.stages.len() {
                s.stages[s.next] = now.duration_since(s.last).as_secs_f64() * 1000.;
                s.next += 1;
            }
            s.last = now;
        }
    }

    /// Report I/O errors to diagnostic callers; they must not interrupt gameplay.
    pub fn finish(sample: Option<Self>, instructions: u64) -> io::Result<()> {
        let Some(s) = sample else {
            return Ok(());
        };
        let total = s.start.elapsed().as_secs_f64() * 1000.;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&s.path)?;
        write_row(&mut file, s.cycle, s.widgets, instructions, total, s.stages)
    }
}

fn write_row(
    out: &mut impl Write,
    cycle: i32,
    widgets: usize,
    instructions: u64,
    total: f64,
    stages: [f64; 6],
) -> io::Result<()> {
    writeln!(
        out,
        "{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
        cycle,
        widgets,
        instructions,
        total,
        stages[0],
        stages[1],
        stages[2],
        stages[3],
        stages[4],
        stages[5]
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scene_exclusions_are_explicit_bounded_and_separate_from_renderer_options() {
        assert_eq!(Benchmark::default().scene_filter(),crate::benchmark_scene::Mode::Full);
        let full=Benchmark::from_options(&["--scene-filter=full".into()]).unwrap();
        assert_eq!(full.0.scene_filter(),crate::benchmark_scene::Mode::Full);assert!(full.1.is_empty());
        for name in ["essential","no-furnishings","no-floor-finish","no-upper-architecture"] {
            let options=["--headless".into(),"--sample-frames=300".into(),format!("--scene-filter={name}"),"--ao=4".into()];
            #[cfg(feature="custom-assets")]
            {
                let (b,renderer)=Benchmark::from_options(&options).unwrap();
                assert_eq!(b.scene_filter().name(),name);assert_eq!(renderer,["--ao=4"]);
            }
            #[cfg(not(feature="custom-assets"))]
            assert!(Benchmark::from_options(&options).is_err());
            assert!(Benchmark::from_options(&[format!("--scene-filter={name}")]).is_err());
        }
        for options in [vec!["--scene-filter=unknown"],vec!["--scene-filter=full","--scene-filter=full"],
            vec!["--scene-filter=essential","--sample-frames=300"],vec!["--scene-filter=essential","--headless"]] {
            assert!(Benchmark::from_options(&options.into_iter().map(String::from).collect::<Vec<_>>()).is_err());
        }
    }
    #[test]
    fn bounded_samples_reject_invalid_windows_and_preserve_renderer_flags() {
        let (b,rest)=Benchmark::from_options(&["--headless","--sample-frames=300","--warmup-frames=64","--orbit-step=64","--ao=4"].map(String::from)).unwrap();
        assert!(b.headless());assert!(b.run().is_some());assert_eq!(rest,["--ao=4"]);
        assert_eq!(b.sample_yaw(0),0);assert_eq!(b.sample_yaw(256),0);assert_eq!(b.sample_yaw(1),64);
        assert!(Benchmark::default().run().is_none());assert!(!Benchmark::default().headless());
        for option in ["--sample-frames=0","--sample-frames=59","--sample-frames=481","--sample-frames=NaN","--warmup-frames=64"] {
            assert!(Benchmark::from_options(&[option.into()]).is_err());
        }
        for extra in ["--warmup-frames=0","--warmup-frames=4097","--sample-frames=300","--zero-area-frames=1"] {
            assert!(Benchmark::from_options(&["--sample-frames=300".into(),extra.into()]).is_err());
        }
        assert!(Benchmark::from_options(&["--headless".into(),"--headless".into()]).is_err());
    }
    #[test]
    fn zero_area_probe_is_explicit_bounded_and_not_a_renderer_option() {
        assert_eq!(Benchmark::default().zero_area_frames(),None);
        for count in [1,32,120] {
            let (b,rest)=Benchmark::from_options(&[format!("--zero-area-frames={count}")]).unwrap();
            assert_eq!(b.zero_area_frames(),Some(count));assert!(rest.is_empty());
        }
        for value in ["0","121","-1","NaN","4294967296"] {
            assert!(Benchmark::from_options(&[format!("--zero-area-frames={value}")]).is_err());
        }
        assert!(Benchmark::from_options(&["--zero-area-frames=1".into(),"--zero-area-frames=2".into()]).is_err());
    }
    #[test]
    fn benchmark_preserves_original_bounds_and_wrapped_orbit() {
        let options=["--orbit-step=-512","--zoom=128,512","--window=640x2160","--ao=4"].map(String::from);
        let (b,rest)=Benchmark::from_options(&options).unwrap();
        assert_eq!(rest,["--ao=4"]);assert_eq!(b.yaw(0),15872);assert_eq!(b.zoom(),Some([128,512]));assert_eq!(b.window(),Some([640,2160]));
        let (positive,_)=Benchmark::from_options(&["--orbit-step=512".into()]).unwrap();
        assert_eq!(positive.yaw(16383),511);assert_eq!(positive.yaw(i32::MAX),511);
        assert_eq!(Benchmark::default().yaw(-1),-1);
        assert_eq!(Benchmark::default().zoom(),None);assert_eq!(Benchmark::default().window(),None);
    }
    #[test]
    fn malformed_or_duplicate_benchmark_controls_are_rejected() {
        for option in ["--orbit-step=-513","--orbit-step=513","--orbit-step=NaN","--zoom=127,512","--zoom=128,513","--zoom=1,2,3","--window=639x480","--window=640x479","--window=3841x480","--window=640x2161","--window=640x480x1"] {
            assert!(Benchmark::from_options(&[option.into()]).is_err(),"{option}");
        }
        for option in ["--orbit-step=0","--zoom=256,256","--window=765x503"] {
            assert!(Benchmark::from_options(&[option.into(),option.into()]).is_err());
        }
    }
    #[test]
    fn failed_output_disables_only_the_owning_stream() {
        let mut broken=Some(Profile::new(std::env::temp_dir()));
        let mut other=Some(Profile::new("unused".into()));
        let sample=broken.as_mut().unwrap().begin(0,1);
        assert!(Profile::complete(&mut broken,sample,0).is_some());
        assert!(broken.is_none());assert!(other.as_mut().unwrap().begin(0,1).is_some());
        assert!(Profile::complete(&mut broken,None,0).is_none());
    }
    #[test]
    fn cadence_matches_original_and_is_instance_owned() {
        let mut a = Profile::new("unused-a".into());
        let mut b = Profile::new("unused-b".into());
        for cycle in 0..130 {
            assert_eq!(a.begin(cycle, 1).is_some(), cycle % 64 == 0);
        }
        assert!(b.begin(130, 1).is_some());
        a.count = u64::MAX;
        assert!(a.begin(131, 1).is_none());
        assert!(a.begin(132, 1).is_some());
    }
    #[test]
    fn csv_keeps_original_column_order_and_precision() {
        let mut row = Vec::new();
        write_row(&mut row, -1, 7, 99, 21., [1., 2., 3., 4., 5., 6.]).unwrap();
        assert_eq!(
            String::from_utf8(row).unwrap(),
            "-1,7,99,21.000000,1.000000,2.000000,3.000000,4.000000,5.000000,6.000000\n"
        );
    }
    #[test]
    fn disabled_sample_is_a_noop_and_extra_marks_are_bounded() {
        Sample::mark(&mut None);
        Sample::finish(None, 0).unwrap();
        let mut p = Profile::new("unused".into());
        let mut s = p.begin(0, 0);
        for _ in 0..10 {
            Sample::mark(&mut s);
        }
        assert_eq!(s.as_ref().unwrap().next, 6);
        assert!(s.unwrap().stages.iter().all(|v| v.is_finite() && *v >= 0.));
    }
}
