//! Actor health bars and hit-splat composition (kh.ap and sf.al).
use crate::{
    actor_overlay::{BarUpdate, Bars, HealthBar, Hit, Hitsplat},
    disk_cache::{name_hash, DiskCache},
    font::{Font, Style},
    raster::Raster,
    sprite_pixels::Pixels,
    Error, Result,
};
use std::collections::HashMap;
const HIT_OFFSETS: &[i8] = &[
    0, 0, 0, -20, -15, -10, 15, -10, 15, 10, 0, 20, -15, 10, -30, -10, -15, -30, 0, -40, 15, -30,
    30, -10, 30, 10, 15, 30, 0, 40, -15, 30, -30, 10, -45, -10, -30, -30, -15, -50, 0, -60, 15,
    -50, 30, -30, 45, -10, 45, 10, 30, 30, 15, 50, 0, 60, -15, 50, -30, 30, -45, 10, -60, -10, -45,
    -30, -30, -50, -15, -70, 0, -80, 15, -70, 30, -50, 45, -30, 60, -10, 60, 10, 45, 30, 30, 50,
    15, 70, 0, 80, -15, 70, -30, 50, -45, 30, -60, 10, -75, -10, -60, -30, -45, -50, -30, -70, -15,
    -90, 0, -100, 15, -90, 30, -70, 45, -50, 60, -30, 75, -10, 75, 10, 60, 30, 45, 50, 30, 70, 15,
    90, 0, 100, -15, 90, -30, 70, -45, 50, -60, 30, -75, 10, -90, -10, -75, -30, -60, -50, -45,
    -70, -30, -90, -15, -110, 0, 120, 15, -110, 30, -90, 45, -70, 60, -50, 75, -30, 90, -10, 90,
    10, 75, 30, 60, 50, 45, 70, 30, 90, 15, 110, 0, 120, -15, 110, -30, 90, -45, 70, -60, 50, -75,
    30, -90, 10, -105, -10, -90, -30, -75, -50,
];
pub struct ChildView<'a> {
    pub id: u16,
    pub scene: &'a crate::scene::Scene,
    pub transform: &'a crate::world_entity::SceneTransform,
    pub heights: &'a HashMap<u64, i32>,
}
pub struct Assets {
    sprites: HashMap<i32, Pixels>,
    fonts: HashMap<i32, Font>,
    fallback: i32,
    speech_font: i32,
    text_icons: Vec<crate::sprite::SpriteSheet>,
    chat_effect_varp: Option<u16>,
    target_varp: Option<u16>,
    heads: Option<crate::head_overlay::Assets>,
}
fn alpha(value: i32) -> u16 {
    if (0..255).contains(&value) {
        value as u16
    } else {
        256
    }
}
impl Assets {
    pub fn from_assets(
        fallback: i32,
        fonts: HashMap<i32, Font>,
        sprites: HashMap<i32, Pixels>,
    ) -> Result<Self> {
        if !fonts.contains_key(&fallback) {
            return Err(Error::Invalid("overlay fallback font missing"));
        }
        Ok(Self {
            fallback,
            speech_font: fallback,
            text_icons: Vec::new(),
            chat_effect_varp: None,
            target_varp: None,
            heads: None,
            fonts,
            sprites,
        })
    }

    pub fn load(cache: &mut DiskCache) -> Result<Self> {
        let id = cache
            .reference(13)?
            .groups
            .iter()
            .find(|g| g.name_hash == Some(name_hash("p11_full")))
            .ok_or(Error::Invalid("overlay fallback font"))?
            .id as i32;
        let mut fonts = HashMap::new();
        fonts.insert(id, Font::load(cache, id as u32)?);
        let speech_font = cache
            .reference(13)?
            .groups
            .iter()
            .find(|g| g.name_hash == Some(name_hash("b12_full")))
            .ok_or(Error::Invalid("overhead font missing"))?
            .id as i32;
        fonts.insert(speech_font, Font::load(cache, speech_font as u32)?);
        let [chat_effect_varp, target_varp] = crate::overhead::overlay_varps(cache)?;
        let heads = Some(crate::head_overlay::Assets::load(cache)?);
        Ok(Self {
            speech_font,
            text_icons: crate::font_icons::load(cache)?,
            chat_effect_varp,
            target_varp,
            heads,
            sprites: HashMap::new(),
            fonts,
            fallback: id,
        })
    }
    /// Tile/entity scene arrows share the NPC hint sprite and screen anchor.
    pub fn draw_scene_hint(&self, raster:&mut Raster, point:[i32;2])->Result<()> {
        if point[0]<0 {return Ok(());}
        if let Some(heads)=&self.heads {
            heads.hints.get(0).ok_or(Error::Invalid("scene hint icon"))?.draw(raster,point[0]-12,point[1]-28,256)?;
        }
        Ok(())
    }
    pub fn draw(
        &mut self,
        client: &mut crate::client_state::ClientState,
        scene: &crate::scene::Scene,
        camera: &crate::render3d::Camera,
        heights: &HashMap<u64, i32>,
        raster: &mut Raster,
    ) -> Result<()> {
        self.draw_at(client, scene, camera, heights, raster, 0)
    }
    pub fn draw_at(
        &mut self,
        client: &mut crate::client_state::ClientState,
        scene: &crate::scene::Scene,
        camera: &crate::render3d::Camera,
        heights: &HashMap<u64, i32>,
        raster: &mut Raster,
        origin_y: i32,
    ) -> Result<()> {
        self.draw_at_views(client, scene, camera, heights, raster, origin_y, &[])
    }
    pub fn draw_at_views(
        &mut self,
        client: &mut crate::client_state::ClientState,
        scene: &crate::scene::Scene,
        camera: &crate::render3d::Camera,
        heights: &HashMap<u64, i32>,
        raster: &mut Raster,
        origin_y: i32,
        children: &[ChildView<'_>],
    ) -> Result<()> {
        let mut speech = Vec::new();
        let local = client.player_state.as_ref().map(|s| s.local);
        let target = self
            .target_varp
            .map(|id| client.scripts.variables.player(id))
            .filter(|&v| v != -1)
            .map(|v| (v & 2047) as u16);
        let mut order = Vec::new();
        if let Some(players) = &client.player_state {
            for &index in &players.high {
                if Some(index) != local && Some(index) != target {
                    order.push((true, index));
                }
            }
        }
        order.extend(client.npc_state.indices.iter().map(|&index| (false, index)));
        if client.preferences.render_self {
            if let Some(local) = local {
                order.push((true, local));
            }
        }
        if let Some(target) = target {
            order.push((true, target));
        }
        for (is_player, index) in order {
            if is_player {
                let Some(player) = client
                    .player_state
                    .as_mut()
                    .and_then(|s| s.root_actor_mut(index))
                else {
                    continue;
                };
                let Some(&height) = heights.get(&(0x2000000000000000 | index as u64)) else {
                    continue;
                };
                let Some(appearance) = &player.appearance else {
                    continue;
                };
                let friend = client.social.is_friend_online(&appearance.name);
                let speech_visible = player
                    .overlays
                    .speech
                    .visible(client.chat.public_filter, friend);
                let name = crate::head_overlay::name_visible(
                    client.preferences.player_names,
                    Some(index) == local,
                    friend,
                    client
                        .friends_chat
                        .as_ref()
                        .is_some_and(|c| c.contains(&appearance.name)),
                )
                .then(|| appearance.name.clone());
                let head = crate::head_overlay::Actor::Player {
                    name,
                    skull: appearance.skull,
                    prayer: appearance.overhead,
                    hint: client.hint.kind == 3 && client.hint.index == index,
                };
                self.actor(
                    &mut speech,
                    speech_visible,
                    &head,
                    origin_y,
                    &mut client.scripts,
                    &client.overlay_definitions,
                    &mut player.overlays,
                    player.path.position,
                    player.position[0],
                    height,
                    client.actor_cycle,
                    scene,
                    camera,
                    None,
                    raster,
                )?;
            } else {
                let Some(npc) = client.npc_state.npcs.get_mut(&index) else {
                    continue;
                };
                let Some(&height) = heights.get(&(0x4000000000000000 | index as u64)) else {
                    continue;
                };
                let icons = npc
                    .appearance
                    .head_icons
                    .as_ref()
                    .map(|v| v.to_vec())
                    .unwrap_or_else(|| {
                        client.npcs[&(npc.id as u32)]
                            .head_icons
                            .iter()
                            .map(|&(g, i)| (g, i as i32))
                            .collect()
                    });
                let head = crate::head_overlay::Actor::Npc {
                    icons,
                    hint: client.hint.kind == 1 && client.hint.index == index,
                };
                self.actor(
                    &mut speech,
                    true,
                    &head,
                    origin_y,
                    &mut client.scripts,
                    &client.overlay_definitions,
                    &mut npc.overlays,
                    npc.path.position,
                    client.data.position[0],
                    height,
                    client.actor_cycle,
                    scene,
                    camera,
                    None,
                    raster,
                )?;
            }
        }
        for view in children {
            let generation=client.world_entities.get(&0).and_then(|w|w.entities.get(&view.id)).map(|e|e.spawn_generation);
            let indices = client.player_state.as_ref().and_then(|s| s.referenced_view(view.id,generation?))
                .map(|v| v.players.keys().copied().collect::<Vec<_>>()).unwrap_or_default();
            for index in indices {
                if Some(index) == local && !client.preferences.render_self && Some(index) != target { continue; }
                let Some(player) = client
                    .player_state
                    .as_mut()
                    .and_then(|s| s.referenced_view_mut(view.id,generation?)).and_then(|v| v.players.get_mut(&index))
                else {
                    continue;
                };
                let Some(&height) = view.heights.get(&(0x2000000000000000 | index as u64)) else {
                    continue;
                };
                let Some(appearance) = &player.appearance else {
                    continue;
                };
                let friend = client.social.is_friend_online(&appearance.name);
                let speech_visible = player
                    .overlays
                    .speech
                    .visible(client.chat.public_filter, friend);
                let name = crate::head_overlay::name_visible(
                    client.preferences.player_names,
                    Some(index) == local,
                    friend,
                    client
                        .friends_chat
                        .as_ref()
                        .is_some_and(|c| c.contains(&appearance.name)),
                )
                .then(|| appearance.name.clone());
                let head = crate::head_overlay::Actor::Player {
                    name,
                    skull: appearance.skull,
                    prayer: appearance.overhead,
                    hint: client.hint.kind == 3 && client.hint.index == index,
                };
                self.actor(
                    &mut speech,
                    speech_visible,
                    &head,
                    origin_y,
                    &mut client.scripts,
                    &client.overlay_definitions,
                    &mut player.overlays,
                    player.path.position,
                    player.position[0],
                    height,
                    client.actor_cycle,
                    view.scene,
                    camera,
                    Some(view),
                    raster,
                )?;
            }
            let entity = client
                .world_entities
                .get_mut(&0)
                .and_then(|world| world.entities.get_mut(&view.id))
                .ok_or(Error::Invalid("overlay child view missing"))?;
            for &index in &entity.npcs.indices {
                let Some(npc) = entity.npcs.npcs.get_mut(&index) else {
                    continue;
                };
                let Some(&height) = view.heights.get(&(0x4000000000000000 | index as u64)) else {
                    continue;
                };
                let icons = npc
                    .appearance
                    .head_icons
                    .as_ref()
                    .map(|v| v.to_vec())
                    .unwrap_or_else(|| {
                        client.npcs[&(npc.id as u32)]
                            .head_icons
                            .iter()
                            .map(|&(g, i)| (g, i as i32))
                            .collect()
                    });
                let head = crate::head_overlay::Actor::Npc { icons, hint: client.hint.kind == 1 && client.hint.index == index };
                self.actor(
                    &mut speech,
                    true,
                    &head,
                    origin_y,
                    &mut client.scripts,
                    &client.overlay_definitions,
                    &mut npc.overlays,
                    npc.path.position,
                    entity.plane as i32,
                    height,
                    client.actor_cycle,
                    view.scene,
                    camera,
                    Some(view),
                    raster,
                )?;
            }
        }
        let disabled = self
            .chat_effect_varp
            .is_some_and(|id| client.scripts.variables.player(id) != 0);
        crate::overhead::draw_with_icons(
            &self.fonts[&self.speech_font],
            raster,
            &mut speech,
            client.actor_cycle,
            disabled,
            &self.text_icons,
        )?;
        Ok(())
    }
    fn actor(
        &mut self,
        speech: &mut Vec<crate::overhead::Entry>,
        speech_visible: bool,
        head: &crate::head_overlay::Actor,
        origin_y: i32,
        scripts: &mut crate::script_runtime::Runtime<crate::script_runtime::CacheSource>,
        definitions: &crate::actor_overlay::Definitions,
        state: &mut crate::actor_overlay::State,
        position: [i32; 2],
        plane: i32,
        height: i32,
        cycle: i32,
        scene: &crate::scene::Scene,
        camera: &crate::render3d::Camera,
        view: Option<&ChildView<'_>>,
        raster: &mut Raster,
    ) -> Result<()> {
        use crate::script_runtime::Source;
        let x = position[0] - scene.base_x * 128;
        let z = position[1] - scene.base_y * 128;
        let Some(ground) = scene.elevation(plane as usize, x, z) else {
            return Ok(());
        };
        let size = [raster.width(), raster.height()];
        let project =
            |point| camera.project(view.map_or(point, |view| view.transform.point(point)), size);

        let mut vertical = -2;
        if speech_visible && speech.len() < 50 {
            if let (Some(text), Some(point)) = (
                &state.speech.text,
                project([x as f32, (ground - height) as f32, z as f32]),
            ) {
                if point[0] > -1 {
                    speech.push(crate::overhead::Entry {
                        position: [point[0], point[1] - vertical],
                        text: text.clone(),
                        color: state.speech.color,
                        effect: state.speech.effect,
                        remaining: state.speech.remaining,
                        pattern: state.speech.pattern.clone(),
                    });
                    vertical += 12;
                }
            }
        }
        let bar_point = project([x as f32, (ground - height - 15) as f32, z as f32]);
        let vertical = self.bars(
            &mut scripts.source.cache,
            raster,
            &mut state.bars,
            bar_point,
            cycle,
            vertical,
        )?;
        if let Some(heads) = &mut self.heads {
            heads.draw(
                &mut scripts.source.cache,
                raster,
                head,
                bar_point,
                vertical,
                origin_y,
                cycle,
            )?;
        }
        let Some(hit_point) = project([x as f32, (ground - height / 2) as f32, z as f32]) else {
            return Ok(());
        };
        for (slot, hit) in state.hits.slots.iter().enumerate() {
            if hit.end <= cycle {
                continue;
            }
            let Some(def) = definitions.hit(hit.kind, |bit, id| {
                if bit {
                    let b = scripts.source.varbit(id)?;
                    Ok(b.get(scripts.variables.player(b.base)))
                } else {
                    Ok(scripts.variables.player(id as u16))
                }
            })?
            else {
                continue;
            };
            self.hit(
                &mut scripts.source.cache,
                raster,
                hit,
                def,
                slot,
                hit_point,
                cycle,
            )?;
        }
        Ok(())
    }
    fn sprite(&mut self, cache: &mut DiskCache, id: i32) -> Result<()> {
        if id >= 0 && !self.sprites.contains_key(&id) {
            match Pixels::load(cache, id as u32) {
                Ok(sprite) => { self.sprites.insert(id, sprite); }
                // Java returns null until the group arrives; cache only success.
                Err(Error::PendingAsset { .. }) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
    pub fn bars(
        &mut self,
        cache: &mut DiskCache,
        raster: &mut Raster,
        bars: &mut Bars,
        point: Option<[i32; 2]>,
        cycle: i32,
        mut vertical: i32,
    ) -> Result<i32> {
        let clip = raster.clip();
        for bar in &mut bars.entries {
            let Some(update) = bar.current(cycle).cloned() else {
                continue;
            };
            let def = &bar.definition;
            for id in def.sprites {
                self.sprite(cache, id)?;
            }
            let front = self.sprites.get(&def.sprites[0]);
            let back = self.sprites.get(&def.sprites[1]);
            let sprites = front.zip(back);
            let (width, padding) = if let Some((front, _)) = sprites {
                let padding = if (def.padding as usize) * 2 < front.size[0] {
                    def.padding as i32
                } else {
                    0
                };
                (front.size[0] as i32 - 2 * padding, padding)
            } else {
                (def.width as i32, 0)
            };
            let (mut fill, opacity) = bar_fill(def, &update, cycle, width)?;
            if let Some((front, back)) = sprites {
                fill += if fill == width { 2 * padding } else { padding };
                vertical += back.size[1] as i32;
                if let Some([x, y]) = point {
                    let x = x - (width >> 1) - padding;
                    let y = y - vertical;
                    back.draw(raster, x, y, alpha(opacity))?;
                    raster.set_clip(
                        x.max(clip[0] as i32),
                        y.max(clip[1] as i32),
                        (x + fill).min(clip[2] as i32),
                        (y + back.size[1] as i32).min(clip[3] as i32),
                    );
                    front.draw(raster, x, y, alpha(opacity))?;
                    raster.set_clip(
                        clip[0] as i32,
                        clip[1] as i32,
                        clip[2] as i32,
                        clip[3] as i32,
                    );
                }
            } else {
                vertical += 5;
                if let Some([x, y]) = point {
                    let x = x - (width >> 1);
                    let y = y - vertical;
                    raster.fill_rect(x, y, fill, 5, 0x00ff00, 256);
                    raster.fill_rect(x + fill, y, width - fill, 5, 0xff0000, 256);
                }
            }
            vertical += 2;
        }
        bars.entries.retain(|bar| !bar.updates.is_empty());
        Ok(if vertical == -2 { 5 } else { vertical })
    }
    pub fn hit(
        &mut self,
        cache: &mut DiskCache,
        raster: &mut Raster,
        hit: &Hit,
        def: &Hitsplat,
        slot: usize,
        point: [i32; 2],
        cycle: i32,
    ) -> Result<()> {
        let remaining = hit.end.wrapping_sub(cycle);
        let duration = def.duration as i32;
        if remaining <= 0 || remaining > duration {
            return Ok(());
        }
        let offsets = HIT_OFFSETS
            .get(slot * 2..slot * 2 + 2)
            .ok_or(Error::Invalid("hit-splat slot offset"))?;
        for id in def.sprites {
            self.sprite(cache, id)?;
        }
        let mut font_id = if def.font < 0 {
            self.fallback
        } else {
            def.font
        };
        if !self.fonts.contains_key(&font_id) {
            match Font::load(cache, font_id as u32) {
                Ok(font) => { self.fonts.insert(font_id, font); }
                // sf.al uses the default font when px.ao has no font yet.
                Err(Error::PendingAsset { .. }) => font_id = self.fallback,
                Err(error) => return Err(error),
            }
        }
        let font = &self.fonts[&font_id];
        let text = def.text.replace("%1", &hit.value.to_string());
        let text_width = font.metrics.width(&text, &[]);
        let sprites = [0, 2, 1, 3].map(|i| self.sprites.get(&def.sprites[i]));
        let repeats = if let Some(middle) = sprites[2] {
            if sprites[1].is_none() && sprites[3].is_none() {
                1
            } else {
                if middle.size[0] == 0 {
                    return Err(Error::Invalid("empty hit-splat middle sprite"));
                }
                1 + text_width / middle.size[0] as i32
            }
        } else {
            0
        };
        let mut width = sprites[0].map_or(0, |s| s.size[0] as i32) + 2;
        let left = width;
        width += sprites[1].map_or(0, |s| s.size[0] as i32);
        let middle = width;
        let mut text_x = width;
        if let Some(sprite) = sprites[2] {
            let span = sprite.size[0] as i32 * repeats;
            width += span;
            text_x += (span - text_width) / 2;
        } else {
            width += text_width;
        }
        let right = width;
        width += sprites[3].map_or(0, |s| s.size[0] as i32);
        let x = point[0] + offsets[0] as i32 - (width >> 1) + def.offset[0] as i32
            - def.offset[0] as i32 * remaining / duration;
        let y = point[1] + offsets[1] as i32 - 12 + remaining * def.offset[1] as i32 / duration
            - def.offset[1] as i32;
        let opacity = if def.fade >= 0 {
            let denominator = duration - def.fade;
            if denominator == 0 {
                return Err(Error::Invalid("hit-splat fade duration"));
            }
            alpha((remaining << 8) / denominator)
        } else {
            256
        };
        let mut draw = |sprite: Option<&Pixels>, offset: i32| -> Result<()> {
            if let Some(s) = sprite {
                s.draw(raster, x + offset - s.offset[0] as i32, y, opacity)?;
            }
            Ok(())
        };
        draw(sprites[0], 0)?;
        draw(sprites[1], left)?;
        if let Some(s) = sprites[2] {
            for i in 0..repeats {
                draw(Some(s), middle + i * s.size[0] as i32)?;
            }
        }
        draw(sprites[3], right)?;
        font.draw(
            raster,
            &text,
            x + text_x,
            y + 15 + def.baseline as i32,
            Style {
                color: def.color,
                shadow: Some(0),
                alpha: opacity,
            },
            &[],
        )?;
        Ok(())
    }
}
pub fn bar_fill(def: &HealthBar, update: &BarUpdate, cycle: i32, width: i32) -> Result<(i32, i32)> {
    if def.width == 0 {
        return Err(Error::Invalid("zero health-bar width"));
    }
    let elapsed = cycle.wrapping_sub(update.start);
    let target = width.wrapping_mul(update.fill[1]) / (def.width as i32);
    let mut opacity = 255;
    let mut fill = if update.interpolation > elapsed {
        let stepped = if def.step == 0 {
            0
        } else {
            elapsed / def.step * def.step
        };
        let from = width.wrapping_mul(update.fill[0]) / (def.width as i32);
        stepped.wrapping_mul(target - from) / update.interpolation + from
    } else {
        if def.fade >= 0 {
            let denominator = def.duration as i32 - def.fade;
            if denominator == 0 {
                return Err(Error::Invalid("health-bar fade duration"));
            }
            opacity = ((def.duration as i32 + update.interpolation - elapsed) << 8) / denominator;
        }
        target
    };
    if update.fill[1] > 0 && fill < 1 {
        fill = 1;
    }
    Ok((fill, opacity))
}
