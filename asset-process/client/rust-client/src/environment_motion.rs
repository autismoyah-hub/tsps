//! Explicit authored rigid motion, separate from actor and skeletal clocks.
use crate::{model_bounds::Bounds, render3d::Triangle, Error, Result};
use std::{collections::HashMap, time::Instant};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Mode {
    #[default]
    Off = 0,
    Machinery = 1,
    Water = 2,
    Furnace = 3,
    All = 4,
    Reduced = 5,
}
impl std::str::FromStr for Mode {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self> {
        match value {
            "off" => Ok(Self::Off),
            "machinery" => Ok(Self::Machinery),
            "water" => Ok(Self::Water),
            "furnace" => Ok(Self::Furnace),
            "all" => Ok(Self::All),
            "reduced" => Ok(Self::Reduced),
            _ => Err(Error::Invalid("environment motion mode")),
        }
    }
}
struct Face {
    index: usize,
    bind: Triangle,
    kind: u32,
    pivot: [f32; 3],
    group: usize,
}
struct Group {
    kind: u32,
    position: [i32; 3],
    selectable: bool,
    bounds: std::cell::Cell<Option<Bounds>>,
}
pub struct Motion {
    faces: Vec<Face>,
    groups: Vec<Group>,
    last: Instant,
    time: f64,
    mode: Mode,
    active: bool,
    profile: Option<[[u8; 32]; 2]>,
}
impl Motion {
    pub fn empty() -> Self {
        Self {
            faces: Vec::new(),
            groups: Vec::new(),
            last: Instant::now(),
            time: 0.,
            mode: Mode::Off,
            active: true,
            profile: None,
        }
    }
    pub fn bind(
        triangles: &[Triangle],
        mode: Mode,
        profile: [[u8; 32]; 2],
        allowed: impl Fn(&Triangle) -> bool,
    ) -> Result<Self> {
        let mut out = Self::empty();
        out.mode = mode;
        out.profile = Some(profile);
        let mut groups = HashMap::<usize, usize>::new();
        for (index, t) in triangles.iter().enumerate() {
            let id = (t.tag >> 32).wrapping_sub(1) as u32;
            if t.view != 0 || !(64950..=64956).contains(&id) || !allowed(t) {
                continue;
            }
            if out.faces.len() >= 200_000 {
                return Err(Error::Invalid("environment motion face limit"));
            }
            let bounds = t
                .model_bounds
                .ok_or(Error::Invalid("environment motion model bounds"))?;
            let batch = t
                .model_order
                .ok_or(Error::Invalid("environment motion scene submission"))?
                .batch;
            if bounds.angle != 0 {
                return Err(Error::Invalid(
                    "environment motion requires unrotated shape10",
                ));
            }
            let mut x = ((t.tag >> 14) & 16383) as f32 * 128. + 64.;
            let mut z = (t.tag & 16383) as f32 * 128. + 64.;
            if id == 64952 {
                x += 87.;
                z -= 128.;
            }
            if id == 64950 || id == 64956 {
                x -= 133.;
            }
            let pivot = [
                x,
                if id == 64950 || id == 64956 {
                    -328.
                } else {
                    -360.
                },
                z,
            ];
            if t.vertices
                .iter()
                .flatten()
                .any(|v| !v.is_finite() || v.abs() > 1_000_000.)
            {
                return Err(Error::Invalid("environment motion vertex bounds"));
            }
            let group = if let Some(&group) = groups.get(&batch) {
                if out.groups[group].position != bounds.position || out.groups[group].kind != id {
                    return Err(Error::Invalid("environment motion mixed draw origins"));
                }
                group
            } else {
                let group = out.groups.len();
                out.groups.push(Group {
                    kind: id,
                    position: bounds.position,
                    selectable: bounds.selection.is_some(),
                    bounds: std::cell::Cell::new(None),
                });
                groups.insert(batch, group);
                group
            };
            out.faces.push(Face {
                index,
                bind: *t,
                kind: id,
                pivot,
                group,
            });
        }
        Ok(out)
    }
    pub fn indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.faces.iter().map(|f| f.index)
    }
    pub fn mode(&self) -> Mode {
        self.mode
    }
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
    /// Runtime controls may change only an explicitly admitted motion profile.
    pub fn set_admitted_mode(&mut self, mode: Mode) -> bool {
        if self.profile.is_none() {return false;}
        self.set_mode(mode);
        true
    }
    pub fn active(&mut self, active: bool) {
        if self.active != active {
            self.last = Instant::now();
            self.active = active;
        }
    }
    pub fn elapsed(&self) -> f64 {
        self.time
    }
    pub fn advance(&mut self, triangles: &mut [Triangle]) -> Result<()> {
        let now = Instant::now();
        let dt = now.duration_since(self.last);
        self.last = now;
        self.advance_by(dt, triangles)
    }
    pub fn advance_by(
        &mut self,
        delta: std::time::Duration,
        triangles: &mut [Triangle],
    ) -> Result<()> {
        let next = self.time + if self.active { delta.as_secs_f64() } else { 0. };
        self.pose_at(next, triangles)?;
        self.time = next;
        Ok(())
    }
    /// Reconstruct from immutable bind poses, without changing the independent clock.
    pub fn pose_at(&self, seconds: f64, triangles: &mut [Triangle]) -> Result<()> {
        if !seconds.is_finite()
            || seconds < 0.
            || seconds > 1e12
            || self.faces.iter().any(|f| f.index >= triangles.len())
        {
            return Err(Error::Invalid("environment motion sample"));
        }
        let mode = self.mode as u8;
        for group in &self.groups {
            group.bounds.set(None);
        }
        let theta = (seconds * 0.85) as f32;
        let (s, c) = theta.sin_cos();
        let pin = [12. * c + 17. * s, 12. * s - 17. * c];
        let slider = pin[0] + (1924. - (15. - pin[1]).powi(2)).sqrt();
        let slide = slider - 42.;
        for f in &self.faces {
            let mut t = f.bind;
            let enabled = mode == 4
                || mode == 5 && (f.kind <= 64952 || f.kind == 64956)
                || mode == 1 && (f.kind <= 64952 || f.kind == 64956)
                || mode == 2 && f.kind == 64953
                || mode == 3 && f.kind == 64954;
            if enabled {
                for (p, b) in t.vertices.iter_mut().zip(f.bind.vertices) {
                    let q = [b[0] - f.pivot[0], b[1] - f.pivot[1], b[2] - f.pivot[2]];
                    *p = match f.kind {
                        64950 => [
                            f.pivot[0] + q[0] * c - q[1] * s,
                            f.pivot[1] + q[0] * s + q[1] * c,
                            b[2],
                        ],
                        64952 => {
                            let (ss, cc) = (-theta * 52. / 35.).sin_cos();
                            [
                                f.pivot[0] + q[0] * cc - q[1] * ss,
                                f.pivot[1] + q[0] * ss + q[1] * cc,
                                b[2],
                            ]
                        }
                        64951 => [b[0] + slide, b[1], b[2]],
                        64956 => {
                            let angle = (15. - pin[1]).atan2(slider - pin[0]) - 32f32.atan2(30.);
                            let (ss, cc) = angle.sin_cos();
                            let q = [q[0] - 12., q[1] + 17.];
                            [
                                f.pivot[0] + pin[0] + q[0] * cc - q[1] * ss,
                                f.pivot[1] + pin[1] + q[0] * ss + q[1] * cc,
                                b[2],
                            ]
                        }
                        64953 => [
                            b[0],
                            b[1] + (theta * 2. + q[0] * 0.04).sin() * 1.5,
                            b[2] + (seconds as f32 * 0.2).fract() * 48.,
                        ],
                        64954 => [b[0], b[1] - (theta * 5.).sin() * 9., b[2]],
                        64955 => [
                            b[0] + (theta * 1.2).sin() * 8.,
                            b[1] - (seconds as f32 * 0.3).fract() * 70.,
                            b[2],
                        ],
                        _ => b,
                    };
                }
                let group = &self.groups[f.group];
                let local = t
                    .vertices
                    .map(|v| std::array::from_fn(|i| v[i] - group.position[i] as f32));
                t.local_vertices = Some(local);
                let bounds = Bounds::new(&local, group.position);
                let bounds = if group.selectable {
                    bounds.with_selection(&local, 0, false)
                } else {
                    bounds
                };
                group.bounds.set(Some(
                    group
                        .bounds
                        .get()
                        .map_or(bounds, |previous| previous.union(bounds).unwrap()),
                ));
                if f.kind == 64954 {
                    let v = ((theta * 7.).sin() * 8.) as i32;
                    t.colors = f
                        .bind
                        .colors
                        .map(|h| (h & !127) | ((h & 127) + v).clamp(2, 126));
                    #[cfg(feature="enhanced-renderer")]
                    { t.material_hsl=f.bind.material_hsl.map(|h| {
                        ((h as i32 & !127) | ((h as i32 & 127)+v).clamp(2,126)) as u16
                    }); }
                }
                if f.kind == 64955 {
                    t.alpha = 170;
                    t.near_alpha = Some(170);
                    if let Some(order) = &mut t.model_order {
                        order.has_alpha = true;
                    }
                }
            }
            triangles[f.index] = t;
        }
        for face in &self.faces {
            if let Some(bounds) = self.groups[face.group].bounds.get() {
                triangles[face.index].model_bounds = Some(bounds);
            }
        }
        Ok(())
    }
    pub fn retain(&mut self, old: &Self) {
        if self.profile == old.profile {
            self.time = old.time;
            self.last = old.last;
            self.active = old.active;
            self.mode = old.mode;
        }
    }
}
