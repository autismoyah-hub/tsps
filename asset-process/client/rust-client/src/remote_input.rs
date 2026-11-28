//! Remote-widget hit testing and the active vz.fk click contract.
use crate::{
    interface_state::Interfaces, protocol::Packet, remote_card::ParameterValue,
    remote_url::Context, Error, Result,
};
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Packet(Packet),
    Url(Option<String>),
}
pub fn hit(state: &Interfaces, id: u32, point: [i32; 2], origin: [i32; 2]) -> Result<bool> {
    let widget = state
        .widgets
        .get(&id)
        .ok_or(Error::Invalid("remote hit widget"))?;
    let runtime = &state.runtime[&id];
    if widget.kind != 11 || runtime.remote_status != 2 {
        return Ok(false);
    }
    let Some(loader) = runtime.remote_loader.as_ref() else {
        return Ok(false);
    };
    let card = loader
        .card
        .as_ref()
        .ok_or(Error::Invalid("remote hit card null"))?;
    let size = runtime.aligned.get().unwrap_or_default().size;
    let low = [
        (card.click_bounds[0] * size[0] as f32) as i32,
        (card.click_bounds[1] * size[1] as f32) as i32,
    ];
    let high = [
        low[0].wrapping_add((card.click_bounds[2] * size[0] as f32) as i32),
        low[1].wrapping_add((card.click_bounds[3] * size[1] as f32) as i32),
    ];
    let point = [
        point[0].wrapping_sub(origin[0]),
        point[1].wrapping_sub(origin[1]),
    ];
    Ok(point[0] >= low[0] && point[1] >= low[1] && point[0] < high[0] && point[1] < high[1])
}
pub fn click(state: &Interfaces, context: &mut Context, id: u32) -> Result<Option<Action>> {
    let Some(runtime) = state.runtime.get(&id) else {
        return Ok(None);
    };
    let Some(loader) = runtime.remote_loader.as_ref() else {
        return Ok(None);
    };
    let card = loader
        .card
        .as_ref()
        .ok_or(Error::Invalid("remote click card null"))?;
    if !loader.crm_attached {
        return Err(Error::Invalid("remote click CRM null"));
    }
    context.crm.event(1, card.id.clone());
    let widget = &state.widgets[&id];
    let maps = runtime.remote_mappings.as_ref();
    if maps.is_none_or(|maps| maps.endpoints.is_empty()) {
        // GameWidget.be discards each replaceAll result; the endpoint remains unchanged.
        return Ok(Some(Action::Url(
            (widget.kind == 11).then(|| card.endpoint.clone()).flatten(),
        )));
    }
    if state.interaction_flags(id) & (1 << 22) == 0 || widget.kind != 11 {
        return Ok(None);
    }
    let mut values = [0i32; 3];
    let mut count = 0;
    for parameter in &card.parameters {
        if parameter.name == "user_id" {
            continue;
        }
        let ParameterValue::Int(value) = &parameter.value else {
            return Ok(None);
        };
        let slot = values
            .get_mut(count)
            .ok_or(Error::Invalid("remote click parameter array overflow"))?;
        *slot = *value;
        count += 1;
    }
    let endpoint = card
        .endpoint
        .as_ref()
        .and_then(|endpoint| maps.unwrap().endpoints.get(&Some(endpoint.clone())))
        .copied()
        .unwrap_or(-1);
    let mut payload = Vec::with_capacity(22);
    fn middle(out: &mut Vec<u8>, value: i32) {
        let b = value.to_be_bytes();
        out.extend_from_slice(&[b[2], b[3], b[0], b[1]]);
    }
    middle(&mut payload, runtime.logical_id as i32);
    payload.extend_from_slice(&endpoint.to_be_bytes());
    middle(&mut payload, values[0]);
    middle(&mut payload, values[2]);
    payload.extend_from_slice(&values[1].to_le_bytes());
    payload.extend_from_slice(&(runtime.child_index as u16).to_le_bytes());
    Ok(Some(Action::Packet(Packet {
        opcode: 91,
        payload,
    })))
}
pub struct PointerEvent {
    event: crate::interface_state::EditorPointer,
    reference: Option<crate::widget_selection::Object>,
    dynamic: bool,
}
impl crate::client_state::ClientState {
    pub fn capture_widget_pointer(
        &mut self,
        layout: &crate::widget_layout::Layout,
    ) -> Vec<PointerEvent> {
        let events = self.interfaces.widget_pointer_events(
            layout,
            &self.input,
            self.preferences.mouse_camera,
        );
        events
            .into_iter()
            .map(|event| {
                let dynamic = self.interfaces.runtime[&event.widget].child_index >= 0;
                let reference = self.interfaces.hold_widget(event.widget);
                PointerEvent {
                    event,
                    reference,
                    dynamic,
                }
            })
            .collect()
    }
    pub fn dispatch_widget_pointer(&mut self, events: Vec<PointerEvent>) -> Result<()> {
        let mut saw_editor = false;
        for queued in events {
            let event = queued.event;
            let id = event.widget;
            let valid_dynamic = self.interfaces.runtime.get(&id).is_some_and(|r| {
                r.instance == event.instance
                    && self.interfaces.dynamic.get(&(r.logical_id, r.child_index)) == Some(&id)
            });
            let mounted = queued
                .reference
                .as_ref()
                .and_then(|reference| self.interfaces.mount_widget_object(reference));
            let result: Result<()> = (|| {
                if !self
                    .interfaces
                    .runtime
                    .get(&id)
                    .is_some_and(|r| r.instance == event.instance)
                {
                    return Ok(());
                }

                let kind = self.interfaces.widgets[&id].kind;
                saw_editor |= kind == 12;
                if queued.dynamic && !valid_dynamic {
                    return Ok(());
                }
                match kind {
                    12 => self
                        .interfaces
                        .dispatch_editor_pointer_one(event, &self.input)?,
                    11 if event.press && hit(&self.interfaces, id, event.point, [0, 0])? => {
                        match click(&self.interfaces, &mut self.remote, id)? {
                            Some(Action::Packet(packet)) => self.outbound.push_back(packet),
                            Some(Action::Url(Some(url))) => self.remote.pending_urls.push_back(url),
                            _ => (),
                        }
                    }
                    _ => (),
                }
                Ok(())
            })();
            if let Some(mounted) = mounted {
                self.interfaces.unmount_selected_widget(mounted);
            }
            result?;
        }
        if !saw_editor && self.input.pressed_buttons & 1 != 0 {
            self.interfaces.blur_editor();
        }
        Ok(())
    }
}
/// Native URL opening is bounded to web URLs; other handlers remain unsupported.
pub fn open_web_url(url: &str) -> Result<()> {
    let parsed = crate::remote_request::absolute_url(url).ok_or(Error::Invalid("external URL"))?;
    if !(parsed.starts_with("https://") || parsed.starts_with("http://")) || parsed.contains('\0') {
        return Err(Error::Unsupported("external URL scheme"));
    }
    #[cfg(all(windows, not(feature = "offscreen")))]
    {
        #[link(name = "shell32")]
        extern "system" {
            fn ShellExecuteW(
                window: *mut std::ffi::c_void,
                operation: *const u16,
                file: *const u16,
                parameters: *const u16,
                directory: *const u16,
                show: i32,
            ) -> *mut std::ffi::c_void;
        }
        let verb = "open\0".encode_utf16().collect::<Vec<_>>();
        let path = parsed
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let result = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                verb.as_ptr(),
                path.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                1,
            )
        } as isize;
        if result <= 32 {
            return Err(Error::Invalid("native browser launch failed"));
        }
        Ok(())
    }
    #[cfg(any(not(windows), feature = "offscreen"))]
    {
        Err(Error::Unsupported("native browser platform"))
    }
}
