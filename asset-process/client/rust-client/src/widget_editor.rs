//! Prepared editable-widget text, fonts, geometry and paint settings.
use crate::{
    editable_text::Text,
    editable_text_layout::{positions, Options},
    font::Metrics,
    Result,
};
#[derive(Clone, Debug)]
pub struct Editor {
    pub text: Text,
    pub placeholder: Vec<u16>,
    pub pending_placeholder: String,
    pub options: Options,
    pub colors: [u32; 3],
    pub scroll: [i32; 2],
    pub size: [i32; 2],
    pub metrics: Option<Metrics>,
    pub font_id: i32,
    pub enter_mode: i32,
    pub click_anchor: i32,
    pub selection_mode: u8,
    pub click_ticks: u8,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            text: Text::default(),
            placeholder: Vec::new(),
            pending_placeholder: String::new(),
            options: Options {
                width: i32::MAX,
                max_lines: i32::MAX,
                wrap: 1,
                mask: 0,
                align: [0; 2],
                line_height: 0,
            },
            colors: [0; 3],
            scroll: [0; 2],
            size: [0; 2],
            metrics: None,
            font_id: -1,
            enter_mode: 0,
            click_anchor: 0,
            selection_mode: 0,
            click_ticks: 0,
        }
    }
}
/// GameWidget's queued text expands lt/gt/br and removes other complete tags.
pub fn pending_text(text: &str) -> Vec<u16> {
    let text = text.encode_utf16().collect::<Vec<_>>();
    let (mut start, mut tag) = (0, None);
    let mut out = Vec::new();
    for (index, &ch) in text.iter().enumerate() {
        if ch == 60 {
            out.extend_from_slice(&text[start..index]);
            tag = Some(index);
        } else if ch == 62 {
            if let Some(open) = tag.take() {
                match &text[open + 1..index] {
                    [108, 116] => out.push(60),
                    [103, 116] => out.push(62),
                    [98, 114] => out.push(10),
                    _ => {}
                }
                start = index + 1;
            }
        }
    }
    out.extend_from_slice(&text[start..]);
    out
}
impl Editor {
    pub fn tick(&mut self) {
        self.text.tick();
        self.click_ticks = self.click_ticks.saturating_sub(1);
    }
    fn mouse_index(&self, point: [i32; 2]) -> Result<i32> {
        if !self.text.ready {
            return Ok(0);
        }
        let aligned = |size: i32, mode: i32| match mode {
            1 => size / 2,
            2 => size,
            _ => 0,
        };
        let point = [
            point[0] + self.scroll[0] - aligned(self.size[0], self.options.align[0]),
            point[1] + self.scroll[1] - aligned(self.size[1], self.options.align[1]),
        ];
        Ok(crate::editable_text_layout::hit_test(
            &self.text.value,
            self.metrics.as_ref().unwrap(),
            self.options,
            point,
        )? as i32)
    }
    fn select_ranges(
        &mut self,
        first: std::ops::Range<usize>,
        second: std::ops::Range<usize>,
    ) -> Result<()> {
        if second.start < first.start {
            self.select(first.end as i32, second.start as i32)
        } else {
            self.select(first.start as i32, second.end as i32)
        }
    }
    pub fn mouse_press(&mut self, point: [i32; 2], control: bool, shift: bool) -> Result<()> {
        let index = self.mouse_index(point)?;
        if control && shift {
            self.selection_mode = 1;
            self.select_ranges(
                self.text.boundary(self.click_anchor, false)?,
                self.text.boundary(index, false)?,
            )?;
        } else if control {
            self.selection_mode = 1;
            let range = self.text.boundary(index, false)?;
            self.select(range.start as i32, range.end as i32)?;
            self.click_anchor = range.start as i32;
        } else if shift {
            self.select(self.click_anchor, index)?;
        } else {
            if self.click_ticks > 0 && self.click_anchor == index {
                let line = self.text.anchor != self.text.cursor;
                self.selection_mode = if line { 2 } else { 1 };
                let range = self.text.boundary(index, line)?;
                self.select(range.start as i32, range.end as i32)?;
            } else {
                self.selection_mode = 0;
                self.select(index, index)?;
                self.click_anchor = index;
            }
            self.click_ticks = 25;
        }
        Ok(())
    }
    pub fn mouse_drag(&mut self, point: [i32; 2]) -> Result<()> {
        if !self.text.ready || !self.text.focused {
            return Ok(());
        }
        let index = self.mouse_index(point)?;
        if self.selection_mode == 0 {
            self.select(self.text.anchor, index)?;
        } else if self.selection_mode <= 2 {
            let line = self.selection_mode == 2;
            self.select_ranges(
                self.text.boundary(self.click_anchor, line)?,
                self.text.boundary(index, line)?,
            )?;
        }
        Ok(())
    }

    fn limited_prefix(
        value: &[u16],
        font: &Metrics,
        options: Options,
        minimum: usize,
    ) -> Result<usize> {
        if options.max_lines == 0 {
            return Ok(value.len());
        }
        if font.ascent == 0 {
            return Err(crate::Error::Invalid("zero editable font ascent"));
        }
        let fits = |text: &[u16]| -> Result<bool> {
            let points = positions(text, font, options)?;
            Ok(points.last().map_or(0, |p| font.ascent + p[1]) / font.ascent <= options.max_lines)
        };
        if fits(value)? {
            return Ok(value.len());
        }
        // zi.ai preserves the earlier line positions when deleting wrapped text.
        // Reflowing the complete prefix can fit more text than Java retains.
        if options.max_lines > 1 && options.wrap == 1 && options.align[1] == 0 {
            let mut points = positions(value, font, options)?;
            let mut end = value.len();
            while end > minimum {
                end -= 1;
                points.pop();
                let mut changed = end;
                while changed > 0 {
                    changed -= 1;
                    if matches!(value[changed], 32 | 9) {
                        break;
                    }
                }
                let mut start = 0;
                if changed < end {
                    let mut y = points[changed][1];
                    let mut previous = options.align[0] != 0;
                    for index in (1..=changed).rev() {
                        if points[index - 1][1] < y {
                            if !previous {
                                start = index;
                                break;
                            }
                            previous = false;
                            y = points[index - 1][1];
                        }
                    }
                }
                let y = if start == 0 { 0 } else { points[start][1] };
                let tail = positions(&value[start..end], font, options)?;
                for (point, next) in points[start..].iter_mut().zip(tail) {
                    *point = [next[0], next[1].wrapping_add(y)];
                }
                if points.last().map_or(0, |p| font.ascent.wrapping_add(p[1])) / font.ascent
                    <= options.max_lines
                {
                    break;
                }
            }
            return Ok(end);
        }
        // Prefix line counts are monotone unless Java's signed layout arithmetic wraps.
        let extent = (value.len() as i128 + 1) * (options.line_height as i128).abs()
            + (font.ascent as i128).abs();
        let bounded = extent <= i32::MAX as i128 && value.len() as i128 * 765 <= i32::MAX as i128;
        if bounded && options.max_lines > 0 && (0..=2).contains(&options.align[1]) {
            if options.align[1] == 2 || options.line_height <= 0 {
                return Ok(minimum);
            }
            let (mut low, mut high) = (minimum, value.len());
            while low < high {
                let middle = low + (high - low + 1) / 2;
                if fits(&value[..middle])? {
                    low = middle;
                } else {
                    high = middle - 1;
                }
            }
            return Ok(low);
        }
        let mut end = value.len();
        while end > minimum {
            end -= 1;
            if fits(&value[..end])? {
                break;
            }
        }
        Ok(end)
    }

    fn limit_insert(
        value: &mut Vec<u16>,
        start: usize,
        mut end: usize,
        font: Option<&Metrics>,
        options: Options,
    ) -> Result<usize> {
        if let Some(font) = font {
            if font.ascent == 0 {
                return Err(crate::Error::Invalid("zero editable font ascent"));
            }
            if end == value.len() {
                end = Self::limited_prefix(value, font, options, start)?;
                value.truncate(end);
                return Ok(end);
            }
            while options.max_lines != 0 && end > start {
                let points = positions(value, font, options)?;
                if points.last().map_or(0, |p| font.ascent + p[1]) / font.ascent
                    <= options.max_lines
                {
                    break;
                }
                end -= 1;
                value.remove(end);
            }
        }
        Ok(end)
    }
    pub fn paste(&mut self, input: &[u16]) -> Result<()> {
        let previous = self.text.cursor;
        let start = self.text.events.len();
        let font = self.metrics.as_ref();
        let options = self.options;
        self.text.paste_with(input, |value, start, end| {
            Self::limit_insert(value, start, end, font, options)
        })?;
        self.finish_limits(start)?;
        if previous != self.text.cursor {
            self.follow_cursor()?;
        }
        Ok(())
    }

    pub fn cursor_point(&self) -> Result<[i32; 2]> {
        if self.text.cursor <= 0 {
            return Ok([0, 0]);
        }
        let Some(font) = self.metrics.as_ref() else {
            return Ok([0, 0]);
        };
        let points = positions(&self.text.value, font, self.options)?;
        let index = self.text.cursor as usize - 1;
        let Some(&point) = points.get(index) else {
            return Ok([0, 0]);
        };
        Ok([
            point[0]
                + self.advance(
                    self.text.value[index],
                    self.options.mask == 2 || index + 1 != points.len(),
                ),
            point[1],
        ])
    }
    pub fn navigation_key(&mut self, key: i32, extend: bool, control: bool) -> Result<()> {
        if !self.text.ready {
            return Ok(());
        }
        let previous = self.text.cursor;
        match key {
            48 if control => self.text.select(0, self.text.value.len() as i32),
            85 => {
                if control {
                    self.text.delete_word(false)?;
                } else {
                    self.text.backspace()?;
                }
            }
            101 => {
                if control {
                    self.text.delete_word(true)?;
                } else {
                    self.text.delete()?;
                }
            }
            96 | 97 => self.text.move_horizontal(key == 97, control, extend)?,
            102 | 103 => self.text.move_boundary(key == 103, control, extend)?,
            98 | 99 if control => {
                if key == 98 {
                    self.scroll[1] = (self.scroll[1] - self.options.line_height).max(0);
                } else {
                    let font = self.metrics.as_ref().unwrap();
                    let points = positions(&self.text.value, font, self.options)?;
                    let height = points.last().map_or(0, |p| font.ascent + p[1]);
                    self.scroll[1] = (self.scroll[1] + self.options.line_height)
                        .min((height - self.size[1]).max(0));
                }
            }
            104 | 105 if control => {
                let point = self.cursor_point()?;
                let y = self.scroll[1] + if key == 105 { self.size[1] } else { 0 };
                let cursor = crate::editable_text_layout::hit_test(
                    &self.text.value,
                    self.metrics.as_ref().unwrap(),
                    self.options,
                    [point[0], y],
                )? as i32;
                self.text
                    .select(if extend { self.text.anchor } else { cursor }, cursor);
            }
            98 | 99 | 104 | 105 => {
                let down = key == 99 || key == 105;
                if (down && self.text.cursor < self.text.value.len() as i32)
                    || (!down && self.text.cursor > 0)
                {
                    let count = if key >= 104 {
                        self.size[1]
                            .checked_div(self.options.line_height)
                            .ok_or(crate::Error::Invalid("zero editable page line height"))?
                    } else {
                        1
                    };
                    let cursor = crate::editable_text_layout::vertical(
                        &self.text.value,
                        self.metrics.as_ref().unwrap(),
                        self.options,
                        self.text.cursor,
                        if down { count } else { -count },
                    )?;
                    self.text
                        .select(if extend { self.text.anchor } else { cursor }, cursor);
                }
            }
            _ => {}
        }
        if self.text.cursor != previous {
            self.follow_cursor()?;
        }
        Ok(())
    }

    pub fn insert(&mut self, ch: u16) -> Result<()> {
        let previous = self.text.cursor;
        let start = self.text.events.len();
        let font = self.metrics.as_ref();
        let options = self.options;
        self.text.insert_with(ch, |value, index| {
            Self::limit_insert(value, index, index + 1, font, options)
        })?;
        if self.text.events.len() == start {
            return Ok(());
        }
        self.finish_limits(start)?;
        if previous != self.text.cursor {
            self.follow_cursor()?;
        }
        Ok(())
    }

    // zi.ab applies insertion limits even when lh is awaiting font reinitialization.
    fn limit_buffer(
        mut value: Vec<u16>,
        font: Option<&Metrics>,
        options: Options,
    ) -> Result<Vec<u16>> {
        let Some(font) = font else {
            return Ok(value);
        };
        let end = Self::limited_prefix(&value, font, options, 0)?;
        value.truncate(end);
        Ok(value)
    }

    fn enforce_lines(&mut self) -> Result<bool> {
        if !self.text.ready {
            return Ok(false);
        }
        let font = self.metrics.as_ref().unwrap();
        let points = positions(&self.text.value, font, self.options)?;
        let height = points.last().map_or(0, |p| font.ascent + p[1]);
        if font.ascent == 0 {
            return Err(crate::Error::Invalid("zero editable font ascent"));
        }
        if height / font.ascent <= self.options.max_lines {
            return Ok(false);
        }
        let end = crate::editable_text_layout::vertical(
            &self.text.value,
            font,
            self.options,
            0,
            self.options.max_lines,
        )? - 1;
        let end = usize::try_from(end)
            .map_err(|_| crate::Error::Invalid("negative editable line limit boundary"))?;
        self.text.value.truncate(end);
        self.text.select(self.text.anchor, self.text.cursor);
        Ok(true)
    }
    // Limit-induced selection changes precede the mutation's content callback.
    fn finish_limits(&mut self, start: usize) -> Result<()> {
        let content = self.text.events.len() > start && self.text.events.last() == Some(&1437);
        if content {
            self.text.events.pop();
        }
        let truncated = self.enforce_lines()?;
        if content || truncated {
            self.text.events.push(1437);
        }
        Ok(())
    }
    pub fn set_width(&mut self, width: i32) -> Result<()> {
        self.options.width =
            if width < 0 || self.metrics.as_ref().is_some_and(|f| width < f.ascent * 2) {
                i32::MAX
            } else {
                width
            };
        let previous = self.text.cursor;
        let start = self.text.events.len();
        self.finish_limits(start)?;
        if previous != self.text.cursor {
            self.follow_cursor()?;
        }
        Ok(())
    }
    pub fn set_max_lines(&mut self, lines: i32) -> Result<()> {
        self.options.max_lines = lines;
        let previous = self.text.cursor;
        let start = self.text.events.len();
        self.finish_limits(start)?;
        if previous != self.text.cursor {
            self.follow_cursor()?;
        }
        Ok(())
    }
    pub fn set_limit(&mut self, limit: i32) -> Result<()> {
        let previous = self.text.cursor;
        let start = self.text.events.len();
        self.text.set_limit(limit)?;
        self.finish_limits(start)?;
        if previous != self.text.cursor {
            self.follow_cursor()?;
        }
        Ok(())
    }
    pub fn set_filter(&mut self, filter: i32) -> Result<()> {
        let previous = self.text.cursor;
        let start = self.text.events.len();
        let font = self.metrics.as_ref();
        let options = self.options;
        self.text
            .set_filter_with(filter, |v| Self::limit_buffer(v, font, options))?;
        if self.text.events.len() == start {
            return Ok(());
        }
        self.finish_limits(start)?;
        if previous != self.text.cursor {
            self.follow_cursor()?;
        }
        self.set_scroll(self.scroll)
    }

    pub fn prepare(
        &mut self,
        pending: &mut String,
        size: [i32; 2],
        font: Option<(i32, Metrics)>,
    ) -> Result<()> {
        self.size = size;
        self.set_scroll(self.scroll)?;
        let selection = [self.text.anchor, self.text.cursor];
        let scroll = self.scroll;
        let initialize = !self.text.ready;
        if initialize {
            if let Some((id, metrics)) = font {
                self.initialize(id, metrics)?;
            }
        }
        if self.text.ready {
            if !pending.is_empty() {
                self.set_text(&pending_text(pending))?;
                pending.clear();
            }
            if !self.pending_placeholder.is_empty() {
                self.placeholder = pending_text(&self.pending_placeholder);
                self.pending_placeholder.clear();
            }
            if initialize {
                self.set_scroll(scroll)?;
                self.select(selection[0], selection[1])?;
            } else {
                self.set_scroll(self.scroll)?;
            }
        }
        self.tick();
        Ok(())
    }

    pub fn set_alignment(&mut self, x: i32, y: i32, line: i32) {
        if (0..=2).contains(&x) && (0..=2).contains(&y) {
            self.options.align = [x, y];
        }
        self.options.line_height = line;
    }
    pub fn placeholder_options(&self) -> Options {
        Options {
            width: self.options.width,
            max_lines: i32::MAX,
            wrap: 1,
            mask: 0,
            align: [0; 2],
            line_height: self.metrics.as_ref().map_or(0, |m| m.ascent),
        }
    }
    pub fn initialize(&mut self, font_id: i32, metrics: Metrics) -> Result<()> {
        let selection = [self.text.anchor, self.text.cursor];
        let changed_font = self.font_id != font_id || self.metrics.is_none();
        self.font_id = font_id;
        if changed_font && self.options.line_height == 0 {
            self.options.line_height = metrics.ascent;
        }
        self.metrics = Some(metrics);
        let start = self.text.events.len();
        self.text.initialize()?;
        self.finish_limits(start)?;
        self.text.select(selection[0], selection[1]);
        Ok(())
    }
    pub fn set_scroll(&mut self, value: [i32; 2]) -> Result<()> {
        self.scroll = value;
        if self.text.ready {
            let metrics = self.metrics.as_ref().unwrap();
            let points = positions(&self.text.value, metrics, self.options)?;
            let mut width = 0;
            for (i, point) in points.iter().enumerate() {
                if i + 1 == points.len() || points[i + 1][1] != point[1] {
                    width = width.max(point[0] + self.advance(self.text.value[i], true));
                }
            }
            let height = points.last().map_or(0, |p| metrics.ascent + p[1]);
            self.scroll[0] = value[0].clamp(0, (width - self.size[0] + 2).max(0));
            self.scroll[1] = value[1].clamp(0, (height - self.size[1] + 1).max(0));
        }
        Ok(())
    }
    fn advance(&self, ch: u16, mask: bool) -> i32 {
        if ch == 10 {
            return 0;
        }
        let metrics = self.metrics.as_ref().unwrap();
        if mask && self.options.mask != 0 {
            return metrics.advances[42] as i32;
        }
        let a = metrics.advances.get(ch as usize).copied().unwrap_or(0) as i32;
        if a == 0 {
            metrics.advances[32] as i32 * if ch == 9 { 3 } else { 1 }
        } else {
            a
        }
    }
    pub fn follow_cursor(&mut self) -> Result<()> {
        if !self.text.ready {
            return Ok(());
        }
        let points = positions(
            &self.text.value,
            self.metrics.as_ref().unwrap(),
            self.options,
        )?;
        let point = if self.text.cursor <= 0 {
            [0, 0]
        } else if let Some(&p) = points.get(self.text.cursor as usize - 1) {
            let index = self.text.cursor as usize - 1;
            [
                p[0] + self.advance(
                    self.text.value[index],
                    self.options.mask == 2 || index + 1 != points.len(),
                ),
                p[1],
            ]
        } else {
            return Ok(());
        };
        let bounds = [
            point[0] - 10,
            point[1] - 3,
            point[0] + 10,
            point[1] + self.options.line_height + 3,
        ];
        let mut scroll = self.scroll;
        for axis in 0..2 {
            if bounds[axis] < scroll[axis] {
                scroll[axis] = bounds[axis];
            } else if bounds[axis + 2] > scroll[axis] + self.size[axis] {
                scroll[axis] = bounds[axis + 2] - self.size[axis];
            }
        }
        self.set_scroll(scroll)
    }
    pub fn select(&mut self, anchor: i32, cursor: i32) -> Result<()> {
        let previous = self.text.cursor;
        self.text.select(anchor, cursor);
        if previous != self.text.cursor {
            self.follow_cursor()?;
        }
        Ok(())
    }
    pub fn set_text(&mut self, value: &[u16]) -> Result<()> {
        let previous = self.text.cursor;
        let start = self.text.events.len();
        let font = self.metrics.as_ref();
        let options = self.options;
        self.text
            .set_text_with(value, |v| Self::limit_buffer(v, font, options))?;
        if self.text.events.len() == start {
            return Ok(());
        }
        self.finish_limits(start)?;
        if self.text.cursor != previous {
            self.follow_cursor()?;
        }
        self.set_scroll(self.scroll)
    }
}
