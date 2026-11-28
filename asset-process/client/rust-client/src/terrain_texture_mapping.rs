//! Original GameProjection tile texture bases. Paints keep their own first-face order.
pub fn indices(paint: bool, face: [usize; 3], flat: bool) -> [usize; 3] {
    if flat {
        [0, 1, 3]
    } else if paint {
        if face.contains(&0) {
            [0, 1, 3]
        } else {
            [2, 3, 1]
        }
    } else {
        face
    }
}
