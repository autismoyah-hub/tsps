use crate::{buffer::Reader, interface_packet::SubInterface, interface_state::Interfaces, vm::Value, widget::Widget, Result};
fn listener(state: &mut Interfaces, id: u32, marker: Option<i32>) {
    state.runtime.get_mut(&id).unwrap().listeners.insert(1424,
        (marker.map(|v| vec![Value::Int(v)]).unwrap_or_default(), None));
}
#[test]
fn original_move_geometry_callback_boundaries() -> Result<()> {
    let mut r = Reader::new(include_bytes!("../tests/fixtures/interface-move-geometry-jvm.bin"));
    for case in 0..r.u32()? {
        let source_group = r.u32()? as u16;
        let destination_group = r.u32()? as u16;
        let destination = r.u32()?;
        let mode = r.u32()?;
        let expected = (0..r.u32()?).map(|_| {
            let mut row = [0; 9]; for v in &mut row { *v = r.u32()? as i32; } Ok(row)
        }).collect::<Result<Vec<_>>>()?;
        let final_mounts = [r.u32()? as u16, r.u32()? as u16];
        let final_continue = r.u8()? != 0;
        let mut state = Interfaces::default();
        for group in 0..3 {
            state.install_group(group, (0..if group == 0 {3} else {1}).map(|index| {
                Widget::empty(((group as u32)<<16)+index, if group == 0 && index < 2 && (case / 72) % 2 == 0 {0} else {4})
            }).collect())?;
            for index in 0..if group == 0 {3} else {1} {
                listener(&mut state, ((group as u32)<<16)+index, Some(group as i32*10+index as i32));
            }
        }
        for (&id,runtime) in &state.runtime {
            runtime.aligned.set(Some(crate::widget_layout::Geometry { position:[0,0],size:[if id>>16==0 {300+(id as i32)*200}else{-99},200] }));
        }
        for (&id,widget) in &mut state.widgets {
            if id>>16!=0 { widget.raw_size=[10,0];widget.size_alignment=[1,0]; } else if case >= 144 { widget.scroll_size[0]=700+id as i32*200; }
        }
        state.root = Some(0);
        state.continue_widget = Some(2);
        for (parent, group) in [(0, source_group), (1, destination_group)] {
            if group != 0 && !(parent == 1 && destination == 0) {
                state.children.insert(parent, SubInterface {parent, group, kind:0});
            }
        }
        state.move_sub(0, destination)?;
        let mut actual = Vec::new();
        while let Some(event) = state.next_event()? {
            let Value::Int(marker) = event.arguments[0] else {panic!("marker")};
            actual.push([marker,
                state.children.get(&0).map_or(0, |s| s.group as i32),
                state.children.get(&1).map_or(0, |s| s.group as i32),
                i32::from(state.widgets.contains_key(&0)),
                i32::from(state.widgets.contains_key(&65536)),
                i32::from(state.widgets.contains_key(&131072)), i32::from(state.continue_widget.is_some()),
                state.runtime.get(&65536).map_or(-999, |r|r.aligned.get().map_or(-888,|g|g.size[0])),
                state.runtime.get(&131072).map_or(-999, |r|r.aligned.get().map_or(-888,|g|g.size[0]))]);
            if actual.len() == 1 { match mode {
                1 => listener(&mut state, 2, Some(99)),
                2 => listener(&mut state, 1, None),
                3 => listener(&mut state, 0, Some(98)),
                _ => {}
            }}
        }
        assert_eq!(state.continue_widget.is_some(), final_continue);
        assert_eq!(actual, expected, "case{case} source{source_group} dest{destination_group} host{destination} mode{mode}");
        for parent in 0..2 { assert_eq!(state.children.get(&parent).map_or(0,|s|s.group), final_mounts[parent as usize]); }
    }
    assert_eq!(r.remaining(), 0);
    Ok(())
}
