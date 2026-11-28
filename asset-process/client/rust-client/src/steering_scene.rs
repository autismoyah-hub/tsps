//! Three-dimensional steering indicators submitted by pj.cb / of.ce.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub position: [i32; 3],
    pub angle: u16,
    pub active: bool,
}
/// The original uses float zoom arithmetic, then double fixed-table trigonometry.
pub fn position(height: i32, focal: i32, focus: [i32; 3], angle: i32) -> [i32; 3] {
    let scale = height as f32 / 334.0;
    let distance = ((1400.0f32 - 4.0f32 * focal as f32 / scale) as i32).max(512);
    let theta = (angle & 2047) as f64 * 0.0030679615757712823;
    let sin = (theta.sin() * 65536.0) as i32;
    let cos = (theta.cos() * 65536.0) as i32;
    [
        focus[0].wrapping_sub((sin as f64 / 65536.0 * distance as f64) as i32),
        focus[1],
        focus[2].wrapping_sub((distance as f64 * (cos as f64 / 65536.0)) as i32),
    ]
}
pub fn placements(
    height: i32,
    focal: i32,
    focus: [i32; 3],
    steering: crate::steering::Steering,
    hovered: Option<u8>,
    ready: [bool; 2],
) -> [Option<Placement>; 2] {
    let active = (steering.remaining > 0).then_some(steering.direction);
    let hovered = hovered.filter(|&d| Some(d) != active);
    [active.filter(|_| ready[1]), hovered.filter(|_| ready[0])].map(|d| {
        d.map(|direction| {
            let angle = direction as u16 * 128;
            Placement {
                position: position(height, focal, focus, angle as i32),
                angle,
                active: Some(direction) == active,
            }
        })
    })
}

/// Graphics defaults name the hover model first and the active model second.
pub fn load(
    cache: &mut crate::disk_cache::DiskCache,
) -> crate::Result<[Option<(crate::model::Model, Vec<[i32; 3]>)>; 2]> {
    let ids = crate::minimap::steering_models(&cache.file(17, 3, 0, [0; 4])?)?;
    let mut models = [None, None];
    for (i, id) in ids.into_iter().enumerate() {
        if let Some(id) = id {
            if cache
                .reference(7)?
                .groups
                .binary_search_by_key(&id, |g| g.id)
                .is_ok()
            {
                let model = crate::model::Model::decode(&cache.file(7, id, 0, [0; 4])?)?;
                let colors = model.light(128, 43690, [0, -1, 0])?;
                models[i] = Some((model, colors));
            }
        }
    }
    Ok(models)
}
