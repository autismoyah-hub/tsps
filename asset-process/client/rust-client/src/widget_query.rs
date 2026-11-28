//! Widget query cursor (original lm, script opcodes211-216).
use crate::{
    interface_state::Interfaces,
    vm::{Object, Value},
    Error, Result,
};
#[derive(Default, Debug)]
pub struct Query {
    owner: Option<(u32, u64)>,
    indices: Vec<i32>,
    cursor: usize,
}
impl Query {
    pub fn begin(
        &mut self,
        state: &Interfaces,
        mode: i32,
        owner: Option<u32>,
        child: i32,
    ) -> Result<()> {
        self.owner = owner.and_then(|id| state.runtime.get(&id).map(|r| (id, r.instance)));
        self.indices.clear();
        self.cursor = 0;
        if !(0..=2).contains(&mode) {
            return Err(Error::Invalid("widget query mode"));
        }
        let Some(owner) = owner else { return Ok(()) };
        if state.widgets.get(&owner).is_none_or(|w| w.kind != 0)
            || state
                .runtime
                .get(&owner)
                .is_none_or(|r| r.child_index != -1)
        {
            return Err(Error::Invalid("widget query owner"));
        }
        let mut slots = state
            .dynamic
            .iter()
            .filter(|((p, _), _)| *p == owner)
            .map(|((_, index), id)| (*index, state.runtime[id].tree_parent))
            .collect::<Vec<_>>();
        slots.sort_unstable();
        if mode == 0 {
            self.indices.extend(slots.iter().map(|s| s.0));
            return Ok(());
        }
        if child != -1 && !state.dynamic.contains_key(&(owner, child)) {
            return Ok(());
        }
        let mut children = std::collections::HashMap::<i32, Vec<i32>>::new();
        for (index, parent) in slots {
            children.entry(parent).or_default().push(index);
        }
        if mode == 1 {
            self.indices = children.remove(&child).unwrap_or_default();
            return Ok(());
        }
        let mut pending = children.remove(&child).unwrap_or_default();
        pending.reverse();
        while let Some(index) = pending.pop() {
            self.indices.push(index);
            if let Some(descendants) = children.remove(&index) {
                pending.extend(descendants.into_iter().rev());
            }
        }
        Ok(())
    }
    pub fn indices(&self) -> &[i32] {
        &self.indices
    }
    pub fn next_index(&mut self) -> i32 {
        match self.indices.get(self.cursor) {
            Some(&index) => {
                self.cursor += 1;
                index
            }
            None => -1,
        }
    }
    fn owner(&self, state: &Interfaces) -> Option<u32> {
        self.owner
            .filter(|(id, instance)| {
                state
                    .runtime
                    .get(id)
                    .is_some_and(|r| r.instance == *instance)
            })
            .map(|v| v.0)
    }
    pub fn next_widget(&mut self, state: &Interfaces) -> Option<u32> {
        let index = self.next_index();
        self.owner(state)
            .and_then(|owner| state.dynamic.get(&(owner, index)).copied())
    }
    pub fn filter(&mut self, state: &Interfaces, key: i32, value: &Option<Value>, default: &Value) {
        let owner = self.owner(state);
        self.indices.retain(|index| {
            owner
                .and_then(|o| state.dynamic.get(&(o, *index)))
                .is_some_and(|id| {
                    let actual = state.runtime[id].params.get(&key).unwrap_or(default);
                    // ja.ct(-1) is Java null; preserve null matching for absent string parameters.
                    if value.is_none() {
                        matches!(actual, Value::Object(Object::Null))
                    } else {
                        crate::param_def::equal(actual, value)
                    }
                })
        });
        // lm.al filters in place but intentionally leaves the iteration cursor unchanged.
    }
}
impl Query {
    pub fn execute(
        &mut self,
        opcode: u16,
        dot: bool,
        s: &mut crate::vm::Stacks,
        cache: &mut crate::disk_cache::DiskCache,
        state: &mut Interfaces,
        active: &mut [Option<u32>; 2],
    ) -> Result<bool> {
        let slot = usize::from(dot);
        match opcode {
            211 => {
                let [mode, owner, index] = crate::interface_script::ints(s)?;
                let owner = owner as u32;
                state.load_group(cache, (owner >> 16) as u16)?;
                self.begin(
                    state,
                    mode,
                    state.widgets.contains_key(&owner).then_some(owner),
                    index,
                )?;
                s.push_int(self.indices.len() as i32)?;
            }
            212 => {
                let mode = s.pop_int()?;
                let id = active[slot].ok_or(Error::Invalid("active widget query"))?;
                let r = state
                    .runtime
                    .get(&id)
                    .ok_or(Error::Invalid("active widget query"))?;
                let (owner, index) = if r.child_index == -1 {
                    (id, -1)
                } else {
                    (r.logical_id, r.child_index)
                };
                self.begin(state, mode, Some(owner), index)?;
                s.push_int(self.indices.len() as i32)?;
            }
            213 => {
                let id = self.next_widget(state);
                s.push_int(id.is_some() as i32)?;
                if id.is_some() {
                    active[slot] = id; state.select_widget(slot,id);
                }
            }
            214 => s.push_int(self.next_index())?,
            215 => s.push_object(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
                crate::vm::Array::managed(
                    crate::vm::Array::Ints(self.indices.clone()),
                    crate::script_array_storage::Element::Int(0),
                    self.indices.len(),
                    true,
                ),
            ))))?,
            216 => {
                let kind = s.pop_int()?;
                let value = crate::param_def::pop(s, kind)?;
                let key = s.pop_int()?;
                let default = state.parameter_default(cache, key)?;
                self.filter(state, key, &value, &default);
                s.push_int(self.indices.len() as i32)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
