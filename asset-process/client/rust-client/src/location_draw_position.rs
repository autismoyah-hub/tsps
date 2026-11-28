//! Scene renderable center, separate from definition-sized terrain contouring.
//! GameScene.bv/bp/an/bi use one tile except game-object shapes 10 and 11.
pub fn center(shape: u8, tile: [i32; 2], rotated_size: [u8; 2]) -> [i32; 2] {
    let size = if matches!(shape, 10 | 11) {
        rotated_size
    } else {
        [1; 2]
    };
    std::array::from_fn(|axis| {
        tile[axis]
            .wrapping_mul(128)
            .wrapping_add(i32::from(size[axis]) * 64)
    })
}
