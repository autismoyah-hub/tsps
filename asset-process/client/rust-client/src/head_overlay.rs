//! Name labels, skull/prayer icons and actor hint arrows (kh.ap).
use crate::{
    disk_cache::{name_hash, DiskCache},
    font::{Font, Style},
    raster::Raster,
    sprite::SpriteSheet,
    sprite_pixels::Pixels,
    Error, Result,
};
use std::collections::HashMap;
pub enum Actor {
    Player {
        name: Option<String>,
        skull: i8,
        prayer: i8,
        hint: bool,
    },
    Npc {
        icons: Vec<(i32, i32)>,
        hint: bool,
    },
}
pub struct Assets {
    pub name_font: Font,
    pub skulls: Vec<Pixels>,
    pub prayers: Vec<Pixels>,
    pub hints: Vec<Pixels>,
    groups: HashMap<i32, Vec<Pixels>>,
}
fn sheet(cache: &mut DiskCache, id: u32) -> Result<Vec<Pixels>> {
    let sheet = SpriteSheet::decode(&cache.file(8, id, 0, [0; 4])?, 16 << 20)?;
    (0..sheet.sprites.len())
        .map(|i| Pixels::from_sheet(&sheet, i))
        .collect()
}
impl Assets {
    pub fn from_assets(
        name_font: Font,
        skulls: Vec<Pixels>,
        prayers: Vec<Pixels>,
        hints: Vec<Pixels>,
        groups: HashMap<i32, Vec<Pixels>>,
    ) -> Self {
        Self {
            name_font,
            skulls,
            prayers,
            hints,
            groups,
        }
    }

    pub fn load(cache: &mut DiskCache) -> Result<Self> {
        let groups = crate::minimap::sprite_groups(&cache.file(17, 3, 0, [0; 4])?)?;
        let font = cache
            .reference(13)?
            .groups
            .iter()
            .find(|g| g.name_hash == Some(name_hash("p12_full")))
            .ok_or(Error::Invalid("player name font"))?
            .id;
        Ok(Self {
            name_font: Font::load(cache, font)?,
            skulls: sheet(cache, groups[3].ok_or(Error::Invalid("skull icons"))?)?,
            prayers: sheet(cache, groups[4].ok_or(Error::Invalid("prayer icons"))?)?,
            hints: sheet(cache, groups[5].ok_or(Error::Invalid("head hint icons"))?)?,
            groups: HashMap::new(),
        })
    }
    pub fn draw(
        &mut self,
        cache: &mut DiskCache,
        raster: &mut Raster,
        actor: &Actor,
        point: Option<[i32; 2]>,
        mut vertical: i32,
        origin_y: i32,
        cycle: i32,
    ) -> Result<()> {
        let Some([x, y]) = point.filter(|p| p[0] > -1) else {
            return Ok(());
        };
        match actor {
            Actor::Player {
                name,
                skull,
                prayer,
                hint,
            } => {
                if let Some(name) = name {
                    vertical += 4;
                    self.name_font.draw(
                        raster,
                        name,
                        x - self.name_font.metrics.width(name, &[]) / 2,
                        y - vertical,
                        Style::default(),
                        &[],
                    )?;
                    vertical += 18;
                }
                for (index, sprites) in [(*skull, &self.skulls), (*prayer, &self.prayers)] {
                    if index >= 0 {
                        let sprite = sprites
                            .get(index as usize)
                            .ok_or(Error::Invalid("player head icon index"))?;
                        vertical += 25;
                        sprite.draw(raster, x - 12, y - vertical, 256)?;
                    }
                }
                if *hint {
                    let sprite = self
                        .hints
                        .get(1)
                        .ok_or(Error::Invalid("player hint icon"))?;
                    vertical += sprite.size[1] as i32;
                    sprite.draw(raster, x - 12, y - vertical, 256)?;
                }
            }
            Actor::Npc { icons, hint } => {
                for (slot, &(group, index)) in icons.iter().enumerate() {
                    if group < 0 || index < 0 {
                        continue;
                    }
                    if !self.groups.contains_key(&group) {
                        match sheet(cache, group as u32) {
                            Ok(sprites) => { self.groups.insert(group, sprites); }
                            Err(Error::PendingAsset { .. }) => continue,
                            Err(error) => return Err(error),
                        }
                    }
                    if let Some(sprite) = self.groups[&group].get(index as usize) {
                        let height = sprite.size[1] as i32;
                        let y = y + (slot as i32) * origin_y
                            - (slot as i32 + 2) * height
                            - 2 * (slot as i32 + 1)
                            - 4;
                        sprite.draw(raster, x - (sprite.size[0] as i32 >> 1), y, 256)?;
                    }
                }
                if *hint && cycle % 20 < 10 {
                    self.hints
                        .first()
                        .ok_or(Error::Invalid("NPC hint icon"))?
                        .draw(raster, x - 12, y - 28, 256)?;
                }
            }
        }
        Ok(())
    }
}
pub fn name_visible(mask: u8, local: bool, friend: bool, friends_chat: bool) -> bool {
    if local {
        mask & 8 != 0
    } else {
        mask & 4 != 0 || (mask & 1 != 0 && friend) || (mask & 2 != 0 && friends_chat)
    }
}
