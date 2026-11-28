//! Legacy non-depth 14-bit edge walker (fr.av).
pub fn spans(vertices: [[f32; 3]; 3], height: usize, mut emit: impl FnMut(usize, i32, i32)) {
    let mut v = vertices.map(|v| [v[0] as i32, v[1] as i32]);
    v.sort_by_key(|v| v[1]);
    let [a, b, c] = v;
    let slope = |p: [i32; 2], q: [i32; 2]| {
        if q[1] == p[1] {
            0
        } else {
            q[0].wrapping_sub(p[0]).wrapping_shl(14) / (q[1] - p[1])
        }
    };
    let ac = slope(a, c);
    let ab = slope(a, b);
    let bc = slope(b, c);
    for y in a[1].max(0)..c[1].min(height as i32) {
        let long = a[0]
            .wrapping_shl(14)
            .wrapping_add(ac.wrapping_mul(y - a[1]));
        let short = if y < b[1] {
            a[0].wrapping_shl(14)
                .wrapping_add(ab.wrapping_mul(y - a[1]))
        } else {
            b[0].wrapping_shl(14)
                .wrapping_add(bc.wrapping_mul(y - b[1]))
        };
        emit(y as usize, long.min(short) >> 14, long.max(short) >> 14);
    }
}
pub fn gouraud(
    vertices: [[f32; 3]; 3],
    colors: [i32; 3],
    height: usize,
    mut emit: impl FnMut(usize, i32, i32, i32, i32),
) {
    let v = vertices.map(|v| [v[0] as i32, v[1] as i32]);
    let dx1 = v[1][0] - v[0][0];
    let dy1 = v[1][1] - v[0][1];
    let dx2 = v[2][0] - v[0][0];
    let dy2 = v[2][1] - v[0][1];
    let determinant = dx1 * dy2 - dx2 * dy1;
    if determinant == 0 {
        return;
    }
    let dc1 = colors[1] - colors[0];
    let dc2 = colors[2] - colors[0];
    let cx =
        (dc1.wrapping_mul(dy2).wrapping_sub(dc2.wrapping_mul(dy1))).wrapping_shl(8) / determinant;
    let cy =
        (dc2.wrapping_mul(dx1).wrapping_sub(dc1.wrapping_mul(dx2))).wrapping_shl(8) / determinant;
    let top = (0..3).min_by_key(|&i| v[i][1]).unwrap();
    let initial = colors[top]
        .wrapping_shl(8)
        .wrapping_sub(cx.wrapping_mul(v[top][0]))
        .wrapping_add(cx);
    spans(vertices, height, |row, start, end| {
        emit(
            row,
            start,
            end,
            initial.wrapping_add(cy.wrapping_mul(row as i32 - v[top][1])),
            cx,
        )
    });
}

fn shade9(
    vertices: [[f32; 3]; 3],
    colors: [i32; 3],
    height: usize,
    mut emit: impl FnMut(usize, i32, i32, i32, i32),
) {
    let v = vertices.map(|v| [v[0] as i32, v[1] as i32]);
    let dx1 = v[1][0] - v[0][0];
    let dy1 = v[1][1] - v[0][1];
    let dx2 = v[2][0] - v[0][0];
    let dy2 = v[2][1] - v[0][1];
    let determinant = dx1 * dy2 - dx2 * dy1;
    if determinant == 0 {
        return;
    }
    let dc1 = colors[1] - colors[0];
    let dc2 = colors[2] - colors[0];
    let cx =
        (dc1.wrapping_mul(dy2).wrapping_sub(dc2.wrapping_mul(dy1))).wrapping_shl(9) / determinant;
    let cy =
        (dc2.wrapping_mul(dx1).wrapping_sub(dc1.wrapping_mul(dx2))).wrapping_shl(9) / determinant;
    let top = (0..3).min_by_key(|&i| v[i][1]).unwrap();
    let initial = colors[top]
        .wrapping_shl(9)
        .wrapping_sub(cx.wrapping_mul(v[top][0]))
        .wrapping_add(cx);
    spans(vertices, height, |row, start, end| {
        emit(
            row,
            start,
            end,
            initial.wrapping_add(cy.wrapping_mul(row as i32 - v[top][1])),
            cx,
        )
    });
}

pub fn texture(
    vertices: [[f32; 3]; 3],
    colors: [i32; 3],
    mapping: [[i32; 3]; 3],
    focal: i32,
    center_y: i32,
    height: usize,
    mut emit: impl FnMut(usize, i32, i32, i32, i32, [i32; 3], [i32; 3]),
) {
    use std::num::Wrapping as W;
    let [mx0, my0, mz0] = mapping[0].map(W);
    let [mut mx1, mut my1, mut mz1] = mapping[1].map(W);
    let [mut mx2, mut my2, mut mz2] = mapping[2].map(W);
    mx1 = mx0 - mx1;
    my1 = my0 - my1;
    mz1 = mz0 - mz1;
    mx2 -= mx0;
    my2 -= my0;
    mz2 -= mz0;
    let base = [
        (mx2 * my0 - my2 * mx0) << 14,
        (mx1 * my0 - my1 * mx0) << 14,
        (my1 * mx2 - mx1 * my2) << 14,
    ];
    let x = [
        my2 * mz0 - mz2 * my0,
        my1 * mz0 - mz1 * my0,
        mz1 * my2 - my1 * mz2,
    ]
    .map(|v| (((v.0 as i64) << 17) / focal as i64) as i32);
    let y = [
        mz2 * mx0 - mx2 * mz0,
        mz1 * mx0 - mx1 * mz0,
        mx1 * mz2 - mz1 * mx2,
    ]
    .map(|v| W((((v.0 as i64) << 14) / focal as i64) as i32));
    shade9(vertices, colors, height, |row, start, end, shade, step| {
        let offset = W(row as i32 - center_y);
        emit(
            row,
            start,
            end,
            shade,
            step,
            std::array::from_fn(|i| (base[i] + y[i] * offset).0),
            x,
        );
    });
}
