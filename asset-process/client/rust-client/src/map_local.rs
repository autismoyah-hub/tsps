//! Original po.ez map coordinate, including its root-base subtraction.
pub fn coordinate(
    root_base: [i32; 2],
    actor: Option<crate::camera_focus::Focus>,
    entities: Option<&crate::world_entity::State>,
) -> [i32; 3] {
    let Some(actor) = actor else {
        return [-1, 0, 0];
    };
    let projected = actor.projected(entities);
    [
        projected.plane as i32,
        (projected.fine[0] >> 7).wrapping_sub(root_base[0]),
        (projected.fine[1] >> 7).wrapping_sub(root_base[1]),
    ]
}
