//! Original fy.ak texture triangle setup.
use std::num::Wrapping as W;
#[allow(unused_mut, unused_parens)]
fn scan<const TERRAIN: bool>(
    vertices: [[f32; 3]; 3],
    colors: [i32; 3],
    mapping: [[i32; 3]; 3],
    focal: i32,
    center_y: i32,
    height: usize,
    mut emit: impl FnMut(usize, i32, i32, i32, i32, f32, f32, [i32; 3], [i32; 3]),
) {
    let [x0, y0, mut z0] = vertices[0];
    let [x1, y1, mut z1] = vertices[1];
    let [x2, y2, mut z2] = vertices[2];
    let [mut shade0, mut shade1, mut shade2] = colors.map(W);
    let [mut mx0, mut my0, mut mz0] = mapping[0].map(W);
    let [mut mx1, mut my1, mut mz1] = mapping[1].map(W);
    let [mut mx2, mut my2, mut mz2] = mapping[2].map(W);
    let mut edge_x0: f32 = ((x0 + 0.5) as i32 as f32);
    let mut edge_x1: f32 = ((x1 + 0.5) as i32 as f32);
    let mut edge_x2: f32 = ((x2 + 0.5) as i32 as f32);
    let mut row0: f32 = ((y0 + 0.5) as i32 as f32);
    let mut row1: f32 = ((y1 + 0.5) as i32 as f32);
    let mut row2: f32 = ((y2 + 0.5) as i32 as f32);
    let mut rounded_dx1: f32 = edge_x1 - edge_x0;
    let mut rounded_dy1: f32 = row1 - row0;
    let mut rounded_dx2: f32 = edge_x2 - edge_x0;
    let mut rounded_dy2: f32 = row2 - row0;
    let mut rounded_determinant: f32 = rounded_dx1 * rounded_dy2 - rounded_dx2 * rounded_dy1;
    if rounded_determinant != 0.0 {
        let mut ds1: f32 = (shade1 - shade0).0 as f32;
        let mut ds2: f32 = (shade2 - shade0).0 as f32;
        let shade_dx =
            W(((ds1 * rounded_dy2 - ds2 * rounded_dy1) * 512.0 / rounded_determinant) as i32);
        let shade_dy =
            W(((ds2 * rounded_dx1 - ds1 * rounded_dx2) * 512.0 / rounded_determinant) as i32);
        let mut dx1: f32 = x1 - x0;
        let mut dy1: f32 = y1 - y0;
        let mut dx2: f32 = x2 - x0;
        let mut dy2: f32 = y2 - y0;
        let mut determinant: f32 = dx1 * dy2 - dx2 * dy1;
        if determinant != 0.0 {
            let mut dz1: f32 = z1 - z0;
            let mut dz2: f32 = z2 - z0;
            let mut depth_dx: f32 = (dz1 * dy2 - dz2 * dy1) / determinant;
            let mut depth_dy: f32 = (dz2 * dx1 - dz1 * dx2) / determinant;
            let mut slope12: f32;
            if row2 != row1 {
                slope12 = (edge_x2 - edge_x1) / (row2 - row1);
            } else {
                slope12 = 0.0;
            }

            let mut slope01: f32;
            if row1 != row0 {
                slope01 = rounded_dx1 / rounded_dy1;
            } else {
                slope01 = 0.0;
            }

            let mut slope02: f32;
            if row2 != row0 {
                slope02 = rounded_dx2 / rounded_dy2;
            } else {
                slope02 = 0.0;
            }

            let zoom = focal;
            mx1 = mx0 - mx1;
            my1 = my0 - my1;
            mz1 = mz0 - mz1;
            mx2 -= mx0;
            my2 -= my0;
            mz2 -= mz0;
            let mut u_row = (mx2 * my0 - my2 * mx0) << 14;
            let u_step8 = W(((((my2 * mz0 - mz2 * my0).0 as i64)
                << (if TERRAIN { 0 } else { 3 })
                << 14)
                / zoom as i64) as i32);
            let u_step_y = W(((((mz2 * mx0 - mx2 * mz0).0 as i64) << 14) / zoom as i64) as i32);
            let mut v_row = (mx1 * my0 - my1 * mx0) << 14;
            let v_step8 = W(((((my1 * mz0 - mz1 * my0).0 as i64)
                << (if TERRAIN { 0 } else { 3 })
                << 14)
                / zoom as i64) as i32);
            let v_step_y = W(((((mz1 * mx0 - mx1 * mz0).0 as i64) << 14) / zoom as i64) as i32);
            let mut w_row = (my1 * mx2 - mx1 * my2) << 14;
            let w_step8 = W(((((mz1 * my2 - my1 * mz2).0 as i64)
                << (if TERRAIN { 0 } else { 3 })
                << 14)
                / zoom as i64) as i32);
            let w_step_y = W(((((mx1 * mz2 - mz1 * mx2).0 as i64) << 14) / zoom as i64) as i32);
            let clip_height = height as f32;
            if row0 <= row1 && row0 <= row2 {
                if !(row0 >= clip_height) {
                    if row1 > clip_height {
                        row1 = clip_height;
                    }

                    if row2 > clip_height {
                        row2 = clip_height;
                    }

                    shade0 = (shade0 << 9) - shade_dx * W(edge_x0 as i32) + shade_dx;
                    z0 = origin(x0, y0, z0, depth_dx, depth_dy);
                    if row1 < row2 {
                        edge_x2 = edge_x0;
                        if row0 < 0.0 {
                            edge_x2 -= slope02 * row0;
                            edge_x0 -= slope01 * row0;
                            shade0 -= shade_dy * W(row0 as i32);
                            row0 = 0.0;
                        }

                        if row1 < 0.0 {
                            edge_x1 -= slope12 * row1;
                            row1 = 0.0;
                        }

                        let row_offset = W(row0 as i32) - W(center_y);
                        u_row += u_step_y * row_offset;
                        v_row += v_step_y * row_offset;
                        w_row += w_step_y * row_offset;
                        if (row0 == row1 || !(slope02 < slope01))
                            && (row0 != row1 || !(slope02 > slope12))
                        {
                            row0 = ((row0 + 0.5) as i32 as f32);
                            row1 = ((row1 + 0.5) as i32 as f32);
                            row2 = ((row2 + 0.5) as i32 as f32) - row1;
                            row1 -= row0;
                            let mut row = W(row0 as i32);

                            while {
                                row1 -= 1.0;
                                row1 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x0 as i32,
                                    edge_x2 as i32,
                                    shade0.0,
                                    shade_dx.0,
                                    z0,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x2 += slope02;
                                edge_x0 += slope01;
                                shade0 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z0 += depth_dy;
                            }

                            while {
                                row2 -= 1.0;
                                row2 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x1 as i32,
                                    edge_x2 as i32,
                                    shade0.0,
                                    shade_dx.0,
                                    z0,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x2 += slope02;
                                edge_x1 += slope12;
                                shade0 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z0 += depth_dy;
                            }
                        } else {
                            row0 = ((row0 + 0.5) as i32 as f32);
                            row1 = ((row1 + 0.5) as i32 as f32);
                            row2 = ((row2 + 0.5) as i32 as f32) - row1;
                            row1 -= row0;
                            let mut row = W(row0 as i32);

                            while {
                                row1 -= 1.0;
                                row1 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x2 as i32,
                                    edge_x0 as i32,
                                    shade0.0,
                                    shade_dx.0,
                                    z0,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x2 += slope02;
                                edge_x0 += slope01;
                                shade0 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z0 += depth_dy;
                            }

                            while {
                                row2 -= 1.0;
                                row2 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x2 as i32,
                                    edge_x1 as i32,
                                    shade0.0,
                                    shade_dx.0,
                                    z0,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x2 += slope02;
                                edge_x1 += slope12;
                                shade0 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z0 += depth_dy;
                            }
                        }
                    } else {
                        edge_x1 = edge_x0;
                        if row0 < 0.0 {
                            edge_x1 -= slope02 * row0;
                            edge_x0 -= slope01 * row0;
                            if TERRAIN {
                                shade0 -= shade_dy * W(row0 as i32);
                            } else {
                                shade0 = W((shade0.0 as f32 - shade_dy.0 as f32 * row0) as i32);
                            }
                            row0 = 0.0;
                        }

                        if row2 < 0.0 {
                            edge_x2 -= slope12 * row2;
                            row2 = 0.0;
                        }

                        let row_offset = W(row0 as i32) - W(center_y);
                        u_row += u_step_y * row_offset;
                        v_row += v_step_y * row_offset;
                        w_row += w_step_y * row_offset;
                        if (row0 == row2 || !(slope02 < slope01))
                            && (row0 != row2 || !(slope12 > slope01))
                        {
                            row0 = ((row0 + 0.5) as i32 as f32);
                            row2 = ((row2 + 0.5) as i32 as f32);
                            row1 = ((row1 + 0.5) as i32 as f32) - row2;
                            row2 -= row0;
                            let mut row = W(row0 as i32);

                            while {
                                row2 -= 1.0;
                                row2 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x0 as i32,
                                    edge_x1 as i32,
                                    shade0.0,
                                    shade_dx.0,
                                    z0,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x1 += slope02;
                                edge_x0 += slope01;
                                shade0 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z0 += depth_dy;
                            }

                            while {
                                row1 -= 1.0;
                                row1 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x0 as i32,
                                    edge_x2 as i32,
                                    shade0.0,
                                    shade_dx.0,
                                    z0,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x2 += slope12;
                                edge_x0 += slope01;
                                shade0 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z0 += depth_dy;
                            }
                        } else {
                            row0 = ((row0 + 0.5) as i32 as f32);
                            row2 = ((row2 + 0.5) as i32 as f32);
                            row1 = ((row1 + 0.5) as i32 as f32) - row2;
                            row2 -= row0;
                            let mut row = W(row0 as i32);

                            while {
                                row2 -= 1.0;
                                row2 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x1 as i32,
                                    edge_x0 as i32,
                                    shade0.0,
                                    shade_dx.0,
                                    z0,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x1 += slope02;
                                edge_x0 += slope01;
                                shade0 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z0 += depth_dy;
                            }

                            while {
                                row1 -= 1.0;
                                row1 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x2 as i32,
                                    edge_x0 as i32,
                                    shade0.0,
                                    shade_dx.0,
                                    z0,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x2 += slope12;
                                edge_x0 += slope01;
                                shade0 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z0 += depth_dy;
                            }
                        }
                    }
                }
            } else if row1 <= row2 {
                if !(row1 >= clip_height) {
                    if row2 > clip_height {
                        row2 = clip_height;
                    }

                    if row0 > clip_height {
                        row0 = clip_height;
                    }

                    shade1 = (shade1 << 9) - shade_dx * W(edge_x1 as i32) + shade_dx;
                    z1 = origin(x1, y1, z1, depth_dx, depth_dy);
                    if row2 < row0 {
                        edge_x0 = edge_x1;
                        if row1 < 0.0 {
                            edge_x0 -= slope01 * row1;
                            edge_x1 -= slope12 * row1;
                            shade1 -= shade_dy * W(row1 as i32);
                            row1 = 0.0;
                        }

                        if row2 < 0.0 {
                            edge_x2 -= slope02 * row2;
                            row2 = 0.0;
                        }

                        let row_offset = W(row1 as i32) - W(center_y);
                        u_row += u_step_y * row_offset;
                        v_row += v_step_y * row_offset;
                        w_row += w_step_y * row_offset;
                        if (row1 == row2 || !(slope01 < slope12))
                            && (row1 != row2 || !(slope01 > slope02))
                        {
                            row1 = ((row1 + 0.5) as i32 as f32);
                            row2 = ((row2 + 0.5) as i32 as f32);
                            row0 = ((row0 + 0.5) as i32 as f32) - row2;
                            row2 -= row1;
                            let mut row = W(row1 as i32);

                            while {
                                row2 -= 1.0;
                                row2 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x1 as i32,
                                    edge_x0 as i32,
                                    shade1.0,
                                    shade_dx.0,
                                    z1,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x0 += slope01;
                                edge_x1 += slope12;
                                shade1 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z1 += depth_dy;
                            }

                            while {
                                row0 -= 1.0;
                                row0 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x2 as i32,
                                    edge_x0 as i32,
                                    shade1.0,
                                    shade_dx.0,
                                    z1,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x0 += slope01;
                                edge_x2 += slope02;
                                shade1 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z1 += depth_dy;
                            }
                        } else {
                            row1 = ((row1 + 0.5) as i32 as f32);
                            row2 = ((row2 + 0.5) as i32 as f32);
                            row0 = ((row0 + 0.5) as i32 as f32) - row2;
                            row2 -= row1;
                            let mut row = W(row1 as i32);

                            while {
                                row2 -= 1.0;
                                row2 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x0 as i32,
                                    edge_x1 as i32,
                                    shade1.0,
                                    shade_dx.0,
                                    z1,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x0 += slope01;
                                edge_x1 += slope12;
                                shade1 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z1 += depth_dy;
                            }

                            while {
                                row0 -= 1.0;
                                row0 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x0 as i32,
                                    edge_x2 as i32,
                                    shade1.0,
                                    shade_dx.0,
                                    z1,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x0 += slope01;
                                edge_x2 += slope02;
                                shade1 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z1 += depth_dy;
                            }
                        }
                    } else {
                        edge_x2 = edge_x1;
                        if row1 < 0.0 {
                            edge_x2 -= slope01 * row1;
                            edge_x1 -= slope12 * row1;
                            shade1 -= shade_dy * W(row1 as i32);
                            row1 = 0.0;
                        }

                        if row0 < 0.0 {
                            edge_x0 -= slope02 * row0;
                            row0 = 0.0;
                        }

                        let row_offset = W(row1 as i32) - W(center_y);
                        u_row += u_step_y * row_offset;
                        v_row += v_step_y * row_offset;
                        w_row += w_step_y * row_offset;
                        row1 = ((row1 + 0.5) as i32 as f32);
                        row0 = ((row0 + 0.5) as i32 as f32);
                        row2 = ((row2 + 0.5) as i32 as f32) - row0;
                        row0 -= row1;
                        let mut row = W(row1 as i32);
                        if slope01 < slope12 {
                            while {
                                row0 -= 1.0;
                                row0 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x2 as i32,
                                    edge_x1 as i32,
                                    shade1.0,
                                    shade_dx.0,
                                    z1,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x2 += slope01;
                                edge_x1 += slope12;
                                shade1 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z1 += depth_dy;
                            }

                            while {
                                row2 -= 1.0;
                                row2 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x0 as i32,
                                    edge_x1 as i32,
                                    shade1.0,
                                    shade_dx.0,
                                    z1,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x0 += slope02;
                                edge_x1 += slope12;
                                shade1 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z1 += depth_dy;
                            }
                        } else {
                            while {
                                row0 -= 1.0;
                                row0 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x1 as i32,
                                    edge_x2 as i32,
                                    shade1.0,
                                    shade_dx.0,
                                    z1,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x2 += slope01;
                                edge_x1 += slope12;
                                shade1 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z1 += depth_dy;
                            }

                            while {
                                row2 -= 1.0;
                                row2 >= 0.0
                            } {
                                emit(
                                    row.0 as usize,
                                    edge_x1 as i32,
                                    edge_x0 as i32,
                                    shade1.0,
                                    shade_dx.0,
                                    z1,
                                    depth_dx,
                                    [u_row.0, v_row.0, w_row.0],
                                    [u_step8.0, v_step8.0, w_step8.0],
                                );
                                edge_x0 += slope02;
                                edge_x1 += slope12;
                                shade1 += shade_dy;
                                row += W(1);
                                u_row += u_step_y;
                                v_row += v_step_y;
                                w_row += w_step_y;
                                z1 += depth_dy;
                            }
                        }
                    }
                }
            } else if !(row2 >= clip_height) {
                if row0 > clip_height {
                    row0 = clip_height;
                }

                if row1 > clip_height {
                    row1 = clip_height;
                }

                shade2 = (shade2 << 9) - shade_dx * W(edge_x2 as i32) + shade_dx;
                z2 = origin(x2, y2, z2, depth_dx, depth_dy);
                if row0 < row1 {
                    edge_x1 = edge_x2;
                    if row2 < 0.0 {
                        edge_x1 -= slope12 * row2;
                        edge_x2 -= slope02 * row2;
                        shade2 -= shade_dy * W(row2 as i32);
                        row2 = 0.0;
                    }

                    if row0 < 0.0 {
                        edge_x0 -= slope01 * row0;
                        row0 = 0.0;
                    }

                    let row_offset = W(row2 as i32) - W(center_y);
                    u_row += u_step_y * row_offset;
                    v_row += v_step_y * row_offset;
                    w_row += w_step_y * row_offset;
                    row2 = ((row2 + 0.5) as i32 as f32);
                    row0 = ((row0 + 0.5) as i32 as f32);
                    row1 = ((row1 + 0.5) as i32 as f32) - row0;
                    row0 -= row2;
                    let mut row = W(row2 as i32);
                    if slope12 < slope02 {
                        while {
                            row0 -= 1.0;
                            row0 >= 0.0
                        } {
                            emit(
                                row.0 as usize,
                                edge_x1 as i32,
                                edge_x2 as i32,
                                shade2.0,
                                shade_dx.0,
                                z2,
                                depth_dx,
                                [u_row.0, v_row.0, w_row.0],
                                [u_step8.0, v_step8.0, w_step8.0],
                            );
                            edge_x1 += slope12;
                            edge_x2 += slope02;
                            shade2 += shade_dy;
                            row += W(1);
                            u_row += u_step_y;
                            v_row += v_step_y;
                            w_row += w_step_y;
                            z2 += depth_dy;
                        }

                        while {
                            row1 -= 1.0;
                            row1 >= 0.0
                        } {
                            emit(
                                row.0 as usize,
                                edge_x1 as i32,
                                edge_x0 as i32,
                                shade2.0,
                                shade_dx.0,
                                z2,
                                depth_dx,
                                [u_row.0, v_row.0, w_row.0],
                                [u_step8.0, v_step8.0, w_step8.0],
                            );
                            edge_x1 += slope12;
                            edge_x0 += slope01;
                            shade2 += shade_dy;
                            row += W(1);
                            u_row += u_step_y;
                            v_row += v_step_y;
                            w_row += w_step_y;
                            z2 += depth_dy;
                        }
                    } else {
                        while {
                            row0 -= 1.0;
                            row0 >= 0.0
                        } {
                            emit(
                                row.0 as usize,
                                edge_x2 as i32,
                                edge_x1 as i32,
                                shade2.0,
                                shade_dx.0,
                                z2,
                                depth_dx,
                                [u_row.0, v_row.0, w_row.0],
                                [u_step8.0, v_step8.0, w_step8.0],
                            );
                            edge_x1 += slope12;
                            edge_x2 += slope02;
                            shade2 += shade_dy;
                            row += W(1);
                            u_row += u_step_y;
                            v_row += v_step_y;
                            w_row += w_step_y;
                            z2 += depth_dy;
                        }

                        while {
                            row1 -= 1.0;
                            row1 >= 0.0
                        } {
                            emit(
                                row.0 as usize,
                                edge_x0 as i32,
                                edge_x1 as i32,
                                shade2.0,
                                shade_dx.0,
                                z2,
                                depth_dx,
                                [u_row.0, v_row.0, w_row.0],
                                [u_step8.0, v_step8.0, w_step8.0],
                            );
                            edge_x1 += slope12;
                            edge_x0 += slope01;
                            shade2 += shade_dy;
                            row += W(1);
                            u_row += u_step_y;
                            v_row += v_step_y;
                            w_row += w_step_y;
                            z2 += depth_dy;
                        }
                    }
                } else {
                    edge_x0 = edge_x2;
                    if row2 < 0.0 {
                        edge_x0 -= slope12 * row2;
                        edge_x2 -= slope02 * row2;
                        shade2 -= shade_dy * W(row2 as i32);
                        row2 = 0.0;
                    }

                    if row1 < 0.0 {
                        edge_x1 -= slope01 * row1;
                        row1 = 0.0;
                    }

                    let row_offset = W(row2 as i32) - W(center_y);
                    u_row += u_step_y * row_offset;
                    v_row += v_step_y * row_offset;
                    w_row += w_step_y * row_offset;
                    row2 = ((row2 + 0.5) as i32 as f32);
                    row1 = ((row1 + 0.5) as i32 as f32);
                    row0 = ((row0 + 0.5) as i32 as f32) - row1;
                    row1 -= row2;
                    let mut row = W(row2 as i32);
                    if slope12 < slope02 {
                        while {
                            row1 -= 1.0;
                            row1 >= 0.0
                        } {
                            emit(
                                row.0 as usize,
                                edge_x0 as i32,
                                edge_x2 as i32,
                                shade2.0,
                                shade_dx.0,
                                z2,
                                depth_dx,
                                [u_row.0, v_row.0, w_row.0],
                                [u_step8.0, v_step8.0, w_step8.0],
                            );
                            edge_x0 += slope12;
                            edge_x2 += slope02;
                            shade2 += shade_dy;
                            row += W(1);
                            u_row += u_step_y;
                            v_row += v_step_y;
                            w_row += w_step_y;
                            z2 += depth_dy;
                        }

                        while {
                            row0 -= 1.0;
                            row0 >= 0.0
                        } {
                            emit(
                                row.0 as usize,
                                edge_x1 as i32,
                                edge_x2 as i32,
                                shade2.0,
                                shade_dx.0,
                                z2,
                                depth_dx,
                                [u_row.0, v_row.0, w_row.0],
                                [u_step8.0, v_step8.0, w_step8.0],
                            );
                            edge_x1 += slope01;
                            edge_x2 += slope02;
                            shade2 += shade_dy;
                            row += W(1);
                            u_row += u_step_y;
                            v_row += v_step_y;
                            w_row += w_step_y;
                            z2 += depth_dy;
                        }
                    } else {
                        while {
                            row1 -= 1.0;
                            row1 >= 0.0
                        } {
                            emit(
                                row.0 as usize,
                                edge_x2 as i32,
                                edge_x0 as i32,
                                shade2.0,
                                shade_dx.0,
                                z2,
                                depth_dx,
                                [u_row.0, v_row.0, w_row.0],
                                [u_step8.0, v_step8.0, w_step8.0],
                            );
                            edge_x0 += slope12;
                            edge_x2 += slope02;
                            shade2 += shade_dy;
                            row += W(1);
                            u_row += u_step_y;
                            v_row += v_step_y;
                            w_row += w_step_y;
                            z2 += depth_dy;
                        }

                        while {
                            row0 -= 1.0;
                            row0 >= 0.0
                        } {
                            emit(
                                row.0 as usize,
                                edge_x2 as i32,
                                edge_x1 as i32,
                                shade2.0,
                                shade_dx.0,
                                z2,
                                depth_dx,
                                [u_row.0, v_row.0, w_row.0],
                                [u_step8.0, v_step8.0, w_step8.0],
                            );
                            edge_x1 += slope01;
                            edge_x2 += slope02;
                            shade2 += shade_dy;
                            row += W(1);
                            u_row += u_step_y;
                            v_row += v_step_y;
                            w_row += w_step_y;
                            z2 += depth_dy;
                        }
                    }
                }
            }
        }
    }
}
fn origin(x: f32, y: f32, mut z: f32, dx: f32, dy: f32) -> f32 {
    z = z - dx * x + dx;
    if y < 0.0 {
        z -= dy * (y as i32 as f32);
    }
    z - dy * (y - ((y + 0.5) as i32 as f32))
}

pub fn spans(
    vertices: [[f32; 3]; 3],
    colors: [i32; 3],
    mapping: [[i32; 3]; 3],
    focal: i32,
    center_y: i32,
    height: usize,
    emit: impl FnMut(usize, i32, i32, i32, i32, f32, f32, [i32; 3], [i32; 3]),
) {
    scan::<false>(vertices, colors, mapping, focal, center_y, height, emit)
}

pub fn terrain_spans(
    vertices: [[f32; 3]; 3],
    colors: [i32; 3],
    mapping: [[i32; 3]; 3],
    focal: i32,
    center_y: i32,
    height: usize,
    emit: impl FnMut(usize, i32, i32, i32, i32, f32, f32, [i32; 3], [i32; 3]),
) {
    scan::<true>(vertices, colors, mapping, focal, center_y, height, emit)
}
