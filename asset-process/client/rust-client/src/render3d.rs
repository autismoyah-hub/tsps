//! Native CPU triangle pipeline: near clipping, perspective textures and depth.
use crate::{raster::Raster, texture::Texture, Error, Result};
use std::collections::HashMap;

/// Original ee.ap mapped depth, interpolated by the software rasterizer.
pub fn raster_depth(z: f32) -> f32 {
    let z = 1.0100503f32 * (z - 75.0);
    150.75377f32 / z + 1.0100503f32
}
/// fy.br tests the sign bit of the subtraction, including IEEE edge cases.
pub fn depth_pass(current: f32, incoming: f32) -> bool {
    (current - incoming).is_sign_negative()
}

/// Original GameScene.av containment and GameScriptEventBuilder.ab depth.
pub fn terrain_pick(point: [i32; 2], vertices: [[i32; 2]; 3], depth: [f32; 3]) -> Option<f32> {
    let [x, y] = point;
    if (0..2).any(|axis| {
        vertices.iter().all(|v| point[axis] < v[axis])
            || vertices.iter().all(|v| point[axis] > v[axis])
    }) {
        return None;
    }
    let edge = |a: [i32; 2], b: [i32; 2]| {
        y.wrapping_sub(a[1])
            .wrapping_mul(b[0].wrapping_sub(a[0]))
            .wrapping_sub(x.wrapping_sub(a[0]).wrapping_mul(b[1].wrapping_sub(a[1])))
    };
    let [a, b, c] = vertices;
    let [e0, e1, e2] = [edge(a, b), edge(b, c), edge(c, a)];
    let inside = if e0 == 0 {
        if e1 == 0 {
            true
        } else if e1 < 0 {
            e2 <= 0
        } else {
            e2 >= 0
        }
    } else if e0 < 0 {
        e1 <= 0 && e2 <= 0
    } else {
        e1 >= 0 && e2 >= 0
    };
    if !inside {
        return None;
    }
    let d = |a: i32, b: i32| a.wrapping_sub(b);
    let sum =
        |a: i32, b: i32, c: i32, e: i32| a.wrapping_mul(b).wrapping_add(c.wrapping_mul(e)) as f32;
    let denominator = sum(d(a[1], c[1]), d(c[0], b[0]), d(b[1], c[1]), d(a[0], c[0]));
    let u = sum(d(c[0], b[0]), d(y, c[1]), d(b[1], c[1]), d(x, c[0])) / denominator;
    let v = sum(d(x, c[0]), d(c[1], a[1]), d(a[0], c[0]), d(y, c[1])) / denominator;
    Some(depth[1] * v + depth[0] * u + (1.0 - u - v) * depth[2])
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalize(v: [f32; 3]) -> Result<[f32; 3]> {
    let length = dot(v, v).sqrt();
    if !length.is_finite() || length < 0.00001 {
        return Err(Error::Invalid("camera direction"));
    }
    Ok(v.map(|a| a / length))
}
pub struct Camera {
    pub position: [f32; 3],
    right: [f32; 3],
    up: [f32; 3],
    forward: [f32; 3],
    angle_trig: Option<[f32; 4]>,
    fixed_trig: Option<[i32; 4]>,
    pub focal: f32,
    pub near: f32,
    pub far: i32,
    pub screen_offset: [f32; 2],
    /// Fixed projection divisor for orthographic interface models.
    pub orthographic: Option<f32>,
}
impl Camera {
    /// GameScene.dg clamps the root projection without changing the global eye
    /// used by child-scene matrices or camera tracking.
    pub fn root_scene(mut position: [i32; 3], pitch: i32, yaw: i32, focal: f32, size: [usize; 2]) -> Result<Self> {
        for (dimension, axis) in [0, 2].into_iter().enumerate() {
            let extent = size[dimension].checked_mul(128)
                .filter(|&n| n > 0).and_then(|n| i32::try_from(n).ok())
                .ok_or(Error::Invalid("root scene dimensions"))?;
            position[axis] = position[axis].clamp(0, extent - 1);
        }
        Self::angles(position, pitch, yaw, focal)
    }

    pub fn angles(position: [i32; 3], pitch: i32, yaw: i32, focal: f32) -> Result<Self> {
        if !focal.is_finite() || focal <= 0.0 {
            return Err(Error::Invalid("camera focal length"));
        }
        let radians = |angle: i32| (angle & 16383) as f64 * 3.834951969714103E-4;
        let (sp, cp) = (radians(pitch).sin() as f32, radians(pitch).cos() as f32);
        let (sy, cy) = (radians(yaw).sin() as f32, radians(yaw).cos() as f32);
        Ok(Self {
            position: position.map(|v| v as f32),
            right: [cy, 0.0, sy],
            up: [-sy * sp, -cp, cy * sp],
            forward: [-sy * cp, sp, cy * cp],
            angle_trig: Some([sp, cp, sy, cy]),
            fixed_trig: Some(crate::model_bounds::trig(pitch, yaw)),
            focal,
            near: 50.0,
            far: 3500,
            screen_offset: [0.0; 2],
            orthographic: None,
        })
    }
    pub fn look_at(position: [f32; 3], target: [f32; 3], focal: f32) -> Result<Self> {
        if !focal.is_finite() || focal <= 0.0 {
            return Err(Error::Invalid("camera focal length"));
        }
        let forward = normalize(sub(target, position))?;
        let right = normalize(cross(forward, [0.0, -1.0, 0.0]))?;
        let up = cross(right, forward);
        Ok(Self {
            position,
            right,
            up,
            forward,
            angle_trig: None,
            fixed_trig: None,
            focal,
            near: 50.0,
            far: 3500,
            screen_offset: [0.0; 2],
            orthographic: None,
        })
    }
    pub fn project(&self, position: [f32; 3], size: [usize; 2]) -> Option<[i32; 2]> {
        let v = self.transform(position);
        if v[2] < self.near {
            return None;
        }
        Some(self.screen(v, size).map(|v| v as i32))
    }
    /// GameIntProjection / GameFloatProjection reject an entire terrain tile
    /// if any of its vertices crosses the near plane, including hidden faces.
    pub fn terrain_visible(&self, vertices: impl IntoIterator<Item = [f32; 3]>) -> bool {
        vertices.into_iter().all(|v| self.transform(v)[2] >= self.near)
    }
    fn model_frustum(&self, size: [usize; 2]) -> Option<crate::model_bounds::Frustum> {
        if self.orthographic.is_some() || self.near != 50.0 { return None; }
        let [x, y] = self.center(size).map(|v| v as i32);
        Some(crate::model_bounds::Frustum {
            eye: self.position.map(|v| v as i32), trig: self.fixed_trig?, focal: self.focal as i32, far: self.far,
            clip: [-x, size[0] as i32 - x, -y, size[1] as i32 - y],
        })
    }
    fn center(&self, size: [usize; 2]) -> [f32; 2] {
        std::array::from_fn(|i| (size[i] / 2) as f32 + self.screen_offset[i])
    }
    fn screen(&self, v: [f32; 3], size: [usize; 2]) -> [f32; 2] {
        let divisor = self.orthographic.unwrap_or(v[2]);
        let center = self.center(size);
        std::array::from_fn(|i| center[i] + v[i] * self.focal / divisor)
    }
    /// Reconstruct root-world presentation coordinates after Classic's child
    /// transforms and near clipping. Picking continues to use the original data.
    #[cfg(feature = "enhanced-renderer")]
    pub fn presentation_world(&self, p: [f32; 3]) -> [f32; 3] {
        let v = if let Some([sp, cp, sy, cy]) = self.angle_trig {
            let y = p[1] * cp + p[2] * sp;
            let z = p[2] * cp - p[1] * sp;
            [p[0] * cy - z * sy, y, z * cy + p[0] * sy]
        } else {
            std::array::from_fn(|i| self.right[i]*p[0] - self.up[i]*p[1] + self.forward[i]*p[2])
        };
        std::array::from_fn(|i| v[i] + self.position[i])
    }
    /// Camera-space coordinates. Angle-based world cameras preserve the
    /// original yaw-then-pitch operation order instead of combining matrices.
    pub fn transform(&self, v: [f32; 3]) -> [f32; 3] {
        let v = sub(v, self.position);
        if let Some([sp, cp, sy, cy]) = self.angle_trig {
            let [x, y, z] = v;
            let rx = z * sy + x * cy;
            let rz = z * cy - x * sy;
            [rx, y * cp - rz * sp, y * sp + rz * cp]
        } else {
            [dot(v, self.right), -dot(v, self.up), dot(v, self.forward)]
        }
    }
}
#[derive(Clone, Copy)]
pub struct Triangle {
    /// Optional unlit per-face material; Classic raster paths never read this.
    #[cfg(feature="enhanced-renderer")]
    pub material_hsl: Option<u16>,
    pub texture_mapping: Option<TextureMapping>,
    /// Encoded face alpha restored by GameModel.gt after near clipping.
    pub near_alpha: Option<u8>,
    /// Unplaced model or terrain vertices for original child matrix projection.
    pub local_vertices: Option<[[f32; 3]; 3]>,
    /// Bounds of the original renderable, before draw orientation/translation.
    pub model_bounds: Option<crate::model_bounds::Bounds>,
    /// Signed model-face offset applied only to mapped depth after clipping.
    pub depth_bias: i16,
    pub model_order: Option<crate::model_order::Submission>,
    pub vertices: [[f32; 3]; 3],
    pub colors: [i32; 3],
    /// Preserve the original flat-face path when applying scene overrides.
    pub flat: bool,
    pub uv: [[f32; 2]; 3],
    pub texture: Option<u16>,
    pub alpha: u8,
    pub tag: u64,
    /// Owning scene, independent of the native geometry tag's actor/object ID.
    pub view: u16,
}
#[derive(Clone, Copy, Debug)]
pub struct TextureMapping {
    pub vertices: [[f32;3];3],
    pub local: Option<[[f32;3];3]>,
}
impl Triangle {
    /// Preserve a single-tile model's picking bounds without raster geometry.
    pub fn selection_only(bounds: crate::model_bounds::Bounds, tag: u64) -> Self {
        Self {
            #[cfg(feature="enhanced-renderer")] material_hsl:None,
            texture_mapping: None,
            near_alpha: None,
            local_vertices: None, model_bounds: Some(bounds), model_order: None, depth_bias: 0,
            vertices: [bounds.position.map(|v| v as f32); 3], colors: [0; 3],
            flat: true, uv: [[0.; 2]; 3], texture: None, alpha: 255, tag, view: 0,
        }
    }

    /// fv.al/aq apply scene HSL overrides to untextured Gouraud/flat faces.
    pub fn scene_tint(&mut self, color: u16, palette: &[u32]) {
        if self.texture.is_some() {
            return;
        }
        let tint = crate::actor_visual::Tint {
            hsl: [
                (color >> 10 & 63) as u8,
                (color >> 7 & 7) as u8,
                (color & 127) as u8,
            ],
            weight: 127,
            ..crate::actor_visual::Tint::default()
        };
        #[cfg(feature="enhanced-renderer")]
        { self.material_hsl=self.material_hsl.map(|c|tint.color(c as i32) as u16); }
        self.colors = if self.flat {
            [tint.color(palette[self.colors[0] as usize] as i32); 3]
        } else {
            self.colors.map(|c| tint.color(c))
        };
    }
}
// Model faces are contiguous in scene/actor submissions. Cache the original
// renderable decision across their faces, then retain it through alpha sorting.
fn model_visible(t: &Triangle, frustum: Option<&crate::model_bounds::Frustum>,
                 last: &mut Option<(crate::model_bounds::Bounds, bool)>) -> bool {
    // Child views use GameModel.mh's matrix path, which has no by-style cull.
    if t.view != 0 { return true; }
    let (Some(bounds), Some(frustum)) = (t.model_bounds, frustum) else { return true; };
    if let Some((previous, visible)) = last {
        if *previous == bounds { return *visible; }
    }
    let visible = frustum.visible(bounds);
    *last = Some((bounds, visible));
    visible
}
// Submission-local memo only: exact identity, no reordering or cross-frame reuse.
#[inline]
fn cached_value<K:Copy+Eq,V:Copy>(last:&mut Option<(K,V)>,key:K,compute:impl FnOnce()->V)->V {
    if let Some((previous,value))=*last {if previous==key {return value;}}
    let value=compute();*last=Some((key,value));value
}
#[cfg(test)]
mod submission_cache_tests {
    use super::cached_value;
    #[test]
    fn exact_identity_and_interleaving_preserve_first_insertion_indices() {
        let mut last=None;let mut map=std::collections::HashMap::new();let mut computes=0;
        let keys=[(0u16,10u64,0usize),(1,10,0),(0,11,0),(0,10,1)];
        let sequence=[0,0,1,1,0,2,2,3,0];
        let mut indices=Vec::new();
        for i in sequence {
            let key=keys[i];
            for _ in 0..1000 {
                let index=cached_value(&mut last,key,|| {computes+=1;let next=map.len();*map.entry(key).or_insert(next)});
                assert_eq!(index,i);
            }
            indices.push(map[&key]);
        }
        assert_eq!(indices,sequence);
        assert_eq!(computes,6,"one computation per exact-identity run, including returning groups");
        assert_eq!(map.len(),4);
        let mut other=None;let mut calls=0;
        assert!(!cached_value(&mut other,keys[0],||{calls+=1;false}));
        assert!(!cached_value(&mut other,keys[0],||{calls+=1;true}));
        assert_eq!(calls,1,"negative authored admission is also memoized locally");
    }
}
// Keep borrowed geometry through scheduling; only admitted wide art needs a
// temporary metadata-adjusted value at the point of rasterization.
#[derive(Clone, Copy)]
struct DrawFace<'a> {
    source: &'a Triangle,
    #[cfg(feature="custom-assets")]
    wide: bool,
}
impl<'a> DrawFace<'a> {
    fn plain(source:&'a Triangle)->Self {
        Self {source, #[cfg(feature="custom-assets")] wide:false}
    }
    fn is_wide(self)->bool {
        #[cfg(feature="custom-assets")]
        {return self.wide;}
        #[cfg(not(feature="custom-assets"))]
        {false}
    }
    fn with<R>(self, use_face:impl FnOnce(&Triangle)->R)->R {
        #[cfg(feature="custom-assets")]
        if self.wide {
            return use_face(&Triangle {model_bounds:None,model_order:None,..*self.source});
        }
        use_face(self.source)
    }
}
fn flush_terrain<'a>(faces: &mut Vec<DrawFace<'a>>, visible: bool, emit: &mut impl FnMut(DrawFace<'a>)) {
    for face in faces.drain(..) {
        if visible { emit(face); }
    }
}
#[derive(Clone, Copy)]
struct Vertex {
    // Preserve exact root geometry for GPU retention. Clipped intersections and
    // child matrix projections must instead reconstruct their camera-space result.
    #[cfg(feature="enhanced-renderer")]
    presentation_world: Option<[f32; 3]>,
    screen: Option<[f32; 2]>,
    // GameModel.gt forwards raw fourth-vertex depth on the second clipped triangle.
    mapped_depth: Option<f32>,
    p: [f32; 3],
    color: f32,
    uv: [f32; 2],
}
impl Vertex {
    fn lerp(a: Self, b: Self, t: f32) -> Self {
        Self {
            #[cfg(feature="enhanced-renderer")]
            presentation_world: None,
            screen: None,
            mapped_depth: None,
            p: std::array::from_fn(|i| a.p[i] + (b.p[i] - a.p[i]) * t),
            color: a.color + (b.color - a.color) * t,
            uv: std::array::from_fn(|i| a.uv[i] + (b.uv[i] - a.uv[i]) * t),
        }
    }
}
struct ModelRegion {
    view: u16,
    tag: u64,
    bounds: crate::model_bounds::Bounds,
    faces: Vec<([i32; 4], i32)>,
}
/// Root scene invocations may enter a prepared child model stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelSceneCall { Model(u64,usize), Child(u16) }
/// A fully expanded first-pass invocation in a scene namespace.
pub use crate::scene_raster_order::Primitive as SceneRasterCall;
/// Opt-in timings around existing submission work; no per-face clock reads.
#[cfg(feature="native-profiling")]
#[derive(Clone, Copy, Debug, Default)]
pub struct SubmissionProfile {
    /// Authored expansion, collection/direct draws, opaque schedule/draw, alpha schedule/draw.
    pub stage_ms: [f64;4],
    pub model_groups: usize,
    pub terrain_groups: usize,
}
pub struct Frame {
    #[cfg(feature="native-profiling")]
    submission_profile: Option<SubmissionProfile>,
    #[cfg(feature="custom-assets")]
    authored_visual_batches: std::collections::HashSet<(u64,usize)>,
    #[cfg(feature="enhanced-renderer")]
    presentation_geometry: Option<Vec<Triangle>>,
    #[cfg(feature="enhanced-renderer")]
    presentation_overflow: bool,
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    depth: Vec<f32>,
    pub tags: Vec<u64>,
    pub views: Vec<u16>,
    /// Terrain hit testing is independent of actors and objects covering it.
    pub ground_tags: Vec<u64>,
    pub ground_views: Vec<u16>,
    ground_depth: Vec<f32>,
    ground_faces: Vec<(u16, u64, [[i32; 2]; 3], [f32; 3])>,
    model_regions: Vec<ModelRegion>,
    model_admissions: HashMap<(u16,u64,usize),[bool;2]>,
    model_opaque_orders: HashMap<(u16,u64,usize),crate::object_scene_order::Key>,
    model_later_orders: HashMap<u16,(std::collections::HashSet<(u64,usize)>,Vec<(u64,usize)>)>,
    model_first_orders: HashMap<u16,(std::collections::HashSet<(u64,usize)>,Vec<(u64,usize)>)>,
    root_model_calls: Option<(std::collections::HashSet<u16>,[Vec<ModelSceneCall>;2])>,
    child_projections: HashMap<u16, (crate::skeletal::Matrix, crate::skeletal::Matrix)>,
    child_eyes: HashMap<u16,[i32;3]>,
    scene_raster_calls: std::collections::BTreeMap<u16,crate::scene_raster_order::Program>,
    temporary_sources: HashMap<(u16,u64),usize>,
    temporary_models: HashMap<(u16,u64,usize),(usize,bool)>,
    last_model_matrix: Option<(u16, crate::model_bounds::Bounds, crate::skeletal::Matrix)>,
    pub triangles: usize,
    /// Original fc.az: world/widget shading uses groups of four; item icons use individual pixels.
    pub coarse_shading: bool,
    legacy_raster: bool,
    legacy_icon: bool,
    legacy_origin: i32,
    legacy_aborted: bool,
    #[cfg(test)]
    force_polygon: bool,
    write_color: bool,
}
impl Frame {
    #[cfg(feature="native-profiling")]
    pub fn begin_submission_profile(&mut self) { self.submission_profile=Some(SubmissionProfile::default()); }
    #[cfg(feature="native-profiling")]
    pub fn submission_profile(&self)->Option<SubmissionProfile> { self.submission_profile }
    #[cfg(feature="native-profiling")]
    fn mark_submission(&mut self, start:&mut Option<std::time::Instant>, stage:usize) {
        if let (Some(start),Some(profile))=(start.as_mut(),self.submission_profile.as_mut()) {
            let now=std::time::Instant::now();
            profile.stage_ms[stage]+=now.duration_since(*start).as_secs_f64()*1000.;
            *start=now;
        }
    }
    #[cfg(feature="custom-assets")]
    pub(crate) fn register_authored_art(&mut self,batches:impl IntoIterator<Item=(u64,usize)>) {
        self.authored_visual_batches.extend(batches);
    }
    /// Opt-in observation of accepted color triangles. No Classic state is replaced.
    #[cfg(feature="enhanced-renderer")]
    pub fn begin_presentation_capture(&mut self) {
        self.presentation_geometry = Some(Vec::new());
        self.presentation_overflow = false;
    }
    #[cfg(feature="enhanced-renderer")]
    pub fn take_presentation_geometry(&mut self) -> Result<Option<Vec<Triangle>>> {
        let geometry = self.presentation_geometry.take();
        if std::mem::take(&mut self.presentation_overflow) {
            return Err(Error::Invalid("Enhanced presentation triangle budget"));
        }
        Ok(geometry)
    }
    pub fn new(width: usize, height: usize) -> Result<Self> {
        let count = width
            .checked_mul(height)
            .ok_or(Error::Invalid("3D framebuffer size"))?;
        if count > 16 << 20 || width == 0 || height == 0 {
            return Err(Error::Invalid("3D framebuffer size"));
        }
        Ok(Self {
            #[cfg(feature="custom-assets")]
            authored_visual_batches: Default::default(),
            #[cfg(feature="enhanced-renderer")]
            presentation_geometry: None,
            #[cfg(feature="enhanced-renderer")]
            presentation_overflow: false,
            width,
            height,
            pixels: vec![0xff10151a; count],
            depth: vec![0.0; count],
            tags: vec![0; count],
            views: vec![0; count],
            ground_tags: vec![0; count],
            ground_views: vec![0; count],
            ground_depth: vec![0.0; count],
            ground_faces: Vec::new(),
            model_regions: Vec::new(),
            model_admissions: HashMap::new(),
            model_opaque_orders: HashMap::new(),
            model_later_orders: HashMap::new(),
            model_first_orders: HashMap::new(),
            root_model_calls: None,
            scene_raster_calls: Default::default(),
            child_projections: HashMap::new(),
            child_eyes: HashMap::new(),
            temporary_sources: HashMap::new(),
            temporary_models: HashMap::new(),
            last_model_matrix: None,
            triangles: 0,
            coarse_shading: true,
            legacy_raster: false,
            legacy_icon: false,
            legacy_origin: 0,
            legacy_aborted: false,
            #[cfg(test)]
            force_polygon: false,
            write_color: true,
            #[cfg(feature="native-profiling")]
            submission_profile: None,
        })
    }
    pub fn clear(&mut self, color: u32) {
        #[cfg(feature="custom-assets")]
        self.authored_visual_batches.clear();
        #[cfg(feature="enhanced-renderer")]
        { if let Some(v)=&mut self.presentation_geometry {v.clear();} self.presentation_overflow=false; }
        self.pixels.fill(color);
        self.depth.fill(0.0);
        self.tags.fill(0);
        self.views.fill(0);
        self.ground_tags.fill(0);
        self.ground_views.fill(0);
        self.ground_depth.fill(0.0);
        self.ground_faces.clear();
        self.model_regions.clear();
        self.model_admissions.clear();
        self.model_opaque_orders.clear();
        self.model_later_orders.clear();
        self.model_first_orders.clear();
        self.root_model_calls=None;
        self.scene_raster_calls.clear();
        self.child_projections.clear();
        self.child_eyes.clear();
        self.temporary_models.clear();
        self.temporary_sources.clear();
        self.last_model_matrix = None;
        self.triangles = 0;
    }
    /// Retain terrain hits behind other scenes for per-input-mode selection.
    pub fn terrain_hits(&self, point: [i32; 2]) -> HashMap<u16, (u64, f32)> {
        self.terrain_hits_where(point, |_, _| true)
    }
    /// Reject ineligible surfaces before choosing the nearest hit in each view.
    pub fn terrain_hits_where(&self, point: [i32; 2], mut eligible: impl FnMut(u16, u64) -> bool) -> HashMap<u16, (u64, f32)> {
        let mut hits: HashMap<u16, (u64, f32)> = HashMap::new();
        if point[0] < 0
            || point[1] < 0
            || point[0] >= self.width as i32
            || point[1] >= self.height as i32
        {
            return hits;
        }
        for &(view, tag, vertices, depths) in &self.ground_faces {
            if let Some(depth) = terrain_pick(point, vertices, depths) {
                if !eligible(view, tag) { continue; }
                let hit = hits.entry(view).or_insert((0, -1.0));
                if depth > hit.1 {
                    *hit = (tag, depth);
                }
            }
        }
        hits
    }
    pub fn raster(&self) -> Result<Raster> {
        Raster::from_pixels(self.width, self.height, self.pixels.clone())
    }
    /// Register the original local-scene to camera projection once per view.
    pub fn child_projection(&mut self, view: u16, world: crate::skeletal::Matrix, eye: [i32;3], angles: [i32;2]) {
        self.child_eyes.insert(view,crate::matrix_pick::transform(&crate::matrix_pick::inverse(world),eye.map(|v|v as f32)).map(|v|v as i32));
        let projection = crate::matrix_pick::then(world, crate::matrix_pick::camera(eye, angles));
        self.child_projections.insert(view, (projection, crate::matrix_pick::inverse(projection)));
        self.last_model_matrix = None;
    }
    /// Original child-scene camera before camera orientation is composed.
    pub fn child_eye(&self,view:u16)->Option<[i32;3]> {self.child_eyes.get(&view).copied()}
    /// Original temporary-object zone rank and local-player identity.
    /// Call after rebasing model batches; static model slots remain unchanged.
    pub fn temporary_model(&mut self,view:u16,tag:u64,batch:usize,zone_rank:usize,local:bool) {
        self.temporary_models.insert((view,tag,batch),(zone_rank,local));
    }
    pub fn temporary_source(&mut self,view:u16,tag:u64,ordinal:usize){self.temporary_sources.entry((view,tag)).or_insert(ordinal);}
    fn camera_positions(&mut self, camera: &Camera, t: &Triangle) -> [[f32;3];3] {
        if let (Some(local), Some(&(projection, _))) = (t.local_vertices, self.child_projections.get(&t.view)) {
            if let Some(bounds) = t.model_bounds {
                let matrix = match self.last_model_matrix {
                    Some((view, previous, matrix)) if view == t.view && previous == bounds => matrix,
                    _ => {
                        let matrix = crate::matrix_pick::model(bounds.angle, bounds.position, projection);
                        self.last_model_matrix = Some((t.view, bounds, matrix));
                        matrix
                    }
                };
                return local.map(|v| crate::matrix_pick::transform(&matrix, v.map(|v| v as i32 as f32)));
            }
            return local.map(|v| crate::matrix_pick::transform(&projection, v));
        }
        t.vertices.map(|v| camera.transform(v))
    }
    pub fn has_model_selection(&self, view: u16, tag: u64) -> bool {
        self.model_regions.iter().any(|r| r.view == view && r.tag == tag)
    }
    pub fn model_hits(&self, point: [i32; 2], ray: crate::viewport_steering::Ray) -> Vec<(u64, i32)> {
        self.scene_model_hits(point, ray).into_iter().filter_map(|(tag,view,depth)| (view == 0).then_some((tag,depth))).collect()
    }
    /// Original model selection is independent of depth-buffer visibility.
    pub fn scene_model_hits(&self, point: [i32; 2], ray: crate::viewport_steering::Ray) -> Vec<(u64, u16, i32)> {
        if point[0] < 0 || point[1] < 0 || point[0] >= self.width as i32 || point[1] >= self.height as i32 {
            return Vec::new();
        }
        let Some(root_endpoints) = ray.endpoints(point) else { return Vec::new(); };
        let mut rays = HashMap::from([(0, (ray.eye, root_endpoints))]);
        for (&view, &(_, inverse)) in &self.child_projections {
            if let Some(child_ray) = crate::matrix_pick::ray_with_far(&inverse, ray.size, ray.focal, point, ray.far) { rays.insert(view, child_ray); }
        }
        let mut hits = crate::model_pick::Hits::default();
        for region in &self.model_regions {
            let Some(&(eye, endpoints)) = rays.get(&region.view) else { continue; };
            let shape = region.bounds.selection.unwrap();
            let bounds = shape.bounds();
            let position = std::array::from_fn(|i| region.bounds.position[i].wrapping_sub(eye[i]));
            if !bounds.intersects(position, endpoints) { continue; }
            if shape.single_tile {
                hits.push((region.view, region.tag), bounds.depth(position, endpoints).wrapping_add(32));
            } else {
                for &(rect, depth) in &region.faces {
                    if point[0].wrapping_add(5) >= rect[0] && point[0].wrapping_sub(5) <= rect[1]
                        && point[1].wrapping_add(5) >= rect[2] && point[1].wrapping_sub(5) <= rect[3] {
                        hits.push((region.view, region.tag), depth);
                    }
                }
            }
        }
        hits.sorted().into_iter().map(|((view,tag),depth)| (tag,view,depth)).collect()
    }
    /// Register traversal admission for an ordinary root scene object model.
    /// The second pass still obeys the model's alpha-array gate.
    pub fn model_admission(&mut self, view:u16, tag:u64, batch:usize, phases:[bool;2]) {
        self.model_admissions.insert((view,tag,batch),phases);
    }
    /// Rank ordinary scene objects inside their view's first-pass model slots.
    pub fn model_opaque_order(&mut self,view:u16,tag:u64,batch:usize,key:crate::object_scene_order::Key) {
        self.model_opaque_orders.insert((view,tag,batch),key);
    }
    /// Replace the owned models' later-pass invocations, including repetitions.
    /// Other groups keep their existing slots; full scene interleaving is separate.
    pub fn model_later_order(&mut self,view:u16,members:impl IntoIterator<Item=(u64,usize)>,calls:Vec<(u64,usize)>) {
        self.model_later_orders.insert(view,(members.into_iter().collect(),calls));
    }
    /// Replace owned first-pass calls, including saved-bridge repetitions.
    pub fn model_first_order(&mut self,view:u16,members:impl IntoIterator<Item=(u64,usize)>,calls:Vec<(u64,usize)>) {
        self.model_first_orders.insert(view,(members.into_iter().collect(),calls));
    }
    /// GameScene.lz enters a child at each parent invocation. Terrain remains outside this model stream.
    pub fn root_model_call_order(&mut self,children:impl IntoIterator<Item=u16>,calls:[Vec<ModelSceneCall>;2]) {
        // Zero is the root namespace, never a separately prepared child stream.
        let children=children.into_iter().filter(|&view|view!=0).collect::<std::collections::HashSet<_>>();
        self.root_model_calls=(!children.is_empty()).then_some((children,calls));
    }
    /// Replace owned root first-pass primitives when no child entries are needed.
    /// Missing calls suppress owned primitives; repeated calls remain repeated.
    /// The second model pass remains governed by model admission and traversal.
    pub fn scene_raster_order(&mut self,members:impl IntoIterator<Item=SceneRasterCall>,calls:Vec<SceneRasterCall>) {
        self.scene_view_raster_order(0,members,calls.into_iter().map(crate::scene_raster_order::Call::Primitive).collect(),std::iter::empty());
    }
    pub fn scene_view_raster_order(&mut self,view:u16,members:impl IntoIterator<Item=SceneRasterCall>,calls:Vec<crate::scene_raster_order::Call>,children:impl IntoIterator<Item=u16>) {
        self.scene_raster_calls.insert(view,crate::scene_raster_order::Program {members:members.into_iter().collect(),calls,children:children.into_iter().filter(|&v|v!=0).collect()});
    }
    fn model_pass_schedule(&self,groups:&[Vec<&Triangle>],phase:usize,initial:Vec<usize>)->Vec<usize> {
        let orders=if phase==0 {&self.model_first_orders}else{&self.model_later_orders};
        if orders.is_empty() && self.root_model_calls.is_none() {return initial;}
        let mut result=initial.iter().map(|&i|vec![i]).collect::<Vec<_>>();
        for (&view,(members,calls)) in orders {
            let mut lookup=HashMap::new();let mut slots=Vec::new();
            for (slot,&i) in initial.iter().enumerate() {
                let first=groups[i][0];let key=(first.tag,first.model_order.unwrap().batch);
                if first.view==view && members.contains(&key) {slots.push(slot);lookup.insert(key,i);}
            }
            let mut ordered=calls.iter().filter_map(|key|lookup.get(key).copied());
            for (position,&slot) in slots.iter().enumerate() {
                result[slot]=if position+1==slots.len() {ordered.by_ref().collect()} else {ordered.next().into_iter().collect()};
            }
        }
        let scheduled=result.into_iter().flatten().collect::<Vec<_>>();
        let Some((children,calls))=&self.root_model_calls else{return scheduled;};
        let mut child_streams=HashMap::<u16,Vec<usize>>::new();
        for &index in &scheduled {
            let view=groups[index][0].view;
            if children.contains(&view) {child_streams.entry(view).or_default().push(index);}
        }
        let root_models=groups.iter().enumerate().filter_map(|(index,group)| {
            let first=group[0];(first.view==0).then_some(((first.tag,first.model_order.unwrap().batch),index))
        }).collect::<HashMap<_,_>>();
        let eligible=|index:usize| {
            let first=groups[index][0];children.contains(&first.view) ||
                (first.view==0 && orders.get(&0).is_some_and(|(members,_)|members.contains(&(first.tag,first.model_order.unwrap().batch))))
        };
        let mut remaining=scheduled.iter().filter(|&&index|eligible(index)).count();
        let mut ordered=calls[phase].iter().map(|call|match call {
            ModelSceneCall::Model(tag,batch)=>root_models.get(&(*tag,*batch)).copied().into_iter().collect::<Vec<_>>(),
            ModelSceneCall::Child(view)=>child_streams.get(view).cloned().unwrap_or_default(),
        }).filter(|command|!command.is_empty());
        let mut result=Vec::new();
        for index in scheduled {
            if eligible(index) {
                remaining-=1;
                if remaining==0 {result.extend(ordered.by_ref().flatten());}
                else if let Some(command)=ordered.next() {result.extend(command);}
            }else {result.push(index);}
        }
        result
    }
    fn retain_model_region(&mut self, camera: &Camera, t: &Triangle) {
        self.retain_model_region_phase(camera,t,None);
    }
    fn retain_model_region_phase(&mut self, camera: &Camera, t: &Triangle, phase:Option<usize>) {
        if (t.view != 0 && !self.child_projections.contains_key(&t.view)) || t.tag == 0 || camera.orthographic.is_some() || camera.angle_trig.is_none() { return; }
        let Some(bounds) = t.model_bounds.filter(|b| b.selection.is_some()) else { return; };
        if let Some(order)=t.model_order {
            if let Some(&phases)=self.model_admissions.get(&(t.view,t.tag,order.batch)) {
                // Root static GameModel.by rejects phase one when cb is absent.
                let mut invoked=[phases[0],phases[1] && order.has_alpha];
                if let Some(phase)=phase {invoked[1-phase]=false;}
                if !invoked.iter().any(|&v|v) {return;}
                // Single-tile bounds are tested before the per-face phase gate.
                if !bounds.selection.unwrap().single_tile && order.mode!=crate::model_order::Mode::Priority
                    && !invoked[usize::from(t.alpha!=0)] {return;}
            }
        }
        if self.model_regions.last().is_none_or(|r| r.view != t.view || r.tag != t.tag || r.bounds != bounds) {
            self.model_regions.push(ModelRegion { view: t.view, tag: t.tag, bounds, faces: Vec::new() });
        }
        if bounds.selection.unwrap().single_tile { return; }
        let positions = self.camera_positions(camera, t);
        // GameModel.fr only tests faces with all three projected vertices.
        if positions.iter().any(|v| v[2] < 50.0) { return; }
        let screen = positions.map(|v| camera.screen(v, [self.width, self.height]).map(|v| v as i32));
        let rect = [screen.iter().map(|v| v[0]).min().unwrap(), screen.iter().map(|v| v[0]).max().unwrap(),
                    screen.iter().map(|v| v[1]).min().unwrap(), screen.iter().map(|v| v[1]).max().unwrap()];
        let depth = (positions[0][2] + positions[1][2] + positions[2][2]) as i32 / 3;
        self.model_regions.last_mut().unwrap().faces.push((rect, depth));
    }
    pub fn draw(
        &mut self,
        camera: &Camera,
        triangles: &[Triangle],
        palette: &[u32],
        textures: &HashMap<u16, Texture>,
    ) {
        self.draw_iter(camera, triangles.iter(), palette, textures);
    }
    fn model_screen(&self, camera: &Camera, p: [f32; 3]) -> [f32; 2] {
        if self.legacy_icon {
            let center = camera.center([self.width, self.height]);
            [
                center[0]
                    + ((p[0] as i32).wrapping_mul(camera.focal as i32)
                        / (camera.orthographic.unwrap_or(p[2]) as i32))
                        as f32,
                center[1]
                    + ((p[1] as i32).wrapping_mul(camera.focal as i32)
                        / (camera.orthographic.unwrap_or(p[2]) as i32))
                        as f32,
            ]
        } else {
            camera.screen(p, [self.width, self.height])
        }
    }
    /// GameModel.bj: integer projection, spherical depth buckets and one mixed-alpha pass.
    pub fn draw_icon(
        &mut self,
        camera: &Camera,
        triangles: &[Triangle],
        priorities: Option<&[u8]>,
        radius: i32,
        origin: i32,
        palette: &[u32],
        textures: &HashMap<u16, Texture>,
    ) {
        // The Java entry point catches projection division by zero before drawing any faces.
        if radius >= 3000
            || camera.orthographic == Some(0.)
            || (camera.orthographic.is_none()
                && triangles.iter().any(|t| {
                    self.camera_positions(camera, t)
                        .iter()
                        .any(|v| v[2] as i32 == 0)
                }))
        {
            return;
        }
        self.legacy_icon = true;
        self.legacy_origin = origin;
        self.draw_legacy(camera, triangles, priorities, radius, palette, textures);
        self.legacy_icon = false;
        self.legacy_origin = 0;
    }
    pub fn depths(&self) -> &[f32] {
        &self.depth
    }
    /// Original GameModel.fr mode dispatch for one renderable.
    pub fn draw_model_ordered(
        &mut self,
        camera: &Camera,
        triangles: &[Triangle],
        mode: crate::model_order::Mode,
        priorities: Option<&[u8]>,
        has_alpha: bool,
        offset: i32,
        origin: i32,
        palette: &[u32],
        textures: &HashMap<u16, Texture>,
    ) {
        let faces = triangles.iter().collect::<Vec<_>>();
        for phase in 0..2 {
            self.draw_model_phase(
                camera, &faces, mode, priorities, has_alpha, offset, origin, None, phase, palette,
                textures,
            );
        }
    }
    fn draw_model_phase(
        &mut self,
        camera: &Camera,
        triangles: &[&Triangle],
        mode: crate::model_order::Mode,
        priorities: Option<&[u8]>,
        has_alpha: bool,
        offset: i32,
        origin: i32,
        bucket_count: Option<i32>,
        phase: u8,
        palette: &[u32],
        textures: &HashMap<u16, Texture>,
    ) {
        let mode = mode as u8;
        let previous = self.write_color;
        let mut faces = Vec::new();
        for (index, t) in triangles.iter().enumerate() {
            if mode != 3 && (t.alpha != 0) != (phase == 1) {
                continue;
            }
            let positions = self.camera_positions(camera, t);
            let front = if positions.iter().any(|p| p[2] < camera.near) {
                crate::model_near_clip::front_facing(positions)
            } else {
                let p = positions.map(|p| camera.screen(p, [self.width, self.height]));
                (p[0][0] - p[1][0]) * (p[2][1] - p[1][1])
                    - (p[0][1] - p[1][1]) * (p[2][0] - p[1][0])
                    > 0.
            };
            if !front {
                continue;
            }
            let sum = positions
                .iter()
                .map(|p| (p[2] as i32).wrapping_sub(origin))
                .fold(0i32, |a, b| a.wrapping_add(b));
            faces.push(crate::model_order::Face {
                index,
                depth: sum / 3 + offset,
                priority: priorities.map_or(0, |p| p[index]),
            });
        }
        let mode = if mode == 0 {
            if phase == 1 && has_alpha {
                2
            } else {
                1
            }
        } else {
            mode
        };
        if mode == 2 || mode == 3 {
            if faces.iter().any(|f| f.depth < 0 || f.depth >= 6000) {
                self.write_color = previous;
                return;
            }
            if let Some(count) = bucket_count {
                faces.retain(|f| f.depth < count);
            }
        }
        let passes = if mode == 3 { 2 } else { 1 };
        'passes: for pass in 0..passes {
            self.write_color = pass == 0;
            let write_depth = if mode == 3 { pass == 1 } else { mode != 4 };
            let mut pass_faces = faces.clone();
            if pass == 1 {
                pass_faces.extend_from_slice(&faces);
            }
            let order = if mode == 2 || mode == 3 {
                crate::model_order::sorted(pass_faces, mode == 3 && priorities.is_some())
            } else {
                pass_faces.iter().map(|f| f.index).collect()
            };
            for index in order {
                let t = &triangles[index];
                if t.alpha == 255 || (t.alpha != 0) != (phase == 1) {
                    continue;
                }
                if !self.triangle(camera, t, palette, textures, write_depth) {
                    break 'passes;
                }
            }
        }
        self.write_color = previous;
    }
    /// Original non-depth model path, used by item sprites. Priorities match submitted faces.
    pub fn draw_legacy(
        &mut self,
        camera: &Camera,
        triangles: &[Triangle],
        priorities: Option<&[u8]>,
        depth_offset: i32,
        palette: &[u32],
        textures: &HashMap<u16, Texture>,
    ) {
        let previous = self.legacy_raster;
        self.legacy_raster = true;
        for phase in if self.legacy_icon { 2..3 } else { 0..2 } {
            self.legacy_aborted = false;
            let faces = triangles
                .iter()
                .enumerate()
                .filter_map(|(index, t)| {
                    if t.alpha == 255 || (phase < 2 && (t.alpha != 0) != (phase == 1)) {
                        return None;
                    }
                    let positions = self.camera_positions(camera, t);
                    let front = if !self.legacy_icon && positions.iter().any(|p| p[2] < camera.near)
                    {
                        crate::model_near_clip::front_facing(positions)
                    } else {
                        let p = positions.map(|p| self.model_screen(camera, p));
                        (p[0][0] - p[1][0]) * (p[2][1] - p[1][1])
                            - (p[0][1] - p[1][1]) * (p[2][0] - p[1][0])
                            > 0.
                    };
                    if !front {
                        return None;
                    }
                    Some(crate::model_order::Face {
                        index,
                        depth: positions
                            .iter()
                            .map(|p| (p[2] as i32).wrapping_sub(self.legacy_origin))
                            .sum::<i32>()
                            / 3
                            + depth_offset,
                        priority: priorities.map_or(0, |p| p[index]),
                    })
                })
                .collect();
            for index in crate::model_order::sorted(faces, priorities.is_some()) {
                if self.legacy_aborted {
                    break;
                }
                if !self.triangle(camera, &triangles[index], palette, textures, false) {
                    break;
                }
            }
        }
        self.legacy_raster = previous;
    }
    fn draw_model_group(
        &mut self,
        camera: &Camera,
        faces: &[&Triangle],
        phase: u8,
        palette: &[u32],
        textures: &HashMap<u16, Texture>,
    ) {
        let Some(first) = faces.first() else {
            return;
        };
        let bounds = first.model_bounds.unwrap();
        let order = first.model_order.unwrap();
        let has_alpha = faces.iter().any(|t| t.model_order.unwrap().has_alpha);
        if let Some(phases)=self.model_admissions.get(&(first.view,first.tag,order.batch)) {
            if !phases[phase as usize] || (phase==1 && !has_alpha) {return;}
        }
        // GameModel.mh's early alpha-phase gate only checks cb, unlike the root by path.
        if phase == 1
            && first.view != 0
            && self.child_projections.contains_key(&first.view)
            && !has_alpha
        {
            return;
        }
        let radius_squared = bounds.radius.wrapping_mul(bounds.radius);
        let extent = |height: i32| {
            (radius_squared.wrapping_add(height.wrapping_mul(height)) as f64)
                .sqrt()
                .ceil() as i32
        };
        let offset = extent(bounds.height);
        let diameter = offset.wrapping_add(extent(bounds.below));
        if diameter >= 6000 {
            return;
        }
        let origin = if first.view != 0 && self.child_projections.contains_key(&first.view) {
            let origin_triangle = Triangle {
                vertices: [bounds.position.map(|v| v as f32); 3],
                local_vertices: Some([[0.; 3]; 3]),
                ..**first
            };
            self.camera_positions(camera, &origin_triangle)[0][2] as i32
        } else if let Some([sp, cp, sy, cy]) = camera.fixed_trig {
            let [x, y, z] =
                std::array::from_fn(|i| bounds.position[i].wrapping_sub(camera.position[i] as i32));
            let yaw = z.wrapping_mul(cy).wrapping_sub(x.wrapping_mul(sy)) >> 16;
            y.wrapping_mul(sp).wrapping_add(yaw.wrapping_mul(cp)) >> 16
        } else {
            camera.transform(bounds.position.map(|v| v as f32))[2] as i32
        };
        let has_priorities = faces.iter().any(|t| {
            let v = t.model_order.unwrap();
            v.has_priorities || v.priority != order.priority
        });
        let priorities = has_priorities.then(|| faces.iter()
            .map(|t| t.model_order.unwrap().priority).collect::<Vec<_>>());
        self.draw_model_phase(
            camera,
            faces,
            order.mode,
            priorities.as_deref(),
            has_alpha,
            offset,
            origin,
            Some(diameter),
            phase,
            palette,
            textures,
        );
    }
    /// Terrain faces must be contiguous by (view, tag), as emitted by WorldMesh.
    pub fn draw_iter<'a>(
        &mut self,
        camera: &Camera,
        triangles: impl IntoIterator<Item = &'a Triangle>,
        palette: &[u32],
        textures: &HashMap<u16, Texture>,
    ) {
        #[cfg(feature="native-profiling")]
        let mut profile_start=self.submission_profile.map(|_|std::time::Instant::now());
        #[cfg(feature="custom-assets")]
        if !self.authored_visual_batches.is_empty() {
            // Admission is frozen for this draw. Iterator classification can then
            // run while the Frame is mutably borrowed, without copying geometry.
            let registered=self.authored_visual_batches.clone();
            let mut decisions=HashMap::new();let mut last_wide=None;
            let input=triangles.into_iter().map(|t| {
                let wide=t.model_order.is_some_and(|order|
                    cached_value(&mut last_wide,(t.view,t.tag,order.batch),|| {
                        if t.view!=0 || !registered.contains(&(t.tag,order.batch)) {return false;}
                        *decisions.entry((t.tag,order.batch)).or_insert_with(|| {
                            let Some(bounds)=t.model_bounds.filter(|b|b.selection.is_none()) else{return false;};
                            let r=f64::from(bounds.radius);
                            (r*r+f64::from(bounds.height).powi(2)).sqrt().ceil()
                                +(r*r+f64::from(bounds.below).powi(2)).sqrt().ceil()>=6000.
                        })
                    }));
                DrawFace {source:t,wide}
            });
            #[cfg(feature="native-profiling")]
            self.mark_submission(&mut profile_start,0);
            self.draw_iter_native(camera,input,palette,textures);
            return;
        }
        #[cfg(feature="native-profiling")]
        self.mark_submission(&mut profile_start,0);
        self.draw_iter_native(camera,triangles.into_iter().map(DrawFace::plain),palette,textures);
    }
    fn draw_iter_native<'a>(
        &mut self,
        camera: &Camera,
        triangles: impl IntoIterator<Item = DrawFace<'a>>,
        palette: &[u32],
        textures: &HashMap<u16, Texture>,
    ) {
        #[cfg(feature="native-profiling")]
        let mut profile_start=self.submission_profile.map(|_|std::time::Instant::now());
        let frustum = camera.model_frustum([self.width, self.height]);
        let mut last = None;
        let mut alpha = Vec::new();
        let mut failed = Vec::new();
        let mut groups: Vec<Vec<&Triangle>>=Vec::new();
        let mut group_indices=HashMap::new();
        let mut last_group=None;
        let child_projections = self.child_projections.clone();
        let raster_program = self.scene_raster_calls.clone();
        let mut terrain_groups: Vec<Vec<DrawFace<'a>>> = Vec::new();
        let mut terrain_indices = HashMap::new();
        let mut last_terrain_group=None;
        let mut emit = |face: DrawFace<'a>| {
            let t=face.source;
            if !raster_program.is_empty() && t.tag & (1u64<<63) != 0 {
                let key=(t.view,t.tag);
                let index=cached_value(&mut last_terrain_group,key,||*terrain_indices.entry(key).or_insert_with(|| {terrain_groups.push(Vec::new());terrain_groups.len()-1}));
                terrain_groups[index].push(face);return;
            }
            if !face.is_wide() {
            if !model_visible(t, frustum.as_ref(), &mut last) { return; }
            if let Some(order)=t.model_order.filter(|_|t.model_bounds.is_some()) {
                let key=(t.view,t.tag,order.batch);
                let index=cached_value(&mut last_group,key,||*group_indices.entry(key).or_insert_with(|| {groups.push(Vec::new());groups.len()-1}));
                groups[index].push(t);return;
            }
            self.retain_model_region(camera, t);
            }
            face.with(|t| {
            if t.alpha == 0 {
                let key=(t.view,t.tag,t.model_bounds);
                if !failed.contains(&key) && !self.triangle(camera,t,palette,textures,true) {failed.push(key);}
            } else if t.alpha < 255 {
                let z = self.camera_positions(camera, t).iter().map(|v| v[2]).sum::<f32>();
                alpha.push((z, face));
            }
            });
        };
        let mut terrain = Vec::with_capacity(6);
        let mut terrain_key = None;
        let mut visible = true;
        for face in triangles {
            let t=face.source;
            let ground = t.tag & 0x8000000000000000 != 0;
            let key = (t.view, t.tag);
            if !ground || terrain_key != Some(key) {
                flush_terrain(&mut terrain, visible, &mut emit);
                terrain_key = None;
                visible = true;
            }
            if ground {
                terrain_key = Some(key);
                visible = visible && if let (Some(local), Some((matrix, _))) = (t.local_vertices, child_projections.get(&t.view)) {
                    local.iter().all(|&v| crate::matrix_pick::transform(matrix, v)[2] >= camera.near)
                } else { camera.terrain_visible(t.vertices) };
                terrain.push(face);
            } else {
                emit(face);
            }
        }
        flush_terrain(&mut terrain, visible, &mut emit);
        drop(emit);
        #[cfg(feature="native-profiling")]
        {
            self.mark_submission(&mut profile_start,1);
            if let Some(profile)=self.submission_profile.as_mut() {
                profile.model_groups=profile.model_groups.saturating_add(groups.len());
                profile.terrain_groups=profile.terrain_groups.saturating_add(terrain_groups.len());
            }
        }
        if !self.temporary_models.is_empty() {
            let keys=groups.iter().enumerate().map(|(index,group)| {
                let first=group[0];let order=first.model_order.unwrap();
                let &(zone,local)=self.temporary_models.get(&(first.view,first.tag,order.batch))?;
                let mut position=first.model_bounds.unwrap().position;
                position[1]=position[1].wrapping_add(order.scene_height_offset);
                let position=position.map(|v|v as f32);
                let z=self.child_projections.get(&first.view).map_or_else(
                    ||camera.transform(position)[2],
                    |(matrix,_)|crate::matrix_pick::transform(matrix,position)[2]);
                Some((first.view,zone,crate::scene_temp_order::depth(z,local),order.source_ordinal.or_else(||self.temporary_sources.get(&(first.view,order.actor_spot.unwrap_or(first.tag))).copied()).unwrap_or(index)))
            }).collect::<Vec<_>>();
            groups=crate::scene_temp_order::schedule_with_source(&keys).into_iter()
                .map(|i|std::mem::take(&mut groups[i])).collect();
        }
        let opaque_keys=groups.iter().map(|group| {
            let first=group[0];let order=first.model_order.unwrap();
            self.model_opaque_orders.get(&(first.view,first.tag,order.batch)).copied().map(|key|(first.view,key))
        }).collect::<Vec<_>>();
        let initial=crate::object_scene_order::schedule(&opaque_keys);
        let models=if raster_program.is_empty(){self.model_pass_schedule(&groups,0,initial)}else{initial};
        let mut first_calls=terrain_groups.iter().enumerate().map(|(index,group)|
            (SceneRasterCall::Terrain(group[0].source.view,group[0].source.tag),true,index)).collect::<Vec<_>>();
        first_calls.extend(models.into_iter().map(|index| {
            let t=groups[index][0];(SceneRasterCall::Model(t.view,t.tag,t.model_order.unwrap().batch),false,index)
        }));
        if !raster_program.is_empty() {
            let lookup=first_calls.iter().map(|&(key,terrain,index)|(key,(key,terrain,index))).collect::<HashMap<_,_>>();
            let initial=first_calls.iter().map(|&(key,_,_)|key).collect::<Vec<_>>();
            first_calls=crate::scene_raster_order::schedule(&initial,&raster_program).into_iter().filter_map(|key|lookup.get(&key).copied()).collect();
        }
        for (_,terrain,index) in first_calls {
            if terrain {
                for face in &terrain_groups[index] {face.with(|t|self.triangle(camera,t,palette,textures,true));}
                continue;
            }
            let group=&groups[index];
            // Picking observes the same renderable order as the color pass,
            // including equal-depth temporary objects and consecutive tag reduction.
            for triangle in group {self.retain_model_region_phase(camera,triangle,Some(0));}
            self.draw_model_group(camera,group,0,palette,textures);
        }
        #[cfg(feature="native-profiling")]
        self.mark_submission(&mut profile_start,2);
        failed.clear();
        alpha.sort_by(|a, b| b.0.total_cmp(&a.0));
        for (_, face) in alpha {
            face.with(|t| {
                let key=(t.view,t.tag,t.model_bounds);
                if !failed.contains(&key) && !self.triangle(camera,t,palette,textures,true) {failed.push(key);}
            });
        }
        for index in self.model_pass_schedule(&groups,1,(0..groups.len()).collect()) {
            let group=&groups[index];
            let first=group[0];let order=first.model_order.unwrap();
            if self.model_admissions.contains_key(&(first.view,first.tag,order.batch)) {
                // Original picking runs again on alpha-pass model invocation.
                // Later equal-depth hits affect stable duplicate reduction.
                for triangle in group {self.retain_model_region_phase(camera,triangle,Some(1));}
            }
            self.draw_model_group(camera,group,1,palette,textures);
        }
        #[cfg(feature="native-profiling")]
        self.mark_submission(&mut profile_start,3);
    }
    /// GameScene's skybox uses model order and disables depth writes (fq.as).
    /// It follows camera orientation but ignores the camera's world position.
    pub fn draw_background(
        &mut self,
        camera: &Camera,
        triangles: &[Triangle],
        palette: &[u32],
        textures: &HashMap<u16, Texture>,
    ) {
        let centered = Camera {
            position: [0.0; 3],
            right: camera.right,
            up: camera.up,
            forward: camera.forward,
            angle_trig: camera.angle_trig,
            fixed_trig: camera.fixed_trig,
            focal: camera.focal,
            near: camera.near,
            far: camera.far,
            screen_offset: camera.screen_offset,
            orthographic: camera.orthographic,
        };
        let frustum = centered.model_frustum([self.width, self.height]);
        let mut last = None;
        let mut failed=Vec::new();
        for triangle in triangles {
            if !model_visible(triangle, frustum.as_ref(), &mut last) { continue; }
            if triangle.alpha < 255 {
                let key=(triangle.view,triangle.tag,triangle.model_bounds);
                if !failed.contains(&key) && !self.triangle(&centered,triangle,palette,textures,false) {failed.push(key);}
            }
        }
    }
    fn triangle(
        &mut self,
        camera: &Camera,
        t: &Triangle,
        palette: &[u32],
        textures: &HashMap<u16, Texture>,
        write_depth: bool,
    ) -> bool {
        let positions = self.camera_positions(camera, t);
        let texture_mapping = t.texture.and(t.texture_mapping).map(|mapping| {
            let mapped = Triangle { vertices: mapping.vertices, local_vertices: mapping.local, ..*t };
            self.camera_positions(camera, &mapped).map(|p| p.map(|c| c as i32))
        });
        let input: [Vertex; 3] = std::array::from_fn(|i| Vertex {
            #[cfg(feature="enhanced-renderer")]
            presentation_world: (self.presentation_geometry.is_some()
                && !(t.local_vertices.is_some() && self.child_projections.contains_key(&t.view)))
                .then_some(t.vertices[i]),
            screen: self.legacy_icon.then(||self.model_screen(camera,positions[i])),
            mapped_depth: None,
            p: positions[i],
            color: t.colors[i] as f32,
            uv: t.uv[i],
        });
        if input.iter().any(|v| v.p.iter().any(|c| !c.is_finite())) {
            return true;
        }
        // The ordinary three-vertex polygon needs no heap allocation. Keep the
        // exact original vertex fields and projected call, including abort state.
        let direct=self.legacy_icon || input.iter().all(|v|v.p[2]>=camera.near);
        #[cfg(test)]
        let direct=direct && !self.force_polygon;
        if direct {
            if !(self.legacy_raster && self.legacy_aborted) {
                self.projected(camera,input,t,texture_mapping,palette,textures,write_depth);
            }
            return true;
        }
        // Both clipping paths produce an empty polygon when every vertex is
        // behind near. Icons deliberately bypass clipping and must be retained.
        let behind = !self.legacy_icon && input.iter().all(|v| v.p[2] < camera.near);
        #[cfg(test)]
        let behind = behind && !self.force_polygon;
        if behind { return true; }
        let model_clip = !self.legacy_icon && t.tag & 0x8000000000000000 == 0
            && camera.near == 50.0
            && camera.orthographic.is_none()
            && input.iter().any(|v| v.p[2] < 50.0);
        let mut polygon = if self.legacy_icon {input.to_vec()} else if model_clip {
            if !crate::model_near_clip::front_facing(input.map(|v| v.p)) { return true; }
            let center = camera.center([self.width, self.height]).map(|v| v as i32);
            let Some(clipped) = crate::model_near_clip::polygon(
                input.map(|v| v.p), t.colors, center, camera.focal as i32,
            ) else { return false; };
            clipped.into_iter().map(|v| {
                let mut vertex = Vertex::lerp(input[v.edge[0]], input[v.edge[1]], v.fraction);
                vertex.screen = Some(v.screen.map(|v| v as f32));
                vertex.p[2] = v.depth;
                vertex.color = v.color as f32;
                vertex
            }).collect::<Vec<_>>()
        } else {
        let mut polygon = Vec::with_capacity(4);
        for i in 0..3 {
            let a = input[i];
            let b = input[(i + 1) % 3];
            let inside_a = a.p[2] >= camera.near;
            let inside_b = b.p[2] >= camera.near;
            if inside_a {
                polygon.push(a);
            }
            if inside_a != inside_b {
                polygon.push(Vertex::lerp(
                    a,
                    b,
                    (camera.near - a.p[2]) / (b.p[2] - a.p[2]),
                ));
            }
        }
            polygon
        };
        if model_clip && polygon.len() == 4 {
            polygon[3].mapped_depth = Some(polygon[3].p[2]);
        }
        let clipped_triangle;
        let t = if model_clip {
            clipped_triangle = Triangle { alpha: t.near_alpha.unwrap_or(t.alpha), ..*t };
            &clipped_triangle
        } else { t };
        for i in 1..polygon.len().saturating_sub(1) {
            if self.legacy_raster && self.legacy_aborted {break;}
            self.projected(
                camera,
                [polygon[0], polygon[i], polygon[i + 1]],
                t,
                texture_mapping,
                palette,
                textures,
                write_depth,
            );
        }
        true
    }
    fn projected(
        &mut self,
        camera: &Camera,
        v: [Vertex; 3],
        t: &Triangle,
        texture_mapping: Option<[[i32;3];3]>,
        palette: &[u32],
        textures: &HashMap<u16, Texture>,
        write_depth: bool,
    ) {
        let p = v.map(|v| v.screen.unwrap_or_else(|| camera.screen(v.p, [self.width, self.height])));
        let edge = |a: [f32; 2], b: [f32; 2], p: [f32; 2]| {
            (p[0] - a[0]) * (b[1] - a[1]) - (p[1] - a[1]) * (b[0] - a[0])
        };
        let area = edge(p[0], p[1], p[2]);
        if area < 0.001 {
            return;
        }
        if write_depth && t.tag & 0x8000000000000000 != 0 {
            let vertices = p.map(|p| p.map(|v| v as i32));
            let min = std::array::from_fn::<_, 2, _>(|a| {
                vertices.iter().map(|p| p[a]).min().unwrap().max(0)
            });
            let max = [self.width as i32 - 1, self.height as i32 - 1];
            let max = std::array::from_fn::<_, 2, _>(|a| {
                vertices.iter().map(|p| p[a]).max().unwrap().min(max[a])
            });
            let depths = v.map(|v| 1.0 / v.p[2]);
            self.ground_faces.push((t.view, t.tag, vertices, depths));
            for y in min[1]..=max[1] {
                for x in min[0]..=max[0] {
                    let index = y as usize * self.width + x as usize;
                    if let Some(depth) = terrain_pick([x, y], vertices, depths) {
                        if depth > self.ground_depth[index] {
                            self.ground_depth[index] = depth;
                            self.ground_tags[index] = t.tag;
                            self.ground_views[index] = t.view;
                        }
                    }
                }
            }
        }
        // Original terrain projection registers selection before its color sentinel gate.
        if t.tag & (1u64<<63)!=0 && t.texture.is_none() && t.colors[0]==12345678 {return;}
        let min_x = p
            .iter()
            .map(|v| v[0])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as usize;
        let max_x = p
            .iter()
            .map(|v| v[0])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(self.width as f32) as usize;
        let min_y = p
            .iter()
            .map(|v| v[1])
            .fold(f32::INFINITY, f32::min)
            .floor()
            .max(0.0) as usize;
        let max_y = p
            .iter()
            .map(|v| v[1])
            .fold(f32::NEG_INFINITY, f32::max)
            .ceil()
            .min(self.height as f32) as usize;
        if min_x >= max_x || min_y >= max_y {
            return;
        }
        #[cfg(feature="enhanced-renderer")]
        if self.write_color && !self.presentation_overflow {
            if let Some(geometry) = &mut self.presentation_geometry {
                if geometry.len() == 1_000_000 {
                    geometry.clear();
                    self.presentation_overflow = true;
                } else {
                    let uv = texture_mapping.map_or_else(|| v.map(|v|v.uv),
                        |mapping| texture_uv(mapping.map(|p|p.map(|v|v as f32)),v.map(|v|v.p)));
                    geometry.push(Triangle {
                        vertices:v.map(|v|v.presentation_world.unwrap_or_else(||camera.presentation_world(v.p))),
                        colors:if t.flat {t.colors}else{v.map(|v|v.color as i32)},
                        uv, local_vertices:None, model_bounds:None, model_order:None,
                        texture_mapping:None, ..*t
                    });
                }
            }
        }
        self.triangles += 1;
        let inv = v.map(|v| 1.0 / v.p[2]);
        let mapped = v.map(|v| v.mapped_depth.unwrap_or_else(|| raster_depth(v.p[2] - t.depth_bias as f32)));
        let texture = t.texture.and_then(|id| textures.get(&id));
        if self.legacy_raster {
            let vertices=std::array::from_fn(|i|[p[i][0],p[i][1],mapped[i]]);
            if let (Some(texture),Some(mapping))=(texture.filter(|t|t.size==128),texture_mapping) {
                let center=camera.center([self.width,self.height]).map(|v|v as i32);
                crate::legacy_raster::texture(vertices,if t.flat {[t.colors[0];3]}else{v.map(|v|v.color as i32)},mapping,camera.focal as i32,center[1],self.height,
                    |y,start,end,shade,shade_step,mapping,increments| {
                        crate::texture_span::Span {row:y*self.width,width:self.width,center:center[0],start,end,shade,shade_step,initial:0.,depth_step:0.,mapping,increments,alpha:t.alpha,opaque:texture.definition.opaque}.draw_legacy(&mut self.pixels,&texture.pixels,0xff000000);
                    });
                return;
            }
            if texture.is_none() {
                if t.flat {
                    let rgb=*palette.get(t.colors[0].clamp(0,65535) as usize).unwrap_or(&0xff00ff);
                    crate::legacy_raster::spans(vertices,self.height,|y,start,end| {
                        for x in start.max(0)..end.min(self.width as i32) {
                            let index=y*self.width+x as usize;
                            if t.alpha==254 {if index+1<self.pixels.len(){self.pixels[index]=self.pixels[index+1];}}
                            else {self.pixels[index]=crate::model_alpha::blend(rgb,self.pixels[index],t.alpha);}
                        }
                    });
                } else {
                    crate::legacy_raster::gouraud(vertices,v.map(|v|v.color as i32),self.height,|y,start,end,initial,step| {
                        let start=start.max(0);let end=end.min(self.width as i32);
                        for x in start..end {
                            let sample=if self.coarse_shading {start+(x-start)/4*4}else{x};
                            let color=initial.wrapping_add(step.wrapping_mul(sample)).max(0)>>8;
                            if self.legacy_aborted {return;}
                            let Some(&rgb)=palette.get(color as usize) else {self.legacy_aborted=true;return;};let index=y*self.width+x as usize;
                            self.pixels[index]=crate::model_alpha::blend(rgb,self.pixels[index],t.alpha);
                        }
                    });
                }
                return;
            }
        }
        if let (Some(texture), Some(mapping)) = (texture.filter(|t| t.size == 128), texture_mapping) {
            let vertices = std::array::from_fn(|i| [p[i][0], p[i][1], mapped[i]]);
            let colors = if t.flat { [t.colors[0];3] } else { v.map(|v| v.color as i32) };
            let center = camera.center([self.width, self.height]).map(|v| v as i32);
            let terrain = t.tag & (1 << 63) != 0;
            let scan = if terrain { crate::texture_raster::terrain_spans } else { crate::texture_raster::spans };
            scan(vertices, colors, mapping, camera.focal as i32, center[1], self.height,
                |y,start,end,shade,shade_step,initial,depth_step,mapping,increments| {
                    let span = crate::texture_span::Span { row:y*self.width, width:self.width, center:center[0], start,end,
                        shade,shade_step,initial,depth_step,mapping,increments,alpha:t.alpha,opaque:texture.definition.opaque };
                    let tags=&mut self.tags;let views=&mut self.views;
                    let hit=|index| {if write_depth {tags[index]=t.tag;views[index]=t.view;}};
                    if terrain {span.draw_terrain(&mut self.pixels,&mut self.depth,&texture.pixels,write_depth,0xff000000,hit);}else{span.draw_masked(&mut self.pixels,&mut self.depth,&texture.pixels,write_depth,0xff000000,self.write_color,hit);}
                });
            return;
        }
        if t.flat && texture.is_none() {
            let vertices = std::array::from_fn(|i| [p[i][0], p[i][1], mapped[i]]);
            let rgb = *palette.get(t.colors[0].clamp(0, 65535) as usize).unwrap_or(&0xff00ff);
            crate::flat_raster::spans(vertices, self.height, |y, start, end, initial, step| {
                let row = y * self.width;
                if t.alpha == 254 {
                    if !self.write_color {return;}
                    let tags = &mut self.tags;
                    let views = &mut self.views;
                    crate::model_alpha::shift_span(&mut self.pixels, &self.depth, row, self.width,
                        start, end, initial, step, |index| {
                            if write_depth { tags[index] = t.tag; views[index] = t.view; }
                        });
                    return;
                }
                let start = start.max(0);
                let end = end.min(self.width as i32);
                let mut z = initial + step * start as f32;
                for x in start..end {
                    let index = row + x as usize;
                    if depth_pass(self.depth[index], z) {
                        if t.alpha == 0 {
                            if self.write_color {self.pixels[index] = 0xff000000 | rgb;}
                            if write_depth { self.depth[index] = z; }
                        } else {
                            if self.write_color {self.pixels[index] = crate::model_alpha::blend(rgb, self.pixels[index], t.alpha);}
                        }
                        if write_depth { self.tags[index] = t.tag; self.views[index] = t.view; }
                    }
                    z += step;
                }
            });
            return;
        }
        if texture.is_none() {
            let vertices = std::array::from_fn(|i| [p[i][0], p[i][1], mapped[i]]);
            crate::gouraud_raster::spans(vertices, v.map(|v| v.color), self.height,
                |y, start, end, shade, shade_step, initial, step| {
                    crate::gouraud_raster::shade(start, end, self.width, shade, shade_step,
                        initial, step, self.coarse_shading, |x, color, z| {
                            let index = y * self.width + x;
                            if !depth_pass(self.depth[index], z) { return; }
                            let rgb = *palette.get(color).unwrap_or(&0xff00ff);
                            if t.alpha == 0 {
                                if self.write_color {self.pixels[index] = 0xff000000 | rgb;}
                                if write_depth { self.depth[index] = z; }
                            } else {
                                if self.write_color {self.pixels[index] = crate::model_alpha::blend(rgb, self.pixels[index], t.alpha);}
                            }
                            if write_depth { self.tags[index] = t.tag; self.views[index] = t.view; }
                        });
                });
            return;
        }
        for y in min_y..max_y {
            for x in min_x..max_x {
                let pixel = [x as f32 + 0.5, y as f32 + 0.5];
                let weights = [
                    edge(p[1], p[2], pixel) / area,
                    edge(p[2], p[0], pixel) / area,
                    edge(p[0], p[1], pixel) / area,
                ];
                if weights.iter().any(|&w| w < 0.0) {
                    continue;
                }
                let depth = (0..3).map(|i| weights[i] * inv[i]).sum::<f32>();
                let index = y * self.width + x;
                // Difference form preserves the original gradient path at infinite depth
                // and avoids rounding a constant-depth face through three products.
                let test_depth = mapped[0]
                    + weights[1] * (mapped[1] - mapped[0])
                    + weights[2] * (mapped[2] - mapped[0]);
                if !depth_pass(self.depth[index], test_depth) {
                    continue;
                }
                let light = if t.flat {
                    t.colors[0] as f32
                } else {
                    (0..3).map(|i| weights[i] * v[i].color).sum::<f32>()
                };
                let rgb = if let Some(texture) = texture {
                    let uv: [f32; 2] = std::array::from_fn(|axis| {
                        (0..3)
                            .map(|i| weights[i] * inv[i] * v[i].uv[axis])
                            .sum::<f32>()
                            / depth
                    });
                    let size = texture.size;
                    let (tx, ty) = (
                        ((uv[0] * size as f32).floor() as i32).rem_euclid(size as i32) as usize,
                        ((uv[1] * size as f32).floor() as i32).rem_euclid(size as i32) as usize,
                    );
                    let color = texture.pixels[ty * size + tx];
                    if color == 0 && !texture.definition.opaque {
                        continue;
                    }
                    let brightness = (light * 2.0).clamp(0.0, 255.0) as u32;
                    (((color >> 16 & 255) * brightness >> 8) << 16)
                        | (((color >> 8 & 255) * brightness >> 8) << 8)
                        | ((color & 255) * brightness >> 8)
                } else {
                    let index = (light as i32).clamp(0, 65535) as usize;
                    *palette.get(index).unwrap_or(&0xff00ff)
                };
                if t.alpha == 0 {
                    if self.write_color {self.pixels[index] = 0xff000000 | rgb;}
                    if write_depth {
                        self.depth[index] = test_depth;
                    }
                } else {
                    if self.write_color {self.pixels[index] = crate::model_alpha::blend(rgb, self.pixels[index], t.alpha);}
                }
                if write_depth {
                    self.tags[index] = t.tag;
                    self.views[index] = t.view;
                }
            }
        }
    }
}
/// Texture-space coordinates relative to the model's mapping triangle.
pub fn texture_uv(mapping: [[f32; 3]; 3], vertices: [[f32; 3]; 3]) -> [[f32; 2]; 3] {
    let u = sub(mapping[1], mapping[0]);
    let v = sub(mapping[2], mapping[0]);
    let uu = dot(u, u);
    let uv = dot(u, v);
    let vv = dot(v, v);
    let determinant = uu * vv - uv * uv;
    if determinant.abs() < 0.00001 {
        return [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
    }
    vertices.map(|p| {
        let w = sub(p, mapping[0]);
        let wu = dot(w, u);
        let wv = dot(w, v);
        [
            (wu * vv - wv * uv) / determinant,
            (wv * uu - wu * uv) / determinant,
        ]
    })
}
#[cfg(test)]
mod unclipped_triangle_tests {
    use super::*;
    #[test]
    fn direct_vertices_match_old_polygon_pixels_depth_picks_and_capture() -> Result<()> {
        let palette=(0..65536u32).map(|v|v.wrapping_mul(123457)&0xffffff).collect::<Vec<_>>();
        let camera=Camera::angles([0;3],0,0,64.)?;
        let mut textures=HashMap::new();
        textures.insert(1,Texture {definition:crate::texture::Definition {sprite:0,average_hsl:100,opaque:false,direction:0,speed:0},size:64,
            pixels:(0..4096).map(|i|if i%3==0 {0}else{0xffa060}).collect()});
        let mut drawn=0;
        for mode in 0..3 {for z in [[49.;3],[49.9;3],[-5000.,-100.,49.],[50.;3],[200.;3],[49.,200.,200.],[1.,4096.,200.]] {for flat in [false,true] {
        for texture in [None,Some(1),Some(2)] {for alpha in [0,64,255] {for reverse in [false,true] {
        for ground in [false,true] {for aborted in [false,true] {
            let vertices=[[-20.,-20.,z[0]],[0.,20.,z[1]],[20.,-20.,z[2]]];
            let mut t=Triangle::selection_only(crate::model_bounds::Bounds::new(&vertices,[0;3]),123);
            t.model_bounds=None;t.vertices=vertices;t.colors=[100,200,300];t.flat=flat;t.alpha=alpha;
            t.uv=[[0.,0.],[0.5,1.],[1.,0.]];t.texture=texture;
            t.texture_mapping=texture.map(|_|TextureMapping{vertices,local:None});
            if reverse {t.vertices.swap(1,2);t.uv.swap(1,2);}
            if ground {t.tag|=1u64<<63;}
            let mut direct=Frame::new(64,64)?;let mut old=Frame::new(64,64)?;
            for frame in [&mut direct,&mut old] {
                frame.legacy_raster=mode!=0;frame.legacy_icon=mode==2;frame.legacy_aborted=aborted;
                #[cfg(feature="enhanced-renderer")]
                frame.begin_presentation_capture();
            }
            old.force_polygon=true;
            assert_eq!(direct.triangle(&camera,&t,&palette,&textures,true),old.triangle(&camera,&t,&palette,&textures,true));
            assert_eq!(direct.pixels,old.pixels);assert_eq!(direct.depth,old.depth);
            assert_eq!(direct.tags,old.tags);assert_eq!(direct.views,old.views);
            assert_eq!(direct.ground_tags,old.ground_tags);assert_eq!(direct.ground_views,old.ground_views);
            assert_eq!(direct.ground_depth,old.ground_depth);assert_eq!(direct.ground_faces,old.ground_faces);
            assert_eq!(direct.triangles,old.triangles);assert_eq!(direct.legacy_aborted,old.legacy_aborted);
            drawn+=direct.triangles;
            #[cfg(feature="enhanced-renderer")]
            {
                let a=direct.take_presentation_geometry()?.unwrap();let b=old.take_presentation_geometry()?.unwrap();
                assert_eq!(a.len(),b.len());
                for (a,b) in a.iter().zip(&b) {
                    assert_eq!((a.vertices,a.colors,a.uv,a.alpha,a.tag,a.view,a.flat,a.texture,a.material_hsl,a.depth_bias),
                        (b.vertices,b.colors,b.uv,b.alpha,b.tag,b.view,b.flat,b.texture,b.material_hsl,b.depth_bias));
                    assert_eq!(a.local_vertices,b.local_vertices);assert_eq!(a.model_bounds,b.model_bounds);
                    assert_eq!(a.near_alpha,b.near_alpha);
                    assert_eq!(format!("{:?}",a.texture_mapping),format!("{:?}",b.texture_mapping));
                }
            }
        }}}}}}}}
        assert!(drawn>0,"the differential must include rasterized faces");
        Ok(())
    }
}
