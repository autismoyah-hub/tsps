use crate::{Error, Result};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node {
    pub index: i32,
    pub parent: i32,
    pub kind: u8,
    pub bounds: [i32; 2],
}
pub trait Slots {
    fn length(&self) -> Option<usize>;
    fn node(&self, index: i32) -> Option<Node>;
    fn clear(&mut self, index: i32);
}
/// Java eh.cw scans the selected object's retained bounds against the current array.
pub fn remove(selected: Node, slots: &mut impl Slots) -> Result<()> {
    let length = slots
        .length()
        .ok_or(Error::Invalid("delete widget child array"))?;
    if selected.kind == 0 {
        for index in selected.bounds[0]..=selected.bounds[1] {
            if usize::try_from(index)
                .ok()
                .is_none_or(|index| index >= length)
            {
                return Err(Error::Invalid("delete widget array bounds"));
            }
            if let Some(child) = slots
                .node(index)
                .filter(|child| child.parent == selected.index)
            {
                remove(child, slots)?;
            }
        }
    }
    if usize::try_from(selected.index)
        .ok()
        .is_none_or(|index| index >= length)
    {
        return Err(Error::Invalid("delete widget array bounds"));
    }
    slots.clear(selected.index);
    Ok(())
}
