//! Explicit optional offscreen Enhanced rendering.
//! Adapted from a31c18d src/enhanced.rs. No ambient environment selection,
//! No native activation or Classic picking mutation lives here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Classic,
    Lite,
    Gpu,
}
impl Mode {
    pub fn parse(v: &str) -> Option<Self> {
        match v {
            "classic" => Some(Self::Classic),
            "lite" => Some(Self::Lite),
            "gpu" => Some(Self::Gpu),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Lite => "lite",
            Self::Gpu => "gpu",
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    pub mode: Mode,
    pub scale: f32,
    pub shadows: u32,
    pub ao: u32,
    pub aa: bool,
    pub exposure: f32,
    pub texture_size: u32,
    pub adapter: Option<u32>,
    pub light: [f32; 3],
}
impl Settings {
    pub fn for_mode(mode: Mode) -> Self {
        Self {
            mode,
            scale: if mode == Mode::Lite { 0.75 } else { 1. },
            shadows: if mode == Mode::Gpu {
                2048
            } else if mode == Mode::Lite {
                512
            } else {
                0
            },
            ao: if mode == Mode::Gpu { 8 } else { 0 },
            aa: true,
            exposure: 1.15,
            texture_size: if mode == Mode::Lite { 64 } else { 128 },
            adapter: None,
            light: [-0.45, -0.8, -0.35],
        }
    }
    /// Explicit launch-time equivalents of the experimental environment controls.
    /// Parsing and validation finish before callers open content or connections.
    pub fn from_launch_args(mode: &str, adapter: &str, options: &[String]) -> Result<Self, String> {
        let mut settings=Self::for_mode(Mode::parse(mode).ok_or("Expected classic/lite/gpu")?);
        settings.adapter=if adapter=="auto" {None} else {Some(adapter.parse().map_err(|_|"Invalid adapter index")?)};
        let mut seen=std::collections::HashSet::new();
        for option in options {
            let (name,value)=option.split_once('=').ok_or("Renderer options require --name=value")?;
            if !seen.insert(name) {return Err(format!("Repeated renderer option: {name}"));}
            match name {
                "--scale"=>settings.scale=value.parse().map_err(|_|"Invalid render scale")?,
                "--shadows"=>settings.shadows=value.parse().map_err(|_|"Invalid shadow resolution")?,
                "--ao"=>settings.ao=value.parse().map_err(|_|"Invalid AO sample count")?,
                "--aa"=>settings.aa=match value {"on"=>true,"off"=>false,_=>return Err("AA must be on/off".into())},
                "--exposure"=>settings.exposure=value.parse().map_err(|_|"Invalid exposure")?,
                "--texture-size"=>settings.texture_size=value.parse().map_err(|_|"Invalid texture size")?,
                "--light"=>{
                    let values=value.split(',').map(str::parse::<f32>).collect::<Result<Vec<_>,_>>().map_err(|_|"Invalid light direction")?;
                    settings.light=values.try_into().map_err(|_|"Light needs three comma-separated components")?;
                }
                _=>return Err("Unknown renderer option".into()),
            }
        }
        settings.validate()?;
        Ok(settings)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.scale.is_finite() || !(0.25..=1.5).contains(&self.scale) {
            return Err("scale outside 0.25..1.5".into());
        }
        if ![0, 256, 512, 1024, 2048, 4096].contains(&self.shadows) {
            return Err("invalid shadow resolution".into());
        }
        if ![64, 128].contains(&self.texture_size)
            || self.light.iter().any(|v| !v.is_finite())
            || !self.light.iter().map(|v| v * v).sum::<f32>().is_finite()
            || self.light.iter().map(|v| v * v).sum::<f32>() < 0.001
        {
            return Err("invalid texture size/light direction".into());
        }
        if self.ao > 16 || !self.exposure.is_finite() || !(0.1..=4.).contains(&self.exposure) {
            return Err("invalid AO/exposure".into());
        }
        Ok(())
    }
}

impl Default for Mode {
    fn default() -> Self { Self::Classic }
}
impl Default for Settings {
    fn default() -> Self { Self::for_mode(Mode::Classic) }
}

use crate::{render3d::{Camera, Triangle}, texture::Texture};
use std::{collections::HashMap, time::Instant};
/// Retain only an opaque root-scenery prefix, without regrouping draw order.
/// Keep every (tag, view) together so smooth normals cannot cross the split.
/// Callers still use content hashing: admitted/clipped geometry can change.
pub fn opaque_scenery_prefix(triangles: &[&Triangle], scenery: &std::collections::HashSet<(u64,u16)>) -> usize {
    // Root terrain uses the native high-bit marker; object identities come from
    // placements, avoiding a scan of every triangle in the full world mesh.
    let eligible=triangles.iter().take_while(|t|t.view==0 && t.alpha==0
        && (t.tag&(1u64<<63)!=0 || scenery.contains(&(t.tag,t.view)))).count();
    if eligible==0 {return 0;}
    let mut last=HashMap::new();
    for (i,t) in triangles[..eligible].iter().enumerate() {last.insert((t.tag,t.view),i);}
    for (i,t) in triangles.iter().enumerate().skip(eligible) {
        if let Some(end)=last.get_mut(&(t.tag,t.view)) {*end=i;}
    }
    let mut through=0;let mut split=0;
    for (i,t) in triangles[..eligible].iter().enumerate() {
        through=through.max(last[&(t.tag,t.view)]);
        if through==i {split=i+1;}
    }
    split
}
#[cfg(windows)]
#[path="enhanced/d3d11.rs"]
mod d3d11;
/// Preserve caller roof/cutaway filtering; exclude non-root and non-opaque faces.
pub fn root_shadow_casters<'a>(triangles:impl IntoIterator<Item=&'a Triangle>)->Vec<&'a Triangle> {
    triangles.into_iter().filter(|t|t.view==0 && t.alpha==0 && t.colors[0]!=12345678).collect()
}
/// Prepare shadow-only copies through the same child-to-root transform used by
/// native scene submission. Caller applies child admission and roof/cutaway policy.
/// Original local vertices, scene ownership and picking metadata are not mutated.
pub fn append_transformed_shadow_casters<'a>(out:&mut Vec<Triangle>,triangles:impl IntoIterator<Item=&'a Triangle>,
    transform:&crate::world_entity::SceneTransform,focus:[f32;3],light:[f32;3])->Result<(),String> {
    let length_squared=dot(light,light);
    if focus.iter().chain(light.iter()).any(|v|!v.is_finite()) || !length_squared.is_finite() || length_squared<0.001 {
        return Err("invalid transformed shadow volume".into());
    }
    let (light,right,down)=light_basis(light);
    for (index,t) in triangles.into_iter().enumerate() {
        if index>=1_000_000 {return Err("transformed shadow source exceeds bound".into());}
        if t.alpha!=0 || t.colors[0]==12345678 {continue;}
        let vertices=t.vertices.map(|point|transform.point(point));
        if vertices.iter().flatten().any(|v|!v.is_finite()) {return Err("nonfinite transformed shadow geometry".into());}
        if !intersects_shadow_volume(vertices,focus,[right,down,light]) {continue;}
        if out.len()>=1_000_000 {return Err("transformed shadow output exceeds bound".into());}
        let mut copy=*t;copy.vertices=vertices;copy.view=0;out.push(copy);
    }
    Ok(())
}
/// Geometry has already passed the real client's model/actor/terrain transforms.
/// Optional albedo overrides allow an exact original-asset benchmark.
#[derive(Clone, Copy)]
pub struct SceneFrame<'a> {
    pub camera: &'a Camera,
    pub triangles: &'a [&'a Triangle],
    /// First N triangles are a stable source generation. None uses content hashing.
    pub stable_count: usize,
    pub stable_revision: Option<u64>,
    pub albedo: Option<&'a [[u16; 3]]>,
    pub palette: &'a [u32],
    pub textures: &'a HashMap<u16, Texture>,
    pub size: [u32; 2],
    pub light_focus: [f32; 3],
    pub ambient: i32,
    pub contrast: i32,
}
#[derive(Default, Clone, Debug)]
pub struct Stats {
    pub frames: u64,
    pub triangles: usize,
    pub extra_shadow_triangles: usize,
    pub draw_calls: u32,
    pub prepare_ms: f64,
    pub render_ms: f64,
    pub readback_ms: f64,
    pub gpu_ms: Option<f64>,
    pub buffer_bytes: u64,
    pub retained_prefix_bytes: u64,
    pub prefix_cache_hits: u64,
    pub target_bytes: u64,
    pub texture_bytes: u64,
    pub uploads: u64,
    pub init_ms: f64,
    pub adapter: String,
}
impl Stats {
    pub fn json(&self) -> String {
        format!("{{\"frames\":{},\"triangles\":{},\"extra_shadow_triangles\":{},\"draw_calls\":{},\"prepare_ms\":{:0.4},\"render_ms\":{:0.4},\"readback_ms\":{:0.4},\"gpu_ms\":{},\"buffer_bytes\":{},\"retained_prefix_bytes\":{},\"prefix_cache_hits\":{},\"target_bytes\":{},\"texture_bytes\":{},\"uploads\":{},\"init_ms\":{:0.4},\"adapter\":{}}}",self.frames,self.triangles,self.extra_shadow_triangles,self.draw_calls,self.prepare_ms,self.render_ms,self.readback_ms,self.gpu_ms.map_or("null".into(),|v|format!("{v:0.4}")),self.buffer_bytes,self.retained_prefix_bytes,self.prefix_cache_hits,self.target_bytes,self.texture_bytes,self.uploads,self.init_ms,json_string(&self.adapter))
    }
}
/// D3D11 timestamp results are usable only when all three HRESULTs are S_OK.
#[cfg(any(windows,test))]
fn timestamp_ms(status:[i32;3],frequency:u64,disjoint:bool,start:u64,end:u64)->Option<f64> {
    if status!=[0;3] || frequency==0 || disjoint || end<start {return None;}
    Some((end-start)as f64/frequency as f64*1000.)
}
#[cfg(test)]
mod timestamp_tests {
    use super::timestamp_ms;
    #[test]
    fn pending_failed_disjoint_and_invalid_queries_are_absent() {
        assert_eq!(timestamp_ms([0;3],1000,false,10,25),Some(15.));
        assert_eq!(timestamp_ms([0;3],1000,false,10,10),Some(0.));
        for slot in 0..3 {for result in [1,-1,i32::MIN,2] {
            let mut status=[0;3];status[slot]=result;
            assert_eq!(timestamp_ms(status,1000,false,0,0),None);
            assert_eq!(timestamp_ms(status,1000,false,10,25),None);
        }}
        assert_eq!(timestamp_ms([0;3],0,false,10,25),None);
        assert_eq!(timestamp_ms([0;3],1000,true,10,25),None);
        assert_eq!(timestamp_ms([0;3],1000,false,25,10),None);
        assert!(timestamp_ms([0;3],1,false,0,u64::MAX).unwrap().is_finite());
    }
}
fn json_string(value:&str)->String {
    use std::fmt::Write;
    let mut out=String::from("\"");
    for ch in value.chars() {
        match ch {
            '"'=>out.push_str("\\\""),
            '\\'=>out.push_str("\\\\"),
            ch if ch<'\u{20}'=>{write!(&mut out,"\\u{:04x}",ch as u32).unwrap();},
            ch=>out.push(ch),
        }
    }
    out.push('"');out
}
pub struct Renderer {
    settings: Settings,
    pub stats: Stats,
    #[cfg(windows)]
    gpu: Option<d3d11::Gpu>,
    pub fallback: Option<String>,
}
impl Renderer {
    pub fn new(settings: Settings) -> Result<Self, String> {
        settings.validate()?;
        Ok(Self {
            settings,
            stats: Stats::default(),
            #[cfg(windows)]
            gpu: None,
            fallback: None,
        })
    }
    pub fn enabled(&self) -> bool {
        self.settings.mode != Mode::Classic && self.fallback.is_none()
    }
    pub fn settings(&self) -> &Settings { &self.settings }
    /// Reconfiguration is explicit and recreates device-owned resources.
    pub fn configure(&mut self, settings: Settings) -> Result<(), String> {
        settings.validate()?;
        if settings != self.settings { self.settings = settings; self.reset(); }
        Ok(())
    }
    pub fn render(&mut self, input: SceneFrame<'_>) -> Result<Vec<u32>, String> {
        self.render_with_background(input, None)
    }
    /// Optional completed Classic sky pixels, before world geometry or UI.
    /// The background is display-only and never enters depth, IDs or shadows.
    pub fn render_with_background(&mut self, input: SceneFrame<'_>, background: Option<&[u32]>) -> Result<Vec<u32>, String> {
        self.render_with_shadow_casters(input,background,&[])
    }
    /// Additional root-space geometry affects only shadows, never color depth or IDs.
    /// Callers retain roof/cutaway admission before submitting offscreen casters.
    pub fn render_with_shadow_casters(&mut self,input:SceneFrame<'_>,background:Option<&[u32]>,casters:&[&Triangle])->Result<Vec<u32>,String> {
        input.validate()?;
        if input.triangles.len().saturating_add(casters.len())>1_000_000 || casters.iter().any(|t|t.view!=0) {
            return Err("invalid bounded root shadow caster contract".into());
        }
        SceneFrame{triangles:casters,stable_count:0,stable_revision:None,albedo:None,..input}.validate()?;
        if background.is_some_and(|pixels| pixels.len() != input.size[0] as usize * input.size[1] as usize) {
            return Err("Enhanced background dimensions differ".into());
        }
        if !self.enabled() {
            return Err("Classic selected or enhanced backend disabled".into());
        }
        #[cfg(windows)]
        {
            if self.gpu.is_none() {
                let start = Instant::now();
                self.gpu = Some(d3d11::Gpu::new(&self.settings)?);
                self.stats.init_ms = start.elapsed().as_secs_f64() * 1000.;
                self.stats.adapter = self.gpu.as_ref().unwrap().identity.clone();
            }
            self.gpu
                .as_mut()
                .unwrap()
                .render(&input, &self.settings, &mut self.stats, background, casters)
        }
        #[cfg(not(windows))]
        {
            let _ = input;
            Err("Experimental GPU backend currently requires Windows".into())
        }
    }
    /// Explicit device recreation test hook; no world state or packets change.
    pub fn reset(&mut self) {
        #[cfg(windows)]
        {
            self.gpu = None;
        }
        self.fallback = None;
    }
    pub fn disable(&mut self, reason: String) {
        eprintln!("Enhanced renderer -> Classic fallback: {reason}");
        self.fallback = Some(reason);
        #[cfg(windows)]
        {
            self.gpu = None;
        }
    }
    /// Diagnostic GPU identities only; never writes a Classic Frame.
    #[cfg(windows)]
    pub fn diagnostic_ids(&self) -> Option<DiagnosticIds> {
        self.gpu.as_ref().map(|gpu| gpu.diagnostic_ids())
    }
    #[cfg(windows)]
    pub fn adapters() -> Result<Vec<Adapter>, String> {
        d3d11::adapters()
    }
}

pub(crate) fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
pub(crate) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
pub(crate) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub(crate) fn normal(a: [f32; 3]) -> [f32; 3] {
    let d = dot(a, a).sqrt().max(1e-12);
    a.map(|x| x / d)
}

/// Native-resolution GPU identifiers, for backend comparison only.
pub struct DiagnosticIds {
    pub size: [u32; 2],
    pub values: Vec<(u64, u16)>,
}
impl SceneFrame<'_> {
    pub fn validate(&self) -> Result<(), String> {
        if self.size.contains(&0) || self.size.iter().any(|&v| v > 16384)
            || self.size[0] as u64 * self.size[1] as u64 > 8_294_400
            || self.triangles.len() > 1_000_000 || self.stable_count > self.triangles.len()
            || self.albedo.is_some_and(|a| a.len() != self.triangles.len())
            || self.textures.len() > 511 {
            return Err("invalid bounded scene contract".into());
        }
        let c = self.camera;
        if c.position.iter().chain(c.screen_offset.iter()).chain(self.light_focus.iter()).any(|v| !v.is_finite())
            || !c.focal.is_finite() || c.focal <= 0. || !c.near.is_finite() || c.near <= 0.
            || c.far as f32 <= c.near || c.orthographic.is_some()
            || [[0.,0.,0.], [1.,0.,0.], [0.,1.,0.], [0.,0.,1.]].iter().any(|&p| c.transform(p).iter().any(|v| !v.is_finite())) {
            return Err("unsupported or nonfinite Enhanced camera".into());
        }
        if self.triangles.iter().any(|t| t.vertices.iter().flatten().chain(t.uv.iter().flatten()).any(|v| !v.is_finite())) {
            return Err("nonfinite Enhanced geometry".into());
        }
        if self.textures.values().any(|t| ![64,128].contains(&t.size) || t.pixels.len() != t.size * t.size) {
            return Err("invalid Enhanced texture storage".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Adapter {
    pub index: u32,
    pub name: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub dedicated_bytes: usize,
    pub shared_bytes: usize,
    pub software: bool,
    pub d3d11: bool,
    pub feature_level: i32,
}
pub(crate) fn light_basis(light: [f32; 3]) -> ([f32; 3], [f32; 3], [f32; 3]) {
    let light = normal(light);
    let up = if light[1].abs() > 0.99 { [1., 0., 0.] } else { [0., -1., 0.] };
    let right = normal(cross(light, up));
    (light, right, normal(cross(light, right)))
}
/// Conservative clipping against shadowVS's orthographic volume. A crossing
/// triangle stays admitted even when every vertex is outside a different plane.
/// The margin covers CPU/GPU arithmetic differences; uncertain extremes stay in.
pub(crate) fn intersects_shadow_volume(vertices:[[f32;3];3],focus:[f32;3],axes:[[f32;3];3])->bool {
    let mut projected=[[0.;3];3];let mut margin=[0.;3];
    for (i,p) in vertices.into_iter().enumerate() {
        let d=sub(p,focus);
        margin[i]=1.+16.*f32::EPSILON*(p.into_iter().map(f32::abs).sum::<f32>()+focus.into_iter().map(f32::abs).sum::<f32>());
        projected[i]=axes.map(|axis|dot(d,axis));
        if !margin[i].is_finite() || projected[i].iter().any(|v|!v.is_finite()) {return true;}
    }
    for (axis,bound) in [768.,768.,2048.].into_iter().enumerate() {
        if (0..3).all(|i|projected[i][axis]< -bound-margin[i])
            || (0..3).all(|i|projected[i][axis]>bound+margin[i]) {return false;}
    }
    true
}
#[cfg(test)]
mod basis_tests {
    use super::*;
    #[test]
    fn vertical_lights_keep_a_finite_orthonormal_shadow_basis() {
        for direction in [[0., 1., 0.], [0., -1., 0.], [-0.45,-0.8,-0.35]] {
            let (light, right, down) = light_basis(direction);
            for axis in [light,right,down] { assert!((dot(axis,axis)-1.).abs() < 1e-5); }
            for (a,b) in [(light,right),(light,down),(right,down)] { assert!(dot(a,b).abs() < 1e-5); }
        }
    }
    #[test]
    fn shadow_volume_rejects_only_wholly_outside_faces() {
        let axes=[[1.,0.,0.],[0.,1.,0.],[0.,0.,1.]];
        let focus=[100.,200.,300.];
        for axis in 0..3 {for sign in [-1.,1.] {
            let bound=if axis==2 {2048.}else{768.};
            let mut points=[focus;3];for p in &mut points {p[axis]+=sign*(bound+10.);}
            assert!(!intersects_shadow_volume(points,focus,axes));
            points[0]=focus;assert!(intersects_shadow_volume(points,focus,axes));
            for p in &mut points {*p=focus;p[axis]+=sign*(bound+0.5);}
            assert!(intersects_shadow_volume(points,focus,axes),"rounding margin stays admitted");
        }}
        assert!(intersects_shadow_volume([[-1000.,-1000.,0.],[1000.,-1000.,0.],[0.,1000.,0.]],[0.;3],axes));
        assert!(intersects_shadow_volume([[f32::MAX;3];3],[-f32::MAX;3],axes),"uncertain arithmetic is retained");
    }
}
