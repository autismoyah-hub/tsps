//! Classic skeletal transforms, including RuneLite's floating-point mode.
use crate::{
    animation::{Frame, Skeleton},
    model::Model,
    Error, Result,
};
#[derive(Clone, Copy, Default)]
pub struct Context {
    pub pivot: [f32; 3],
    pub floating: bool,
}
impl Model {
    pub fn animate_classic(
        &mut self,
        skeleton: &Skeleton,
        frame: &Frame,
        floating: bool,
    ) -> Result<()> {
        self.animate_classic_mask(skeleton,frame,None,false,floating)
    }
    pub fn animate_classic_mask(&mut self,skeleton:&Skeleton,frame:&Frame,mask:Option<&[u32]>,selected:bool,floating:bool)->Result<()> {
        if skeleton.id != frame.skeleton {
            return Err(Error::Invalid("animation skeleton mismatch"));
        }
        if self.vertex_skins.is_none() {
            return Ok(());
        }
        if frame.has_alpha && self.alpha.is_none() {
            self.alpha = Some(vec![0; self.faces.len()]);
        }
        let mut context = Context {
            pivot: [0.0; 3],
            floating,
        };
        let mut mask_index=0;
        for t in &frame.transforms {
            if let Some(mask)=mask {
                while t.label as u32>*mask.get(mask_index).ok_or(Error::Invalid("animation interleave sentinel"))? {mask_index+=1;}
                let matches=t.label as u32==mask[mask_index];
                if matches!=selected && skeleton.transform_types.get(t.label)!=Some(&0) {continue;}
            }
            let labels = skeleton
                .labels
                .get(t.label)
                .ok_or(Error::Invalid("animation transform label"))?;
            self.transform(
                &mut context,
                skeleton.transform_types[t.label],
                labels,
                t.values.map(|v| v as f32),
            );
        }
        Ok(())
    }
    pub fn transform(&mut self, context: &mut Context, kind: u8, labels: &[u8], values: [f32; 3]) {
        let mut indices = Vec::new();
        if let Some(skins) = &self.vertex_skins {
            for &label in labels {
                indices.extend(
                    skins
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &s)| (s == label).then_some(i)),
                );
            }
        }
        if kind == 0 {
            let mut sums = [0.0f32; 3];
            for &i in &indices {
                for axis in 0..3 {
                    let sum = sums[axis] + self.vertices[i][axis];
                    sums[axis] = if context.floating {
                        sum
                    } else {
                        sum as i32 as f32
                    };
                }
            }
            context.pivot = std::array::from_fn(|axis| {
                if indices.is_empty() {
                    values[axis]
                } else if context.floating {
                    values[axis] + sums[axis] / indices.len() as f32
                } else {
                    (values[axis] as i32).wrapping_add((sums[axis] as i32) / (indices.len() as i32))
                        as f32
                }
            });
        } else if kind == 1 {
            for i in indices {
                for axis in 0..3 {
                    self.vertices[i][axis] += values[axis];
                }
            }
        } else if kind == 2 {
            let angles = values.map(|v| {
                if context.floating {
                    v * 8.0 * 0.0030679617f32
                } else {
                    ((v as i32 & 255) * 8) as f32
                }
            });
            let trigs = angles.map(|a| {
                if context.floating {
                    [libm::sin(a as f64) as f32, libm::cos(a as f64) as f32]
                } else {
                    let a = a as f64 * 0.0030679615;
                    [
                        (libm::sin(a) * 65536.0) as i32 as f32,
                        (libm::cos(a) * 65536.0) as i32 as f32,
                    ]
                }
            });
            for i in indices {
                let v = &mut self.vertices[i];
                for axis in 0..3 {
                    v[axis] -= context.pivot[axis];
                }
                for axis in [2, 0, 1] {
                    if angles[axis] == 0.0 {
                        continue;
                    }
                    let (s, c) = (trigs[axis][0], trigs[axis][1]);
                    let (a, b, invert) = match axis {
                        2 => (0, 1, false),
                        0 => (1, 2, true),
                        _ => (0, 2, false),
                    };
                    let (x, y) = (v[a], v[b]);
                    if context.floating {
                        if invert {
                            v[a] = c * x - s * y;
                            v[b] = s * x + c * y;
                        } else {
                            v[a] = s * y + c * x;
                            v[b] = c * y - s * x;
                        }
                    } else {
                        let (s, c, x, y) = (s as i32, c as i32, x as i32, y as i32);
                        if invert {
                            v[a] = (c.wrapping_mul(x).wrapping_sub(s.wrapping_mul(y)) >> 16) as f32;
                            v[b] = (s.wrapping_mul(x).wrapping_add(c.wrapping_mul(y)) >> 16) as f32;
                        } else {
                            v[a] = (s.wrapping_mul(y).wrapping_add(c.wrapping_mul(x)) >> 16) as f32;
                            v[b] = (c.wrapping_mul(y).wrapping_sub(s.wrapping_mul(x)) >> 16) as f32;
                        }
                    }
                }
                for axis in 0..3 {
                    v[axis] += context.pivot[axis];
                }
            }
        } else if kind == 3 {
            for i in indices {
                for axis in 0..3 {
                    let v = self.vertices[i][axis] - context.pivot[axis];
                    self.vertices[i][axis] = if context.floating {
                        values[axis] * v / 128.0
                    } else {
                        (values[axis] as i32)
                            .wrapping_mul(v as i32)
                            .wrapping_div(128) as f32
                    };
                    self.vertices[i][axis] += context.pivot[axis];
                }
            }
        } else if kind == 5 {
            if let (Some(skins), Some(alpha)) = (&self.face_skins, &mut self.alpha) {
                for &label in labels {
                    for (i, &s) in skins.iter().enumerate() {
                        if s == label {
                            alpha[i] = (alpha[i] as i32)
                                .wrapping_add((values[0] as i32).wrapping_mul(8))
                                .clamp(0, 255) as u8;
                        }
                    }
                }
            }
        }
    }
}
