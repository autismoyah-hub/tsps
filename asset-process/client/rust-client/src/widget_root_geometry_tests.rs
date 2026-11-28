use crate::{buffer::Reader, interface_packet::{SubInterface, Update}, interface_state::Interfaces, vm::Value, widget::Widget, Result};
fn listener(state: &mut Interfaces, id: u32, marker: Option<i32>) {
    state.runtime.get_mut(&id).unwrap().listeners.insert(1424,
        (marker.map(|v| vec![Value::Int(v)]).unwrap_or_default(), None));
}
#[test]
fn original_root_geometry_callback_boundaries() -> Result<()> {
    let mut cache = crate::disk_cache::DiskCache::open("tests/fixtures/event-script-cache", 64<<20)?;
    let items = crate::item::Definitions { items: Default::default() };
    let mut r = Reader::new(include_bytes!("../tests/fixtures/interface-root-geometry-jvm.bin"));
    for case in 0..r.u32()? {
        let destination = r.u32()?;
        let retain = r.u32()? != 0;
        let mode = r.u32()?;
        let expected = (0..r.u32()?).map(|_| {
            let mut row = [0; 14]; for v in &mut row { *v = r.u32()? as i32; } Ok(row)
        }).collect::<Result<Vec<_>>>()?;
        let final_mounts = [r.u32()? as u16, r.u32()? as u16, r.u32()? as u16];
        let retains_masks = r.u8()? != 0;
        let final_continue = r.u8()? != 0;
        let mut state = Interfaces::default();
        state.events.insert((2,-1),[123,456]);
        for group in 0..4 {
            state.install_group(group, (0..if group == 0 {3} else {1}).map(|index| {
                let mut w=Widget::empty(((group as u32)<<16)+index, if group == 0 && index < 2 {0} else {4});
                w.raw_size=[10+index as i32,0];w.size_alignment=[1,0];
                if group==0 {w.listeners[0]=Some(vec![crate::widget::Argument::Int(100+index as i32)]);if case%64>=32&&index==1 {w.parent=Some(0);}}
                w
            }).collect())?;
            for index in 0..if group == 0 {3} else {1} {
                listener(&mut state, ((group as u32)<<16)+index, Some(group as i32*10+index as i32));
            }
        }
        for runtime in state.runtime.values() {runtime.aligned.set(Some(crate::widget_layout::Geometry{position:[0,0],size:[-99,0]}));}
        state.geometry_canvas.set([765,503]);
        state.root = ((case/16)%2!=0).then_some(0);
        state.continue_widget = Some(2);
        for (parent, group) in [(0, 1), (1, 2)] {
            if group != 0 {
                state.children.insert(parent, SubInterface {parent, group, kind:0});
            }
        }
        let mut children = vec![SubInterface { parent:destination, group:3, kind:0 }];
        if retain {children.push(SubInterface {parent:1, group:2, kind:0});}
        state.apply(&mut cache, &items, &mut Default::default(), if case>=64 {Update::OpenTop(0)} else {Update::Resync {root:0,children,events:vec![]}})?;
        let mut actual = Vec::new();
        while let Some(event) = state.next_event_with_cache(&mut cache)? {
            let Value::Int(marker) = event.arguments[0] else {panic!("marker")};
            actual.push([marker,
                state.children.get(&0).map_or(0, |s| s.group as i32),
                state.children.get(&1).map_or(0, |s| s.group as i32),
                state.children.get(&2).map_or(0, |s| s.group as i32),
                i32::from(state.widgets.contains_key(&0)),
                i32::from(state.widgets.contains_key(&65536)),
                i32::from(state.widgets.contains_key(&131072)), i32::from(state.widgets.contains_key(&196608)), i32::from(state.events.contains_key(&(2,-1))), i32::from(state.continue_widget.is_some()),
                state.runtime[&0].aligned.get().map_or(-888,|g|g.size[0]),
                state.runtime[&1].aligned.get().map_or(-888,|g|g.size[0]),
                state.runtime[&2].aligned.get().map_or(-888,|g|g.size[0]),
                state.runtime[&196608].aligned.get().map_or(-888,|g|g.size[0])]);
            if actual.len() == 1 { match mode {
                1 => listener(&mut state, 2, Some(99)),
                2 => listener(&mut state, 1, None),
                3 => listener(&mut state, 0, Some(98)),
                _ => {}
            }}
        }
        assert_eq!(state.events.contains_key(&(2,-1)), retains_masks);
        assert_eq!(state.continue_widget.is_some(), final_continue);
        assert_eq!(actual, expected, "case{case} host{destination} retain{retain} mode{mode}");
        for parent in 0..3 { assert_eq!(state.children.get(&parent).map_or(0,|s|s.group), final_mounts[parent as usize]); }
    }
    assert_eq!(r.remaining(), 0);
    Ok(())
}
