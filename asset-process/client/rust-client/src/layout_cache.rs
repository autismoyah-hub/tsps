//! Exact input snapshots for layout reuse. No hash-only validity or mutation hooks.
//! Publicly mutable definitions/runtime are compared on every call. Keep this
//! field list in step with Interfaces::preview_layout_uncached and Layout::visit.
use crate::{interface_state::{Interfaces,Runtime},widget::Widget,widget_layout::{Geometry,Layout}};
use std::collections::HashMap;

#[derive(PartialEq,Eq)]
struct Definition {
    key:u32,id:u32,parent:Option<u32>,kind:u8,if3:bool,hidden:bool,
    position:[i32;2],size:[i32;2],position_alignment:[i8;2],size_alignment:[i8;2],scroll_size:[i32;2],
}
impl Definition {
    fn of(key:u32,w:&Widget)->Self {
        Self{key,id:w.id,parent:w.parent,kind:w.kind,if3:w.if3,hidden:w.hidden,position:w.raw_position,size:w.raw_size,
            position_alignment:w.position_alignment,size_alignment:w.size_alignment,scroll_size:w.scroll_size}
    }
}
#[derive(PartialEq,Eq)]
struct State {
    key:u32,logical:u32,child:i32,tree_parent:i32,bounds:[i32;2],scroll:[i32;2],aligned:Option<Geometry>,instance:u64,
}
impl State {
    fn of(key:u32,r:&Runtime)->Self {
        Self{key,logical:r.logical_id,child:r.child_index,tree_parent:r.tree_parent,bounds:r.child_bounds,
            scroll:r.scroll,aligned:r.aligned.get(),instance:r.instance}
    }
}
pub(crate) struct Cache {
    canvas:[i32;2],root:Option<u16>,hover:Option<(u32,u64)>,
    mounts:crate::interface_mounts::Mounts,dynamic:HashMap<(u32,i32),u32>,
    definitions:Vec<Definition>,runtime:Vec<State>,pub layout:Layout,
}
impl Cache {
    pub fn new(state:&Interfaces,canvas:[i32;2],hover:Option<(u32,u64)>,layout:Layout)->Self {
        Self{canvas,root:state.root,hover,mounts:state.children.clone(),dynamic:state.dynamic.clone(),
            definitions:state.widgets.iter().map(|(&id,w)|Definition::of(id,w)).collect(),
            runtime:state.runtime.iter().map(|(&id,r)|State::of(id,r)).collect(),layout}
    }
    pub fn matches(&self,state:&Interfaces,canvas:[i32;2],hover:Option<(u32,u64)>)->bool {
        self.canvas==canvas && self.root==state.root && self.hover==hover
            && self.mounts==state.children && self.dynamic==state.dynamic
            && self.definitions.len()==state.widgets.len() && self.runtime.len()==state.runtime.len()
            && self.definitions.iter().zip(state.widgets.iter()).all(|(old,(&id,w))|*old==Definition::of(id,w))
            && self.runtime.iter().zip(state.runtime.iter()).all(|(old,(&id,r))|*old==State::of(id,r))
    }
}

#[cfg(test)] mod tests {
    use super::*;
    fn state()->Interfaces {
        let mut s=Interfaces::default();
        let mut root=Widget::empty(65536,0);root.raw_size=[200,180];root.scroll_size=[400,360];
        let mut child=Widget::empty(65537,0);child.parent=Some(65536);child.raw_position=[20,30];child.raw_size=[90,80];
        s.install_group(1,vec![root,child]).unwrap();s.root=Some(1);s
    }
    #[test] fn snapshots_detect_every_layout_definition_and_runtime_input() {
        let mut s=state();let layout=s.preview_layout_uncached([300,250]).unwrap();
        let cache=Cache::new(&s,[300,250],None,layout);
        assert!(cache.matches(&s,[300,250],None));
        assert!(!cache.matches(&s,[301,250],None));
        assert!(!cache.matches(&s,[300,250],Some((65536,1))));
        macro_rules! definition {($field:ident,$value:expr)=>{{
            let old=s.widgets[&65537].$field;
            s.widgets.get_mut(&65537).unwrap().$field=$value;
            assert!(!cache.matches(&s,[300,250],None),stringify!($field));
            s.widgets.get_mut(&65537).unwrap().$field=old;
        }}}
        definition!(id,77);definition!(parent,None);definition!(kind,9);definition!(if3,!s.widgets[&65537].if3);
        definition!(hidden,true);definition!(raw_position,[7,8]);definition!(raw_size,[11,12]);
        definition!(size_alignment,[1,2]);definition!(position_alignment,[3,4]);definition!(scroll_size,[77,88]);
        macro_rules! runtime {($field:ident,$value:expr)=>{{
            let old=s.runtime[&65537].$field;
            s.runtime.get_mut(&65537).unwrap().$field=$value;
            assert!(!cache.matches(&s,[300,250],None),stringify!($field));
            s.runtime.get_mut(&65537).unwrap().$field=old;
        }}}
        runtime!(logical_id,99);runtime!(child_index,4);runtime!(tree_parent,3);runtime!(child_bounds,[1,7]);runtime!(scroll,[3,5]);runtime!(instance,777);
        let old=s.runtime[&65537].aligned.get();s.runtime[&65537].aligned.set(Some(Geometry{position:[9,8],size:[7,6]}));
        assert!(!cache.matches(&s,[300,250],None));s.runtime[&65537].aligned.set(old);
        s.dynamic.insert((65536,4),65537);assert!(!cache.matches(&s,[300,250],None));s.dynamic.clear();
        s.children.insert(65536,crate::interface_packet::SubInterface{parent:65536,group:2,kind:0});
        assert!(!cache.matches(&s,[300,250],None));s.children.clear();
        s.root=None;assert!(!cache.matches(&s,[300,250],None));s.root=Some(1);
        assert!(cache.matches(&s,[300,250],None));
        // Animation, text and event counters do not change stored layout geometry.
        s.runtime.get_mut(&65537).unwrap().frame=42;s.widgets.get_mut(&65537).unwrap().text="new text".into();
        assert!(cache.matches(&s,[300,250],None));
    }
    #[test] fn cached_layout_matches_fresh_traversal_across_mutation_and_lifecycle() {
        let mut s=state();
        for step in 0..80 {
            let canvas=[300+step%3,250];
            s.runtime.get_mut(&65536).unwrap().scroll=[step%35,step%27];
            s.widgets.get_mut(&65537).unwrap().hidden=step%7==0;
            s.widgets.get_mut(&65537).unwrap().kind=0;
            s.runtime[&65537].aligned.set(if step%4==0{None}else{Some(Geometry{position:[step,step/2],size:[90,80]})});
            let child=s.create_child(65537,0,5,true).unwrap();
            s.widgets.get_mut(&65537).unwrap().kind=if step%5==0{11}else{0};
            s.widgets.get_mut(&child).unwrap().raw_size=[10+step,12];
            for _ in 0..3 {
                let expected=s.preview_layout_uncached(canvas).unwrap();
                assert_eq!(s.preview_layout(canvas).unwrap(),expected);
                assert_eq!(s.layout(canvas).unwrap(),expected);
            }
            s.delete_child(child).unwrap();
            assert_eq!(s.preview_layout(canvas).unwrap(),s.preview_layout_uncached(canvas).unwrap());
        }
        s.root=None;assert!(s.preview_layout([300,250]).unwrap().order.is_empty());
        s.root=Some(1);assert_eq!(s.preview_layout([300,250]).unwrap(),s.preview_layout_uncached([300,250]).unwrap());
        assert!(s.preview_layout([-1,250]).is_err());
    }
}
