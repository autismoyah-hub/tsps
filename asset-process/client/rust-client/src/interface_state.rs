//! Mutable interface trees, packet effects and pending interface script events.
use crate::{
    disk_cache::DiskCache,
    interface_packet::{Events, SubInterface, Update},
    inventory::Inventories,
    item::Definitions,
    widget::{Argument, Widget},
    widget_layout::Layout,
    Error, Result,
};
use std::{cell::RefCell, collections::{HashMap, HashSet, VecDeque}, rc::{Rc, Weak}};
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelSource {
    None,
    Raw(i32),
    Item(i32),
    Npc(i32),
    NpcBody(i32),
    Object(i32),
    ActiveNpc(u16),
    Player,
    PlayerSelf(bool),
}
/// A drag callback holds Java widget objects even after their child-array slots are cleared.
pub type DragObject = Rc<RefCell<DragReference>>;
pub struct DragReference {
    id: u32,
    instance: u64,
    retired: Option<DragSnapshot>,
}
#[derive(Clone)]
pub struct DragSnapshot {
    pub identity: (u32, i32, i32),
    parent: Option<u32>,
    flags: u32,
    pub complete_listener: Option<Vec<crate::vm::Value>>,
}
/// GameWidget.hl/ha are initialized together and retained by the widget instance.
#[derive(Clone, Debug, Default)]
pub struct RemoteMappings {
    pub fonts: Rc<RefCell<HashMap<i32, i32>>>,
    pub endpoints: HashMap<Option<String>, i32>,
}
#[derive(Clone, Debug)]
pub struct Runtime {
    pub shortcuts: crate::widget_shortcuts::Bindings,
    pub remote_mappings: Option<Box<RemoteMappings>>,
    pub remote_loader: Option<Box<crate::remote_widget::Loader>>,
    pub remote_image: Option<crate::remote_request::Request>,
    pub remote_status: i32,
    pub remote_changed: std::cell::Cell<bool>,
    pub instance: u64,
    pub condition: bool,
    pub(crate) aligned: std::cell::Cell<Option<crate::widget_layout::Geometry>>,
    pub cs1_values: Vec<i32>,
    pub listeners: HashMap<u16, (Vec<crate::vm::Value>, Option<Vec<i32>>)>,
    pub params: HashMap<i32, crate::vm::Value>,
    pub drag_parent: Option<u32>,
    pub(crate) drag_parent_reference: Option<(u32, u64)>,
    pub prioritize_menu: bool,
    pub logical_id: u32,
    pub child_index: i32,
    /// Java co: parent slot in the owner's flat dynamic child array.
    pub tree_parent: i32,
    /// Java cc/ck: inclusive child-slot bounds, widened on insertion and never shrunk.
    pub child_bounds: [i32; 2],
    pub scroll: [i32; 2],
    pub item: i32,
    pub count: i32,
    pub quantity_mode: u8,
    pub model: ModelSource,
    pub model_zoom: i32,
    pub rotation_speed: [i16; 2],
    pub frame: usize,
    pub frame_cycle: i32,
    pub player_colors: [u8; 5],
    pub player_appearance: Option<crate::widget_appearance::Composition>,
    pub player_body: u8,
    pub player_item: Option<i32>,
    pub key_modes: Option<Box<crate::widget_key_modes::KeyModes>>,
    pub editor: Option<Box<crate::widget_editor::Editor>>,
}
impl Runtime {
    fn new(w: &Widget, instance: u64) -> Self {
        Self {
            shortcuts: Default::default(),
            remote_mappings: None,
            remote_loader: None,
            remote_image: None,
            remote_status: -1,
            remote_changed: Default::default(),
            instance,
            condition: false,
            // Script queries still read zero before alignment. Standalone definition
            // previews can initialize once without overwriting later script geometry.
            aligned: std::cell::Cell::new(None),
            cs1_values: Vec::new(),
            listeners: HashMap::new(),
            params: HashMap::new(),
            drag_parent: None,
            drag_parent_reference: None,
            prioritize_menu: false,
            logical_id: w.id,
            child_index: -1,
            tree_parent: -1,
            child_bounds: [65535, 0],
            scroll: [0; 2],
            item: -1,
            count: 0,
            quantity_mode: 2,
            model: if w.models[0] == -1 {
                ModelSource::None
            } else {
                ModelSource::Raw(w.models[0])
            },
            model_zoom: w.model_zoom as i32,
            rotation_speed: [0; 2],
            frame: 0,
            frame_cycle: 0,
            player_colors: [0; 5],
            player_appearance: None,
            player_body: 0,
            player_item: None,
            key_modes: (w.kind == 12).then(Default::default),
            editor: (w.kind == 12).then(Default::default),
        }
    }
}
#[derive(Clone, Debug)]
pub struct ScriptEvent {
    pub widget: u32,
    pub context: crate::script_event::Context,
    pub arguments: Vec<crate::vm::Value>,
}
pub type LifecycleObject = Rc<RefCell<LifecycleReference>>;
pub type LifecycleArray = Rc<RefCell<Vec<Option<LifecycleObject>>>>;
#[derive(Clone, Debug)]
pub struct LifecycleReference {
    id: u32,
    instance: u64,
    retired: Option<LifecycleSnapshot>,
}
#[derive(Clone, Debug)]
struct LifecycleSnapshot {
    logical_id: u32,
    child_index: i32,
    kind: u8,
    children: Option<LifecycleArray>,
    listeners: [Option<Vec<crate::vm::Value>>; 2],
}
#[derive(Clone, Debug)]
pub enum PendingEvent {
    Script(ScriptEvent),
    /// An end-of-cycle (Java `ct`-class) editor listener callback, queued only by
    /// `queue_editor_listener` (dd/cx/fe -> GameClient.ct.ag). `instance` is
    /// the producing widget's runtime instance. The live loop does not gate
    /// packets on these entries, and runs one only while that exact instance is
    /// still current (see `deferred_current`).
    Deferred { event: ScriptEvent, instance: u64 },
    OnLoadArray(LifecycleArray, usize),
    CloseModal(VecDeque<u32>),
    MoveSub { source: u32, destination: u32, moving: Option<SubInterface> },
    OpenSub(SubInterface),
    MountSub(SubInterface),
    ResyncChildren { children: VecDeque<SubInterface>, parents: HashSet<u32>, events: Vec<Events> },
    ResyncClose { parents: VecDeque<u32>, events: Vec<Events> },
    Lifecycle(u16),
    LifecycleGroup(u16, u16, usize),
    LifecycleArray(LifecycleArray, usize, u16, usize),
    LifecycleWidget(LifecycleObject, u16, usize),
    LifecycleMount(LifecycleObject, u16, usize),
    LifecycleListener(LifecycleObject, u16),
    /// Live-drain generation boundary (live_runtime::drain_interface_events).
    /// Present only while one live drain runs; never left in the queue.
    Boundary,
}
impl PendingEvent {
    /// Entries that run at the end of a cycle rather than with the packet that
    /// queued them (GameClient.aft drains ct after the packet batch).
    pub fn is_end_of_cycle(&self) -> bool {
        matches!(self, PendingEvent::Deferred { .. })
    }
    /// Rebuilds a dequeued event with its class: `Some(instance)` for an
    /// end-of-cycle callback, `None` for a packet-synchronous one.
    pub fn requeued(event: ScriptEvent, instance: Option<u64>) -> Self {
        match instance {
            Some(instance) => PendingEvent::Deferred { event, instance },
            None => PendingEvent::Script(event),
        }
    }
}
#[derive(Clone, Debug)]
pub struct EditorPointer {
    pub widget: u32,
    pub instance: u64,
    pub press: bool,
    pub point: [i32;2],
}

#[derive(Default)]
pub struct Interfaces {
    layout_cache: RefCell<Option<crate::layout_cache::Cache>>,
    pub resize_events: RefCell<VecDeque<crate::widget_resize::Queued>>,
    pub(crate) geometry_canvas: std::cell::Cell<[i32; 2]>,
    pub(crate) selected_widgets: [Option<crate::widget_selection::Object>;2],
    pub(crate) held_widgets: HashMap<u64, crate::widget_selection::WeakObject>,
    pub closing_modal: bool,
    pub widget_query: crate::widget_query::Query,
    // Weak registries share identities only while a suspended traversal retains them.
    lifecycle_objects: HashMap<u64, Weak<RefCell<LifecycleReference>>>,
    drag_objects: HashMap<u64, Weak<RefCell<DragReference>>>,
    lifecycle_arrays: HashMap<u32, Weak<RefCell<Vec<Option<LifecycleObject>>>>>,
    lifecycle_groups: HashMap<u16, Weak<RefCell<Vec<Option<LifecycleObject>>>>>,
    child_array_lengths: HashMap<u32, usize>,
    static_group_lengths: HashMap<u16, usize>,
    pub shortcut_scope: crate::widget_scope::Scope,
    pub tooltip: crate::widget_tooltip::State,
    next_instance: u64,
    legacy_hover: Option<(u32, u64)>,
    editor_owner: Option<(u32, u64)>,
    editor_modifiers: [bool;2],
    pub local_player: Option<crate::player_model::Appearance>,
    pub(crate) appearance_kits: Option<HashMap<u32, crate::identity_kit::Kit>>,
    pub drag_visual: Option<(u32, [i32; 2], [i32; 4])>,
    pub continue_widget: Option<u32>,
    pub(crate) continue_instance: Option<(u32, u64)>,
    parameter_definitions: HashMap<i32, crate::param_def::Definition>,
    pub root: Option<u16>,
    pub children: crate::interface_mounts::Mounts,
    pub widgets: HashMap<u32, Widget>,
    pub runtime: HashMap<u32, Runtime>,
    pub events: HashMap<(u32, i32), [u32; 2]>,
    pub scripts: VecDeque<PendingEvent>,
    pub dynamic: HashMap<(u32, i32), u32>,
    next_dynamic: u32,
    loaded: HashSet<u16>,
}
impl Interfaces {
    pub(crate) fn static_group_length(&self, group: u16) -> usize {
        self.static_group_lengths.get(&group).copied().unwrap_or(0)
    }
    pub fn child_array_length(&self, owner: u32) -> usize {
        self.child_array_lengths.get(&owner).copied().unwrap_or(0)
    }

    pub fn editor_pointer_events(&self, layout: &Layout, input: &crate::input::Input, mouse_camera: bool) -> Vec<EditorPointer> {
        self.widget_pointer_events(layout,input,mouse_camera).into_iter().filter(|event|self.widgets[&event.widget].kind==12).collect()
    }
    pub(crate) fn widget_pointer_events(&self, layout: &Layout, input: &crate::input::Input, mouse_camera: bool) -> Vec<EditorPointer> {
        let buttons=if mouse_camera {1}else{5};let mut events=Vec::new();
        for &id in &layout.order {
            let w=&self.widgets[&id];let r=&self.runtime[&id];let p=&layout.widgets[&id];
            if !w.if3 || p.hidden {continue;}
            let interactive=matches!(w.kind,0|11|12) || w.content==1338 || w.listeners.iter().any(Option::is_some) || !r.listeners.is_empty() || self.interaction_flags(id)!=0 || self.operation_mask(id)!=0;
            if !interactive {continue;}
            let inside=|point:[i32;2]|point[0]>=p.clip[0]&&point[0]<p.clip[2]&&point[1]>=p.clip[1]&&point[1]<p.clip[3];
            let over=inside(input.pointer);
            if w.no_click_through && over {events.clear();}
            if !matches!(w.kind,11|12) {continue;}
            let point=[input.pointer[0]-p.absolute[0],input.pointer[1]-p.absolute[1]];
            if input.pressed_buttons&buttons!=0 && inside(input.press_position) {events.push(EditorPointer{widget:id,instance:r.instance,press:true,point});}
            if input.buttons&buttons!=0 && over {events.push(EditorPointer{widget:id,instance:r.instance,press:false,point});}
        }
        events
    }
    pub(crate) fn dispatch_editor_pointer_one(&mut self,event:EditorPointer,input:&crate::input::Input)->Result<()> {
        let id=event.widget;
        if !self.runtime.get(&id).is_some_and(|r|r.instance==event.instance&&r.editor.as_ref().is_some_and(|e|e.text.enabled)) {return Ok(());}
        if event.press {self.focus_editor(Some(id));}
        let editor=self.runtime.get_mut(&id).unwrap().editor.as_mut().unwrap();
        if event.press {editor.mouse_press(event.point,input.held.contains(&82),input.held.contains(&81))?;}
        else {editor.mouse_drag(event.point)?;}
        self.flush_editor_events(id);
        Ok(())
    }
    pub fn dispatch_editor_pointer(&mut self, events: Vec<EditorPointer>, input: &crate::input::Input) -> Result<()> {
        let saw_editor=!events.is_empty();
        for event in events {self.dispatch_editor_pointer_one(event,input)?;}
        if !saw_editor && input.pressed_buttons&1!=0 {self.blur_editor();}
        Ok(())
    }

    pub fn editor_modifiers(&self) -> [bool;2] {self.editor_modifiers}
    pub fn editor_key(&mut self, key: i32, down: bool) -> Result<bool> {
        self.editor_key_with_clipboard(key,down,|_|None,|_|false)
    }
    pub fn editor_key_with_clipboard(&mut self, key: i32, down: bool, mut read: impl FnMut(usize)->Option<Vec<u16>>, mut write: impl FnMut(&[u16])->bool) -> Result<bool> {
        if !down {
            if key==81 {self.editor_modifiers[0]=false;}
            if key==82 {self.editor_modifiers[1]=false;}
            return Ok(false);
        }
        let Some(id)=self.focused_editor() else {return Ok(false);};
        let Some(modes)=self.runtime[&id].key_modes.as_ref() else {return Ok(false);};
        let mode=modes.key(key);
        if crate::widget_key_modes::KeyModes::handles(mode) {
            if key==81 {self.editor_modifiers[0]=true;}
            else if key==82 {self.editor_modifiers[1]=true;}
            else if self.runtime[&id].editor.as_ref().is_some_and(|e|e.text.ready) {
                let [shift,control]=self.editor_modifiers;
                if (65..=67).contains(&key) && control {
                    let editor=self.runtime.get_mut(&id).unwrap().editor.as_mut().unwrap();
                    if key==67 {
                        if let Some(text)=read(1<<20) {let _=editor.paste(&text);}
                    } else {
                        let selected=editor.text.selected_text()?;
                        if !selected.is_empty() && write(selected) && key==65 {editor.navigation_key(85,false,false)?;}
                    }
                }
                else if key==13 {self.blur_editor();}
                else if key==84 {
                    let editor=self.runtime[&id].editor.as_ref().unwrap();
                    if editor.enter_mode==0 || (shift && editor.options.max_lines>1) {
                        self.runtime.get_mut(&id).unwrap().editor.as_mut().unwrap().insert(10)?;
                    }else{self.queue_editor_listener(id,1438);self.blur_editor();}
                }else{self.runtime.get_mut(&id).unwrap().editor.as_mut().unwrap().navigation_key(key,shift,control)?;}
                self.flush_editor_events(id);
            }
        }
        Ok(crate::widget_key_modes::KeyModes::consumes(mode))
    }

    pub fn editor_character(&mut self, ch: u16) -> Result<bool> {
        let Some(id)=self.focused_editor() else {return Ok(false);};
        let runtime=self.runtime.get_mut(&id).unwrap();
        let Some(editor)=runtime.editor.as_mut().filter(|e|e.text.ready) else {return Ok(false);};
        let Some(modes)=runtime.key_modes.as_ref() else {return Ok(false);};
        // Characters outside the 192-entry policy cannot be handled or consumed.
        let valid=(32..127).contains(&ch)||(128..160).contains(&ch)||(161..=255).contains(&ch);
        if !valid {return Ok(false);}
        let mode=modes.character(ch as i32);
        if crate::widget_key_modes::KeyModes::handles(mode) {editor.insert(ch)?;}
        self.flush_editor_events(id);
        Ok(crate::widget_key_modes::KeyModes::consumes(mode))
    }

    /// Current editor owner, guarded against replacement of a widget instance.
    pub fn focused_editor(&self) -> Option<u32> {
        self.editor_owner.and_then(|(id, instance)| {
            self.runtime.get(&id).filter(|r| r.instance == instance).map(|_| id)
        })
    }
    pub(crate) fn flush_editor_events(&mut self, id: u32) {
        let events = self.runtime.get_mut(&id).and_then(|r| r.editor.as_mut())
            .map(|editor| editor.text.take_events()).unwrap_or_default();
        for opcode in events { self.queue_editor_listener(id, opcode); }
    }
    fn queue_editor_listener(&mut self, id: u32, opcode: u16) {
        if let Some((arguments, _)) = self.listener(id, opcode) {
            let r = &self.runtime[&id];
            self.scripts.push_back(PendingEvent::Deferred { event: ScriptEvent {
                widget: id,
                context: crate::script_event::Context {
                    widget: Some((r.logical_id, r.child_index)),
                    ..Default::default()
                },
                arguments,
            }, instance: r.instance });
        }
    }
    pub fn blur_editor(&mut self) {
        let owner = self.focused_editor();
        self.editor_owner = None;
        let Some(id) = owner else { return; };
        if self.runtime[&id].editor.as_ref().is_some_and(|e| e.text.focused) {
            self.queue_editor_listener(id, 1436);
        }
        if let Some(editor) = &mut self.runtime.get_mut(&id).unwrap().editor {
            editor.text.set_focus(false);
        }
        self.flush_editor_events(id);
    }
    /// Native focus attempts deliver the gain callback even for disabled editors.
    pub fn focus_editor(&mut self, id: Option<u32>) {
        if self.focused_editor() != id { self.blur_editor(); }
        let Some(id) = id else { return; };
        let Some(runtime) = self.runtime.get(&id) else { return; };
        self.editor_owner = Some((id, runtime.instance));
        if runtime.editor.as_ref().is_some_and(|e| !e.text.focused) {
            self.queue_editor_listener(id, 1436);
        }
        if let Some(editor) = &mut self.runtime.get_mut(&id).unwrap().editor {
            editor.text.set_focus(true);
        }
        self.flush_editor_events(id);
    }
    /// Opcode 1140 first releases the existing owner, including on a false request.
    pub fn set_editor_focus(&mut self, id: u32, value: bool) {
        self.blur_editor();
        let Some(editor) = self.runtime.get_mut(&id).and_then(|r| r.editor.as_mut()) else { return; };
        let previous = editor.text.focused;
        editor.text.set_focus(value);
        let changed = previous != editor.text.focused;
        self.flush_editor_events(id);
        if value && changed { self.focus_editor(Some(id)); }
    }
    pub fn set_editor_enabled(&mut self, id: u32, value: bool) {
        if !value && self.focused_editor() == Some(id) { self.blur_editor(); }
        if let Some(editor) = self.runtime.get_mut(&id).and_then(|r| r.editor.as_mut()) {
            editor.text.set_enabled(value);
        }
    }

    pub fn prepare_editors(&mut self,cache:&mut DiskCache,layout:&Layout)->Result<()> {
        for &id in &layout.order {
            let w=self.widgets.get_mut(&id).unwrap();let r=self.runtime.get_mut(&id).unwrap();let Some(editor)=&mut r.editor else {continue;};
            if w.hidden {continue;}
            let font=if !editor.text.ready&&w.font>=0 {Some((w.font,crate::font::Font::load(cache,w.font as u32)?.metrics))}else{None};
            editor.prepare(&mut w.text,layout.widgets[&id].geometry.size,font)?;
            self.flush_editor_events(id);
        }
        Ok(())
    }

    pub fn legacy_hovered(&self, id: u32) -> bool {
        self.runtime.get(&id).is_some_and(|r| self.legacy_hover == Some((id, r.instance)))
    }
    pub fn tick_tooltip(&mut self, layout: &Layout, pointer: [i32; 2], suppress: bool) -> bool {
        let previous_hover = self.legacy_hover;
        let mut hovered = None;
        if !suppress {
            let mut hidden = HashSet::new();
            for &id in &layout.order {
                let w = &self.widgets[&id];
                let p = &layout.widgets[&id];
                if p.parent.is_some_and(|parent| hidden.contains(&parent))
                    || (w.hidden && (w.if3 || (w.kind == 0 && !self.legacy_hovered(id)))) {
                    hidden.insert(id);
                    continue;
                }
                if w.if3 || pointer[0] < p.clip[0] || pointer[0] >= p.clip[2]
                    || pointer[1] < p.clip[1] || pointer[1] >= p.clip[3] { continue; }
                if w.mouse_over >= 0 || w.colors[2] != 0 {
                    let target = if w.mouse_over < 0 { Some(id) }
                        else if self.runtime[&id].child_index >= 0 {
                            w.parent.and_then(|parent| self.dynamic.get(&(parent, w.mouse_over as i32)).copied())
                        } else { Some((self.runtime[&id].logical_id & 0xffff0000) | w.mouse_over as u32) };
                    self.legacy_hover = target.and_then(|id| self.runtime.get(&id).map(|r| (id, r.instance)));
                }
                if w.kind == 8 { hovered = Some((id, self.runtime[&id].instance)); }
            }
        }
        self.tooltip.tick(hovered);
        self.legacy_hover != previous_hover
    }
    pub(crate) fn ensure_appearance_kits(&mut self, cache: &mut DiskCache) -> Result<()> {
        if self.appearance_kits.is_none() { self.appearance_kits = Some(crate::identity_kit::Kit::load_all(cache)?); }
        Ok(())
    }

    pub fn retain_drag_object(&mut self, id: u32) -> Option<DragObject> {
        let instance = self.runtime.get(&id)?.instance;
        self.drag_objects.retain(|_, object| object.strong_count() != 0);
        if let Some(object) = self.drag_objects.get(&instance).and_then(Weak::upgrade) {
            return Some(object);
        }
        let object = Rc::new(RefCell::new(DragReference { id, instance, retired: None }));
        self.drag_objects.insert(instance, Rc::downgrade(&object));
        Some(object)
    }
    pub fn drag_snapshot(&self, object: &DragObject) -> Option<DragSnapshot> {
        let reference = object.borrow();
        if let Some(snapshot) = &reference.retired { return Some(snapshot.clone()); }
        let runtime = self.runtime.get(&reference.id)?;
        if runtime.instance != reference.instance { return None; }
        let widget = self.widgets.get(&reference.id)?;
        Some(DragSnapshot {
            identity: (runtime.logical_id, runtime.child_index, runtime.item),
            parent: if runtime.child_index >= 0 { Some(runtime.logical_id) } else { widget.parent },
            flags: widget.flags,
            complete_listener: self.listener(reference.id, 1410).map(|v| v.0),
        })
    }
    pub fn retained_drag_ancestor(&self, source: &DragSnapshot) -> Option<u32> {
        let flags = self.events.get(&(source.identity.0, source.identity.1))
            .map_or(source.flags, |m| m[0]);
        let depth = (flags >> 17) & 7;
        if depth == 0 { return None; }
        let mut parent = source.parent?;
        for step in 0..depth {
            if !self.widgets.contains_key(&parent) { return None; }
            if step + 1 < depth { parent = self.script_parent_id(parent)?; }
        }
        Some(parent)
    }
    fn retire_drag_object(&mut self, id: u32) {
        let Some(instance) = self.runtime.get(&id).map(|r| r.instance) else { return; };
        let Some(object) = self.drag_objects.remove(&instance).and_then(|r| r.upgrade()) else { return; };
        let snapshot = self.drag_snapshot(&object);
        object.borrow_mut().retired = snapshot;
    }
    pub fn drag_ancestor(&self, id: u32) -> Option<u32> {
        let depth = (self.interaction_flags(id) >> 17) & 7;
        if depth == 0 {
            return None;
        }
        let mut parent = id;
        for _ in 0..depth {
            parent = self.script_parent_id(parent)?;
            if !self.widgets.contains_key(&parent) {
                return None;
            }
        }
        Some(parent)
    }
    pub fn drag_parent(&self, id: u32) -> Option<u32> {
        self.drag_parent_identity(id).map(|(parent, _)| parent)
    }
    pub fn interaction_flags(&self, id: u32) -> u32 {
        let Some(runtime) = self.runtime.get(&id) else {
            return 0;
        };
        self.events
            .get(&(runtime.logical_id, runtime.child_index))
            .map_or_else(|| self.widgets.get(&id).map_or(0, |w| w.flags), |m| m[0])
    }
    pub fn operation_mask(&self, id: u32) -> u32 {
        let Some(r) = self.runtime.get(&id) else {
            return 0;
        };
        self.events.get(&(r.logical_id, r.child_index)).map_or_else(
            || self.widgets.get(&id).map_or(0, |w| (w.flags >> 1) & 1023),
            |m| m[1],
        )
    }
    pub fn listener(
        &self,
        id: u32,
        opcode: u16,
    ) -> Option<(Vec<crate::vm::Value>, Option<Vec<i32>>)> {
        let runtime = self.runtime.get(&id)?;
        if let Some(listener) = runtime.listeners.get(&opcode) {
            return (!listener.0.is_empty()).then(|| listener.clone());
        }
        let index = match opcode {
            1403 => 1,
            1404 => 2,
            1406 => 3,
            1416 => 4,
            1407 => 5,
            1414 => 6,
            1415 => 7,
            1408 => 8,
            1409 => 9,
            1412 => 10,
            1400 => 11,
            1411 => 12,
            1402 => 13,
            1401 => 14,
            1405 => 15,
            1410 => 16,
            1417 => 17,
            _ => return None,
        };
        let widget = self.widgets.get(&id)?;
        let args = widget.listeners[index].as_ref()?;
        let values = args
            .iter()
            .map(|a| match a {
                Argument::Int(v) => crate::vm::Value::Int(*v),
                Argument::Text(v) => crate::vm::Value::Object(crate::vm::Object::String(v.clone())),
                Argument::Null => crate::vm::Value::Object(crate::vm::Object::Null),
            })
            .collect();
        let triggers = match opcode {
            1407 => widget.triggers[0].clone(),
            1414 => widget.triggers[1].clone(),
            1415 => widget.triggers[2].clone(),
            _ => None,
        };
        Some((values, triggers))
    }
    pub fn install_group(&mut self, group: u16, widgets: Vec<Widget>) -> Result<()> {
        if self.loaded.contains(&group) {
            return Err(Error::Invalid("interface group already loaded"));
        }
        let mut ids = HashSet::new();
        if widgets
            .iter()
            .any(|w| w.id >> 16 != group as u32 || !ids.insert(w.id))
        {
            return Err(Error::Invalid("interface group component ids"));
        }
        self.static_group_lengths.insert(group, widgets.iter().map(|w| (w.id & 65535) as usize + 1).max().unwrap_or(0));
        let static_parents=widgets.iter().filter_map(|w|w.parent.filter(|parent|parent>>16==group as u32).map(|parent|(parent,(w.id&65535) as i32))).collect::<Vec<_>>();
        for w in widgets {
            self.next_instance += 1;
            self.runtime.insert(w.id, Runtime::new(&w, self.next_instance));
            self.widgets.insert(w.id, w);
        }
        for (parent,index) in static_parents {
            if let Some(runtime)=self.runtime.get_mut(&parent) {
                runtime.child_bounds[0]=runtime.child_bounds[0].min(index);
                runtime.child_bounds[1]=runtime.child_bounds[1].max(index);
            }
        }
        self.loaded.insert(group);
        Ok(())
    }
    pub fn parameter_default(
        &mut self,
        cache: &mut DiskCache,
        id: i32,
    ) -> Result<crate::vm::Value> {
        if !self.parameter_definitions.contains_key(&id) {
            let bytes = if id < 0 {
                vec![0]
            } else {
                cache.file(2, 11, id as u32, [0; 4])?
            };
            self.parameter_definitions
                .insert(id, crate::param_def::Definition::decode(&bytes)?);
        }
        Ok(self.parameter_definitions[&id].default.clone())
    }
    /// Java cb is the array owner for dynamic widgets and the parent ID for static ones.
    pub fn script_parent_id(&self, id: u32) -> Option<u32> {
        let r = self.runtime.get(&id)?;
        if r.child_index >= 0 { Some(r.logical_id) } else { self.widgets.get(&id)?.parent }
    }
    pub(crate) fn child_array_allocated(&self,owner:u32)->bool {
        self.child_array_lengths.get(&owner).is_some_and(|&length|length>0)
    }
    /// ll.cq distinguishes a null array entry from an out-of-bounds lookup.
    pub(crate) fn checked_script_child(&self, owner: u32, index: i32) -> Result<Option<u32>> {
        let length = self.static_group_lengths.get(&((owner >> 16) as u16)).copied().unwrap_or(0);
        if (owner & 65535) as usize >= length {
            return Err(Error::Invalid("widget component index"));
        }
        if index < -1 && self.child_array_allocated(owner) {
            return Err(Error::Invalid("widget child index"));
        }
        Ok(self.script_child(owner, index))
    }
    pub fn script_child(&self, owner: u32, index: i32) -> Option<u32> {
        if index == -1 {
            self.widgets.contains_key(&owner).then_some(owner)
        } else {
            self.dynamic.get(&(owner, index)).copied()
        }
    }
    pub fn create_child(&mut self, parent: u32, index: i32, kind: u8, strict: bool) -> Result<u32> {
        self.create_child_inner(parent,index,kind,strict,false)
    }
    pub(crate) fn begin_remote_children(&mut self,parent:u32,count:usize)->Result<()> {
        if self.widgets.get(&parent).is_none_or(|w|w.kind!=11) {return Err(Error::Invalid("remote child parent"))}
        self.widgets.get_mut(&parent).unwrap().no_click_through=true;
        self.delete_children(parent);
        self.child_array_lengths.insert(parent,count);
        Ok(())
    }
    pub(crate) fn create_remote_child(&mut self,parent:u32,index:i32,kind:u8)->Result<u32> {
        self.create_child_inner(parent,index,kind,true,true)
    }
    fn create_child_inner(&mut self, parent: u32, index: i32, kind: u8, strict: bool, remote:bool) -> Result<u32> {
        if index < 0 || index > 65535 {
            return Err(Error::Invalid("dynamic widget index"));
        }
        if self.widgets.get(&parent).is_none_or(|w| if remote {w.kind!=11} else {w.kind!=0}) {
            return Err(Error::Invalid("dynamic widget parent"));
        }
        if let Some(&old) = self.dynamic.get(&(parent, index)) {
            if strict {
                return Err(Error::Invalid("dynamic widget already exists"));
            }
            self.delete_child(old)?;
        }
        let id = 0xfffe0000u32
            .checked_add(self.next_dynamic)
            .filter(|&id| id < u32::MAX)
            .ok_or(Error::Invalid("dynamic widget capacity"))?;
        self.next_dynamic += 1;
        let mut w = Widget::empty(id, kind);
        w.parent = Some(parent);
        self.next_instance += 1;
        let mut runtime = Runtime::new(&w, self.next_instance);
        runtime.logical_id = self.runtime[&parent].logical_id;
        runtime.child_index = index;
        self.widgets.insert(id, w);
        self.runtime.insert(id, runtime);
        self.dynamic.insert((parent, index), id);
        let len = self.child_array_lengths.entry(parent).or_default();
        if index as usize >= *len {
            // Java allocates a new array when the requested slot exceeds its length.
            *len = index as usize + 1;
            self.lifecycle_arrays.remove(&parent);
        } else if let Some(array) = self.lifecycle_arrays.get(&parent).and_then(Weak::upgrade) {
            array.borrow_mut()[index as usize] = Some(self.lifecycle_object(id));
        }
        Ok(id)
    }
    pub(crate) fn delete_script_widget(&mut self,owner:u32,selected:crate::widget_delete::Node)->Result<()> {
        struct Slots<'a>{state:&'a mut Interfaces,owner:u32}
        impl crate::widget_delete::Slots for Slots<'_> {
            fn length(&self)->Option<usize>{self.state.child_array_lengths.get(&self.owner).copied()}
            fn node(&self,index:i32)->Option<crate::widget_delete::Node>{
                let id=self.state.dynamic.get(&(self.owner,index))?;
                let runtime=self.state.runtime.get(id)?;let widget=self.state.widgets.get(id)?;
                Some(crate::widget_delete::Node{index,parent:runtime.tree_parent,kind:widget.kind,bounds:runtime.child_bounds})
            }
            fn clear(&mut self,index:i32){
                let Some(id)=self.state.dynamic.get(&(self.owner,index)).copied() else{return;};
                if let Some(array)=self.state.lifecycle_arrays.get(&self.owner).and_then(Weak::upgrade) {
                    array.borrow_mut()[index as usize]=None;
                }
                self.state.retire_lifecycle_object(id);
                self.state.dynamic.remove(&(self.owner,index));
                self.state.lifecycle_arrays.remove(&id);self.state.child_array_lengths.remove(&id);
                self.state.widgets.remove(&id);self.state.runtime.remove(&id);
            }
        }
        // Mutate as slots are visited: an out-of-bounds error preserves earlier removals.
        crate::widget_delete::remove(selected,&mut Slots{state:self,owner})
    }
    pub fn delete_child(&mut self, id: u32) -> Result<()> {
        if self.runtime.get(&id).is_none_or(|r| r.child_index < 0) {
            return Err(Error::Invalid("delete static widget"));
        }
        if let Some((parent, index)) = self.dynamic.iter().find_map(|(&(p, i), &child)| (child == id).then_some((p, i))) {
            if let Some(array) = self.lifecycle_arrays.get(&parent).and_then(Weak::upgrade) {
                array.borrow_mut()[index as usize] = None;
            }
        }
        self.retire_lifecycle_object(id);
        self.delete_children(id);
        self.dynamic.retain(|_, v| *v != id);
        self.widgets.remove(&id);
        self.runtime.remove(&id);
        Ok(())
    }
    pub fn delete_children(&mut self, parent: u32) {
        // Clearing children replaces the array; active traversals retain its old slots.
        self.lifecycle_arrays.remove(&parent);
        self.child_array_lengths.remove(&parent);
        let ids = self
            .dynamic
            .iter()
            .filter(|((p, _), id)| *p == parent || self.widgets.get(id).is_some_and(|w| w.parent == Some(parent)))
            .map(|(_, &id)| id)
            .collect::<Vec<_>>();
        for id in ids {
            let _ = self.delete_child(id);
        }
    }
    pub fn load_group(&mut self, cache: &mut DiskCache, group: u16) -> Result<()> {
        if self.loaded.contains(&group) {
            return Ok(());
        }
        let widgets = Widget::load_group(cache, group as u32)?;
        self.install_group(group, widgets)
    }
    fn on_load(&mut self, group: u16) {
        // ps.ab resets every static animation clock before ps.av runs any callback.
        for runtime in self.runtime.values_mut().filter(|r| r.child_index == -1 && r.logical_id >> 16 == group as u32) {
            runtime.frame = 0;
            runtime.frame_cycle = 0;
        }
        let array = self.lifecycle_group(group);
        self.scripts.push_back(PendingEvent::OnLoadArray(array, 0));
    }
    fn event_masks(&mut self, e: Events) -> Result<()> {
        if e.first < -1 || e.last > 65534 {
            return Err(Error::Invalid("interface event range"));
        }
        for child in e.first..=e.last {
            self.events.insert((e.widget, child), e.masks);
        }
        Ok(())
    }
    pub fn masks(&self, id: u32, child: i32) -> [u32; 2] {
        let logical = self.runtime.get(&id).map_or(id, |r| r.logical_id);
        self.events
            .get(&(logical, child))
            .copied()
            .unwrap_or([self.widgets.get(&id).map_or(0, |w| w.flags), 0])
    }
    pub fn lifecycle_events(&self, opcode: u16) -> Result<Vec<ScriptEvent>> {
        if !matches!(opcode, 1423 | 1424) {
            return Err(Error::Invalid("interface lifecycle reason"));
        }
        let mut events = Vec::new();
        if let Some(group) = self.root {
            self.visit_lifecycle_group(group, opcode, &mut events, 0)?;
        }
        Ok(events)
    }
    fn visit_lifecycle_group(
        &self,
        group: u16,
        opcode: u16,
        events: &mut Vec<ScriptEvent>,
        depth: usize,
    ) -> Result<()> {
        let mut ids = self
            .runtime
            .iter()
            .filter(|(_, r)| r.child_index == -1 && r.logical_id >> 16 == group as u32)
            .map(|(&id, _)| id)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        for id in ids {
            self.visit_lifecycle_widget(id, opcode, events, depth + 1)?;
        }
        Ok(())
    }
    fn visit_lifecycle_widget(
        &self,
        id: u32,
        opcode: u16,
        events: &mut Vec<ScriptEvent>,
        depth: usize,
    ) -> Result<()> {
        if depth > 128 {
            return Err(Error::Invalid("interface lifecycle hierarchy depth"));
        }
        let Some(w) = self.widgets.get(&id) else {
            return Ok(());
        };
        let r = &self.runtime[&id];
        if w.kind == 0 {
            let mut children = self
                .dynamic
                .iter()
                .filter(|((parent, _), _)| *parent == id)
                .map(|((_, index), &child)| (*index, child))
                .collect::<Vec<_>>();
            children.sort_unstable();
            for (_, child) in children {
                self.visit_lifecycle_widget(child, opcode, events, depth + 1)?;
            }
            if r.child_index == -1 {
                if let Some(sub) = self.children.get(&id) {
                    self.visit_lifecycle_group(sub.group, opcode, events, depth + 1)?;
                }
            }
        }
        if opcode == 1424
            && r.child_index >= 0
            && self.dynamic.get(&(r.logical_id, r.child_index)) != Some(&id)
        {
            return Ok(());
        }
        if let Some((arguments, _)) = self.listener(id, opcode) {
            events.push(ScriptEvent {
                widget: id,
                arguments,
                context: crate::script_event::Context {
                    widget: Some((r.logical_id, r.child_index)),
                    ..Default::default()
                },
            });
        }
        Ok(())
    }
    pub fn queue_lifecycle(&mut self, opcode: u16) -> Result<()> {
        if !matches!(opcode, 1423 | 1424) {
            return Err(Error::Invalid("interface lifecycle reason"));
        }
        self.scripts.push_back(PendingEvent::Lifecycle(opcode));
        Ok(())
    }
    fn lifecycle_object(&mut self, id: u32) -> LifecycleObject {
        let instance = self.runtime[&id].instance;
        if let Some(object) = self.lifecycle_objects.get(&instance).and_then(Weak::upgrade) {
            return object;
        }
        let object = Rc::new(RefCell::new(LifecycleReference { id, instance, retired: None }));
        self.lifecycle_objects.insert(instance, Rc::downgrade(&object));
        object
    }
    fn lifecycle_group(&mut self, group: u16) -> LifecycleArray {
        if let Some(array) = self.lifecycle_groups.get(&group).and_then(Weak::upgrade) {
            return array;
        }
        let mut ids = self.runtime.iter()
            .filter(|(_, r)| r.child_index == -1 && r.logical_id >> 16 == group as u32)
            .map(|(&id, _)| id).collect::<Vec<_>>();
        ids.sort_unstable();
        let array = Rc::new(RefCell::new(ids.into_iter()
            .map(|id| Some(self.lifecycle_object(id))).collect()));
        self.lifecycle_groups.insert(group, Rc::downgrade(&array));
        array
    }
    fn lifecycle_children(&mut self, id: u32) -> Option<LifecycleArray> {
        if let Some(array) = self.lifecycle_arrays.get(&id).and_then(Weak::upgrade) {
            return Some(array);
        }
        let children = self.dynamic.iter().filter(|((parent, _), _)| *parent == id)
            .map(|((_, index), &child)| (*index as usize, child)).collect::<Vec<_>>();
        let len = *self.child_array_lengths.entry(id).or_insert_with(|| {
            children.iter().map(|(index, _)| index + 1).max().unwrap_or(0)
        });
        if len == 0 { return None; }
        let mut slots = vec![None; len];
        for (index, child) in children { slots[index] = Some(self.lifecycle_object(child)); }
        let array = Rc::new(RefCell::new(slots));
        self.lifecycle_arrays.insert(id, Rc::downgrade(&array));
        Some(array)
    }
    fn lifecycle_snapshot(&mut self, object: &LifecycleObject) -> Option<LifecycleSnapshot> {
        let reference = object.borrow();
        if let Some(snapshot) = &reference.retired { return Some(snapshot.clone()); }
        let id = reference.id;
        let runtime = self.runtime.get(&id)?;
        if runtime.instance != reference.instance { return None; }
        let (logical_id, child_index) = (runtime.logical_id, runtime.child_index);
        let kind = self.widgets.get(&id)?.kind;
        let listeners = [1423, 1424].map(|opcode| self.listener(id, opcode).map(|v| v.0));
        drop(reference);
        Some(LifecycleSnapshot { logical_id, child_index, kind, listeners,
            children: if kind == 0 { self.lifecycle_children(id) } else { None } })
    }
    fn retire_lifecycle_object(&mut self, id: u32) {
        self.retire_selected_widget(id);
        self.retire_drag_object(id);
        let Some(instance) = self.runtime.get(&id).map(|r| r.instance) else { return; };
        let Some(object) = self.lifecycle_objects.get(&instance).and_then(Weak::upgrade) else { return; };
        if object.borrow().retired.is_some() { return; }
        let Some(snapshot) = self.lifecycle_snapshot(&object) else { return; };
        let children = snapshot.children.as_ref().map(|a| a.borrow().iter().flatten()
            .map(|child| child.borrow().id).collect::<Vec<_>>()).unwrap_or_default();
        object.borrow_mut().retired = Some(snapshot);
        for child in children { self.retire_lifecycle_object(child); }
    }
    /// Drain callbacks that do not need to load another interface group.
    /// Packet update consumers should use next_event_with_cache instead.
    pub fn next_event(&mut self) -> Result<Option<ScriptEvent>> {
        Ok(self.next_event_using(None)?.map(|(event, _)| event))
    }
    pub fn next_event_with_cache(&mut self, cache: &mut DiskCache) -> Result<Option<ScriptEvent>> {
        Ok(self.next_event_using(Some(cache))?.map(|(event, _)| event))
    }
    /// As `next_event_with_cache`, also returning the producing instance of an
    /// end-of-cycle callback (`None` for packet-synchronous events). Order is
    /// the same FIFO order.
    pub fn next_event_tagged_with_cache(&mut self, cache: &mut DiskCache) -> Result<Option<(ScriptEvent, Option<u64>)>> {
        self.next_event_using(Some(cache))
    }
    /// Whether an end-of-cycle callback may still run: its widget id still
    /// holds the producing instance and, for a dynamic child, that widget is
    /// still registered at its child index (ps.ac's attached-child check).
    /// This is a bounded fail-closed usability decision, not Java parity: Java
    /// runs stale static callbacks, and their by-id ops resolve to a
    /// replacement sharing the id.
    /// The slot is found under whatever parent `create_child_inner` keyed it by
    /// (a static owner for cc_create/106/107, the kind-11 parent for remote
    /// children), never the inherited `logical_id`. Every removal from
    /// `dynamic` (delete_child, delete_script_widget, group unload) also
    /// removes that widget's runtime, and dynamic ids are never reused, so
    /// the instance check already implies the slot check; the slot check is a
    /// defensive restatement of ps.ac.
    pub fn deferred_current(&self, event: &ScriptEvent, instance: u64) -> bool {
        if !self.runtime.get(&event.widget).is_some_and(|r| r.instance == instance) { return false; }
        match event.context.widget {
            Some((_, child)) if child >= 0 => self.dynamic.iter().any(|(&(_, index), &id)| id == event.widget && index == child),
            _ => true,
        }
    }
    fn next_event_using(&mut self, mut cache: Option<&mut DiskCache>) -> Result<Option<(ScriptEvent, Option<u64>)>> {
        while let Some(event) = self.scripts.pop_front() {
            if let PendingEvent::MountSub(sub) = &event {
                if cache.is_none() && !self.loaded.contains(&sub.group) {
                    self.scripts.push_front(event);
                    return Err(Error::Invalid("pending interface load requires cache"));
                }
            }
            match event {
                PendingEvent::Script(event) => return Ok(Some((event, None))),
                PendingEvent::Deferred { event, instance } => return Ok(Some((event, Some(instance)))),
                // Entries behind the boundary were appended during this live
                // generation; leave them, and the boundary, for the caller.
                PendingEvent::Boundary => {
                    self.scripts.push_front(PendingEvent::Boundary);
                    return Ok(None);
                }
                PendingEvent::OnLoadArray(array, index) => {
                    let slot = array.borrow().get(index).cloned();
                    let Some(slot) = slot else { continue; };
                    let object = slot.ok_or(Error::Invalid("unloaded onLoad array slot"))?;
                    let (id, instance) = { let object = object.borrow(); (object.id, object.instance) };
                    let runtime = self.runtime.get(&id).filter(|r| r.instance == instance)
                        .ok_or(Error::Invalid("retired onLoad widget"))?;
                    let context = crate::script_event::Context {
                        widget: Some((runtime.logical_id, runtime.child_index)), ..Default::default()
                    };
                    let arguments = self.widgets[&id].listeners[0].as_ref().map(|args| args.iter().map(|arg| match arg {
                        Argument::Int(value) => crate::vm::Value::Int(*value),
                        Argument::Text(value) => crate::vm::Value::Object(crate::vm::Object::String(value.clone())),
                        Argument::Null => crate::vm::Value::Object(crate::vm::Object::Null),
                    }).collect());
                    self.scripts.push_front(PendingEvent::OnLoadArray(array, index + 1));
                    if let Some(arguments) = arguments { return Ok(Some((ScriptEvent { widget:id, arguments, context }, None))); }
                }
                action @ (PendingEvent::OpenSub(_) | PendingEvent::MountSub(_)
                    | PendingEvent::ResyncChildren { .. } | PendingEvent::ResyncClose { .. } | PendingEvent::CloseModal(_)) => {
                    // Mutations and their synchronous callbacks precede the saved continuation.
                    let tail = std::mem::take(&mut self.scripts);
                    let result = self.resume_mutation(action, cache.as_deref_mut());
                    self.scripts.extend(tail);
                    result?;
                }
                PendingEvent::MoveSub { source, destination, moving } => {
                    if let Some(mut sub) = moving {
                        self.children.remove(&source);
                        sub.parent = destination;
                        self.children.insert(destination, sub);
                    }
                    // ps.aa realigns the destination before its post-move listeners,
                    // even when no source attachment was found.
                    if self.widgets.contains_key(&destination) {
                        self.align_widget_descendants(destination, self.geometry_canvas.get())?;
                    }
                    self.scripts.push_front(PendingEvent::Lifecycle(1424));
                }
                PendingEvent::Lifecycle(opcode) => {
                    if let Some(root) = self.root {
                        self.scripts
                            .push_front(PendingEvent::LifecycleGroup(root, opcode, 0));
                    }
                }
                PendingEvent::LifecycleGroup(group, opcode, depth) => {
                    if depth > 128 { return Err(Error::Invalid("interface lifecycle hierarchy depth")); }
                    let array = self.lifecycle_group(group);
                    self.scripts.push_front(PendingEvent::LifecycleArray(array, 0, opcode, depth));
                }
                PendingEvent::LifecycleArray(array, index, opcode, depth) => {
                    let slot = array.borrow().get(index).cloned();
                    if let Some(slot) = slot {
                        self.scripts.push_front(PendingEvent::LifecycleArray(array, index + 1, opcode, depth));
                        if let Some(object) = slot {
                            self.scripts.push_front(PendingEvent::LifecycleWidget(object, opcode, depth + 1));
                        }
                    }
                }
                PendingEvent::LifecycleWidget(object, opcode, depth) => {
                    if depth > 128 { return Err(Error::Invalid("interface lifecycle hierarchy depth")); }
                    let Some(snapshot) = self.lifecycle_snapshot(&object) else { continue; };
                    self.scripts.push_front(PendingEvent::LifecycleListener(object.clone(), opcode));
                    if snapshot.kind == 0 {
                        self.scripts.push_front(PendingEvent::LifecycleMount(object, opcode, depth));
                        if let Some(array) = snapshot.children {
                            self.scripts.push_front(PendingEvent::LifecycleArray(array, 0, opcode, depth));
                        }
                    }
                }
                PendingEvent::LifecycleMount(object, opcode, depth) => {
                    if let Some(snapshot) = self.lifecycle_snapshot(&object) {
                        if snapshot.child_index == -1 {
                            if let Some(sub) = self.children.get(&snapshot.logical_id) {
                                self.scripts.push_front(PendingEvent::LifecycleGroup(sub.group, opcode, depth + 1));
                            }
                        }
                    }
                }
                PendingEvent::LifecycleListener(object, opcode) => {
                    let Some(snapshot) = self.lifecycle_snapshot(&object) else { continue; };
                    let id = object.borrow().id;
                    if opcode == 1424 && snapshot.child_index >= 0 {
                        let attached = self.dynamic.get(&(snapshot.logical_id, snapshot.child_index))
                            .and_then(|id| self.runtime.get(id))
                            .is_some_and(|r| r.instance == object.borrow().instance);
                        if !attached { continue; }
                    }
                    if let Some(arguments) = snapshot.listeners[usize::from(opcode == 1424)].clone() {
                        return Ok(Some((ScriptEvent { widget: id, arguments,
                            context: crate::script_event::Context {
                                widget: Some((snapshot.logical_id, snapshot.child_index)),
                                ..Default::default()
                            } }, None)));
                    }
                }
            }
        }
        self.prune_lifecycle_registries();
        Ok(None)
    }
    /// Drops registry entries whose traversal has released them. Only dead
    /// weak references are removed, so pruning never changes event order.
    fn prune_lifecycle_registries(&mut self) {
        self.lifecycle_objects.retain(|_, object| object.strong_count() != 0);
        self.lifecycle_arrays.retain(|_, array| array.strong_count() != 0);
        self.lifecycle_groups.retain(|_, array| array.strong_count() != 0);
    }
    #[cfg(test)]
    pub(crate) fn lifecycle_registry_sizes(&self) -> [usize; 3] {
        [self.lifecycle_objects.len(), self.lifecycle_arrays.len(), self.lifecycle_groups.len()]
    }
    /// Starts a live generation: an execution already in progress, work
    /// already queued, and the structural expansions and mutation
    /// continuations they produce (push_front, and the take/extend of
    /// close_modal and resume_mutation) run before the boundary; entries
    /// appended later wait for the next generation. Installed even when the
    /// queue is empty, so a running execution's appends are bounded too.
    pub fn begin_generation(&mut self) {
        self.scripts.push_back(PendingEvent::Boundary);
    }
    /// Ends a live generation: removes its boundary wherever it now is and
    /// prunes lifecycle registries the generation released (the cleanup
    /// next_event_using performs when it empties the queue).
    pub fn end_generation(&mut self) {
        self.scripts.retain(|event| !matches!(event, PendingEvent::Boundary));
        self.prune_lifecycle_registries();
    }
    /// Close modal mounts in table order, guarding 3103 until all callbacks finish.
    pub fn close_modal(&mut self) -> Result<()> {
        let parents = self.children.values().filter(|s| s.kind == 0 || s.kind == 3)
            .map(|s| s.parent).collect();
        self.closing_modal = true;
        let tail = std::mem::take(&mut self.scripts);
        self.scripts.push_back(PendingEvent::CloseModal(parents));
        let result = self.next_event_using(None);
        if let Ok(Some((event, instance))) = &result {
            self.scripts.push_front(PendingEvent::requeued(event.clone(), *instance));
        }
        self.scripts.extend(tail);
        result.map(|_| ())
    }
    pub fn close(&mut self, parent: u32) -> Result<()> {
        self.close_with_unload(parent, true)
    }
    fn close_with_unload(&mut self, parent: u32, unload: bool) -> Result<()> {
        if let Some(s) = self.children.remove(&parent) {
            self.events.retain(|(id, _), _| id >> 16 != s.group as u32);
            if unload {
                let removed = self
                    .runtime
                    .iter()
                    .filter(|(_, r)| r.logical_id >> 16 == s.group as u32)
                    .map(|(&id, _)| id)
                    .collect::<HashSet<_>>();
                // ll.nn clears the retained static array before detaching it.
                if let Some(array) = self.lifecycle_groups.remove(&s.group).and_then(|a| a.upgrade()) {
                    array.borrow_mut().fill(None);
                }
                for &id in &removed { self.retire_lifecycle_object(id); }
                self.lifecycle_arrays.retain(|id, _| !removed.contains(id));
                self.child_array_lengths.retain(|id, _| !removed.contains(id));
                self.widgets.retain(|id, _| !removed.contains(id));
                self.runtime.retain(|id, _| !removed.contains(id));
                self.dynamic.retain(|_, id| !removed.contains(id));
                self.loaded.remove(&s.group);
                self.static_group_lengths.remove(&s.group);
            }
            self.queue_lifecycle(1424)?;
        }
        Ok(())
    }
    fn resume_mutation(&mut self, action: PendingEvent, cache: Option<&mut DiskCache>) -> Result<()> {
        match action {
            PendingEvent::CloseModal(mut parents) => {
                if let Some(parent) = parents.pop_front() {
                    self.close(parent)?;
                    self.scripts.push_back(PendingEvent::CloseModal(parents));
                } else {
                    self.set_continue_widget(None);
                    self.closing_modal = false;
                }
            }
            PendingEvent::OpenSub(s) => {
                if let Some(old) = self.children.get(&s.parent) {
                    self.close_with_unload(s.parent, old.group != s.group)?;
                }
                self.scripts.push_back(PendingEvent::MountSub(s));
            }
            PendingEvent::MountSub(s) => {
                if !self.loaded.contains(&s.group) {
                    self.load_group(cache.ok_or(Error::Invalid("pending interface load requires cache"))?, s.group)?;
                }
                let group = s.group;
                let parent = s.parent;
                self.children.insert(parent, s);
                self.set_continue_widget(None);
                // ps.aj resizes the host's descendants before running on-load scripts.
                if self.widgets.contains_key(&parent) {
                    self.align_widget_descendants(parent, self.geometry_canvas.get())?;
                }
                self.on_load(group);
                self.queue_lifecycle(1424)?;
            }
            PendingEvent::ResyncChildren { mut children, parents, events } => {
                while let Some(s) = children.pop_front() {
                    // Matching mounts retain their kind, widgets and animation clocks.
                    if self.children.get(&s.parent).is_none_or(|old| old.group != s.group) {
                        self.scripts.push_back(PendingEvent::OpenSub(s));
                        self.scripts.push_back(PendingEvent::ResyncChildren { children, parents, events });
                        return Ok(());
                    }
                }
                let parents = self.children.keys().filter(|id| !parents.contains(id)).copied().collect();
                self.scripts.push_back(PendingEvent::ResyncClose { parents, events });
            }
            PendingEvent::ResyncClose { mut parents, events } => {
                if let Some(parent) = parents.pop_front() {
                    self.close(parent)?;
                    self.scripts.push_back(PendingEvent::ResyncClose { parents, events });
                } else {
                    self.events.clear();
                    for event in events { self.event_masks(event)?; }
                }
            }
            _ => return Err(Error::Invalid("interface mutation continuation")),
        }
        Ok(())
    }
    fn resume_until_callback(&mut self, cache: &mut DiskCache) -> Result<()> {
        if let Some((event, instance)) = self.next_event_using(Some(cache))? {
            self.scripts.push_front(PendingEvent::requeued(event, instance));
        }
        Ok(())
    }
    fn open(&mut self, cache: &mut DiskCache, s: SubInterface) -> Result<()> {
        self.scripts.push_back(PendingEvent::OpenSub(s));
        self.resume_until_callback(cache)
    }
    pub fn move_sub(&mut self, source: u32, destination: u32) -> Result<()> {
        let moving = self.children.get(&source).cloned();
        let unload =
            moving.as_ref().map(|s| s.group) != self.children.get(&destination).map(|s| s.group);
        self.close_with_unload(destination, unload)?;
        // ps.aa runs destination-close listeners before relocating the source.
        // Resume the mutation only after that traversal's callbacks have completed.
        self.scripts.push_back(PendingEvent::MoveSub { source, destination, moving });
        // Keep callback-free updates immediate, but leave the first callback queued.
        if let Some((event, instance)) = self.next_event_using(None)? {
            self.scripts.push_front(PendingEvent::requeued(event, instance));
        }
        Ok(())
    }
    pub fn apply(
        &mut self,
        cache: &mut DiskCache,
        items: &Definitions,
        inventories: &mut Inventories,
        u: Update,
    ) -> Result<()> {
        if matches!(u, Update::PlayerBody { .. }) { self.ensure_appearance_kits(cache)?; }
        if matches!(u, Update::PlayerSelf { .. }) && self.local_player.is_none() { return Ok(()); }
        match u {
            Update::OpenTop(group) => {
                if group == 65535 {
                    self.root = None;
                } else {
                    self.load_group(cache, group)?;
                    self.root = Some(group);
                    self.align_root_group(group)?;
                    self.on_load(group);
                    self.resume_until_callback(cache)?;
                }
            }
            Update::OpenSub(s) => self.open(cache, s)?,
            Update::CloseSub(id) => self.close(id)?,
            Update::MoveSub {
                source,
                destination,
            } => self.move_sub(source, destination)?,
            Update::Resync {
                root,
                children,
                events,
            } => {
                if root != 65535 {
                    self.load_group(cache, root)?;
                }
                let new_root = (root != 65535).then_some(root);
                if self.root != new_root {
                    self.root = new_root;
                    if let Some(g) = new_root {
                        self.align_root_group(g)?;
                        self.on_load(g);
                    }
                }
                let parents = children.iter().map(|s| s.parent).collect();
                self.scripts.push_back(PendingEvent::ResyncChildren {
                    children: children.into(), parents, events,
                });
                self.resume_until_callback(cache)?;
            }
            Update::Events(e) => self.event_masks(e)?,
            Update::ClearInventory(id) => inventories.clear_widget(id),
            other => {
                let id = match &other {
                    Update::Position { widget, .. }
                    | Update::Scroll { widget, .. }
                    | Update::Rotate { widget, .. }
                    | Update::Text { widget, .. }
                    | Update::Hide { widget, .. }
                    | Update::Angle { widget, .. }
                    | Update::Object { widget, .. }
                    | Update::Color { widget, .. }
                    | Update::Animation { widget, .. }
                    | Update::NpcHead { widget, .. }
                    | Update::NpcHeadActive { widget, .. }
                    | Update::Model { widget, .. }
                    | Update::PlayerColor { widget, .. }
                    | Update::PlayerBody { widget, .. }
                    | Update::PlayerObject { widget, .. }
                    | Update::PlayerSelf { widget, .. } => *widget,
                    Update::PlayerHead(widget) => *widget,
                    _ => unreachable!(),
                };
                self.load_group(cache, (id >> 16) as u16)?;
                let w = self
                    .widgets
                    .get_mut(&id)
                    .ok_or(Error::Invalid("interface component missing"))?;
                let realign = matches!(&other, Update::Position { position, .. }
                    if *position != w.raw_position || w.position_alignment != [0, 0]);
                let r = self.runtime.get_mut(&id).unwrap();
                match other {
                    Update::Position { position, .. } => {
                        w.raw_position = position;
                        w.position_alignment = [0; 2];
                    }
                    Update::Scroll { y, .. } => {
                        if w.kind == 0 {
                            r.scroll[1] =
                                (y as i32).clamp(0, w.scroll_size[1].wrapping_sub(
                                    r.aligned.get().unwrap_or_default().size[1]).max(0));
                        }
                    }
                    Update::Rotate { speed, .. } => r.rotation_speed = speed,
                    Update::Text { text, .. } => w.text = text,
                    Update::Hide { hidden, .. } => w.hidden = hidden,
                    Update::Angle { angles, zoom, .. } => {
                        w.model_angles[..2].copy_from_slice(&angles);
                        w.model_zoom = zoom;
                        r.model_zoom = zoom as i32;
                    }
                    Update::Object { item, count, .. } => {
                        if !w.if3 && item == -1 {
                            r.model = ModelSource::None;
                            return Ok(());
                        }
                        r.item = item;
                        r.count = count;
                        if item != -1 {
                            let d = items
                                .items
                                .get(&(item as u32))
                                .ok_or(Error::Invalid("interface item"))?;
                            let d = items
                                .items
                                .get(&d.count_variant(count))
                                .ok_or(Error::Invalid("interface item variant"))?;
                            w.model_angles = d.angles.map(|a| a as u16);
                            w.model_offset = d.offset;
                            if !w.if3 {
                                if count == 0 {
                                    return Err(Error::Invalid("legacy item zoom divisor"));
                                }
                                r.model = ModelSource::Item(item);
                                r.model_zoom = d.zoom.wrapping_mul(100) / count;
                            } else {
                                r.quantity_mode = if d.stackable == 1 { 1 } else { 2 };
                                r.model_zoom = d.zoom;
                                let width = if w.model_width_scale > 0 {
                                    w.model_width_scale as i32
                                } else {
                                    w.raw_size[0]
                                };
                                if width > 0 {
                                    r.model_zoom = r.model_zoom.wrapping_mul(32) / width;
                                }
                            }
                        }
                    }
                    Update::Color { packed, .. } => {
                        let p = packed as u32;
                        w.colors[0] =
                            ((p >> 10) & 31) << 19 | ((p >> 5) & 31) << 11 | (p & 31) << 3;
                    }
                    Update::Animation { id, .. } => {
                        let id = id as i32;
                        if w.sequences[0] != id || id == -1 {
                            w.sequences[0] = id;
                            r.frame = 0;
                            r.frame_cycle = 0;
                        }
                    }
                    Update::NpcHead { id, .. } => r.model = ModelSource::Npc(id as i32),
                    Update::NpcHeadActive { index, .. } => r.model = ModelSource::ActiveNpc(index),
                    Update::PlayerHead(_) => r.model = ModelSource::Player,
                    Update::Model { id, .. } => {
                        w.models[0] = id;
                        r.model = ModelSource::Raw(id);
                    }
                    Update::PlayerColor { index, color, .. } => {
                        r.player_appearance.as_mut().ok_or(Error::Invalid("widget player composition missing"))?.color(index as i32, color as i32)?;
                        *r.player_colors
                            .get_mut(index as usize)
                            .ok_or(Error::Invalid("player appearance color index"))? = color;
                    }
                    Update::PlayerBody { body, .. } => {
                        r.player_appearance.as_mut().ok_or(Error::Invalid("widget player composition missing"))?.body(self.local_player.as_ref().map_or(0, |a| a.gender), body, self.appearance_kits.as_ref().unwrap(), items)?;
                        r.player_body = body;
                    }
                    Update::PlayerObject { item, .. } => {
                        let definition = items.items.get(&(item as u32)).ok_or(Error::Invalid("widget appearance item"))?;
                        r.player_appearance.as_mut().ok_or(Error::Invalid("widget player composition missing"))?.equip(definition)?;
                        r.player_item = Some(item);
                    },
                    Update::PlayerSelf { copy_objects, .. } => {
                        r.model = ModelSource::PlayerSelf(copy_objects);
                        r.player_appearance = self.local_player.as_ref().map(|a| crate::widget_appearance::Composition::snapshot(a, copy_objects));
                    }
                    _ => unreachable!(),
                }
                if realign {
                    self.align_script_widget(id, self.geometry_canvas.get(), true)?;
                }
            }
        }
        Ok(())
    }
    /// Compute a script geometry query from the ancestor chain, without cloning the entire HUD.
    pub fn geometry(
        &self,
        id: u32,
        canvas: [i32; 2],
    ) -> Result<Option<crate::widget_layout::Geometry>> {
        if canvas.iter().any(|&v| !(0..=16384).contains(&v)) {
            return Err(Error::Invalid("interface canvas size"));
        }
        let mut chain = Vec::new();
        let mut next = id;
        let mut seen = HashSet::new();
        loop {
            if chain.len() > 128 || !seen.insert(next) {
                return Err(Error::Invalid("interface hierarchy depth/cycle"));
            }
            let Some(w) = self.widgets.get(&next) else {
                return Ok(None);
            };
            chain.push(w);
            let runtime=&self.runtime[&next];
            let effective_parent=if runtime.child_index>=0&&runtime.tree_parent>=0 {
                let Some(parent)=self.dynamic.get(&(runtime.logical_id,runtime.tree_parent)).copied() else{return Ok(None);};
                Some(parent)
            }else{w.parent};
            if let Some(parent) = effective_parent {
                next = parent;
            } else {
                let group = (self.runtime.get(&next).map_or(next, |r| r.logical_id) >> 16) as u16;
                if self.root == Some(group) {
                    break;
                }
                let mut attachments = self.children.values().filter(|sub| sub.group == group);
                let Some(sub) = attachments.next() else {
                    return Ok(None);
                };
                if attachments.next().is_some() {
                    return Err(Error::Invalid("duplicate interface attachment"));
                }
                next = sub.parent;
            }
            if self.widgets.get(&next).is_none_or(|w| w.kind != 0) {
                return Ok(None);
            }
        }
        let mut extent = canvas;
        let mut geometry = crate::widget_layout::Geometry::default();
        for w in chain.into_iter().rev() {
            geometry = match self.runtime[&w.id].aligned.get() {
                Some(stored) => stored,
                None => crate::widget_layout::align(w, extent, Default::default(), [1, 1])?,
            };
            extent = std::array::from_fn(|axis| {
                if w.scroll_size[axis] != 0 {
                    w.scroll_size[axis]
                } else {
                    geometry.size[axis]
                }
            });
        }
        Ok(Some(geometry))
    }
    pub fn layout(&self, canvas: [i32; 2]) -> Result<Layout> {
        let layout = self.preview_layout(canvas)?;
        self.record_layout_geometry(&layout);
        Ok(layout)
    }
    pub(crate) fn record_layout_geometry(&self, layout: &Layout) {
        for &id in &layout.order {
            self.runtime[&id].aligned.set(Some(layout.widgets[&id].geometry));
        }
    }
    /// Compute visibility and hit geometry without consuming stored size-change comparisons.
    pub fn preview_layout(&self, canvas: [i32; 2]) -> Result<Layout> {
        if let Some(cache)=self.layout_cache.borrow().as_ref().filter(|cache|cache.matches(self,canvas,self.legacy_hover)) {
            self.geometry_canvas.set(canvas);
            return Ok(cache.layout.clone());
        }
        let layout=self.preview_layout_uncached(canvas)?;
        *self.layout_cache.borrow_mut()=Some(crate::layout_cache::Cache::new(self,canvas,self.legacy_hover,layout.clone()));
        Ok(layout)
    }
    pub(crate) fn preview_layout_uncached(&self, canvas: [i32; 2]) -> Result<Layout> {
        // Recompute reachability because mount owners and logical groups are
        // freely mutable. Keep retained definitions outside this closure loaded.
        let mut active=self.root.into_iter().collect::<HashSet<_>>();
        loop {
            let mut changed=false;
            for (&owner,sub) in self.children.iter() {
                if self.widgets.contains_key(&owner)
                    && self.runtime.get(&owner).is_some_and(|r|active.contains(&((r.logical_id>>16) as u16))) {
                    changed|=active.insert(sub.group);
                }
            }
            if !changed {break;}
        }
        // Index once per layout. Definitions and runtime remain freely mutable
        // between calls, so this does not introduce an invalidation cache.
        let mut groups = HashMap::<u16, Vec<(&Widget, &Runtime)>>::new();
        for w in self.widgets.values() {
            let runtime=&self.runtime[&w.id];
            let group=(runtime.logical_id >> 16) as u16;
            if active.contains(&group) {groups.entry(group).or_default().push((w,runtime));}
        }
        let mut included = HashSet::new();
        let mut pending = self.root.into_iter().map(|g| (g, None)).collect::<Vec<_>>();
        let mut widgets = Vec::new();
        while let Some((group, parent)) = pending.pop() {
            if !included.insert(group) {
                return Err(Error::Invalid("duplicate or cyclic interface attachment"));
            }
            let mut group_widgets = groups.remove(&group).unwrap_or_default();
            group_widgets.sort_by_cached_key(|(_,runtime)| {
                (
                    runtime.child_index >= 0,
                    runtime.logical_id,
                    runtime.child_index,
                )
            });
            for (w,runtime) in group_widgets {
                if let Some(sub) = self.children.get(&w.id) {
                    pending.push((sub.group, Some(w.id)));
                }
                let effective_parent=if runtime.child_index>=0&&runtime.tree_parent>=0 {
                    let Some(container)=self.dynamic.get(&(runtime.logical_id,runtime.tree_parent)).copied() else{continue;};
                    let bounds=self.runtime[&container].child_bounds;
                    if !(bounds[0]..=bounds[1]).contains(&runtime.child_index){continue;}
                    Some(container)
                }else{w.parent.or(parent)};
                widgets.push((w, effective_parent));
            }
        }
        let scroll = |id| self.runtime[&id].scroll;
        let stored = |id| self.runtime[&id].aligned.get();
        let mut layout = Layout::build_attached_with_runtime(&widgets, canvas, &scroll, &stored)?;
        self.geometry_canvas.set(canvas);
        for &id in &layout.order {
            let w = &self.widgets[&id];
            let parent_hidden = layout.widgets[&id].parent.is_some_and(|p| layout.widgets[&p].hidden);
            layout.widgets.get_mut(&id).unwrap().hidden = parent_hidden
                || (w.hidden && (w.if3 || (w.kind == 0 && !self.legacy_hovered(id))));
        }
        Ok(layout)
    }
}
