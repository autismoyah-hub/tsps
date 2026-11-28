//! Coordinate deduplication and model combination (GameModelData constructor/zz).
use crate::{
    model::{Format, Model},
    Error, Result,
};
use std::collections::HashMap;
impl Model {
    pub fn combine(parts: &[Model]) -> Result<Self> {
        let mut priority = 255;
        let mut priorities = false;
        for p in parts {
            if p.priorities.is_some() {
                priorities = true;
            } else {
                if priority == 255 {
                    priority = p.priority;
                }
                if priority != p.priority {
                    priorities = true;
                }
            }
        }
        let mut out = Self {
            format: Format::SkinnedExtended,
            vertices: Vec::new(),
            faces: Vec::new(),
            colors: Vec::new(),
            render_types: parts
                .iter()
                .any(|p| p.render_types.is_some())
                .then(Vec::new),
            priorities: priorities.then(Vec::new),
            priority,
            alpha: parts.iter().any(|p| p.alpha.is_some()).then(Vec::new),
            vertex_skins: Some(Vec::new()),
            face_skins: parts.iter().any(|p| p.face_skins.is_some()).then(Vec::new),
            bone_weights: parts
                .iter()
                .any(|p| p.bone_weights.is_some())
                .then(Vec::new),
            face_textures: parts
                .iter()
                .any(|p| p.face_textures.is_some())
                .then(Vec::new),
            texture_coords: parts
                .iter()
                .any(|p| p.texture_coords.is_some())
                .then(Vec::new),
            texture_types: Vec::new(),
            texture_triangles: Vec::new(),
            face_flags: parts.iter().any(|p| p.face_flags.is_some()).then(Vec::new),
            metadata: None,
        };
        let mut coordinates = HashMap::<[i32; 3], u16>::new();
        fn vertex(
            out: &mut Model,
            points: &mut HashMap<[i32; 3], u16>,
            p: &Model,
            index: u16,
        ) -> Result<u16> {
            let index = index as usize;
            let key = p.vertices[index].map(|v| v as i32);
            if let Some(&v) = points.get(&key) {
                return Ok(v);
            }
            let v = u16::try_from(out.vertices.len())
                .map_err(|_| Error::Invalid("combined model exceeds 65536 vertices"))?;
            points.insert(key, v);
            out.vertices.push(key.map(|v| v as f32));
            out.vertex_skins
                .as_mut()
                .unwrap()
                .push(p.vertex_skins.as_ref().map_or(0, |s| s[index]));
            if let Some(weights) = &mut out.bone_weights {
                weights.push(
                    p.bone_weights
                        .as_ref()
                        .map_or(Vec::new(), |w| w[index].clone()),
                );
            }
            Ok(v)
        }
        fn byte(out: &mut Option<Vec<u8>>, source: &Option<Vec<u8>>, index: usize, default: u8) {
            if let Some(out) = out {
                out.push(source.as_ref().map_or(default, |s| s[index]));
            }
        }
        for p in parts {
            let texture_offset = out.texture_types.len();
            for (i, face) in p.faces.iter().enumerate() {
                byte(&mut out.render_types, &p.render_types, i, 0);
                byte(&mut out.priorities, &p.priorities, i, p.priority);
                byte(&mut out.alpha, &p.alpha, i, 0);
                byte(&mut out.face_skins, &p.face_skins, i, 0);
                byte(&mut out.face_flags, &p.face_flags, i, 0);
                if let Some(textures) = &mut out.face_textures {
                    textures.push(p.face_textures.as_ref().map_or(-1, |t| t[i]));
                }
                if let Some(coords) = &mut out.texture_coords {
                    let t = p.texture_coords.as_ref().map_or(-1, |t| t[i]);
                    coords.push(if t == -1 {
                        -1
                    } else {
                        (t as i32 + texture_offset as i32) as i8
                    });
                }
                out.colors.push(p.colors[i]);
                let f = [
                    vertex(&mut out, &mut coordinates, p, face[0])?,
                    vertex(&mut out, &mut coordinates, p, face[1])?,
                    vertex(&mut out, &mut coordinates, p, face[2])?,
                ];
                out.faces.push(f);
            }
            for (&kind, face) in p.texture_types.iter().zip(&p.texture_triangles) {
                out.texture_types.push(kind);
                let f = if kind == 0 {
                    [
                        vertex(&mut out, &mut coordinates, p, face[0])?,
                        vertex(&mut out, &mut coordinates, p, face[1])?,
                        vertex(&mut out, &mut coordinates, p, face[2])?,
                    ]
                } else {
                    [0; 3]
                };
                out.texture_triangles.push(f);
            }
        }
        Ok(out)
    }
}
