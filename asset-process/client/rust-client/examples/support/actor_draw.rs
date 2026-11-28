//! Asset system helper (examples only; nothing here is part of the client): draw one assembled figure the way
//! the LIVE client draws a player standing in the world, instead of the plain per-pixel depth test the figure
//! tools used before.
//!
//! What the live client does (all in src/, quoted so this can be checked):
//!   world_actors.rs  place_actor(.., crate::model_order::Mode::Priority) for every player body, and for an NPC
//!                    `Mode::actor(false, npc.z_buffer, use_player)` which is Priority only when the NPC's
//!                    definition clears z_buffer (opcode 147) or it borrows a player body; otherwise Auto.
//!   render3d.rs      draw_iter -> draw_model_group -> draw_model_phase. For Mode::Priority (3):
//!                      `let passes = if mode == 3 { 2 } else { 1 };`
//!                      `self.write_color = pass == 0;`
//!                      `let write_depth = if mode == 3 { pass == 1 } else { mode != 4 };`
//!                      `crate::model_order::sorted(pass_faces, mode == 3 && priorities.is_some())`
//!                    so pass 0 paints colour, faces ordered by priority group and then far-to-near by the face's
//!                    average depth, WITHOUT writing depth (the faces of one model never hide each other by
//!                    depth; a later face simply paints over an earlier one), while still being tested against
//!                    the depth other models wrote earlier; pass 1 then writes the model's depth without colour.
//!                    The per-face depth offset (`depth_bias`, Model::face_depth_bias = face_flags * 2) only
//!                    shifts the depth compared and written, so inside one player it does not change which face
//!                    shows; it changes how the player's faces compare with OTHER models' depth.
//!
//! How this file reproduces it without touching src/: it builds the same `Triangle`s world_actors.rs `place`
//! builds (model_bounds = the model's cylinder from Bounds::new, model_order = Submission::new with mode
//! Priority) and hands them to the public `Frame::draw`, which is the function the live frame uses
//! (`draw_iter`). The grouping, depth buckets, priority interleave, the two passes and the "skip the model when
//! a face falls outside its bounding sphere" rule are therefore the client's own code, not a copy.
//! What is NOT the live world view: the camera. Vertices arrive already turned and pushed back by
//! interface_model::vertices (the old figure camera, focal 512), so pictures register with the old ones.
use gielinor_client::{
    disk_cache::DiskCache, floor, item_icon::Icons, model::Model, model_bounds::Bounds,
    model_order::{Mode, Submission}, render3d::{texture_uv, Camera, Frame, TextureMapping, Triangle},
    sprite_pixels::Pixels, texture::{self, Texture},
};
use std::collections::HashMap;
pub type R<T> = Result<T, Box<dyn std::error::Error>>;
/// Which routine paints a figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Draw {
    /// The live world path for a player: priority-ordered colour pass, then a depth-only pass. The default.
    Actor,
    /// The old figure picture: every face depth-tested per pixel against the others (item_icon draw_models).
    Depth,
    /// The live path for an ordinary NPC or a scenery object (Mode::Auto): faces in model order, depth-tested
    /// and depth-written per pixel, priorities ignored, per-face depth offsets honoured.
    Auto,
    /// The client's interface path for a player model (item_icon draw_widget_models -> draw_icon ->
    /// draw_legacy): priority order with no depth buffer at all and whole-pixel projection.
    Widget,
}
impl Draw {
    pub fn parse(s: &str) -> R<Self> {
        Ok(match s {
            "actor" | "player" | "priority" => Self::Actor,
            "depth" | "zbuffer" => Self::Depth,
            "auto" | "npc" | "scenery" => Self::Auto,
            "widget" | "interface" => Self::Widget,
            _ => return Err(format!("draw={s}: use actor (default), depth, auto or widget").into()),
        })
    }
}
/// View depth of the model's own origin after interface_model::vertices with no offset and no height centring,
/// when the model was first shifted down by `origin_y` (asset_pose shifts by half the rest height).
pub fn origin_depth(origin_y: i32, pitch: i32, zoom: i32) -> i32 {
    let a = pitch as f64 * 0.0030679615;
    let (sx, cx) = ((a.sin() * 65536.0) as i32, (a.cos() * 65536.0) as i32);
    let (y, z) = (origin_y + (sx.wrapping_mul(zoom) >> 16), cx.wrapping_mul(zoom) >> 16);
    y.wrapping_mul(sx).wrapping_add(z.wrapping_mul(cx)) >> 16
}
pub struct Painter {
    palette: Vec<u32>,
    definitions: HashMap<u16, texture::Definition>,
    textures: HashMap<u16, Texture>,
}
impl Painter {
    /// Brightness 0.8 and 128-pixel textures, as Icons::load(cache, 0.8) uses for the depth mode.
    pub fn load(cache: &mut DiskCache) -> R<Self> {
        Ok(Self { palette: floor::palette(0.8)?, definitions: texture::Definition::load_all(cache)?, textures: HashMap::new() })
    }
    /// `vertices` are the view-space vertices (camera at the origin looking along +z); `origin_z` is the view
    /// depth of the model's own origin (its feet), which the client uses as the centre of its depth buckets.
    /// Returns the picture and a note when the live client would drop faces or the whole model.
    pub fn draw(&mut self, cache: &mut DiskCache, icons: &Icons, model: &Model, colors: &[[i32; 3]], vertices: &[[f32; 3]],
                origin_z: i32, size: [usize; 2], mode: Draw) -> R<(Pixels, Option<String>)> {
        match mode {
            Draw::Depth => return Ok((icons.draw_model(model, colors, vertices, size, None)?, None)),
            Draw::Widget => return Ok((icons.draw_widget_models(&[(model, colors, vertices)], size, None, 0, origin_z)?, None)),
            _ => {}
        }
        if let Some(ids) = &model.face_textures {
            for &id in ids {
                if id != -1 && !self.textures.contains_key(&(id as u16)) {
                    if let Some(d) = self.definitions.get(&(id as u16)) {
                        self.textures.insert(id as u16, Texture::load(cache, d.clone(), 128, 0.8)?);
                    }
                }
            }
        }
        // world_actors.rs place: bounds come from the posed model's own vertices, before the draw turn
        let mut bounds = Bounds::new(&model.vertices, [0, 0, origin_z]);
        bounds.selection = None;
        let order_mode = if mode == Draw::Actor { Mode::Priority } else { Mode::Auto };
        let mut triangles = Vec::new();
        for (i, face) in model.faces.iter().enumerate() {
            if colors[i][2] == -2 {
                continue;
            }
            let color = if colors[i][2] == -1 { [colors[i][0]; 3] } else { colors[i] };
            let positions = face.map(|v| vertices[v as usize]);
            let map = model.texture_coords.as_ref()
                .and_then(|t| if t[i] == -1 { None } else { model.texture_triangles.get(t[i] as u8 as usize) })
                .unwrap_or(face);
            triangles.push(Triangle {
                #[cfg(feature = "enhanced-renderer")]
                material_hsl: None,
                texture_mapping: Some(TextureMapping { vertices: map.map(|v| vertices[v as usize]), local: None }),
                local_vertices: None,
                model_bounds: Some(bounds),
                model_order: Some(Submission { mode: order_mode, ..Submission::new(model, i, 0) }),
                depth_bias: model.face_depth_bias(i),
                view: 0,
                vertices: positions,
                colors: color,
                flat: colors[i][2] == -1,
                uv: texture_uv(map.map(|v| vertices[v as usize]), positions),
                texture: model.face_textures.as_ref().and_then(|t| (t[i] != -1).then_some(t[i] as u16)),
                alpha: model.face_alpha(i),
                near_alpha: Some(model.alpha.as_ref().map_or(0, |a| a[i])),
                tag: 1,
            });
        }
        // The client's own gate (draw_model_group / draw_model_phase), repeated only to REPORT it: a face whose
        // bucket is outside the model's bounding sphere is dropped, and one below 0 or at 6000 drops the model.
        let extent = |h: i32| ((bounds.radius * bounds.radius + h * h) as f64).sqrt().ceil() as i32;
        let (offset, diameter) = (extent(bounds.height), extent(bounds.height) + extent(bounds.below));
        let (mut outside, mut fatal) = (0usize, false);
        for t in &triangles {
            let depth = t.vertices.iter().map(|p| p[2] as i32 - origin_z).sum::<i32>() / 3 + offset;
            fatal |= depth < 0 || depth >= 6000;
            outside += usize::from(depth >= diameter);
        }
        let note = if diameter >= 6000 || (fatal && mode == Draw::Actor) {
            Some("the live client would not draw this model at all here (a face lies outside its depth range)".to_string())
        } else if outside > 0 && mode == Draw::Actor {
            Some(format!("the live client drops {outside} faces here (beyond the model's bounding sphere)"))
        } else { None };
        let mut frame = Frame::new(size[0], size[1])?;
        frame.clear(0);
        let camera = Camera::look_at([0.0; 3], [0.0, 0.0, 1.0], 512.0)?;
        frame.draw(&camera, &triangles, &self.palette, &self.textures);
        Ok((Pixels { canvas: size, offset: [0; 2], size, pixels: frame.pixels }, note))
    }
}
