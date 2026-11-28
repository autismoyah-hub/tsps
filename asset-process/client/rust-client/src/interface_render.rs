//! Cache-backed interface text, sprites, item icons and primitive drawing.
use crate::{
    disk_cache::DiskCache,
    font::{Font, Style},
    interface_state::Interfaces,
    item::Definitions,
    item_icon::{Icons, Options},
    raster::Raster,
    sprite_pixels::Pixels,
    widget_layout::Layout,
    Result,
};
use std::collections::HashMap;
#[derive(Default, Debug)]
pub struct Report {
    pub drawn: usize,
    pub pending: Vec<(u32, &'static str)>,
}
pub struct Renderer {
    pub click_cross_sprites: crate::click_cross::Sprites,
    pub metrics: Option<crate::client_metrics::Display>,
    pub world_drawn: bool,
    pub map_overview_flash: Option<crate::map_overview_flash::Markers>,
    pub map_overview: Option<std::sync::Arc<Pixels>>,
    pub map_detail: HashMap<u32, std::sync::Arc<Pixels>>,
    pub minimap: Option<crate::minimap::View>,
    minimap_masks: HashMap<(i32, bool, bool, u8, u32), crate::minimap::Mask>,
    fonts: HashMap<i32, Font>,
    pub text_icons: std::sync::Arc<Vec<crate::sprite::SpriteSheet>>,
    sprites: HashMap<(i32, bool, bool, u8, u32), Pixels>,
    remote_images: HashMap<String,(crate::remote_request::Request,Option<Pixels>)>,
    icons: Icons,
    models: crate::item_model::Models,
    object_models: crate::object_model::Models,
    objects: Option<HashMap<u32, crate::object::Definition>>,
    animations: Option<crate::animation_runtime::Animations>,
    npcs: Option<HashMap<u32, crate::npc::Definition>>,
    varbits: HashMap<u16, crate::script_runtime::Varbit>,
    kits: Option<HashMap<u32, crate::identity_kit::Kit>>,
    pub player: Option<crate::player_model::Appearance>,
    pub designer: Option<crate::player_model::Appearance>,
    pub preview: Option<crate::player_preview::Snapshot>,
    pub player_pose: Option<crate::actor_sequence::State>,
    pub cycle: i32,
    appearance_preview: crate::appearance_preview::Preview,
    pub active_npcs: HashMap<u16, (u16, Option<crate::actor_update::Customisation>)>,
}
impl Renderer {
    pub fn set_brightness(&mut self,cache:&mut DiskCache,brightness:f64)->Result<bool> {
        self.icons.set_brightness(cache,brightness)
    }
    pub fn menu_font(&mut self, cache: &mut DiskCache) -> Result<&Font> {
        let id = cache
            .reference(13)?
            .groups
            .iter()
            .find(|g| g.name_hash == Some(crate::disk_cache::name_hash("b12_full")))
            .ok_or(crate::Error::Invalid("menu font"))?
            .id;
        if !self.fonts.contains_key(&(id as i32)) {
            self.fonts.insert(id as i32, Font::load(cache, id)?);
        }
        Ok(&self.fonts[&(id as i32)])
    }
    pub fn load(cache: &mut DiskCache, brightness: f64) -> Result<Self> {
        let mut renderer = Self {
            click_cross_sprites: Default::default(),
            metrics: None,
            world_drawn: false,
            map_overview: None,
            map_overview_flash: None,
            map_detail: HashMap::new(),
            minimap: None,
            minimap_masks: HashMap::new(),
            fonts: HashMap::new(),
            text_icons: std::sync::Arc::new(crate::font_icons::load(cache)?),
            sprites: HashMap::new(),
            remote_images: HashMap::new(),
            icons: Icons::load(cache, brightness)?,
            models: Default::default(),
            object_models: Default::default(),
            objects: None,
            animations: None,
            npcs: None,
            varbits: HashMap::new(),
            kits: None,
            player: None,
            designer: None,
            preview: None,
            player_pose: None,
            cycle: 0,
            appearance_preview: Default::default(),
            active_npcs: HashMap::new(),
        };
        renderer.menu_font(cache)?;
        Ok(renderer)
    }
    pub fn draw_status(
        &mut self,
        cache: &mut DiskCache,
        raster: &mut Raster,
        lines: &[&str],
    ) -> Result<()> {
        let id = cache
            .reference(13)?
            .groups
            .iter()
            .find(|g| g.name_hash == Some(crate::disk_cache::name_hash("b12_full")))
            .ok_or(crate::Error::Invalid("status font"))?
            .id;
        if !self.fonts.contains_key(&(id as i32)) {
            self.fonts.insert(id as i32, Font::load(cache, id)?);
        }
        let font = &self.fonts[&(id as i32)];
        let top = raster.height() as i32 / 2 - lines.len() as i32 * 10;
        for (i, line) in lines.iter().enumerate() {
            let x = (raster.width() as i32 - font.metrics.width(line, &self.text_icons)) / 2;
            font.draw(
                raster,
                line,
                x,
                top + i as i32 * 20,
                Style {
                    color: 0xffffff,
                    shadow: Some(0),
                    alpha: 256,
                },
                &self.text_icons,
            )?;
        }
        Ok(())
    }
    pub fn draw(
        &mut self,
        cache: &mut DiskCache,
        defs: &Definitions,
        state: &Interfaces,
        layout: &Layout,
        raster: &mut Raster,
    ) -> Result<Report> {
        self.draw_with_variables(cache, defs, state, layout, raster, None)
    }
    pub fn draw_with_variables(
        &mut self,
        cache: &mut DiskCache,
        defs: &Definitions,
        state: &Interfaces,
        layout: &Layout,
        raster: &mut Raster,
        variables: Option<&crate::script_runtime::Variables>,
    ) -> Result<Report> {
        let saved = raster.clip();
        let result = self.draw_inner(cache, defs, state, layout, raster, variables);
        raster.set_clip(
            saved[0] as i32,
            saved[1] as i32,
            saved[2] as i32,
            saved[3] as i32,
        );
        result
    }
    fn draw_inner(
        &mut self,
        cache: &mut DiskCache,
        defs: &Definitions,
        state: &Interfaces,
        layout: &Layout,
        raster: &mut Raster,
        variables: Option<&crate::script_runtime::Variables>,
    ) -> Result<Report> {
        let mut report = Report::default();
        let canvas_clip = raster.clip().map(|v| v as i32);
        let visual = state
            .drag_visual
            .map(|(id, position, clip)| layout.drag_visual(id, position, clip));
        let layout = visual.as_ref().unwrap_or(layout);
        for &id in &layout.order {
            // A missing asset invalidates this widget, not the rest of the frame.
            let result = (|| -> Result<()> {
            let dragging = state.drag_visual.filter(|v| v.0 == id);
            let p = &layout.widgets[&id];
            if p.hidden {
                return Ok(());
            }
            let original_widget = &state.widgets[&id];
            let mut preview_widget;
            let w = if matches!(original_widget.content, 324 | 325 | 327 | 328) {
                preview_widget = original_widget.clone();
                self.appearance_preview.update(&mut preview_widget, self.designer.as_ref().map_or(0, |a| a.gender), self.cycle);
                &preview_widget
            } else { original_widget };
            if (w.if3 || !matches!(w.kind, 4 | 5 | 8)) && (p.clip[0] >= p.clip[2] || p.clip[1] >= p.clip[3]) { return Ok(()); }
            let runtime = &state.runtime[&id];
            let [x, y] = p.absolute;
            let [width, height] = p.geometry.size;
            raster.set_clip(p.clip[0], p.clip[1], p.clip[2], p.clip[3]);
            let alpha = if dragging.is_some() && !w.scrollbar {
                128
            } else {
                256 - w.transparency as u16
            };
            let index = usize::from(runtime.condition);
            let color = if state.legacy_hovered(id) && w.colors[index + 2] != 0 {
                w.colors[index + 2]
            } else { w.colors[index] };
            if w.content == 1336 {
                if let Some(metrics) = self.metrics {
                    let font_id = cache
                        .reference(13)?
                        .groups
                        .iter()
                        .find(|g| g.name_hash == Some(crate::disk_cache::name_hash("p12_full")))
                        .ok_or(crate::Error::Invalid("metrics font"))?
                        .id;
                    if !self.fonts.contains_key(&(font_id as i32)) {
                        self.fonts
                            .insert(font_id as i32, Font::load(cache, font_id)?);
                    }
                    metrics.draw(&self.fonts[&(font_id as i32)], raster, [x, y], width)?;
                    report.drawn += 1;
                }
                return Ok(());
            }
            if w.content == 1400 {
                if let Some(image) = self.map_detail.get(&id) {
                    crate::map_view::draw_region(image, raster, [x, y, width, height])?;
                    report.drawn += 1;
                } else { report.pending.push((id, "world map")); }
                return Ok(());
            }
            if w.content == 1401 {
                if let Some(image) = &self.map_overview {
                    image.draw_scaled(raster, [x, y, width, height], 256)?;
                    if let Some(markers)=&self.map_overview_flash {markers.draw(raster,[x,y,width,height]);}
                    report.drawn += 1;
                } else { report.pending.push((id, "world map overview")); }
                return Ok(());
            }
            if w.content == 1337 {
                if !self.world_drawn {
                    report.pending.push((id, "world viewport"));
                }
                return Ok(());
            }
            if matches!(w.content, 1338 | 1339) {
                if let Some(view) = &self.minimap {
                    let sprite_id = w.sprites[0];
                    if sprite_id >= 0 {
                        let key = (
                            sprite_id,
                            w.flip_horizontal,
                            w.flip_vertical,
                            w.outline,
                            w.sprite_shadow,
                        );
                        if !self.minimap_masks.contains_key(&key) {
                            let mut sprite = Pixels::load(cache, sprite_id as u32)?;
                            sprite.widget_effects(
                                w.flip_horizontal,
                                w.flip_vertical,
                                w.outline,
                                w.sprite_shadow,
                            );
                            self.minimap_masks
                                .insert(key, crate::minimap::Mask::from_sprite(&sprite));
                        }
                        let mask = &self.minimap_masks[&key];
                        if w.content == 1338 {
                            view.draw(raster, [x, y], mask)?;
                        } else if !view.compass_disabled {
                            crate::minimap::rotated(
                                &view.assets.compass,
                                raster,
                                [x, y, mask.width as i32, mask.starts.len() as i32],
                                [25, 25],
                                view.angle,
                                256,
                                [x, y],
                                mask,
                            )?;
                        } else {
                            for (row, (&start, &width)) in
                                mask.starts.iter().zip(&mask.widths).enumerate()
                            {
                                raster.fill_rect(x + start, y + row as i32, width, 1, 0, 256);
                            }
                        }
                        report.drawn += 1;
                        return Ok(());
                    }
                }
            }
            if matches!(
                w.content,
                1338 | 1339 | 1400 | 1401 | 1402
            ) {
                report.pending.push((id, "special content"));
            }
            match w.kind {
                0 | 11 => {}
                3 => {
                    if w.filled && matches!(w.fill_mode, 1 | 2) {
                        raster.fill_gradient([x, y, width, height], [w.colors[0], w.colors[1]],
                            (w.fill_mode == 2).then_some([255 - w.transparency, 255 - w.transparency_bottom as u8]));
                    } else { raster.widget_rect([x, y, width, height], color, alpha, w.filled); }
                    report.drawn += 1;
                }
                4 => {
                    let clip = dragging.map(|v| v.2).unwrap_or_else(|| p.parent.map_or(canvas_clip, |id| layout.widgets[&id].clip));
                    raster.set_clip(clip[0].max(canvas_clip[0]), clip[1].max(canvas_clip[1]), clip[2].min(canvas_clip[2]), clip[3].min(canvas_clip[3]));
                    if w.font < 0 {
                        return Ok(());
                    }
                    if !self.fonts.contains_key(&w.font) {
                        self.fonts.insert(w.font, Font::load(cache, w.font as u32)?);
                    }
                    let text = if w.if3 && runtime.item != -1 {
                        let d = defs.items.get(&(runtime.item as u32));
                        d.map_or_else(
                            || w.text.clone(),
                            |d| crate::widget_text::item(&d.name, d.stackable, runtime.count),
                        )
                    } else {
                        let text = if runtime.condition && !w.alternate_text.is_empty() {
                            &w.alternate_text
                        } else {
                            &w.text
                        };
                        if w.if3 {
                            text.clone()
                        } else {
                            crate::cs1::substitute(text, &runtime.cs1_values)
                        }
                    };
                    let waiting = state.is_continue_widget(id);
                    let text = if waiting {
                        "Please wait...".into()
                    } else {
                        text
                    };
                    let color = if waiting { w.colors[0] } else { color };
                    self.fonts[&w.font].draw_paragraph(
                        raster,
                        &text,
                        [x, y, width, height],
                        Style {
                            color,
                            shadow: w.text_shadow.then_some(0),
                            alpha: u16::from(255 - w.transparency),
                        },
                        w.text_alignment,
                        w.line_height as i32,
                        &self.text_icons,
                    )?;
                    report.drawn += 1;
                }
                5 => {
                    let sprite = if runtime.item != -1 {
                        self.icons.get(
                            cache,
                            defs,
                            runtime.item as u32,
                            Options {
                                quantity: runtime.count,
                                outline: w.outline,
                                shadow: w.sprite_shadow,
                                quantity_mode: runtime.quantity_mode,
                                ..Options::default()
                            },
                        )?
                    } else if let Some(request)=&runtime.remote_image {
                        if self.remote_images.get(&request.url).is_none_or(|(old,_)|!old.same_request(request)) {
                            let Some(response)=request.result() else {return Ok(())};
                            let pixels=response.and_then(|bytes|crate::image_decode::decode(&bytes,16<<20).ok());
                            if self.remote_images.len()>=512 {self.remote_images.clear();}
                            self.remote_images.insert(request.url.clone(),(request.clone(),pixels));
                        }
                        let Some(mut pixels)=self.remote_images[&request.url].1.clone() else {return Ok(())};
                        if w.flip_horizontal || w.flip_vertical || w.outline!=0 || w.sprite_shadow!=0 {
                            pixels.widget_effects(w.flip_horizontal,w.flip_vertical,w.outline,w.sprite_shadow);
                        }
                        pixels
                    } else {
                        let id = if w.if3 || runtime.condition {
                            w.sprites[0]
                        } else {
                            w.sprites[1]
                        };
                        if id < 0 {
                            return Ok(());
                        }
                        let key = (
                            id,
                            w.flip_horizontal,
                            w.flip_vertical,
                            w.outline,
                            w.sprite_shadow,
                        );
                        if !self.sprites.contains_key(&key) {
                            let mut p = Pixels::load(cache, id as u32)?;
                            p.widget_effects(
                                w.flip_horizontal,
                                w.flip_vertical,
                                w.outline,
                                w.sprite_shadow,
                            );
                            if self.sprites.len() >= 512 {
                                self.sprites.clear();
                            }
                            self.sprites.insert(key, p);
                        }
                        self.sprites[&key].clone()
                    };
                    let clip = dragging.map(|v| v.2).unwrap_or_else(|| p.parent.map_or(canvas_clip, |id| layout.widgets[&id].clip));
                    raster.set_clip(clip[0].max(canvas_clip[0]), clip[1].max(canvas_clip[1]), clip[2].min(canvas_clip[2]), clip[3].min(canvas_clip[3]));
                    sprite.draw_widget(raster, [x, y, width, height], w.sprite_angle, w.sprite_tiling, w.if3, alpha)?;
                    report.drawn += 1;
                }
                6 => {
                    use crate::interface_state::ModelSource;
                    let mut selected = if matches!(w.content, 327 | 328) { ModelSource::Player } else { runtime.model.clone() };
                    let mut npc_override = None;
                    if !w.if3
                        && runtime.condition
                        && matches!(selected, ModelSource::Raw(_) | ModelSource::None)
                    {
                        selected = ModelSource::Raw(w.models[1]);
                    }
                    if let ModelSource::ActiveNpc(index) = selected {
                        let Some((definition, custom)) = self.active_npcs.get(&index) else {
                            return Ok(());
                        };
                        selected = ModelSource::Npc(*definition as i32);
                        if let Some(custom) = custom {
                            if custom.use_player {
                                selected = ModelSource::Player;
                            } else {
                                npc_override = Some(crate::npc_model::Overrides {
                                    models: custom.models.clone(),
                                    recolor: custom.recolor.clone(),
                                    retexture: custom.retexture.clone(),
                                    use_player: false,
                                });
                            }
                        }
                    }
                    let item = if runtime.item >= 0 {
                        Some(runtime.item)
                    } else if let ModelSource::Item(id) = runtime.model {
                        Some(id)
                    } else {
                        None
                    };
                    let design = w.content == 327 && item.is_none();
                    let local_body = w.content == 328 && item.is_none();
                    if local_body { selected = ModelSource::Player; }
                    let preview_appearance = if local_body {
                        let Some(preview) = self.preview.as_ref() else { return Ok(()); };
                        if preview.alpha == 255 { return Ok(()); }
                        if self.animations.is_none() { self.animations = Some(crate::animation_runtime::Animations::load(cache)?); }
                        Some(if let Some(action) = preview.action.as_ref() {
                            preview.appearance.sequence_equipment(self.animations.as_ref().unwrap().sequences.get(&(action.id as u32)).ok_or(crate::Error::Invalid("local preview action sequence"))?)
                        } else { preview.appearance.clone() })
                    } else { None };
                    let widget_body = !design && item.is_none() && matches!(selected, ModelSource::PlayerSelf(_));
                    let npc_body = !design && item.is_none() && matches!(selected, ModelSource::NpcBody(_));
                    let object_body = !design && item.is_none() && matches!(selected, ModelSource::Object(_));
                    let mut npc_scale = None;
                    let (mut model, ambient, contrast) = if design {
                        if self.kits.is_none() { self.kits = Some(crate::identity_kit::Kit::load_all(cache)?); }
                        let kits = self.kits.as_ref().unwrap();
                        let standard;
                        let appearance = if let Some(a) = self.designer.as_ref() { a } else {
                            standard = crate::player_model::Appearance::standard(kits, 0); &standard
                        };
                        (appearance.mesh(cache, &mut self.models, kits, defs, false)?, 64, 850)
                    } else if let Some(item) = item {
                        let d = defs
                            .items
                            .get(&(item as u32))
                            .ok_or(crate::Error::Invalid("widget model item"))?;
                        let d = defs
                            .items
                            .get(&d.count_variant(runtime.count))
                            .ok_or(crate::Error::Invalid("widget model item variant"))?;
                        (
                            self.models.inventory(cache, defs, d.id, 1, None)?,
                            d.ambient + 64,
                            d.contrast + 768,
                        )
                    } else {
                        match selected {
                            ModelSource::Raw(model) if model >= 0 => {
                                let model = if !w.if3 && runtime.condition {
                                    w.models[1]
                                } else {
                                    model
                                };
                                if model < 0 {
                                    return Ok(());
                                }
                                ((*self.models.raw(cache, model as u32)?).clone(), 64, 768)
                            }
                            ModelSource::None | ModelSource::Raw(_) => return Ok(()),
                            ModelSource::Object(id) => {
                                if id < 0 { return Ok(()); }
                                if self.objects.is_none() { self.objects = Some(crate::object::Definition::load_all(cache)?); }
                                let Some(def) = self.objects.as_ref().unwrap().get(&(id as u32)) else { return Ok(()); };
                                // be.aa requests shape10, rotation0 and no height map; no morph or contour.
                                let Some(model) = self.object_models.object(cache, def, 10, 0)? else { return Ok(()); };
                                ((*model).clone(), 64 + def.ambient as i32, 768 + def.contrast)
                            }
                            ModelSource::Player | ModelSource::PlayerSelf(_) => {
                                if widget_body && self.player.is_none() { return Ok(()); }
                                let Some(player) = (if local_body { preview_appearance.as_ref() } else if widget_body {
                                    runtime.player_appearance.as_ref().map(|c| &c.appearance)
                                } else { self.player.as_ref() }) else { return Ok(()); };
                                if let Some(npc) = player.npc {
                                    if self.npcs.is_none() {
                                        self.npcs = Some(crate::npc::Definition::load_all(cache)?);
                                    }
                                    let Some(d) = crate::npc_model::resolve(
                                        self.npcs.as_ref().unwrap(),
                                        npc as u32,
                                        |bit, n| {
                                            if !bit {
                                                return Ok(variables.map_or(0, |v| v.player(n)));
                                            }
                                            if !self.varbits.contains_key(&n) {
                                                self.varbits.insert(
                                                    n,
                                                    crate::script_runtime::Varbit::decode(
                                                        &cache.file(2, 14, n as u32, [0; 4])?,
                                                    )?,
                                                );
                                            }
                                            let b = self.varbits[&n];
                                            Ok(b.get(variables.map_or(0, |v| v.player(b.base))))
                                        },
                                    )?
                                    else {
                                        return Ok(());
                                    };
                                    let Some(model) = crate::npc_model::mesh(
                                        cache,
                                        &mut self.models,
                                        d,
                                        !(widget_body || local_body),
                                        None,
                                    )?
                                    else {
                                        return Ok(());
                                    };
                                    if widget_body || local_body {
                                        npc_scale = Some([d.scale[0] as i32, d.scale[1] as i32, d.scale[0] as i32]);
                                        (model, 64 + d.ambient, 850 + d.contrast)
                                    } else { (model, 64, 768) }
                                } else {
                                    if self.kits.is_none() {
                                        self.kits =
                                            Some(crate::identity_kit::Kit::load_all(cache)?);
                                    }
                                    (
                                        player.mesh_with_colors(
                                            cache,
                                            &mut self.models,
                                            self.kits.as_ref().unwrap(),
                                            defs,
                                            !(widget_body || local_body),
                                            if widget_body { runtime.player_appearance.as_ref().unwrap().colors } else { player.colors.map(i32::from) },
                                        )?,
                                        64,
                                        if widget_body || local_body { 850 } else { 768 },
                                    )
                                }
                            }
                            ModelSource::Npc(npc) | ModelSource::NpcBody(npc) if npc >= 0 => {
                                if self.npcs.is_none() {
                                    self.npcs = Some(crate::npc::Definition::load_all(cache)?);
                                }
                                let npcs = self.npcs.as_ref().unwrap();
                                let Some(d) =
                                    crate::npc_model::resolve(npcs, npc as u32, |bit, n| {
                                        if !bit {
                                            return Ok(variables.map_or(0, |v| v.player(n)));
                                        }
                                        if !self.varbits.contains_key(&n) {
                                            self.varbits.insert(
                                                n,
                                                crate::script_runtime::Varbit::decode(
                                                    &cache.file(2, 14, n as u32, [0; 4])?,
                                                )?,
                                            );
                                        }
                                        let b = self.varbits[&n];
                                        Ok(b.get(variables.map_or(0, |v| v.player(b.base))))
                                    })?
                                else {
                                    return Ok(());
                                };
                                let Some(model) = crate::npc_model::mesh(
                                    cache,
                                    &mut self.models,
                                    d,
                                    !npc_body,
                                    npc_override.as_ref(),
                                )?
                                else {
                                    return Ok(());
                                };
                                if npc_body {
                                    npc_scale = Some([d.scale[0] as i32, d.scale[1] as i32, d.scale[0] as i32]);
                                    (model, 64 + d.ambient, 850 + d.contrast)
                                } else { (model, 64, 768) }
                            }
                            ModelSource::NpcBody(_) => return Ok(()),
                            _ => {
                                report.pending.push((id, "actor interface model"));
                                return Ok(());
                            }
                        }
                    };
                    let mut colors = model.light(ambient, contrast, if design || widget_body || local_body || npc_body { [-30, -50, -30] } else { [-50, -10, -50] })?;
                    let sequence = w.sequences[usize::from(runtime.condition)];
                    if local_body {
                        let preview = self.preview.as_ref().unwrap();
                        self.animations.as_mut().unwrap().apply_actor(cache, &mut model, preview.action.as_ref(), preview.pose.as_ref(), preview.idle)?;
                        self.icons.actor_colors(&model, &mut colors, preview.tint, preview.cycle);
                    }
                    if widget_body {
                        if let Some(pose) = self.player_pose.as_ref().filter(|p| p.id >= 0 && p.frame >= 0) {
                            if self.animations.is_none() { self.animations = Some(crate::animation_runtime::Animations::load(cache)?); }
                            self.animations.as_mut().unwrap().apply(cache, &mut model, pose.id as u32, &pose.clock())?;
                        }
                    }
                    if !design && !widget_body && !local_body && item.is_none() && sequence >= 0 {
                        if self.animations.is_none() {
                            self.animations =
                                Some(crate::animation_runtime::Animations::load(cache)?);
                        }
                        let clock = crate::animation_runtime::Clock { frame: runtime.frame, ..Default::default() };
                        if npc_body || object_body {
                            self.animations.as_mut().unwrap().apply(cache, &mut model, sequence as u32, &clock)?;
                        } else {
                            self.animations.as_mut().unwrap().apply_interface(cache, &mut model, sequence as u32, &clock)?;
                        }
                    }
                    if let Some(scale) = npc_scale { model.scale(scale); }
                    let angles = if w.if3 {
                        w.model_angles.map(i32::from)
                    } else {
                        [w.model_angles[0] as i32, w.model_angles[1] as i32, 0]
                    };
                    let vertices = crate::interface_model::vertices(
                        &model,
                        angles,
                        if w.if3 { w.model_offset } else { [0; 2] },
                        runtime.model_zoom,
                        w.if3 && item.is_some(),
                    )?;
                    let attachment = if local_body { self.preview.as_ref().unwrap().attachment.clone() } else { None };
                    let attachment_vertices = attachment.as_ref().map(|(model, _)| crate::interface_model::vertices(model, angles, if w.if3 { w.model_offset } else { [0; 2] }, runtime.model_zoom, false)).transpose()?;
                    let mut parts = vec![(&model, colors.as_slice(), vertices.as_slice())];
                    if let (Some((model, colors)), Some(vertices)) = (attachment.as_ref(), attachment_vertices.as_ref()) {
                        parts.push((model, colors.as_slice(), vertices.as_slice()));
                    }
                    let actor_alpha = if local_body { self.preview.as_ref().unwrap().alpha } else { 0 };
                    let origin=crate::interface_model::depth_origin(&model,angles[0],if w.if3 {w.model_offset}else{[0;2]},runtime.model_zoom,w.if3 && item.is_some());
                    let sprite = self.icons.draw_widget_models(&parts, [width as usize, height as usize], (w.if3 && w.model_orthographic).then_some(runtime.model_zoom), actor_alpha,origin)?;
                    sprite.draw(raster, x, y, 256)?;
                    report.drawn += 1;
                }
                8 => {
                    if !state.tooltip.visible(id, runtime.instance) { return Ok(()); }
                    let clip = p.parent.map_or(canvas_clip, |id| layout.widgets[&id].clip);
                    raster.set_clip(clip[0].max(canvas_clip[0]), clip[1].max(canvas_clip[1]), clip[2].min(canvas_clip[2]), clip[3].min(canvas_clip[3]));
                    let font_id = cache.reference(13)?.groups.iter()
                        .find(|g| g.name_hash == Some(crate::disk_cache::name_hash("p12_full")))
                        .ok_or(crate::Error::Invalid("tooltip font"))?.id;
                    if !self.fonts.contains_key(&(font_id as i32)) {
                        self.fonts.insert(font_id as i32, Font::load(cache, font_id)?);
                    }
                    let text = crate::cs1::substitute(&w.text, &runtime.cs1_values);
                    crate::widget_tooltip::draw_with_icons(raster, &self.fonts[&(font_id as i32)], &text, [x, y, width, height], clip, &self.text_icons)?;
                    report.drawn += 1;
                }
                9 => {
                    // Endpoint bounds decide visibility; yu.ej draws the stroke against the parent clip.
                    let clip = dragging.map(|v| v.2).unwrap_or_else(|| p.parent.map_or(canvas_clip, |id| layout.widgets[&id].clip));
                    raster.set_clip(clip[0].max(canvas_clip[0]), clip[1].max(canvas_clip[1]), clip[2].min(canvas_clip[2]), clip[3].min(canvas_clip[3]));
                    let (a, b) = if w.line_reversed {
                        ([x, y.wrapping_add(height)], [x.wrapping_add(width), y])
                    } else {
                        ([x, y], [x.wrapping_add(width), y.wrapping_add(height)])
                    };
                    crate::widget_line::draw(raster, a, b, w.line_width, w.colors[0]);
                    report.drawn += 1;
                }
                12 => {
                    let Some(editor)=runtime.editor.as_ref().filter(|e|e.text.ready) else {return Ok(());};
                    if w.font<0 {return Ok(());}
                    if !self.fonts.contains_key(&w.font) {self.fonts.insert(w.font,Font::load(cache,w.font as u32)?);}
                    let placeholder=!editor.text.focused&&editor.text.value.is_empty();
                    let (text,options,color)=if placeholder {(&editor.placeholder,editor.placeholder_options(),editor.colors[2])}else{(&editor.text.value,editor.options,w.colors[0])};
                    crate::editable_text_raster::draw(raster,&self.fonts[&w.font],text,options,crate::editable_text_raster::Paint {
                        rect:[x,y,width,height],scroll:editor.scroll,cursor:editor.text.cursor.max(0) as usize,anchor:editor.text.anchor.max(0) as usize,color,shadow:w.text_shadow.then_some(w.sprite_shadow),selected:editor.colors[0],background:editor.colors[1],caret:editor.text.caret_visible(),show_last:true,
                    })?;
                    report.drawn+=1;
                }
                _ => report.pending.push((id, "widget kind")),
            }
                Ok(())
            })();
            match result {
                Ok(()) => {},
                Err(crate::Error::PendingAsset { .. }) => report.pending.push((id, "asset pending")),
                Err(error) => return Err(error),
            }
        }
        Ok(report)
    }
}
