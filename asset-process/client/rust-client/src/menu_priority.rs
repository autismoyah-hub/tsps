//! Original menu opcode classes and final stable priority ordering.
use crate::{
    context_menu::{Action, Entry},
    interface_state::Interfaces,
};
pub fn sort(entries: &mut [Entry<Action>], interfaces: &Interfaces, low_priority: &[Action]) {
    crate::menu_sort::prioritize(entries, |entry| match entry.action.scene().1 {
        Action::Interface { id, operation, .. } => {
            if operation as i32 - 1 > interfaces.widgets[&id].operation_priority {
                1007
            } else {
                57
            }
        }
        Action::Map { opcode, .. } => opcode as i32,
        Action::ExamineNpc(_) | Action::Examine { .. } => 1003,
        Action::Object { operation: 5, .. } => 1001,
        Action::WorldEntity { operation: 6, .. } => 1013,
        _ if low_priority.contains(&entry.action) => 2001,
        _ => 0,
    });
}
