//! Camera-centered background model (packet135, GameClient.df, wf.jt and hp.nt).
use crate::{
    disk_cache::DiskCache,
    model::Model,
    render3d::{texture_uv, Triangle},
    Result,
};
pub struct State {
    pub requested: i32,
    pub displayed: i32,
    pub triangles: Option<Vec<Triangle>>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            requested: -1,
            displayed: -1,
            triangles: None,
        }
    }
}
impl State {
    pub fn request(&mut self, id: i32) {
        self.requested = id;
        if id == -1 {
            self.displayed = -1;
            self.triangles = None;
        }
    }
    pub fn load(&mut self, cache: &mut DiskCache) -> Result<()> {
        if self.requested < 0 || self.requested == self.displayed {
            return Ok(());
        }
        let id = self.requested as u32;
        if !cache
            .reference(7)?
            .groups
            .iter()
            .any(|g| g.id == id && g.files.iter().any(|f| f.id == 0))
        {
            return Ok(());
        }
        let model = Model::decode(&cache.file(7, id, 0, [0; 4])?)?;
        let triangles = mesh(&model)?;
        self.displayed = self.requested;
        self.triangles = Some(triangles);
        Ok(())
    }
}
pub fn mesh(model: &Model) -> Result<Vec<Triangle>> {
    let colors = model.light(128, 43690, [0, -1, 0])?;
    let mut out = Vec::new();
    let bounds = crate::model_bounds::Bounds::new(&model.vertices, [0; 3]);
    for (i, face) in model.faces.iter().enumerate() {
        if colors[i][2] == -2 {
            continue;
        }
        let flat = colors[i][2] == -1;
        let color = if flat { [colors[i][0]; 3] } else { colors[i] };
        let vertices = face.map(|v| model.vertices[v as usize]);
        let texture = model
            .face_textures
            .as_ref()
            .and_then(|t| (t[i] != -1).then_some(t[i] as u16));
        let mapping = model
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
        out.push(Triangle {
            #[cfg(feature="enhanced-renderer")] material_hsl:None, texture_mapping: Some(crate::render3d::TextureMapping { vertices: mapping.map(|v| model.vertices[v as usize]), local: None }),
            local_vertices: None,
            model_bounds: Some(bounds),
                model_order: None, depth_bias: model.face_depth_bias(i),
            view: 0,
            vertices,
            colors: color,
            flat,
            uv: texture_uv(mapping.map(|v| model.vertices[v as usize]), vertices),
            texture,
            alpha: model.face_alpha(i),
            near_alpha: Some(model.alpha.as_ref().map_or(0, |a| a[i])),
            tag: 0,
        });
    }
    Ok(out)
}
