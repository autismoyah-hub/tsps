//! World-map region selection and screen placement (GameWorldMapRenderer.ae/ap).
use crate::{map_scale, raster::Raster, sprite_pixels::Pixels, Error, Result};
#[derive(Debug)]
pub struct Placement {
    pub region: [u16; 2],
    pub rect: [i32; 4],
}
pub fn placements(
    bounds: [i32; 4],
    local: [i32; 4],
    screen: [i32; 4],
    zoom: f32,
) -> Result<Vec<Placement>> {
    if !zoom.is_finite()
        || !(1.0..=8.0).contains(&zoom)
        || bounds[0] < 0
        || bounds[1] < 0
        || bounds[2] > 65535
        || bounds[3] > 65535
    {
        return Err(Error::Invalid("map view dimensions"));
    }
    let base = [bounds[0].wrapping_mul(64), bounds[1].wrapping_mul(64)];
    let lo = [
        base[0].wrapping_add(local[0]),
        base[1].wrapping_add(local[1]),
    ];
    let hi = [
        base[0].wrapping_add(local[2]),
        base[1].wrapping_add(local[3]),
    ];
    let size = (zoom * 64.0) as i32;
    let start = [(lo[0] / 64).max(bounds[0]), (lo[1] / 64).max(bounds[1])];
    let end = [(hi[0] / 64).min(bounds[2]), (hi[1] / 64).min(bounds[3])];
    let mut result = Vec::new();
    if end[0] as i64 - start[0] as i64 > 4096 || end[1] as i64 - start[1] as i64 > 4096 {
        return Err(Error::Invalid("map view region count"));
    }
    for x in start[0]..=end[0] {
        for y in start[1]..=end[1] {
            let sx = screen[0].wrapping_add((x * 64).wrapping_sub(lo[0]).wrapping_mul(size) / 64);
            let sy = screen[3].wrapping_sub(
                (y * 64)
                    .wrapping_sub(lo[1])
                    .wrapping_add(64)
                    .wrapping_mul(size)
                    / 64,
            );
            result.push(Placement {
                region: [x as u16, y as u16],
                rect: [sx, sy, size, size],
            });
        }
    }
    Ok(result)
}
pub fn draw_region(image: &Pixels, raster: &mut Raster, rect: [i32; 4]) -> Result<()> {
    if rect[2] as usize != image.canvas[0] || rect[3] as usize != image.canvas[1] {
        return map_scale::draw(image, raster, rect);
    }
    // GameSpritePixels.ay is an opaque copy: zero pixels become black here.
    let clip = raster.clip();
    let x = rect[0] as i64 + image.offset[0] as i64;
    let y = rect[1] as i64 + image.offset[1] as i64;
    for py in y.max(clip[1] as i64)..(y + image.size[1] as i64).min(clip[3] as i64) {
        for px in x.max(clip[0] as i64)..(x + image.size[0] as i64).min(clip[2] as i64) {
            let color = image.pixels[(py - y) as usize * image.size[0] + (px - x) as usize];
            raster.fill_rect(px as i32, py as i32, 1, 1, color, 256);
        }
    }
    Ok(())
}

use crate::{
    disk_cache::DiskCache, map_area::Area, map_composite::Composite, map_decoration,
    map_floor::Floors, map_geography::Geography, object, sprite::SpriteSheet,
};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
#[derive(PartialEq, Eq)]
struct FrameKey {
    center: [i32; 2],
    canvas: [i32; 2],
    zoom: u32,
    brightness: u64,
}
fn resolve_icon(
    cache: &mut DiskCache,
    varbits: &mut HashMap<u16, crate::script_runtime::Varbit>,
    variables: &[i32],
    def: &object::Definition,
) -> Result<Option<u32>> {
    let bit = if let Some(id) = def.transform_varbit {
        if !varbits.contains_key(&id) {
            varbits.insert(
                id,
                crate::script_runtime::Varbit::decode(&cache.file(2, 14, id as u32, [0; 4])?)?,
            );
        }
        let bit = varbits[&id];
        bit.get(variables.get(bit.base as usize).copied().unwrap_or(0))
    } else {
        0
    };
    let varp = def
        .transform_varp
        .and_then(|id| variables.get(id as usize))
        .copied()
        .unwrap_or(0);
    Ok(def.transformed_id(bit, varp))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IconRef {
    Fixed(usize),
    Location([u16; 2], [i32; 2]),
}
pub struct Renderer {
    pub generation: u64,
    hits: HashMap<IconRef, crate::map_hit::Symbol>,
    index: crate::map_index::Index<IconRef>,
    index_valid: bool,
    locations: HashMap<[u16; 2], Vec<crate::map_locations::Location>>,
    varbits: HashMap<u16, crate::script_runtime::Varbit>,
    symbols: crate::map_symbols::Symbols,
    overlay: Option<(Arc<Pixels>, crate::map_settings::Settings, Arc<Pixels>)>,
    floors: Floors,
    objects: HashMap<u32, object::Definition>,
    scenes: Vec<Pixels>,
    selected: Option<(i32, bool, Composite)>,
    regions: VecDeque<([u16; 2], u8, Arc<Pixels>)>,
    bytes: usize,
    frame: Option<(FrameKey, Arc<Pixels>)>,
}
impl Renderer {
    pub fn load(cache: &mut DiskCache) -> Result<Self> {
        let ids = crate::minimap::sprite_groups(&cache.file(17, 3, 0, [0; 4])?)?;
        let sheet = SpriteSheet::decode(
            &cache.file(
                8,
                ids[2].ok_or(Error::Invalid("map scenes missing"))?,
                0,
                [0; 4],
            )?,
            16 << 20,
        )?;
        let scenes = (0..sheet.sprites.len())
            .map(|i| Pixels::from_sheet(&sheet, i))
            .collect::<Result<_>>()?;
        Ok(Self {
            generation: 0,
            hits: HashMap::new(),
            index: Default::default(),
            index_valid: false,
            locations: HashMap::new(),
            varbits: HashMap::new(),
            symbols: crate::map_symbols::Symbols::load(cache)?,
            overlay: None,
            floors: Floors::load(cache, 0.725)?,
            objects: object::Definition::load_all(cache)?,
            scenes,
            selected: None,
            regions: VecDeque::new(),
            bytes: 0,
            frame: None,
        })
    }
    pub fn image(
        &mut self,
        cache: &mut DiskCache,
        area: &Area,
        members: bool,
        center: [i32; 2],
        zoom: f32,
        canvas: [i32; 2],
        brightness: f64,
        settings: &crate::map_settings::Settings,
        variables: &[i32],
        refresh: bool,
    ) -> Result<Arc<Pixels>> {
        let terrain = self.terrain(
            cache, area, members, center, zoom, canvas, brightness, variables,
        )?;
        if settings.hidden {
            return Ok(terrain);
        }
        let tiles = canvas.map(|n| (n as f32 / zoom).ceil() as i32);
        let local = [
            center[0].wrapping_sub(tiles[0] / 2),
            center[1].wrapping_sub(tiles[1] / 2),
            center[0].wrapping_add(tiles[0] / 2),
            center[1].wrapping_add(tiles[1] / 2),
        ];
        let regions = placements(area.bounds, local, [0, 0, canvas[0], canvas[1]], zoom)?;
        if refresh {
            for region in &regions {
                if let Some(icons) = self.locations.get_mut(&region.region) {
                    if crate::map_locations::refresh(icons, &self.objects, |def| {
                        resolve_icon(cache, &mut self.varbits, variables, def)
                    })? {
                        self.overlay = None;
                    }
                }
            }
        }
        for region in &regions {
            if let Some(icons) = self.locations.get(&region.region) {
                for icon in icons {
                    let key = IconRef::Location(region.region, icon.tile);
                    if self
                        .hits
                        .get(&key)
                        .is_none_or(|h| h.element != icon.element)
                    {
                        self.hits
                            .insert(key, self.symbols.hit_symbol(cache, icon.element)?);
                    }
                    self.hits.get_mut(&key).unwrap().center = crate::map_icon::position(
                        icon.tile,
                        [region.rect[0], region.rect[1]],
                        region.rect[2],
                        true,
                    );
                }
            }
            for (index, icon) in self.selected.as_ref().unwrap().2.icons.iter().enumerate() {
                if icon.coord[1] >> 6 == region.region[0] as i32
                    && icon.coord[2] >> 6 == region.region[1] as i32
                {
                    if let Some(hit) = self.hits.get_mut(&IconRef::Fixed(index)) {
                        hit.center = crate::map_icon::position(
                            [icon.coord[1], icon.coord[2]],
                            [region.rect[0], region.rect[1]],
                            region.rect[2],
                            false,
                        );
                    }
                }
            }
        }
        if let Some((previous, state, image)) = &self.overlay {
            if Arc::ptr_eq(previous, &terrain) && state == settings {
                return Ok(image.clone());
            }
        }

        let mut raster =
            Raster::from_pixels(terrain.size[0], terrain.size[1], terrain.pixels.clone())?;
        self.symbols.draw(
            cache,
            &mut raster,
            &self.selected.as_ref().unwrap().2.icons,
            &self.locations,
            &regions,
            settings,
        )?;
        let image = Arc::new(Pixels {
            canvas: terrain.canvas,
            offset: [0; 2],
            size: terrain.size,
            pixels: raster.pixels().to_vec(),
        });
        self.overlay = Some((terrain, settings.clone(), image.clone()));
        Ok(image)
    }
    pub fn event(&self, key: IconRef) -> Option<crate::map_event::Event> {
        match key {
            IconRef::Fixed(index) => {
                self.selected
                    .as_ref()?
                    .2
                    .icons
                    .get(index)
                    .map(|i| crate::map_event::Event {
                        element: i.element,
                        source: i.coord,
                        destination: i.coord,
                    })
            }
            IconRef::Location(region, tile) => self
                .locations
                .get(&region)?
                .iter()
                .find(|i| i.tile == tile)
                .map(|i| crate::map_event::Event {
                    element: i.element,
                    source: i.source,
                    destination: i.coord,
                }),
        }
    }
    /// Query rectangles use current scroll; icon centers come from the last visible draw.
    pub fn hits(
        &self,
        area: &Area,
        center: [i32; 2],
        zoom: f32,
        rect: [i32; 4],
        drawn_origin: [i32; 2],
        point: [i32; 2],
    ) -> Result<Vec<IconRef>> {
        if !self.ready(area.id) {
            return Ok(Vec::new());
        }
        let tiles = [
            (rect[2] as f32 / zoom).ceil() as i32,
            (rect[3] as f32 / zoom).ceil() as i32,
        ];
        let local = [
            center[0].wrapping_sub(tiles[0] / 2).wrapping_sub(1),
            center[1].wrapping_sub(tiles[1] / 2).wrapping_sub(1),
            center[0].wrapping_add(tiles[0] / 2).wrapping_add(1),
            center[1].wrapping_add(tiles[1] / 2).wrapping_add(1),
        ];
        let regions = placements(
            area.bounds,
            local,
            [
                rect[0],
                rect[1],
                rect[0].wrapping_add(rect[2]),
                rect[1].wrapping_add(rect[3]),
            ],
            zoom,
        )?;
        let relative = [
            point[0].wrapping_sub(drawn_origin[0]),
            point[1].wrapping_sub(drawn_origin[1]),
        ];
        let mut result = Vec::new();
        for region in regions {
            if !crate::map_hit::contains(region.rect, point) {
                continue;
            }
            let clip = [
                region.rect[0].wrapping_sub(drawn_origin[0]),
                region.rect[1].wrapping_sub(drawn_origin[1]),
                region.rect[2],
                region.rect[3],
            ];
            let mut keys = Vec::new();
            if let Some(icons) = self.locations.get(&region.region) {
                keys.extend(
                    icons
                        .iter()
                        .map(|i| IconRef::Location(region.region, i.tile)),
                );
            }
            keys.extend(
                self.selected
                    .as_ref()
                    .unwrap()
                    .2
                    .icons
                    .iter()
                    .enumerate()
                    .filter(|(_, i)| {
                        i.coord[1] >> 6 == region.region[0] as i32
                            && i.coord[2] >> 6 == region.region[1] as i32
                    })
                    .map(|(index, _)| IconRef::Fixed(index)),
            );
            for key in keys {
                if let Some(hit) = self.hits.get(&key) {
                    if hit.hit(clip, relative)? {
                        result.push(key);
                    }
                }
            }
        }
        Ok(result)
    }
    pub fn menu(
        &self,
        area: &Area,
        center: [i32; 2],
        zoom: f32,
        rect: [i32; 4],
        drawn_origin: [i32; 2],
        point: [i32; 2],
    ) -> Result<Vec<crate::map_menu::Entry>> {
        let hits = self.hits(area, center, zoom, rect, drawn_origin, point)?;
        Ok(crate::map_menu::entries(
            hits.into_iter().filter_map(|key| self.event(key)),
            &self.symbols.elements,
        ))
    }
    pub fn clear_icon_index(&mut self) {
        self.index = Default::default();
        self.index_valid = false;
    }
    pub fn reset_area(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.selected = None;
        self.hits.clear();
        self.locations.clear();
        self.regions.clear();
        self.bytes = 0;
        self.frame = None;
        self.overlay = None;
        self.index = Default::default();
        self.index_valid = false;
    }
    pub fn ready(&self, id: i32) -> bool {
        self.selected.as_ref().is_some_and(|v| v.0 == id)
    }
    pub fn icon(&self, key: IconRef) -> Option<crate::map_composite::Icon> {
        match key {
            IconRef::Fixed(index) => self.selected.as_ref()?.2.icons.get(index).cloned(),
            IconRef::Location(region, tile) => self
                .locations
                .get(&region)?
                .iter()
                .find(|i| i.tile == tile)
                .map(|i| crate::map_composite::Icon {
                    element: i.element,
                    coord: i.coord,
                }),
        }
    }
    fn rebuild_index(&mut self, area: &Area) -> Result<()> {
        let mut groups = std::collections::BTreeMap::<[u16; 2], Vec<(i32, IconRef)>>::new();
        if let Some((_, _, composite)) = &self.selected {
            for (index, icon) in composite.icons.iter().enumerate() {
                let region = [icon.coord[1] >> 6, icon.coord[2] >> 6];
                if region[0] < area.bounds[0]
                    || region[0] > area.bounds[2]
                    || region[1] < area.bounds[1]
                    || region[1] > area.bounds[3]
                    || self
                        .symbols
                        .elements
                        .get(&(icon.element as u32))
                        .is_some_and(|d| !d.world_map)
                {
                    continue;
                }
                groups
                    .entry(region.map(|n| n as u16))
                    .or_default()
                    .push((icon.element, IconRef::Fixed(index)));
            }
        }
        for (region, icons) in &self.locations {
            for icon in icons {
                groups
                    .entry(*region)
                    .or_default()
                    .push((icon.element, IconRef::Location(*region, icon.tile)));
            }
        }
        self.index.rebuild(groups.into_values().flatten())?;
        self.index_valid = true;
        Ok(())
    }
    pub fn icon_start(&mut self, area: &Area) -> Result<std::collections::VecDeque<IconRef>> {
        self.rebuild_index(area)?;
        Ok(self.index.elements().map(|(_, key)| *key).collect())
    }
    pub fn nearest(&mut self, area: &Area, id: i32, point: [i32; 3]) -> Result<Option<[i32; 3]>> {
        if !area.contains_position(point[1], point[2]) {
            return Ok(None);
        }
        self.rebuild_index(area)?;
        Ok(crate::map_index::nearest(
            point,
            self.index
                .get(id)
                .unwrap_or(&[])
                .iter()
                .filter_map(|&key| self.icon(key).map(|i| i.coord)),
        ))
    }
    pub fn cached_bytes(&self) -> usize {
        self.bytes
    }
    fn region_image(
        &mut self,
        cache: &mut DiskCache,
        area: &Area,
        destination: [u16; 2],
        scale: u8,
        variables: &[i32],
    ) -> Result<Option<Arc<Pixels>>> {
        if let Some(index) = self
            .regions
            .iter()
            .position(|(region, size, _)| *region == destination && *size == scale)
        {
            let entry = self.regions.remove(index).unwrap();
            let image = entry.2.clone();
            self.regions.push_back(entry);
            return Ok(Some(image));
        }
        let composite = &self.selected.as_ref().unwrap().2;
        if !composite.regions.contains_key(&destination)
            && !composite
                .chunks
                .iter()
                .any(|c| c.region.destination == destination)
        {
            return Ok(None);
        }
        let group = ((destination[0] as u32) << 8) | destination[1] as u32;
        let bytes = cache.file(18, group, area.id as u32, [0; 4])?;
        let geography = Geography::decode(&bytes, composite, destination)?;
        if !self.locations.contains_key(&destination) {
            let icons = crate::map_locations::collect(
                &bytes,
                composite,
                destination,
                &self.objects,
                &self.symbols.elements,
                |def| resolve_icon(cache, &mut self.varbits, variables, def),
            )?;
            for icon in &icons {
                self.hits.insert(
                    IconRef::Location(destination, icon.tile),
                    self.symbols.hit_symbol(cache, icon.element)?,
                );
            }
            self.locations.insert(destination, icons);
        }
        let present = cache
            .reference(20)?
            .groups
            .iter()
            .find(|g| g.id == group)
            .is_some_and(|g| g.files.iter().any(|f| f.id == area.id as u32));
        let ground = if present {
            crate::image_decode::decode(&cache.file(20, group, area.id as u32, [0; 4])?, 4096)?
                .pixels
        } else {
            vec![0; 4096]
        };
        let mut image =
            self.floors
                .render(&geography, &ground, area.background as u32, scale as usize)?;
        let mut raster = Raster::from_pixels(image.size[0], image.size[1], image.pixels)?;
        map_decoration::region(
            &mut raster,
            &geography,
            scale as i32,
            &self.objects,
            &self.scenes,
        )?;
        image.pixels = raster.pixels().to_vec();
        let bytes = image.pixels.len() * 4;
        let image = Arc::new(image);
        while self.bytes + bytes > 36 << 20 || self.regions.len() >= 256 {
            let Some((_, _, old)) = self.regions.pop_front() else {
                break;
            };
            self.bytes -= old.pixels.len() * 4;
        }
        self.bytes += bytes;
        self.regions.push_back((destination, scale, image.clone()));
        Ok(Some(image))
    }
    pub fn prepare_area(
        &mut self,
        cache: &mut DiskCache,
        area: &Area,
        members: bool,
    ) -> Result<()> {
        if self.selected.as_ref().map(|v| (v.0, v.1)) != Some((area.id, members)) {
            let composite =
                Composite::decode(&cache.file(19, 1, area.id as u32, [0; 4])?, members)?;
            self.generation = self.generation.wrapping_add(1);
            self.hits.clear();
            for (index, icon) in composite.icons.iter().enumerate() {
                if self
                    .symbols
                    .elements
                    .get(&(icon.element as u32))
                    .is_some_and(|d| d.world_map)
                {
                    self.hits.insert(
                        IconRef::Fixed(index),
                        self.symbols.hit_symbol(cache, icon.element)?,
                    );
                }
            }
            self.selected = Some((area.id, members, composite));
            self.regions.clear();
            self.locations.clear();
            self.index = Default::default();
            self.index_valid = false;
            self.overlay = None;
            self.bytes = 0;
            self.frame = None;
        }
        Ok(())
    }
    pub fn overview_markers(
        &mut self,
        area: &Area,
        settings: &crate::map_settings::Settings,
    ) -> Result<crate::map_overview_flash::Markers> {
        let mut points = Vec::new();
        if settings.flash_cycle > 0
            && settings.period > 0
            && settings.flash_cycle % settings.period < settings.period / 2
        {
            if !self.index_valid {
                self.rebuild_index(area)?;
            }
            if let Some(ids) = &settings.flashing {
                for id in ids {
                    if let Some(keys) = self.index.get(*id) {
                        for key in keys {
                            if let Some(icon) = self.icon(*key) {
                                points.push([icon.coord[1], icon.coord[2]]);
                            }
                        }
                    }
                }
            }
        }
        Ok(crate::map_overview_flash::Markers {
            base: [area.bounds[0] * 64, area.bounds[1] * 64],
            extent: [
                (area.bounds[2] - area.bounds[0] + 1) * 64,
                (area.bounds[3] - area.bounds[1] + 1) * 64,
            ],
            points,
            cycle: settings.flash_cycle,
            period: settings.period,
        })
    }
    fn terrain(
        &mut self,
        cache: &mut DiskCache,
        area: &Area,
        members: bool,
        center: [i32; 2],
        zoom: f32,
        canvas: [i32; 2],
        brightness: f64,
        variables: &[i32],
    ) -> Result<Arc<Pixels>> {
        if canvas.iter().any(|&n| n <= 0 || n > 4096)
            || !zoom.is_finite()
            || !(1.0..=8.0).contains(&zoom)
            || !brightness.is_finite()
            || brightness <= 0.0
            || brightness > 10.0
        {
            return Err(Error::Invalid("map frame dimensions"));
        }
        self.prepare_area(cache, area, members)?;
        let key = FrameKey {
            center,
            canvas,
            zoom: zoom.to_bits(),
            brightness: brightness.to_bits(),
        };
        if let Some((previous, image)) = &self.frame {
            if *previous == key {
                return Ok(image.clone());
            }
        }
        let tiles = canvas.map(|n| (n as f32 / zoom).ceil() as i32);
        let local = [
            center[0].wrapping_sub(tiles[0] / 2),
            center[1].wrapping_sub(tiles[1] / 2),
            center[0].wrapping_add(tiles[0] / 2),
            center[1].wrapping_add(tiles[1] / 2),
        ];
        let mut raster = Raster::new(canvas[0] as usize, canvas[1] as usize, 16 << 20)?;
        let channel = |shift: u32| {
            (libm::pow(
                ((area.extra as u32 >> shift) & 255u32) as f64 / 256.0,
                brightness,
            ) * 256.0) as u32
        };
        raster.clear(0xff000000 | (channel(16) << 16) | (channel(8) << 8) | channel(0));
        for placement in placements(area.bounds, local, [0, 0, canvas[0], canvas[1]], zoom)? {
            if let Some(image) =
                self.region_image(cache, area, placement.region, zoom.ceil() as u8, variables)?
            {
                draw_region(&image, &mut raster, placement.rect)?;
            }
        }
        let image = Arc::new(Pixels {
            canvas: canvas.map(|n| n as usize),
            offset: [0; 2],
            size: canvas.map(|n| n as usize),
            pixels: raster.pixels().to_vec(),
        });
        self.frame = Some((key, image.clone()));
        Ok(image)
    }
}
