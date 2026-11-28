//! Actor speech lifetime, coloring, overlap resolution and font effects.
use crate::{
    font::{Font, Style},
    raster::Raster,
    Result,
};
const PATTERN_COLORS: &[u32] = &[
    16777215, 14942979, 16747520, 16772352, 32806, 2375822, 7547266, 16720268, 11884176, 5261772,
    10724259, 13970688, 15693351, 16577588, 494960, 2208255, 10178454, 16756679, 13722276, 8105443,
    16751190, 2543274, 7591918, 10246609, 10021057, 11894492, 2894892, 9699842, 6371605, 13680896,
    4882723, 14504, 8388736, 14025328, 10682978, 4004472,
];
#[derive(Clone, Debug, Default)]
pub struct Speech {
    pub text: Option<String>,
    pub remaining: i32,
    pub color: u8,
    pub effect: u8,
    pub automatic: bool,
    pub filtered: bool,
    pub pattern: Option<Vec<u32>>,
}
impl Speech {
    pub fn tick(&mut self) {
        if self.remaining > 0 {
            self.remaining -= 1;
            if self.remaining == 0 {
                self.text = None;
            }
        }
    }
    pub fn say(&mut self, text: &str, player: bool) {
        self.text = Some(
            if player {
                text.strip_prefix('~').unwrap_or(text)
            } else {
                text
            }
            .to_owned(),
        );
        self.remaining = if player { 150 } else { 100 };
        if player {
            self.automatic = false;
            self.color = 0;
            self.effect = 0;
        }
    }
    pub fn visible(&self, filter: i32, friend: bool) -> bool {
        !self.filtered
            && (filter == 4
                || (!self.automatic && (filter == 0 || filter == 3 || (filter == 1 && friend))))
    }
}
pub fn pattern(bytes: &[u8]) -> Option<Vec<u32>> {
    if bytes.is_empty() || bytes.len() > 8 {
        return None;
    }
    bytes
        .iter()
        .map(|&i| PATTERN_COLORS.get(i as usize).copied())
        .collect()
}
#[derive(Clone, Debug)]
pub struct Entry {
    pub position: [i32; 2],
    pub text: String,
    pub color: u8,
    pub effect: u8,
    pub remaining: i32,
    pub pattern: Option<Vec<u32>>,
}
pub fn color(kind: u8, cycle: i32, remaining: i32) -> u32 {
    let age = 150 - remaining;
    match kind {
        0 => 0xffff00,
        1 => 0xff0000,
        2 => 0x00ff00,
        3 => 0x00ffff,
        4 => 0xff00ff,
        5 => 0xffffff,
        6 => {
            if cycle % 20 < 10 {
                0xff0000
            } else {
                0xffff00
            }
        }
        7 => {
            if cycle % 20 < 10 {
                0x0000ff
            } else {
                0x00ffff
            }
        }
        8 => {
            if cycle % 20 < 10 {
                45056
            } else {
                8454016
            }
        }
        9 if age < 50 => (0xff0000 + 1280 * age) as u32,
        9 if age < 100 => (0xffff00 - (age - 50) * 327680) as u32,
        9 if age < 150 => (65280 + (age - 100) * 5) as u32,
        10 if age < 50 => (0xff0000 + age * 5) as u32,
        10 if age < 100 => (0xff00ff - (age - 50) * 327680) as u32,
        10 if age < 150 => (255 + (age - 100) * 327680 - (age - 100) * 5) as u32,
        11 if age < 50 => (0xffffff - age * 327685) as u32,
        11 if age < 100 => (65280 + (age - 50) * 327685) as u32,
        11 if age < 150 => (0xffffff - (age - 100) * 327680) as u32,
        _ => 0xffff00,
    }
}
fn rainbow(hue: usize) -> u32 {
    let hue = hue as f64 / 64.0 + 0.0078125;
    let saturation = 0.9375;
    let light = 0.5;
    let high = saturation + light - saturation * light;
    let low = 2.0 * light - high;
    let channel = |mut h: f64| {
        if h > 1.0 {
            h -= 1.0;
        }
        if h < 0.0 {
            h += 1.0;
        }
        let v = if h * 6.0 < 1.0 {
            low + (high - low) * 6.0 * h
        } else if h * 2.0 < 1.0 {
            high
        } else if h * 3.0 < 2.0 {
            low + (high - low) * (0.6666666666666666 - h) * 6.0
        } else {
            low
        };
        (v.powf(0.95f32 as f64) * 256.0) as u32
    };
    ((channel(hue + 0.3333333333333333) << 16)
        | (channel(hue) << 8)
        | channel(hue - 0.3333333333333333))
        & 0xffffff
}
pub fn draw(
    font: &Font,
    raster: &mut Raster,
    entries: &mut [Entry],
    cycle: i32,
    disabled: bool,
) -> Result<()> {
    draw_with_icons(font, raster, entries, cycle, disabled, &[])
}
pub fn draw_with_icons(
    font: &Font,
    raster: &mut Raster,
    entries: &mut [Entry],
    cycle: i32,
    disabled: bool,
    icons: &[crate::sprite::SpriteSheet],
) -> Result<()> {
    let widths: Vec<_> = entries
        .iter()
        .map(|e| font.metrics.width(&e.text, icons) / 2)
        .collect();
    let height = font.metrics.ascent;
    for i in 0..entries.len() {
        let x = entries[i].position[0];
        let mut y = entries[i].position[1];
        loop {
            let mut changed = false;
            for j in 0..i {
                let prior = &entries[j];
                if y + 2 > prior.position[1] - height
                    && y - height < prior.position[1] + 2
                    && x - widths[i] < prior.position[0] + widths[j]
                    && x + widths[i] > prior.position[0] - widths[j]
                    && prior.position[1] - height < y
                {
                    y = prior.position[1] - height;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        entries[i].position[1] = y;
        let e = &mut entries[i];
        let length = e.text.encode_utf16().count();
        let age = 150 - e.remaining;
        if e.color == 12 && e.pattern.is_none() {
            e.pattern = Some(
                (0..length)
                    .map(|i| rainbow((64.0f32 * (i as f32 / length as f32)) as usize))
                    .collect(),
            );
        }
        let style = Style {
            color: if disabled {
                0xffff00
            } else {
                color(e.color, cycle, e.remaining)
            },
            shadow: Some(0),
            alpha: 256,
        };
        if disabled {
            font.draw(raster, &e.text, x - widths[i], y, style, icons)?;
            continue;
        }
        let mut offsets = Vec::new();
        let mut left = x - widths[i];
        let mut baseline = y;
        let clip = raster.clip();
        match e.effect {
            0 => {}
            1 => {
                offsets = (0..length)
                    .map(|i| {
                        [
                            0,
                            ((i as f64 / 2.0 + cycle as f64 / 5.0).sin() * 5.0) as i32,
                        ]
                    })
                    .collect()
            }
            2 => {
                offsets = (0..length)
                    .map(|i| {
                        [
                            ((i as f64 / 5.0 + cycle as f64 / 5.0).sin() * 5.0) as i32,
                            ((i as f64 / 3.0 + cycle as f64 / 5.0).sin() * 5.0) as i32,
                        ]
                    })
                    .collect()
            }
            3 => {
                let amplitude = (7.0 - age as f64 / 8.0).max(0.0);
                offsets = (0..length)
                    .map(|i| {
                        [
                            0,
                            ((i as f64 / 1.5 + cycle as f64).sin() * amplitude) as i32,
                        ]
                    })
                    .collect();
            }
            4 => {
                left = x + 50 - age * (font.metrics.width(&e.text, icons) + 100) / 150;
                raster.set_clip(
                    (x - 50).max(clip[0] as i32),
                    clip[1] as i32,
                    (x + 50).min(clip[2] as i32),
                    clip[3] as i32,
                );
            }
            5 => {
                baseline += if age < 25 {
                    age - 25
                } else if age > 125 {
                    age - 125
                } else {
                    0
                };
                raster.set_clip(
                    clip[0] as i32,
                    (y - height - 1).max(clip[1] as i32),
                    clip[2] as i32,
                    (y + 5).min(clip[3] as i32),
                );
            }
            _ => continue,
        }
        let result = font.draw_offsets_with_icons(
            raster,
            &e.text,
            left,
            baseline,
            style,
            e.pattern.as_deref(),
            &offsets,
            icons,
        );
        raster.set_clip(
            clip[0] as i32,
            clip[1] as i32,
            clip[2] as i32,
            clip[3] as i32,
        );
        result?;
    }
    Ok(())
}

pub fn overlay_varps(cache: &mut crate::disk_cache::DiskCache) -> Result<[Option<u16>; 2]> {
    let mut result = [None; 2];
    for (id, data) in cache.group(2, 16, [0; 4])? {
        let mut r = crate::buffer::Reader::new(&data);
        loop {
            match r.u8()? {
                0 => break,
                5 => match r.u16()? {
                    6 => result[0] = Some(id as u16),
                    19 => result[1] = Some(id as u16),
                    _ => {}
                },
                _ => return Err(crate::Error::Invalid("varp definition opcode")),
            }
        }
        if r.remaining() != 0 {
            return Err(crate::Error::Invalid("varp definition trailing bytes"));
        }
    }
    Ok(result)
}
