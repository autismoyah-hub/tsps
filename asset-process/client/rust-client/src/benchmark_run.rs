//! Explicit profiling-only steady-state window; no gameplay timing changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event { Waiting, Warmed, Sample, Complete { first: u64, last: u64 } }
pub struct Run {
    warmup: u32,
    target: u32,
    warmed: u32,
    sampled: u32,
    first: u64,
    pub resets: u32,
}
impl Run {
    pub fn new(warmup:u32,target:u32)->Self {
        assert!(warmup>0 && target>0);
        Self{warmup,target,warmed:0,sampled:0,first:0,resets:0}
    }
    pub fn sample_index(&self)->u32 {self.sampled}
    pub fn observe(&mut self,ready:bool,frame:u64)->Event {
        if !ready {
            if self.warmed!=0 || self.sampled!=0 {self.resets+=1;}
            self.warmed=0;self.sampled=0;self.first=0;
            return Event::Waiting;
        }
        if self.warmed<self.warmup {
            self.warmed+=1;
            return if self.warmed==self.warmup {Event::Warmed}else{Event::Waiting};
        }
        if self.sampled==0 {self.first=frame;}
        self.sampled+=1;
        if self.sampled==self.target {Event::Complete{first:self.first,last:frame}}else{Event::Sample}
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn excludes_warmup_and_restarts_after_readiness_loss() {
        let mut run=Run::new(2,3);
        assert_eq!(run.observe(false,1),Event::Waiting);
        assert_eq!(run.observe(true,2),Event::Waiting);
        assert_eq!(run.observe(true,3),Event::Warmed);
        assert_eq!(run.sample_index(),0);
        assert_eq!(run.observe(true,4),Event::Sample);
        assert_eq!(run.sample_index(),1);
        assert_eq!(run.observe(false,5),Event::Waiting);
        assert_eq!(run.resets,1);assert_eq!(run.sample_index(),0);
        assert_eq!(run.observe(true,6),Event::Waiting);
        assert_eq!(run.observe(true,7),Event::Warmed);
        assert_eq!(run.observe(true,8),Event::Sample);
        assert_eq!(run.observe(true,9),Event::Sample);
        assert_eq!(run.observe(true,10),Event::Complete{first:8,last:10});
    }
}
