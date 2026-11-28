//! Exact float operation order of original ud.am/ar/ef and ie.ao.
use crate::skeletal::Matrix;
pub fn inverse(m: Matrix) -> Matrix {
    let determinant = m[8] * (m[3] * m[5]) * m[14]
        + (m[8] * (m[7] * m[2]) * m[13]
            + (m[12] * (m[11] * (m[5] * m[2]))
                + (m[2] * m[4] * m[9] * m[15]
                    + (m[13] * (m[11] * (m[0] * m[6]))
                        + (m[10] * (m[0] * m[5]) * m[15]
                            - m[5] * m[0] * m[11] * m[14]
                            - m[6] * m[0] * m[9] * m[15])
                        + m[14] * (m[9] * (m[0] * m[7]))
                        - m[13] * (m[10] * (m[0] * m[7]))
                        - m[1] * m[4] * m[10] * m[15]
                        + m[14] * (m[4] * m[1] * m[11])
                        + m[15] * (m[8] * (m[1] * m[6]))
                        - m[12] * (m[1] * m[6] * m[11])
                        - m[8] * (m[1] * m[7]) * m[14]
                        + m[1] * m[7] * m[10] * m[12])
                    - m[2] * m[4] * m[11] * m[13]
                    - m[5] * m[2] * m[8] * m[15]))
            - m[9] * (m[7] * m[2]) * m[12]
            - m[14] * (m[3] * m[4] * m[9])
            + m[13] * (m[4] * m[3] * m[10]))
        - m[12] * (m[3] * m[5] * m[10])
        - m[8] * (m[6] * m[3]) * m[13]
        + m[12] * (m[9] * (m[6] * m[3]));
    let var2 = 1.0f32 / determinant;
    let var3 = (m[5] * m[10] * m[15] - m[14] * (m[5] * m[11]) - m[6] * m[9] * m[15]
        + m[11] * m[6] * m[13]
        + m[7] * m[9] * m[14]
        - m[7] * m[10] * m[13])
        * var2;
    let var4 = (m[15] * (m[2] * m[9]) + (m[14] * (m[11] * m[1]) + -m[1] * m[10] * m[15])
        - m[13] * (m[11] * m[2])
        - m[14] * (m[3] * m[9])
        + m[10] * m[3] * m[13])
        * var2;
    let var5 = var2
        * (m[15] * (m[6] * m[1]) - m[7] * m[1] * m[14] - m[15] * (m[5] * m[2])
            + m[2] * m[7] * m[13]
            + m[3] * m[5] * m[14]
            - m[13] * (m[6] * m[3]));
    let var6 = (m[11] * (m[2] * m[5]) + (m[11] * (m[6] * -m[1]) + m[10] * (m[1] * m[7]))
        - m[9] * (m[2] * m[7])
        - m[5] * m[3] * m[10]
        + m[9] * (m[6] * m[3]))
        * var2;
    let var7 = var2
        * (m[12] * (m[10] * m[7])
            + (m[15] * (m[10] * -m[4]) + m[11] * m[4] * m[14] + m[15] * (m[6] * m[8])
                - m[12] * (m[6] * m[11])
                - m[8] * m[7] * m[14]));
    let var8 = var2
        * (m[14] * (m[8] * m[3])
            + (m[15] * (m[10] * m[0]) - m[0] * m[11] * m[14] - m[15] * (m[2] * m[8])
                + m[2] * m[11] * m[12])
            - m[12] * (m[3] * m[10]));
    let var9 = var2
        * (m[12] * (m[6] * m[3])
            + (m[15] * (m[4] * m[2]) + (m[6] * -m[0] * m[15] + m[7] * m[0] * m[14])
                - m[2] * m[7] * m[12]
                - m[14] * (m[3] * m[4])));
    let var10 = (m[0] * m[6] * m[11] - m[10] * (m[7] * m[0]) - m[2] * m[4] * m[11]
        + m[8] * (m[2] * m[7])
        + m[3] * m[4] * m[10]
        - m[6] * m[3] * m[8])
        * var2;
    let var11 = var2
        * (m[13] * (m[7] * m[8])
            + (m[15] * (m[9] * m[4]) - m[13] * (m[11] * m[4]) - m[5] * m[8] * m[15]
                + m[5] * m[11] * m[12])
            - m[12] * (m[9] * m[7]));
    let var12 = var2
        * (m[9] * m[3] * m[12]
            + (m[11] * m[0] * m[13] + m[15] * (-m[0] * m[9]) + m[1] * m[8] * m[15]
                - m[1] * m[11] * m[12]
                - m[3] * m[8] * m[13]));
    let var13 = (m[13] * (m[4] * m[3])
        + (m[12] * (m[1] * m[7])
            + (m[15] * (m[5] * m[0]) - m[7] * m[0] * m[13] - m[15] * (m[4] * m[1])))
        - m[3] * m[5] * m[12])
        * var2;
    let var14 = var2
        * (m[7] * m[0] * m[9] + m[5] * -m[0] * m[11] + m[1] * m[4] * m[11]
            - m[8] * (m[1] * m[7])
            - m[3] * m[4] * m[9]
            + m[8] * (m[3] * m[5]));
    let var15 = var2
        * (m[8] * m[5] * m[14] + (m[14] * (-m[4] * m[9]) + m[13] * (m[4] * m[10]))
            - m[10] * m[5] * m[12]
            - m[13] * (m[8] * m[6])
            + m[6] * m[9] * m[12]);
    let var16 = var2
        * (m[8] * m[2] * m[13]
            + (m[14] * (m[9] * m[0]) - m[0] * m[10] * m[13] - m[8] * m[1] * m[14]
                + m[1] * m[10] * m[12])
            - m[9] * m[2] * m[12]);
    let var17 = var2
        * (m[4] * m[1] * m[14] + (m[14] * (-m[0] * m[5]) + m[0] * m[6] * m[13])
            - m[1] * m[6] * m[12]
            - m[2] * m[4] * m[13]
            + m[12] * (m[5] * m[2]));
    let var18 = (m[4] * m[2] * m[9]
        + (m[5] * m[0] * m[10] - m[9] * (m[6] * m[0]) - m[4] * m[1] * m[10] + m[1] * m[6] * m[8])
        - m[8] * (m[5] * m[2]))
        * var2;
    [
        var3, var4, var5, var6, var7, var8, var9, var10, var11, var12, var13, var14, var15, var16,
        var17, var18,
    ]
}
pub fn transform(m: &Matrix, [x, y, z]: [f32; 3]) -> [f32; 3] {
    [
        m[12] + (x * m[0] + m[4] * y + z * m[8]),
        m[9] * z + (m[1] * x + m[5] * y) + m[13],
        m[14] + (m[6] * y + x * m[2] + m[10] * z),
    ]
}
/// Quantize projected points before inversion, then subtract the quantized eye.
pub fn ray(
    inverse: &Matrix,
    size: [i32; 2],
    focal: i32,
    point: [i32; 2],
) -> Option<([i32; 3], [[i32; 3]; 2])> {
    ray_with_far(inverse, size, focal, point, 3500)
}
/// ie.ao reads the current fv.ae far clip for both root and child selections.
pub fn ray_with_far(
    inverse: &Matrix,
    size: [i32; 2],
    focal: i32,
    point: [i32; 2],
    far: i32,
) -> Option<([i32; 3], [[i32; 3]; 2])> {
    if focal <= 0 || inverse.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let eye = transform(inverse, [0.; 3]).map(|v| v as i32);
    let endpoints = [50i32, far].map(|depth| {
        let x = point[0].wrapping_sub(size[0] / 2).wrapping_mul(depth) / focal;
        let y = point[1].wrapping_sub(size[1] / 2).wrapping_mul(depth) / focal;
        let p = transform(inverse, [x as f32, y as f32, depth as f32]);
        std::array::from_fn(|i| (p[i] as i32).wrapping_sub(eye[i]))
    });
    Some((eye, endpoints))
}

/// Original ud.ak: apply next after the existing transform.
pub fn then(m: Matrix, next: Matrix) -> Matrix {
    let var3 = m[2] * next[8] + (next[0] * m[0] + next[4] * m[1]) + next[12] * m[3];
    let var4 = m[3] * next[13] + (m[0] * next[1] + m[1] * next[5] + m[2] * next[9]);
    let var5 = m[2] * next[10] + (m[0] * next[2] + next[6] * m[1]) + next[14] * m[3];
    let var6 = next[11] * m[2] + (m[0] * next[3] + next[7] * m[1]) + next[15] * m[3];
    let var7 = next[4] * m[5] + next[0] * m[4] + m[6] * next[8] + next[12] * m[7];
    let var8 = m[7] * next[13] + (m[6] * next[9] + (next[1] * m[4] + next[5] * m[5]));
    let var9 = m[7] * next[14] + (m[6] * next[10] + (m[4] * next[2] + m[5] * next[6]));
    let var10 = next[15] * m[7] + (m[5] * next[7] + next[3] * m[4] + m[6] * next[11]);
    let var11 = next[12] * m[11] + (m[8] * next[0] + next[4] * m[9] + m[10] * next[8]);
    let var12 = m[11] * next[13] + (m[10] * next[9] + (m[9] * next[5] + next[1] * m[8]));
    let var13 = next[14] * m[11] + (m[10] * next[10] + (next[2] * m[8] + m[9] * next[6]));
    let var14 = next[3] * m[8] + m[9] * next[7] + m[10] * next[11] + next[15] * m[11];
    let var15 = m[12] * next[0] + m[13] * next[4] + next[8] * m[14] + m[15] * next[12];
    let var16 = next[9] * m[14] + (next[1] * m[12] + m[13] * next[5]) + next[13] * m[15];
    let var17 = m[14] * next[10] + (next[6] * m[13] + next[2] * m[12]) + m[15] * next[14];
    let var18 = m[12] * next[3] + m[13] * next[7] + m[14] * next[11] + next[15] * m[15];
    [
        var3, var4, var5, var6, var7, var8, var9, var10, var11, var12, var13, var14, var15, var16,
        var17, var18,
    ]
}
pub fn camera(eye: [i32; 3], angles: [i32; 2]) -> Matrix {
    let mut u = [1.0f32, 0., 0., 0., 1., 0., 0., 0., 1., 0., 0., 0.];
    {
        let angle = -((angles[0] & 16383) as f32 * 3.834952E-4f32);

        let var3 = (angle as f64).cos() as f32;
        let var4 = (angle as f64).sin() as f32;
        let var5 = u[1];
        let var6 = u[4];
        let var7 = u[7];
        let var8 = u[10];
        u[1] = var3 * var5 - var4 * u[2];
        u[2] = u[2] * var3 + var5 * var4;
        u[4] = var6 * var3 - var4 * u[5];
        u[5] = var6 * var4 + var3 * u[5];
        u[7] = var3 * var7 - u[8] * var4;
        u[8] = var7 * var4 + u[8] * var3;
        u[10] = var3 * var8 - var4 * u[11];
        u[11] = var8 * var4 + u[11] * var3;
    }
    {
        let angle = -((angles[1] & 16383) as f32 * 3.834952E-4f32);

        let var3 = (angle as f64).cos() as f32;
        let var4 = (angle as f64).sin() as f32;
        let var5 = u[0];
        let var6 = u[3];
        let var7 = u[6];
        let var8 = u[9];
        u[0] = u[2] * var4 + var5 * var3;
        u[2] = var3 * u[2] - var5 * var4;
        u[3] = var3 * var6 + u[5] * var4;
        u[5] = var3 * u[5] - var4 * var6;
        u[6] = var4 * u[8] + var7 * var3;
        u[8] = var3 * u[8] - var4 * var7;
        u[9] = var8 * var3 + var4 * u[11];
        u[11] = var3 * u[11] - var4 * var8;
    }
    u[9] += eye[0] as f32;
    u[10] += eye[1] as f32;
    u[11] += eye[2] as f32;
    inverse([
        u[0], u[1], u[2], 0., u[3], u[4], u[5], 0., u[6], u[7], u[8], 0., u[9], u[10], u[11], 1.,
    ])
}

pub fn identity() -> Matrix {
    [
        1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ]
}
pub fn scene(
    centre: [f32; 2],
    rotation: [f32; 5],
    translation: [f32; 3],
    vertical: [f32; 2],
    animation: Option<Matrix>,
) -> Matrix {
    let mut local = identity();
    local[12] = -centre[0];
    local[13] = vertical[0];
    local[14] = -centre[1];
    let mut scale = identity();
    scale[5] = vertical[1];
    local = then(local, scale);
    if let Some(animation) = animation {
        local = then(local, animation);
    }
    let [xx, xz, zx, zz, yy] = rotation;
    let root = [
        xx,
        0.,
        zx,
        0.,
        0.,
        yy,
        0.,
        0.,
        xz,
        0.,
        zz,
        0.,
        translation[0],
        translation[1],
        translation[2],
        1.,
    ];
    then(local, root)
}

pub fn model(angle: u16, position: [i32; 3], projection: Matrix) -> Matrix {
    let half = angle as f32 * 0.0030679614f32 * 0.5;
    let y = (half as f64).sin() as f32;
    let w = (half as f64).cos() as f32;
    let (ww, yy, wy) = (w * w, y * y, w * y);
    let local = [
        ww + 0. - 0. - yy,
        0.,
        0. - wy - wy + 0.,
        0.,
        0.,
        ww + yy - 0. - 0.,
        0.,
        0.,
        0. + 0. + wy + wy,
        0.,
        ww + 0. - yy - 0.,
        0.,
        position[0] as f32,
        position[1] as f32,
        position[2] as f32,
        1.,
    ];
    then(local, projection)
}
