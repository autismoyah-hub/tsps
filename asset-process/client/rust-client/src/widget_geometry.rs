//! Java's stored widget dimensions and explicit script realignment.
use crate::{
    interface_state::Interfaces,
    widget_layout::{align, Geometry},
    Error, Result,
};
use std::collections::HashSet;
impl Interfaces {
    /// hl.ae: resize the active root and descendants, optionally enqueueing size-change callbacks.
    pub fn resize_canvas(&self, canvas: [i32; 2], notify: bool) -> Result<()> {
        if canvas.iter().any(|&value| !(0..=16384).contains(&value)) {
            return Err(Error::Invalid("interface canvas size"));
        }
        self.geometry_canvas.set(canvas);
        if let Some(group) = self.root {
            self.align_root_group_notifying(group, notify)?;
        }
        Ok(())
    }
    pub(crate) fn align_root_group(&self, group: u16) -> Result<()> {
        self.align_root_group_notifying(group, false)
    }
    fn align_root_group_notifying(&self, group: u16, notify: bool) -> Result<()> {
        let base = (group as u32) << 16;
        for index in 0..self.static_group_length(group) {
            let id = base | index as u32;
            if self.widgets.get(&id).is_some_and(|w| w.parent.is_none()) {
                self.align_widget_notifying(id, self.geometry_canvas.get(), true, notify)?;
            }
        }
        Ok(())
    }

    pub(crate) fn script_geometry(&self, id: u32, _canvas: [i32; 2]) -> Result<Option<Geometry>> {
        // cg.ai/fe.bh read Java's stored fields, including their initial zeros.
        // hj.ax also reads the parent's stored size, not a freshly laid-out tree.
        Ok(self.runtime.get(&id).map(|r| r.aligned.get().unwrap_or_default()))
    }
    pub(crate) fn align_script_widget(
        &self,
        id: u32,
        canvas: [i32; 2],
        resize: bool,
    ) -> Result<()> {
        self.align_widget_notifying(id, canvas, resize, false)
    }
    fn align_widget_notifying(&self, id: u32, canvas: [i32; 2], resize: bool, notify: bool) -> Result<()> {
        if canvas.iter().any(|&value| !(0..=16384).contains(&value)) {
            return Err(Error::Invalid("interface canvas size"));
        }
        let widget = self
            .widgets
            .get(&id)
            .ok_or(Error::Invalid("script widget"))?;
        let runtime = &self.runtime[&id];
        if resize {
            // hj.ax uses the parent's displayed size, not its scroll extent.
            let parent = if runtime.child_index >= 0 {
                self.script_child(runtime.logical_id, runtime.tree_parent)
            } else {
                widget.parent
            };
            let extent = if let Some(parent) = parent {
                self.script_geometry(parent, canvas)?
                    .map_or(canvas, |g| g.size)
            } else {
                canvas
            };
            let previous = runtime.aligned.get().unwrap_or_default();
            let geometry = align(widget, extent, previous, [1, 1])?;
            runtime.aligned.set(Some(geometry));
            if notify && previous.size != geometry.size {
                self.queue_resize(id, 0)?;
            }
        }
        if widget.kind != 0 {
            return Ok(());
        }
        self.align_descendants_notifying(id, canvas, notify)
    }
    /// Remote-ready dm.af aligns descendants and queues changed-size listeners before the impression.
    pub(crate) fn align_remote_children(&self,id:u32)->Result<()> {
        self.align_descendants_notifying(id,self.geometry_canvas.get(),true)
    }
    pub(crate) fn align_widget_descendants(&self, id: u32, canvas: [i32; 2]) -> Result<()> {
        self.align_descendants_notifying(id, canvas, false)
    }
    fn align_descendants_notifying(&self, id: u32, canvas: [i32; 2], notify: bool) -> Result<()> {
        let runtime = &self.runtime[&id];
        let own_dynamic = runtime.child_index < 0
            && !self.selected_is_retired_instance(id, runtime.instance)
            && self.child_array_allocated(id);
        if runtime.child_bounds[0] > runtime.child_bounds[1] && !own_dynamic && !self.children.contains_key(&id) {
            return Ok(());
        }
        self.align_script_children(id, canvas, &mut HashSet::from([id]), 0, notify)
    }
    fn align_script_children(
        &self,
        id: u32,
        canvas: [i32; 2],
        path: &mut HashSet<u32>,
        depth: usize,
        notify: bool,
    ) -> Result<()> {
        if depth > 128 {
            return Err(Error::Invalid("interface hierarchy depth/cycle"));
        }
        let widget = &self.widgets[&id];
        let runtime = &self.runtime[&id];
        // dm.af also aligns mounted groups under non-container hosts.
        // Recursive traversal still stops at ordinary leaf widgets.
        if depth > 0 && widget.kind != 0 {
            return Ok(());
        }
        let retired_static =
            runtime.child_index < 0 && self.selected_is_retired_instance(id, runtime.instance);
        let geometry = self.script_geometry(id, canvas)?.unwrap_or_default();
        let extent = std::array::from_fn(|axis| {
            if widget.scroll_size[axis] != 0 {
                widget.scroll_size[axis]
            } else {
                geometry.size[axis]
            }
        });
        let mut targets = Vec::new();
        if runtime.child_index < 0 {
            let base = runtime.logical_id & 0xffff0000;
            let length = self.static_group_length((base >> 16) as u16) as i32;
            let end = runtime.child_bounds[1].min(length - 1);
            for index in runtime.child_bounds[0].max(0)..=end {
                let child = base | index as u32;
                if self.widgets.get(&child).is_some_and(|w| w.parent == Some(id)) {
                    targets.push(child);
                }
            }
        }
        // The current flat array is indexed in Java slot order. A retired static
        // widget's old array is not represented by the current dynamic map.
        if !retired_static {
            let length = self.child_array_length(runtime.logical_id) as i32;
            let start = if runtime.child_index < 0 { 0 } else { runtime.child_bounds[0].max(0) };
            let end = if runtime.child_index < 0 { length - 1 } else { runtime.child_bounds[1].min(length - 1) };
            for index in start..=end {
                if let Some(&child) = self.dynamic.get(&(runtime.logical_id, index)) {
                    if self.runtime[&child].tree_parent == runtime.child_index {
                        targets.push(child);
                    }
                }
            }
        }
        if let Some(sub) = self.children.get(&id) {
            let base = (sub.group as u32) << 16;
            for index in 0..self.static_group_length(sub.group) {
                let child = base | index as u32;
                if self.widgets.get(&child).is_some_and(|w| w.parent.is_none()) {
                    targets.push(child);
                }
            }
        }
        for child in targets {
            if !path.insert(child) {
                return Err(Error::Invalid("interface hierarchy depth/cycle"));
            }
            let r = &self.runtime[&child];
            let previous = r.aligned.get().unwrap_or_default();
            let geometry = align(
                &self.widgets[&child],
                extent,
                r.aligned.get().unwrap_or_default(),
                [1, 1],
            )?;
            r.aligned.set(Some(geometry));
            if notify && previous.size != geometry.size {
                self.queue_resize(child, 0)?;
            }
            self.align_script_children(child, canvas, path, depth + 1, notify)?;
            path.remove(&child);
        }
        Ok(())
    }
}

