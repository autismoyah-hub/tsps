//! Interleave scene primitives while retaining unowned fallback slots.
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Primitive {
    Terrain(u16, u64),
    Model(u16, u64, usize),
}
impl Primitive {
    pub fn view(self) -> u16 {
        match self {
            Self::Terrain(view, _) | Self::Model(view, _, _) => view,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Call {
    Primitive(Primitive),
    Child(u16),
}
#[derive(Clone, Default)]
pub struct Program {
    pub members: HashSet<Primitive>,
    pub calls: Vec<Call>,
    /// Includes prepared children with no invocation this frame.
    pub children: HashSet<u16>,
}
pub fn schedule(initial: &[Primitive], programs: &BTreeMap<u16, Program>) -> Vec<Primitive> {
    let available = initial.iter().copied().collect::<HashSet<_>>();
    let mut scheduled = initial.to_vec();
    for (&view, program) in programs {
        let owned = |key: &Primitive| key.view() == view && program.members.contains(key);
        let mut remaining = scheduled.iter().filter(|key| owned(key)).count();
        let mut ordered = program.calls.iter().filter_map(|call| match call {
            Call::Primitive(key)
                if key.view() == view
                    && program.members.contains(key)
                    && available.contains(key) =>
            {
                Some(*key)
            }
            _ => None,
        });
        let mut next = Vec::with_capacity(scheduled.len());
        for key in scheduled {
            if !owned(&key) {
                next.push(key);
                continue;
            }
            remaining -= 1;
            if remaining == 0 {
                next.extend(ordered.by_ref());
            } else if let Some(key) = ordered.next() {
                next.push(key);
            }
        }
        scheduled = next;
    }
    let Some(root) = programs.get(&0).filter(|p| !p.children.is_empty()) else {
        return scheduled;
    };
    let children = root
        .children
        .iter()
        .copied()
        .filter(|&view| view != 0)
        .collect::<HashSet<_>>();
    let mut child_streams = HashMap::<u16, Vec<Primitive>>::new();
    for &key in &scheduled {
        if children.contains(&key.view()) {
            child_streams.entry(key.view()).or_default().push(key);
        }
    }
    let eligible = |key: &Primitive| {
        children.contains(&key.view()) || (key.view() == 0 && root.members.contains(key))
    };
    let mut remaining = scheduled.iter().filter(|key| eligible(key)).count();
    let mut ordered = root
        .calls
        .iter()
        .map(|call| match call {
            Call::Primitive(key)
                if key.view() == 0 && root.members.contains(key) && available.contains(key) =>
            {
                vec![*key]
            }
            Call::Child(view) => child_streams.get(view).cloned().unwrap_or_default(),
            _ => Vec::new(),
        })
        .filter(|calls| !calls.is_empty());
    scheduled
        .into_iter()
        .flat_map(|key| {
            if !eligible(&key) {
                return vec![key];
            }
            remaining -= 1;
            if remaining == 0 {
                ordered.by_ref().flatten().collect()
            } else {
                ordered.next().unwrap_or_default()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use Primitive::{Model as M, Terrain as T};
    fn program(members: Vec<Primitive>, calls: Vec<Call>, children: Vec<u16>) -> Program {
        Program {
            members: members.into_iter().collect(),
            calls,
            children: children.into_iter().collect(),
        }
    }
    #[test]
    fn child_terrain_enters_each_parent_invocation() {
        let root = T(0, 1);
        let child = T(7, 1);
        let model = M(7, 1, 0);
        let fallback = M(9, 1, 0);
        let programs = BTreeMap::from([
            (
                0,
                program(
                    vec![root],
                    vec![Call::Child(7), Call::Primitive(root), Call::Child(7)],
                    vec![7],
                ),
            ),
            (
                7,
                program(
                    vec![child, model],
                    vec![Call::Primitive(model), Call::Primitive(child)],
                    vec![],
                ),
            ),
        ]);
        assert_eq!(
            schedule(&[root, child, fallback, model], &programs),
            vec![model, child, root, fallback, model, child]
        );
    }
    #[test]
    fn uninvoked_child_is_suppressed_and_detached_view_keeps_slots() {
        let root = T(0, 1);
        let child = T(7, 1);
        let detached = T(8, 1);
        let model = M(8, 2, 3);
        let programs = BTreeMap::from([
            (0, program(vec![root], vec![Call::Primitive(root)], vec![7])),
            (
                7,
                program(vec![child], vec![Call::Primitive(child)], vec![]),
            ),
            (
                8,
                program(
                    vec![detached, model],
                    vec![Call::Primitive(model), Call::Primitive(detached)],
                    vec![],
                ),
            ),
        ]);
        assert_eq!(
            schedule(&[root, child, detached, model], &programs),
            vec![root, model, detached]
        );
    }
    #[test]
    fn child_only_root_and_culled_primitives() {
        let child = T(7, 1);
        let missing = M(0, 2, 3);
        let programs = BTreeMap::from([
            (
                0,
                program(
                    vec![missing],
                    vec![Call::Primitive(missing), Call::Child(7), Call::Child(7)],
                    vec![7],
                ),
            ),
            (
                7,
                program(vec![child], vec![Call::Primitive(child)], vec![]),
            ),
        ]);
        assert_eq!(schedule(&[child], &programs), vec![child, child]);
        assert_eq!(schedule(&[], &programs), vec![]);
    }
}
