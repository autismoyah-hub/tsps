//! Assemble terrain and object geometry for native scene rendering.
use crate::{
    disk_cache::DiskCache,
    floor::{self, Floors},
    object::Definition,
    object_model::Models,
    render3d::{texture_uv, Triangle},
    scene::Scene,
    tile_mesh::TileMesh,
    Error, Result,
};
use std::{collections::HashMap, sync::Arc};
struct AnimatedObject {
    key: [i32; 4],
    id: u32,
    part: usize,
    generation: u64,
    placement: u64,
    initial_tick: bool,
    random_start: bool,
    model: Arc<crate::model::Model>,
    sequence: u32,
    clock: crate::animation_runtime::Clock,
    rotation: u8,
    scene_shape: u8,
    plane: usize,
    centre: [i32; 2],
    base: i32,
    offset: [i32; 2],
    contour: i32,
    contour_origin: [i32; 3],
    triangles: Vec<(usize, usize)>,
    #[cfg(feature = "private-test-adapter")]
    model_ids: Vec<u32>,
    #[cfg(feature = "private-test-adapter")]
    randomized: bool,
    #[cfg(feature = "private-test-adapter")]
    last: Option<crate::location_animation_receipt::Observation>,
}
pub struct WorldMesh {
    #[cfg(feature="custom-assets")]
    authored_visual_batches: std::collections::HashSet<(u64,usize)>,
    #[cfg(feature = "custom-assets")]
    cutaway: crate::environment_cutaway::Cutaway,
    #[cfg(feature = "custom-assets")]
    motion: crate::environment_motion::Motion,
    pub triangles: Vec<Triangle>,
    pub terrain_submissions: Vec<crate::terrain_scene::Submission>,
    pub object_submissions: Vec<crate::object_submission::Submission>,
    pub boundary_submissions: Vec<crate::object_submission::BoundarySubmission>,
    pub decoration_submissions: Vec<crate::object_submission::DecorationSubmission>,
    pub ground_submissions: Vec<crate::object_submission::GroundSubmission>,
    minimum_planes: Vec<u8>,
    pub object_instances: usize,
    pub pending_assets: bool,
    pub item_support: HashMap<[u8; 3], i32>,
    animated: Vec<AnimatedObject>,
    transform_varps: HashMap<u16, i32>,
    /// What the latest `advance` wrote for each animated location part.
    #[cfg(feature = "private-test-adapter")]
    observations: Vec<crate::location_animation_receipt::Observation>,
}
fn shade(color: i32, light: i32) -> i32 {
    if color == -2 {
        12345678
    } else if color == -1 {
        light.clamp(2, 126)
    } else {
        (color & 65408) + ((color & 127) * light / 128).clamp(2, 126)
    }
}
impl WorldMesh {
    /// Register only package-admitted, exactly placed root art for this frame.
    /// Bounds and animation metadata remain intact in the stored mesh.
    #[cfg(feature="custom-assets")]
    pub fn register_authored_art(&self,frame:&mut crate::render3d::Frame) {
        frame.register_authored_art(self.authored_visual_batches.iter().copied());
    }
    #[cfg(feature = "custom-assets")]
    pub fn update_environment_cutaway(&self,camera:&crate::render3d::Camera,focus:[f32;3]) {
        self.cutaway.update(camera.position,focus);
    }

    /// Filter submission without rebuilding geometry or changing pick identity.
    pub fn visible_triangles(&self, max_plane: u8) -> impl Iterator<Item = &Triangle> {
        self.visible_triangles_or(max_plane, |_|false)
    }
    /// Scene-registered models have separate pass decisions, which may admit an
    /// alpha pass through a different footprint tile than the object's origin.
    pub fn visible_triangles_or<'a>(&'a self, max_plane:u8, mut included:impl FnMut(&Triangle)->bool+'a) -> impl Iterator<Item=&'a Triangle>+'a {
        self.triangles.iter().zip(&self.minimum_planes)
            .enumerate().filter_map(move |(_index,(triangle,&plane))| {
                #[cfg(feature = "custom-assets")]
                if !self.cutaway.visible(_index){return None;}
                (plane<=max_plane || included(triangle)).then_some(triangle)
            })
    }
    pub fn transforms_changed(&self, values: &[i32]) -> bool {
        self.transform_varps
            .iter()
            .any(|(&id, &value)| values.get(id as usize).copied().unwrap_or(0) != value)
    }
    pub fn build(scene: &Scene, cache: &mut DiskCache, max_plane: usize) -> Result<Self> {
        Self::build_with_varps(scene, cache, max_plane, &[])
    }
    pub fn build_with_varps(
        scene: &Scene,
        cache: &mut DiskCache,
        max_plane: usize,
        values: &[i32],
    ) -> Result<Self> {
        if max_plane >= 4 {
            return Err(Error::Invalid("world plane"));
        }
        let (width, height) = (scene.width, scene.height);
        let floors = Floors::load(cache)?;
        let defs = Definition::load_all(cache)?;
        let renderer=scene.definition_renderer();
        renderer.validate(&defs)?;
        let mut models = Models::default();
        let stride = height + 1;
        let plane_size = (width + 1) * stride;
        let mut shadows = vec![0_i8; 4 * plane_size];
        shadows[..plane_size].copy_from_slice(scene.terrain_shadows());
        for object in scene.initial_objects().iter().filter(|object| !object.runtime) {
            let def = defs.get(&object.id).ok_or(Error::Invalid("shadow object definition"))?;
            let selected=renderer.get(object.plane as usize,[object.x as i32,object.y as i32],def);
            let def=selected.as_ref();
            if !def.clipped { continue; }
            let mut radius = None;
            if matches!(object.shape, 10 | 11) && !object.rejected
                && def.animation.is_none() && def.transforms.is_none() && !def.non_flat_shading {
                let Some(model) = models.object(cache, def, 10, object.rotation)? else { continue; };
                radius = Some(crate::terrain_light::model_radius(&model.vertices));
            }
            let start = usize::from(object.plane) * plane_size;
            crate::terrain_light::stamp(&mut shadows[start..start + plane_size], stride,
                usize::from(object.x), usize::from(object.y), crate::terrain_light::Caster {
                    shape: object.shape, rotation: object.rotation, size: def.dimensions(object.rotation),
                    clipped: def.clipped, accepted: !object.rejected, radius,
                });
        }
        let static_lighting = crate::scene_light::bake_static(scene, cache, &defs, &mut models)?;
        let mut varbits = HashMap::new();
        let mut out = Self {
            #[cfg(feature="custom-assets")]
            authored_visual_batches: Default::default(),
            #[cfg(feature = "custom-assets")]
            cutaway: scene.environment_cutaway()?,
            #[cfg(feature = "custom-assets")]
            motion: crate::environment_motion::Motion::empty(),
            triangles: Vec::new(),
            terrain_submissions: Vec::new(),
            object_submissions: Vec::new(),
            boundary_submissions: Vec::new(),
            decoration_submissions: Vec::new(),
            ground_submissions: Vec::new(),
            minimum_planes: Vec::new(),
            object_instances: 0,
            pending_assets: false,
            item_support: HashMap::new(),
            animated: Vec::new(),
            transform_varps: HashMap::new(),
            #[cfg(feature = "private-test-adapter")]
            observations: Vec::new(),
        };
        for plane in 0..4 {
            if plane > max_plane
                && !(1..width.saturating_sub(1)).any(|x| {
                    (1..height.saturating_sub(1)).any(|y| {
                        scene.loaded(plane, x, y)
                            && scene
                                .tile_render_plane(plane, x, y)
                                .is_some_and(|p| p <= max_plane)
                    })
                })
            {
                continue;
            }
            let mut lights = vec![96; width * height];
            for x in 1..width.saturating_sub(1) {
                for y in 1..height.saturating_sub(1) {
                    let points = [(x, y), (x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];
                    lights[x * height + y] = crate::terrain_light::brightness(
                        points.map(|(x, y)| scene.height(plane, x, y).unwrap()),
                        points.map(|(x, y)| shadows[plane * plane_size + x * stride + y]),
                    )?;
                }
            }
            for x in 1..width.saturating_sub(1) {
                for y in 1..height.saturating_sub(1) {
                    if !scene.loaded(plane, x, y) {
                        continue;
                    }
                    if scene
                        .tile_render_plane(plane, x, y)
                        .is_none_or(|p| p > max_plane)
                    {
                        continue;
                    }
                    let tile = scene.tile(plane, x, y).unwrap();
                    let underlay = tile.underlay & 32767;
                    let overlay = (tile.overlay as u16) & 32767;
                    if underlay == 0 && overlay == 0 {
                        continue;
                    }
                    let hsl = if underlay == 0 {
                        -1
                    } else {
                        let (mut hue, mut sat, mut light, mut mult, mut count) = (0, 0, 0, 0, 0);
                        for dx in x.saturating_sub(4)..=(x + 5).min(width - 1) {
                            for dy in y.saturating_sub(4)..=(y + 5).min(height - 1) {
                                let id = scene.tile(plane, dx, dy).unwrap().underlay & 32767;
                                if id != 0 {
                                    let d = floors
                                        .underlays
                                        .get(&(id - 1))
                                        .ok_or(Error::Invalid("world underlay"))?;
                                    hue += d.hue;
                                    sat += d.saturation;
                                    light += d.lightness;
                                    mult += d.hue_multiplier;
                                    count += 1;
                                }
                            }
                        }
                        floor::pack_hsl(hue * 256 / mult, sat / count, light / count)
                    };
                    let light = [
                        lights[x * height + y],
                        lights[(x + 1) * height + y],
                        lights[(x + 1) * height + y + 1],
                        lights[x * height + y + 1],
                    ];
                    let heights = [
                        scene.height(plane, x, y).unwrap(),
                        scene.height(plane, x + 1, y).unwrap(),
                        scene.height(plane, x + 1, y + 1).unwrap(),
                        scene.height(plane, x, y + 1).unwrap(),
                    ];
                    let (shape, overlay_hsl, texture) = if overlay == 0 {
                        (0, -2, None)
                    } else {
                        let def = floors
                            .overlays
                            .get(&(overlay - 1))
                            .ok_or(Error::Invalid("world overlay"))?;
                        let hsl = if def.texture.is_some() {
                            -1
                        } else if def.rgb == 0xff00ff {
                            -2
                        } else {
                            floor::pack_hsl(
                                def.primary.hue,
                                def.primary.saturation,
                                def.primary.lightness,
                            )
                        };
                        (tile.overlay_shape + 1, hsl, def.texture.map(u16::from))
                    };
                    out.terrain_submissions.push(crate::terrain_scene::Submission {
                        physical_plane:plane,tile:[x,y],shaped:shape>1,
                        visible_paint:shade(if shape==0 {hsl}else{overlay_hsl},light[2])!=12345678,
                    });
                    let mesh = TileMesh::for_scene(
                        shape,
                        tile.overlay_rotation,
                        x as i32,
                        y as i32,
                        heights,
                        light.map(|l| shade(hsl, l)),
                        light.map(|l| shade(overlay_hsl, l)),
                    )?;
                    for (i, f) in mesh.faces.iter().enumerate() {
                        // Keep hidden faces as clip-only geometry: the original
                        // projection checks every vertex before drawing any face.
                        let colors = if !mesh.overlay[i] && underlay == 0 {
                            [12345678; 3]
                        } else { mesh.colors[i] };
                        let vertices = f.map(|v| mesh.vertices[v].map(|v| v as f32));
                        let uv = vertices.map(|v| {
                            [
                                (v[0] - x as f32 * 128.0) / 128.0,
                                (v[2] - y as f32 * 128.0) / 128.0,
                            ]
                        });
                        let mapping = crate::terrain_texture_mapping::indices(shape == 1, *f, heights.iter().all(|&h| h == heights[0]));
                        let mapping = mapping.map(|i| mesh.vertices[i].map(|v| v as f32));
                        out.minimum_planes
                            .push(scene.tile_render_plane(plane, x, y).unwrap() as u8);
                        out.triangles.push(Triangle {
                            near_alpha: None,
                            #[cfg(feature="enhanced-renderer")] material_hsl:if mesh.overlay[i] {(overlay_hsl>=0).then_some(overlay_hsl as u16)} else {(hsl>=0).then_some(hsl as u16)}, texture_mapping: Some(crate::render3d::TextureMapping { vertices: mapping, local: Some(mapping) }),
                            local_vertices: Some(vertices),
                            model_bounds: None,
                            model_order: None, depth_bias: 0,
                            view: 0,
                            vertices,
                            colors,
                            flat: false,
                            uv,
                            texture: if mesh.overlay[i] { texture } else { None },
                            alpha: 0,
                            tag: 1 << 63 | ((plane as u64) << 28) | ((x as u64) << 14) | y as u64,
                        });
                    }
                }
            }
        }
        let render_objects = scene.objects.iter().enumerate().map(|(index, object)| (index, object, None))
            .chain(scene.retained_objects().iter().map(|old| (old.original_index, &old.object, Some(old))));
        for (object_index, obj, retained) in render_objects {
            let traversal_id = retained.map_or_else(|| scene.object_traversal_id(object_index), |old| Some(old.traversal_id));
            if scene.object_render_plane(obj).is_none_or(|p| p > max_plane) {
                continue;
            }
            let def = defs
                .get(&obj.id)
                .ok_or(Error::Invalid("world object definition"))?;
            let selected=renderer.get(obj.plane as usize,[obj.x as i32,obj.y as i32],def);
            let mut def=selected.as_ref();
            let key = [
                scene
                    .object_scene_plane(obj)
                    .map_or(-1, |plane| plane as i32),
                scene.base_x + obj.x as i32,
                scene.base_y + obj.y as i32,
                crate::location_update::category(obj.shape)? as i32,
            ];
            let overridden = match retained {
                Some(old) => old.animation.as_ref(),
                None => scene.location_animations.get(&key),
            };
            let placement_size = def.dimensions(obj.rotation);
            let initial_animation = def.animation;
            let randomize_animation = def.randomize_animation;
            let dynamic =
                overridden.is_some() || initial_animation.is_some() || def.transforms.is_some();
            let supports_items = def.support_items == 1
                && overridden.is_none()
                && def.animation.is_none()
                && def.transforms.is_none()
                && (9..=21).contains(&obj.shape);
            if def.transforms.is_some() {
                let (bit, value) = if let Some(id) = def.transform_varbit {
                    if !varbits.contains_key(&id) {
                        varbits.insert(
                            id,
                            crate::script_runtime::Varbit::decode(
                                &cache.file(2, 14, id as u32, [0; 4])?,
                            )?,
                        );
                    }
                    let bit = &varbits[&id];
                    let raw = values.get(bit.base as usize).copied().unwrap_or(0);
                    out.transform_varps.insert(bit.base, raw);
                    (bit.get(raw), 0)
                } else if let Some(id) = def.transform_varp {
                    let raw = values.get(id as usize).copied().unwrap_or(0);
                    out.transform_varps.insert(id, raw);
                    (0, raw)
                } else {
                    (0, 0)
                };
                let Some(id) = def.transformed_id(bit, value) else {
                    continue;
                };
                def = defs
                    .get(&id)
                    .ok_or(Error::Invalid("object transform definition"))?;
            }
            let [xs, ys] = placement_size;
            let (x, y) = (obj.x as usize, obj.y as usize);
            let (plane, r) = (obj.plane as usize, obj.rotation);
            let height_plane = scene.object_height_plane(obj);
            let centre_x = x as i32 * 128 + xs as i32 * 64;
            let centre_y = y as i32 * 128 + ys as i32 * 64;
            let draw_center = crate::location_draw_position::center(obj.shape, [x as i32, y as i32], [xs, ys]);
            let (x1, x2) = if x + xs as usize <= width {
                (x + xs as usize / 2, x + (xs as usize + 1) / 2)
            } else {
                (x, x + 1)
            };
            let (y1, y2) = if y + ys as usize <= height {
                (y + ys as usize / 2, y + (ys as usize + 1) / 2)
            } else {
                (y, y + 1)
            };
            let base = (scene.height(height_plane, x1, y1).unwrap()
                + scene.height(height_plane, x2, y1).unwrap()
                + scene.height(height_plane, x1, y2).unwrap()
                + scene.height(height_plane, x2, y2).unwrap())
                >> 2;
            let parts = overridden
                .copied()
                .unwrap_or(crate::location_animation::parts(
                    obj.shape,
                    r,
                    initial_animation.map(u32::from),
                    0,
                )?);
            let offsets = if (4..=8).contains(&obj.shape) {
                scene.decoration_part_offsets(object_index).unwrap_or([[0, 0]; 2])
            } else { [[0, 0]; 2] };
            for (part, specification) in parts.into_iter().enumerate() {
                let Some(specification) = specification else {
                    continue;
                };
                let offset = offsets[part];
                let shape = specification.shape;
                let rotation = specification.rotation;
                let contour_origin = if dynamic {
                    crate::location_animation::contour_origin(
                        def.size,
                        rotation,
                        [x, y],
                        |x, z| scene.height(height_plane, x, z),
                    )?
                } else {
                    [centre_x, base, centre_y]
                };
                let Some(model) = models.object(cache, def, shape, rotation)? else {
                    continue;
                };
                let baked = if !dynamic && !obj.runtime {
                    static_lighting.get(&(obj.initial_index.unwrap_or(object_index), part))
                } else { None };
                let model = baked.map_or(model, |entry| entry.model.clone());
                let colors = if let Some(entry) = baked { entry.colors.clone() } else {
                    model.light(64 + def.ambient as i32, 768 + def.contrast, [-50, -10, -50])?
                };
                let local_vertices = if baked.is_some() { model.vertices.clone() } else {
                    crate::model_contour::rendered(&model, contour_origin, def.contour, |x,z| scene.height(height_plane,x,z))?
                };
                let position = [draw_center[0] + offset[0], base, draw_center[1] + offset[1]];
                let draw_bounds = crate::model_bounds::Bounds::new(&local_vertices, position).with_selection(&local_vertices, if obj.shape == 11 { 256 } else { 0 }, false);
                #[cfg(feature = "custom-assets")]
                let draw_bounds={let mut bounds=draw_bounds;if scene.visual_only_id(obj.id){bounds.selection=None;}bounds};
                let transform = crate::model_draw::Transform::new(if obj.shape == 11 { 256 } else { 0 }, position)?;
                let mut support_height = 0.0f32;
                let vertices = local_vertices.iter().copied().map(|v| {
                    support_height = support_height.max(-v[1]);
                    transform.vertex(v)
                }).collect::<Vec<_>>();
                if let Some(id) = traversal_id.filter(|_| supports_items) {
                    for tile in scene.object_support_tiles(id) {
                        let value = out.item_support.entry(tile).or_insert(0);
                        *value = (*value).max(support_height.ceil() as i32);
                    }
                } else if let Some(support_plane) =
                    scene.object_scene_plane(obj).filter(|_| supports_items)
                {
                    let [width, height] = if matches!(obj.shape, 10 | 11) {
                        [xs, ys]
                    } else {
                        [1, 1]
                    };
                    for dx in 0..width {
                        for dy in 0..height {
                            let (tx, ty) = (x + dx as usize, y + dy as usize);
                            if tx < scene.width && ty < scene.height {
                                let value = out
                                    .item_support
                                    .entry([support_plane as u8, tx as u8, ty as u8])
                                    .or_insert(0);
                                *value = (*value).max(support_height.ceil() as i32);
                            }
                        }
                    }
                }
                let mut animation_triangles = Vec::new();
                let batch = out.triangles.len();
                for (i, face) in model.faces.iter().enumerate() {
                    if colors[i][2] == -2 {
                        continue;
                    }
                    let mut color = colors[i];
                    if color[2] == -1 {
                        color = [color[0]; 3];
                    }
                    let positions = face.map(|v| vertices[v as usize]);
                    let texture = model
                        .face_textures
                        .as_ref()
                        .and_then(|t| (t[i] != -1).then_some(t[i] as u16));
                    let mapface = model
                        .texture_coords
                        .as_ref()
                        .and_then(|t| {
                            if t[i] == -1 {
                                None
                            } else {
                                model.texture_triangles.get(t[i] as u8 as usize)
                            }
                        })
                        .unwrap_or(face);
                    let uv = texture_uv(mapface.map(|v| vertices[v as usize]), positions);
                    animation_triangles.push((out.triangles.len(), i));
                    out.minimum_planes
                        .push(scene.object_render_plane(obj).unwrap() as u8);
                    #[cfg(feature = "custom-assets")]
                    out.cutaway.bind_face(obj.id,i,out.triangles.len());
                    out.triangles.push(Triangle {
                        #[cfg(feature="enhanced-renderer")] material_hsl:Some(model.colors[i]), texture_mapping: Some(crate::render3d::TextureMapping { vertices: mapface.map(|v| vertices[v as usize]), local: Some(mapface.map(|v| local_vertices[v as usize])) }),
                        local_vertices: Some(face.map(|v| local_vertices[v as usize])),
                        model_bounds: Some(draw_bounds),
                        model_order: Some(crate::model_order::Submission::new(&model, i, batch)), depth_bias: model.face_depth_bias(i),
                        view: 0,
                        vertices: positions,
                        colors: color,
                        flat: colors[i][2] == -1,
                        uv,
                        texture,
                        alpha: model.face_alpha(i),
                        near_alpha: Some(model.alpha.as_ref().map_or(0, |a| a[i])),
                        tag: ((obj.id as u64 + 1) << 32)
                            | ((plane as u64) << 28)
                            | ((x as u64) << 14)
                            | y as u64,
                    });
                }
                #[cfg(feature="custom-assets")]
                if out.triangles.len()>batch && scene.authored_visual_placement(obj) {
                    out.authored_visual_batches.insert((out.triangles[batch].tag,batch));
                }
                out.object_instances += 1;
                if obj.shape<=3 && out.triangles.len()>batch {
                    if let Some(identity)=scene.boundary_part_identity(object_index,part) {
                        out.boundary_submissions.push(crate::object_submission::BoundarySubmission {identity,batch});
                    }
                }
                if (4..=8).contains(&obj.shape) && out.triangles.len()>batch {
                    if let Some(identity)=scene.decoration_part_identity(object_index,part) {
                        out.decoration_submissions.push(crate::object_submission::DecorationSubmission{identity,batch});
                    }
                }
                if obj.shape==22 && out.triangles.len()>batch {
                    if let Some(identity)=scene.ground_identity(object_index) {
                        out.ground_submissions.push(crate::object_submission::GroundSubmission{identity,batch});
                    }
                }
                if (9..=21).contains(&obj.shape) && out.triangles.len()>batch {
                    out.object_submissions.push(crate::object_submission::Submission {
                        traversal_id,
                        batch, object:obj.clone(), size:if matches!(obj.shape,10|11) {placement_size.map(usize::from)} else {[1,1]},
                    });
                }
                if let Some(sequence) = specification.sequence {
                    out.animated.push(AnimatedObject {
                        key,
                        id: obj.id,
                        part,
                        generation: specification.generation,
                        placement: retained.map_or_else(|| scene.location_generations.get(&key).copied().unwrap_or(0), |old| old.generation),
                        initial_tick: true,
                        random_start: specification.generation == 0 && randomize_animation,
                        model,
                        sequence: sequence as u32,
                        clock: Default::default(),
                        rotation,
                        scene_shape: obj.shape,
                        plane: height_plane,
                        centre: draw_center,
                        base,
                        offset,
                        contour: def.contour,
                        contour_origin,
                        triangles: animation_triangles,
                        #[cfg(feature = "private-test-adapter")]
                        model_ids: def.model_ids(shape),
                        #[cfg(feature = "private-test-adapter")]
                        randomized: false,
                        #[cfg(feature = "private-test-adapter")]
                        last: None,
                    });
                }
            }
        }
        #[cfg(feature = "custom-assets")]
        out.cutaway.finish(out.triangles.len())?;
        #[cfg(feature = "custom-assets")]
        {out.motion=scene.environment_motion(&out.triangles)?;}
        out.pending_assets = models.pending();
        Ok(out)
    }
    #[cfg(feature = "custom-assets")]
    pub fn environment_motion_active(&mut self,active:bool){self.motion.active(active);}
    #[cfg(feature = "custom-assets")]
    pub fn sample_environment_motion(&mut self,seconds:f64)->Result<()> {self.motion.pose_at(seconds,&mut self.triangles)}
    #[cfg(feature = "custom-assets")]
    pub fn environment_motion_faces(&self)->usize {self.motion.indices().count()}
    #[cfg(feature = "custom-assets")]
    pub fn set_environment_motion_mode(&mut self,mode:crate::environment_motion::Mode)->bool {
        self.motion.set_admitted_mode(mode)
    }
    pub fn animated_objects(&self) -> usize {
        self.animated.len()
    }
    /// Keep unaffected locations at their current frame across scene mesh rebuilds.
    pub fn retain_clocks(
        &mut self,
        previous: &Self,
        animations: &crate::animation_runtime::Animations,
    ) {
        #[cfg(feature = "custom-assets")]
        self.cutaway.retain(&previous.cutaway);
        #[cfg(feature = "custom-assets")]
        self.motion.retain(&previous.motion);
        for object in &mut self.animated {
            if let Some(old) = previous.animated.iter().find(|old| {
                old.key == object.key
                    && old.id == object.id
                    && old.placement == object.placement
                    && old.part == object.part
                    && old.sequence == object.sequence
            }) {
                let continuous = old.generation == object.generation
                    || animations
                        .sequences
                        .get(&object.sequence)
                        .is_some_and(|s| s.restart == 0);
                if continuous {
                    object.clock = old.clock.clone();
                    object.initial_tick = old.initial_tick;
                    object.random_start = old.random_start;
                }
            }
        }
    }
    pub fn advance(
        &mut self,
        scene: &Scene,
        cache: &mut DiskCache,
        animations: &mut crate::animation_runtime::Animations,
        ticks: u32,
    ) -> Result<()> {
        #[cfg(feature = "custom-assets")]
        self.motion.advance(&mut self.triangles)?;
        #[cfg(feature = "private-test-adapter")]
        self.observations.clear();
        for obj in &mut self.animated {
            if obj.clock.stopped {
                // The slots keep the pose written when the clock stopped.
                #[cfg(feature = "private-test-adapter")]
                self.observations.extend(obj.last.clone());
                continue;
            }
            if !animations.sequences.contains_key(&obj.sequence) {
                animations.sequences.insert(
                    obj.sequence,
                    crate::animation::Sequence::decode(obj.sequence, &[0])?,
                );
            }
            let sequence = &animations.sequences[&obj.sequence];
            if std::mem::take(&mut obj.random_start) && sequence.loop_frames != -1 {
                let mut bytes = [0; 16];
                getrandom::getrandom(&mut bytes).map_err(|e| Error::Io(e.to_string()))?;
                let draws = std::array::from_fn(|i| {
                    (u64::from_ne_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap()) >> 11) as f64
                        / 9007199254740992.0
                });
                obj.clock.randomize(sequence, draws)?;
                #[cfg(feature = "private-test-adapter")]
                {
                    obj.randomized = true;
                }
            }
            let ticks = if std::mem::take(&mut obj.initial_tick) {
                1
            } else {
                ticks
            };
            let (flags, _events) = obj.clock.advance(sequence, ticks)?;
            if flags & 9 == 9 {
                obj.clock.stopped = true;
            }
            let height_offset = crate::location_animation::height_offset(
                Some((if obj.clock.stopped { -1 } else { obj.sequence as i32 }, sequence.extra)),
                None,
            );
            let mut model = (*obj.model).clone();
            model.rotate_quarter(4 - (obj.rotation & 3));
            let _applied = if obj.clock.stopped {
                crate::animation_runtime::Applied::Stopped
            } else {
                animations.apply_observed(cache, &mut model, obj.sequence, &obj.clock)?
            };
            model.rotate_quarter(obj.rotation);
            let local_vertices = crate::model_contour::rendered(&model,obj.contour_origin,obj.contour,|x,z| scene.height(obj.plane,x,z))?;
            let position = [obj.centre[0]+obj.offset[0], obj.base.wrapping_sub(height_offset), obj.centre[1]+obj.offset[1]];
            let draw_bounds = crate::model_bounds::Bounds::new(&local_vertices, position).with_selection(&local_vertices, if obj.scene_shape == 11 { 256 } else { 0 }, false);
            let vertices = crate::model_draw::vertices(&local_vertices, if obj.scene_shape == 11 { 256 } else { 0 }, position)?;
            #[cfg(feature = "private-test-adapter")]
            let mut written = Vec::with_capacity(obj.triangles.len());
            for &(triangle, face_index) in &obj.triangles {
                let face = &model.faces[face_index];
                let target = &mut self.triangles[triangle];
                target.vertices = face.map(|v| vertices[v as usize]);
                target.model_bounds = Some(draw_bounds);
                target.local_vertices = Some(face.map(|v| local_vertices[v as usize]));
                #[cfg(feature = "private-test-adapter")]
                written.extend(target.local_vertices);
                #[cfg(feature="enhanced-renderer")] { target.material_hsl=Some(model.colors[face_index]); }
                target.alpha = model.face_alpha(face_index);
                if let Some(order) = &mut target.model_order {
                    *order = crate::model_order::Submission::new(&model, face_index, order.batch);
                    order.scene_height_offset = height_offset;
                }
                target.near_alpha = Some(model.alpha.as_ref().map_or(0, |a| a[face_index]));
                let mapping = model
                    .texture_coords
                    .as_ref()
                    .and_then(|t| {
                        if t[face_index] == -1 {
                            None
                        } else {
                            model.texture_triangles.get(t[face_index] as u8 as usize)
                        }
                    })
                    .unwrap_or(face);
                target.uv = texture_uv(mapping.map(|v| vertices[v as usize]), target.vertices);
                target.texture_mapping = Some(crate::render3d::TextureMapping { vertices: mapping.map(|v| vertices[v as usize]), local: Some(mapping.map(|v| local_vertices[v as usize])) });
            }
            // Record exactly what was just written, from the outcome actually applied.
            #[cfg(feature = "private-test-adapter")]
            {
                let sequence = &animations.sequences[&obj.sequence];
                let observation = crate::location_animation_receipt::Observation::record(
                    obj.target(),
                    obj.model_ids.clone(),
                    sequence,
                    sequence.frames.is_empty() && sequence.cached_animation.is_none(),
                    obj.randomized,
                    &obj.clock,
                    &_applied,
                    &local_vertices,
                    &written,
                    obj.triangles.first().map_or(0, |&(t, _)| self.triangles[t].tag),
                );
                obj.last = Some(observation.clone());
                self.observations.push(observation);
            }
        }
        Ok(())
    }
    /// Each animated part's target, scene tag and triangle slots.
    #[cfg(feature = "private-test-adapter")]
    pub fn animated_slots(
        &self,
    ) -> impl Iterator<Item = (crate::location_animation_receipt::Target, u64, Vec<usize>)> + '_ {
        self.animated.iter().map(|o| {
            let slots: Vec<usize> = o.triangles.iter().map(|&(t, _)| t).collect();
            (o.target(), slots.first().map_or(0, |&t| self.triangles[t].tag), slots)
        })
    }
    /// The latest advance's observations, declared targets first.
    #[cfg(feature = "private-test-adapter")]
    pub fn location_receipt(
        &self,
        declared: &[crate::location_animation_receipt::Target],
    ) -> crate::location_animation_receipt::Receipt {
        crate::location_animation_receipt::Receipt::collect(self.observations.clone(), declared, self.pending_assets)
    }
}
#[cfg(feature = "private-test-adapter")]
impl AnimatedObject {
    fn target(&self) -> crate::location_animation_receipt::Target {
        crate::location_animation_receipt::Target { key: self.key, location_id: self.id, part: self.part }
    }
}

/// A hand-built mesh around one animated location part, for receipt tests.
#[cfg(all(test, feature = "private-test-adapter"))]
pub(crate) mod test_support {
    use super::*;
    use crate::animation_runtime::test_assets::{model, SEQUENCE};
    use crate::location_animation_receipt::Target;
    pub const KEY: [i32; 4] = [0, 3205, 3206, 2];
    pub const ID: u32 = 51000;
    /// Scene-local tile (5, 6): the tag encodes these, as `build` does.
    pub const TILE: [u64; 2] = [5, 6];
    pub const CENTRE: [i32; 2] = [5 * 128 + 64, 6 * 128 + 64];
    /// The faces of `test_assets::model`, restated as literals.
    pub const FACES: [[usize; 3]; 12] = [
        [0, 1, 2], [0, 2, 3], [0, 3, 1], [1, 3, 2], [4, 5, 6], [4, 6, 7],
        [4, 7, 5], [5, 7, 6], [0, 4, 1], [1, 4, 5], [2, 6, 3], [3, 6, 7],
    ];
    pub fn tag() -> u64 {
        ((ID as u64 + 1) << 32) | (TILE[0] << 14) | TILE[1]
    }
    pub fn target(part: usize) -> Target {
        Target { key: KEY, location_id: ID, part }
    }
    fn placeholder(tag: u64) -> Triangle {
        let mut t = Triangle::selection_only(crate::model_bounds::Bounds::new(&[[0.0; 3]], [0; 3]), tag);
        t.model_bounds = None;
        t.colors = [100, 200, 300];
        t
    }
    fn object(part: usize, sequence: u32, slots: Vec<(usize, usize)>) -> AnimatedObject {
        AnimatedObject {
            key: KEY,
            id: ID,
            part,
            generation: 0,
            placement: 0,
            initial_tick: false,
            random_start: false,
            model: Arc::new(model()),
            sequence,
            clock: Default::default(),
            rotation: 0,
            scene_shape: 10,
            plane: 0,
            centre: CENTRE,
            base: 0,
            offset: [0, 0],
            contour: -1,
            contour_origin: [CENTRE[0], 0, CENTRE[1]],
            triangles: slots,
            model_ids: vec![60000],
            randomized: false,
            last: None,
        }
    }
    fn empty() -> WorldMesh {
        WorldMesh {
            authored_visual_batches: Default::default(),
            cutaway: Default::default(),
            motion: crate::environment_motion::Motion::empty(),
            triangles: Vec::new(),
            terrain_submissions: Vec::new(),
            object_submissions: Vec::new(),
            boundary_submissions: Vec::new(),
            decoration_submissions: Vec::new(),
            ground_submissions: Vec::new(),
            minimum_planes: Vec::new(),
            object_instances: 0,
            pending_assets: false,
            item_support: HashMap::new(),
            animated: Vec::new(),
            transform_varps: HashMap::new(),
            observations: Vec::new(),
        }
    }
    /// One animated part on render plane `plane`, its 12 faces in slots 0..12.
    pub fn one(plane: u8, sequence: u32) -> WorldMesh {
        let mut mesh = empty();
        for _ in FACES {
            mesh.triangles.push(placeholder(tag()));
            mesh.minimum_planes.push(plane);
        }
        mesh.animated.push(object(0, sequence, (0..12).map(|i| (i, i)).collect()));
        mesh
    }
    pub fn standard() -> WorldMesh {
        one(0, SEQUENCE)
    }
    /// A static triangle carrying the location's tag, outside its slots.
    pub fn add_same_tag_static(mesh: &mut WorldMesh) {
        mesh.triangles.push(placeholder(tag()));
        mesh.minimum_planes.push(0);
    }
    /// A second part whose first face reuses part 0's slot 0.
    pub fn add_overlapping_part(mesh: &mut WorldMesh) {
        let start = mesh.triangles.len();
        let mut slots = vec![(0, 0)];
        for i in 1..12 {
            mesh.triangles.push(placeholder(tag()));
            mesh.minimum_planes.push(0);
            slots.push((start + i - 1, i));
        }
        mesh.animated.push(object(1, SEQUENCE, slots));
    }
    /// A part added after the last advance: slots, but no observation yet.
    pub fn add_unadvanced_part(mesh: &mut WorldMesh) {
        let start = mesh.triangles.len();
        for _ in FACES {
            mesh.triangles.push(placeholder(tag()));
            mesh.minimum_planes.push(0);
        }
        mesh.animated.push(object(2, SEQUENCE, (0..12).map(|i| (start + i, i)).collect()));
    }
    /// `count` other single-face locations, each with its own id and tag.
    pub fn add_other_locations(mesh: &mut WorldMesh, count: usize) {
        for i in 0..count {
            let id = ID + 1 + i as u32;
            let slot = mesh.triangles.len();
            mesh.triangles.push(placeholder(((id as u64 + 1) << 32) | (TILE[0] << 14) | TILE[1]));
            mesh.minimum_planes.push(0);
            let mut other = object(0, SEQUENCE, vec![(slot, 0)]);
            other.id = id;
            other.key = [KEY[0], KEY[1] + 1 + i as i32, KEY[2], KEY[3]];
            mesh.animated.push(other);
        }
    }
}

#[cfg(all(test, feature = "private-test-adapter"))]
mod receipt_tests {
    use super::test_support::*;
    use crate::animation_runtime::test_assets::*;
    use crate::location_animation_receipt::{pose_digest, slots_digest, Outcome};
    fn literal_slots(pose: &[[i32; 3]; 8]) -> Vec<[[f32; 3]; 3]> {
        FACES.iter().map(|f| f.map(|v| pose[v].map(|c| c as f32))).collect()
    }
    fn literal_pose(pose: &[[i32; 3]; 8]) -> Vec<[f32; 3]> {
        pose.iter().map(|v| v.map(|c| c as f32)).collect()
    }
    #[test]
    fn advance_records_the_exact_pose_and_slots_it_writes() {
        assert_eq!(model().faces, FACES.map(|f| f.map(|v| v as u16)).to_vec());
        let mut cache = EmptyCache::new("mesh-advance");
        let mut animations = animations(sequence(LENGTHS), [1, 1], PIVOT);
        let scene = crate::scene::Scene::empty_sized(3200, 3200, 16, 16).unwrap();
        let mut mesh = standard();
        // Cumulative ticks 0/50/51/125/126/225 across lengths 50/75/100.
        for (ticks, frame, elapsed) in [(0, 0, 0), (50, 0, 50), (1, 1, 1), (74, 1, 75), (1, 2, 1), (99, 2, 100)] {
            mesh.advance(&scene, &mut cache.1, &mut animations, ticks).unwrap();
            assert_eq!(mesh.observations.len(), 1);
            let o = &mesh.observations[0];
            assert_eq!(o.target, target(0));
            assert_eq!(o.outcome, Outcome::Classic { frame_index: frame, frame_id: FRAMES[frame], skeleton_id: SKELETON });
            assert_eq!((o.clock.frame, o.clock.elapsed, o.clock.loops, o.clock.stopped), (frame, elapsed, 0, false));
            assert_eq!(o.frame_length, Some(LENGTHS[frame] as u32));
            assert_eq!(o.pose_digest, pose_digest(&literal_pose(&POSES[frame])), "frame {frame}");
            assert_eq!(o.slots_digest, slots_digest(&literal_slots(&POSES[frame])), "frame {frame}");
            // The digest describes what the slots hold now.
            let held: Vec<_> = mesh.triangles[..12].iter().map(|t| t.local_vertices.unwrap()).collect();
            assert_eq!(held, literal_slots(&POSES[frame]));
            assert_eq!((o.tag, o.triangles, o.model_ids.clone()), (tag(), 12, vec![60000]));
            assert!(!o.sequence_placeholder && !o.random_start);
        }
        // Past 225 the non-looping sequence stops; the unanimated pose is written
        // and reported as stopped, then re-reported while the clock stays stopped.
        for _ in 0..2 {
            mesh.advance(&scene, &mut cache.1, &mut animations, 1).unwrap();
            let o = &mesh.observations[0];
            assert_eq!((o.outcome.clone(), o.clock.stopped), (Outcome::Stopped, true));
            assert_eq!(o.pose_digest, pose_digest(&literal_pose(&BASE)));
            assert_eq!(o.slots_digest, slots_digest(&literal_slots(&BASE)));
        }
    }
    #[test]
    fn an_absent_sequence_is_recorded_as_a_placeholder() {
        let mut cache = EmptyCache::new("mesh-placeholder");
        let mut animations = animations(sequence(LENGTHS), [1, 1], PIVOT);
        let scene = crate::scene::Scene::empty_sized(3200, 3200, 16, 16).unwrap();
        let mut mesh = one(0, SEQUENCE + 9);
        mesh.advance(&scene, &mut cache.1, &mut animations, 1).unwrap();
        let o = &mesh.observations[0];
        assert!(o.sequence_placeholder);
        assert_eq!(o.outcome, Outcome::Stopped);
    }
}
