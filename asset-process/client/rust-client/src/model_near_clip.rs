//! GameModel.gt near-plane polygon construction, after camera projection.
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub screen: [i32; 2],
    pub depth: f32,
    pub color: i32,
    /// Original edge and its fixed-point fraction, for texture interpolation.
    pub edge: [usize; 2],
    pub fraction: f32,
}
/// The original reciprocal table has 2048 entries. An out-of-range edge
/// aborts model drawing; return None rather than indexing outside that table.
pub fn polygon(
    points: [[f32; 3]; 3],
    colors: [i32; 3],
    center: [i32; 2],
    focal: i32,
) -> Option<Vec<Vertex>> {
    let integer = points.map(|v| v.map(|v| v as i32));
    let mut out = Vec::with_capacity(4);
    for i in 0..3 {
        let a = integer[i];
        if a[2] >= 50 {
            out.push(Vertex {
                screen: std::array::from_fn(|axis| {
                    (center[axis] as f32 + points[i][axis] * focal as f32 / points[i][2]) as i32
                }),
                depth: points[i][2],
                color: colors[i],
                edge: [i, i],
                fraction: 0.0,
            });
        } else {
            // Original emits the previous-edge crossing before the next edge.
            for j in [(i + 2) % 3, (i + 1) % 3] {
                let b = integer[j];
                if b[2] < 50 {
                    continue;
                }
                let delta = b[2].wrapping_sub(a[2]);
                if !(1..2048).contains(&delta) {
                    return None;
                }
                let fraction = 50i32.wrapping_sub(a[2]).wrapping_mul(65536 / delta);
                let interpolate =
                    |x: i32, y: i32| x.wrapping_add(y.wrapping_sub(x).wrapping_mul(fraction) >> 16);
                out.push(Vertex {
                    screen: std::array::from_fn(|axis| {
                        center[axis]
                            .wrapping_add(interpolate(a[axis], b[axis]).wrapping_mul(focal) / 50)
                    }),
                    depth: 50.0,
                    color: interpolate(colors[i], colors[j]),
                    edge: [i, j],
                    fraction: fraction as f32 / 65536.0,
                });
            }
        }
    }
    Some(out)
}

/// Near-crossing model faces use the integer camera-space normal (GameModel.fr).
pub fn front_facing(points: [[f32; 3]; 3]) -> bool {
    let [a, b, c] = points.map(|v| v.map(|v| v as i32));
    let a: [i32; 3] = std::array::from_fn(|i| a[i].wrapping_sub(b[i]));
    let c: [i32; 3] = std::array::from_fn(|i| c[i].wrapping_sub(b[i]));
    let cross = [
        a[1].wrapping_mul(c[2])
            .wrapping_sub(a[2].wrapping_mul(c[1])),
        a[2].wrapping_mul(c[0])
            .wrapping_sub(a[0].wrapping_mul(c[2])),
        a[0].wrapping_mul(c[1])
            .wrapping_sub(a[1].wrapping_mul(c[0])),
    ];
    b[0].wrapping_mul(cross[0])
        .wrapping_add(b[1].wrapping_mul(cross[1]))
        .wrapping_add(b[2].wrapping_mul(cross[2]))
        > 0
}
