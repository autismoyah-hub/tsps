//! Resize callbacks from GameGraphicsObject.an and the ps.ac event drain.
use crate::{
    disk_cache::DiskCache,
    interface_state::{Interfaces, ScriptEvent},
    script_event::Context,
    vm::{Control, Stacks},
    Error, Result,
};

#[derive(Clone, Debug)]
pub struct Queued {
    pub event: ScriptEvent,
    pub(crate) before_widget: Option<u32>,
    instance: u64,
    owner: Option<u32>,
}
impl Queued {
    fn into_interface(self) -> crate::interface_events::Queued {
        crate::interface_events::Queued {
            phase: crate::interface_events::Phase::General,
            pointer_listener: None,
            widget: self.event.widget, instance: self.instance, owner: self.owner,
            arguments: self.event.arguments, context: self.event.context,
        }
    }
    pub fn valid(&self, interfaces: &Interfaces) -> bool {
        let (_, child) = self.event.context.widget.unwrap();
        // Static objects remain valid even after their group is replaced/unloaded.
        if child < 0 {
            return true;
        }
        self.owner
            .filter(|owner| interfaces.widgets.contains_key(owner))
            .and_then(|owner| interfaces.dynamic.get(&(owner, child)))
            .and_then(|id| interfaces.runtime.get(id))
            .is_some_and(|runtime| runtime.instance == self.instance)
    }
}
impl Interfaces {
    pub(crate) fn take_resizes_before(&self, id: u32) -> Vec<crate::interface_events::Queued> {
        let mut queue = self.resize_events.borrow_mut();
        let mut events = Vec::new();
        for _ in 0..queue.len() {
            let event = queue.pop_front().unwrap();
            if event.before_widget == Some(id) { events.push(event.into_interface()); }
            else { queue.push_back(event); }
        }
        events
    }
    pub fn queue_resize(&self, id: u32, depth: i32) -> Result<bool> {
        let runtime = self
            .runtime
            .get(&id)
            .ok_or(Error::Invalid("resize widget"))?;
        let Some((arguments, _)) = self.listener(id, 1427) else {
            return Ok(false);
        };
        self.resize_events.borrow_mut().push_back(Queued {
            before_widget: None,
            event: ScriptEvent {
                widget: id,
                arguments,
                context: Context {
                    widget: Some((runtime.logical_id, runtime.child_index)),
                    resize_depth: depth,
                    ..Default::default()
                },
            },
            instance: runtime.instance,
            owner: self.widgets.get(&id).and_then(|widget| widget.parent),
        });
        Ok(true)
    }
}

pub fn execute(
    opcode: u16,
    dot: bool,
    depth: i32,
    stacks: &mut Stacks,
    cache: &mut DiskCache,
    interfaces: &mut Interfaces,
    active: &[Option<u32>; 2],
) -> Result<Option<Control>> {
    if !matches!(opcode, 1927 | 2927) {
        return Ok(None);
    }
    // The original checks this even when the explicit operand/target is absent.
    if depth >= 10 {
        return Err(Error::Invalid("resize callback depth"));
    }
    interfaces.sync_selected_widgets(active);
    let mounted = (opcode == 1927)
        .then(|| interfaces.mount_selected_widget(usize::from(dot)))
        .flatten();
    let result = (|| {
        let id = if opcode == 2927 {
            let id = stacks.pop_int()? as u32;
            interfaces.load_group(cache, (id >> 16) as u16)?;
            id
        } else {
            active[usize::from(dot)].ok_or(Error::Invalid("resize widget"))?
        };
        Ok(Some(if interfaces.queue_resize(id, depth + 1)? {
            Control::Continue
        } else {
            Control::Halt
        }))
    })();
    if let Some(mounted) = mounted {
        interfaces.unmount_selected_widget(mounted);
    }
    result
}
