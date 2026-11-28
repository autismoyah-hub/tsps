//! Per-cycle keyboard and pointer transitions consumed by scripts and UI events.
use crate::{vm::Stacks, Result};
/// Map supported physical buttons (left/right/middle = 0/1/2) to the original
/// logical button. Alt takes precedence over Meta/right, as in sw.aa.
pub fn mouse_button(physical: u8, alt: bool, meta: bool) -> u8 {
    if physical >= 5 {
        physical
    } else if alt || physical == 2 {
        2
    } else if meta || physical == 1 {
        1
    } else {
        0
    }
}

/// Original tg.keyTyped/dv.af admission, before any editor or script consumer.
fn typed_character(value: u16) -> bool {
    (1..128).contains(&value)
        || (160..=255).contains(&value)
        || (128..160).any(|byte| crate::buffer::cp1252(byte) as u16 == value)
}

/// Preserve fractional Win32 wheel detents separately for each axis, like AWT.
#[derive(Default)]
pub struct WheelAccumulator([i32; 2]);
impl WheelAccumulator {
    pub fn rotate(&mut self, delta: i16, horizontal: bool) -> i32 {
        let remainder = &mut self.0[usize::from(horizontal)];
        *remainder += i32::from(delta);
        let turns = *remainder / 120;
        *remainder %= 120;
        if horizontal {
            turns
        } else {
            -turns
        }
    }
}

/// Ordered platform events can feed separate render and simulation consumers.
#[derive(Clone, Copy, Debug)]
pub enum Update {
    Pointer([i32; 2]),
    Button(u8, bool),
    /// Platform button coordinates are independent of the last cursor movement.
    ButtonAt(u8, bool, [i32; 2]),
    Key(i32, bool),
    Character(u16),
    Wheel(i32),
    LoseFocus,
}
/// GameClient drains the original ready key range once per widget tick.
/// Pending events and the new batch share the original 127 usable ring slots.
#[derive(Default)]
pub struct KeyBuffer {
    queued: Vec<(i32, i32)>,
    pending: Vec<(i32, i32)>,
}

pub struct Input {
    pub held: std::collections::HashSet<i32>,
    pub pressed: std::collections::HashSet<i32>,
    pub released: std::collections::HashSet<i32>,
    pub pointer: [i32; 2],
    pub press_position: [i32; 2],
    pub buttons: u8,
    pub pressed_buttons: u8,
    pub released_buttons: u8,
    pub wheel: i32,
    pub text: Vec<u16>,
    pub key_events: Vec<(i32, i32)>,
    pub pressed_order: Vec<i32>,
    pub released_order: Vec<i32>,
    /// Enabled for native widget ticks; direct snapshots retain their input API.
    pub key_buffer: Option<KeyBuffer>,
}
impl Default for Input {
    fn default() -> Self {
        Self {
            held: Default::default(),
            pressed: Default::default(),
            released: Default::default(),
            // The original MouseHandler starts outside the canvas. Focus and
            // button events do not establish a cursor position; motion does.
            pointer: [-1, -1],
            press_position: [0, 0],
            buttons: 0,
            pressed_buttons: 0,
            released_buttons: 0,
            wheel: 0,
            text: Vec::new(),
            key_events: Vec::new(),
            pressed_order: Vec::new(),
            released_order: Vec::new(),
            key_buffer: None,
        }
    }
}
impl Input {
    /// Construct the native widget consumer before receiving platform events.
    pub fn buffered() -> Self {
        Self {
            key_buffer: Some(KeyBuffer::default()),
            ..Self::default()
        }
    }

    /// fz.cl -> tg.ax -> fz.ae: expose the previous batch, then admit this
    /// tick's events into the space left before that previous batch is drained.
    pub fn advance_key_events(&mut self) {
        let Some(buffer) = self.key_buffer.as_mut() else {
            return;
        };
        std::mem::swap(&mut self.key_events, &mut buffer.pending);
        buffer.pending.clear();
        buffer
            .pending
            .extend(buffer.queued.drain(..).take(127 - self.key_events.len()));
        self.text.clear();
        self.text.extend(
            self.key_events
                .iter()
                .filter_map(|&(key, value)| (key == -1).then_some(value as u16)),
        );
    }

    /// Title/reconnect input belongs to the frame consumer, not later widgets.
    pub fn discard_key_events(&mut self) {
        self.key_events.clear();
        self.text.clear();
        if let Some(buffer) = self.key_buffer.as_mut() {
            buffer.queued.clear();
            buffer.pending.clear();
        }
    }

    fn queue_key_event(&mut self, event: (i32, i32)) {
        if let Some(buffer) = self.key_buffer.as_mut() {
            // No tick can admit more than 127 entries from the platform batch.
            if buffer.queued.len() < 127 {
                buffer.queued.push(event);
            }
        } else if self.key_events.len() < 128 {
            self.key_events.push(event);
        }
    }

    pub fn apply(&mut self, update: Update) {
        match update {
            Update::Pointer(point) => self.pointer = point,
            Update::Button(button, down) => self.button(button, down),
            Update::ButtonAt(button, down, point) => self.button_at(button, down, point),
            Update::Key(key, down) => self.key(key, down),
            Update::Character(value) => self.character(value),
            Update::Wheel(delta) => self.wheel = self.wheel.wrapping_add(delta),
            Update::LoseFocus => self.lose_focus(),
        }
    }

    pub fn begin_cycle(&mut self) {
        self.pressed.clear();
        self.released.clear();
        self.pressed_buttons = 0;
        self.released_buttons = 0;
        self.wheel = 0;
        self.text.clear();
        self.key_events.clear();
        self.pressed_order.clear();
        self.released_order.clear();
    }
    pub fn key(&mut self, key: i32, down: bool) {
        if !(0..112).contains(&key) {
            return;
        }
        if down {
            self.held.insert(key);
            self.pressed.insert(key);
            if self.pressed_order.len() < 128 {
                self.pressed_order.push(key);
            }
            self.queue_key_event((key, 0));
            self.released.remove(&key);
        } else {
            self.held.remove(&key);
            self.released.insert(key);
            if self.released_order.len() < 128 {
                self.released_order.push(key);
            }
            self.pressed.remove(&key);
        }
    }
    pub fn button(&mut self, button: u8, down: bool) {
        self.button_at(button, down, self.pointer);
    }
    pub fn button_at(&mut self, button: u8, down: bool, point: [i32; 2]) {
        if button >= 3 {
            return;
        }
        let bit = 1 << button;
        if down {
            self.press_position = point;
            // Java retains one current button and the most recent press
            // since the last cycle, even when physical buttons overlap.
            self.buttons = bit;
            self.pressed_buttons = bit;
        } else {
            // Any release clears the current button, including a release of
            // an earlier physical press that is no longer the current one.
            self.released_buttons |= self.buttons;
            self.buttons = 0;
        }
    }
    pub fn character(&mut self, value: u16) {
        if !typed_character(value) {
            return;
        }
        if self.key_buffer.is_none() {
            self.text.push(value);
        }
        self.queue_key_event((-1, value as i32));
    }
    pub fn lose_focus(&mut self) {
        let mut held = self.held.drain().collect::<Vec<_>>();
        held.sort_unstable();
        // The original handler queues ordinary releases for held keys in
        // ascending order, including clearing the same-cycle pressed flag.
        for key in held {
            self.key(key, false);
        }
        self.released_buttons |= self.buttons;
        self.buttons = 0;
    }
    pub fn execute(&self, opcode: u16, s: &mut Stacks) -> Result<bool> {
        if !(3500..=3502).contains(&opcode) {
            return Ok(false);
        }
        let key = s.pop_int()?;
        let keys = match opcode {
            3500 => &self.held,
            3501 => &self.pressed,
            _ => &self.released,
        };
        s.push_int(keys.contains(&key) as i32)?;
        Ok(true)
    }
}

/// Give the editor each platform event once before updating frame and simulation input.
pub fn dispatch(
    update: Update,
    views: [&mut Input; 2],
    editor: Option<&mut crate::interface_state::Interfaces>,
    read: impl FnMut(usize) -> Option<Vec<u16>>,
    write: impl FnMut(&[u16]) -> bool,
) -> Result<()> {
    if matches!(update, Update::Character(value) if !typed_character(value)) {
        return Ok(());
    }
    if let Some(editor) = editor {
        let consumed = match update {
            Update::Character(ch) => editor.editor_character(ch)?,
            Update::Key(key, down) => editor.editor_key_with_clipboard(key, down, read, write)?,
            Update::LoseFocus => {
                editor.editor_key(81, false)?;
                editor.editor_key(82, false)?;
                false
            }
            _ => false,
        };
        if consumed {
            return Ok(());
        }
    }
    for view in views {
        view.apply(update);
    }
    Ok(())
}

/// Physical Win32 button index before original Alt/Meta logical remapping.
pub fn windows_mouse_button(message: u32, flags: usize) -> Option<(u8, bool)> {
    Some(match message {
        0x201 => (0, true),
        0x202 => (0, false),
        0x204 => (1, true),
        0x205 => (1, false),
        0x207 => (2, true),
        0x208 => (2, false),
        0x20b | 0x20c => {
            let extra = (flags >> 16) & 0xffff;
            if !(1..=2).contains(&extra) {
                return None;
            }
            (extra as u8 + 2, message == 0x20b)
        }
        _ => return None,
    })
}
