//! Explicit, bounded offline-window timing. Never enabled by environment variables.
use crate::{raster::Raster,Error,Result};
use std::{fmt::Write,path::{Path,PathBuf}};
pub const WARMUP:u64=20;
pub const TOTAL:u64=160;
#[derive(Clone,Copy,Default)]
pub struct Sample {
    pub period_ms:f64,
    pub pose_ms:f64,
    pub classic_ms:f64,
    pub enhanced_ms:f64,
    pub presentation_ms:f64,
    /// Backend prepare/render/readback/GPU durations; absent in Classic.
    pub backend:[Option<f64>;4],
}
/// Benchmark captures are PNG-only. Encodes first, then creates the file with create_new so
/// prior evidence is never overwritten; a zero-size raster is refused before any file exists.
pub fn write_png_new(path:&Path,raster:&Raster)->Result<()> {
    use std::io::Write;
    let bytes=raster.png()?;
    let mut file=std::fs::OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&bytes)?;file.flush()?;Ok(())
}
pub struct Recorder { directory:PathBuf, frames:u64, failed:bool, finished:bool, csv:String }
impl Recorder {
    pub fn new(directory:PathBuf)->Result<Self> {
        // Atomically reserve a new directory; never overwrite prior evidence.
        std::fs::create_dir(&directory)?;
        std::fs::write(directory.join("scope.txt"),"OFFLINE HIDDEN WINDOW TIMING; NOT FULL CLIENT OR GAMEPLAY\nstatus=incomplete\n")?;
        Ok(Self{directory,frames:0,failed:false,finished:false,csv:"frame,frame_period_ms,animation_pose_ms,classic_ms,enhanced_ms,presentation_ms,prepare_ms,render_ms,readback_ms,gpu_ms\n".into()})
    }
    pub fn record(&mut self,s:Sample,presentation_ok:bool)->Result<bool> {
        let values=[s.period_ms,s.pose_ms,s.classic_ms,s.enhanced_ms,s.presentation_ms].map(Some)
            .into_iter().chain(s.backend);
        if self.failed||self.finished||self.frames>=TOTAL||!presentation_ok||values.flatten().any(|v|!v.is_finite()||v<0.) {
            self.failed=true;return Err(Error::Invalid("invalid or failed study benchmark frame"));
        }
        self.frames+=1;
        if self.frames>WARMUP {
            write!(self.csv,"{},{:.6},{:.6},{:.6},{:.6},{:.6}",self.frames,s.period_ms,s.pose_ms,s.classic_ms,s.enhanced_ms,s.presentation_ms).unwrap();
            for value in s.backend {self.csv.push(',');if let Some(value)=value {write!(self.csv,"{value:.6}").unwrap();}}
            self.csv.push('\n');
        }
        Ok(self.frames==TOTAL)
    }
    pub fn finish(&mut self,frame:&Raster,details:&str)->Result<()> {
        if self.failed||self.finished||self.frames!=TOTAL{return Err(Error::Invalid("incomplete study benchmark"));}
        std::fs::write(self.directory.join("frames.csv"),&self.csv)?;
        write_png_new(&self.directory.join("completed-frame.png"),frame)?;
        // Completion is published last, after both evidence files succeed.
        std::fs::write(self.directory.join("scope.txt"),format!("OFFLINE HIDDEN WINDOW TIMING; NOT FULL CLIENT OR GAMEPLAY\nstatus=complete\nframes={TOTAL}\nwarmup={WARMUP}\nmeasured_frames={}\ninput=fixed initial study view\n{details}",TOTAL-WARMUP))?;
        self.finished=true;Ok(())
    }
    pub fn completed(&self)->bool {self.finished}
}
#[cfg(test)]
mod tests {
    use super::*;
    struct Temp(PathBuf);
    impl Temp {
        fn new()->Self {static NEXT:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
            Self(std::env::temp_dir().join(format!("gielinor-study-profile-{}-{}",std::process::id(),NEXT.fetch_add(1,std::sync::atomic::Ordering::Relaxed))))}
    }
    impl Drop for Temp {fn drop(&mut self){for n in ["scope.txt","frames.csv","completed-frame.png","keep","capture.png"]{let _=std::fs::remove_file(self.0.join(n));}let _=std::fs::remove_dir(&self.0);}}
    #[test]
    fn source_warmup_and_total_require_successful_presentations() {
        let temp=Temp::new();let mut r=Recorder::new(temp.0.clone()).unwrap();
        for frame in 1..=TOTAL {assert_eq!(r.record(Sample{period_ms:16.,..Default::default()},true).unwrap(),frame==TOTAL);}
        assert_eq!(r.csv.lines().count(),141);assert!(r.csv.lines().nth(1).unwrap().starts_with("21,16.000000,"));
        assert!(r.csv.lines().last().unwrap().starts_with("160,"));assert!(r.csv.lines().last().unwrap().ends_with(",,,,"));
        let frame=Raster::from_pixels(3,2,vec![0xff102030;6]).unwrap();
        r.finish(&frame,"mode=classic\n").unwrap();assert!(r.completed());
        assert!(std::fs::read_to_string(temp.0.join("scope.txt")).unwrap().contains("status=complete\n"));
        let replacement=Raster::from_pixels(1,1,vec![0]).unwrap();
        assert!(r.finish(&replacement,"").is_err());assert_eq!(std::fs::read(temp.0.join("completed-frame.png")).unwrap(),frame.png().unwrap());
        assert!(!temp.0.join("completed-frame.bmp").exists());
    }
    #[test]
    fn cancellation_bad_samples_and_failed_presentation_never_complete() {
        for sample in [Sample::default(),Sample{period_ms:f64::NAN,..Default::default()},Sample{backend:[Some(-1.),None,None,None],..Default::default()}] {
            let temp=Temp::new();let mut r=Recorder::new(temp.0.clone()).unwrap();
            assert!(r.finish(&Raster::from_pixels(1,1,vec![0]).unwrap(),"").is_err());assert!(r.record(sample,false).is_err());
            assert!(r.record(Sample::default(),true).is_err());assert!(!r.completed());
            assert!(!temp.0.join("completed-frame.png").exists());
            assert!(std::fs::read_to_string(temp.0.join("scope.txt")).unwrap().contains("status=incomplete\n"));
        }
        for sample in [Sample{period_ms:f64::INFINITY,..Default::default()},Sample{backend:[None,None,None,Some(-1.)],..Default::default()}] {
            let temp=Temp::new();let mut r=Recorder::new(temp.0.clone()).unwrap();assert!(r.record(sample,true).is_err());
        }
    }
    /// Chunks as (type, data), after checking the signature and every stored length and CRC.
    fn png_chunks(png:&[u8])->Vec<([u8;4],Vec<u8>)> {
        assert_eq!(&png[..8],b"\x89PNG\r\n\x1a\n");
        let (mut at,mut out)=(8,Vec::new());
        while at<png.len() {
            let length=u32::from_be_bytes(png[at..at+4].try_into().unwrap()) as usize;
            let kind:[u8;4]=png[at+4..at+8].try_into().unwrap();
            let crc=u32::from_be_bytes(png[at+8+length..at+12+length].try_into().unwrap());
            assert_eq!(crc,crate::crypto::crc32(&png[at+4..at+8+length]));
            out.push((kind,png[at+8..at+8+length].to_vec()));at+=12+length;
        }
        assert_eq!(at,png.len());out
    }
    #[test]
    fn png_capture_is_valid_sized_and_never_clobbers() {
        let temp=Temp::new();std::fs::create_dir(&temp.0).unwrap();let path=temp.0.join("capture.png");
        let frame=Raster::from_pixels(5,3,(0..15).collect()).unwrap();
        write_png_new(&path,&frame).unwrap();let first=std::fs::read(&path).unwrap();
        let chunks=png_chunks(&first);
        assert_eq!(chunks.iter().map(|c|c.0).collect::<Vec<_>>(),[*b"IHDR",*b"IDAT",*b"IEND"]);
        assert_eq!(&chunks[0].1[..8],&[0,0,0,5,0,0,0,3]);assert!(chunks[2].1.is_empty());
        assert!(write_png_new(&path,&Raster::from_pixels(1,1,vec![0]).unwrap()).is_err());
        assert_eq!(std::fs::read(&path).unwrap(),first);
        std::fs::remove_file(&path).unwrap();
        for (w,h) in [(0,0),(0,2),(2,0)] {
            assert!(write_png_new(&path,&Raster::from_pixels(w,h,Vec::new()).unwrap()).is_err());
            assert!(!path.exists(),"{w}x{h} must not create a file");
        }
    }
    #[test]
    fn existing_evidence_directory_is_preserved() {
        let temp=Temp::new();std::fs::create_dir(&temp.0).unwrap();std::fs::write(temp.0.join("keep"),b"preserve").unwrap();
        assert!(Recorder::new(temp.0.clone()).is_err());assert_eq!(std::fs::read(temp.0.join("keep")).unwrap(),b"preserve");
    }
}
