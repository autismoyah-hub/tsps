//! Explicit diagnostic rendering only; canonical scene and gameplay stay intact.
use std::{cell::Cell,collections::{BTreeSet,HashSet}};
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub enum Mode { #[default] Full, Essential, NoFurnishings, NoFloorFinish, NoUpperArchitecture }
impl Mode {
    pub fn parse(s:&str)->Option<Self> {Some(match s {
        "full"=>Self::Full,"essential"=>Self::Essential,"no-furnishings"=>Self::NoFurnishings,
        "no-floor-finish"=>Self::NoFloorFinish,"no-upper-architecture"=>Self::NoUpperArchitecture,_=>return None,
    })}
    pub fn name(self)->&'static str {match self {
        Self::Full=>"full",Self::Essential=>"essential",Self::NoFurnishings=>"no-furnishings",
        Self::NoFloorFinish=>"no-floor-finish",Self::NoUpperArchitecture=>"no-upper-architecture",
    }}
    fn excludes(self,id:u32)->bool {match self {
        Self::Full=>false,
        Self::Essential=>(65100..=65125).contains(&id)||(64950..=64956).contains(&id)||[64713,64714].contains(&id),
        Self::NoFurnishings=>(65100..=65125).contains(&id),
        Self::NoFloorFinish=>(65040..=65048).contains(&id),
        Self::NoUpperArchitecture=>(64920..=64944).contains(&id),
    }}
}
pub struct Filter { mode:Mode,tags:HashSet<u64>,ids:BTreeSet<u32>,last:Cell<Option<(u64,bool)>> }
impl Filter {
    /// Admission must come from the exact canonical root placement check.
    /// Diagnostic omissions intentionally alter render/pick visibility only.
    pub fn new(mode:Mode,candidates:impl IntoIterator<Item=(u32,u64,bool)>)->Self {
        let mut out=Self{mode,tags:HashSet::new(),ids:BTreeSet::new(),last:Cell::new(None)};
        if mode==Mode::Full {return out;}
        for (id,tag,admitted) in candidates {
            if admitted && mode.excludes(id) {out.tags.insert(tag);out.ids.insert(id);}
        }
        out
    }
    pub fn allows(&self,view:u16,tag:u64)->bool {
        if view!=0 || self.tags.is_empty() {return true;}
        // Mesh faces are usually contiguous by placement. Membership is immutable
        // for this filter, so reuse only an exact preceding root tag decision.
        if let Some((previous,allowed))=self.last.get() {if previous==tag {return allowed;}}
        let allowed=!self.tags.contains(&tag);
        self.last.set(Some((tag,allowed)));allowed
    }
    pub fn report(&self,faces:impl IntoIterator<Item=(u16,u64)>)->Report {
        let mut source=0;let mut excluded=0;
        for (view,tag) in faces {source+=1;if !self.allows(view,tag){excluded+=1;}}
        Report{mode:self.mode,ids:self.ids.iter().copied().collect(),placements:self.tags.len(),source,excluded,..Report::default()}
    }
}
#[derive(Default)]
pub struct Report {mode:Mode,ids:Vec<u32>,placements:usize,source:usize,excluded:usize,pub excluded_draw_faces:usize,pub excluded_shadow_candidates:usize}
impl Report {
    pub fn full(source:usize)->Self {Self{source,..Self::default()}}
    pub fn json(&self)->String {format!("{{\"mode\":\"{}\",\"excluded_ids\":{:?},\"excluded_placements\":{},\"source_faces\":{},\"excluded_faces\":{},\"excluded_draw_faces\":{},\"excluded_shadow_candidates\":{}}}",self.mode.name(),self.ids,self.placements,self.source,self.excluded,self.excluded_draw_faces,self.excluded_shadow_candidates)}
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn full_is_inert_and_unadmitted_or_child_geometry_is_preserved() {
        let entries=[(65100,17,true),(65101,18,false),(1,19,true)];
        let full=Filter::new(Mode::Full,entries);
        for view in [0,1] {for tag in [17,18,19] {assert!(full.allows(view,tag));}}
        let filter=Filter::new(Mode::NoFurnishings,entries);
        assert!(!filter.allows(0,17));assert!(filter.allows(1,17));
        assert!(filter.allows(0,18));assert!(filter.allows(0,19));
        assert!(filter.allows(0,1<<63));assert!(filter.allows(0,4<<60));
        let report=filter.report([(0,17),(0,17),(1,17),(0,18)]);
        assert_eq!((report.source,report.excluded,report.placements),(4,2,1));
        assert_eq!(report.ids,[65100]);
    }
    #[test] fn explicit_categories_preserve_architecture_except_requested_group() {
        for id in 0..65536 {
            assert_eq!(Mode::NoFurnishings.excludes(id),(65100..=65125).contains(&id));
            assert_eq!(Mode::NoFloorFinish.excludes(id),(65040..=65048).contains(&id));
            assert_eq!(Mode::NoUpperArchitecture.excludes(id),(64920..=64944).contains(&id));
            assert!(!Mode::Full.excludes(id));
        }
        for id in [64713,64714,64950,64956,65100,65125] {assert!(Mode::Essential.excludes(id));}
        for id in [64712,64920,64944,64949,64957,65040,65048,65126] {assert!(!Mode::Essential.excludes(id));}
        for mode in [Mode::Full,Mode::Essential,Mode::NoFurnishings,Mode::NoFloorFinish,Mode::NoUpperArchitecture] {
            assert_eq!(Mode::parse(mode.name()),Some(mode));
        }
        assert_eq!(Mode::parse("all"),None);
    }
    #[test] fn membership_reuse_matches_direct_lookup_across_runs_and_child_interleaving() {
        let filter=Filter::new(Mode::Essential,[(65100,17,true),(64950,29,true),(65101,31,false)]);
        let expected=HashSet::from([17,29]);
        for pass in 0..32 {
            for tag in [17,17,29,31,17,u64::MAX,29,29,31] {
                for _ in 0..(pass+1) {
                    assert_eq!(filter.allows(0,tag),!expected.contains(&tag));
                    assert!(filter.allows(1,tag));
                    assert!(filter.allows(2,17));
                    assert_eq!(filter.allows(0,tag),!expected.contains(&tag));
                }
            }
        }
        let faces=[(0,17),(0,17),(1,17),(0,29),(0,31),(0,17)];
        for _ in 0..3 {
            let report=filter.report(faces);
            assert_eq!((report.source,report.excluded),(6,4));
        }
    }
}
