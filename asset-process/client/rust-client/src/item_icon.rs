//! Inventory icon composition using the native software model renderer.
use crate::{
    disk_cache::{name_hash, DiskCache},
    floor,
    font::{Font, Style},
    item::Definitions,
    item_model::{icon_vertices, Models},
    raster::Raster,
    render3d::{texture_uv, Camera, Frame, Triangle},
    sprite_pixels::Pixels,
    texture::{self, Texture},
    Error, Result,
};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Options {
    pub quantity: i32,
    pub outline: u8,
    pub shadow: u32,
    pub quantity_mode: u8,
    pub enlarged: bool,
    pub size: [usize; 2],
}
impl Default for Options {
    fn default() -> Self {
        Self {
            quantity: 1,
            outline: 1,
            shadow: 0,
            quantity_mode: 2,
            enlarged: false,
            size: [36, 32],
        }
    }
}
pub struct Icons {
    brightness:f64,
    models: Models,
    palette: Vec<u32>,
    textures: HashMap<u16, Texture>,
    font: Font,
    cached: HashMap<(u32, Options), Pixels>,
}
impl Icons {
    pub fn draw_model(
        &self,
        model: &crate::model::Model,
        colors: &[[i32; 3]],
        vertices: &[[f32; 3]],
        size: [usize; 2],
        orthographic: Option<i32>,
    ) -> Result<Pixels> {
        self.draw_models(&[(model, colors, vertices)], size, orthographic)
    }
    pub fn actor_colors(&self, model: &crate::model::Model, colors: &mut [[i32; 3]], tint: crate::actor_visual::Tint, cycle: i32) {
        if !tint.active(cycle) { return; }
        for (i, color) in colors.iter_mut().enumerate() {
            if i as i32 >= model.faces.len() as i16 as i32 || color[2] == -2
                || model.face_textures.as_ref().is_some_and(|t| t[i] != -1) { continue; }
            if color[2] == -1 { color[0] = tint.color(self.palette[color[0] as usize] as i32); }
            else { *color = color.map(|value| tint.color(value)); }
        }
    }
    /// Submit body and already-lit attachments together without quantizing animated vertices.
    pub fn draw_models(&self, parts: &[(&crate::model::Model, &[[i32; 3]], &[[f32; 3]])], size: [usize; 2], orthographic: Option<i32>) -> Result<Pixels> {
        self.draw_models_alpha(parts, size, orthographic, 0)
    }
    /// Render with an actor alpha override while preserving encoded alpha for near clipping.
    pub fn draw_models_alpha(&self, parts: &[(&crate::model::Model, &[[i32; 3]], &[[f32; 3]])], size: [usize; 2], orthographic: Option<i32>, actor_alpha: u8) -> Result<Pixels> {
        self.draw_models_mode(parts,size,orthographic,actor_alpha,None)
    }
    /// ga's type6 widgets use GameModel.bj/bt and the non-depth rasterizer.
    pub fn draw_widget_models(&self,parts:&[(&crate::model::Model,&[[i32;3]],&[[f32;3]])],size:[usize;2],orthographic:Option<i32>,actor_alpha:u8,origin:i32)->Result<Pixels> {
        self.draw_models_mode(parts,size,orthographic,actor_alpha,Some(origin))
    }
    fn draw_models_mode(&self,parts:&[(&crate::model::Model,&[[i32;3]],&[[f32;3]])],size:[usize;2],orthographic:Option<i32>,actor_alpha:u8,origin:Option<i32>)->Result<Pixels> {
        let mut frame = Frame::new(size[0], size[1])?;
        frame.clear(0);
        let mut triangles = Vec::new();
        let mut priorities=Vec::new();
        let default_priority=parts.first().map_or(0,|p|p.0.priority);
        let has_priorities=parts.iter().any(|p|p.0.priorities.is_some() || p.0.priority!=default_priority);
        for &(model, colors, vertices) in parts {
        for (i, face) in model.faces.iter().enumerate() {
            let mut color = colors[i];
            if color[2] == -2 {
                continue;
            }
            if color[2] == -1 {
                color = [color[0]; 3];
            }
            let positions = face.map(|v| vertices[v as usize]);
            let mapping = model
                .texture_coords
                .as_ref()
                .and_then(|c| {
                    if c[i] == -1 {
                        None
                    } else {
                        model.texture_triangles.get(c[i] as u8 as usize)
                    }
                })
                .unwrap_or(face);
            priorities.push(model.priorities.as_ref().map_or(model.priority,|p|p[i]));
            triangles.push(Triangle {
                #[cfg(feature="enhanced-renderer")] material_hsl:None, texture_mapping: Some(crate::render3d::TextureMapping { vertices: mapping.map(|v| vertices[v as usize]), local: None }),
                local_vertices: None,
                model_bounds: None,
                model_order: None, depth_bias: model.face_depth_bias(i),
                view: 0,
                vertices: positions,
                colors: color,
                        flat: colors[i][2] == -1,
                uv: texture_uv(mapping.map(|v| vertices[v as usize]), positions),
                texture: model
                    .face_textures
                    .as_ref()
                    .and_then(|t| (t[i] != -1).then_some(t[i] as u16)),
                alpha: if actor_alpha == 255 { 255 } else { crate::actor_visual::face_alpha(model.face_alpha(i), actor_alpha) },
                near_alpha: Some(model.alpha.as_ref().map_or(0, |a| a[i])),
                tag: 1,
            });
        }
        }
        let mut camera = Camera::look_at([0.0; 3], [0.0, 0.0, 1.0], 512.0)?;
        if let Some(zoom) = orthographic {
            if zoom == 0 {
                return Err(Error::Invalid("zero orthographic model zoom"));
            }
            camera.orthographic = Some(zoom as f32);
        }
        if let Some(origin)=origin {
            let squared=parts.iter().flat_map(|p|&p.0.vertices).map(|&[x,y,z]|x*x+z*z+y*y).fold(0.0f32,f32::max);
            let radius=(squared as f64).sqrt().ceil() as i32;
            frame.draw_icon(&camera,&triangles,has_priorities.then_some(priorities.as_slice()),radius,origin,&self.palette,&self.textures);
        } else {
            frame.draw(&camera, &triangles, &self.palette, &self.textures);
        }
        Ok(Pixels {
            canvas: size,
            offset: [0; 2],
            size,
            pixels: frame.pixels,
        })
    }
    pub fn load(cache: &mut DiskCache, brightness: f64) -> Result<Self> {
        let font_id = cache
            .reference(13)?
            .groups
            .iter()
            .find(|g| g.name_hash == Some(name_hash("p11_full")))
            .ok_or(Error::Invalid("small font missing"))?
            .id;
        let font = Font::load(cache, font_id)?;
        let mut textures = HashMap::new();
        for (id, d) in texture::Definition::load_all(cache)? {
            textures.insert(id, Texture::load(cache, d, 128, brightness)?);
        }
        Ok(Self {
            brightness,
            models: Models::default(),
            palette: floor::palette(brightness)?,
            textures,
            font,
            cached: HashMap::new(),
        })
    }
    /// sd.ga invalidates item sprites and texture pixels after gamma changes.
    pub fn set_brightness(&mut self,cache:&mut DiskCache,brightness:f64)->Result<bool> {
        if self.brightness.to_bits()==brightness.to_bits() {return Ok(false);}
        let palette=floor::palette(brightness)?;
        let textures=self.textures.iter().map(|(&id,t)|Ok((id,Texture::load(cache,t.definition.clone(),t.size,brightness)?))).collect::<Result<HashMap<_,_>>>()?;
        self.palette=palette;self.textures=textures;self.cached.clear();self.brightness=brightness;Ok(true)
    }
    pub fn get(
        &mut self,
        cache: &mut DiskCache,
        defs: &Definitions,
        id: u32,
        options: Options,
    ) -> Result<Pixels> {
        self.render(cache, defs, id, options, &mut HashSet::new())
    }
    fn render(
        &mut self,
        cache: &mut DiskCache,
        defs: &Definitions,
        id: u32,
        mut options: Options,
        active: &mut HashSet<u32>,
    ) -> Result<Pixels> {
        if options.size != [36, 32] || options.quantity == -1 {
            options.quantity_mode = 0;
        } else if options.quantity_mode == 2 && options.quantity != 1 {
            options.quantity_mode = 1;
        }
        if let Some(p) = self.cached.get(&(id, options)) {
            return Ok(p.clone());
        }
        if !active.insert(id) {
            return Err(Error::Invalid("cyclic item icon"));
        }
        let d = defs
            .items
            .get(&id)
            .ok_or(Error::Invalid("item icon definition"))?;
        let d = defs
            .items
            .get(&d.count_variant(options.quantity))
            .ok_or(Error::Invalid("item icon variant"))?;
        let overlay = if d.note[1] != -1 {
            Some(self.render(
                cache,
                defs,
                d.note[0] as u32,
                Options {
                    quantity: 10,
                    outline: 1,
                    shadow: 0,
                    quantity_mode: 0,
                    enlarged: true,
                    size: options.size,
                },
                active,
            )?)
        } else if d.bought[1] != -1 {
            Some(self.render(
                cache,
                defs,
                d.bought[0] as u32,
                Options {
                    quantity_mode: 0,
                    enlarged: false,
                    ..options
                },
                active,
            )?)
        } else if d.placeholder[1] != -1 {
            Some(self.render(
                cache,
                defs,
                d.placeholder[0] as u32,
                Options {
                    outline: 0,
                    shadow: 0,
                    quantity_mode: 0,
                    enlarged: false,
                    ..options
                },
                active,
            )?)
        } else {
            None
        };
        let [w, h] = options.size;
        let mut frame = Frame::new(w, h)?;
        frame.coarse_shading = false;
        frame.clear(0);
        if d.placeholder[1] != -1 {
            frame
                .pixels
                .copy_from_slice(&overlay.as_ref().unwrap().pixels);
        }
        let model = self.models.inventory(cache, defs, d.id, 1, None)?;
        let colors = model.light(d.ambient + 64, d.contrast + 768, [-50, -10, -50])?;
        let mut zoom = d.zoom;
        if w != 36 {
            zoom = zoom.wrapping_mul(32) / (w as i32);
        }
        if options.enlarged {
            zoom = (zoom as f64 * 1.5) as i32;
        } else if options.outline == 2 {
            zoom = (zoom as f64 * 1.04) as i32;
        }
        let vertices = icon_vertices(&model, d, zoom)?;
        let mut triangles = Vec::new();
        let mut priorities = Vec::new();
        for (i, face) in model.faces.iter().enumerate() {
            let mut color = colors[i];
            if color[2] == -2 {
                continue;
            }
            if color[2] == -1 {
                color = [color[0]; 3];
            }
            let positions = face.map(|v| vertices[v as usize]);
            let mapping = model
                .texture_coords
                .as_ref()
                .and_then(|c| {
                    if c[i] == -1 {
                        None
                    } else {
                        model.texture_triangles.get(c[i] as u8 as usize)
                    }
                })
                .unwrap_or(face);
            priorities.push(model.priorities.as_ref().map_or(model.priority,|p|p[i]));
            triangles.push(Triangle {
                #[cfg(feature="enhanced-renderer")] material_hsl:None, texture_mapping: Some(crate::render3d::TextureMapping { vertices: mapping.map(|v| vertices[v as usize]), local: None }),
                local_vertices: None,
                model_bounds: None,
                model_order: None, depth_bias: model.face_depth_bias(i),
                view: 0,
                vertices: positions,
                colors: color,
                        flat: colors[i][2] == -1,
                uv: texture_uv(mapping.map(|v| vertices[v as usize]), positions),
                texture: model
                    .face_textures
                    .as_ref()
                    .and_then(|t| (t[i] != -1).then_some(t[i] as u16)),
                alpha: model.face_alpha(i),
                near_alpha: Some(model.alpha.as_ref().map_or(0, |a| a[i])),
                tag: id as u64 + 1,
            });
        }
        let mut camera = Camera::look_at([0.0; 3], [0.0, 0.0, 1.0], 512.0)?;
        camera.screen_offset = [
            (h >> 1) as f32 - (w / 2) as f32,
            (h >> 1) as f32 - (h / 2) as f32,
        ];
        let [radius,origin]=crate::item_model::icon_depth_bounds(&model,d,zoom);
        frame.draw_icon(&camera,&triangles,model.priorities.as_ref().map(|_|priorities.as_slice()),radius,origin,&self.palette,&self.textures);
        let mut sprite = Pixels {
            canvas: options.size,
            offset: [0; 2],
            size: options.size,
            pixels: frame.pixels,
        };
        let composite = |target: &mut Pixels, overlay: &Pixels| {
            for (dest, &src) in target.pixels.iter_mut().zip(&overlay.pixels) {
                if src != 0 {
                    *dest = src;
                }
            }
        };
        if d.bought[1] != -1 {
            composite(&mut sprite, overlay.as_ref().unwrap());
        }
        if options.outline >= 1 {
            sprite.outline(1);
        }
        if options.outline >= 2 {
            sprite.outline(0xffffff);
        }
        if options.shadow != 0 {
            sprite.shadow(options.shadow);
        }
        if d.note[1] != -1 {
            composite(&mut sprite, overlay.as_ref().unwrap());
        }
        if options.quantity_mode == 1 || (options.quantity_mode == 2 && d.stackable == 1) {
            let mut raster = Raster::from_pixels(w, h, sprite.pixels)?;
            self.font.draw(
                &mut raster,
                &quantity_text(options.quantity),
                0,
                9,
                Style {
                    color: 0xffff00,
                    shadow: Some(1),
                    alpha: 256,
                },
                &[],
            )?;
            sprite.pixels = raster.pixels().to_vec();
        }
        active.remove(&id);
        if !options.enlarged && options.size == [36, 32] {
            if self.cached.len() >= 512 {
                self.cached.clear();
            }
            self.cached.insert((id, options), sprite.clone());
        }
        Ok(sprite)
    }
}
pub fn quantity_text(quantity: i32) -> String {
    if quantity < 100000 {
        format!("<col=ffff00>{quantity}</col>")
    } else if quantity < 10000000 {
        format!("<col=ffffff>{}K</col>", quantity / 1000)
    } else {
        format!("<col=00ff80>{}M</col>", quantity / 1000000)
    }
}
