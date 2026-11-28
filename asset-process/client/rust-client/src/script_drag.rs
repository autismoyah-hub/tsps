//! Script-requested drag start (GameClanChannel.bc / us.tt / uw.ag).
use crate::{
    interface_state::Interfaces,
    widget_drag::{Drag, Motion},
};
pub fn start(
    interfaces: &Interfaces,
    widget: Option<u32>,
    offset: [i32; 2],
    origin: [i32; 2],
    blocked: bool,
    entries: &[crate::context_menu::Entry<crate::context_menu::Action>],
) -> Option<Drag> {
    if blocked {
        return None;
    }
    let widget = widget?;
    let source = interfaces.widgets.get(&widget)?;
    let (parent, parent_instance) = interfaces.drag_parent_identity(widget)?;
    Some(Drag {
        instances: [
            interfaces.runtime.get(&widget)?.instance,
            parent_instance,
        ],
        widget,
        parent,
        motion: Motion {
            age: 0,
            dragging: false,
            active: true,
            offset,
            origin,
            dead_zone: source.drag_zone,
            dead_time: source.drag_threshold,
        },
        point: std::array::from_fn(|axis| origin[axis].wrapping_add(offset[axis])),
        entries: entries.to_vec(),
    })
}

/// Implicit3109 uses the selected Java object even after its array slot is cleared.
pub fn start_selected(
    interfaces: &mut Interfaces,
    active: &[Option<u32>; 2],
    dot: bool,
    offset: [i32; 2],
    origin: [i32; 2],
    entries: &[crate::context_menu::Entry<crate::context_menu::Action>],
) -> Option<Drag> {
    interfaces.sync_selected_widgets(active);
    let slot = usize::from(dot);
    let mounted = interfaces.mount_selected_widget(slot);
    let drag = start(interfaces, active[slot], offset, origin, false, entries);
    if let Some(mounted) = mounted {
        interfaces.unmount_selected_widget(mounted);
    }
    drag
}

use crate::{disk_cache::DiskCache, vm::Stacks, Error, Result};
/// Resolve the parent in the displayed tree before touching the retained source.
/// Mounting a retired static source here would mask a replacement with the same ID.
pub fn set_parent(
    interfaces: &mut Interfaces,
    active: &[Option<u32>; 2],
    dot: bool,
    explicit: bool,
    stacks: &mut Stacks,
    cache: &mut DiskCache,
) -> Result<bool> {
    let slot = usize::from(dot);
    let source = if explicit {
        let id = stacks.pop_int()? as u32;
        interfaces.load_group(cache, (id >> 16) as u16)?;
        if !interfaces.runtime.contains_key(&id) {
            return Err(Error::Invalid("script widget"));
        }
        id
    } else {
        interfaces.sync_selected_widgets(active);
        interfaces
            .selected_identity(slot)
            .ok_or(Error::Invalid("script widget"))?;
        active[slot].ok_or(Error::Invalid("active widget"))?
    };
    let [owner, index] = crate::interface_script::ints(stacks)?;
    interfaces.load_group(cache, ((owner as u32) >> 16) as u16)?;
    let parent = interfaces.checked_script_child(owner as u32, index)?;
    let reference = parent.and_then(|id| interfaces.runtime.get(&id).map(|r| (id, r.instance)));
    if explicit {
        let runtime = interfaces
            .runtime
            .get_mut(&source)
            .ok_or(Error::Invalid("script widget"))?;
        runtime.drag_parent = parent;
        runtime.drag_parent_reference = reference;
    } else {
        interfaces.set_selected_drag_parent(slot, parent, reference)?;
    }
    Ok(true)
}
