//! Explicit upper-architecture cutaway, confined to the optional authored floor.
//! Lower walls/collision never change. Roof skin is a separate reference layer.
use crate::{Error, Result};
use std::{cell::Cell, collections::HashMap};
struct Group {
    key: [u32; 3],
    bounds: [[f32; 3]; 2],
    hidden: Cell<bool>,
}
#[derive(Default)]
pub struct Cutaway {
    groups: Vec<Group>,
    ranges: HashMap<u32, Vec<(usize, usize, usize)>>,
    faces: Vec<(usize, usize)>,
    owners: Vec<u32>,
    epoch: Cell<u64>,
}
impl Cutaway {
    pub fn parse(text: &str, offset: [f32; 2]) -> Result<Self> {
        if text.len() > 65536 || text.lines().next() != Some("CINDERWORKS_CUTAWAY 1") {
            return Err(Error::Invalid("architectural cutaway header/size"));
        }
        if offset.iter().any(|v| !v.is_finite() || v.abs() > 32768.) {
            return Err(Error::Invalid("cutaway scene offset"));
        }
        let mut out = Self::default();
        for line in text
            .lines()
            .skip(1)
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        {
            let a = line.split_whitespace().collect::<Vec<_>>();
            if a.len() != 10 || a[0] != "G" {
                return Err(Error::Invalid("architectural cutaway row"));
            }
            let id = a[1]
                .parse::<u32>()
                .map_err(|_| Error::Invalid("cutaway object ID"))?;
            let first = a[2]
                .parse::<usize>()
                .map_err(|_| Error::Invalid("cutaway face range"))?;
            let end = a[3]
                .parse::<usize>()
                .map_err(|_| Error::Invalid("cutaway face range"))?;
            let v = a[4..]
                .iter()
                .map(|v| {
                    v.parse::<f32>()
                        .map_err(|_| Error::Invalid("cutaway bounds"))
                })
                .collect::<Result<Vec<_>>>()?;
            if !(64920..64940).contains(&id)
                || first >= end
                || end > 20000
                || out.groups.len() >= 512
                || v.iter().any(|n| !n.is_finite() || n.abs() > 32768.)
                || (0..3).any(|i| v[i] > v[i + 3])
            {
                return Err(Error::Invalid("cutaway range/bounds limit"));
            }
            let group = out.groups.len();
            out.groups.push(Group {
                key: [id, first as u32, end as u32],
                bounds: [
                    [v[0] + offset[0], v[1], v[2] + offset[1]],
                    [v[3] + offset[0], v[4], v[5] + offset[1]],
                ],
                hidden: Cell::new(false),
            });
            let ranges = out.ranges.entry(id).or_default();
            if ranges.iter().any(|&(a, b, _)| first < b && a < end) {
                return Err(Error::Invalid("overlapping cutaway face ranges"));
            }
            ranges.push((first, end, group));
        }
        Ok(out)
    }
    pub fn bind_face(&mut self, id: u32, face: usize, triangle: usize) {
        if let Some(ranges) = self.ranges.get(&id) {
            if let Some(&(_, _, group)) = ranges.iter().find(|&&(a, b, _)| a <= face && face < b) {
                self.faces.push((triangle, group));
            }
        }
    }
    pub fn finish(&mut self, total: usize) -> Result<()> {
        if total > 16_000_000 {
            return Err(Error::Invalid("cutaway triangle limit"));
        }
        let mut seen = std::collections::HashSet::new();
        if self
            .faces
            .iter()
            .any(|&(face, _)| face >= total || !seen.insert(face))
        {
            return Err(Error::Invalid("cutaway face owner bounds/duplicate"));
        }
        if self.groups.is_empty() {
            return Ok(());
        }
        let mut owners = vec![u32::MAX; total];
        for &(face, group) in &self.faces {
            owners[face] = group as u32;
        }
        self.owners = owners;
        Ok(())
    }
    pub fn indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.faces.iter().map(|&(i, _)| i)
    }
    pub fn epoch(&self) -> u64 {
        self.epoch.get()
    }
    pub fn retain(&self, previous: &Self) {
        for (a, b) in self.groups.iter().zip(&previous.groups) {
            if a.key == b.key && a.bounds == b.bounds {
                a.hidden.set(b.hidden.get());
            }
        }
    }
    pub fn visible(&self, index: usize) -> bool {
        self.owners
            .get(index)
            .is_none_or(|&owner| owner == u32::MAX || !self.groups[owner as usize].hidden.get())
    }
    pub fn update(&self, eye: [f32; 3], focus: [f32; 3]) {
        // Protect the actor head/torso and a96unit interaction margin. A64unit
        // exit margin gives hysteresis, avoiding rapid edge toggling during orbit.
        let mut changed = false;
        for group in &self.groups {
            let margin = if group.hidden.get() { 64. } else { 24. };
            let mut obscures = false;
            for (dx, dz) in [(0., 0.), (-96., 0.), (96., 0.), (0., -96.), (0., 96.)] {
                for height in [100., 180.] {
                    let target = [focus[0] + dx, focus[1] - height, focus[2] + dz];
                    if intersects(eye, target, group.bounds, margin) {
                        obscures = true;
                        break;
                    }
                }
                if obscures {
                    break;
                }
            }
            if group.hidden.get() != obscures {
                group.hidden.set(obscures);
                changed = true;
            }
        }
        if changed {
            self.epoch.set(self.epoch.get().wrapping_add(1));
        }
    }
}
fn intersects(a: [f32; 3], b: [f32; 3], bounds: [[f32; 3]; 2], margin: f32) -> bool {
    let (mut lo, mut hi) = (0f32, 1f32);
    for axis in 0..3 {
        let delta = b[axis] - a[axis];
        let mn = bounds[0][axis] - margin;
        let mx = bounds[1][axis] + margin;
        if delta.abs() < 1e-6 {
            if a[axis] < mn || a[axis] > mx {
                return false;
            }
        } else {
            let x = (mn - a[axis]) / delta;
            let y = (mx - a[axis]) / delta;
            lo = lo.max(x.min(y));
            hi = hi.min(x.max(y));
            if lo > hi {
                return false;
            }
        }
    }
    hi > 0.001 && lo < 0.999
}

/// Immutable package-bound metadata; safe to share with background scene loading.
pub struct Metadata {
    identity: [u8; 32],
    package: [u8; 32],
    text: String,
}
impl Metadata {
    pub fn read(
        path: &std::path::Path,
        expected: [u8; 32],
        package: &crate::asset_package::Package,
    ) -> Result<Self> {
        use std::io::Read;
        let info = std::fs::symlink_metadata(path)?;
        if !info.is_file() || info.file_type().is_symlink() || info.len() > 65536 {
            return Err(Error::Invalid("cutaway metadata bounds/type"));
        }
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(65537)
            .read_to_end(&mut bytes)?;
        Self::decode(&bytes, expected, package)
    }
    pub fn decode(
        bytes: &[u8],
        expected: [u8; 32],
        package: &crate::asset_package::Package,
    ) -> Result<Self> {
        use sha2::{Digest, Sha256};
        if bytes.len() > 65536 || <[u8; 32]>::from(Sha256::digest(bytes)) != expected {
            return Err(Error::Invalid("cutaway metadata admission"));
        }
        let text =
            std::str::from_utf8(bytes).map_err(|_| Error::Invalid("cutaway metadata UTF8"))?;
        let parsed = Cutaway::parse(text, [0., 0.])?;
        for (&id, ranges) in &parsed.ranges {
            let def = crate::object::Definition::decode_local(
                id,
                package
                    .object(id)
                    .ok_or(Error::Invalid("cutaway object not owned"))?,
            )?;
            if !crate::environment_style::visual_only(&def, package)
                || def.models.len() != 1
                || def
                    .model_shapes
                    .as_ref()
                    .is_some_and(|s| s.as_slice() != [10])
            {
                return Err(Error::Invalid(
                    "cutaway requires a visual-only shape10 mesh",
                ));
            }
            let model = crate::model::Model::decode(
                package
                    .model(def.models[0])
                    .ok_or(Error::Invalid("cutaway model not owned"))?,
            )?;
            if ranges.iter().any(|&(_, end, _)| end > model.faces.len()) {
                return Err(Error::Invalid("cutaway range exceeds admitted mesh"));
            }
        }
        Ok(Self {
            identity: expected,
            package: package.identity(),
            text: text.to_owned(),
        })
    }
    pub fn identity(&self) -> [u8; 32] {
        self.identity
    }
    pub fn package_identity(&self) -> [u8; 32] {
        self.package
    }
    pub fn for_scene(&self, base: [i32; 2]) -> Result<Cutaway> {
        let origin = crate::authored_floor::ORIGIN;
        let offset = [
            (origin[0] as i64 - base[0] as i64) as f32 * 128.,
            (origin[1] as i64 - base[1] as i64) as f32 * 128.,
        ];
        Cutaway::parse(&self.text, offset)
    }
}
