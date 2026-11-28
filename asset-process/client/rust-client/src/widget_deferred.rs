//! GameGraphicsObject.an deferred widget operations, consumed by eo's finally path.
use crate::{disk_cache::DiskCache, interface_state::Interfaces, vm::Stacks, Error, Result};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Operation {
    pub operation: u8,
    pub widget: u32,
    pub child: i32,
    pub item: i32,
}

pub fn execute(
    opcode: u16,
    dot: bool,
    stacks: &mut Stacks,
    cache: &mut DiskCache,
    interfaces: &mut Interfaces,
    active: &[Option<u32>; 2],
    queue: &mut VecDeque<Operation>,
) -> Result<bool> {
    if !matches!(opcode, 1928 | 2928) {
        return Ok(false);
    }
    interfaces.sync_selected_widgets(active);
    let mounted = if opcode == 1928 {
        interfaces.mount_selected_widget(usize::from(dot))
    } else {
        None
    };
    let result = (|| {
        let (id, operation) = if opcode == 1928 {
            let operation = stacks.pop_int()?;
            validate(operation)?;
            (
                active[usize::from(dot)].ok_or(Error::Invalid("deferred widget"))?,
                operation,
            )
        } else {
            let [widget, child, operation] = crate::interface_script::ints(stacks)?;
            validate(operation)?;
            interfaces.load_group(cache, (widget as u32 >> 16) as u16)?;
            (
                interfaces
                    .script_child(widget as u32, child)
                    .ok_or(Error::Invalid("deferred widget"))?,
                operation,
            )
        };
        let runtime = interfaces
            .runtime
            .get(&id)
            .ok_or(Error::Invalid("deferred widget"))?;
        queue.push_back(Operation {
            operation: operation as u8,
            widget: runtime.logical_id,
            child: runtime.child_index,
            item: runtime.item,
        });
        Ok(true)
    })();
    if let Some(mounted) = mounted {
        interfaces.unmount_selected_widget(mounted);
    }
    result
}
fn validate(operation: i32) -> Result<()> {
    if !(1..=10).contains(&operation) {
        return Err(Error::Invalid("deferred widget operation"));
    }
    Ok(())
}
