//! Ordinary world-entity operations in dx.af construction order.
use crate::{context_menu::{Action, Entry}, world_entity::{Definition, Entity}};
pub fn entries(id: u16, entity: &Entity, definition: &Definition) -> Vec<Entry<Action>> {
    let target = format!("<col={:x}>{}",15574765,definition.name);
    let mut entries=Vec::new();
    for operation in (0..5).rev() {
        if entity.operation_enabled(operation) {
            if let Some(option)=&definition.operations[operation as usize] {
                entries.push(Entry {option:option.clone(),target:target.clone(),
                    action:Action::WorldEntity{id,operation:operation as u8+1},
                    submenu:false,children:Vec::new()});
            }
        }
    }
    entries.push(Entry {option:"Examine".into(),target,
        action:Action::WorldEntity{id:definition.id as u16,operation:6},
        submenu:false,children:Vec::new()});
    entries
}
