//! Pointer transitions and timer events for active interface trees.
use crate::{
    input::Input, interface_state::Interfaces, script_event::Context, vm::Value,
    widget_layout::Layout,
};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase { Timer, Deferred, General }
pub struct Queued {
    pub phase: Phase,
    pub pointer_listener: Option<u16>,
    pub widget: u32,
    pub instance: u64,
    pub owner: Option<u32>,
    pub arguments: Vec<Value>,
    pub context: Context,
}
impl Queued {
    pub fn valid(&self, state: &Interfaces) -> bool {
        let Some((_, child)) = self.context.widget else {
            return true;
        };
        // ps.ac retains static objects, but checks the current dynamic child slot.
        if child < 0 {
            return true;
        }
        self.owner.filter(|owner| state.widgets.contains_key(owner))
            .and_then(|owner| state.dynamic.get(&(owner, child)))
            .and_then(|id| state.runtime.get(id))
            .is_some_and(|runtime| runtime.instance == self.instance)
    }
}
#[derive(Default)]
pub struct Events {
    instances: HashMap<u32, u64>,
    hovered: HashSet<u32>,
    clicked: HashSet<u32>,
    changes: [Changes; 3],
    observed: HashMap<u32, [u64; 3]>,
    transmitted: HashMap<u16, u64>,
    last_transmit: HashMap<u32, u64>,
    epoch: u64,
    /// Objects whose 1400 listener the latest collect queued; cleared per collect.
    #[cfg(feature = "private-test-adapter")]
    last_pressed: Vec<(u32, u64)>,
}
#[derive(Default)]
struct Changes {
    generation: u64,
    ids: [i32; 32],
}
impl Events {
    /// Widget and object instance for each 1400 event the latest `collect`
    /// queued. A later no_click_through widget under the pointer can still cancel
    /// such an event in the same collect, and nothing here shows a listener ran:
    /// treat it as queued-event provenance only.
    #[cfg(feature = "private-test-adapter")]
    pub fn last_pressed(&self) -> &[(u32, u64)] {
        &self.last_pressed
    }
    /// Varp, inventory and stat listeners use the client's 32-change history.
    pub fn changed(&mut self, opcode: u16, id: i32) {
        if matches!(opcode, 1418 | 1420 | 1421 | 1422 | 1425 | 1426 | 1428 | 1429) {
            self.transmitted.insert(opcode, self.epoch + 1);
            return;
        }
        let channel = match opcode {
            1407 => 0,
            1414 => 1,
            1415 => 2,
            _ => return,
        };
        let changes = &mut self.changes[channel];
        changes.ids[(changes.generation & 31) as usize] = id;
        changes.generation = changes.generation.wrapping_add(1);
    }
    pub fn collect(
        &mut self,
        state: &Interfaces,
        layout: &Layout,
        input: &Input,
        mouse_camera: bool,
        suppress_pointer: bool,
    ) -> Vec<Queued> {
        #[cfg(feature = "private-test-adapter")]
        self.last_pressed.clear();
        let mut events: Vec<Queued> = Vec::new();
        let mut timers = Vec::new();
        let mut deferred = Vec::new();
        let allowed = layout.hit_test(input.pointer).into_iter().collect::<HashSet<_>>();
        let press_allowed = layout.hit_test(input.press_position).into_iter().collect::<HashSet<_>>();
        let buttons = if mouse_camera { 1 } else { 5 };
        for &id in &layout.order {
            let w = &state.widgets[&id];
            let p = &layout.widgets[&id];
            if !w.if3 || p.hidden {
                continue;
            }
            events.extend(state.take_resizes_before(id));
            // pb.pg cancels prior general mouse events, but leaves click state and
            // deferred release/leave events intact. Only canceled events clear hover.
            if allowed.contains(&id) {
                if w.no_click_through {
                    events.retain(|event| {
                        if event.pointer_listener.is_some() {
                            self.hovered.remove(&event.widget);
                            false
                        } else { true }
                    });
                } else if w.no_scroll_through {
                    events.retain(|event| event.pointer_listener != Some(1417));
                }
            }
            let runtime = &state.runtime[&id];
            if self.instances.insert(id, runtime.instance) != Some(runtime.instance) {
                // Pointer flags and transmit observations belong to each GameWidget object.
                self.hovered.remove(&id);
                self.clicked.remove(&id);
                self.observed.remove(&id);
                self.last_transmit.remove(&id);
            }
            let mut queue = |op, point: [i32; 2], wheel: bool| {
                if let Some((arguments, _)) = state.listener(id, op) {
                    let target = match op {
                        1408 => &mut timers,
                        1402 | 1404 => &mut deferred,
                        _ => &mut events,
                    };
                    target.push(Queued {
                        pointer_listener: matches!(op,1400|1401|1402|1403|1404|1411|1412|1417).then_some(op),
                        phase: match op {1408=>Phase::Timer,1402|1404=>Phase::Deferred,_=>Phase::General},
                        widget: id,
                        instance: runtime.instance,
                        owner: w.parent,
                        arguments,
                        context: Context {
                            widget: Some((runtime.logical_id, runtime.child_index)),
                            mouse: [
                                if wheel { 0 } else { point[0].wrapping_sub(p.absolute[0]) },
                                if wheel {
                                    input.wheel
                                } else {
                                    point[1].wrapping_sub(p.absolute[1])
                                },
                            ],
                            ..Default::default()
                        },
                    });
                }
            };
            if runtime.remote_changed.replace(false) { queue(1434, p.absolute, false); }
            // GameWidget.ec gates pointer and transmit history. Cleared script
            // listeners retain an empty runtime entry, matching its sticky flag.
            if runtime.listeners.is_empty() && w.listeners.iter().all(Option::is_none) {
                continue;
            }
            let over = allowed.contains(&id) && !suppress_pointer;
            let held = over && input.buttons & buttons != 0;
            let pressed = press_allowed.contains(&id)
                && !suppress_pointer
                && input.pressed_buttons & buttons != 0;
            // ps.ap queues wheel callbacks before menu/drag pointer suppression.
            if allowed.contains(&id) && input.wheel != 0 {
                queue(1417, input.pointer, true);
            }
            if !self.clicked.contains(&id) && pressed {
                self.clicked.insert(id);
                #[cfg(feature = "private-test-adapter")]
                if state.listener(id, 1400).is_some() {
                    self.last_pressed.push((id, runtime.instance));
                }
                queue(1400, input.press_position, false);
            }
            if self.clicked.contains(&id) && held {
                queue(1411, input.pointer, false);
            }
            if self.clicked.contains(&id) && !held {
                self.clicked.remove(&id);
                queue(1402, input.pointer, false);
            }
            if held {
                queue(1401, input.pointer, false);
            }
            if !self.hovered.contains(&id) && over {
                self.hovered.insert(id);
                queue(1403, input.pointer, false);
            }
            if self.hovered.contains(&id) && over {
                queue(1412, input.pointer, false);
            }
            if self.hovered.contains(&id) && !over {
                self.hovered.remove(&id);
                queue(1404, input.pointer, false);
            }
            queue(1408, p.absolute, false);
            for opcode in [1419, 1430, 1431] {
                if let Some((arguments, _)) = state.listener(id, opcode) {
                    let keys = match opcode {
                        1419 => input.key_events.clone(),
                        1430 => input.pressed_order.iter().map(|&key| (key, 0)).collect(),
                        _ => input.released_order.iter().map(|&key| (key, 0)).collect(),
                    };
                    for (key_code, key_char) in keys {
                        events.push(Queued {
                            pointer_listener: None,
                            phase: Phase::General,
                            widget: id,
                            instance: runtime.instance,
                            owner: w.parent,
                            arguments: arguments.clone(),
                            context: Context {
                                widget: Some((runtime.logical_id, runtime.child_index)),
                                key_code,
                                key_char,
                                ..Default::default()
                            },
                        });
                    }
                }
            }
            let observed = self.observed.entry(id).or_default();
            for (channel, opcode) in [1407, 1414, 1415].into_iter().enumerate() {
                let changes = &self.changes[channel];
                if let Some((arguments, triggers)) = state.listener(id, opcode) {
                    let count = changes.generation.wrapping_sub(observed[channel]);
                    if count > 0
                        && (count > 32
                            || triggers.as_ref().is_none_or(|ids| {
                                (0..count).any(|n| {
                                    ids.contains(
                                        &changes.ids
                                            [(observed[channel].wrapping_add(n) & 31) as usize],
                                    )
                                })
                            }))
                    {
                        events.push(Queued {
                            pointer_listener: None,
                            phase: Phase::General,
                            widget: id,
                            instance: runtime.instance,
                            owner: w.parent,
                            arguments,
                            context: Context {
                                widget: Some((runtime.logical_id, runtime.child_index)),
                                ..Default::default()
                            },
                        });
                    }
                    observed[channel] = changes.generation;
                }
            }
            // ps compares chat/friend timestamps against a shared per-widget watermark,
            // then advances that watermark even when no listener is installed.
            // A fresh GameWidget.gc is -1, so even initial zero timestamps are newer.
            let previous = self.last_transmit.get(&id).copied();
            for opcode in [1418, 1420, 1421, 1428, 1429, 1425, 1426, 1422] {
                if previous.is_none_or(|previous| {
                    self.transmitted.get(&opcode).copied().unwrap_or(0) > previous
                }) {
                    if let Some((arguments, _)) = state.listener(id, opcode) {
                        events.push(Queued {
                            pointer_listener: None,
                            phase: Phase::General,
                            widget: id,
                            instance: runtime.instance,
                            owner: w.parent,
                            arguments,
                            context: Context {
                                widget: Some((runtime.logical_id, runtime.child_index)),
                                ..Default::default()
                            },
                        });
                    }
                }
            }
            self.last_transmit.insert(id, self.epoch + 1);
        }
        self.epoch += 1;
        self.last_transmit
            .retain(|id, _| state.widgets.contains_key(id));
        self.instances.retain(|id, _| state.widgets.contains_key(id));
        self.hovered.retain(|id| state.widgets.contains_key(id));
        self.clicked.retain(|id| state.widgets.contains_key(id));
        self.observed.retain(|id, _| state.widgets.contains_key(id));
        // ps.ac drains pb.af (timers), pb.ae (release/leave), then pb.ag.
        // Each queue preserves collection order across the entire widget tree.
        timers.extend(deferred);
        timers.extend(events);
        timers
    }
}

/// Each script has an independent error boundary, as in the original event pump.
pub fn dispatch(
    events: Vec<Queued>,
    mut run: impl FnMut(Queued) -> crate::Result<u64>,
) -> (u64, Vec<crate::Error>) {
    let mut instructions = 0;
    let mut errors = Vec::new();
    for event in events {
        match run(event) {
            Ok(count) => instructions += count,
            Err(error) => errors.push(error),
        }
    }
    (instructions, errors)
}

/// Drain with ps.ac look-ahead: appends made by the last callback wait for the next drain.
pub fn drain_prefetched<T>(queue: &mut std::collections::VecDeque<T>, mut run: impl FnMut(T, &mut std::collections::VecDeque<T>)) {
    let mut current = queue.pop_front();
    while let Some(event) = current {
        current = queue.pop_front();
        run(event, queue);
    }
}
