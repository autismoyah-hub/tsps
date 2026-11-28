//! Timed object/player attachments (zone record 2 / ci.dq).
use crate::{
    buffer::{Reader, Transform},
    location_update::category,
    model::Model,
    object::Definition,
    scene::Scene,
    Error, Result,
};
#[derive(Clone, Copy, Debug)]
pub struct Request {
    pub player: u16,
    pub object: u16,
    pub at: u8,
    pub shape: u8,
    pub rotation: u8,
    pub start: u16,
    pub end: u16,
    pub bounds: [i8; 4],
}
impl Request {
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let x0 = r.u8()? as i8;
        let player = r.u16_le()?;
        let packed = r.u8()?;
        let y0 = r.u8()? as i8;
        let object = r.u16_le()?;
        let end = r.u16()?;
        let start = r.u16_add(true)?;
        let x1 = r.transformed(Transform::Add)? as i8;
        let at = r.transformed(Transform::Sub)?;
        let y1 = r.transformed(Transform::Sub)? as i8;
        if r.remaining() != 0 {
            return Err(Error::Invalid("location merge trailing bytes"));
        }
        category(packed >> 2)?;
        Ok(Self {
            player,
            object,
            at,
            shape: packed >> 2,
            rotation: packed & 3,
            start,
            end,
            bounds: [x0, y0, x1, y1],
        })
    }
    pub fn key(&self, origin: [i32; 3]) -> Result<[i32; 4]> {
        Ok([
            origin[0],
            origin[1] + ((self.at >> 4) & 7) as i32,
            origin[2] + (self.at & 7) as i32,
            category(self.shape)? as i32,
        ])
    }
    pub fn placement(
        &self,
        origin: [i32; 3],
        size: [u8; 2],
        cycle: i32,
        mut height: impl FnMut(i32, i32) -> Result<i32>,
    ) -> Result<Placement> {
        let key = self.key(origin)?;
        let [w, h] = if matches!(self.rotation, 1 | 3) {
            [size[1] as i32, size[0] as i32]
        } else {
            size.map(i32::from)
        };
        let [x, y] = [key[1], key[2]];
        let elevation = (height(x + (w >> 1), y + (h >> 1))?
            .wrapping_add(height(x + ((w + 1) >> 1), y + (h >> 1))?)
            .wrapping_add(height(x + (w >> 1), y + ((h + 1) >> 1))?)
            .wrapping_add(height(x + ((w + 1) >> 1), y + ((h + 1) >> 1))?))
            >> 2;
        let [x0, y0, x1, y1] = self.bounds.map(i32::from);
        Ok(Placement {
            start: cycle.wrapping_add(self.start as i32),
            end: cycle.wrapping_add(self.end as i32),
            centre: [x * 128 + w * 64, elevation, y * 128 + h * 64],
            bounds: [
                x + x0.min(x1),
                y + y0.min(y1),
                x + x0.max(x1),
                y + y0.max(y1),
            ],
        })
    }
}
#[derive(Clone, Debug)]
pub struct Placement {
    pub start: i32,
    pub end: i32,
    pub centre: [i32; 3],
    pub bounds: [i32; 4],
}
impl Placement {
    pub fn active(&self, cycle: i32) -> bool {
        cycle >= self.start && cycle < self.end
    }
}
#[derive(Clone, Debug)]
pub struct Attachment {
    pub placement: Placement,
    pub model: Model,
    pub colors: Vec<[i32; 3]>,
}
impl Attachment {
    pub fn prepare(
        request: Request,
        origin: [i32; 3],
        cycle: i32,
        scene: &Scene,
        def: &Definition,
        mut model: Model,
    ) -> Result<Self> {
        let placement = request.placement(origin, def.size, cycle, |x, y| {
            scene
                .height(
                    origin[0] as usize,
                    (x - scene.base_x) as usize,
                    (y - scene.base_y) as usize,
                )
                .ok_or(Error::Invalid("location merge height"))
        })?;
        let colors = model.light(64 + def.ambient as i32, 768 + def.contrast, [-50, -10, -50])?;
        if def.contour >= 0 {
            let height = model
                .vertices
                .iter()
                .map(|v| -(v[1] as i32))
                .max()
                .unwrap_or(0)
                .max(0);
            for vertex in &mut model.vertices {
                if let Some(ground) = scene.height_fine(
                    origin[0] as usize,
                    placement.centre[0] - scene.base_x * 128 + vertex[0] as i32,
                    placement.centre[2] - scene.base_y * 128 + vertex[2] as i32,
                ) {
                    let delta = ground - placement.centre[1];
                    if def.contour == 0 {
                        vertex[1] = (vertex[1] as i32 + delta) as f32;
                    } else if height > 0 {
                        let ratio = (-(vertex[1] as i32) << 16) / height;
                        if ratio < def.contour {
                            vertex[1] = (vertex[1] as i32
                                + delta * (def.contour - ratio) / def.contour)
                                as f32;
                        }
                    }
                }
            }
        }
        Ok(Self {
            placement,
            model,
            colors,
        })
    }
    pub fn relative_model(&self, position: [i32; 3], target: u16) -> Model {
        let mut model = self.model.clone();
        model.translate(std::array::from_fn(|i| {
            self.placement.centre[i] - position[i]
        }));
        model.rotate_quarter(match target {
            512 => 3,
            1024 => 2,
            1536 => 1,
            _ => 0,
        });
        model
    }
}
