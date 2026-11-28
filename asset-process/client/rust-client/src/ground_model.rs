//! Ground-pile terrain normals and quaternion alignment (lu.el / GameTileItem).
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let length = ((v[1] * v[1] + v[0] * v[0] + v[2] * v[2]) as f64).sqrt() as f32;
    let scale = 1.0 / length;
    v.map(|x| x * scale)
}
/// Corner order is southwest, southeast, northeast, northwest.
pub fn normal(h: [i32; 4]) -> [f32; 3] {
    if h.iter().all(|x| *x == h[0]) {
        return [0.0, -1.0, 0.0];
    }
    let h = h.map(|x| x as f32);
    normalize([
        128.0 * (h[1] - h[0]) + 128.0 * (h[2] - h[3]),
        -32768.0,
        128.0 * (h[3] - h[0]) + 128.0 * (h[2] - h[1]),
    ])
}
fn multiply(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [
        b[1] * a[2] + (a[3] * b[0] + a[0] * b[3]) - a[1] * b[2],
        a[0] * b[2] + (a[1] * b[3] - b[0] * a[2] + b[1] * a[3]),
        a[3] * b[2] + (a[1] * b[0] + a[2] * b[3] - b[1] * a[0]),
        b[3] * a[3] - a[0] * b[0] - b[1] * a[1] - a[2] * b[2],
    ]
}
pub fn align(vertices: &mut [[f32; 3]], n: [f32; 3]) {
    if n == [0.0, -1.0, 0.0] {
        return;
    }
    let axis = normalize([-n[2], 0.0, n[0]]);
    let angle = (-n[1] as f64).acos() as f32;
    let half = (angle * 0.5) as f64;
    let s = half.sin() as f32;
    let mut q = [axis[0] * s, axis[1] * s, axis[2] * s, half.cos() as f32];
    let scale =
        1.0 / ((q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]) as f64).sqrt() as f32;
    for x in &mut q {
        *x *= scale;
    }
    for v in vertices {
        let p = multiply(
            multiply([-q[0], -q[1], -q[2], q[3]], [v[0], v[1], v[2], 0.0]),
            q,
        );
        *v = [p[0], p[1], p[2]];
    }
}

/// GameModel.calculateBoundsCylinder rounds the tallest upward vertex with ceil.
pub fn support_height(vertices: &[[f32; 3]]) -> i32 {
    vertices
        .iter()
        .fold(0.0f32, |height, v| height.max(-v[1]))
        .ceil() as i32
}
