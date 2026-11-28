//! Vertex normals and face lighting from GameModelData.cw/be.
use crate::{model::Model, Error, Result};
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Normal {
    pub vector: [i32; 3],
    pub count: i32,
}
pub struct Normals {
    pub vertices: Vec<Normal>,
    pub faces: Vec<[i32; 3]>,
    pub merged: Vec<Option<Normal>>,
    pub hidden: Vec<bool>,
}
impl Model {
    pub fn mirror(&mut self) {
        for v in &mut self.vertices {
            v[2] = (v[2] as i32).wrapping_neg() as f32;
        }
        for f in &mut self.faces {
            f.swap(0, 2);
        }
    }
    pub fn rotate_quarter(&mut self, turns: u8) {
        for v in &mut self.vertices {
            let [x, y, z] = *v;
            *v = match turns & 3 {
                0 => [x, y, z],
                1 => [z, y, -x],
                2 => [-x, y, -z],
                _ => [-z, y, x],
            };
        }
    }
    pub fn rotate(&mut self, angle: u16) -> Result<()> {
        if angle >= 2048 {
            return Err(Error::Invalid("model angle"));
        }
        let radians = angle as f64 * 0.0030679615;
        let sin = (radians.sin() * 65536.0) as i32;
        let cos = (radians.cos() * 65536.0) as i32;
        for v in &mut self.vertices {
            let (x, z) = (v[0] as i32, v[2] as i32);
            v[0] = (z.wrapping_mul(sin).wrapping_add(x.wrapping_mul(cos)) >> 16) as f32;
            v[2] = (z.wrapping_mul(cos).wrapping_sub(x.wrapping_mul(sin)) >> 16) as f32;
        }
        Ok(())
    }
    pub fn scale(&mut self, scale: [i32; 3]) {
        for v in &mut self.vertices {
            for i in 0..3 {
                v[i] = scale[i] as f32 * v[i] / 128.0;
            }
        }
    }
    pub fn translate(&mut self, offset: [i32; 3]) {
        for v in &mut self.vertices {
            for i in 0..3 {
                v[i] += offset[i] as f32;
            }
        }
    }
    pub fn recolor(&mut self, from: u16, to: u16) {
        for c in &mut self.colors {
            if *c == from {
                *c = to;
            }
        }
    }
    pub fn retexture(&mut self, from: u16, to: u16) {
        if let Some(ts) = &mut self.face_textures {
            for t in ts {
                if *t == from as i16 {
                    *t = to as i16;
                }
            }
        }
    }
    pub fn normals(&self) -> Normals {
        let mut result = Normals {
            vertices: vec![Normal::default(); self.vertices.len()],
            faces: vec![[0; 3]; self.faces.len()],
            merged: Vec::new(),
            hidden: Vec::new(),
        };
        for (i, face) in self.faces.iter().enumerate() {
            let a = self.vertices[face[0] as usize].map(|v| v as i32);
            let b = self.vertices[face[1] as usize].map(|v| v as i32);
            let c = self.vertices[face[2] as usize].map(|v| v as i32);
            let d: [i32; 3] = std::array::from_fn(|i| b[i].wrapping_sub(a[i]));
            let e: [i32; 3] = std::array::from_fn(|i| c[i].wrapping_sub(a[i]));
            let mut n = [
                d[1].wrapping_mul(e[2])
                    .wrapping_sub(e[1].wrapping_mul(d[2])),
                d[2].wrapping_mul(e[0])
                    .wrapping_sub(e[2].wrapping_mul(d[0])),
                d[0].wrapping_mul(e[1])
                    .wrapping_sub(e[0].wrapping_mul(d[1])),
            ];
            while n.iter().any(|v| !(-8192..=8192).contains(v)) {
                n = n.map(|v| v >> 1);
            }
            let length = ((n.iter().map(|v| v * v).sum::<i32>() as f64).sqrt() as i32).max(1);
            n = n.map(|v| v * 256 / length);
            match self.render_types.as_ref().map_or(0, |t| t[i]) {
                0 => {
                    for &vertex in face {
                        let normal = &mut result.vertices[vertex as usize];
                        for j in 0..3 {
                            normal.vector[j] = normal.vector[j].wrapping_add(n[j]);
                        }
                        normal.count += 1;
                    }
                }
                1 => result.faces[i] = n,
                _ => {}
            }
        }
        result
    }
    pub fn light(&self, ambient: i32, contrast: i32, direction: [i32; 3]) -> Result<Vec<[i32; 3]>> {
        self.light_with_normals(ambient, contrast, direction, &self.normals())
    }
    pub fn light_with_normals(
        &self,
        ambient: i32,
        contrast: i32,
        direction: [i32; 3],
        normals: &Normals,
    ) -> Result<Vec<[i32; 3]>> {
        let length = (direction
            .iter()
            .fold(0i32, |s, &v| s.wrapping_add(v.wrapping_mul(v))) as f64)
            .sqrt() as i32;
        let denominator = contrast.wrapping_mul(length) >> 8;
        let dot =
            |n: [i32; 3]| (0..3).fold(0i32, |s, i| s.wrapping_add(n[i].wrapping_mul(direction[i])));
        let mut colors = vec![[0; 3]; self.faces.len()];
        for (i, face) in self.faces.iter().enumerate() {
            let mut kind = if normals.hidden.get(i).copied().unwrap_or(false) {
                2
            } else {
                self.render_types.as_ref().map_or(0, |t| t[i])
            };
            match self.alpha.as_ref().map_or(0, |a| a[i]) {
                254 => kind = 3,
                255 => kind = 2,
                _ => {}
            }
            let textured = self.face_textures.as_ref().is_some_and(|t| t[i] != -1);
            let shade = |brightness: i32| {
                if textured {
                    brightness.clamp(2, 126)
                } else {
                    (self.colors[i] as i32 & 65408)
                        + ((self.colors[i] as i32 & 127).wrapping_mul(brightness) >> 7)
                            .clamp(2, 126)
                }
            };
            if kind == 0 {
                for corner in 0..3 {
                    let vertex = face[corner] as usize;
                    let n = normals
                        .merged
                        .get(vertex)
                        .copied()
                        .flatten()
                        .unwrap_or(normals.vertices[vertex]);
                    let divisor = denominator.wrapping_mul(n.count);
                    if divisor == 0 {
                        return Err(Error::Invalid("zero vertex light divisor"));
                    }
                    colors[i][corner] =
                        shade(ambient.wrapping_add(dot(n.vector).wrapping_div(divisor)));
                }
            } else if kind == 1 {
                let divisor = denominator.wrapping_add(denominator / 2);
                if divisor == 0 {
                    return Err(Error::Invalid("zero face light divisor"));
                }
                colors[i][0] =
                    shade(ambient.wrapping_add(dot(normals.faces[i]).wrapping_div(divisor)));
                colors[i][2] = -1;
            } else if kind == 3 && !textured {
                colors[i][0] = 128;
                colors[i][2] = -1;
            } else {
                colors[i][2] = -2;
            }
        }
        Ok(colors)
    }
}

impl Normals {
    /// Merge only the original normals; repeated neighbors accumulate in the
    /// overlay without feeding earlier merges back into subsequent sums.
    pub fn merge(
        &mut self,
        model: &Model,
        other: &mut Self,
        other_model: &Model,
        offset: [i32; 3],
        hide_faces: bool,
    ) {
        let mut coordinates = std::collections::HashMap::<[i32; 3], Vec<usize>>::new();
        for (index, vertex) in other_model.vertices.iter().enumerate() {
            if other.vertices[index].count != 0 {
                coordinates
                    .entry(vertex.map(|v| v as i32))
                    .or_default()
                    .push(index);
            }
        }
        let mut left = vec![false; model.vertices.len()];
        let mut right = vec![false; other_model.vertices.len()];
        let mut matches = 0;
        for (index, vertex) in model.vertices.iter().enumerate() {
            if self.vertices[index].count == 0 {
                continue;
            }
            let key = std::array::from_fn(|axis| (vertex[axis] as i32).wrapping_sub(offset[axis]));
            let Some(neighbors) = coordinates.get(&key) else {
                continue;
            };
            for &neighbor in neighbors {
                let a = self.vertices[index];
                let b = other.vertices[neighbor];
                if self.merged.is_empty() {
                    self.merged.resize(self.vertices.len(), None);
                }
                if other.merged.is_empty() {
                    other.merged.resize(other.vertices.len(), None);
                }
                self.merged[index].get_or_insert(a).add(b);
                other.merged[neighbor].get_or_insert(b).add(a);
                left[index] = true;
                right[neighbor] = true;
                matches += 1;
            }
        }
        if hide_faces && matches >= 3 {
            self.hidden.resize(model.faces.len(), false);
            other.hidden.resize(other_model.faces.len(), false);
            for (index, face) in model.faces.iter().enumerate() {
                self.hidden[index] |= face.iter().all(|v| left[*v as usize]);
            }
            for (index, face) in other_model.faces.iter().enumerate() {
                other.hidden[index] |= face.iter().all(|v| right[*v as usize]);
            }
        }
    }
}
impl Normal {
    fn add(&mut self, other: Self) {
        for axis in 0..3 {
            self.vector[axis] = self.vector[axis].wrapping_add(other.vector[axis]);
        }
        self.count = self.count.wrapping_add(other.count);
    }
}
