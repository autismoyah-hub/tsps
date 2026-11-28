//! Revision 239 widget opcodes, hosted by the native script VM.
use crate::{
    disk_cache::DiskCache,
    interface_state::{Interfaces, ModelSource},
    item::Definitions,
    vm::{Object, Stacks},
    Error, Result,
};
pub fn ints<const N: usize>(s: &mut Stacks) -> Result<[i32; N]> {
    let mut out = [0; N];
    for i in (0..N).rev() {
        out[i] = s.pop_int()?;
    }
    Ok(out)
}
pub fn string(s: &mut Stacks) -> Result<String> {
    match s.pop_object()? {
        Object::String(v) => Ok(v),
        Object::Null => Ok("null".into()),
        _ => Err(Error::Invalid("script string argument")),
    }
}
pub fn execute(
    opcode: u16,
    dot: bool,
    s: &mut Stacks,
    cache: &mut DiskCache,
    state: &mut Interfaces,
    active: &mut [Option<u32>; 2],
    canvas: [i32; 2],
    items: &Definitions,
) -> Result<bool> {
    state.geometry_canvas.set(canvas);
    state.sync_selected_widgets(active);
    if matches!(opcode,1301|2301) {return crate::script_drag::set_parent(state,active,dot,opcode==2301,s,cache);}
    let mounted=if matches!(opcode,106|107) || (1000..2000).contains(&opcode) {state.mount_selected_widget(usize::from(dot))}else{None};
    let result=execute_inner(opcode,dot,s,cache,state,active,canvas,items);
    if let Some(mounted)=mounted {state.unmount_selected_widget(mounted);}
    state.sync_selected_widgets(active);
    result
}
fn execute_inner(
    opcode: u16,
    dot: bool,
    s: &mut Stacks,
    cache: &mut DiskCache,
    state: &mut Interfaces,
    active: &mut [Option<u32>; 2],
    canvas: [i32; 2],
    items: &Definitions,
) -> Result<bool> {
    if (211..=216).contains(&opcode) {
        let mut query = std::mem::take(&mut state.widget_query);
        let result = query.execute(opcode, dot, s, cache, state, active);
        state.widget_query = query;
        return result;
    }
    let slot = usize::from(dot);
    if (217..=222).contains(&opcode) {
        let ordinal=if opcode==217{s.pop_int()?}else{0};
        let selected=state.selected_navigation(slot);
        let node=selected.map(|(_,node)|node);
        if opcode==222 {crate::widget_navigation::target(opcode,node,ordinal,&[])?;return Ok(true);}
        if opcode==217&&node.is_none_or(|node|node.kind!=0) {return Err(Error::Invalid("widget navigation container"));}
        if node.is_none()||matches!(opcode,220|221)&&node.is_some_and(|node|node.kind!=0) {s.push_int(0)?;return Ok(true);}
        let owner=selected.and_then(|(owner,_)|owner);
        if owner.is_none() {return Err(Error::Invalid("widget navigation owner"));}
        if let Some(owner)=owner {state.load_group(cache,(owner>>16)as u16)?;}
        if opcode==217&&owner.and_then(|owner|state.widgets.get(&owner)).is_none_or(|widget|widget.kind!=0) {return Err(Error::Invalid("widget navigation owner"));}
        let children=state.dynamic.iter().filter(|((parent,_),_)|Some(*parent)==owner)
            .filter_map(|((_,index),child)|state.runtime.get(child).map(|runtime|(*index,runtime.tree_parent))).collect::<Vec<_>>();
        let target=crate::widget_navigation::target(opcode,node,ordinal,&children)?
            .and_then(|index|owner.and_then(|owner|state.dynamic.get(&(owner,index)).copied()));
        s.push_int(target.is_some() as i32)?;
        if target.is_some() {active[slot]=target;state.select_widget(slot,target);}
        return Ok(true);
    }

    if (3135..=3140).contains(&opcode) {
        let logical = state.selected_identity(slot).map(|(logical,_)|logical);
        return state.shortcut_scope.execute(opcode,s,logical);
    }

    match opcode {
        210 => {
            let kind2 = s.pop_int()?;
            let kind1 = s.pop_int()?;
            let value2 = crate::param_def::pop(s, kind2)?;
            let key2 = s.pop_int()?;
            let value1 = crate::param_def::pop(s, kind1)?;
            let key1 = s.pop_int()?;
            let default1 = state.parameter_default(cache, key1)?;
            let default2 = state.parameter_default(cache, key2)?;
            let parent = s.pop_int()? as u32;
            state.load_group(cache, (parent >> 16) as u16)?;
            let target = state
                .dynamic
                .iter()
                .filter(|((p, _), _)| *p == parent)
                .filter(|(_, id)| {
                    let r = &state.runtime[id];
                    (key1 < 0
                        || crate::param_def::equal(
                            r.params.get(&key1).unwrap_or(&default1),
                            &value1,
                        ))
                        && (key2 < 0
                            || crate::param_def::equal(
                                r.params.get(&key2).unwrap_or(&default2),
                                &value2,
                            ))
                })
                .min_by_key(|((_, child), _)| *child)
                .map(|(_, &id)| id);
            s.push_int(target.is_some() as i32)?;
            if target.is_some() {
                active[slot] = target; state.select_widget(slot,active[slot]);
            }
            return Ok(true);
        }
        2703 => {
            let [key, parent, child] = ints(s)?;
            state.load_group(cache, (parent as u32 >> 16) as u16)?;
            let id = if child == -1 {
                parent as u32
            } else {
                *state
                    .dynamic
                    .get(&(parent as u32, child))
                    .ok_or(Error::Invalid("widget parameter child"))?
            };
            let default = state.parameter_default(cache, key)?;
            let value = state
                .runtime
                .get(&id)
                .ok_or(Error::Invalid("widget parameter target"))?
                .params
                .get(&key)
                .cloned()
                .unwrap_or(default);
            crate::param_def::push(s, value)?;
            return Ok(true);
        }
        2702 => {
            let parent = s.pop_int()? as u32;
            s.push_int(state.children.contains_key(&parent) as i32)?;
            return Ok(true);
        }
        2704 => {
            let [parent, child, kind] = ints(s)?;
            let value = match kind {
                -1 => None,
                0 => Some(crate::vm::Value::Int(s.pop_int()?)),
                1 => Some(crate::vm::Value::Long(s.pop_long()?)),
                2 => Some(crate::vm::Value::Object(s.pop_object()?)),
                _ => return Err(Error::Invalid("widget parameter type")),
            };
            let key = s.pop_int()?;
            state.load_group(cache, (parent as u32 >> 16) as u16)?;
            let id = if child == -1 {
                parent as u32
            } else {
                *state
                    .dynamic
                    .get(&(parent as u32, child))
                    .ok_or(Error::Invalid("widget parameter child"))?
            };
            let params = &mut state
                .runtime
                .get_mut(&id)
                .ok_or(Error::Invalid("widget parameter target"))?
                .params;
            if let Some(v) = value {
                params.insert(key, v);
            } else {
                params.remove(&key);
            }
            return Ok(true);
        }
        2706 => {
            s.push_int(state.root.map_or(-1, |g| g as i32))?;
            return Ok(true);
        }
        100 => {
            let [parent, kind, index, strict] = ints(s)?;
            state.load_group(cache, ((parent as u32) >> 16) as u16)?;
            active[slot] =
                Some(state.create_child(parent as u32, index, kind as u8, strict != 0)?);
            state.select_widget(slot,active[slot]);
            return Ok(true);
        }
        101 => {
            let (owner,node)=state.selected_delete_node(slot).ok_or(Error::Invalid("active widget"))?;
            if node.index<0 {return Err(Error::Invalid("delete static widget"));}
            state.load_group(cache,(owner>>16)as u16)?;
            state.delete_script_widget(owner,node)?;
            return Ok(true);
        }
        102 => {
            state.delete_children(s.pop_int()? as u32);
            return Ok(true);
        }
        105 => {
            let [parent, index, next] = ints(s)?;
            let source = *state
                .dynamic
                .get(&(parent as u32, index))
                .ok_or(Error::Invalid("copy widget source"))?;
            let mut w = state.widgets[&source].clone();
            if w.kind == 0 {
                return Err(Error::Invalid("copy container widget"));
            }
            let mut runtime = state.runtime[&source].clone();
            // GameWidget copy creates mg(source): content is shared, request is cleared.
            if let Some(loader) = runtime.remote_loader.as_mut() { loader.request = None; }
            runtime.remote_changed.set(false);
            let target = state.create_child(parent as u32, next, w.kind, true)?;
            w.id = target;
            runtime.child_index = next;
            runtime.instance = state.runtime[&target].instance;
            if w.kind == 12 {
                // ex.105 calls el.cv on the copy: the editor bundle is newly initialized.
                runtime.editor = Some(Default::default());
                runtime.key_modes = Some(Default::default());
                runtime.listeners.retain(|opcode, _| !(1436..=1439).contains(opcode));
            }
            state.widgets.insert(target, w);
            state.runtime.insert(target, runtime);
            if let Some(container)=state.script_child(parent as u32,state.runtime[&target].tree_parent) {
                let bounds=&mut state.runtime.get_mut(&container).unwrap().child_bounds;
                bounds[0]=bounds[0].min(next);bounds[1]=bounds[1].max(next);
            }
            active[slot] = Some(target); state.select_widget(slot,active[slot]);
            return Ok(true);
        }
        106 | 107 => {
            let [kind, index] = ints(s)?;
            let selected = active[slot].ok_or(Error::Invalid("active widget"))?;
            let parent = if opcode == 106 {
                selected
            } else {
                state.script_parent_id(selected)
                    .and_then(|owner| state.script_child(owner, state.runtime[&selected].tree_parent))
                    .ok_or(Error::Invalid("nested widget parent"))?
            };
            if state.widgets.get(&parent).is_none_or(|w| w.kind != 0) {
                return Err(Error::Invalid("nested widget container"));
            }
            let owner = state.script_parent_id(parent).ok_or(Error::Invalid("nested widget owner"))?;
            let tree_parent = state.runtime[&parent].child_index;
            let id = state.create_child(owner, index, kind as u8, true)?;
            let bounds=&mut state.runtime.get_mut(&parent).unwrap().child_bounds;
            bounds[0]=bounds[0].min(index);bounds[1]=bounds[1].max(index);
            state.runtime.get_mut(&id).unwrap().tree_parent = tree_parent;
            // A static selected container has be=-1: the new widget is a root-array child.
            state.widgets.get_mut(&id).unwrap().parent = Some(if tree_parent == -1 { owner } else { parent });
            active[slot] = Some(id); state.select_widget(slot,active[slot]);
            return Ok(true);
        }
        204..=209 => {
            let selected=state.selected_navigation(slot);
            let node=selected.map(|(_,node)|node);
            let owner=selected.and_then(|(owner,_)|owner);
            if opcode==209 {s.push_int(node.map_or(-1,|node|node.parent))?;return Ok(true);}
            let Some(owner)=owner else{s.push_int(0)?;return Ok(true);};
            state.load_group(cache,(owner>>16)as u16)?;
            let children=state.dynamic.iter().filter(|((parent,_),_)|*parent==owner)
                .filter_map(|((_,index),id)|state.runtime.get(id).map(|runtime|(*index,runtime.tree_parent))).collect::<Vec<_>>();
            let (value,target)=crate::widget_navigation::relative(opcode,node,state.child_array_allocated(owner),&children)?;
            let target=target.and_then(|index|state.script_child(owner,index));
            s.push_int(value)?;
            if target.is_some() {active[slot]=target;state.select_widget(slot,target);}
            return Ok(true);
        }
        200 | 201 => {
            let id = if opcode == 200 {
                let [parent, index] = ints(s)?;
                state.dynamic.get(&(parent as u32, index)).copied()
            } else {
                let id = s.pop_int()? as u32;
                state.load_group(cache, (id >> 16) as u16)?;
                state.widgets.contains_key(&id).then_some(id)
            };
            s.push_int(id.is_some() as i32)?;
            if id.is_some() {
                active[slot] = id; state.select_widget(slot,active[slot]);
            }
            return Ok(true);
        }
        _ => {}
    }
    if !(1000..=1802).contains(&opcode) && !(2000..=2802).contains(&opcode) {
        return Ok(false);
    }
    let (op, id) = if opcode >= 2000 {
        let id = s.pop_int()? as u32;
        state.load_group(cache, (id >> 16) as u16)?;
        (opcode - 1000, id)
    } else {
        (opcode, active[slot].ok_or(Error::Invalid("active widget"))?)
    };
    let geometry = if matches!(op, 1100 | 1500..=1503) {
        state.script_geometry(id, canvas)?
    } else {
        None
    };
    let masks = if op == 1800 {
        let r = &state.runtime[&id];
        Some(state.masks(r.logical_id, r.child_index))
    } else {
        None
    };
    if op == 1703 {
        let key = s.pop_int()?;
        let default = state.parameter_default(cache, key)?;
        let value = state
            .runtime
            .get(&id)
            .ok_or(Error::Invalid("widget parameter target"))?
            .params
            .get(&key)
            .cloned()
            .unwrap_or(default);
        crate::param_def::push(s, value)?;
        return Ok(true);
    }
    if (1350..=1355).contains(&op) {
        let runtime=state.runtime.get_mut(&id).ok_or(Error::Invalid("shortcut widget"))?;
        return runtime.shortcuts.execute(op,opcode>=2000,s);
    }
    if op == 1140 || op == 1141 {
        let value = s.pop_int()? == 1;
        if op == 1140 { state.set_editor_focus(id, value); }
        else { state.set_editor_enabled(id, value); }
        return Ok(true);
    }
    if op == 1506 {
        let runtime = state.runtime.get(&id).ok_or(Error::Invalid("script widget"))?;
        let parent = state.script_parent_id(id).or_else(|| {
            state.children.values().find(|sub| sub.group == (runtime.logical_id >> 16) as u16).map(|sub| sub.parent)
        });
        s.push_int(parent.map_or(-1, |id| id as i32))?;
        return Ok(true);
    }
    if op == 1210 { state.ensure_appearance_kits(cache)?; }
    let w = state
        .widgets
        .get_mut(&id)
        .ok_or(Error::Invalid("script widget"))?;
    let r = state.runtime.get_mut(&id).unwrap();
    if (1400..=1439).contains(&op) && !matches!(op, 1413 | 1432 | 1433) {
        let mut signature = string(s)?;
        let triggers = if signature.ends_with('Y') {
            signature.pop();
            let count = s.pop_int()?;
            if count < 0 || count > 1000 {
                return Err(Error::Invalid("listener trigger count"));
            }
            let mut ids = vec![0; count as usize];
            for i in (0..ids.len()).rev() {
                ids[i] = s.pop_int()?;
            }
            Some(ids)
        } else {
            None
        };
        let mut args = Vec::new();
        for ch in signature.chars().rev() {
            args.push(match ch {
                's' | 'W' | 'X' => crate::vm::Value::Object(s.pop_object()?),
                'Ï' => crate::vm::Value::Long(s.pop_long()?),
                _ => crate::vm::Value::Int(s.pop_int()?),
            });
        }
        let script = s.pop_int()?;
        if op >= 1436 && r.editor.is_none() {
            // eq.au still enables GameWidget.ec when there is no editor to receive the callback.
            r.listeners.insert(op, (Vec::new(), None));
            return Ok(true);
        }
        if script == -1 {
            // An explicit clear must override a listener decoded from the cache.
            r.listeners.insert(op, (Vec::new(), None));
        } else {
            args.push(crate::vm::Value::Int(script));
            args.reverse();
            r.listeners.insert(op, (args, triggers));
        }
        return Ok(true);
    }
    match op {
        1704 => {
            let kind = s.pop_int()?;
            let value = match kind {
                -1 => None,
                0 => Some(crate::vm::Value::Int(s.pop_int()?)),
                1 => Some(crate::vm::Value::Long(s.pop_long()?)),
                2 => Some(crate::vm::Value::Object(s.pop_object()?)),
                _ => return Err(Error::Invalid("widget parameter type")),
            };
            let key = s.pop_int()?;
            if let Some(v) = value {
                r.params.insert(key, v);
            } else {
                r.params.remove(&key);
            }
        }
        1000 => {
            let [x, y, xa, ya] = ints(s)?;
            w.raw_position = [x, y];
            w.position_alignment = [xa as i8, ya as i8];
        }
        1001 => {
            let [x, y, xa, ya] = ints(s)?;
            w.raw_size = [x, y];
            w.size_alignment = [xa as i8, ya as i8];
        }
        1003 => w.hidden = s.pop_int()? == 1,
        1005 => w.no_click_through = s.pop_int()? == 1,
        1006 => w.no_scroll_through = s.pop_int()? == 1,
        1100 => {
            let size = geometry.map_or(w.raw_size, |g| g.size);
            let scroll=ints(s)?;
            if let Some(editor)=&mut r.editor {editor.set_scroll(scroll)?;return Ok(true);}
            r.scroll = scroll;
            for i in 0..2 {
                r.scroll[i] = r.scroll[i].clamp(0, (w.scroll_size[i] - size[i]).max(0));
            }
        }
        1101 => w.colors[0] = s.pop_int()? as u32,
        1102 => w.filled = s.pop_int()? == 1,
        1103 => w.transparency = s.pop_int()? as u8,
        1104 => w.line_width = s.pop_int()? as u8,
        1105 => w.sprites[0] = s.pop_int()?,
        1106 => w.sprite_angle = s.pop_int()? as u16,
        1107 => w.sprite_tiling = s.pop_int()? == 1,
        1108 => {
            let model = s.pop_int()?;
            w.models[0] = model;
            r.model = ModelSource::Raw(model);
        }
        1109 => {
            let [x, y, xa, ya, za, zoom] = ints(s)?;
            w.model_offset = [x, y];
            w.model_angles = [xa as u16, ya as u16, za as u16];
            w.model_zoom = zoom as u16;
            r.model_zoom = zoom;
        }
        1110 => {
            let seq = s.pop_int()?;
            if w.sequences[0] != seq {
                w.sequences[0] = seq;
                r.frame = 0;
                r.frame_cycle = 0;
            }
        }
        1111 => w.model_orthographic = s.pop_int()? == 1,
        1112 => {let text=string(s)?;if let Some(editor)=r.editor.as_mut().filter(|e|e.text.ready) {editor.set_text(&text.encode_utf16().collect::<Vec<_>>())?;}else{w.text=text;}},
        1113 => {w.font=s.pop_int()?;if let Some(editor)=&mut r.editor {editor.text.ready=false;}},
        1114 => {
            let [x, y, h] = ints(s)?;
            if let Some(editor)=&mut r.editor {editor.set_alignment(x,y,h);}else{w.text_alignment = [x as u8, y as u8];w.line_height = h as u8;}
        }
        1115 => w.text_shadow = s.pop_int()? == 1,
        1116 => w.outline = s.pop_int()? as u8,
        1117 => w.sprite_shadow = s.pop_int()? as u32,
        1118 => w.flip_vertical = s.pop_int()? == 1,
        1119 => w.flip_horizontal = s.pop_int()? == 1,
        1120 => w.scroll_size = ints(s)?,
        1122 => w.sprites[1] = s.pop_int()?,
        1123 => w.colors[1] = s.pop_int()? as u32,
        1124 => w.transparency_bottom = s.pop_int()?,
        1125 => {
            let mode = s.pop_int()?;
            if (0..=2).contains(&mode) { w.fill_mode = mode as u8; }
        }
        1126 => w.line_reversed = s.pop_int()? == 1,
        1127 => w.model_transparent = s.pop_int()? == 1,
        1131 => {
            let [key,value]=ints(s)?;
            if w.kind==11 {r.remote_mappings.get_or_insert_with(Default::default).fonts.borrow_mut().insert(key,value);}
        }
        1132 => {
            let key=match s.pop_object()? {
                Object::String(value)=>Some(value), Object::Null=>None,
                _=>return Err(Error::Invalid("remote widget mapping key")),
            };
            let value=s.pop_int()?;
            if w.kind==11 {r.remote_mappings.get_or_insert_with(Default::default).endpoints.insert(key,value);}
        }
        1133 | 1134 | 1136 => {let color=s.pop_int()? as u32;if let Some(editor)=&mut r.editor {editor.colors[match op {1133=>0,1134=>1,_=>2}]=color;}},
        1135 => {let text=string(s)?;if let Some(editor)=&mut r.editor {editor.pending_placeholder=text;}},
        1137 => {let width=s.pop_int()?;if let Some(editor)=&mut r.editor {editor.set_width(width)?;}},
        1138 => {let lines=s.pop_int()?;if let Some(editor)=&mut r.editor {editor.set_max_lines(lines)?;}},
        1139 => {let limit=s.pop_int()?;if let Some(editor)=&mut r.editor {editor.set_limit(limit)?;}},
        1142 => {let [anchor,cursor]=ints(s)?;if let Some(editor)=&mut r.editor {editor.select(anchor,cursor)?;}},
        1143 => {let cursor=s.pop_int()?;if let Some(editor)=&mut r.editor {editor.select(cursor,cursor)?;}},
        1144 => {let mode=s.pop_int()?;if let Some(editor)=&mut r.editor {editor.options.wrap=mode;}},
        1145 => {let mode=s.pop_int()?;if let Some(editor)=&mut r.editor {editor.enter_mode=mode;}},
        1146 => {let mode=s.pop_int()?;if let Some(editor)=&mut r.editor {editor.set_filter(mode)?;}},
        1147 => {let mode=s.pop_int()?;if let Some(editor)=&mut r.editor {editor.options.mask=mode;}},
        1148 | 1149 => {
            let [key,mode]=ints(s)?;
            if let Some(modes)=&mut r.key_modes {if op==1148 {modes.set_key(key,mode)}else{modes.set_char(key,mode)}}
        }
        1150 => {
            let text = match s.pop_object()? {
                Object::String(value) => Some(value), Object::Null => None,
                _ => return Err(Error::Invalid("remote inline text type")),
            };
            if w.kind == 11 {
                if let Some(text) = text {
                    r.remote_loader = Some(Box::default());
                    r.remote_loader.as_mut().unwrap().inline(&text)?;
                }
            }
        }
        1200 | 1205 | 1212 => {
            let [item, count] = ints(s)?;
            let d = items
                .items
                .get(&(item as u32))
                .ok_or(Error::Invalid("script item"))?;
            let d = items
                .items
                .get(&d.count_variant(count))
                .ok_or(Error::Invalid("script item variant"))?;
            r.item = item;
            r.count = count;
            w.model_angles = d.angles.map(|v| v as u16);
            w.model_offset = d.offset;
            r.model_zoom = d.zoom;
            r.quantity_mode = if op == 1205 {
                0
            } else if op == 1212 || d.stackable == 1 {
                1
            } else {
                2
            };
            let scale = if w.model_width_scale > 0 {
                w.model_width_scale as i32
            } else {
                w.raw_size[0]
            };
            if scale > 0 {
                r.model_zoom = r.model_zoom.wrapping_mul(32) / scale;
            }
        }
        1201 => r.model = ModelSource::Npc(s.pop_int()?),
        1214 => r.model = ModelSource::Object(s.pop_int()?),
        1215 => r.model = ModelSource::NpcBody(s.pop_int()?),
        1202 => r.model = ModelSource::Player,
        1207 => {
            let copy_objects = s.pop_int()? == 1;
            r.model = ModelSource::PlayerSelf(copy_objects);
            r.player_appearance = state.local_player.as_ref().map(|a| crate::widget_appearance::Composition::snapshot(a, copy_objects));
            if !copy_objects && r.player_appearance.is_none() { return Err(Error::Invalid("widget player composition missing")); }
        }
        1208 => {
            let id = s.pop_int()?;
            let definition = items.items.get(&(id as u32)).ok_or(Error::Invalid("widget appearance item"))?;
            r.player_appearance.as_mut().ok_or(Error::Invalid("widget player composition missing"))?.equip(definition)?;
            r.player_item = Some(id);
        }
        1209 => {
            let [index, color] = ints(s)?;
            r.player_appearance.as_mut().ok_or(Error::Invalid("widget player composition missing"))?.color(index, color)?;
        }
        1210 => {
            let body = s.pop_int()?;
            let body = u8::try_from(body).map_err(|_| Error::Invalid("widget appearance body"))?;
            r.player_appearance.as_mut().ok_or(Error::Invalid("widget player composition missing"))?.body(state.local_player.as_ref().map_or(0, |a| a.gender), body, state.appearance_kits.as_ref().unwrap(), items)?;
            r.player_body = body;
        }

        1300 => {
            let index = s.pop_int()?;
            let text = string(s)?;
            w.set_operation(index, text);
        }
        1310 => w.clear_sub_operations(s.pop_int()?)?,
        1311 => {
            let [operation, sub] = ints(s)?;
            if !(1..=10).contains(&operation) || sub < 1 {
                return Err(Error::Invalid("widget sub-operation index"));
            }
            w.set_sub_operation(operation, sub, string(s)?)?;
        }
        1312 => w.set_operation_priority(s.pop_int()?),
        1302 => w.scrollbar = s.pop_int()? == 1,
        1303 => w.drag_zone = s.pop_int()?,
        1304 => w.drag_threshold = s.pop_int()?,
        1308 => r.prioritize_menu = s.pop_int()? == 1,
        1309 => {
            s.pop_int()?;
        }
        1305 => w.data_text = string(s)?,
        1306 => w.spell_action = string(s)?,
        1307 => w.actions.clear(),
        1500..=1503 => {
            let g = geometry.unwrap_or_default();
            s.push_int(match op {
                1500 => g.position[0],
                1501 => g.position[1],
                1502 => g.size[0],
                _ => g.size[1],
            })?;
        }
        1504 => s.push_int(w.hidden as i32)?,
        1505 => s.push_int(if r.child_index >= 0 { r.logical_id as i32 } else { w.parent.map_or(-1, |i| i as i32) })?,
        1600 => s.push_int(r.scroll[0])?,
        1601 => s.push_int(r.scroll[1])?,
        1602 => s.push_object(Object::String(r.editor.as_ref().map_or_else(||w.text.clone(),|e|String::from_utf16_lossy(&e.text.value))))?,
        1603 => s.push_int(w.scroll_size[0])?,
        1604 => s.push_int(w.scroll_size[1])?,
        1605 => s.push_int(r.model_zoom)?,
        1606 => s.push_int(w.model_angles[0] as i32)?,
        1607 => s.push_int(w.model_angles[2] as i32)?,
        1608 => s.push_int(w.model_angles[1] as i32)?,
        1609 => s.push_int(w.transparency as i32)?,
        1610 => s.push_int(w.transparency_bottom)?,
        1613 => s.push_int(w.fill_mode as i32)?,
        1614 => s.push_int(w.model_transparent as i32)?,
        1611 => s.push_int(w.colors[0] as i32)?,
        1612 => s.push_int(w.colors[1] as i32)?,
        1700 => s.push_int(r.item)?,
        1701 => s.push_int(if r.item != -1 { r.count } else { 0 })?,
        1702 => s.push_int(r.child_index)?,
        1617 | 1618 | 1620 => s.push_int(r.editor.as_ref().map_or(0,|e|e.colors[match op {1617=>0,1618=>1,_=>2}]) as i32)?,
        1619 => s.push_object(Object::String(r.editor.as_ref().map_or_else(String::new,|e|String::from_utf16_lossy(&e.placeholder))))?,
        1621 => s.push_int(r.editor.as_ref().map_or(0,|e|e.options.width))?,
        1622 => s.push_int(r.editor.as_ref().map_or(0,|e|e.options.max_lines))?,
        1623 => s.push_int(r.editor.as_ref().map_or(0,|e|e.text.max_chars))?,
        1624 => s.push_int(r.editor.as_ref().is_some_and(|e|e.text.focused) as i32)?,
        1625 => s.push_int(r.editor.as_ref().is_some_and(|e|e.text.enabled) as i32)?,
        1626 => s.push_object(Object::String(if let Some(editor)=&r.editor {String::from_utf16_lossy(editor.text.selected_text()?)}else{String::new()}))?,
        1627 => {let [anchor,cursor]=r.editor.as_ref().map_or([0;2],|e|[e.text.anchor,e.text.cursor]);s.push_int(anchor.min(cursor))?;s.push_int(anchor.max(cursor))?;},
        1628 => s.push_int(r.editor.as_ref().map_or(0,|e|e.text.cursor))?,
        1629 => s.push_int(r.editor.as_ref().map_or(0,|e|e.options.wrap))?,
        1630 => s.push_int(r.editor.as_ref().map_or(0,|e|e.enter_mode))?,
        1631 => s.push_int(r.editor.as_ref().map_or(0,|e|e.text.filter))?,
        1632 => s.push_int(r.editor.as_ref().map_or(0,|e|e.options.mask))?,
        1633 | 1634 => {
            let key=s.pop_int()?;
            s.push_int(r.key_modes.as_ref().map_or(0,|modes|if op==1633 {modes.key(key)}else{modes.character(key)}) as i32)?;
        }
        1800 => s.push_int(((masks.unwrap()[0] >> 11) & 63) as i32)?,
        1801 => {
            let index = s.pop_int()?;
            s.push_object(Object::String(if index > 0 {
                w.actions
                    .get(index as usize - 1)
                    .cloned()
                    .unwrap_or_default()
            } else {
                String::new()
            }))?;
        }
        1802 => s.push_object(Object::String(w.data_text.clone()))?,
        _ => return Err(Error::Io(format!("unported widget opcode {opcode}"))),
    }
    if matches!(op,1000|1001|1120) {state.align_script_widget(id,canvas,op!=1120)?;}
    state.flush_editor_events(id);
    Ok(true)
}
