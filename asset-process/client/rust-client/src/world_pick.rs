//! Original cf.cu / GameFloatProjection picking policy, using Java scene tags.
//! These tags are distinct from the native renderer's current geometry tags.
const NON_SELECTABLE: u64 = 1 << 19;

pub fn tag(plane: i32, x: i32, z: i32, kind: u8, non_selectable: bool, id: u32, view: i32) -> u64 {
    (x as u64 & 127)
        | ((z as u64 & 127) << 7)
        | ((plane as u64 & 3) << 14)
        | ((kind as u64 & 7) << 16)
        | if non_selectable { NON_SELECTABLE } else { 0 }
        | ((id as u64) << 20)
        | ((view as u64 & 4095) << 52)
}

pub fn selectable(tag: u64) -> bool {
    tag != 0 && tag & NON_SELECTABLE == 0
}

/// Adapt original tag substitution to native geometry IDs without conflating
/// the child scene's actor index with an index in its parent.
pub fn native_target(
    parent: u64,
    source: u64,
    native: u64,
    view: u16,
    entity: u16,
    parent_view: u16,
) -> Option<(u64, u16)> {
    let selected = Projection { parent }.replace(source);
    if selected == source {
        Some((native, view))
    } else if (selected >> 16) & 7 == 4 {
        Some((0x0800_0000_0000_0000 | entity as u64, parent_view))
    } else {
        None
    }
}

/// Select the enclosing entity tag before traversing its child scene.
pub fn parent_tag(mode: u8, local: bool, interactive: bool, hidden: bool, entity_tag: u64) -> u64 {
    let mode = if local { 1 } else { mode };
    if mode == 3 {
        5 << 16
    } else if interactive && !hidden && mode != 1 {
        (entity_tag & !NON_SELECTABLE) | if mode != 0 { NON_SELECTABLE } else { 0 }
    } else {
        0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Projection {
    pub parent: u64,
}

impl Projection {
    pub fn replace(&self, source: u64) -> u64 {
        if self.parent == 0 || (selectable(source) && !selectable(self.parent)) {
            source
        } else {
            self.parent & !NON_SELECTABLE
        }
    }

    /// Original qk: after a hit retain only the selection flag and sentinel kind.
    pub fn picked(&mut self) {
        if self.parent != 0 {
            self.parent = (self.parent & NON_SELECTABLE) | (5 << 16);
        }
    }
}

/// dx.af chooses a menu plane independently of the scene's draw plane.
/// Definition opcode 2 (currently named `flags`) is the child interaction plane.
pub fn menu_plane(
    root_plane: i32,
    local_view: u16,
    target_view: u16,
    worlds: &std::collections::HashMap<u16, crate::world_entity::State>,
) -> Option<i32> {
    let entity = |view| worlds.get(&0)?.entities.get(&view);
    if target_view == local_view {
        return if target_view == 0 {
            Some(root_plane)
        } else {
            Some(entity(target_view)?.plane as i32)
        };
    }
    if local_view != 0 {
        let local = entity(local_view)?;
        if local.plane != local.definition.as_ref()?.flags {
            return None;
        }
        if target_view == 0 {
            return Some(root_plane);
        }
    }
    Some(entity(target_view)?.definition.as_ref()?.flags as i32)
}
