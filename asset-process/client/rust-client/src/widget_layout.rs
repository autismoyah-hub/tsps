//! Widget sizing and positioning (dx.as/hj.ag), with scroll-aware trees.
use crate::{widget::Widget, Error, Result};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Geometry {
    pub position: [i32; 2],
    pub size: [i32; 2],
}
pub fn align(
    widget: &Widget,
    parent: [i32; 2],
    previous: Geometry,
    aspect: [i32; 2],
) -> Result<Geometry> {
    let mut size = previous.size;
    for axis in 0..2 {
        let raw = widget.raw_size[axis];
        size[axis] = match widget.size_alignment[axis] {
            0 => raw,
            1 => parent[axis].wrapping_sub(raw),
            2 => raw.wrapping_mul(parent[axis]) >> 14,
            _ => size[axis],
        };
    }
    if widget.size_alignment[0] == 4 {
        if aspect[1] == 0 {
            return Err(Error::Invalid("widget aspect denominator"));
        }
        size[0] = aspect[0].wrapping_mul(size[1]).wrapping_div(aspect[1]);
    }
    if widget.size_alignment[1] == 4 {
        if aspect[0] == 0 {
            return Err(Error::Invalid("widget aspect numerator"));
        }
        size[1] = aspect[1].wrapping_mul(size[0]).wrapping_div(aspect[0]);
    }
    let position = std::array::from_fn(|axis| {
        let raw = widget.raw_position[axis];
        let space = parent[axis].wrapping_sub(size[axis]);
        let relative = raw.wrapping_mul(parent[axis]) >> 14;
        match widget.position_alignment[axis] {
            0 => raw,
            1 => raw.wrapping_add(space / 2),
            2 => space.wrapping_sub(raw),
            3 => relative,
            4 => relative.wrapping_add(space / 2),
            _ => space.wrapping_sub(relative),
        }
    });
    Ok(Geometry { position, size })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    pub parent: Option<u32>,
    pub kind: u8,
    pub geometry: Geometry,
    pub absolute: [i32; 2],
    pub clip: [i32; 4],
    pub scroll: [i32; 2],
    pub hidden: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    pub widgets: HashMap<u32, Placement>,
    pub order: Vec<u32>,
}
// Keep sibling order without allocating a separate Vec for every parent.
// Rebuilt per call: definitions and runtime may change freely between layouts.
struct Children {
    ends: HashMap<Option<u32>, (usize, usize)>,
    next: Vec<Option<usize>>,
}
impl Children {
    fn new(widgets: &[(&Widget, Option<u32>)]) -> Result<Self> {
        let mut out=Self {ends:HashMap::new(),next:vec![None;widgets.len()]};
        let mut ids=HashSet::with_capacity(widgets.len());
        for (i,&(w,parent)) in widgets.iter().enumerate() {
            if !ids.insert(w.id) {return Err(Error::Invalid("duplicate widget id"));}
            match out.ends.entry(parent) {
                std::collections::hash_map::Entry::Occupied(mut entry)=>{
                    let tail=&mut entry.get_mut().1;
                    out.next[*tail]=Some(i);*tail=i;
                }
                std::collections::hash_map::Entry::Vacant(entry)=>{entry.insert((i,i));}
            }
        }
        Ok(out)
    }
}
impl Layout {
    pub fn build(
        widgets: &[Widget],
        canvas: [i32; 2],
        scrolls: &HashMap<u32, [i32; 2]>,
    ) -> Result<Self> {
        let borrowed = widgets.iter().map(|w| (w, w.parent)).collect::<Vec<_>>();
        Self::build_attached(&borrowed, canvas, scrolls)
    }
    /// Borrow definitions while supplying the effective parent for attached roots.
    pub(crate) fn build_attached(
        widgets: &[(&Widget, Option<u32>)],
        canvas: [i32; 2],
        scrolls: &HashMap<u32, [i32; 2]>,
    ) -> Result<Self> {
        Self::build_attached_with_geometry(widgets, canvas, scrolls, &|_| None)
    }
    /// Paint stored geometry; only definitions without an initial alignment need sizing.
    pub(crate) fn build_attached_with_geometry(
        widgets: &[(&Widget, Option<u32>)],
        canvas: [i32; 2],
        scrolls: &HashMap<u32, [i32; 2]>,
        stored: &impl Fn(u32) -> Option<Geometry>,
    ) -> Result<Self> {
        Self::build_attached_with_runtime(widgets,canvas,&|id|scrolls.get(&id).copied().unwrap_or_default(),stored)
    }
    pub(crate) fn build_attached_with_runtime(
        widgets: &[(&Widget, Option<u32>)],
        canvas: [i32; 2],
        scrolls: &impl Fn(u32) -> [i32; 2],
        stored: &impl Fn(u32) -> Option<Geometry>,
    ) -> Result<Self> {
        if canvas.iter().any(|&v| !(0..=16384).contains(&v)) {
            return Err(Error::Invalid("interface canvas size"));
        }
        let children = Children::new(widgets)?;
        let mut out = Self {
            widgets: HashMap::with_capacity(widgets.len()),
            order: Vec::with_capacity(widgets.len()),
        };
        out.visit(
            widgets,
            &children,
            None,
            canvas,
            [0; 2],
            [0, 0, canvas[0], canvas[1]],
            false,
            scrolls,
            stored,
            0,
        )?;
        Ok(out)
    }
    fn visit(
        &mut self,
        widgets: &[(&Widget, Option<u32>)],
        children: &Children,
        parent: Option<u32>,
        parent_size: [i32; 2],
        offset: [i32; 2],
        clip: [i32; 4],
        hidden: bool,
        scrolls: &impl Fn(u32) -> [i32; 2],
        stored: &impl Fn(u32) -> Option<Geometry>,
        depth: usize,
    ) -> Result<()> {
        if depth > 128 {
            return Err(Error::Invalid("interface hierarchy depth"));
        }
        let mut next=children.ends.get(&parent).map(|&(head,_)|head);
        while let Some(i)=next {
            next=children.next[i];
            let w = widgets[i].0;
            let geometry = match stored(w.id) {
                Some(geometry) => geometry,
                None => align(w, parent_size, Geometry::default(), [1, 1])?,
            };
            let absolute =
                std::array::from_fn(|axis| offset[axis].wrapping_add(geometry.position[axis]));
            let mut own_clip = clip;
            for axis in 0..2 {
                let end = absolute[axis].wrapping_add(geometry.size[axis]);
                let (low, high) = if w.kind == 9 {
                    (
                        absolute[axis].min(end),
                        absolute[axis].max(end).saturating_add(1),
                    )
                } else {
                    (absolute[axis], end)
                };
                own_clip[axis] = own_clip[axis].max(low);
                own_clip[axis + 2] = own_clip[axis + 2].min(high);
            }
            let requested_scroll=scrolls(w.id);
            let scroll = std::array::from_fn(|axis| {
                requested_scroll[axis].clamp(
                    0,
                    w.scroll_size[axis].wrapping_sub(geometry.size[axis]).max(0),
                )
            });
            let hidden = hidden || w.hidden;
            self.widgets.insert(
                w.id,
                Placement {
                    parent,
                    kind: w.kind,
                    geometry,
                    absolute,
                    clip: own_clip,
                    scroll,
                    hidden,
                },
            );
            self.order.push(w.id);
            if matches!(w.kind,0|11) {
                let extent = std::array::from_fn(|axis| {
                    if w.scroll_size[axis] != 0 {
                        w.scroll_size[axis]
                    } else {
                        geometry.size[axis]
                    }
                });
                let offset = std::array::from_fn(|axis| absolute[axis].wrapping_sub(scroll[axis]));
                self.visit(
                    widgets,
                    children,
                    Some(w.id),
                    extent,
                    offset,
                    own_clip,
                    hidden,
                    scrolls,
                    stored,
                    depth + 1,
                )?;
            }
        }
        Ok(())
    }
    pub fn hit_test(&self, point: [i32; 2]) -> impl Iterator<Item = u32> + '_ {
        self.order.iter().rev().copied().filter(move |id| {
            let p = &self.widgets[id];
            !p.hidden
                && point[0] >= p.clip[0]
                && point[1] >= p.clip[1]
                && point[0] < p.clip[2]
                && point[1] < p.clip[3]
        })
    }
    /// Repaint a dragged subtree at its clamped location, after ordinary widgets.
    /// The hit-test layout remains at its original location.
    pub fn drag_visual(&self, id: u32, position: [i32; 2], clip: [i32; 4]) -> Self {
        let mut out = self.clone();
        let Some(source) = self.widgets.get(&id) else {
            return out;
        };
        let delta: [i32; 2] = std::array::from_fn(|a| position[a].wrapping_sub(source.absolute[a]));
        let mut moved = HashSet::new();
        let mut tail = Vec::new();
        for &current in &self.order {
            let p = &self.widgets[&current];
            if current != id && !p.parent.is_some_and(|parent| moved.contains(&parent)) {
                continue;
            }
            let parent_clip = if current == id {
                clip
            } else {
                out.widgets[&p.parent.unwrap()].clip
            };
            let placed = out.widgets.get_mut(&current).unwrap();
            for axis in 0..2 {
                placed.absolute[axis] = p.absolute[axis].wrapping_add(delta[axis]);
                let start = placed.absolute[axis];
                let end = start.wrapping_add(p.geometry.size[axis]);
                let (low, high) = if p.kind == 9 {
                    (start.min(end), start.max(end).saturating_add(1))
                } else {
                    (start, end)
                };
                placed.clip[axis] = parent_clip[axis].max(low);
                placed.clip[axis + 2] = parent_clip[axis + 2].min(high);
            }
            moved.insert(current);
            tail.push(current);
        }
        out.order.retain(|id| !moved.contains(id));
        out.order.extend(tail);
        out
    }
}

#[cfg(test)]
mod topology_tests {
    use super::*;
    fn widget()->Widget {
        let mut r=crate::buffer::Reader::new(include_bytes!("../tests/fixtures/widgets-jvm.bin"));
        r.u32().unwrap();let id=r.u32().unwrap();let n=r.u32().unwrap();
        Widget::decode(id,r.bytes(n as usize).unwrap()).unwrap()
    }
    #[test]
    fn sibling_index_matches_grouped_reference_for_interleaved_and_orphan_parents() {
        let mut seed=0x2391u32;
        for count in [0,1,31,257,6144] {
            let definitions=(0..count).map(|i|{let mut w=widget();w.id=i;w}).collect::<Vec<_>>();
            let input=definitions.iter().map(|w| {
                seed=seed.wrapping_mul(1664525).wrapping_add(1013904223);
                (w,if seed%7==0 {None}else{Some(seed%211)})
            }).collect::<Vec<_>>();
            let mut reference=HashMap::<Option<u32>,Vec<usize>>::new();
            for (i,(_,parent)) in input.iter().enumerate(){reference.entry(*parent).or_default().push(i);}
            let actual=Children::new(&input).unwrap();
            assert_eq!(actual.ends.len(),reference.len());
            for (parent,expected) in reference {
                let mut next=actual.ends.get(&parent).map(|p|p.0);let mut indices=Vec::new();
                while let Some(i)=next {indices.push(i);next=actual.next[i];assert!(indices.len()<=input.len());}
                assert_eq!(indices,expected);
            }
        }
    }
    #[test]
    fn runtime_scroll_is_live_and_preserves_hidden_descendants_and_hit_order() {
        let mut base=widget();base.kind=0;base.hidden=false;
        base.size_alignment=[0,0];base.position_alignment=[0,0];base.raw_size=[100,100];base.raw_position=[0,0];base.scroll_size=[200,200];
        let mut root=base.clone();root.id=1;root.parent=None;
        let mut child=base.clone();child.id=2;child.parent=Some(1);child.raw_position=[20,30];child.raw_size=[40,40];
        let mut leaf=child.clone();leaf.id=3;leaf.parent=Some(2);leaf.kind=4;leaf.raw_position=[5,6];leaf.raw_size=[10,10];
        let widgets=[root,child,leaf];
        let input=widgets.iter().map(|w|(w,w.parent)).collect::<Vec<_>>();
        let scroll=std::cell::Cell::new([7,9]);
        let build=||Layout::build_attached_with_runtime(&input,[100,100],&|id|if id==1{scroll.get()}else{[0,0]},&|_|None).unwrap();
        let first=build();assert_eq!(first.order,[1,2,3]);assert_eq!(first.widgets[&3].absolute,[18,27]);
        assert_eq!(first.hit_test([20,30]).collect::<Vec<_>>(),[3,2,1]);
        scroll.set([12,15]);let second=build();assert_eq!(second.widgets[&3].absolute,[13,21]);
        let mut hidden=widgets.clone();hidden[1].hidden=true;
        let layout=Layout::build(&hidden,[100,100],&HashMap::new()).unwrap();
        assert_eq!(layout.order,[1,2,3]);assert!(layout.widgets[&3].hidden);
        assert_eq!(layout.hit_test([26,37]).collect::<Vec<_>>(),[1]);
        assert!(Layout::build(&[hidden[0].clone(),hidden[0].clone()],[100,100],&HashMap::new()).is_err());
    }
}
