//! Original decoration placement offsets (GameScene.bp callers ds.cz/yk.eh).
//! Widths are scene entries present when placement occurs, not later mesh state.
pub fn placement(
    shape: u8,
    rotation: u8,
    boundary_width: Option<u8>,
    object_width: Option<u8>,
) -> [[i32; 2]; 2] {
    let r = usize::from(rotation & 3);
    let opposite = (r + 2) & 3;
    let cardinal = [[1, 0], [0, -1], [-1, 0], [0, 1]];
    let diagonal = [[1, -1], [-1, -1], [-1, 1], [1, 1]];
    match shape {
        4 => [cardinal[r], [0, 0]],
        5 => [
            cardinal[r].map(|v| v * (i32::from(boundary_width.unwrap_or(16)) + 1)),
            [0, 0],
        ],
        6 => [
            diagonal[r].map(|v| v * (i32::from(object_width.unwrap_or(16)) / 2 + 1)),
            [0, 0],
        ],
        7 => [diagonal[opposite], [0, 0]],
        8 => [
            diagonal[r].map(|v| v * (i32::from(object_width.unwrap_or(16)) / 2 + 1)),
            diagonal[opposite],
        ],
        _ => [[0, 0]; 2],
    }
}
/// GameScene.tx changes only nonzero coordinates of the primary part.
pub fn adjust_primary(offsets: &mut [[i32; 2]; 2], displacement: i32) {
    for coordinate in &mut offsets[0] {
        if *coordinate != 0 {
            *coordinate = if *coordinate < 0 {
                displacement.wrapping_neg()
            } else {
                displacement
            };
        }
    }
}

#[derive(Default)]
struct Slot {
    boundary_width: Option<u8>,
    object_width: Option<u8>,
    decoration: Option<usize>,
}
/// Transient map construction state; offsets themselves persist with placements.
#[derive(Default)]
pub(crate) struct Construction {
    slots: std::collections::HashMap<[usize; 3], Slot>,
}
impl Construction {
    pub(crate) fn place(
        &mut self,
        object: &crate::scene::PlacedObject,
        width: u8,
        index: usize,
        offsets: &mut [Option<[[i32; 2]; 2]>],
    ) {
        let slot = self
            .slots
            .entry([object.plane as usize, object.x as usize, object.y as usize])
            .or_default();
        if !object.rejected {
            match object.shape {
                0..=3 => slot.boundary_width = Some(width),
                4..=8 => {
                    offsets[index] = Some(placement(
                        object.shape,
                        object.rotation,
                        slot.boundary_width,
                        slot.object_width,
                    ));
                    slot.decoration = Some(index);
                }
                9..=21 => {
                    slot.object_width.get_or_insert(width);
                }
                _ => {}
            }
        }
        if !object.runtime && width != 16 && matches!(object.shape, 0 | 2 | 9) {
            if let Some(value) = slot.decoration.and_then(|i| offsets[i].as_mut()) {
                adjust_primary(
                    value,
                    if object.shape == 9 {
                        i32::from(width) / 2 + 1
                    } else {
                        i32::from(width) + 1
                    },
                );
            }
        }
    }
    pub(crate) fn lower_bridge(&mut self, x: usize, z: usize) {
        self.slots.remove(&[0, x, z]);
        for plane in 1..4 {
            if let Some(slot) = self.slots.remove(&[plane, x, z]) {
                self.slots.insert([plane - 1, x, z], slot);
            }
        }
    }
}
