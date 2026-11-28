//! Four model encodings selected by the original GameModelData constructor.
use crate::{buffer::Reader, Error, Result};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Legacy,
    Extended,
    SkinnedLegacy,
    SkinnedExtended,
}
#[derive(Clone, Debug)]
pub struct Model {
    pub format: Format,
    pub vertices: Vec<[f32; 3]>,
    pub faces: Vec<[u16; 3]>,
    pub colors: Vec<u16>,
    pub render_types: Option<Vec<u8>>,
    pub priorities: Option<Vec<u8>>,
    pub priority: u8,
    pub alpha: Option<Vec<u8>>,
    pub vertex_skins: Option<Vec<u8>>,
    pub face_skins: Option<Vec<u8>>,
    pub bone_weights: Option<Vec<Vec<(u8, u8)>>>,
    pub face_textures: Option<Vec<i16>>,
    pub texture_coords: Option<Vec<i8>>,
    pub texture_types: Vec<u8>,
    pub texture_triangles: Vec<[u16; 3]>,
    /// Encoded signed per-face depth offsets (GameModelData.ah / GameModel.cc).
    pub face_flags: Option<Vec<u8>>,
    pub metadata: Option<([u16; 3], u32)>,
}
fn section<'a>(r: &mut Reader<'a>, size: usize) -> Result<Reader<'a>> {
    Ok(Reader::new(r.bytes(size)?))
}
fn flag(r: &mut Reader<'_>) -> Result<bool> {
    match r.u8()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Error::Invalid("model feature flag")),
    }
}
impl Model {
    pub fn face_alpha(&self, face: usize) -> u8 {
        crate::model_alpha::normalize(self.alpha.as_ref().map_or(0, |a| a[face]))
    }
    pub fn face_depth_bias(&self, face: usize) -> i16 {
        self.face_flags.as_ref().map_or(0, |v| v[face] as i8 as i16 * 2)
    }
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 18 {
            return Err(Error::Truncated);
        }
        let format = match &data[data.len() - 2..] {
            [255, 253] => Format::SkinnedExtended,
            [255, 254] => Format::SkinnedLegacy,
            [255, 255] => Format::Extended,
            _ => Format::Legacy,
        };
        let extended = matches!(format, Format::Extended | Format::SkinnedExtended);
        let skinned = matches!(format, Format::SkinnedLegacy | Format::SkinnedExtended);
        let footer_size = match format {
            Format::Legacy => 18,
            Format::SkinnedExtended => 26,
            _ => 23,
        };
        let end = data
            .len()
            .checked_sub(footer_size)
            .ok_or(Error::Truncated)?;
        let mut footer = Reader::new(&data[end..]);
        let vertices = footer.u16()? as usize;
        let faces = footer.u16()? as usize;
        let textures = footer.u8()? as usize;
        let render = flag(&mut footer)?;
        let priority = footer.u8()?;
        let alpha = flag(&mut footer)?;
        let face_skin = flag(&mut footer)?;
        let face_texture = if extended { flag(&mut footer)? } else { render };
        let vertex_skin = flag(&mut footer)?;
        let bones = if skinned { flag(&mut footer)? } else { false };
        let xlen = footer.u16()? as usize;
        let ylen = footer.u16()? as usize;
        let zlen = footer.u16()? as usize;
        let ilen = footer.u16()? as usize;
        let tlen = if extended { footer.u16()? as usize } else { 0 };
        let vlen = if skinned {
            footer.u16()? as usize
        } else if vertex_skin {
            vertices
        } else {
            0
        };
        let mut body = Reader::new(&data[..end]);
        let texture_types = if extended {
            body.bytes(textures)?.to_vec()
        } else {
            vec![0; textures]
        };
        if texture_types.iter().any(|&t| t > 3) {
            return Err(Error::Invalid("model texture type"));
        }
        let simple = texture_types.iter().filter(|&&t| t == 0).count();
        let complex = textures - simple;
        let cube = texture_types.iter().filter(|&&t| t == 2).count();
        let mut vertex_flags = section(&mut body, vertices)?;
        let (
            mut render_data,
            mut index_types,
            mut priority_data,
            mut face_skin_data,
            mut vertex_data,
            mut alpha_data,
            mut index_data,
            mut texture_data,
            mut coord_data,
            mut color_data,
            mut triangle_data,
            mut xdata,
            mut ydata,
            mut zdata,
        );
        if extended {
            render_data = section(&mut body, if render { faces } else { 0 })?;
            index_types = section(&mut body, faces)?;
            priority_data = section(&mut body, if priority == 255 { faces } else { 0 })?;
            face_skin_data = section(&mut body, if face_skin { faces } else { 0 })?;
            vertex_data = section(&mut body, vlen)?;
            alpha_data = section(&mut body, if alpha { faces } else { 0 })?;
            index_data = section(&mut body, ilen)?;
            texture_data = section(&mut body, if face_texture { faces * 2 } else { 0 })?;
            coord_data = section(&mut body, tlen)?;
            color_data = section(&mut body, faces * 2)?;
            xdata = section(&mut body, xlen)?;
            ydata = section(&mut body, ylen)?;
            zdata = section(&mut body, zlen)?;
            triangle_data = section(&mut body, simple * 6)?;
            // These complex mapping sections are present in the format but
            // the revision 239 decoder deliberately leaves their arrays zero.
            body.bytes(complex * 6 + complex * 6 + complex * 2 + complex + complex * 2 + cube * 2)?;
        } else {
            index_types = section(&mut body, faces)?;
            priority_data = section(&mut body, if priority == 255 { faces } else { 0 })?;
            face_skin_data = section(&mut body, if face_skin { faces } else { 0 })?;
            render_data = section(&mut body, if render { faces } else { 0 })?;
            vertex_data = section(&mut body, vlen)?;
            alpha_data = section(&mut body, if alpha { faces } else { 0 })?;
            index_data = section(&mut body, ilen)?;
            color_data = section(&mut body, faces * 2)?;
            triangle_data = section(&mut body, textures * 6)?;
            xdata = section(&mut body, xlen)?;
            ydata = section(&mut body, ylen)?;
            zdata = section(&mut body, zlen)?;
            texture_data = Reader::new(&[]);
            coord_data = Reader::new(&[]);
        }
        let mut out = Self {
            format,
            vertices: Vec::with_capacity(vertices),
            faces: Vec::with_capacity(faces),
            colors: Vec::with_capacity(faces),
            render_types: render.then(|| vec![0; faces]),
            priorities: (priority == 255).then(|| vec![0; faces]),
            priority: if priority == 255 { 0 } else { priority },
            alpha: alpha.then(|| vec![0; faces]),
            vertex_skins: vertex_skin.then(|| vec![0; vertices]),
            face_skins: face_skin.then(|| vec![0; faces]),
            bone_weights: bones.then(Vec::new),
            face_textures: face_texture.then(|| vec![-1; faces]),
            texture_coords: (face_texture && (!extended || textures > 0)).then(|| vec![0; faces]),
            texture_types,
            texture_triangles: vec![[0; 3]; textures],
            face_flags: None,
            metadata: None,
        };
        let mut previous = [0i32; 3];
        for i in 0..vertices {
            let flags = vertex_flags.u8()?;
            if flags & !7 != 0 {
                return Err(Error::Invalid("model vertex flags"));
            }
            let delta = [
                if flags & 1 != 0 {
                    xdata.signed_smart()? as i32
                } else {
                    0
                },
                if flags & 2 != 0 {
                    ydata.signed_smart()? as i32
                } else {
                    0
                },
                if flags & 4 != 0 {
                    zdata.signed_smart()? as i32
                } else {
                    0
                },
            ];
            let vertex =
                std::array::from_fn(|axis| previous[axis].wrapping_add(delta[axis]) as f32);
            previous = vertex.map(|v| v as i32);
            out.vertices.push(vertex);
            if let Some(skins) = out.vertex_skins.as_mut() {
                skins[i] = vertex_data.u8()?;
            }
        }
        if let Some(weights) = out.bone_weights.as_mut() {
            for _ in 0..vertices {
                let mut values = Vec::new();
                for _ in 0..vertex_data.u8()? {
                    values.push((vertex_data.u8()?, vertex_data.u8()?));
                }
                weights.push(values);
            }
        }
        let (mut any_render, mut any_texture) = (false, false);
        for i in 0..faces {
            let mut color = color_data.u16()?;
            if render {
                let flags = render_data.u8()?;
                if extended {
                    out.render_types.as_mut().unwrap()[i] = flags;
                } else {
                    let kind = flags & 1;
                    any_render |= kind != 0;
                    out.render_types.as_mut().unwrap()[i] = kind;
                    if flags & 2 != 0 {
                        out.texture_coords.as_mut().unwrap()[i] = (flags >> 2) as i8;
                        out.face_textures.as_mut().unwrap()[i] = color as i16;
                        any_texture |= color != 65535;
                        color = 127;
                    } else {
                        out.texture_coords.as_mut().unwrap()[i] = -1;
                    }
                }
            }
            out.colors.push(color);
            if let Some(p) = out.priorities.as_mut() {
                p[i] = priority_data.u8()?;
            }
            if let Some(a) = out.alpha.as_mut() {
                a[i] = alpha_data.u8()?;
            }
            if let Some(s) = out.face_skins.as_mut() {
                s[i] = face_skin_data.u8()?;
            }
            if extended && face_texture {
                let id = texture_data.u16()?.wrapping_sub(1) as i16;
                out.face_textures.as_mut().unwrap()[i] = id;
                if let Some(coords) = out.texture_coords.as_mut() {
                    if id != -1 {
                        coords[i] = coord_data.u8()?.wrapping_sub(1) as i8;
                    }
                }
            }
        }
        let (mut a, mut b, mut c, mut last) = (0i32, 0i32, 0i32, 0i32);
        for _ in 0..faces {
            match index_types.u8()? {
                1 => {
                    a = last + index_data.signed_smart()? as i32;
                    b = a + index_data.signed_smart()? as i32;
                    c = b + index_data.signed_smart()? as i32;
                }
                2 => {
                    b = c;
                    c = last + index_data.signed_smart()? as i32;
                }
                3 => {
                    a = c;
                    c = last + index_data.signed_smart()? as i32;
                }
                4 => {
                    std::mem::swap(&mut a, &mut b);
                    c = last + index_data.signed_smart()? as i32;
                }
                _ => return Err(Error::Invalid("model triangle opcode")),
            }
            last = c;
            if [a, b, c].iter().any(|&i| i < 0 || i as usize >= vertices) {
                return Err(Error::Invalid("model triangle vertex index"));
            }
            out.faces.push([a as u16, b as u16, c as u16]);
        }
        for i in 0..textures {
            if out.texture_types[i] == 0 {
                let tri = [
                    triangle_data.u16()?,
                    triangle_data.u16()?,
                    triangle_data.u16()?,
                ];
                if tri.iter().any(|&v| v as usize >= vertices) {
                    return Err(Error::Invalid("texture triangle vertex"));
                }
                out.texture_triangles[i] = tri;
            }
        }
        if extended && body.u8()? != 0 {
            out.metadata = Some(([body.u16()?, body.u16()?, body.u16()?], body.u32()?));
        }
        if skinned && body.u8()? == 1 {
            out.face_flags = Some(body.bytes(faces)?.to_vec());
        }
        if !extended {
            if let Some(coords) = out.texture_coords.as_mut() {
                let mut any = false;
                for (face, coord) in coords.iter_mut().enumerate() {
                    if *coord != -1 {
                        let tri = out
                            .texture_triangles
                            .get(*coord as u8 as usize)
                            .ok_or(Error::Invalid("legacy texture coordinate"))?;
                        if *tri == out.faces[face] {
                            *coord = -1;
                        } else {
                            any = true;
                        }
                    }
                }
                if !any {
                    out.texture_coords = None;
                }
            }
            if !any_render {
                out.render_types = None;
            }
            if !any_texture {
                out.face_textures = None;
            }
        }
        for r in [
            &body,
            &vertex_flags,
            &vertex_data,
            &xdata,
            &ydata,
            &zdata,
            &index_types,
            &index_data,
            &color_data,
            &priority_data,
            &alpha_data,
            &face_skin_data,
            &render_data,
            &texture_data,
            &coord_data,
            &triangle_data,
        ] {
            if r.remaining() != 0 {
                return Err(Error::Invalid("model section trailing bytes"));
            }
        }
        Ok(out)
    }
}
