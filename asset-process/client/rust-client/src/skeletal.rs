//! Cached animation pose composition and weighted skeletal deformation.
use crate::{animation::Skeleton, cached_animation::CachedAnimation, model::Model, Error, Result};
pub type Matrix = [f32; 16];
fn identity() -> Matrix {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}
fn multiply(a: Matrix, b: Matrix) -> Matrix {
    std::array::from_fn(|i| {
        let row = i % 4;
        let col = i / 4;
        (a[row] * b[col * 4] + a[row + 4] * b[col * 4 + 1])
            + a[row + 8] * b[col * 4 + 2]
            + a[row + 12] * b[col * 4 + 3]
    })
}
fn inverse(m: Matrix) -> Result<Matrix> {
    let mut rows = [[0.0f64; 8]; 4];
    for y in 0..4 {
        for x in 0..4 {
            rows[y][x] = m[x * 4 + y] as f64;
        }
        rows[y][y + 4] = 1.0;
    }
    for col in 0..4 {
        let pivot = (col..4)
            .max_by(|&a, &b| rows[a][col].abs().total_cmp(&rows[b][col].abs()))
            .unwrap();
        if rows[pivot][col].abs() < 1e-20 {
            return Err(Error::Invalid("singular skeletal matrix"));
        }
        rows.swap(col, pivot);
        let value = rows[col][col];
        for x in 0..8 {
            rows[col][x] /= value;
        }
        for y in 0..4 {
            if y != col {
                let scale = rows[y][col];
                for x in 0..8 {
                    rows[y][x] -= scale * rows[col][x];
                }
            }
        }
    }
    Ok(std::array::from_fn(|i| rows[i % 4][i / 4 + 4] as f32))
}
fn rotations(inverse: Matrix) -> [f32; 3] {
    let x = -libm::asin(inverse[6] as f64) as f32;
    let (y, z) = if libm::cos(x as f64).abs() > 0.005 {
        (
            libm::atan2(inverse[2] as f64, inverse[10] as f64) as f32,
            libm::atan2(inverse[4] as f64, inverse[5] as f64) as f32,
        )
    } else {
        let y = libm::atan2(inverse[1] as f64, inverse[0] as f64) as f32;
        (if inverse[6] < 0.0 { y } else { -y }, 0.0)
    };
    [x, y, z]
}
fn quaternion_product(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        b[1] * a[2] + (a[3] * b[0] + a[0] * b[3]) - a[1] * b[2],
        a[0] * b[2] + (a[1] * b[3] - b[0] * a[2] + b[1] * a[3]),
        a[3] * b[2] + (a[1] * b[0] + a[2] * b[3] - b[1] * a[0]),
        b[3] * a[3] - a[0] * b[0] - b[1] * a[1] - a[2] * b[2],
    ]
}
fn compose(rotation: [f32; 3], translation: [f32; 3], scale: [f32; 3]) -> Matrix {
    let mut q = [0.0, 0.0, 0.0, 1.0];
    for axis in [2, 0, 1] {
        let half = rotation[axis] * 0.5;
        let mut r = [0.0; 4];
        r[axis] = libm::sin(half as f64) as f32;
        r[3] = libm::cos(half as f64) as f32;
        q = quaternion_product(q, r);
    }
    let [x, y, z, w] = q;
    let (ww, wx, wy, wz, xx, xy, xz, yy, yz, zz) = (
        w * w,
        w * x,
        w * y,
        w * z,
        x * x,
        y * x,
        x * z,
        y * y,
        y * z,
        z * z,
    );
    let mut m = identity();
    m[0] = ww + xx - zz - yy;
    m[1] = wz + (xy + wz + xy);
    m[2] = xz + (xz - wy - wy);
    m[4] = xy + (xy - wz - wz);
    m[5] = yy + ww - xx - zz;
    m[6] = yz + (wx + yz) + wx;
    m[8] = wy + xz + xz + wy;
    m[9] = yz - wx - wx + yz;
    m[10] = zz + ww - yy - xx;
    for col in 0..3 {
        for row in 0..3 {
            m[col * 4 + row] *= scale[row];
        }
    }
    m[12..15].copy_from_slice(&translation);
    m
}
fn global(
    index: usize,
    local: &[Matrix],
    skeleton: &Skeleton,
    cache: &mut [Option<Matrix>],
    active: &mut [bool],
) -> Result<Matrix> {
    if let Some(m) = cache[index] {
        return Ok(m);
    }
    if active[index] {
        return Err(Error::Invalid("skeletal parent cycle"));
    }
    active[index] = true;
    let parent = skeleton.bones[index].parent;
    let m = if parent >= 0 {
        let p = parent as usize;
        if p >= local.len() {
            return Err(Error::Invalid("skeletal parent index"));
        }
        multiply(global(p, local, skeleton, cache, active)?, local[index])
    } else {
        local[index]
    };
    active[index] = false;
    cache[index] = Some(m);
    Ok(m)
}
pub struct Rig {
    pub pose: u8,
    base: Vec<Matrix>,
    inverse_bind: Vec<Matrix>,
    rotation: Vec<[f32; 3]>,
    scale: Vec<[f32; 3]>,
}
impl Rig {
    pub fn new(skeleton: &Skeleton, pose: u8) -> Result<Self> {
        if pose >= skeleton.poses {
            return Err(Error::Invalid("rig pose"));
        }
        let base = skeleton
            .bones
            .iter()
            .map(|b| b.matrices[pose as usize])
            .collect::<Vec<_>>();
        let (mut cache, mut active) = (vec![None; base.len()], vec![false; base.len()]);
        let mut inverse_bind = Vec::new();
        let mut rotation = Vec::new();
        let mut scale = Vec::new();
        for (i, &m) in base.iter().enumerate() {
            inverse_bind.push(inverse(global(
                i,
                &base,
                skeleton,
                &mut cache,
                &mut active,
            )?)?);
            rotation.push(rotations(inverse(m)?));
            scale.push(std::array::from_fn(|i| {
                ((m[i * 4] * m[i * 4] + m[i * 4 + 1] * m[i * 4 + 1] + m[i * 4 + 2] * m[i * 4 + 2])
                    as f64)
                    .sqrt() as f32
            }));
        }
        Ok(Self {
            pose,
            base,
            inverse_bind,
            rotation,
            scale,
        })
    }
    pub fn matrices(
        &self,
        skeleton: &Skeleton,
        animation: &CachedAnimation,
        time: f32,
    ) -> Result<Vec<Matrix>> {
        if animation.skeleton != skeleton.id {
            return Err(Error::Invalid("rig animation mismatch"));
        }
        self.from_local(skeleton, &self.local_matrices(skeleton, animation, time)?)
    }
    pub fn local_matrices(
        &self,
        skeleton: &Skeleton,
        animation: &CachedAnimation,
        time: f32,
    ) -> Result<Vec<Matrix>> {
        if animation.pose != self.pose || self.base.len() != skeleton.bones.len() {
            return Err(Error::Invalid("rig animation mismatch"));
        }
        let mut rotation = self.rotation.clone();
        let mut scale = self.scale.clone();
        let mut translation = self
            .base
            .iter()
            .map(|m| [m[12], m[13], m[14]])
            .collect::<Vec<_>>();
        for c in &animation.channels {
            if c.kind == 1 {
                let values = match c.component {
                    0..=2 => &mut rotation,
                    3..=5 => &mut translation,
                    _ => &mut scale,
                };
                values
                    .get_mut(c.target)
                    .ok_or(Error::Invalid("rig animation channel target"))?[c.component % 3] =
                    c.curve.evaluate(time);
            }
        }
        Ok((0..self.base.len())
            .map(|i| compose(rotation[i], translation[i], scale[i]))
            .collect::<Vec<_>>())
    }
    /// GameWorldEntity.ae uses the animated local matrix of bone zero,
    /// with the scene's vertical translation convention.
    pub fn scene_matrix(
        &self,
        skeleton: &Skeleton,
        animation: &CachedAnimation,
        time: f32,
    ) -> Result<Option<Matrix>> {
        let mut matrix = self
            .local_matrices(skeleton, animation, time)?
            .into_iter()
            .next();
        if let Some(matrix) = &mut matrix {
            matrix[13] = -matrix[13];
        }
        Ok(matrix)
    }
    pub fn from_local(&self, skeleton: &Skeleton, local: &[Matrix]) -> Result<Vec<Matrix>> {
        if local.len() != self.base.len() {
            return Err(Error::Invalid("rig local pose length"));
        }
        let (mut cache, mut active) = (vec![None; local.len()], vec![false; local.len()]);
        (0..local.len())
            .map(|i| {
                Ok(multiply(
                    global(i, &local, skeleton, &mut cache, &mut active)?,
                    self.inverse_bind[i],
                ))
            })
            .collect()
    }
}
impl Model {
    pub fn skin(&mut self, matrices: &[Matrix]) {
        if let Some(weights) = &self.bone_weights {
            for (i, weights) in weights.iter().enumerate() {
                if weights.is_empty() {
                    continue;
                }
                let mut m = [0.0; 16];
                for &(bone, weight) in weights {
                    if let Some(matrix) = matrices.get(bone as usize) {
                        let w = weight as f32 / 255.0;
                        for j in 0..16 {
                            m[j] += matrix[j] * w;
                        }
                    }
                }
                let [x, y, z] = self.vertices[i];
                let (y, z) = (-y, -z);
                self.vertices[i] = [
                    m[0] * x + m[4] * y + m[8] * z + m[12],
                    -(m[1] * x + m[5] * y + m[9] * z + m[13]),
                    -(m[2] * x + m[6] * y + m[10] * z + m[14]),
                ];
            }
        }
    }
    pub fn animate_cached(
        &mut self,
        skeleton: &Skeleton,
        rig: &Rig,
        animation: &CachedAnimation,
        time: f32,
    ) -> Result<()> {
        let matrices = rig.matrices(skeleton, animation, time)?;
        self.skin(&matrices);
        self.animate_cached_alpha(skeleton, animation, time)
    }
    pub fn animate_cached_alpha(
        &mut self,
        skeleton: &Skeleton,
        animation: &CachedAnimation,
        time: f32,
    ) -> Result<()> {
        if animation.has_alpha {
            if self.alpha.is_none() {
                self.alpha = Some(vec![0; self.faces.len()]);
            }
            if let Some(skins) = &self.face_skins {
                for c in &animation.channels {
                    if c.kind == 1 || skeleton.transform_types[c.target] != 5 {
                        continue;
                    }
                    let change = c.curve.evaluate(time) * 255.0;
                    let labels = &skeleton.labels[c.target];
                    let alpha = self.alpha.as_mut().unwrap();
                    for label in labels {
                        for (i, s) in skins.iter().enumerate() {
                            if s == label {
                                alpha[i] = ((alpha[i] as f32 + change) as i32).clamp(0, 255) as u8;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
