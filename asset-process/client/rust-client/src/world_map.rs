//! World-map area/script state retained across ordinary login resets.
use crate::{
    disk_cache::DiskCache,
    map_area::{self, Area},
    vm::Stacks,
    Result,
};
pub struct State {
    pub loading: crate::map_loading::Loading,
    pub hover: crate::map_hover::Hover<(u64, crate::map_view::IconRef)>,
    pub event: Option<crate::map_event::Event>,
    pub last_jump_area: Option<i32>,
    pub pointer: crate::map_pointer::Pointer,
    pub display_origin: [i32; 2],
    pub icon_iteration: Option<std::collections::VecDeque<crate::map_view::IconRef>>,
    pub heatmap_overlay: crate::map_heatmap::Overlay,
    pub heatmap_frame: Option<(
        std::sync::Arc<crate::sprite_pixels::Pixels>,
        i32,
        std::sync::Arc<crate::sprite_pixels::Pixels>,
    )>,
    pub render_cycle: i32,
    pub last_icon_refresh: i32,
    pub variables: Vec<i32>,
    pub variable_generation: Option<u64>,
    pub settings: crate::map_settings::Settings,
    pub elements: Option<std::collections::HashMap<u32, crate::map_element::Definition>>,
    pub areas: Option<std::collections::BTreeMap<i32, Area>>,
    pub current: Option<i32>,
    pub center: [i32; 2],
    pub target: [i32; 2],
    pub zoom: f32,
    pub target_zoom: f32,
    pub display_size: [i32; 2],
    pub overview: Option<(i32, std::sync::Arc<crate::sprite_pixels::Pixels>)>,
    pub renderer: Option<Box<crate::map_view::Renderer>>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            loading: Default::default(),
            hover: Default::default(),
            event: None,
            last_jump_area: None,
            pointer: Default::default(),
            display_origin: [-1; 2],
            icon_iteration: None,
            heatmap_overlay: Default::default(),
            heatmap_frame: None,
            render_cycle: 0,
            last_icon_refresh: 0,
            variables: Vec::new(),
            variable_generation: None,
            settings: Default::default(),
            elements: None,
            areas: None,
            current: None,
            center: [0; 2],
            target: [-1; 2],
            zoom: 0.0,
            target_zoom: 0.0,
            display_size: [-1; 2],
            overview: None,
            renderer: None,
        }
    }
}
fn unpack(c: i32) -> [i32; 3] {
    if c == -1 {
        [-1, 0, 0]
    } else {
        [(c >> 28) & 3, (c >> 14) & 16383, c & 16383]
    }
}
fn pack([p, x, y]: [i32; 3]) -> i32 {
    if p < 0 {
        -1
    } else {
        p << 28 | x << 14 | y
    }
}
pub fn zoom(percent: i32) -> f32 {
    match percent {
        25 => 1.0,
        37 => 1.5,
        50 => 2.0,
        75 => 3.0,
        100 => 4.0,
        _ => 8.0,
    }
}
impl State {
    pub fn pointer_input(
        &mut self,
        point: [i32; 2],
        held: bool,
        press: bool,
        blocked: bool,
        admin: bool,
        now: i64,
    ) -> Option<crate::protocol::Packet> {
        let area = self.current.and_then(|id| self.areas.as_ref()?.get(&id));
        let packet = self.pointer.project_click(
            area,
            self.center,
            self.zoom,
            self.display_size,
            self.display_origin,
            point,
            press,
            admin,
            now,
        );
        self.pointer.pan(
            &mut self.center,
            self.target,
            self.target_zoom,
            point,
            held,
            press,
            blocked,
            now,
        );
        packet
    }
    /// GameRenderOverview.au: an explicit open resets scroll, zoom and icon iteration.
    fn jump(&mut self, source: [i32; 3]) {
        let Some(area) = self.area() else {
            return;
        };
        let point = area
            .position(source[0], source[1], source[2])
            .or_else(|| area.position(area.origin[0], area.origin[1], area.origin[2]));
        let Some(point) = point else {
            return;
        };
        let center = [
            point[0].wrapping_sub(area.bounds[0].wrapping_mul(64)),
            point[1].wrapping_sub(area.bounds[1].wrapping_mul(64)),
        ];
        let zoom = zoom(area.zoom as i32);
        self.center = center;
        self.target = [-1; 2];
        self.zoom = zoom;
        self.target_zoom = zoom;
        self.pointer.reset_drag();
        self.icon_iteration = None;
        if let Some(renderer) = &mut self.renderer {
            renderer.clear_icon_index();
        }
    }
    /// Original wf: resolve source area, falling back to the main area when forced.
    pub fn open_at(&mut self, source: [i32; 3], force: bool) {
        let Some(areas) = &self.areas else {
            return;
        };
        let id = crate::map_lookup::area_at(areas, source).or_else(|| {
            force
                .then(|| areas.values().filter(|a| a.main).last().map(|a| a.id))
                .flatten()
        });
        let Some(id) = id else {
            return;
        };
        if self.last_jump_area != Some(id) || force {
            self.last_jump_area = Some(id);
            self.select(id);
            self.jump(source);
        }
    }
    /// Original ad: prefer the local actor when it belongs to this area unless forced.
    pub fn open_target(&mut self, id: i32, local: [i32; 3], target: [i32; 3], force: bool) {
        let Some(area) = self.areas.as_ref().and_then(|a| a.get(&id)) else {
            return;
        };
        let source = if !force && area.contains_coord(local[0], local[1], local[2]) {
            local
        } else {
            target
        };
        self.select(id);
        self.jump(source);
    }
    pub fn menu(&self, rect: [i32; 4], point: [i32; 2]) -> Result<Vec<crate::map_menu::Entry>> {
        match (self.area(), &self.renderer) {
            (Some(area), Some(renderer)) => renderer.menu(
                area,
                self.center,
                self.zoom,
                rect,
                self.display_origin,
                point,
            ),
            _ => Ok(Vec::new()),
        }
    }
    pub fn hover_events(
        &mut self,
        active: bool,
        rect: [i32; 4],
        point: [i32; 2],
    ) -> Result<Vec<(i32, crate::map_event::Event)>> {
        if !active {
            return Ok(Vec::new());
        }
        let mut current = Vec::new();
        if let (Some(area), Some(renderer)) = (self.area(), &self.renderer) {
            for key in renderer.hits(
                area,
                self.center,
                self.zoom,
                rect,
                self.display_origin,
                point,
            )? {
                if let Some(event) = renderer.event(key) {
                    current.push(((renderer.generation, key), event));
                }
            }
        }
        // A transform may change an icon after its last hover. Old-area identities
        // retain their old event; live identities resolve the current element.
        if let Some(renderer) = &self.renderer {
            for ((generation, key), event) in &mut self.hover.previous {
                if *generation == renderer.generation {
                    if let Some(now) = renderer.event(*key) {
                        *event = now;
                    }
                }
            }
        }
        Ok(self.hover.update(true, current))
    }
    pub fn render_context(&mut self, cycle: i32, variables: &crate::script_runtime::Variables) {
        self.render_cycle = cycle;
        if self.variable_generation != Some(variables.generation) {
            self.variables = variables.player_values().to_vec();
            self.variable_generation = Some(variables.generation);
        }
    }

    /// GameRenderOverview.ax/ap, once per visible map's interface cycle.
    pub fn advance(&mut self, heatmap: bool) {
        if heatmap {
            self.zoom = self.target_zoom;
        } else if self.zoom < self.target_zoom {
            self.zoom = (self.zoom + self.zoom / 30.0).min(self.target_zoom);
        } else if self.zoom > self.target_zoom {
            self.zoom = (self.zoom - self.zoom / 30.0).max(self.target_zoom);
        }
        if self.target.iter().all(|&v| v != -1) {
            self.pointer.reset_drag();
            for axis in 0..2 {
                let delta = self.target[axis].wrapping_sub(self.center[axis]);
                if delta != 0 {
                    self.center[axis] =
                        self.center[axis].wrapping_add(delta / delta.wrapping_abs().min(8));
                }
            }
            if self.center == self.target {
                self.target = [-1; 2];
            }
        }
    }
    fn refresh_loading(&mut self, cache: &mut DiskCache) {
        let mut available = |group:u32,id:i32| {
            cache.reference(19).is_ok_and(|index| index.groups.iter().any(|g|g.id==group && g.files.iter().any(|f|f.id==id as u32)))
                && cache.group(19,group,[0;4]).is_ok()
        };
        self.loading.select(self.current, &mut available);
        self.loading.advance(available);
    }
    pub fn load(&mut self, cache: &mut DiskCache) -> Result<()> {
        if self.areas.is_none() {
            self.areas = Some(map_area::load_all(cache)?);
            if let Some(id) = self
                .areas
                .as_ref()
                .unwrap()
                .values()
                .filter(|a| a.main)
                .map(|a| a.id)
                .last()
            {
                self.select(id);
            }
        }
        self.refresh_loading(cache);
        Ok(())
    }
    pub fn area(&self) -> Option<&Area> {
        self.current.and_then(|id| self.areas.as_ref()?.get(&id))
    }
    pub fn display(&self) -> [i32; 2] {
        self.area().map_or([-1; 2], |a| {
            [
                self.center[0].wrapping_add(a.bounds[0].wrapping_mul(64)),
                self.center[1].wrapping_add(a.bounds[1].wrapping_mul(64)),
            ]
        })
    }
    pub fn select(&mut self, id: i32) {
        if self.current == Some(id) {
            return;
        }
        let Some(area) = self.areas.as_ref().and_then(|a| a.get(&id)) else {
            return;
        };
        let [p, x, y] = area.origin;
        let point = area.position(p, x, y).unwrap_or([
            area.bounds[0].wrapping_mul(64),
            area.bounds[1].wrapping_mul(64),
        ]);
        self.center = [
            point[0].wrapping_sub(area.bounds[0].wrapping_mul(64)),
            point[1].wrapping_sub(area.bounds[1].wrapping_mul(64)),
        ];
        self.target = [-1; 2];
        self.zoom = zoom(area.zoom as i32);
        self.target_zoom = self.zoom;
        self.current = Some(id);
        self.pointer.reset_drag();
        self.icon_iteration = None;
        if let Some(renderer) = &mut self.renderer {
            renderer.reset_area();
        }
    }
    fn position(&mut self, point: [i32; 2], instant: bool) {
        let Some(area) = self.area() else {
            return;
        };
        if instant || area.contains_position(point[0], point[1]) {
            let relative = [
                point[0].wrapping_sub(area.bounds[0].wrapping_mul(64)),
                point[1].wrapping_sub(area.bounds[1].wrapping_mul(64)),
            ];
            if instant {
                self.center = relative;
                self.pointer.reset_drag();
                self.target = [-1; 2];
            } else {
                self.target = relative;
            }
        }
    }
    pub fn control(&mut self, op: u16, s: &mut Stacks) -> Result<bool> {
        match op {
            6602 => self.select(s.pop_int()?),
            6603 => {
                let z = self.target_zoom;
                s.push_int(if z == 1.0 {
                    25
                } else if z == 1.5 {
                    37
                } else if z == 2.0 {
                    50
                } else if z == 3.0 {
                    75
                } else if z == 4.0 {
                    100
                } else {
                    200
                })?;
            }
            6604 => self.target_zoom = zoom(s.pop_int()?),
            6606 | 6607 => {
                let [_, x, y] = unpack(s.pop_int()?);
                self.position([x, y], op == 6607);
            }
            6608 | 6609 => {
                let [p, x, y] = unpack(s.pop_int()?);
                if let Some(point) = self.area().and_then(|a| a.position(p, x, y)) {
                    self.position(point, op == 6609);
                }
            }
            6610 => {
                for v in self.display() {
                    s.push_int(v)?;
                }
            }
            6615 => {
                let [x, y] = self.display();
                let point = self.area().and_then(|a| a.coord(x, y));
                for v in point.map_or([-1; 2], |p| [p[1], p[2]]) {
                    s.push_int(v)?;
                }
            }
            6616 => s.push_int(self.current.unwrap_or(-1))?,
            6617 => {
                let [p, x, y] = unpack(s.pop_int()?);
                for v in self
                    .area()
                    .and_then(|a| a.position(p, x, y))
                    .unwrap_or([-1; 2])
                {
                    s.push_int(v)?;
                }
            }
            6618 => {
                let [_, x, y] = unpack(s.pop_int()?);
                if let Some(a) = self.area() {
                    s.push_int(a.coord(x, y).map_or(-1, pack))?;
                } else {
                    s.push_int(-1)?;
                    s.push_int(-1)?;
                }
            }
            6622 => {
                for v in self.display_size {
                    s.push_int(v)?;
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
    pub fn detailed_image(
        &mut self,
        cache: &mut DiskCache,
        members: bool,
        canvas: [i32; 2],
        brightness: f64,
    ) -> Result<Option<std::sync::Arc<crate::sprite_pixels::Pixels>>> {
        self.load(cache)?;
        let Some(area) = self.area().cloned() else {
            return Ok(None);
        };
        if self.renderer.is_none() {
            self.renderer = Some(Box::new(crate::map_view::Renderer::load(cache)?));
        }
        self.settings.frame();
        let refresh =
            !self.settings.hidden && self.render_cycle.wrapping_sub(self.last_icon_refresh) > 100;
        if refresh {
            self.last_icon_refresh = self.render_cycle;
        }

        let image = self.renderer.as_mut().unwrap().image(
            cache,
            &area,
            members,
            self.center,
            self.zoom,
            canvas,
            brightness,
            &self.settings,
            &self.variables,
            refresh,
        )?;
        self.display_size = canvas.map(|n| (n as f32 / self.zoom).ceil() as i32);
        Ok(Some(image))
    }
    pub fn compose_heatmap(
        &mut self,
        image: std::sync::Arc<crate::sprite_pixels::Pixels>,
        heatmap: &mut crate::heatmap::State,
    ) -> Result<std::sync::Arc<crate::sprite_pixels::Pixels>> {
        let Some(heat) = &mut heatmap.map else {
            return Ok(image);
        };
        if let Some((previous, cycle, frame)) = &self.heatmap_frame {
            if std::sync::Arc::ptr_eq(previous, &image) && *cycle == heatmap.cycle {
                return Ok(frame.clone());
            }
        }
        let mut frame = (*image).clone();
        self.heatmap_overlay.paint(
            heat,
            &mut frame.pixels,
            frame.size,
            [0, 0, frame.size[0], frame.size[1]],
            [0, 0, frame.size[0] as i32, frame.size[1] as i32],
            self.display(),
            self.display_size,
            self.zoom.ceil() as i32,
            heatmap.cycle,
        )?;
        let frame = std::sync::Arc::new(frame);
        self.heatmap_frame = Some((image, heatmap.cycle, frame.clone()));
        Ok(frame)
    }
    pub fn overview_markers(
        &mut self,
        cache: &mut DiskCache,
        members: bool,
    ) -> Result<Option<crate::map_overview_flash::Markers>> {
        self.load(cache)?;
        let Some(area) = self.area().cloned() else {
            return Ok(None);
        };
        if self.renderer.is_none() {
            self.renderer = Some(Box::new(crate::map_view::Renderer::load(cache)?));
        }
        let renderer = self.renderer.as_mut().unwrap();
        renderer.prepare_area(cache, &area, members)?;
        Ok(Some(renderer.overview_markers(&area, &self.settings)?))
    }
    pub fn overview_image(
        &mut self,
        cache: &mut DiskCache,
    ) -> Result<Option<std::sync::Arc<crate::sprite_pixels::Pixels>>> {
        self.load(cache)?;
        let Some(id) = self.current else {
            return Ok(None);
        };
        if self.overview.as_ref().map(|v| v.0) != Some(id) {
            let bytes = cache.file(19, 2, id as u32, [0; 4])?;
            let image = crate::image_decode::decode(&bytes, 16 << 20)?;
            self.overview = Some((id, std::sync::Arc::new(image)));
        }
        Ok(self.overview.as_ref().map(|(_, image)| image.clone()))
    }
    pub fn execute(&mut self, op: u16, stack: &mut Stacks, cache: &mut DiskCache) -> Result<bool> {
        if !map_area::is_metadata_query(op)
            && !matches!(op,6602..=6610|6615..=6618|6622..=6640|6693..=6696)
        {
            return Ok(false);
        }
        self.load(cache)?;
        if op==6605 {stack.push_int(self.loading.ready as i32)?;return Ok(true);}
        if map_area::metadata_query(op, stack, self.areas.as_ref().unwrap())? {
            return Ok(true);
        }
        if op == 6623 {
            let point = unpack(stack.pop_int()?);
            stack.push_int(
                crate::map_lookup::area_at(self.areas.as_ref().unwrap(), point).unwrap_or(-1),
            )?;
            return Ok(true);
        }
        if matches!(op,6624..=6637|6693..=6696) {
            if self.elements.is_none() {
                self.elements = Some(crate::map_element::Definition::load_all(cache)?);
            }
            return self
                .settings
                .execute(op, stack, self.elements.as_ref().unwrap());
        }
        if matches!(op, 6638..=6640) {
            if op == 6638 {
                let point = unpack(stack.pop_int()?);
                let id = stack.pop_int()?;
                let area = self.area().cloned();
                let coord = match (area, self.renderer.as_mut()) {
                    (Some(area), Some(renderer)) if renderer.ready(area.id) => {
                        renderer.nearest(&area, id, point)?
                    }
                    _ => None,
                };
                stack.push_int(coord.map_or(-1, pack))?;
                return Ok(true);
            }
            if op == 6639 {
                let area = self.area().cloned();
                match (area, self.renderer.as_mut()) {
                    (Some(area), Some(renderer)) if renderer.ready(area.id) => {
                        self.icon_iteration = Some(renderer.icon_start(&area)?)
                    }
                    _ => {
                        stack.push_int(-1)?;
                        stack.push_int(-1)?;
                        return Ok(true);
                    }
                }
            }
            let mut next = None;
            if let (Some(keys), Some(renderer)) = (&mut self.icon_iteration, &self.renderer) {
                while let Some(key) = keys.pop_front() {
                    if let Some(icon) = renderer.icon(key) {
                        if icon.element != -1 {
                            next = Some(icon);
                            break;
                        }
                    }
                }
            }
            stack.push_int(next.as_ref().map_or(-1, |i| i.element))?;
            stack.push_int(next.map_or(-1, |i| pack(i.coord)))?;
            return Ok(true);
        }
        let handled=self.control(op, stack)?;
        if op==6602 {self.refresh_loading(cache);}
        Ok(handled)
    }
}
