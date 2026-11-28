//! Original map hover transitions; inactive widgets retain their previous icons.
use crate::map_event::Event;
pub struct Hover<K> {
    pub previous: Vec<(K, Event)>,
}
impl<K> Default for Hover<K> {
    fn default() -> Self {
        Self {
            previous: Vec::new(),
        }
    }
}
impl<K: PartialEq> Hover<K> {
    pub fn update(&mut self, active: bool, current: Vec<(K, Event)>) -> Vec<(i32, Event)> {
        if !active {
            return Vec::new();
        }
        let mut events = Vec::new();
        for (key, event) in &current {
            events.push((
                if self.previous.iter().any(|(old, _)| old == key) {
                    17
                } else {
                    15
                },
                *event,
            ));
        }
        for (key, event) in &self.previous {
            if !current.iter().any(|(new, _)| new == key) {
                events.push((16, *event));
            }
        }
        self.previous = current;
        events
    }
}
