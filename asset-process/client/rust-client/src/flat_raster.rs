//! Flat triangle scan conversion from original fy.av. Float operation order is significant.
#[allow(unused_mut, unused_parens)]
pub fn spans(
    vertices: [[f32; 3]; 3],
    height: usize,
    mut emit: impl FnMut(usize, i32, i32, f32, f32),
) {
    let [x0, y0, mut z0] = vertices[0];
    let [x1, y1, mut z1] = vertices[1];
    let [x2, y2, mut z2] = vertices[2];

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

        let clip_height = height as f32;
        if row0 <= row1 && row0 <= row2 {
            if !(row0 >= clip_height) {
                if row1 > clip_height {
                    row1 = clip_height;
                }

                if row2 > clip_height {
                    row2 = clip_height;
                }

                z0 = origin(x0, y0, z0, depth_dx, depth_dy);
                if row1 < row2 {
                    edge_x2 = edge_x0;
                    if row0 < 0.0 {
                        edge_x2 -= slope02 * row0;
                        edge_x0 -= slope01 * row0;
                        row0 = 0.0;
                    }

                    if row1 < 0.0 {
                        edge_x1 -= slope12 * row1;
                        row1 = 0.0;
                    }

                    if (row0 == row1 || !(slope02 < slope01))
                        && (row0 != row1 || !(slope02 > slope12))
                    {
                        row0 = ((row0 + 0.5) as i32 as f32);
                        row1 = ((row1 + 0.5) as i32 as f32);
                        row2 = ((row2 + 0.5) as i32 as f32) - row1;
                        row1 -= row0;
                        let mut row = row0 as i32;

                        while {
                            row1 -= 1.0;
                            row1 >= 0.0
                        } {
                            emit(row as usize, edge_x0 as i32, edge_x2 as i32, z0, depth_dx);
                            edge_x2 += slope02;
                            edge_x0 += slope01;
                            row += 1;
                            z0 += depth_dy;
                        }

                        while {
                            row2 -= 1.0;
                            row2 >= 0.0
                        } {
                            emit(row as usize, edge_x1 as i32, edge_x2 as i32, z0, depth_dx);
                            edge_x2 += slope02;
                            edge_x1 += slope12;
                            row += 1;
                            z0 += depth_dy;
                        }
                    } else {
                        row0 = ((row0 + 0.5) as i32 as f32);
                        row1 = ((row1 + 0.5) as i32 as f32);
                        row2 = ((row2 + 0.5) as i32 as f32) - row1;
                        row1 -= row0;
                        let mut row = row0 as i32;

                        while {
                            row1 -= 1.0;
                            row1 >= 0.0
                        } {
                            emit(row as usize, edge_x2 as i32, edge_x0 as i32, z0, depth_dx);
                            edge_x2 += slope02;
                            edge_x0 += slope01;
                            row += 1;
                            z0 += depth_dy;
                        }

                        while {
                            row2 -= 1.0;
                            row2 >= 0.0
                        } {
                            emit(row as usize, edge_x2 as i32, edge_x1 as i32, z0, depth_dx);
                            edge_x2 += slope02;
                            edge_x1 += slope12;
                            row += 1;
                            z0 += depth_dy;
                        }
                    }
                } else {
                    edge_x1 = edge_x0;
                    if row0 < 0.0 {
                        edge_x1 -= slope02 * row0;
                        edge_x0 -= slope01 * row0;
                        row0 = 0.0;
                    }

                    if row2 < 0.0 {
                        edge_x2 -= slope12 * row2;
                        row2 = 0.0;
                    }

                    if (row0 == row2 || !(slope02 < slope01))
                        && (row0 != row2 || !(slope12 > slope01))
                    {
                        row0 = ((row0 + 0.5) as i32 as f32);
                        row2 = ((row2 + 0.5) as i32 as f32);
                        row1 = ((row1 + 0.5) as i32 as f32) - row2;
                        row2 -= row0;
                        let mut row = row0 as i32;

                        while {
                            row2 -= 1.0;
                            row2 >= 0.0
                        } {
                            emit(row as usize, edge_x0 as i32, edge_x1 as i32, z0, depth_dx);
                            edge_x1 += slope02;
                            edge_x0 += slope01;
                            row += 1;
                            z0 += depth_dy;
                        }

                        while {
                            row1 -= 1.0;
                            row1 >= 0.0
                        } {
                            emit(row as usize, edge_x0 as i32, edge_x2 as i32, z0, depth_dx);
                            edge_x2 += slope12;
                            edge_x0 += slope01;
                            row += 1;
                            z0 += depth_dy;
                        }
                    } else {
                        row0 = ((row0 + 0.5) as i32 as f32);
                        row2 = ((row2 + 0.5) as i32 as f32);
                        row1 = ((row1 + 0.5) as i32 as f32) - row2;
                        row2 -= row0;
                        let mut row = row0 as i32;

                        while {
                            row2 -= 1.0;
                            row2 >= 0.0
                        } {
                            emit(row as usize, edge_x1 as i32, edge_x0 as i32, z0, depth_dx);
                            edge_x1 += slope02;
                            edge_x0 += slope01;
                            row += 1;
                            z0 += depth_dy;
                        }

                        while {
                            row1 -= 1.0;
                            row1 >= 0.0
                        } {
                            emit(row as usize, edge_x2 as i32, edge_x0 as i32, z0, depth_dx);
                            edge_x2 += slope12;
                            edge_x0 += slope01;
                            row += 1;
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

                z1 = origin(x1, y1, z1, depth_dx, depth_dy);
                if row2 < row0 {
                    edge_x0 = edge_x1;
                    if row1 < 0.0 {
                        edge_x0 -= slope01 * row1;
                        edge_x1 -= slope12 * row1;
                        row1 = 0.0;
                    }

                    if row2 < 0.0 {
                        edge_x2 -= slope02 * row2;
                        row2 = 0.0;
                    }

                    if (row1 == row2 || !(slope01 < slope12))
                        && (row1 != row2 || !(slope01 > slope02))
                    {
                        row1 = ((row1 + 0.5) as i32 as f32);
                        row2 = ((row2 + 0.5) as i32 as f32);
                        row0 = ((row0 + 0.5) as i32 as f32) - row2;
                        row2 -= row1;
                        let mut row = row1 as i32;

                        while {
                            row2 -= 1.0;
                            row2 >= 0.0
                        } {
                            emit(row as usize, edge_x1 as i32, edge_x0 as i32, z1, depth_dx);
                            edge_x0 += slope01;
                            edge_x1 += slope12;
                            row += 1;
                            z1 += depth_dy;
                        }

                        while {
                            row0 -= 1.0;
                            row0 >= 0.0
                        } {
                            emit(row as usize, edge_x2 as i32, edge_x0 as i32, z1, depth_dx);
                            edge_x0 += slope01;
                            edge_x2 += slope02;
                            row += 1;
                            z1 += depth_dy;
                        }
                    } else {
                        row1 = ((row1 + 0.5) as i32 as f32);
                        row2 = ((row2 + 0.5) as i32 as f32);
                        row0 = ((row0 + 0.5) as i32 as f32) - row2;
                        row2 -= row1;
                        let mut row = row1 as i32;

                        while {
                            row2 -= 1.0;
                            row2 >= 0.0
                        } {
                            emit(row as usize, edge_x0 as i32, edge_x1 as i32, z1, depth_dx);
                            edge_x0 += slope01;
                            edge_x1 += slope12;
                            row += 1;
                            z1 += depth_dy;
                        }

                        while {
                            row0 -= 1.0;
                            row0 >= 0.0
                        } {
                            emit(row as usize, edge_x0 as i32, edge_x2 as i32, z1, depth_dx);
                            edge_x0 += slope01;
                            edge_x2 += slope02;
                            row += 1;
                            z1 += depth_dy;
                        }
                    }
                } else {
                    edge_x2 = edge_x1;
                    if row1 < 0.0 {
                        edge_x2 -= slope01 * row1;
                        edge_x1 -= slope12 * row1;
                        row1 = 0.0;
                    }

                    if row0 < 0.0 {
                        edge_x0 -= slope02 * row0;
                        row0 = 0.0;
                    }

                    row1 = ((row1 + 0.5) as i32 as f32);
                    row0 = ((row0 + 0.5) as i32 as f32);
                    row2 = ((row2 + 0.5) as i32 as f32) - row0;
                    row0 -= row1;
                    let mut row = row1 as i32;
                    if slope01 < slope12 {
                        while {
                            row0 -= 1.0;
                            row0 >= 0.0
                        } {
                            emit(row as usize, edge_x2 as i32, edge_x1 as i32, z1, depth_dx);
                            edge_x2 += slope01;
                            edge_x1 += slope12;
                            row += 1;
                            z1 += depth_dy;
                        }

                        while {
                            row2 -= 1.0;
                            row2 >= 0.0
                        } {
                            emit(row as usize, edge_x0 as i32, edge_x1 as i32, z1, depth_dx);
                            edge_x0 += slope02;
                            edge_x1 += slope12;
                            row += 1;
                            z1 += depth_dy;
                        }
                    } else {
                        while {
                            row0 -= 1.0;
                            row0 >= 0.0
                        } {
                            emit(row as usize, edge_x1 as i32, edge_x2 as i32, z1, depth_dx);
                            edge_x2 += slope01;
                            edge_x1 += slope12;
                            row += 1;
                            z1 += depth_dy;
                        }

                        while {
                            row2 -= 1.0;
                            row2 >= 0.0
                        } {
                            emit(row as usize, edge_x1 as i32, edge_x0 as i32, z1, depth_dx);
                            edge_x0 += slope02;
                            edge_x1 += slope12;
                            row += 1;
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

            z2 = origin(x2, y2, z2, depth_dx, depth_dy);
            if row0 < row1 {
                edge_x1 = edge_x2;
                if row2 < 0.0 {
                    edge_x1 -= slope12 * row2;
                    edge_x2 -= slope02 * row2;
                    row2 = 0.0;
                }

                if row0 < 0.0 {
                    edge_x0 -= slope01 * row0;
                    row0 = 0.0;
                }

                row2 = ((row2 + 0.5) as i32 as f32);
                row0 = ((row0 + 0.5) as i32 as f32);
                row1 = ((row1 + 0.5) as i32 as f32) - row0;
                row0 -= row2;
                let mut row = row2 as i32;
                if slope12 < slope02 {
                    while {
                        row0 -= 1.0;
                        row0 >= 0.0
                    } {
                        emit(row as usize, edge_x1 as i32, edge_x2 as i32, z2, depth_dx);
                        edge_x1 += slope12;
                        edge_x2 += slope02;
                        row += 1;
                        z2 += depth_dy;
                    }

                    while {
                        row1 -= 1.0;
                        row1 >= 0.0
                    } {
                        emit(row as usize, edge_x1 as i32, edge_x0 as i32, z2, depth_dx);
                        edge_x1 += slope12;
                        edge_x0 += slope01;
                        row += 1;
                        z2 += depth_dy;
                    }
                } else {
                    while {
                        row0 -= 1.0;
                        row0 >= 0.0
                    } {
                        emit(row as usize, edge_x2 as i32, edge_x1 as i32, z2, depth_dx);
                        edge_x1 += slope12;
                        edge_x2 += slope02;
                        row += 1;
                        z2 += depth_dy;
                    }

                    while {
                        row1 -= 1.0;
                        row1 >= 0.0
                    } {
                        emit(row as usize, edge_x0 as i32, edge_x1 as i32, z2, depth_dx);
                        edge_x1 += slope12;
                        edge_x0 += slope01;
                        row += 1;
                        z2 += depth_dy;
                    }
                }
            } else {
                edge_x0 = edge_x2;
                if row2 < 0.0 {
                    edge_x0 -= slope12 * row2;
                    edge_x2 -= slope02 * row2;
                    row2 = 0.0;
                }

                if row1 < 0.0 {
                    edge_x1 -= slope01 * row1;
                    row1 = 0.0;
                }

                row2 = ((row2 + 0.5) as i32 as f32);
                row1 = ((row1 + 0.5) as i32 as f32);
                row0 = ((row0 + 0.5) as i32 as f32) - row1;
                row1 -= row2;
                let mut row = row2 as i32;
                if slope12 < slope02 {
                    while {
                        row1 -= 1.0;
                        row1 >= 0.0
                    } {
                        emit(row as usize, edge_x0 as i32, edge_x2 as i32, z2, depth_dx);
                        edge_x0 += slope12;
                        edge_x2 += slope02;
                        row += 1;
                        z2 += depth_dy;
                    }

                    while {
                        row0 -= 1.0;
                        row0 >= 0.0
                    } {
                        emit(row as usize, edge_x1 as i32, edge_x2 as i32, z2, depth_dx);
                        edge_x1 += slope01;
                        edge_x2 += slope02;
                        row += 1;
                        z2 += depth_dy;
                    }
                } else {
                    while {
                        row1 -= 1.0;
                        row1 >= 0.0
                    } {
                        emit(row as usize, edge_x2 as i32, edge_x0 as i32, z2, depth_dx);
                        edge_x0 += slope12;
                        edge_x2 += slope02;
                        row += 1;
                        z2 += depth_dy;
                    }

                    while {
                        row0 -= 1.0;
                        row0 >= 0.0
                    } {
                        emit(row as usize, edge_x2 as i32, edge_x1 as i32, z2, depth_dx);
                        edge_x1 += slope01;
                        edge_x2 += slope02;
                        row += 1;
                        z2 += depth_dy;
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
