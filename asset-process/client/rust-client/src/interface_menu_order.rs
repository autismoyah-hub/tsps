//! Java ps.ap visits widgets in paint order and clears earlier menu entries at barriers.
use crate::{interface_state::Interfaces,widget_layout::Layout};
pub fn widgets(state:&Interfaces,layout:&Layout,point:[i32;2],pointer:[i32;2])->Vec<u32> {
    fn inside(point:[i32;2],clip:[i32;4])->bool {point[0]>=clip[0]&&point[1]>=clip[1]&&point[0]<clip[2]&&point[1]<clip[3]}
    let mut result=Vec::new();
    for &id in &layout.order {
        let placed=&layout.widgets[&id];if placed.hidden {continue;}
        let widget=&state.widgets[&id];
        if widget.if3&&widget.no_click_through&&inside(pointer,placed.clip) {result.clear();}
        if inside(point,placed.clip) {result.push(id);}
    }
    result
}
