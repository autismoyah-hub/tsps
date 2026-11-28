//! Revision 239 menu geometry, row selection and software drawing.
use crate::{
    font::{Font, Style},
    raster::Raster,
    Result,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Map { opcode: u16, event: crate::map_event::Event },
    InView {
        view: u16,
        target: WorldAction,
    },
    Cancel,
    WorldEntity {
        id: u16,
        operation: u8,
    },
    SelectSpell(u32),
    Button {
        id: u32,
        kind: u8,
    },
    Continue(u32),
    Close,
    CastInterface(u32),
    CastWorld {
        kind: u8,
        id: u16,
        tile: [u16; 2],
    },
    Interface {
        id: u32,
        operation: u8,
        sub: Option<u8>,
    },
    Npc {
        index: u16,
        operation: u8,
        sub: u8,
    },
    Ground {
        id: u16,
        tile: [u16; 2],
        operation: u8,
        sub: u8,
    },
    Object {
        id: u16,
        tile: [u16; 2],
        operation: u8,
        sub: u8,
    },
    Walk([u16; 2]),
    Heading(u8),
    Player {
        index: u16,
        operation: u8,
    },
    ExamineNpc(u16),
    Examine {
        kind: u8,
        id: u16,
        tile: [u16; 2],
    },
}
/// A scene action retains its owning view while a menu remains open. Root
/// variants remain available for interface scripts and existing callers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldAction {
    Npc {
        index: u16,
        operation: u8,
        sub: u8,
    },
    Player {
        index: u16,
        operation: u8,
    },
    Object {
        id: u16,
        tile: [u16; 2],
        operation: u8,
        sub: u8,
    },
    Ground {
        id: u16,
        tile: [u16; 2],
        operation: u8,
        sub: u8,
    },
    Cast {
        kind: u8,
        id: u16,
        tile: [u16; 2],
    },
    ExamineNpc(u16),
    Examine {
        kind: u8,
        id: u16,
        tile: [u16; 2],
    },
    Walk([u16; 2]),
    Heading(u8),
    Entity {
        id: u16,
        operation: u8,
    },
}
impl WorldAction {
    pub fn action(self) -> Action {
        match self {
            Self::Npc {
                index,
                operation,
                sub,
            } => Action::Npc {
                index,
                operation,
                sub,
            },
            Self::Player { index, operation } => Action::Player { index, operation },
            Self::Object {
                id,
                tile,
                operation,
                sub,
            } => Action::Object {
                id,
                tile,
                operation,
                sub,
            },
            Self::Ground {
                id,
                tile,
                operation,
                sub,
            } => Action::Ground {
                id,
                tile,
                operation,
                sub,
            },
            Self::Cast { kind, id, tile } => Action::CastWorld { kind, id, tile },
            Self::ExamineNpc(index) => Action::ExamineNpc(index),
            Self::Examine { kind, id, tile } => Action::Examine { kind, id, tile },
            Self::Walk(tile) => Action::Walk(tile),
            Self::Heading(direction) => Action::Heading(direction),
            Self::Entity { id, operation } => Action::WorldEntity { id, operation },
        }
    }
}
impl Action {
    pub fn in_view(self, view: u16) -> Self {
        let target = match self {
            Self::InView { target, .. } => target,
            Self::Npc {
                index,
                operation,
                sub,
            } => WorldAction::Npc {
                index,
                operation,
                sub,
            },
            Self::Player { index, operation } => WorldAction::Player { index, operation },
            Self::Object {
                id,
                tile,
                operation,
                sub,
            } => WorldAction::Object {
                id,
                tile,
                operation,
                sub,
            },
            Self::Ground {
                id,
                tile,
                operation,
                sub,
            } => WorldAction::Ground {
                id,
                tile,
                operation,
                sub,
            },
            Self::CastWorld { kind, id, tile } => WorldAction::Cast { kind, id, tile },
            Self::ExamineNpc(index) => WorldAction::ExamineNpc(index),
            Self::Examine { kind, id, tile } => WorldAction::Examine { kind, id, tile },
            Self::Walk(tile) => WorldAction::Walk(tile),
            Self::Heading(direction) => WorldAction::Heading(direction),
            Self::WorldEntity { id, operation } => WorldAction::Entity { id, operation },
            _ => return self,
        };
        if view == 0 {
            target.action()
        } else {
            Self::InView { view, target }
        }
    }
    pub fn scene(self) -> (u16, Self) {
        match self {
            Self::InView { view, target } => (view, target.action()),
            _ => (0, self),
        }
    }
}
#[derive(Clone)]
pub struct Entry<T> {
    pub option: String,
    pub target: String,
    pub action: T,
    pub submenu: bool,
    pub children: Vec<Entry<T>>,
}
impl<T> Entry<T> {
    pub fn label(&self) -> String {
        let mut text = self.option.clone();
        if !self.target.is_empty() {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(&self.target);
        }
        if self.submenu || !self.children.is_empty() {
            text.push_str(" </col><gt>");
        }
        text
    }
}
pub struct Menu<T> {
    /// Entries retain original insertion order; the last entry is drawn first.
    pub entries: Vec<Entry<T>>,
    pub bounds: [i32; 4],
    pub scroll: i32,
    pub active: Option<(usize, Box<Menu<T>>)>,
    title: String,
    icons: std::sync::Arc<Vec<crate::sprite::SpriteSheet>>,
    candidate: Option<usize>,
    due: i64,
    previous: [i32; 2],
}
impl<T> Menu<T> {
    pub fn open(entries: Vec<Entry<T>>, point: [i32; 2], canvas: [i32; 2], font: &Font) -> Self {
        Self::open_with_icons(entries, point, canvas, font, Default::default())
    }
    pub fn open_with_icons(
        entries: Vec<Entry<T>>,
        point: [i32; 2],
        canvas: [i32; 2],
        font: &Font,
        icons: std::sync::Arc<Vec<crate::sprite::SpriteSheet>>,
    ) -> Self {
        Self::open_title(entries, point, canvas, font, "Choose Option".into(), icons)
    }
    fn open_title(
        entries: Vec<Entry<T>>,
        point: [i32; 2],
        canvas: [i32; 2],
        font: &Font,
        title: String,
        icons: std::sync::Arc<Vec<crate::sprite::SpriteSheet>>,
    ) -> Self {
        let mut width = font.metrics.width(&title, &icons);
        for entry in &entries {
            let mut label = entry.option.clone();
            if !entry.target.is_empty() {
                label.push(' ');
                label.push_str(&entry.target);
            }
            if entry.submenu || !entry.children.is_empty() {
                label.push_str(" <gt>");
            }
            width = width.max(font.metrics.width(&label, &icons));
        }
        width += 8;
        let height = entries.len() as i32 * 15 + 22;
        let x = (point[0] - width / 2).min(canvas[0] - width).max(0);
        let y = point[1].min(canvas[1] - height).max(0);
        Self {
            entries,
            bounds: [x, y, width, height],
            scroll: 0,
            active: None,
            title,
            icons,
            candidate: None,
            due: -1,
            previous: [-1, -1],
        }
    }
    pub fn row_y(&self, index: usize) -> i32 {
        self.bounds[1] + (self.entries.len() as i32 - 1 - index as i32 - self.scroll) * 15 + 31
    }
    pub fn select(&self, point: [i32; 2]) -> Option<usize> {
        let [x, _, width, _] = self.bounds;
        (0..self.entries.len()).find(|&index| {
            let y = self.row_y(index);
            point[0] > x && point[0] < x + width && point[1] > y - 13 && point[1] < y + 3
        })
    }
    pub fn outside_margin(&self, point: [i32; 2]) -> bool {
        if self
            .active
            .as_ref()
            .is_some_and(|(_, menu)| !menu.outside_margin(point))
        {
            return false;
        }
        let [x, y, w, h] = self.bounds;
        point[0] < x - 10 || point[0] > x + w + 10 || point[1] < y - 10 || point[1] > y + h + 10
    }
    /// GameClient.gk gives a scrollable immediate child priority in its margin.
    pub fn scroll_wheel(&mut self, point: [i32; 2], wheel: i32, canvas_height: i32) {
        let inside = |bounds: [i32; 4]| {
            let [x, y, w, h] = bounds;
            point[0] >= x - 10
                && point[0] <= x + w + 10
                && point[1] >= y - 10
                && point[1] <= y + h + 10
        };
        let limit = |height: i32| ((height - canvas_height + 14) / 15).max(0);
        if let Some((_, child)) = &mut self.active {
            let max = limit(child.bounds[3]);
            if max > 0 && inside(child.bounds) {
                child.scroll = child.scroll.wrapping_add(wheel).clamp(0, max);
                return;
            }
        }
        let max = limit(self.bounds[3]);
        if max > 0 && inside(self.bounds) {
            self.scroll = self.scroll.wrapping_add(wheel).clamp(0, max);
        }
    }
    pub fn draw(&self, raster: &mut Raster, font: &Font, pointer: [i32; 2]) -> Result<()> {
        let [x, y, w, h] = self.bounds;
        raster.fill_rect(x, y, w, h, 6116423, 256);
        raster.fill_rect(x + 1, y + 1, w - 2, 16, 0, 256);
        raster.fill_rect(x + 1, y + 18, w - 2, 1, 0, 256);
        raster.fill_rect(x + 1, y + h - 2, w - 2, 1, 0, 256);
        raster.fill_rect(x + 1, y + 18, 1, h - 19, 0, 256);
        raster.fill_rect(x + w - 2, y + 18, 1, h - 19, 0, 256);
        font.draw(
            raster,
            &self.title,
            x + 3,
            y + 14,
            Style {
                color: 6116423,
                shadow: None,
                alpha: 256,
            },
            &self.icons,
        )?;
        for (index, entry) in self.entries.iter().enumerate() {
            if self.entries.len() as i32 - 1 - (index as i32) < self.scroll {
                continue;
            }
            let baseline = self.row_y(index);
            let hover = pointer[0] > x
                && pointer[0] < x + w
                && pointer[1] > baseline - 13
                && pointer[1] < baseline + 3;
            font.draw(
                raster,
                &entry.label(),
                x + 3,
                baseline,
                Style {
                    color: if hover { 0xffff00 } else { 0xffffff },
                    shadow: Some(0),
                    alpha: 256,
                },
                &self.icons,
            )?;
        }
        if let Some((_, child)) = &self.active {
            child.draw(raster, font, pointer)?;
        }
        Ok(())
    }
    pub fn selected_action(&self, point: [i32; 2]) -> Option<&T> {
        self.active
            .as_ref()
            .and_then(|(_, child)| child.selected_action(point))
            .or_else(|| self.select(point).map(|i| &self.entries[i].action))
    }
}
impl<T: Clone> Menu<T> {
    pub fn open_child(&mut self, index: usize, canvas: [i32; 2], font: &Font) {
        self.active = None;
        let Some(entry) = self.entries.get(index).filter(|e| !e.children.is_empty()) else {
            return;
        };
        let mut child = Self::open_title(
            entry.children.clone(),
            [0, 0],
            canvas,
            font,
            entry.target.clone(),
            self.icons.clone(),
        );
        let [x, y, w, _] = self.bounds;
        child.bounds[0] = if x + w + child.bounds[2] > canvas[0] {
            (x - child.bounds[2]).max(0)
        } else {
            x + w
        };
        child.bounds[1] = (y + (self.entries.len() as i32 - 1 - index as i32 - self.scroll) * 15)
            .min(canvas[1] - child.bounds[3])
            .max(0);
        self.active = Some((index, Box::new(child)));
    }
    fn toward_child(&self, point: [i32; 2]) -> bool {
        let Some((_, child)) = &self.active else {
            return false;
        };
        let slope = |p: [i32; 2], x: i32, y: i32| (p[1] as f32 - y as f32) / (x - p[0]) as f32;
        let [x, y, _, height] = child.bounds;
        let right = x > self.bounds[0];
        let edge = if right { x } else { self.bounds[0] };
        let a = slope(point, edge, y);
        let b = slope(self.previous, edge, y);
        let c = slope(point, edge, y + height);
        let d = slope(self.previous, edge, y + height);
        if right {
            (a >= b && c < d) || (a > b && c <= d)
        } else {
            (a <= b && c > d) || (a < b && c >= d)
        }
    }
    /// Injected GameMenu.ap uses game cycles, with a two/eight-cycle change delay.
    pub fn hover(&mut self, point: [i32; 2], cycle: i64, canvas: [i32; 2], font: &Font) -> bool {
        if self
            .active
            .as_mut()
            .is_some_and(|(_, child)| child.hover(point, cycle, canvas, font))
        {
            self.previous = point;
            return true;
        }
        let [x, y, w, h] = self.bounds;
        let inside = point[0] >= x - 10
            && point[0] <= x + w + 10
            && point[1] >= y - 10
            && point[1] <= y + h + 10;
        if inside {
            if let Some(index) = self
                .select(point)
                .filter(|i| self.active.as_ref().is_none_or(|(current, _)| i != current))
            {
                let mut candidate = Some(index);
                let mut due = cycle.max(self.due);
                if self.active.is_some() {
                    if self.toward_child(point) {
                        candidate = None;
                        due = i64::MAX;
                    } else if self.candidate.is_none() {
                        due = cycle + if point == self.previous { 8 } else { 2 };
                    }
                }
                self.candidate = candidate;
                self.due = due;
                if self.due <= cycle {
                    self.candidate = None;
                    self.open_child(index, canvas, font);
                }
            }
        }
        self.previous = point;
        inside
    }
}
