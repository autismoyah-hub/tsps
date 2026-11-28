//! Editable text mutations and selection clock (original lh).
use crate::{Error, Result};
#[derive(Clone, Debug)]
pub struct Text {
    pub value: Vec<u16>,
    pub anchor: i32,
    pub cursor: i32,
    pub filter: i32,
    pub max_chars: i32,
    pub clock: u8,
    pub focused: bool,
    pub enabled: bool,
    pub ready: bool,
    pub(crate) events: Vec<u16>,
}
impl Default for Text {
    fn default() -> Self {
        Self {
            value: Vec::new(),
            anchor: 0,
            cursor: 0,
            filter: 0,
            max_chars: i32::MAX,
            clock: 0,
            focused: false,
            enabled: true,
            ready: false,
            events: Vec::new(),
        }
    }
}
impl Text {
    pub fn selected_text(&self) -> Result<&[u16]> {
        if self.anchor == self.cursor
            || self.anchor > self.value.len() as i32
            || self.cursor > self.value.len() as i32
        {
            return Ok(&[]);
        }
        Ok(&self.value[self.selection()?])
    }
    pub fn paste(&mut self, input: &[u16]) -> Result<()> {
        self.paste_with(input, |_, _, end| Ok(end))
    }
    pub(crate) fn paste_with(
        &mut self,
        input: &[u16],
        prepare: impl FnOnce(&mut Vec<u16>, usize, usize) -> Result<usize>,
    ) -> Result<()> {
        let filtered = input
            .iter()
            .copied()
            .filter(|&ch| self.accepts(ch))
            .collect::<Vec<_>>();
        self.delete_selection()?;
        let index = usize::try_from(self.cursor)
            .map_err(|_| Error::Invalid("negative editable paste cursor"))?;
        if index > self.value.len() {
            return Err(Error::Invalid("editable paste outside text"));
        }
        let end = index + filtered.len();
        self.value.splice(index..index, filtered);
        let cursor = prepare(&mut self.value, index, end)? as i32;
        self.select(cursor, cursor);
        self.enforce_limit()?;
        self.events.push(1437);
        Ok(())
    }

    fn move_cursor(&mut self, cursor: i32, extend: bool) {
        self.select(if extend { self.anchor } else { cursor }, cursor);
    }
    pub(crate) fn boundary(&self, index: i32, line: bool) -> Result<std::ops::Range<usize>> {
        let index =
            usize::try_from(index).map_err(|_| Error::Invalid("negative editable boundary"))?;
        if index > self.value.len() {
            return Err(Error::Invalid("editable boundary outside text"));
        }
        let separator = |ch: u16| ch == 10 || (!line && (ch == 32 || ch == 9));
        let start = self.value[..index]
            .iter()
            .rposition(|&ch| separator(ch))
            .map_or(0, |i| i + 1);
        let end = self.value[index..]
            .iter()
            .position(|&ch| separator(ch))
            .map_or(self.value.len(), |i| index + i);
        Ok(start..end)
    }
    pub fn move_horizontal(&mut self, right: bool, word: bool, extend: bool) -> Result<()> {
        if !word && !extend && self.anchor != self.cursor {
            self.move_cursor(
                if right {
                    self.anchor.max(self.cursor)
                } else {
                    self.anchor.min(self.cursor)
                },
                false,
            );
        } else if right && self.cursor < (self.value.len() as i32) {
            let next = if word {
                self.boundary(self.cursor + 1, false)?.end as i32
            } else {
                self.cursor + 1
            };
            self.move_cursor(next, extend);
        } else if !right && self.cursor > 0 {
            let next = if word {
                self.boundary(self.cursor - 1, false)?.start as i32
            } else {
                self.cursor - 1
            };
            self.move_cursor(next, extend);
        }
        Ok(())
    }
    pub fn move_boundary(&mut self, end: bool, document: bool, extend: bool) -> Result<()> {
        let range = if document {
            0..self.value.len()
        } else {
            self.boundary(self.cursor, true)?
        };
        self.move_cursor(if end { range.end } else { range.start } as i32, extend);
        Ok(())
    }
    pub fn delete_word(&mut self, forward: bool) -> Result<()> {
        if self.delete_selection()? {
            return Ok(());
        }
        if forward && self.cursor < (self.value.len() as i32) {
            let end = self.boundary(self.cursor, false)?.end;
            self.value.drain(self.cursor as usize..end);
            self.events.push(1437);
            self.select(self.cursor, self.cursor);
        } else if !forward && self.cursor > 0 {
            let start = self.boundary(self.cursor - 1, false)?.start;
            self.value.drain(start..self.cursor as usize);
            self.events.push(1437);
            self.select(start as i32, start as i32);
        }
        Ok(())
    }

    pub fn take_events(&mut self) -> Vec<u16> {
        std::mem::take(&mut self.events)
    }
    pub fn accepts(&self, ch: u16) -> bool {
        let alpha = (65..=90).contains(&ch) || (97..=122).contains(&ch);
        let digit = (48..=57).contains(&ch);
        match self.filter {
            1 => alpha || digit,
            2 => alpha,
            3 => digit,
            4 => digit || [75, 107, 77, 109, 66, 98].contains(&ch),
            _ => true,
        }
    }
    pub fn initialize(&mut self) -> Result<()> {
        self.ready = true;
        self.select(self.anchor, self.cursor);
        if self.enforce_limit()? {
            self.events.push(1437);
        }
        Ok(())
    }
    pub fn select(&mut self, anchor: i32, cursor: i32) {
        if !self.ready {
            self.anchor = anchor;
            self.cursor = cursor;
            return;
        }
        let end = self.value.len() as i32;
        let anchor = anchor.min(end);
        let cursor = cursor.min(end);
        let changed = self.anchor != anchor || self.cursor != cursor;
        self.anchor = anchor;
        if cursor != self.cursor {
            self.cursor = cursor;
            self.clock = 0;
        }
        if changed {
            self.events.push(1439);
        }
    }
    pub fn set_focus(&mut self, value: bool) {
        self.focused = value && self.enabled;
        if !self.focused {
            self.select(self.cursor, self.cursor);
        }
    }
    pub fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
        self.focused &= value;
    }
    pub fn tick(&mut self) {
        self.clock = (self.clock + 1) % 60;
    }
    pub fn caret_visible(&self) -> bool {
        self.focused && self.clock < 30
    }
    pub fn set_text(&mut self, text: &[u16]) -> Result<()> {
        self.set_text_with(text, Ok)
    }
    pub(crate) fn set_text_with(
        &mut self,
        text: &[u16],
        prepare: impl FnOnce(Vec<u16>) -> Result<Vec<u16>>,
    ) -> Result<()> {
        if self.value != text {
            self.value = prepare(
                text.iter()
                    .copied()
                    .filter(|&ch| self.accepts(ch))
                    .collect(),
            )?;
            self.select(self.anchor, self.cursor);
            self.enforce_limit()?;
            self.events.push(1437);
        }
        Ok(())
    }
    pub fn set_filter(&mut self, filter: i32) -> Result<()> {
        self.set_filter_with(filter, Ok)
    }
    pub(crate) fn set_filter_with(
        &mut self,
        filter: i32,
        prepare: impl FnOnce(Vec<u16>) -> Result<Vec<u16>>,
    ) -> Result<()> {
        self.filter = filter;
        let filtered = self
            .value
            .iter()
            .copied()
            .filter(|&ch| self.accepts(ch))
            .collect::<Vec<_>>();
        if filtered.len() != self.value.len() {
            self.value = prepare(filtered)?;
            self.enforce_limit()?;
            self.events.push(1437);
        }
        Ok(())
    }
    pub fn set_limit(&mut self, limit: i32) -> Result<()> {
        self.max_chars = limit;
        if self.enforce_limit()? {
            self.events.push(1437);
        }
        Ok(())
    }
    fn enforce_limit(&mut self) -> Result<bool> {
        if self.ready {
            let limit = usize::try_from(self.max_chars)
                .map_err(|_| Error::Invalid("negative editable text limit"))?;
            if self.value.len() > limit {
                self.value.truncate(limit);
                self.select(self.anchor, self.cursor);
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn selection(&self) -> Result<std::ops::Range<usize>> {
        let a = usize::try_from(self.anchor)
            .map_err(|_| Error::Invalid("negative editable selection"))?;
        let c =
            usize::try_from(self.cursor).map_err(|_| Error::Invalid("negative editable cursor"))?;
        if a.max(c) > self.value.len() {
            return Err(Error::Invalid("editable selection outside text"));
        }
        Ok(a.min(c)..a.max(c))
    }
    pub fn delete_selection(&mut self) -> Result<bool> {
        let range = self.selection()?;
        if range.is_empty() {
            return Ok(false);
        }
        let index = range.start;
        self.value.drain(range);
        self.events.push(1437);
        self.select(index as i32, index as i32);
        Ok(true)
    }
    pub fn insert(&mut self, ch: u16) -> Result<()> {
        self.insert_with(ch, |_, index| Ok(index + 1))
    }
    pub(crate) fn insert_with(
        &mut self,
        ch: u16,
        prepare: impl FnOnce(&mut Vec<u16>, usize) -> Result<usize>,
    ) -> Result<()> {
        if self.accepts(ch) {
            self.delete_selection()?;
            let index = usize::try_from(self.cursor)
                .map_err(|_| Error::Invalid("editable insert cursor"))?;
            if index > self.value.len() {
                return Err(Error::Invalid("editable insert outside text"));
            }
            self.value.insert(index, ch);
            let cursor = prepare(&mut self.value, index)? as i32;
            self.select(cursor, cursor);
            self.enforce_limit()?;
            self.events.push(1437);
        }
        Ok(())
    }
    pub fn backspace(&mut self) -> Result<()> {
        if !self.delete_selection()? && self.cursor > 0 {
            let index = self.cursor as usize - 1;
            self.value.remove(index);
            self.events.push(1437);
            self.select(index as i32, index as i32);
        }
        Ok(())
    }
    pub fn delete(&mut self) -> Result<()> {
        if !self.delete_selection()? && (self.cursor as usize) < self.value.len() {
            self.value.remove(self.cursor as usize);
            self.events.push(1437);
            self.select(self.cursor, self.cursor);
        }
        Ok(())
    }
}
