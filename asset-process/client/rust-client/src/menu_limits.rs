//! GameMenu(false) stores at most 64 entries in each child menu.
use crate::context_menu::Entry;
pub const SUBMENU_CAPACITY: usize = 64;
pub fn limit_submenus<T>(entries: &mut [Entry<T>]) {
    for entry in entries {
        entry.children.truncate(SUBMENU_CAPACITY);
        limit_submenus(&mut entry.children);
    }
}

/// Java reserves the first of 500 main-menu slots for Cancel. Call before sorting.
pub const MAIN_ACTION_CAPACITY: usize = 499;
pub fn limit_main<T>(entries: &mut Vec<Entry<T>>) {
    entries.truncate(MAIN_ACTION_CAPACITY);
}
