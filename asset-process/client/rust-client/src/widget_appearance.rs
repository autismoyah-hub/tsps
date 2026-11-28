//! Widget-owned player composition (GameWidget.bk and GamePlayerComposition.ar/am/gt).
use crate::{
    identity_kit::Kit,
    item::{Definition, Definitions},
    player_model::{Appearance, KIT_SLOTS},
    Error, Result,
};
use std::collections::HashMap;
#[derive(Clone, Debug)]
pub struct Composition {
    pub appearance: Appearance,
    /// Script setters retain signed integers; unlike appearance packets they are not bytes.
    pub colors: [i32; 5],
}
impl Composition {
    pub fn snapshot(player: &Appearance, copy_objects: bool) -> Self {
        let mut appearance = player.clone();
        if !copy_objects {
            appearance.equipment = appearance.base_equipment;
        }
        Self {
            colors: player.colors.map(i32::from),
            appearance,
        }
    }
    pub fn color(&mut self, index: i32, color: i32) -> Result<()> {
        *self
            .colors
            .get_mut(index as usize)
            .ok_or(Error::Invalid("widget appearance color index"))? = color;
        Ok(())
    }
    pub fn equip(&mut self, item: &Definition) -> Result<()> {
        let [slot, hidden1, hidden2] = item.wear_positions;
        if !(0..12).contains(&slot) || [hidden1, hidden2].iter().any(|&s| !(-1..12).contains(&s)) {
            return Err(Error::Invalid("widget appearance item slots"));
        }
        self.appearance.equipment[slot as usize] = item.id.wrapping_add(2048);
        for hidden in [hidden1, hidden2] {
            if hidden >= 0 {
                self.appearance.equipment[hidden as usize] = 0;
            }
        }
        Ok(())
    }
    pub fn body(
        &mut self,
        original: u8,
        body: u8,
        kits: &HashMap<u32, Kit>,
        items: &Definitions,
    ) -> Result<()> {
        let a = &mut self.appearance;
        if body == a.gender {
            return Ok(());
        }
        let head = a.equipment[0];
        let hide_beard = if body != original && head >= 2048 {
            let item = items
                .items
                .get(&(head - 2048))
                .ok_or(Error::Invalid("widget appearance head item"))?;
            item.wear_positions[1..].contains(&11)
        } else {
            false
        };
        a.gender = body;
        if body == original {
            for slot in KIT_SLOTS {
                if (1..2048).contains(&a.equipment[slot]) {
                    a.equipment[slot] = a.base_equipment[slot];
                }
            }
        } else {
            if !hide_beard {
                a.equipment[11] = 1;
            }
            let standard = Appearance::standard(kits, body);
            for slot in KIT_SLOTS {
                if (1..2048).contains(&a.equipment[slot]) && standard.equipment[slot] != 0 {
                    a.equipment[slot] = standard.equipment[slot];
                }
            }
        }
        Ok(())
    }
}

/// be.af animates the widget composition using idle definition + current pose frame.
/// This intentionally does not consult the actor's current pose ID or freeze snapshots.
pub fn pose(idle: i32, current: &crate::actor_sequence::State) -> crate::actor_sequence::State {
    crate::actor_sequence::State { id: idle, ..current.clone() }
}
