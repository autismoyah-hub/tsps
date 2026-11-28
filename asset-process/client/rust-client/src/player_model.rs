//! Player equipment/kit assembly and body recoloring.
use crate::{
    disk_cache::DiskCache,
    identity_kit::Kit,
    item::Definitions,
    item_model::{AppearanceOverride, Models},
    model::Model,
    player_colors as colors, Error, Result,
};
use std::collections::HashMap;
pub const KIT_SLOTS: [usize; 7] = [8, 11, 4, 6, 9, 7, 10];
#[derive(Clone, Debug)]
pub struct Appearance {
    pub equipment: [u32; 12],
    pub base_equipment: [u32; 12],
    pub colors: [u8; 5],
    pub gender: u8,
    pub npc: Option<u16>,
    pub overrides: [Option<AppearanceOverride>; 12],
    pub extra: i8,
}
impl Default for Appearance {
    fn default() -> Self {
        Self {
            equipment: [0; 12],
            base_equipment: [0; 12],
            colors: [0; 5],
            gender: 0,
            npc: None,
            overrides: Default::default(),
            extra: -1,
        }
    }
}
impl Appearance {
    pub fn sequence_equipment(&self, sequence: &crate::animation::Sequence) -> Self {
        let mut appearance = self.clone();
        for (slot, id) in [(5, sequence.shield), (3, sequence.weapon)] {
            if id >= 0 {
                appearance.equipment[slot] = (id + 1536) as u32;
            }
        }
        appearance
    }
    /// Native mesh-cache identity includes model overrides in addition to the protocol appearance hash.
    pub fn render_key(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash().hash(&mut hasher);
        self.overrides.hash(&mut hasher);
        self.npc.hash(&mut hasher);
        hasher.finish()
    }
    pub fn hash(&self) -> u64 {
        let mut hash = u64::MAX;
        let mut byte = |b: u8| {
            let mut entry = (hash ^ b as u64) & 255;
            for _ in 0..8 {
                entry = (entry >> 1)
                    ^ if entry & 1 != 0 {
                        (-3932672073523589310i64) as u64
                    } else {
                        0
                    };
            }
            hash = (hash >> 8) ^ entry;
        };
        for id in self.equipment {
            for b in id.to_be_bytes() {
                byte(b);
            }
        }
        for value in self.overrides.iter().flatten() {
            for colors in [&value.recolor, &value.retexture].into_iter().flatten() {
                for c in colors {
                    for b in c.to_be_bytes() {
                        byte(b);
                    }
                }
            }
        }
        for c in self.colors {
            byte(c);
        }
        byte(self.gender);
        hash
    }
    /// Original legacy appearance editor serialization; its 20-byte payload is
    /// intentionally kept separate from the advertised 26-byte V2 packet.
    pub fn design_bytes(&self) -> Vec<u8> {
        let mut out = vec![self.gender];
        for slot in KIT_SLOTS {
            let id = self.equipment[slot];
            out.extend(
                (if id == 0 {
                    65535
                } else {
                    id.wrapping_sub(256) as u16
                })
                .to_be_bytes(),
            );
        }
        out.extend(self.colors);
        out
    }
    pub fn standard(kits: &HashMap<u32, Kit>, gender: u8) -> Self {
        let mut a = Self {
            gender,
            ..Default::default()
        };
        for (part, &slot) in KIT_SLOTS.iter().enumerate() {
            if let Some(id) = kits
                .values()
                .filter(|k| !k.disabled && k.matches(part, gender))
                .map(|k| k.id)
                .min()
            {
                a.equipment[slot] = id + 256;
            }
        }
        a.base_equipment = a.equipment;
        a
    }
    pub fn change_kit(
        &mut self,
        kits: &HashMap<u32, Kit>,
        part: usize,
        forward: bool,
    ) -> Result<()> {
        if part >= 7 {
            return Err(Error::Invalid("appearance body part"));
        }
        let slot = KIT_SLOTS[part];
        if self.equipment[slot] == 0 {
            return Ok(());
        }
        let mut choices = kits
            .values()
            .filter(|k| !k.disabled && k.matches(part, self.gender))
            .map(|k| k.id + 256)
            .collect::<Vec<_>>();
        choices.sort();
        if choices.is_empty() {
            return Ok(());
        }
        let current = self.equipment[slot];
        let next = if forward {
            choices
                .iter()
                .copied()
                .find(|&v| v > current)
                .unwrap_or(choices[0])
        } else {
            choices
                .iter()
                .rev()
                .copied()
                .find(|&v| v < current)
                .unwrap_or(*choices.last().unwrap())
        };
        self.equipment[slot] = next;
        Ok(())
    }
    pub fn change_color(&mut self, part: usize, forward: bool) -> Result<()> {
        let palette = colors::COLORS
            .get(part)
            .ok_or(Error::Invalid("appearance color part"))?;
        let count = if part == 4 {
            palette.len().min(8)
        } else {
            palette.len()
        };
        self.colors[part] = if forward {
            (self.colors[part] as usize + 1) % count
        } else {
            (self.colors[part] as usize + count - 1) % count
        } as u8;
        Ok(())
    }
    /// Original ot.gu edits consume the widget action without sending its generic packet.
    pub fn edit_content(&mut self, kits: &HashMap<u32, Kit>, content: u16) -> Result<bool> {
        match content {
            300..=313 => self.change_kit(kits, ((content - 300) / 2) as usize, content & 1 != 0)?,
            314..=323 => self.change_color(((content - 314) / 2) as usize, content & 1 != 0)?,
            324..=325 => {
                let gender = (content - 324) as u8;
                if gender != self.gender {
                    let standard = Self::standard(kits, gender);
                    self.gender = gender;
                    self.equipment = standard.equipment;
                    self.base_equipment = standard.base_equipment;
                    self.npc = None;
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
    /// Original composition readiness checks every referenced wearable part,
    /// including the third part when the second part is absent.
    pub fn models_ready(
        &self,
        kits: &HashMap<u32, Kit>,
        items: &Definitions,
        mut available: impl FnMut(u32) -> bool,
    ) -> Result<bool> {
        if self.gender > 1 {
            return Err(Error::Invalid("player body type"));
        }
        let mut ready = true;
        for (slot, &id) in self.equipment.iter().enumerate() {
            if (256..2048).contains(&id) {
                if let Some(kit) = kits.get(&(id - 256)) {
                    for &id in &kit.models {
                        ready &= available(id);
                    }
                }
            } else if id >= 2048 {
                let item = items
                    .items
                    .get(&(id - 2048))
                    .ok_or(Error::Invalid("appearance item"))?;
                let mut ids = item.wear_models[self.gender as usize];
                if let Some(id) = self.overrides[slot]
                    .as_ref()
                    .and_then(|a| a.wear_model[self.gender as usize])
                {
                    ids[0] = id as i32;
                }
                if ids[0] != -1 {
                    for id in ids.into_iter().filter(|&id| id != -1) {
                        ready &= available(id as u32);
                    }
                }
            }
        }
        Ok(ready)
    }
    pub fn mesh(
        &self,
        cache: &mut DiskCache,
        models: &mut Models,
        kits: &HashMap<u32, Kit>,
        items: &Definitions,
        head: bool,
    ) -> Result<Model> {
        self.mesh_with_colors(cache, models, kits, items, head, self.colors.map(i32::from))
    }
    /// NPC player-composition overrides carry signed short palette indices.
    pub fn mesh_with_colors(
        &self,
        cache: &mut DiskCache,
        models: &mut Models,
        kits: &HashMap<u32, Kit>,
        items: &Definitions,
        head: bool,
        palette_indices: [i32; 5],
    ) -> Result<Model> {
        if self.npc.is_some() {
            return Err(Error::Invalid(
                "resolve NPC appearance before player assembly",
            ));
        }
        if self.gender > 1 {
            return Err(Error::Invalid("player body type"));
        }
        let mut parts = vec![];
        for (slot, &id) in self.equipment.iter().enumerate() {
            if (256..2048).contains(&id) {
                if let Some(kit) = kits.get(&(id - 256)) {
                    if let Some(model) = kit.mesh(cache, models, head)? {
                        parts.push(model);
                    }
                }
            } else if id >= 2048 {
                let d = items
                    .items
                    .get(&(id - 2048))
                    .ok_or(Error::Invalid("appearance item"))?;
                if let Some(m) = models.wearable(
                    cache,
                    d,
                    self.gender as usize,
                    head,
                    self.overrides[slot].as_ref(),
                )? {
                    parts.push(m);
                }
            }
        }
        let mut model = Model::combine(&parts)?;
        for i in 0..5 {
            let index = palette_indices[i] as usize;
            if let Some(&to) = colors::COLORS[i].get(index) {
                model.recolor(colors::BASE[i], to);
            }
            if let Some(&to) = colors::SECONDARY_COLORS[i].get(index) {
                model.recolor(colors::SECONDARY_BASE[i], to);
            }
        }
        Ok(model)
    }
}
