//! Revision 239 widget drag geometry and release decisions (ps.am).
#[derive(Clone, Debug)]
pub struct Motion {
    pub age: i32,
    pub dragging: bool,
    pub active: bool,
    pub offset: [i32; 2],
    pub origin: [i32; 2],
    pub dead_zone: i32,
    pub dead_time: i32,
}
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub position: [i32; 2],
    pub mouse: [i32; 2],
    pub outcome: u8,
}
impl Motion {
    pub fn step(
        &mut self,
        pointer: [i32; 2],
        size: [i32; 2],
        parent: [i32; 4],
        scroll: [i32; 2],
        visible: [bool; 2],
        held: bool,
        has_menu: bool,
    ) -> Frame {
        let mut frame = Frame {
            position: self.origin,
            mouse: [0; 2],
            outcome: 0,
        };
        if !self.active {
            return frame;
        }
        self.age = self.age.wrapping_add(1);
        if visible[0] && visible[1] {
            for axis in 0..2 {
                let mut p = pointer[axis].wrapping_sub(self.offset[axis]);
                if p < parent[axis] {
                    p = parent[axis];
                }
                if p.wrapping_add(size[axis]) > parent[axis].wrapping_add(parent[axis + 2]) {
                    p = parent[axis]
                        .wrapping_add(parent[axis + 2])
                        .wrapping_sub(size[axis]);
                }
                frame.position[axis] = p;
                frame.mouse[axis] = p.wrapping_sub(parent[axis]).wrapping_add(scroll[axis]);
            }
            let delta = [
                frame.position[0].wrapping_sub(self.origin[0]),
                frame.position[1].wrapping_sub(self.origin[1]),
            ];
            if self.age > self.dead_time
                && delta
                    .iter()
                    .any(|&v| v > self.dead_zone || v < self.dead_zone.wrapping_neg())
            {
                self.dragging = true;
            }
            if !held {
                self.active = false;
                frame.outcome = if self.dragging { 2 } else { 1 };
                if !self.dragging {
                    frame.mouse =
                        std::array::from_fn(|a| self.offset[a].wrapping_add(self.origin[a]));
                }
            }
        } else if self.age > 1 {
            if !self.dragging && has_menu {
                frame.outcome = 3;
                frame.mouse = std::array::from_fn(|a| self.offset[a].wrapping_add(self.origin[a]));
            }
            self.active = false;
        }
        frame
    }
}

pub struct Drag {
    /// Object generations captured at drag start: source, then parent.
    pub instances: [u64; 2],
    pub widget: u32,
    pub parent: u32,
    pub motion: Motion,
    pub entries: Vec<crate::context_menu::Entry<crate::context_menu::Action>>,
    pub point: [i32; 2],
}
