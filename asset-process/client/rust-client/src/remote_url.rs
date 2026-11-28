//! Original URL and authenticated remote-widget dispatch, request polling and child creation.
use crate::{
    disk_cache::DiskCache,
    interface_state::Interfaces,
    remote_request::{absolute_url, Requester},
    vm::{Object, Stacks},
    Error, Result,
};
pub struct Context {
    pub pending_urls: std::collections::VecDeque<String>,
    pub user_id: i64,
    pub user_hash: i64,
    /// vs.ae in the pinned reference returns this fixed value (not a hardware query).
    pub device_id: String,
    pub crm: crate::remote_crm::Session,
    pub requester: Option<Requester>,
    pub registry: crate::remote_registry::Registry,
}
impl Default for Context {
    fn default() -> Self {
        Self {
            pending_urls: Default::default(),
            user_id: -1,
            user_hash: -1,
            device_id: "12345678-0000-0000-0000-123456789012".into(),
            crm: Default::default(),
            requester: Some(Requester::default()),
            registry: Default::default(),
        }
    }
}
pub fn execute(
    opcode: u16,
    dot: bool,
    s: &mut Stacks,
    cache: &mut DiskCache,
    state: &mut Interfaces,
    active: &[Option<u32>; 2],
    context: &mut Context,
) -> Result<bool> {
    if !matches!(opcode, 1130 | 2130 | 1151 | 2151 | 1152 | 2152 | 1707..=1709 | 2707..=2709) {
        return Ok(false);
    }
    let authenticated = matches!(opcode, 1151 | 2151);
    state.sync_selected_widgets(active);
    let mounted = (opcode < 2000)
        .then(|| state.mount_selected_widget(usize::from(dot)))
        .flatten();
    let result = (|| {
        let id = if opcode >= 2000 {
            let id = s.pop_int()? as u32;
            state.load_group(cache, (id >> 16) as u16)?;
            id
        } else {
            active[usize::from(dot)].ok_or(Error::Invalid("remote widget target"))?
        };
        let kind = state
            .widgets
            .get(&id)
            .ok_or(Error::Invalid("remote widget target"))?
            .kind;
        if matches!(opcode,1707..=1709|2707..=2709) {
            let query = if opcode >= 2000 {
                opcode - 1000
            } else {
                opcode
            };
            let runtime = state
                .runtime
                .get(&id)
                .ok_or(Error::Invalid("remote query runtime"))?;
            if query == 1707 {
                s.push_int(i32::from(runtime.remote_status == 2))?;
                return Ok(true);
            }
            let value = s.pop_object()?;
            if kind != 11 {
                if query == 1708 {
                    s.push_int(-1)?;
                } else {
                    s.push_object(Object::String(String::new()))?;
                }
                return Ok(true);
            }
            let key = match &value {
                Object::Null => None,
                Object::String(text) => Some(text.encode_utf16().collect::<Vec<_>>()),
                _ => return Err(Error::Invalid("remote query key type")),
            };
            // Java reserves the destination before evaluating the getter; failures retain its old value.
            if query == 1708 {
                s.reserve_int()?;
            } else {
                s.push_object(value)?;
            }
            let card = if runtime.remote_status == 2 {
                match runtime.remote_loader.as_ref() {
                    None => None,
                    Some(loader) => Some(
                        loader
                            .card
                            .as_ref()
                            .ok_or(Error::Invalid("remote query card null"))?,
                    ),
                }
            } else {
                None
            };
            if query == 1708 {
                s.pop_int()?;
                s.push_int(
                    card.and_then(|card| key.as_ref().and_then(|key| card.meta.get(key)))
                        .copied()
                        .unwrap_or(-1),
                )?;
            } else {
                let value =
                    card.and_then(|card| key.as_ref().and_then(|key| card.meta_strings.get(key)));
                s.pop_object()?;
                s.push_object(value.map_or(Object::Null, |text| Object::String(text.clone())))?;
            }
            return Ok(true);
        }
        if matches!(opcode, 1152 | 2152) {
            if kind != 11 {
                return Err(Error::Invalid("remote dismissal widget type"));
            }
            let card = state
                .runtime
                .get(&id)
                .and_then(|runtime| runtime.remote_loader.as_ref())
                .and_then(|loader| loader.card.as_ref());
            if let Some(card) = card {
                context.crm.event(2, card.id.clone());
            }
            s.push_int(i32::from(card.is_some()))?;
            return Ok(true);
        }
        fn string(value: Object) -> Result<Option<String>> {
            match value {
                Object::Null => Ok(None),
                Object::String(text) => Ok(Some(text)),
                _ => Err(Error::Invalid("remote URL type")),
            }
        }
        let (input, crm_base, location) = if authenticated {
            // Java removes all three object slots before evaluating their casts.
            let location = s.pop_object()?;
            let base = s.pop_object()?;
            let input = s.pop_object()?;
            (string(input)?, string(base)?, string(location)?)
        } else {
            (string(s.pop_object()?)?, None, Some(String::new()))
        };
        if !authenticated && (kind != 11 || context.user_id == -1) {
            return Ok(true);
        }
        let mut text = input;
        if context.user_id != -1 {
            text = Some(
                text.ok_or(Error::Invalid("remote URL null"))?
                    .replace("%userid%", &context.user_id.to_string()),
            );
        }
        if authenticated {
            if context.user_hash != -1 {
                text = Some(
                    text.ok_or(Error::Invalid("remote URL null"))?
                        .replace("%userhash%", &context.user_hash.to_string()),
                );
            }
            if !context.device_id.is_empty() {
                text = Some(
                    text.ok_or(Error::Invalid("remote URL null"))?
                        .replace("%deviceid%", &context.device_id),
                );
            }
        }
        let runtime = state
            .runtime
            .get_mut(&id)
            .ok_or(Error::Invalid("remote widget runtime"))?;
        runtime.remote_loader = Some(Box::default());
        let Some(text) = text.filter(|text| !text.is_empty()) else {
            runtime.remote_loader = None;
            return Ok(true);
        };
        let Some(requester) = context.requester.as_mut() else {
            runtime.remote_loader = None;
            return Ok(true);
        };
        let loader = runtime.remote_loader.as_mut().unwrap();
        loader.raw_url = Some(text.clone());
        let Some(url) = absolute_url(&text) else {
            runtime.remote_loader = None;
            return Ok(true);
        };
        loader.request = Some(requester.request(url)?);
        loader.status = 0;
        let location = location.ok_or(Error::Invalid("remote card location null"))?;
        if !location.is_empty() {
            loader.location = Some(location);
        }
        runtime.remote_mappings.get_or_insert_with(Default::default);
        if authenticated {
            loader.crm_attached = true;
            loader.crm_base = crm_base.clone();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;
            context.crm.start(
                crm_base,
                Some(context.device_id.clone()),
                Some(context.user_hash.to_string()),
                now,
            )?;
        }
        Ok(true)
    })();
    if let Some(mounted) = mounted {
        state.unmount_selected_widget(mounted);
    }
    state.sync_selected_widgets(active);
    result
}
pub fn poll(runtime: &mut crate::interface_state::Runtime, context: &mut Context) -> Result<bool> {
    let Some(loader) = runtime.remote_loader.as_mut() else {
        return Ok(false);
    };
    loader.poll_initial(context)?;
    if loader.status == runtime.remote_status {
        return Ok(false);
    }
    runtime.remote_status = loader.status;
    Ok(loader.status >= 3)
}
/// Build the original image-then-label child array. Missing font mappings fail
/// after preserving earlier children and the new array's full length.
pub fn generate_children(state: &mut Interfaces, id: u32, size: [i32; 2]) -> Result<()> {
    let runtime = state
        .runtime
        .get(&id)
        .ok_or(Error::Invalid("remote parent runtime"))?;
    let card = runtime
        .remote_loader
        .as_ref()
        .and_then(|l| l.card.clone())
        .ok_or(Error::Invalid("remote content absent"))?;
    let mappings = runtime.remote_mappings.clone();
    state.begin_remote_children(id, card.images.len() + card.labels.len())?;
    fn geometry(state: &mut Interfaces, id: u32, placement: [f32; 4], size: [i32; 2]) {
        let w = state.widgets.get_mut(&id).unwrap();
        w.raw_position = [
            (placement[0] * size[0] as f32) as i32,
            (placement[1] * size[1] as f32) as i32,
        ];
        w.raw_size = [
            (placement[2] * size[0] as f32) as i32,
            (size[1] as f32 * placement[3]) as i32,
        ];
    }
    for (i, image) in card.images.iter().enumerate() {
        let child = state.create_remote_child(id, i as i32, 5)?;
        geometry(state, child, image.placement, size);
        state.runtime.get_mut(&child).unwrap().remote_image = Some(image.request.clone());
    }
    for (i, label) in card.labels.iter().enumerate() {
        let font = mappings
            .as_ref()
            .and_then(|m| m.fonts.borrow().get(&label.font).copied())
            .ok_or(Error::Invalid("remote label font absent"))?;
        let child = state.create_remote_child(id, (card.images.len() + i) as i32, 4)?;
        geometry(state, child, label.placement, size);
        let widget = state.widgets.get_mut(&child).unwrap();
        widget.text = label.text.clone();
        widget.font = font;
        widget.text_alignment = label.alignment;
    }
    Ok(())
}
/// GameWidget.mq: update the observed status and generate children, before host lifecycle effects.
pub fn update_widget(
    state: &mut Interfaces,
    id: u32,
    size: [i32; 2],
    context: &mut Context,
) -> Result<bool> {
    let Some(runtime) = state.runtime.get_mut(&id) else {
        return Ok(false);
    };
    let before = runtime.remote_status;
    let terminal = poll(runtime, context)?;
    let ready = before != runtime.remote_status && runtime.remote_status == 2;
    if ready {
        generate_children(state, id, size)?;
    }
    Ok(terminal || ready)
}
pub fn poll_visible(
    state: &mut Interfaces,
    layout: &crate::widget_layout::Layout,
    context: &mut Context,
) -> Result<bool> {
    let mut children_changed = false;
    for &id in &layout.order {
        if !layout.widgets.get(&id).is_some_and(|p| !p.hidden)
            || !state.widgets.get(&id).is_some_and(|w| w.kind == 11)
        {
            continue;
        }
        let changed = update_widget(state, id, layout.widgets[&id].geometry.size, context)?;
        let ready = changed && state.runtime[&id].remote_status == 2;
        if ready {
            let pending = state.resize_events.borrow().len();
            state.align_remote_children(id)?;
            // ps.ap inserts these at this widget, before its remote and input listeners.
            for event in state.resize_events.borrow_mut().iter_mut().skip(pending) {
                event.before_widget = Some(id);
            }
            let loader = state.runtime[&id]
                .remote_loader
                .as_ref()
                .ok_or(Error::Invalid("remote impression loader"))?;
            if !loader.crm_attached {
                return Err(Error::Invalid("remote impression CRM null"));
            }
            let card = loader
                .card
                .as_ref()
                .ok_or(Error::Invalid("remote impression card null"))?;
            context.crm.event(3, card.id.clone());

            children_changed = true;
        }
        if changed {
            state.runtime.get(&id).unwrap().remote_changed.set(true);
        }
    }
    Ok(children_changed)
}
