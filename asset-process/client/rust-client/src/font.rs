//! Cache bitmap fonts and their pairwise kerning (GameFontTypeFace/zx).
use crate::{
    buffer::Reader, disk_cache::DiskCache, raster::Raster, sprite::SpriteSheet,
    writer::encode_cp1252, Error, Result,
};
#[derive(Clone, Debug)]
pub struct Metrics {
    pub advances: [u8; 256],
    pub ascent: i32,
    pub kerning: Option<Vec<i8>>,
}
impl Metrics {
    /// Break at spaces and hyphens, retaining markup and explicit <br> tags.
    pub fn wrap(
        &self,
        text: &str,
        widths: Option<&[i32]>,
        icons: &[SpriteSheet],
    ) -> Result<Vec<String>> {
        if widths.is_some_and(|w| w.is_empty()) {
            return Err(Error::Invalid("empty font line widths"));
        }
        let mut lines = Vec::new();
        let mut builder = String::new();
        let (mut start, mut width, mut previous) = (0, 0, Some(0));
        let mut breakpoint = None;
        let mut tag: Option<String> = None;
        for c in text.chars() {
            if c == '<' {
                tag = Some(String::new());
                continue;
            }
            let mut glyph = None;
            if c == '>' && tag.is_some() {
                let t = tag.take().unwrap();
                builder.push('<');
                builder.push_str(&t);
                builder.push('>');
                match t.as_str() {
                    "br" => {
                        lines.push(builder[start..].to_owned());
                        builder.clear();
                        start = 0;
                        width = 0;
                        breakpoint = None;
                        previous = Some(0);
                    }
                    "lt" | "gt" => {
                        let b = if t == "lt" { b'<' } else { b'>' };
                        width += self.advances[b as usize] as i32 + self.pair(previous, b);
                        previous = Some(b);
                    }
                    _ => {
                        if let Some(icon) = icon(&t, icons) {
                            width += icon.width as i32;
                            previous = Some(0);
                        }
                    }
                }
            } else if let Some(t) = &mut tag {
                t.push(c);
                continue;
            } else if c != '\0' {
                builder.push(c);
                for b in encode_cp1252(&c.to_string()) {
                    let normalized = if b == 160 { 32 } else { b };
                    width += self.advances[normalized as usize] as i32 + self.pair(previous, b);
                    previous = Some(b);
                }
                glyph = Some(c);
            }
            if glyph == Some(' ') {
                breakpoint = Some((builder.len(), width, 1));
            }
            if let Some(widths) = widths {
                if width > widths[lines.len().min(widths.len() - 1)] {
                    if let Some((end, at_width, trim)) = breakpoint {
                        lines.push(builder[start..end - trim].to_owned());
                        start = end;
                        breakpoint = None;
                        width -= at_width;
                        previous = Some(0);
                    }
                }
            }
            if glyph == Some('-') {
                breakpoint = Some((builder.len(), width, 0));
            }
        }
        if builder.len() > start {
            lines.push(builder[start..].to_owned());
        }
        Ok(lines)
    }
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let advances = r.bytes(256)?.try_into().unwrap();
        if data.len() == 257 {
            return Ok(Self {
                advances,
                ascent: r.u8()? as i32,
                kerning: None,
            });
        }
        let lengths = r.bytes(256)?.to_vec();
        let offsets = r.bytes(256)?.to_vec();
        let mut profiles = Vec::new();
        for _ in 0..2 {
            let mut plane = Vec::new();
            for &n in &lengths {
                let mut value = 0i8;
                let mut profile = Vec::new();
                for _ in 0..n {
                    value = value.wrapping_add(r.u8()? as i8);
                    profile.push(value);
                }
                plane.push(profile);
            }
            profiles.push(plane);
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("font metrics trailing bytes"));
        }
        let mut kerning = vec![0i8; 65536];
        for a in 0..256 {
            if a == 32 || a == 160 {
                continue;
            }
            for b in 0..256 {
                if b == 32 || b == 160 {
                    continue;
                }
                let start = offsets[a].max(offsets[b]) as usize;
                let end = (offsets[a] as usize + lengths[a] as usize)
                    .min(offsets[b] as usize + lengths[b] as usize);
                let mut overlap = advances[a].min(advances[b]) as i32;
                for row in start..end {
                    overlap = overlap.min(
                        profiles[1][a][row - offsets[a] as usize] as i32
                            + profiles[0][b][row - offsets[b] as usize] as i32,
                    );
                }
                kerning[a * 256 + b] = (-overlap) as i8;
            }
        }
        Ok(Self {
            advances,
            ascent: offsets[32] as i32 + lengths[32] as i32,
            kerning: Some(kerning),
        })
    }
    fn pair(&self, a: Option<u8>, b: u8) -> i32 {
        a.and_then(|a| {
            self.kerning
                .as_ref()
                .map(|k| k[a as usize * 256 + b as usize] as i32)
        })
        .unwrap_or(0)
    }
    pub fn width(&self, text: &str, icons: &[SpriteSheet]) -> i32 {
        let (mut width, mut previous) = (0, None);
        for token in tokens(text) {
            match token {
                Token::Glyph(c) => {
                    width += self.advances[c as usize] as i32 + self.pair(previous, c);
                    previous = Some(c);
                }
                Token::Tag(t) => {
                    if let Some(icon) = icon(&t, icons) {
                        width += icon.width as i32;
                        previous = None;
                    }
                }
            }
        }
        width
    }
}
enum Token {
    Glyph(u8),
    Tag(String),
}
fn tokens(text: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut tag = None;
    for c in text.chars() {
        if c == '<' {
            tag = Some(String::new());
        } else if c == '>' && tag.is_some() {
            let t = tag.take().unwrap();
            match t.as_str() {
                "lt" => out.push(Token::Glyph(b'<')),
                "gt" => out.push(Token::Glyph(b'>')),
                _ => out.push(Token::Tag(t)),
            }
        } else if let Some(t) = &mut tag {
            t.push(c);
        } else if c != '\0' {
            for b in encode_cp1252(&c.to_string()) {
                out.push(Token::Glyph(if b == 160 { 32 } else { b }));
            }
        }
    }
    out
}
fn icon<'a>(tag: &str, icons: &'a [SpriteSheet]) -> Option<&'a SpriteSheet> {
    tag.strip_prefix("img=")
        .and_then(|s| s.parse::<i32>().ok())
        .and_then(|i| usize::try_from(i).ok())
        .and_then(|i| icons.get(i))
}
pub struct Font {
    pub metrics: Metrics,
    pub glyphs: SpriteSheet,
    pub max_ascent: i32,
    pub max_descent: i32,
}
#[derive(Clone, Copy)]
pub struct Style {
    pub color: u32,
    pub shadow: Option<u32>,
    pub alpha: u16,
}
struct Paint {
    style: Style,
    strike: Option<u32>,
    underline: Option<u32>,
    spacing: i32,
    remainder: i32,
    offsets: Vec<[i32; 2]>,
    colors: Vec<u32>,
    glyph: usize,
    decorated: bool,
}
impl Paint {
    fn new(style: Style) -> Self {
        Self {
            style,
            strike: None,
            underline: None,
            spacing: 0,
            remainder: 0,
            offsets: Vec::new(),
            colors: Vec::new(),
            glyph: 0,
            decorated: false,
        }
    }
}
impl Default for Style {
    fn default() -> Self {
        Self {
            color: 0xffffff,
            shadow: Some(0),
            alpha: 256,
        }
    }
}
impl Font {
    pub fn load(cache: &mut DiskCache, id: u32) -> Result<Self> {
        Self::decode(
            &cache.file(13, id, 0, [0; 4])?,
            &cache.file(8, id, 0, [0; 4])?,
        )
    }
    pub fn decode(metrics: &[u8], sprites: &[u8]) -> Result<Self> {
        let metrics = Metrics::decode(metrics)?;
        let glyphs = SpriteSheet::decode(sprites, 16 << 20)?;
        if glyphs.sprites.len() != 256 {
            return Err(Error::Invalid("font glyph count"));
        }
        let top = glyphs
            .sprites
            .iter()
            .filter(|s| s.height != 0)
            .map(|s| s.y as i32)
            .min()
            .unwrap_or(i32::MAX);
        let bottom = glyphs
            .sprites
            .iter()
            .map(|s| (s.y + s.height) as i32)
            .max()
            .unwrap_or(0);
        Ok(Self {
            max_ascent: metrics.ascent.wrapping_sub(top),
            max_descent: bottom - metrics.ascent,
            metrics,
            glyphs,
        })
    }
    pub fn draw(
        &self,
        raster: &mut Raster,
        text: &str,
        x: i32,
        baseline: i32,
        style: Style,
        icons: &[SpriteSheet],
    ) -> Result<i32> {
        self.draw_styled(
            raster,
            text,
            x,
            baseline,
            style,
            icons,
            &mut Paint::new(style),
        )
    }
    /// The offset/color renderer used by overhead text (GameFontTypeFace.bn).
    pub fn draw_offsets(
        &self,
        raster: &mut Raster,
        text: &str,
        x: i32,
        baseline: i32,
        style: Style,
        colors: Option<&[u32]>,
        offsets: &[[i32; 2]],
    ) -> Result<i32> {
        self.draw_offsets_with_icons(raster, text, x, baseline, style, colors, offsets, &[])
    }
    pub fn draw_offsets_with_icons(
        &self,
        raster: &mut Raster,
        text: &str,
        x: i32,
        baseline: i32,
        style: Style,
        colors: Option<&[u32]>,
        offsets: &[[i32; 2]],
        icons: &[SpriteSheet],
    ) -> Result<i32> {
        let mut paint = Paint::new(style);
        paint.decorated = true;
        paint.offsets = offsets.to_vec();
        let length = text.encode_utf16().count();
        if let Some(colors) = colors.filter(|c| !c.is_empty()) {
            let ratio = colors.len() as f32 / length as f32;
            paint.colors = (0..length)
                .map(|i| colors[(i as f32 * ratio) as usize])
                .collect();
        }
        self.draw_styled(raster, text, x, baseline, style, icons, &mut paint)
    }
    fn draw_styled(
        &self,
        raster: &mut Raster,
        text: &str,
        mut x: i32,
        baseline: i32,
        style: Style,
        icons: &[SpriteSheet],
        paint: &mut Paint,
    ) -> Result<i32> {
        let start = x;
        let y = baseline.wrapping_sub(self.metrics.ascent);
        let (mut color, mut shadow, mut alpha) =
            (paint.style.color, paint.style.shadow, paint.style.alpha);
        let (mut strike, mut underline, mut previous) = (paint.strike, paint.underline, None);
        for token in tokens(text) {
            match token {
                Token::Tag(t) => {
                    // Original bn advances the effect slot even for an invalid image tag.
                    let image_offset = if paint.decorated && t.starts_with("img=") {
                        let offset = paint.offsets.get(paint.glyph).copied().unwrap_or([0; 2]);
                        paint.glyph += 1;
                        offset
                    } else {
                        [0; 2]
                    };
                    if let Some(icon) = icon(&t, icons) {
                        raster.draw_indexed(
                            icon,
                            0,
                            x + image_offset[0],
                            baseline - icon.height as i32 + image_offset[1],
                        )?;
                        x += icon.width as i32;
                        previous = None;
                        continue;
                    }
                    let hex = |s: &str| u32::from_str_radix(s, 16).ok();
                    if let Some(v) = t.strip_prefix("col=").and_then(hex) {
                        color = v;
                    } else if t == "/col" {
                        color = style.color;
                    } else if let Some(v) = t.strip_prefix("str=").and_then(hex) {
                        strike = Some(v);
                    } else if t == "str" {
                        strike = Some(0x800000);
                    } else if t == "/str" {
                        strike = None;
                    } else if let Some(v) = t.strip_prefix("u=").and_then(hex) {
                        underline = Some(v);
                    } else if t == "u" {
                        underline = Some(0);
                    } else if t == "/u" {
                        underline = None;
                        paint.spacing = 0;
                        paint.remainder = 0;
                    } else if let Some(v) = t.strip_prefix("shad=").and_then(hex) {
                        shadow = Some(v);
                    } else if t == "shad" {
                        shadow = Some(0);
                    } else if t == "/shad" {
                        shadow = style.shadow;
                    } else if t == "br" {
                        color = style.color;
                        shadow = style.shadow;
                        alpha = 256;
                        strike = None;
                        underline = None;
                    }
                }
                Token::Glyph(c) => {
                    x += self.metrics.pair(previous, c);
                    let g = &self.glyphs.sprites[c as usize];
                    let offset = paint.offsets.get(paint.glyph).copied().unwrap_or([0; 2]);
                    let glyph_color = paint.colors.get(paint.glyph).copied().unwrap_or(color);
                    paint.glyph += 1;
                    if c != 32 {
                        let gx = x + g.x as i32 + offset[0];
                        let gy = y + g.y as i32 + offset[1];
                        if let Some(shadow) = shadow {
                            raster.draw_mask(
                                &g.indices,
                                g.width,
                                g.height,
                                gx + 1,
                                gy + 1,
                                shadow,
                                alpha,
                            )?;
                        }
                        raster.draw_mask(
                            &g.indices,
                            g.width,
                            g.height,
                            gx,
                            gy,
                            glyph_color,
                            alpha,
                        )?;
                    } else if paint.spacing > 0 {
                        paint.remainder += paint.spacing;
                        x += paint.remainder >> 8;
                        paint.remainder &= 255;
                    }
                    let advance = self.metrics.advances[c as usize] as i32;
                    if let Some(c) = strike {
                        raster.fill_rect(
                            x,
                            y + (self.metrics.ascent as f64 * 0.7) as i32,
                            advance,
                            1,
                            c,
                            256,
                        );
                    }
                    if let Some(c) = underline {
                        raster.fill_rect(
                            x,
                            baseline + i32::from(!paint.decorated),
                            advance,
                            1,
                            c,
                            256,
                        );
                    }
                    x += advance;
                    previous = Some(c);
                }
            }
        }
        paint.style = Style {
            color,
            shadow,
            alpha,
        };
        paint.strike = strike;
        paint.underline = underline;
        Ok(x - start)
    }
    pub fn draw_paragraph(
        &self,
        raster: &mut Raster,
        text: &str,
        [x, y, width, height]: [i32; 4],
        mut style: Style,
        alignment: [u8; 2],
        mut line_height: i32,
        icons: &[SpriteSheet],
    ) -> Result<usize> {
        if style.alpha == 255 {
            style.alpha = 256;
        }
        if line_height == 0 {
            line_height = self.metrics.ascent;
        }
        let widths = [width];
        let limits = if height
            < self
                .max_ascent
                .wrapping_add(self.max_descent)
                .wrapping_add(line_height)
            && height < line_height.wrapping_mul(2)
        {
            None
        } else {
            Some(widths.as_slice())
        };
        let lines = self.metrics.wrap(text, limits, icons)?;
        let count = lines.len() as i32;
        let vertical = if alignment[1] == 3 && count == 1 {
            1
        } else {
            alignment[1]
        };
        let extra = height
            .wrapping_sub(self.max_ascent)
            .wrapping_sub(self.max_descent)
            .wrapping_sub((count - 1).wrapping_mul(line_height));
        let mut baseline = match vertical {
            0 => y.wrapping_add(self.max_ascent),
            1 => y.wrapping_add(self.max_ascent).wrapping_add(extra / 2),
            2 => y
                .wrapping_add(height)
                .wrapping_sub(self.max_descent)
                .wrapping_sub((count - 1).wrapping_mul(line_height)),
            _ => {
                let space = (extra / (count + 1)).max(0);
                line_height = line_height.wrapping_add(space);
                y.wrapping_add(self.max_ascent).wrapping_add(space)
            }
        };
        let mut paint = Paint::new(style);
        for (i, line) in lines.iter().enumerate() {
            let text_width = self.metrics.width(line, icons);
            let left = match alignment[0] {
                1 => x.wrapping_add(width.wrapping_sub(text_width) / 2),
                2 => x.wrapping_add(width).wrapping_sub(text_width),
                _ => x,
            };
            if alignment[0] > 2 && i + 1 < lines.len() {
                let spaces = tokens(line)
                    .iter()
                    .filter(|t| matches!(t, Token::Glyph(b' ')))
                    .count() as i32;
                if spaces > 0 {
                    paint.spacing = width.wrapping_sub(text_width).wrapping_shl(8) / spaces;
                }
            }
            self.draw_styled(raster, line, left, baseline, style, icons, &mut paint)?;
            paint.spacing = 0;
            baseline = baseline.wrapping_add(line_height);
        }
        Ok(lines.len())
    }
}
