//! Original ko.ay world-selector layout and GameFriendsChatMember.aq input.
use crate::{
    disk_cache::DiskCache,
    font::{Font, Style},
    raster::Raster,
    sprite::SpriteSheet,
    sprite_pixels::Pixels,
    world_list::List,
    Result,
};

pub struct Assets {
    back: Vec<Pixels>,
    highlighted: Vec<Pixels>,
    flags: Vec<Pixels>,
    arrows: Vec<Pixels>,
    stars: Vec<Pixels>,
    left: Pixels,
    right: Pixels,
    bold: Font,
    small: Font,
}
#[derive(Default, Debug)]
pub struct State {
    pub page: usize,
    pub max_page: usize,
    pub hovered: Option<usize>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Close,
    Select(usize),
    Sort(i32, i32),
}
fn sheet(bytes: &[u8]) -> Result<Vec<Pixels>> {
    let s = SpriteSheet::decode(bytes, 1 << 20)?;
    (0..s.sprites.len())
        .map(|i| Pixels::from_sheet(&s, i))
        .collect()
}
impl Assets {
    pub fn load(cache: &mut DiskCache) -> Result<Self> {
        let mut bytes = Vec::new();
        for name in [
            "sl_back",
            "sl_flags",
            "sl_arrows",
            "sl_stars",
            "leftarrow",
            "rightarrow",
        ] {
            bytes.push(cache.named_file(8, name, "")?);
        }
        for name in ["b12_full", "p11_full"] {
            let id = cache
                .reference(13)?
                .groups
                .iter()
                .find(|g| g.name_hash == Some(crate::disk_cache::name_hash(name)))
                .ok_or(crate::Error::Invalid("world selector font"))?
                .id;
            bytes.push(cache.file(13, id, 0, [0; 4])?);
            bytes.push(cache.file(8, id, 0, [0; 4])?);
        }
        Self::decode(std::array::from_fn(|i| bytes[i].as_slice()))
    }
    pub fn decode(bytes: [&[u8]; 10]) -> Result<Self> {
        let back = sheet(bytes[0])?;
        // GameSpritePixels.zx's fixed white/128 tint is independent of destination pixels.
        let highlighted = back
            .iter()
            .cloned()
            .map(|mut sprite| {
                for p in &mut sprite.pixels {
                    if *p != 0 {
                        *p = ((*p & 0xfefefe) >> 1) + 0x7f7f7f;
                    }
                }
                sprite
            })
            .collect();
        let flags = sheet(bytes[1])?;
        let arrows = sheet(bytes[2])?;
        let stars = sheet(bytes[3])?;
        let left = sheet(bytes[4])?
            .into_iter()
            .next()
            .ok_or(crate::Error::Invalid("world selector left arrow"))?;
        let right = sheet(bytes[5])?
            .into_iter()
            .next()
            .ok_or(crate::Error::Invalid("world selector right arrow"))?;
        if back.len() < 2 || arrows.len() < 4 || stars.len() < 2 {
            return Err(crate::Error::Invalid("world selector sprite count"));
        }
        Ok(Self {
            back,
            highlighted,
            flags,
            arrows,
            stars,
            left,
            right,
            bold: Font::decode(bytes[6], bytes[7])?,
            small: Font::decode(bytes[8], bytes[9])?,
        })
    }
    pub fn input(
        &self,
        state: &mut State,
        input: &crate::input::Input,
        dimensions: [i32; 2],
        mouse_camera: bool,
    ) -> Option<Action> {
        for &(key, _) in &input.key_events {
            match key {
                13 => return Some(Action::Close),
                96 if state.page > 0 => state.page -= 1,
                97 if state.page < state.max_page => state.page += 1,
                _ => (),
            }
        }
        if input.pressed_buttons & 1 == 0 && (mouse_camera || input.pressed_buttons & 4 == 0) {
            return None;
        }
        let [width, height] = dimensions;
        let left = (width - 765) / 2;
        let [x, y] = input.press_position;
        for (column, offset) in [280, 390, 500, 610].into_iter().enumerate() {
            if (4..=18).contains(&y) {
                if (left + offset..=left + offset + 14).contains(&x) {
                    return Some(Action::Sort(column as i32, 0));
                }
                if (left + offset + 15..=left + offset + 80).contains(&x) {
                    return Some(Action::Sort(column as i32, 1));
                }
            }
        }
        if (left + 708..=left + 758).contains(&x) && (4..=20).contains(&y) {
            return Some(Action::Close);
        }
        if let Some(index) = state.hovered {
            return Some(Action::Select(index));
        }
        if (height / 2 - 50..=height / 2 + 50).contains(&y) {
            if state.page > 0 && (0..=self.left.size[0] as i32).contains(&x) {
                state.page -= 1;
            }
            if state.page < state.max_page
                && (width - self.right.size[0] as i32 - 5..=width).contains(&x)
            {
                state.page += 1;
            }
        }
        None
    }
    pub fn draw(
        &self,
        raster: &mut Raster,
        list: &List,
        state: &mut State,
        pointer: [i32; 2],
    ) -> Result<()> {
        let left = (raster.width() as i32 - 765) / 2;
        raster.fill_rect(left, 23, 765, 480, 0, 256);
        raster.fill_gradient([left, 0, 125, 23], [12425273, 9135624], None);
        raster.fill_gradient([left + 125, 0, 640, 23], [5197647, 2697513], None);
        text(
            &self.bold,
            raster,
            "Select a world",
            [left + 62, 15],
            0,
            true,
        )?;
        self.stars[1].draw(raster, left + 140, 1, 256)?;
        self.stars[0].draw(raster, left + 140, 12, 256)?;
        text(
            &self.small,
            raster,
            "Members only world",
            [left + 152, 10],
            0xffffff,
            false,
        )?;
        text(
            &self.small,
            raster,
            "Free world",
            [left + 152, 21],
            0xffffff,
            false,
        )?;
        for (column, (x, name)) in [
            (280, "World"),
            (390, "Players"),
            (500, "Location"),
            (610, "Type"),
        ]
        .into_iter()
        .enumerate()
        {
            let selected = list.sort_keys[0] == column as i32;
            self.arrows[if selected && list.sort_directions[0] == 0 {
                2
            } else {
                0
            }]
            .draw(raster, left + x, 4, 256)?;
            self.arrows[if selected && list.sort_directions[0] == 1 {
                3
            } else {
                1
            }]
            .draw(raster, left + x + 15, 4, 256)?;
            text(
                &self.bold,
                raster,
                name,
                [left + x + 32, 17],
                0xffffff,
                false,
            )?;
        }
        raster.fill_rect(left + 708, 4, 50, 16, 0, 256);
        text(
            &self.small,
            raster,
            "Cancel",
            [left + 733, 16],
            0xffffff,
            true,
        )?;
        state.hovered = None;
        // A configured empty custom list is valid; the original assumes at least one world.
        if list.worlds.is_empty() {
            state.page = 0;
            state.max_page = 0;
            return Ok(());
        }
        let count = list.worlds.len();
        let (mut columns, mut rows) = (7, 24);
        loop {
            let old = (columns, rows);
            if rows * (columns - 1) >= count {
                columns -= 1;
            }
            if (rows - 1) * columns >= count {
                rows -= 1;
            }
            if (rows - 1) * columns >= count {
                rows -= 1;
            }
            if old == (columns, rows) {
                break;
            }
        }
        let gx = ((765 - columns * 88) / (columns + 1)).min(5);
        let gy = ((480 - rows * 19) / (rows + 1)).min(5);
        let x0 = left + ((765 - columns * 88 - gx * (columns - 1)) / 2) as i32;
        let y0 = 23 + ((480 - rows * 19 - gy * (rows - 1)) / 2) as i32;
        state.max_page = count.div_ceil(rows) - columns;
        state.page = state.page.min(state.max_page);
        if state.page > 0 {
            self.left.draw(
                raster,
                8,
                raster.height() as i32 / 2 - self.left.size[1] as i32 / 2,
                256,
            )?;
        }
        if state.page < state.max_page {
            self.right.draw(
                raster,
                raster.width() as i32 - self.right.size[0] as i32 - 8,
                raster.height() as i32 / 2 - self.right.size[1] as i32 / 2,
                256,
            )?;
        }
        for (slot, (index, world)) in list
            .worlds
            .iter()
            .enumerate()
            .skip(rows * state.page)
            .take(rows * columns)
            .enumerate()
        {
            let x = x0 + (slot / rows * (88 + gx)) as i32;
            let y = y0 + (slot % rows * (19 + gy)) as i32;
            let available = world.population != -1 && world.population <= 1980;
            let hovered = available
                && pointer[0] >= x
                && pointer[0] < x + 88
                && pointer[1] >= y
                && pointer[1] < y + 19;
            let (mut tile, color) = tile(world.properties);
            if tile >= self.back.len() {
                tile = (world.properties & 1) as usize;
            }
            (if hovered {
                &self.highlighted
            } else {
                &self.back
            })[tile]
                .draw(raster, x, y, 256)?;
            if hovered {
                state.hovered = Some(index);
            }
            let flag = if world.properties & 1 != 0 {
                self.flags.len() / 2
            } else {
                0
            };
            if let Some(flag) = usize::try_from(world.location)
                .ok()
                .and_then(|n| self.flags.get(flag + n))
            {
                flag.draw(raster, x + 29, y, 256)?;
            }
            text(
                &self.bold,
                raster,
                &world.id.to_string(),
                [x + 15, y + 14],
                color,
                true,
            )?;
            let population = match world.population {
                -1 => "OFF".into(),
                n if n > 1980 => "FULL".into(),
                n => n.to_string(),
            };
            text(
                &self.small,
                raster,
                &population,
                [x + 60, y + 14],
                0xffffff,
                true,
            )?;
        }
        if let Some(index) = state.hovered {
            let activity = &list.worlds[index].activity;
            let width = self.small.metrics.width(activity, &[]) + 6;
            let height = self.small.metrics.ascent + 8;
            let mut y = pointer[1] + 25;
            if y + height > 480 {
                y = pointer[1] - 25 - height;
            }
            raster.fill_rect(pointer[0] - width / 2, y, width, height, 16777120, 256);
            raster.widget_rect([pointer[0] - width / 2, y, width, height], 0, 256, false);
            text(
                &self.small,
                raster,
                activity,
                [pointer[0], y + self.small.metrics.ascent + 4],
                0,
                true,
            )?;
        }
        Ok(())
    }
}
fn text(
    font: &Font,
    raster: &mut Raster,
    value: &str,
    [x, y]: [i32; 2],
    color: u32,
    center: bool,
) -> Result<()> {
    font.draw(
        raster,
        value,
        x - if center {
            font.metrics.width(value, &[]) / 2
        } else {
            0
        },
        y,
        Style {
            color,
            shadow: None,
            alpha: 256,
        },
        &[],
    )?;
    Ok(())
}
fn tile(properties: i32) -> (usize, u32) {
    let member = (properties & 1) as usize;
    for (flag, base, color) in [
        (33554432, 6, 0),
        (65536, 14, 0),
        (536870912, 4, 0xff0000),
        (1073741824, 8, 0),
        (4, 2, 0),
        (256, 10, 0),
        (134217728, 12, 0),
    ] {
        if properties & flag != 0 {
            return (base + member, color);
        }
    }
    (member, 0)
}
