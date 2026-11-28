//! Topmost content hit after Java ps.ap menu barriers.
use crate::{interface_state::Interfaces, widget_layout::Layout};
pub fn target(
    state: &Interfaces,
    layout: &Layout,
    point: [i32; 2],
    pointer: [i32; 2],
    content: u16,
) -> Option<u32> {
    fn inside(point: [i32; 2], clip: [i32; 4]) -> bool {
        point[0] >= clip[0] && point[1] >= clip[1] && point[0] < clip[2] && point[1] < clip[3]
    }
    for &id in layout.order.iter().rev() {
        let placed = &layout.widgets[&id];
        if placed.hidden {
            continue;
        }
        let widget = &state.widgets[&id];
        // A viewport above the barrier rebuilds its menu, including its own barrier.
        if widget.content == content && inside(point, placed.clip) {
            return Some(id);
        }
        if widget.if3 && widget.no_click_through && inside(pointer, placed.clip) {
            return None;
        }
    }
    None
}
