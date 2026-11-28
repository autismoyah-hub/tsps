//! GameSceneTileModel's thirteen shapes and four rotations.
use crate::{Error, Result};
const VERTICES: [&[u8]; 13] = [
    &[1, 3, 5, 7],
    &[1, 3, 5, 7],
    &[1, 3, 5, 7],
    &[1, 3, 5, 7, 6],
    &[1, 3, 5, 7, 6],
    &[1, 3, 5, 7, 6],
    &[1, 3, 5, 7, 6],
    &[1, 3, 5, 7, 2, 6],
    &[1, 3, 5, 7, 2, 8],
    &[1, 3, 5, 7, 2, 8],
    &[1, 3, 5, 7, 11, 12],
    &[1, 3, 5, 7, 11, 12],
    &[1, 3, 5, 7, 13, 14],
];
const FACES: [&[u8]; 13] = [
    &[0, 1, 2, 3, 0, 0, 1, 3],
    &[1, 1, 2, 3, 1, 0, 1, 3],
    &[0, 1, 2, 3, 1, 0, 1, 3],
    &[0, 0, 1, 2, 0, 0, 2, 4, 1, 0, 4, 3],
    &[0, 0, 1, 4, 0, 0, 4, 3, 1, 1, 2, 4],
    &[0, 0, 4, 3, 1, 0, 1, 2, 1, 0, 2, 4],
    &[0, 1, 2, 4, 1, 0, 1, 4, 1, 0, 4, 3],
    &[0, 4, 1, 2, 0, 4, 2, 5, 1, 0, 4, 5, 1, 0, 5, 3],
    &[0, 4, 1, 2, 0, 4, 2, 3, 0, 4, 3, 5, 1, 0, 4, 5],
    &[0, 0, 4, 5, 1, 4, 1, 2, 1, 4, 2, 3, 1, 4, 3, 5],
    &[
        0, 0, 1, 5, 0, 1, 4, 5, 0, 1, 2, 4, 1, 0, 5, 3, 1, 5, 4, 3, 1, 4, 2, 3,
    ],
    &[
        1, 0, 1, 5, 1, 1, 4, 5, 1, 1, 2, 4, 0, 0, 5, 3, 0, 5, 4, 3, 0, 4, 2, 3,
    ],
    &[
        1, 0, 5, 4, 1, 0, 1, 5, 0, 0, 4, 3, 0, 4, 5, 3, 0, 5, 2, 3, 0, 1, 2, 5,
    ],
];
pub struct TileMesh {
    pub vertices: Vec<[i32; 3]>,
    pub faces: Vec<[usize; 3]>,
    pub colors: Vec<[i32; 3]>,
    pub overlay: Vec<bool>,
}

/// GameWorldView.pt fallback uses wrapping Java int arithmetic at both stages.
pub fn bilinear_height(heights: [i32; 4], point: [i32; 2]) -> i32 {
    let [x, z] = point.map(|v| v & 127);
    let blend =
        |a: i32, b: i32, t: i32| a.wrapping_mul(128 - t).wrapping_add(b.wrapping_mul(t)) >> 7;
    blend(
        blend(heights[0], heights[1], x),
        blend(heights[3], heights[2], x),
        z,
    )
}

/// Original GameWorldView.ih triangle height, without allocating a render mesh.
/// Coordinates are relative to the tile origin; no containing face means fallback.
pub fn surface_height(shape: u8, rotation: u8, heights: [i32; 4], point: [i32; 2]) -> Option<i32> {
    let codes = *VERTICES.get(shape as usize)?;
    if rotation > 3 {
        return None;
    }
    let mut vertices = [[0; 3]; 6];
    for (i, &code) in codes.iter().enumerate() {
        let (x, z, a, b) = vertex(code, rotation);
        vertices[i] = [x, mix(heights, a, b), z];
    }
    for face in FACES[shape as usize].chunks_exact(4) {
        let v = face_indices(face, rotation).map(|i| vertices[i]);
        if let Some(height) =
            crate::render3d::terrain_pick(point, v.map(|p| [p[0], p[2]]), v.map(|p| p[1] as f32))
        {
            return Some(height as i32);
        }
    }
    None
}

fn mix(values: [i32; 4], a: usize, b: usize) -> i32 {
    if a == b {
        values[a]
    } else {
        values[a].wrapping_add(values[b]) >> 1
    }
}

/// GameWorldView.ij: paints use their own triangle order and flat shortcut.
pub fn paint_height(heights: [i32; 4], flat: bool, point: [i32; 2]) -> Option<i32> {
    if flat {
        return Some(heights[0]);
    }
    let corners = [[0, 0], [128, 0], [128, 128], [0, 128]];
    for face in [[2, 3, 1], [0, 1, 3]] {
        if let Some(height) = crate::render3d::terrain_pick(
            point,
            face.map(|i| corners[i]),
            face.map(|i| heights[i] as f32),
        ) {
            return Some(height as i32);
        }
    }
    None
}

fn face_indices(face: &[u8], rotation: u8) -> [usize; 3] {
    std::array::from_fn(|i| {
        if face[i + 1] < 4 {
            (face[i + 1].wrapping_sub(rotation) & 3) as usize
        } else {
            face[i + 1] as usize
        }
    })
}

fn vertex(code: u8, rotation: u8) -> (i32, i32, usize, usize) {
    let mut code = code as i32;
    let rotation = rotation as i32;
    if code <= 8 && code & 1 == 0 {
        code = ((code - rotation * 2 - 1) & 7) + 1;
    } else if (9..=12).contains(&code) {
        code = ((code - 9 - rotation) & 3) + 9;
    } else if code > 12 {
        code = ((code - 13 - rotation) & 3) + 13;
    }
    match code {
        1 => (0, 0, 0, 0),
        2 => (64, 0, 0, 1),
        3 => (128, 0, 1, 1),
        4 => (128, 64, 1, 2),
        5 => (128, 128, 2, 2),
        6 => (64, 128, 2, 3),
        7 => (0, 128, 3, 3),
        8 => (0, 64, 3, 0),
        9 => (64, 32, 0, 1),
        10 => (96, 64, 1, 2),
        11 => (64, 96, 2, 3),
        12 => (32, 64, 3, 0),
        13 => (32, 32, 0, 0),
        14 => (96, 32, 1, 1),
        15 => (96, 96, 2, 2),
        _ => (32, 96, 3, 3),
    }
}
impl TileMesh {
    /// GameScene paints use fixed corner order and ignore shape rotation.
    pub fn for_scene(shape:u8,rotation:u8,x:i32,z:i32,heights:[i32;4],under:[i32;4],over:[i32;4])->Result<Self> {
        let mut mesh=Self::new(shape,if shape<=1 {0}else{rotation},x,z,heights,under,over)?;
        if shape<=1 {mesh.faces[0].rotate_left(1);mesh.colors[0].rotate_left(1);}
        Ok(mesh)
    }

    pub fn new(
        shape: u8,
        rotation: u8,
        x: i32,
        y: i32,
        heights: [i32; 4],
        underlay: [i32; 4],
        overlay: [i32; 4],
    ) -> Result<Self> {
        if shape > 12 || rotation > 3 {
            return Err(Error::Invalid("tile mesh shape/rotation"));
        }
        let mut out = Self {
            vertices: Vec::new(),
            faces: Vec::new(),
            colors: Vec::new(),
            overlay: Vec::new(),
        };
        let mut colors = Vec::new();
        for &code in VERTICES[shape as usize] {
            let (px, py, a, b) = vertex(code, rotation);
            out.vertices
                .push([x * 128 + px, mix(heights, a, b), y * 128 + py]);
            colors.push([mix(underlay, a, b), mix(overlay, a, b)]);
        }
        for f in FACES[shape as usize].chunks_exact(4) {
            let side = f[0] as usize;
            let face = face_indices(f, rotation);
            out.faces.push(face);
            out.colors.push(face.map(|i| colors[i][side]));
            out.overlay.push(side != 0);
        }
        Ok(out)
    }
}
