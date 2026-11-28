use crate::{map_element::Definition, map_event::Event};
pub struct Entry {
    pub opcode: u16,
    pub event: Event,
    pub option: String,
    pub target: String,
}
/// Original cb scans hit icons in order and stops at the first with any action.
pub fn entries(
    icons: impl IntoIterator<Item = Event>,
    definitions: &std::collections::HashMap<u32, Definition>,
) -> Vec<Entry> {
    for event in icons {
        if let Some(def) = definitions.get(&(event.element as u32)) {
            let mut result = Vec::new();
            for slot in (0..5).rev() {
                if let Some(option) = &def.actions[slot] {
                    result.push(Entry {
                        opcode: 1008 + slot as u16,
                        event,
                        option: option.clone(),
                        target: def.target.clone().unwrap_or_default(),
                    });
                }
            }
            if !result.is_empty() {
                return result;
            }
        }
    }
    Vec::new()
}
