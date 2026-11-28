//! Cache-backed map element sprites and labels, drawn after all region terrain.
use crate::{
    disk_cache::{name_hash, DiskCache},
    font::Font,
    map_composite::Icon,
    map_element::Definition,
    map_label::Label,
    map_settings::Settings,
    map_view::Placement,
    raster::Raster,
    sprite_pixels::Pixels,
    Result,
};
use std::collections::{HashMap, VecDeque};
pub struct Symbols {
    pub elements: HashMap<u32, Definition>,
    fonts: [Option<Font>; 3],
    sprites: VecDeque<(u32, Option<Pixels>)>,
}
impl Symbols {
    pub fn load(cache: &mut DiskCache) -> Result<Self> {
        let mut fonts: [Option<Font>; 3] = [None, None, None];
        for (i, name) in [
            "verdana_11pt_regular",
            "verdana_13pt_regular",
            "verdana_15pt_regular",
        ]
        .into_iter()
        .enumerate()
        {
            let id = cache
                .reference(13)?
                .groups
                .iter()
                .find(|g| g.name_hash == Some(name_hash(name)))
                .map(|g| g.id);
            if let Some(id) = id {
                fonts[i] = Some(Font::load(cache, id)?);
            }
        }
        Ok(Self {
            elements: Definition::load_all(cache)?,
            fonts,
            sprites: VecDeque::new(),
        })
    }
    fn sprite(&mut self, cache: &mut DiskCache, id: u32) -> Result<Option<&Pixels>> {
        if let Some(index) = self.sprites.iter().position(|(key, _)| *key == id) {
            let entry = self.sprites.remove(index).unwrap();
            self.sprites.push_back(entry);
        } else {
            let exists = cache
                .reference(8)?
                .groups
                .iter()
                .any(|g| g.id == id && g.files.iter().any(|f| f.id == 0));
            let image = if exists {
                Some(Pixels::load(cache, id)?)
            } else {
                None
            };
            if self.sprites.len() == 256 {
                self.sprites.pop_front();
            }
            self.sprites.push_back((id, image));
        }
        Ok(self.sprites.back().unwrap().1.as_ref())
    }
    pub fn hit_symbol(
        &mut self,
        cache: &mut DiskCache,
        element: i32,
    ) -> Result<crate::map_hit::Symbol> {
        let def = self.elements.get(&(element as u32)).cloned();
        let sprite_size = if let Some(id) = def.as_ref().and_then(|d| d.sprites[0]) {
            self.sprite(cache, id)?
                .map_or([0; 2], |s| s.size.map(|n| n as i32))
        } else {
            [0; 2]
        };
        let label = if let Some(def) = &def {
            if let (Some(text), Some(font)) = (
                &def.name,
                self.fonts
                    .get(def.text_size as usize)
                    .and_then(|f| f.as_ref()),
            ) {
                Label::new(text, def.text_size, font, self.fonts[0].is_some())?
            } else {
                None
            }
        } else {
            None
        };
        Ok(crate::map_hit::Symbol {
            element,
            center: [0; 2],
            sprite_size,
            alignment: def.map_or([Some(1); 2], |d| d.alignment),
            label,
        })
    }
    fn element(
        &mut self,
        cache: &mut DiskCache,
        raster: &mut Raster,
        id: i32,
        center: [i32; 2],
        zoom: f32,
        settings: &Settings,
        flash: bool,
    ) -> Result<()> {
        if id < 0 {
            return Ok(());
        }
        let Some(def) = self.elements.get(&(id as u32)).cloned() else {
            return Ok(());
        };
        if flash {
            if !settings
                .flashing
                .as_ref()
                .is_some_and(|set| set.contains(&id))
            {
                return Ok(());
            }
            if let Some(id) = def.sprites[0] {
                if let Some(image) = self.sprite(cache, id)? {
                    crate::map_flash::draw(
                        image,
                        raster,
                        center,
                        settings.flash_cycle,
                        settings.period,
                    )?;
                }
            }
        } else {
            if settings.excluded.contains(&id) {
                return Ok(());
            }
            if let Some(id) = def.sprites[0] {
                if let Some(image) = self.sprite(cache, id)? {
                    crate::map_icon::draw(image, raster, center, def.alignment)?;
                }
            }
            if let Some(text) = &def.name {
                if let Some(font) = self
                    .fonts
                    .get(def.text_size as usize)
                    .and_then(|f| f.as_ref())
                {
                    if let Some(label) =
                        Label::new(text, def.text_size, font, self.fonts[0].is_some())?
                    {
                        label.draw(font, raster, center, zoom, def.color)?;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn draw(
        &mut self,
        cache: &mut DiskCache,
        raster: &mut Raster,
        icons: &[Icon],
        locations: &HashMap<[u16; 2], Vec<crate::map_locations::Location>>,
        placements: &[Placement],
        settings: &Settings,
    ) -> Result<()> {
        if settings.hidden {
            return Ok(());
        }
        for flash in [false, true] {
            if flash && settings.flashing.is_none() {
                continue;
            }
            for region in placements {
                if let Some(icons) = locations.get(&region.region) {
                    for icon in icons {
                        let center = crate::map_icon::position(
                            icon.tile,
                            [region.rect[0], region.rect[1]],
                            region.rect[2],
                            true,
                        );
                        self.element(
                            cache,
                            raster,
                            icon.element,
                            center,
                            region.rect[2] as f32 / 64.0,
                            settings,
                            flash,
                        )?;
                    }
                }
                for icon in icons {
                    if icon.element < 0
                        || icon.coord[1] >> 6 != region.region[0] as i32
                        || icon.coord[2] >> 6 != region.region[1] as i32
                    {
                        continue;
                    }
                    if self
                        .elements
                        .get(&(icon.element as u32))
                        .is_some_and(|d| !d.world_map)
                    {
                        continue;
                    }
                    let center = crate::map_icon::position(
                        [icon.coord[1], icon.coord[2]],
                        [region.rect[0], region.rect[1]],
                        region.rect[2],
                        false,
                    );
                    self.element(
                        cache,
                        raster,
                        icon.element,
                        center,
                        region.rect[2] as f32 / 64.0,
                        settings,
                        flash,
                    )?;
                }
            }
        }
        Ok(())
    }
}
