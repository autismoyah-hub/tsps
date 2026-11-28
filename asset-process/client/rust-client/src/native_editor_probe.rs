//! Client-local editable-widget acceptance, only after --smoke-inventory completes.
use crate::{canvas_transform::Fit, client_state::ClientState, Error, Result};
#[derive(Default)]
pub struct Probe {
    phase: u8,
    container: Option<u32>,
    editor: Option<u32>,
    started: Option<std::time::Instant>,
    pub capture: bool,
    pub complete: bool,
}
type Message = (u32, usize, isize);
fn key(code: usize) -> Vec<Message> {
    vec![(0x100, code, 0), (0x101, code, 0)]
}
fn chars(text: &str) -> Vec<Message> {
    text.encode_utf16()
        .map(|c| (0x102, c as usize, 0))
        .collect()
}
fn pointer(fit: Fit, point: [i32; 2], held: bool) -> Message {
    let [x, y] = fit.to_physical(point);
    (
        0x200,
        usize::from(held),
        ((y as isize) << 16) | (x as isize & 65535),
    )
}
fn selection_start(fit: Fit, origin: [i32; 2], held: bool) -> Message {
    // Keep the drag inside the clip after physical-pixel quantization.
    pointer(fit, [origin[0] + 2, origin[1] + 3], held)
}
impl Probe {
    pub fn next(&mut self, client: &mut ClientState, fit: Fit) -> Result<Vec<Message>> {
        if self.complete {
            return Ok(Vec::new());
        }
        if self
            .started
            .is_some_and(|s| s.elapsed() > std::time::Duration::from_secs(20))
        {
            return Err(Error::Io(format!(
                "native editor acceptance timed out at phase {}",
                self.phase
            )));
        }
        if self.phase == 0 {
            self.started = Some(std::time::Instant::now());
            let layout = client.interfaces.layout(client.canvas)?;
            let parent = layout
                .order
                .iter()
                .copied()
                .find(|id| {
                    let p = &layout.widgets[id];
                    client.interfaces.widgets[id].kind == 0
                        && !p.hidden
                        && p.absolute == [0, 0]
                        && p.geometry.size[0] >= client.canvas[0]
                        && p.geometry.size[1] >= client.canvas[1]
                        && client.interfaces.runtime[id].scroll == [0, 0]
                })
                .ok_or(Error::Invalid("native editor acceptance root"))?;
            let slot = (0..=65535)
                .find(|i| !client.interfaces.dynamic.contains_key(&(parent, *i)))
                .ok_or(Error::Invalid("native editor acceptance slot"))?;
            let container = client.interfaces.create_child(parent, slot, 0, true)?;
            let w = client.interfaces.widgets.get_mut(&container).unwrap();
            w.raw_position = [30, 350];
            w.raw_size = [330, 88];
            w.no_click_through = true;
            let background = client.interfaces.create_child(container, 0, 3, true)?;
            let w = client.interfaces.widgets.get_mut(&background).unwrap();
            w.raw_size = [330, 88];
            w.filled = true;
            w.colors[0] = 0xfff4d6;
            let editor = client.interfaces.create_child(container, 1, 12, true)?;
            let w = client.interfaces.widgets.get_mut(&editor).unwrap();
            w.raw_position = [8, 8];
            w.raw_size = [314, 48];
            w.font = 495;
            w.colors[0] = 0x102030;
            w.no_click_through = true;
            let e = client
                .interfaces
                .runtime
                .get_mut(&editor)
                .unwrap()
                .editor
                .as_mut()
                .unwrap();
            e.enter_mode = 1;
            e.colors = [0xffffff, 0x365c89, 0x707070];
            e.pending_placeholder = "Click, type, drag and edit".into();
            let label = client.interfaces.create_child(container, 2, 4, true)?;
            let w = client.interfaces.widgets.get_mut(&label).unwrap();
            w.raw_position = [8, 62];
            w.raw_size = [314, 20];
            w.font = 495;
            w.text = "Native editor acceptance".into();
            w.colors[0] = 0x503010;
            self.container = Some(container);
            self.editor = Some(editor);
            self.phase = 1;
            println!("Native editor acceptance installed local widget {editor}");
            return Ok(Vec::new());
        }
        let id = self.editor.unwrap();
        let e = client
            .interfaces
            .runtime
            .get(&id)
            .and_then(|r| r.editor.as_ref())
            .ok_or(Error::Invalid(
                "native editor acceptance widget disappeared",
            ))?;
        let value = String::from_utf16_lossy(&e.text.value);
        let selected = [e.text.anchor, e.text.cursor];
        let layout = client.interfaces.layout(client.canvas)?;
        let origin = layout.widgets[&id].absolute;
        match self.phase {
            1 if e.text.ready => {
                self.phase = 2;
                let movement = pointer(fit, [origin[0] + 2, origin[1] + 3], false);
                println!("Posting Win32 editable widget focus click");
                return Ok(vec![
                    movement,
                    (0x201, 1, movement.2),
                    (0x202, 0, movement.2),
                ]);
            }
            2 if client.interfaces.focused_editor() == Some(id) && e.text.focused => {
                self.phase = 3;
                println!("Win32 editor focus confirmed; posting text");
                return Ok(chars("Rust239"));
            }
            3 if value == "Rust239" => {
                self.phase = 4;
                self.capture = true;
                let width = e
                    .text
                    .value
                    .iter()
                    .map(|&c| e.metrics.as_ref().unwrap().advances[c as usize] as i32)
                    .sum::<i32>();
                let movement = pointer(fit, [origin[0] + width + 2, origin[1] + 3], false);
                println!("Win32 editor text confirmed; posting selection drag");
                return Ok(vec![movement, (0x201, 1, movement.2)]);
            }
            4 if selected == [7, 7] => {
                self.phase = 5;
                // Downscaled windows can round the exact left edge outside the
                // editor, where Java's pointer dispatch correctly ignores it.
                return Ok(vec![selection_start(fit, origin, true)]);
            }
            5 if selected == [7, 0] => {
                self.phase = 6;
                let release = selection_start(fit, origin, false);
                println!("Win32 editor drag selection confirmed");
                return Ok(vec![(0x202, 0, release.2), (0x102, 81, 0)]);
            }
            6 if value == "Q" => {
                self.phase = 7;
                let mut messages = key(0x24);
                messages.push((0x100, 0x10, 0));
                messages.extend(key(0x23));
                messages.push((0x101, 0x10, 0));
                return Ok(messages);
            }
            7 if selected == [0, 1] => {
                self.phase = 8;
                println!("Win32 editor Shift+End selection confirmed");
                return Ok(chars("Z"));
            }
            8 if value == "Z" => {
                self.phase = 9;
                return Ok(key(8));
            }
            9 if value.is_empty() => {
                self.phase = 10;
                println!("Win32 editor backspace confirmed");
                return Ok(chars("native"));
            }
            10 if value == "native" => {
                self.phase = 11;
                return Ok(key(13));
            }
            11 if client.interfaces.focused_editor().is_none() => {
                if value != "native" {
                    return Err(Error::Invalid("native editor submit changed text"));
                }
                client.interfaces.delete_child(self.container.unwrap())?;
                self.complete = true;
                println!("Win32 editor submit/blur confirmed; local probe removed");
            }
            _ => {}
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{font::Metrics, input::Input, interface_state::Interfaces, widget::Widget};

    #[test]
    fn selection_drag_survives_native_window_scaling() {
        for width in 400..=1600 {
            let fit = Fit::new([765, 503], [width, 503]);
            let mut interfaces = Interfaces::default();
            let mut widget = Widget::empty(0, 12);
            widget.if3 = true;
            widget.raw_position = [38, 358];
            widget.raw_size = [314, 48];
            interfaces.install_group(0, vec![widget]).unwrap();
            interfaces.root = Some(0);
            let editor = interfaces
                .runtime
                .get_mut(&0)
                .unwrap()
                .editor
                .as_mut()
                .unwrap();
            editor
                .prepare(
                    &mut "Rust239".into(),
                    [314, 48],
                    Some((
                        495,
                        Metrics {
                            advances: [5; 256],
                            ascent: 12,
                            kerning: None,
                        },
                    )),
                )
                .unwrap();
            interfaces.focus_editor(Some(0));
            interfaces
                .runtime
                .get_mut(&0)
                .unwrap()
                .editor
                .as_mut()
                .unwrap()
                .mouse_press([40, 3], false, false)
                .unwrap();
            let message = selection_start(fit, [38, 358], true);
            let input = Input {
                pointer: fit.to_logical([(message.2 & 65535) as i32, (message.2 >> 16) as i32]),
                buttons: 1,
                ..Default::default()
            };
            let layout = interfaces.layout([765, 503]).unwrap();
            let events = interfaces.editor_pointer_events(&layout, &input, false);
            assert_eq!(events.len(), 1, "window width {width}: {:?}", input.pointer);
            interfaces.dispatch_editor_pointer(events, &input).unwrap();
            let text = &interfaces.runtime[&0].editor.as_ref().unwrap().text;
            assert_eq!([text.anchor, text.cursor], [7, 0], "window width {width}");
        }
    }
}
