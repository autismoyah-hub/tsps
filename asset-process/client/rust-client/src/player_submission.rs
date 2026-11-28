//! Relative player submission order from yi.cc/qu.ck in root and child scenes.
pub fn ordered(
    high: &[u16],
    local: u16,
    target: Option<u16>,
    render_self: bool,
    mut visible: impl FnMut(u16) -> bool,
) -> Vec<u16> {
    let mut ids = Vec::new();
    if render_self && visible(local) {
        ids.push(local);
    }
    if let Some(target) = target {
        if visible(target) {
            ids.push(target);
        }
    }
    ids.extend(
        high.iter()
            .copied()
            .filter(|&index| index != local && Some(index) != target && visible(index)),
    );
    ids
}
