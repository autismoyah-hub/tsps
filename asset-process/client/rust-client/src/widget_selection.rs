//! Strong references for implicit selections and widgets retained across callbacks.
use crate::{
    interface_state::{Interfaces, Runtime},
    widget::Widget,
};
use std::{cell::RefCell, rc::{Rc, Weak}};
pub(crate) type Object = Rc<RefCell<Reference>>;
pub(crate) type WeakObject = Weak<RefCell<Reference>>;
pub(crate) struct Reference {
    id: u32,
    instance: u64,
    retired: Option<(Widget, Runtime)>,
}
pub(crate) struct Mounted {
    reference: Object,
    widget: Option<Widget>,
    runtime: Option<Runtime>,
}
impl Interfaces {
    pub(crate) fn select_widget(&mut self, slot: usize, id: Option<u32>) {
        self.selected_widgets[slot] = id.and_then(|id| self.hold_widget(id));
    }
    pub(crate) fn hold_widget(&mut self, id: u32) -> Option<Object> {
        let instance = self.runtime.get(&id)?.instance;
        if let Some(object) = self.held_widgets.get(&instance).and_then(Weak::upgrade) {
            return Some(object);
        }
        let object = Rc::new(RefCell::new(Reference { id, instance, retired: None }));
        self.held_widgets.insert(instance, Rc::downgrade(&object));
        Some(object)
    }
    pub(crate) fn sync_selected_widgets(&mut self, active: &[Option<u32>; 2]) {
        for slot in 0..2 {
            if self.selected_widgets[slot]
                .as_ref()
                .map(|object| object.borrow().id)
                != active[slot]
            {
                self.select_widget(slot, active[slot]);
            }
        }
    }
    pub(crate) fn retire_selected_widget(&mut self, id: u32) {
        let Some(runtime) = self.runtime.get(&id) else {
            return;
        };
        let Some(widget) = self.widgets.get(&id) else {
            return;
        };
        let held = self.held_widgets.remove(&runtime.instance).and_then(|object| object.upgrade());
        for object in self.selected_widgets.iter().flatten().chain(held.iter()) {
            let mut object = object.borrow_mut();
            if object.id == id && object.instance == runtime.instance && object.retired.is_none() {
                object.retired = Some((widget.clone(), runtime.clone()));
            }
        }
    }
    /// Temporarily expose a retired object to the existing property opcode handlers.
    /// Its replacement and the displayed tree are restored before returning to the VM.
    pub(crate) fn mount_selected_widget(&mut self, slot: usize) -> Option<Mounted> {
        let reference = self.selected_widgets[slot].as_ref()?.clone();
        self.mount_widget_object(&reference)
    }
    pub(crate) fn mount_widget_object(&mut self, reference: &Object) -> Option<Mounted> {
        let reference = reference.clone();
        let (id, widget, runtime) = {
            let object = reference.borrow();
            let (widget, runtime) = object.retired.as_ref()?;
            (object.id, widget.clone(), runtime.clone())
        };
        Some(Mounted {
            reference,
            widget: self.widgets.insert(id, widget),
            runtime: self.runtime.insert(id, runtime),
        })
    }
    pub(crate) fn unmount_selected_widget(&mut self, mounted: Mounted) {
        let id = mounted.reference.borrow().id;
        if let (Some(widget), Some(runtime)) = (self.widgets.remove(&id), self.runtime.remove(&id))
        {
            mounted.reference.borrow_mut().retired = Some((widget, runtime));
        }
        if let Some(widget) = mounted.widget {
            self.widgets.insert(id, widget);
        }
        if let Some(runtime) = mounted.runtime {
            self.runtime.insert(id, runtime);
        }
    }
}

impl Interfaces {
    pub(crate) fn selected_identity(&self, slot: usize) -> Option<(u32, i32)> {
        let object = self.selected_widgets[slot].as_ref()?.borrow();
        let runtime = if let Some((_, runtime)) = &object.retired {
            runtime
        } else {
            self.runtime
                .get(&object.id)
                .filter(|runtime| runtime.instance == object.instance)?
        };
        Some((runtime.logical_id, runtime.child_index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replacement_and_shared_retired_selection_remain_distinct() {
        let mut state = Interfaces::default();
        state.install_group(0, vec![Widget::empty(0, 4)]).unwrap();
        state.sync_selected_widgets(&[Some(0), Some(0)]);
        state.widgets.get_mut(&0).unwrap().hidden = true;
        state.runtime.get_mut(&0).unwrap().scroll = [23, 42];
        state.retire_selected_widget(0);
        let mut replacement = state.runtime[&0].clone();
        replacement.instance += 1;
        replacement.scroll = [0, 0];
        state.runtime.insert(0, replacement);
        state.widgets.insert(0, Widget::empty(0, 4));
        state.select_widget(1, Some(0));
        let old = state.mount_selected_widget(0).unwrap();
        assert!(state.widgets[&0].hidden);
        assert_eq!(state.runtime[&0].scroll, [23, 42]);
        state.runtime.get_mut(&0).unwrap().scroll = [31, 52];
        state.widgets.get_mut(&0).unwrap().text = "retired".into();
        state.unmount_selected_widget(old);
        assert!(!state.widgets[&0].hidden);
        assert_eq!(state.runtime[&0].scroll, [0, 0]);
        assert!(state.widgets[&0].text.is_empty());
        assert!(state.mount_selected_widget(1).is_none());
        let old = state.mount_selected_widget(0).unwrap();
        assert_eq!(state.widgets[&0].text, "retired");
        assert_eq!(state.runtime[&0].scroll, [31, 52]);
        state.unmount_selected_widget(old);
        // Both registers select the same current object, then it is removed entirely.
        state.select_widget(0, Some(0));
        state.retire_selected_widget(0);
        state.widgets.clear();
        state.runtime.clear();
        let a = state.mount_selected_widget(0).unwrap();
        state.widgets.get_mut(&0).unwrap().text = "shared".into();
        state.unmount_selected_widget(a);
        assert!(state.widgets.is_empty());
        assert!(state.runtime.is_empty());
        let b = state.mount_selected_widget(1).unwrap();
        assert_eq!(state.widgets[&0].text, "shared");
        state.unmount_selected_widget(b);
        assert!(state.widgets.is_empty());
        assert!(state.runtime.is_empty());
    }

    #[test]
    fn failed_property_opcode_restores_replacement_and_retains_selection() {
        let mut state = Interfaces::default();
        state.install_group(0, vec![Widget::empty(0, 4)]).unwrap();
        let mut active = [Some(0), None];
        state.sync_selected_widgets(&active);
        state.widgets.get_mut(&0).unwrap().hidden = true;
        state.retire_selected_widget(0);
        let mut replacement = state.runtime[&0].clone();
        replacement.instance += 1;
        state.runtime.insert(0, replacement);
        state.widgets.insert(0, Widget::empty(0, 4));
        let mut cache = crate::disk_cache::DiskCache::open(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/widget-selection-vm-cache"
            ),
            1 << 20,
        )
        .unwrap();
        let items = crate::item::Definitions {
            items: Default::default(),
        };
        let mut stack = crate::vm::Stacks::default();
        // 1003 requires an integer; stack underflow must still restore the current tree.
        assert!(crate::interface_script::execute(
            1003,
            false,
            &mut stack,
            &mut cache,
            &mut state,
            &mut active,
            [765, 503],
            &items
        )
        .is_err());
        assert!(!state.widgets[&0].hidden);
        assert!(crate::interface_script::execute(
            1504,
            false,
            &mut stack,
            &mut cache,
            &mut state,
            &mut active,
            [765, 503],
            &items
        )
        .unwrap());
        assert_eq!(stack.pop_int().unwrap(), 1);
        assert!(!state.widgets[&0].hidden);
    }

    #[test]
    fn deleted_dynamic_selection_keeps_protocol_identity() {
        let mut state = Interfaces::default();
        state.install_group(0, vec![Widget::empty(0, 0)]).unwrap();
        let id = state.create_child(0, 7, 4, false).unwrap();
        state.select_widget(0, Some(id));
        assert_eq!(state.selected_identity(0), Some((0, 7)));
        state.delete_children(0);
        assert_eq!(state.selected_identity(0), Some((0, 7)));
        assert!(!state.widgets.contains_key(&id));
        let replacement = state.create_child(0, 7, 4, false).unwrap();
        state.select_widget(1, Some(replacement));
        assert_eq!(state.selected_identity(0), Some((0, 7)));
        assert_eq!(state.selected_identity(1), Some((0, 7)));
        assert!(!Rc::ptr_eq(
            state.selected_widgets[0].as_ref().unwrap(),
            state.selected_widgets[1].as_ref().unwrap()
        ));
    }
}

impl Interfaces {
    pub(crate) fn selected_navigation(
        &self,
        slot: usize,
    ) -> Option<(Option<u32>, crate::widget_navigation::Selected)> {
        let object = self.selected_widgets[slot].as_ref()?.borrow();
        let (widget, runtime) = if let Some((widget, runtime)) = &object.retired {
            (widget, runtime)
        } else {
            let runtime = self
                .runtime
                .get(&object.id)
                .filter(|runtime| runtime.instance == object.instance)?;
            (self.widgets.get(&object.id)?, runtime)
        };
        let owner = if runtime.child_index >= 0 {
            Some(runtime.logical_id)
        } else {
            widget.parent
        };
        Some((
            owner,
            crate::widget_navigation::Selected {
                index: runtime.child_index,
                parent: runtime.tree_parent,
                kind: widget.kind,
            },
        ))
    }
}

impl Interfaces {
    pub(crate) fn selected_delete_node(
        &self,
        slot: usize,
    ) -> Option<(u32, crate::widget_delete::Node)> {
        let object = self.selected_widgets[slot].as_ref()?.borrow();
        let (widget, runtime) = if let Some((widget, runtime)) = &object.retired {
            (widget, runtime)
        } else {
            let runtime = self
                .runtime
                .get(&object.id)
                .filter(|runtime| runtime.instance == object.instance)?;
            (self.widgets.get(&object.id)?, runtime)
        };
        Some((
            runtime.logical_id,
            crate::widget_delete::Node {
                index: runtime.child_index,
                parent: runtime.tree_parent,
                kind: widget.kind,
                bounds: runtime.child_bounds,
            },
        ))
    }
}

impl Interfaces {
    /// Capture the object generation; unloading its group must not transfer the marker.
    pub fn set_continue_widget(&mut self, id: Option<u32>) {
        self.continue_widget = id;
        self.continue_instance = id.and_then(|id| self.runtime.get(&id).map(|r| (id, r.instance)));
    }
    pub(crate) fn set_selected_continue_widget(&mut self, slot: usize) {
        if let Some(reference) = &self.selected_widgets[slot] {
            let object = reference.borrow();
            self.continue_widget = Some(object.id);
            self.continue_instance = Some((object.id, object.instance));
        }
    }
    /// Java us.bf compares the marked object with the object currently being drawn.
    pub fn is_continue_widget(&self, id: u32) -> bool {
        self.continue_widget == Some(id)
            && self
                .continue_instance
                .filter(|&(marked, _)| marked == id)
                .is_none_or(|(_, instance)| {
                    self.runtime
                        .get(&id)
                        .is_some_and(|r| r.instance == instance)
                })
    }
}

#[cfg(test)]
mod continue_tests {
    use super::*;
    use crate::{buffer::Reader, disk_cache::DiskCache, vm::Stacks, Result};
    #[test]
    fn continue_marker_matches_original_object_identity() -> Result<()> {
        let mut cache = DiskCache::open("tests/fixtures/widget-selection-vm-cache", 1 << 20)?;
        let items = crate::item::Definitions {
            items: Default::default(),
        };
        let mut interfaces = Interfaces::default();
        let mut active = [None; 2];
        let mut child = Widget::empty(1, 0);
        child.parent = Some(0);
        interfaces.install_group(0, vec![Widget::empty(0, 0), child])?;
        interfaces.root = Some(0);
        let mut r = Reader::new(include_bytes!(
            "../tests/fixtures/widget-continue-marker-jvm.bin"
        ));
        let total = r.u32()?;
        for step in 0..total {
            let n = r.u32()?;
            let op = r.u32()? as u16;
            let dot = r.u32()? != 0;
            let mut stack = Stacks::default();
            for _ in 2..n {
                stack.push_int(r.u32()? as i32)?;
            }
            if op == 9000 {
                interfaces.children.insert(
                    987,
                    crate::interface_packet::SubInterface {
                        parent: 987,
                        group: 0,
                        kind: 0,
                    },
                );
                interfaces.close(987)?;
                let mut child = Widget::empty(1, 0);
                child.parent = Some(0);
                interfaces.install_group(0, vec![Widget::empty(0, 0), child])?;
            } else if op == 9001 {
                interfaces.set_continue_widget(None);
            } else if op == 1121 || op == 2121 {
                let identity = if op == 1121 {
                    interfaces.sync_selected_widgets(&active);
                    let identity = interfaces.selected_identity(usize::from(dot)).unwrap();
                    interfaces.set_selected_continue_widget(usize::from(dot));
                    identity
                } else {
                    let id = stack.pop_int()? as u32;
                    let runtime = &interfaces.runtime[&id];
                    let identity = (runtime.logical_id, runtime.child_index);
                    interfaces.set_continue_widget(Some(id));
                    identity
                };
                let packet = crate::interface_action::resume(identity.0, identity.1);
                stack.push_int(packet.opcode as i32)?;
                for byte in packet.payload {
                    stack.push_int(byte as i32)?;
                }
            } else {
                assert!(crate::interface_script::execute(
                    op,
                    dot,
                    &mut stack,
                    &mut cache,
                    &mut interfaces,
                    &mut active,
                    [765, 503],
                    &items
                )?);
            }

            let expected = (0..r.u32()?)
                .map(|_| r.u32().map(|v| v as i32))
                .collect::<Result<Vec<_>>>()?;
            for value in expected.into_iter().rev() {
                assert_eq!(stack.pop_int()?, value, "step {step} opcode {op}");
            }
            assert!(stack.pop_int().is_err());
            assert_eq!(
                interfaces.continue_widget.is_none(),
                r.u8()? != 0,
                "availability step {step}"
            );
            for id in 0..2 {
                assert_eq!(
                    interfaces.is_continue_widget(id),
                    r.u8()? != 0,
                    "waiting id {id} step {step} opcode {op}"
                );
            }
        }
        assert_eq!(r.remaining(), 0);
        println!("{total} continue-marker identity transitions match Java");
        Ok(())
    }
}

impl Interfaces {
    pub(crate) fn set_selected_drag_parent(
        &mut self,
        slot: usize,
        parent: Option<u32>,
        reference: Option<(u32, u64)>,
    ) -> crate::Result<()> {
        let object = self.selected_widgets[slot]
            .as_ref()
            .ok_or(crate::Error::Invalid("script widget"))?;
        let mut object = object.borrow_mut();
        let id = object.id;
        let instance = object.instance;
        let runtime = if let Some((_, runtime)) = &mut object.retired {
            runtime
        } else {
            self.runtime
                .get_mut(&id)
                .filter(|r| r.instance == instance)
                .ok_or(crate::Error::Invalid("script widget"))?
        };
        runtime.drag_parent = parent;
        runtime.drag_parent_reference = reference;
        Ok(())
    }
    pub fn drag_parent_identity(&self, id: u32) -> Option<(u32, u64)> {
        if let Some(parent) = self.drag_ancestor(id) {
            return self.runtime.get(&parent).map(|r| (parent, r.instance));
        }
        let runtime = self.runtime.get(&id)?;
        let parent = runtime.drag_parent?;
        runtime
            .drag_parent_reference
            .filter(|&(id, _)| id == parent)
            .or_else(|| self.runtime.get(&parent).map(|r| (parent, r.instance)))
    }
}

impl Interfaces {
    pub(crate) fn selected_is_retired_instance(&self,id:u32,instance:u64)->bool {
        self.selected_widgets.iter().flatten().any(|object| {
            let object=object.borrow();object.id==id && object.instance==instance && object.retired.is_some()
        })
    }
}
