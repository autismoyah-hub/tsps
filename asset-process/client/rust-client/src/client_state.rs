//! Cache, protocol and script state shared by the native client UI.
use crate::{
    disk_cache::DiskCache,
    game_packet, interface_packet, interface_script,
    interface_state::{Interfaces, ScriptEvent},
    inventory::Inventories,
    item::Definitions,
    script::Script,
    script_runtime::{CacheSource, Runtime},
    vm::{self, Control, Host, Object, Stacks, Value, Variable, Vm},
    Error, Result,
};
use std::rc::Rc;
pub struct EventExecution {
    resize_depth: i32,
    vm: Vm,
    id: i32,
    widget: Option<(u32, i32)>,
}
impl EventExecution {
    pub fn step(&mut self, client: &mut ClientState, budget: usize) -> Result<vm::Status> {
        let previous_depth = std::mem::replace(&mut client.resize_depth, self.resize_depth);
        let result = client.step_vm(&mut self.vm, budget);
        client.resize_depth = previous_depth;
        client.script_slot0 = self.vm.stacks.int_slot0();
        result.map_err(|e| {
            Error::Io(format!(
                "interface script {} widget {:?}: {e}",
                self.id, self.widget
            ))
        })
    }
}
pub struct ClientState {
    pub site_settings: String,
    pub uid192: crate::uid192::State,
    pub remote: crate::remote_url::Context,
    resize_depth: i32,
    deferred_widget_operations: std::collections::VecDeque<crate::widget_deferred::Operation>,
    script_close_pending: bool,
    pub widget_drag: Option<crate::widget_drag::Drag>,
    /// Pointer menu built before interface callbacks; script drags retain its actions.
    pub pointer_menu: Vec<crate::context_menu::Entry<crate::context_menu::Action>>,
    pub selected_spell: Option<crate::spell_action::Selection>,
    pub menu: Option<crate::context_menu::Menu<crate::context_menu::Action>>,
    pub frame_rate: crate::client_metrics::FrameRate,
    #[cfg(feature="native-profiling")]
    pub ui_profile:Option<crate::ui_profile::Profile>,
    pub scripts: Runtime<CacheSource>,
    pub interfaces: Interfaces,
    pub inventories: Inventories,
    pub items: Definitions,
    pub npcs: std::collections::HashMap<u32, crate::npc::Definition>,
    pub spots: std::collections::HashMap<u32, crate::spot_animation::Definition>,
    pub world_spots: Vec<crate::world_spot::Effect>,
    pub view_registry: crate::world_view::Registry,
    pub world_entities: std::collections::HashMap<u16, crate::world_entity::State>,
    /// Registered descendant views whose owning parent view was removed.
    /// Their actor/zone state survives, but their former owner entities do not tick.
    pub detached_world_entities: Vec<crate::world_entity::State>,
    pub world_entity_limit: crate::world_entity_limit::Limit,
    world_entity_definitions: std::collections::HashMap<i32, crate::world_entity::Definition>,
    world_rebuild_generation: u64,
    pub overlay_definitions: crate::actor_overlay::Definitions,
    pub projectiles: Vec<crate::projectile::Projectile>,
    pub scripted_projectiles: crate::scripted_projectile::State,
    pub scripted_events: std::collections::VecDeque<(i32, crate::scripted_projectile::Event)>,
    script_slot0: i32,
    script_int_slots: Rc<std::cell::RefCell<Vec<i32>>>,
    pub world_base: [i32; 2],
    pub active_world: u16,
    pub active_plane: u8,
    pub npc_origin: Option<[i32; 2]>,
    npc_origin_offset: Option<[u8; 2]>,
    pub npc_state: crate::npc_info::State,
    pub actor_events: std::collections::VecDeque<(bool, u16, u32, usize)>,
    pub player_state: Option<crate::player_info::State>,
    huffman: Option<crate::huffman::Huffman>,
    pub(crate) animation_readiness: crate::animation_readiness::Readiness,
    interface_sequences: Option<std::collections::HashMap<u32, crate::animation::Sequence>>,
    pub database: crate::database::Database,
    pub groups: crate::group_state::State,
    pub skybox: crate::skybox::State,
    pub heatmap: crate::heatmap::State,
    pub world_map: crate::world_map::State,
    pub params: crate::script_params::Parameters,
    pub item_search: crate::item_search::Search,
    pub active_widgets: [Option<u32>; 2],
    pub canvas: [i32; 2],
    pub window: crate::window_layout::State,
    pub data: crate::script_data::Data,
    pub viewport: crate::viewport_shape::Settings,
    pub preferences: crate::client_preferences::Preferences,
    pub input: crate::input::Input,
    pub audio: crate::audio_runtime::Audio,
    pub interface_events: crate::interface_events::Events,
    pub cycle: i32,
    pub actor_cycle: i32,
    pub script_failures: std::collections::VecDeque<Error>,
    pub outbound: crate::outbound::Queue,
    pub scene_walk_commands: std::collections::VecDeque<crate::scene_walk::Command>,
    pub logout_timer: i32,
    pub logout_request: Option<u8>,
    pub world_hop: Option<crate::game_packet::WorldHop>,
    pub combat_level: i32,
    pub hidden_operations: [bool; 3],
    pub player_operations: [Option<String>; 8],
    pub player_operation_priority: [bool; 8],
    pub chat: crate::chat::Chat,
    pub chat_delivery: crate::chat_packet::Delivery,
    pub social: crate::social::Social,
    pub friends_chat: Option<crate::friends_chat::Channel>,
    pub market: crate::market::Market,
    pub interaction_policy: crate::interaction_policy::Policies,
    pub steering: crate::steering::Steering,
    friends_chat_changed: bool,
    pub clans: crate::clan_settings::Clans,
    pub clan_channels: crate::clan_channel::Channels,
    pub clan_profile: crate::clan_profile::Profile,
    pub game_messages: std::collections::VecDeque<crate::game_packet::GameMessage>,
    pub minimap_state: u8,
    pub map_flag: Option<u32>,
    pub click_cross: crate::click_cross::Cross,
    pub hint: crate::hint::Hint,
    pub zone: crate::zone::State,
    pub server_tick: i32,
    varp_ids: Vec<u16>,
    varp_types: std::collections::HashMap<u16, u16>,
    pub menu_settings: crate::menu_settings::Settings,
    observed_varps: u64,
    pub camera: crate::camera::State,
    pub appearance_editor: Option<crate::player_model::Appearance>,
    appearance_kits: Option<std::collections::HashMap<u32, crate::identity_kit::Kit>>,
}
impl ClientState {
    /// Start a fresh authenticated world while retaining local preferences and persistent varcs.
    /// World membership changes rebuild context-sensitive definitions transactionally.
    pub fn set_members(&mut self, members: bool) -> Result<()> {
        if self.data.members != members {
            let items = Definitions::load_for_world(&mut self.scripts.source.cache, members)?;
            self.items = items;
            self.item_search = Default::default();
            self.data.members = members;
        }
        Ok(())
    }
    pub fn prepare_login(&mut self) -> Result<()> {
        let definitions = self.scripts.source.cache.group(2, 19, [0; 4])?;
        let count = definitions
            .iter()
            .map(|(id, _)| *id as usize + 1)
            .max()
            .unwrap_or(0);
        let mut persistent = vec![false; count];
        for (id, bytes) in definitions {
            persistent[id as usize] = crate::script_runtime::varc_persistent(&bytes)?;
        }
        let mut fresh = Self::load(self.scripts.source.cache.reopen()?)?;
        fresh.preferences = std::mem::take(&mut self.preferences);
        // Login reset clears queued effects, while music survives until the
        // successful-login title cleanup schedules its fade.
        fresh.audio.music = std::mem::take(&mut self.audio.music);
        fresh.scripts.variables = self.scripts.variables.take_for_login(&persistent);
        fresh.canvas = self.canvas;
        fresh.cycle = self.cycle;
        fresh.actor_cycle = self.actor_cycle;
        fresh.data.world = self.data.world;
        fresh.data.members = self.data.members;
        fresh.data.account_members = self.data.account_members;
        std::mem::swap(&mut fresh.items, &mut self.items);
        fresh.data.world_flags = self.data.world_flags;
        fresh.data.worlds = std::mem::take(&mut self.data.worlds);
        // cj.ji, my.af and GameClient.nq survive the full login-state reset.
        fresh.remote = std::mem::take(&mut self.remote);
        // GameNPCComposition.ad retains GameClient.lw and the open mn.ai file.
        fresh.uid192 = std::mem::take(&mut self.uid192);
        // qn.gu survives the original full world/login reset.
        fresh.site_settings = std::mem::take(&mut self.site_settings);
        fresh.data.client_type = self.data.client_type;
        fresh.data.platform_type = self.data.platform_type;
        fresh.data.mobile = self.data.mobile;
        fresh.data.mouse_reported = self.data.mouse_reported;
        // GameClient's full login reset clears hf but retains the packet-24 ex string.
        fresh.data.server_string = std::mem::take(&mut self.data.server_string);
        fresh.frame_rate = std::mem::take(&mut self.frame_rate);
        // GameClient.gt is script-controlled and survives the full login reset.
        fresh.world_entity_limit = self.world_entity_limit;
        // GameClient.qk survives the full login reset; each scene reloads its model.
        fresh.skybox.request(self.skybox.requested);
        fresh.world_map = std::mem::take(&mut self.world_map);
        *self = fresh;
        Ok(())
    }
    pub fn load(mut cache: DiskCache) -> Result<Self> {
        let definitions = cache.group(2, 19, [0; 4])?;
        let count = definitions
            .iter()
            .map(|(id, _)| *id as usize + 1)
            .max()
            .unwrap_or(0);
        let mut persistent = vec![false; count];
        for (id, bytes) in definitions {
            persistent[id as usize] = crate::script_runtime::varc_persistent(&bytes)?;
        }
        let npcs = crate::npc::Definition::load_all(&mut cache)?;
        let spots = crate::spot_animation::Definition::load_all(&mut cache)?;
        let overlay_definitions = crate::actor_overlay::Definitions::load(&mut cache)?;
        let varp_types = cache
            .group(2, 16, [0; 4])?
            .into_iter()
            .map(|(id, bytes)| Ok((id as u16, crate::menu_settings::definition(&bytes)?)))
            .collect::<Result<std::collections::HashMap<_, _>>>()?;
        let varp_ids = varp_types.keys().copied().collect();
        let mut scripts = Runtime::new(CacheSource::new(cache), 0);
        scripts.variables.set_persistence(persistent);
        let mut data = crate::script_data::Data::default();
        data.client_type = Some(1);
        if let Ok(value) = std::env::var("OSRS_WORLD_FLAGS") {
            data.world_flags = value.parse::<u32>().map_err(|_| Error::Invalid("OSRS_WORLD_FLAGS must be an unsigned decimal integer"))? as i32;
            data.members = data.world_flags & 1 != 0;
        }
        if let Ok(value) = std::env::var("OSRS_WORLD_ID") {
            data.world = value.parse::<u16>().map_err(|_| Error::Invalid("OSRS_WORLD_ID must be an unsigned 16-bit decimal integer"))? as i32;
        }
        let items = Definitions::load_for_world(&mut scripts.source.cache, data.members)?;
        data.worlds.set_source(std::env::var_os("OSRS_WORLD_LIST_FILE").map(std::path::PathBuf::from));
        if !data.worlds.configured() {
            data.worlds.set_url(std::env::var("OSRS_WORLD_LIST_URL").ok());
        }
        Ok(Self {
            menu: None,
            selected_spell: None,
            widget_drag: None,
            pointer_menu: Vec::new(),
            camera: Default::default(),
            appearance_editor: None,
            appearance_kits: None,
            frame_rate: Default::default(),
            #[cfg(feature="native-profiling")]
            ui_profile:None,
            scripts,
            interfaces: Interfaces::default(),
            inventories: Inventories::default(),
            items,
            npcs,
            spots,
            world_spots: Vec::new(),
            view_registry: Default::default(),
            world_entities: Default::default(),
            detached_world_entities: Default::default(),
            world_entity_limit: Default::default(),
            world_entity_definitions: Default::default(),
            world_rebuild_generation: 0,
            overlay_definitions,
            projectiles: Vec::new(),
            scripted_projectiles: Default::default(),
            scripted_events: Default::default(),
            site_settings: String::new(),
            uid192: Default::default(),
            remote: Default::default(),
            script_slot0: 0,
            script_int_slots: Default::default(),
            resize_depth: 0,
            deferred_widget_operations: Default::default(),
            script_close_pending: false,
            world_base: [0; 2],
            active_world: 0,
            active_plane: 0,
            npc_origin: None,
            npc_origin_offset: None,
            npc_state: Default::default(),
            actor_events: Default::default(),
            player_state: None,
            huffman: None,
            interface_sequences: None,
            animation_readiness: Default::default(),
            database: Default::default(),
            groups: Default::default(),
            skybox: Default::default(),
            heatmap: Default::default(),
            world_map: Default::default(),
            params: Default::default(),
            item_search: Default::default(),
            active_widgets: [None; 2],
            canvas: [765, 503],
            window: Default::default(),
            data,
            viewport: Default::default(),
            preferences: Default::default(),
            input: Default::default(),
            audio: Default::default(),
            interface_events: Default::default(),
            cycle: 0,
            actor_cycle: 1,
            script_failures: Default::default(),
            outbound: Default::default(),
            scene_walk_commands: Default::default(),
            logout_timer: 0,
            logout_request: None,
            world_hop: None,
            combat_level: 0,
            hidden_operations: [false; 3],
            player_operations: Default::default(),
            player_operation_priority: [false; 8],
            chat: Default::default(),
            chat_delivery: Default::default(),
            social: Default::default(),
            friends_chat: None,
            market: Default::default(),
            interaction_policy: Default::default(),
            steering: Default::default(),
            friends_chat_changed: false,
            clans: Default::default(),
            clan_channels: Default::default(),
            clan_profile: Default::default(),
            game_messages: Default::default(),
            minimap_state: 0,
            map_flag: None,
            click_cross: Default::default(),
            hint: Default::default(),
            zone: Default::default(),
            server_tick: 0,
            varp_ids,
            varp_types,
            menu_settings: Default::default(),
            observed_varps: 0,
        })
    }
    fn apply_menu_varp(&mut self, id: u16) {
        if self.varp_types.get(&id)==Some(&1) {
            if let Some(brightness)=match self.scripts.variables.player(id) {1=>Some(0.9),2=>Some(0.8),3=>Some(0.7),4=>Some(0.6),_=>None} {
                self.preferences.device.brightness=brightness;
            }
        }
        self.menu_settings.apply(
            id,
            self.varp_types.get(&id).copied().unwrap_or(0),
            self.scripts.variables.player(id),
        );
    }
    // The current VM cannot roll back host side effects on a cache miss.
    // Keep its bounded synchronous execution until cooperative asset suspension exists.
    fn step_vm(&mut self, vm: &mut Vm, budget: usize) -> Result<vm::Status> {
        let previous = self.scripts.source.cache.replace_recovery_wait(true);
        let result = vm.run(self, budget);
        self.scripts.source.cache.wait_for_recovery(previous);
        result
    }
    pub fn next_interface_event(&mut self) -> Result<Option<ScriptEvent>> {
        self.interfaces.geometry_canvas.set(self.canvas);
        self.interfaces.next_event_with_cache(&mut self.scripts.source.cache)
    }
    /// As `next_interface_event`, with an end-of-cycle callback's producing instance.
    pub fn next_interface_event_tagged(&mut self) -> Result<Option<(ScriptEvent, Option<u64>)>> {
        self.interfaces.geometry_canvas.set(self.canvas);
        self.interfaces.next_event_tagged_with_cache(&mut self.scripts.source.cache)
    }
    /// Request known dependencies before starting a packet with side effects.
    pub fn prepare_packet_assets(&mut self, update: &game_packet::Update) -> Result<()> {
        match update {
            game_packet::Update::AmbientStart { id, .. } => {
                crate::ambient_sound::Definition::load(&mut self.scripts.source.cache, *id as u32)?;
            }
            game_packet::Update::Interface(update) => {
                let cache = &mut self.scripts.source.cache;
                let mut pending = None;
                for group in update.groups() {
                    if let Err(error) = cache.group(3, u32::from(group), [0; 4]) {
                        match error {
                            Error::PendingAsset { .. } => pending = Some(error),
                            _ => return Err(error),
                        }
                    }
                }
                if let Some(error) = pending { return Err(error); }
            }
            game_packet::Update::Script(call) if call.id >= 0 => {
                self.scripts.script(call.id)?;
                let cache = &mut self.scripts.source.cache;
                if cache.reference(2)?.groups.iter().any(|g|g.id == 14) {
                    cache.group(2, 14, [0; 4])?;
                }
            }
            _ => {},
        }
        Ok(())
    }
    pub fn interface(&mut self, u: interface_packet::Update) -> Result<()> {
        self.interfaces.geometry_canvas.set(self.canvas);
        if matches!(u, interface_packet::Update::PlayerSelf { .. } | interface_packet::Update::PlayerBody { .. }) {
            self.refresh_widget_player();
        }
        self.interfaces.apply(
            &mut self.scripts.source.cache,
            &self.items,
            &mut self.inventories,
            u,
        )
    }
    fn refresh_widget_player(&mut self) {
        self.interfaces.local_player = self.player_state.as_ref().and_then(|s| s.local_actor())
            .and_then(|p| p.appearance.as_ref()).map(|a| a.appearance.clone());
    }
    fn content_action(&mut self, content: u16) -> Result<bool> {
        match content {
            0 => Ok(true),
            205 => { self.logout_timer = crate::script_logout::TIMER; Ok(true) }
            300..=325 => {
                if self.appearance_kits.is_none() {
                    self.appearance_kits = Some(crate::identity_kit::Kit::load_all(&mut self.scripts.source.cache)?);
                }
                let kits = self.appearance_kits.as_ref().unwrap();
                let appearance = self.appearance_editor.get_or_insert_with(|| crate::player_model::Appearance::standard(kits, 0));
                appearance.edit_content(kits, content)?;
                Ok(false)
            }
            326 => {
                if self.appearance_editor.is_none() {
                    if self.appearance_kits.is_none() {
                        self.appearance_kits = Some(crate::identity_kit::Kit::load_all(&mut self.scripts.source.cache)?);
                    }
                    self.appearance_editor = Some(crate::player_model::Appearance::standard(self.appearance_kits.as_ref().unwrap(), 0));
                }
                self.outbound.push_back(crate::protocol::Packet {
                    opcode: 77,
                    payload: self.appearance_editor.as_ref().unwrap().design_bytes(),
                });
                Ok(true)
            }
            _ => Ok(false),
        }
    }
    pub fn interface_operation(
        &mut self,
        id: u32,
        operation: u8,
        sub_operation: Option<u8>,
    ) -> Result<bool> {
        self.interface_operation_with_base(id, operation, sub_operation, None)
    }
    fn interface_operation_with_base(&mut self,id:u32,operation:u8,sub_operation:Option<u8>,base:Option<&str>)->Result<bool> {
        self.interface_operation_with_item(id, operation, sub_operation, base, None)
    }
    fn interface_operation_with_item(&mut self,id:u32,operation:u8,sub_operation:Option<u8>,base:Option<&str>,captured_item:Option<i32>)->Result<bool> {
        if !(1..=32).contains(&operation) {
            return Err(Error::Invalid("interface operation index"));
        }
        let Some(object) = self.interfaces.hold_widget(id) else { return Ok(false); };
        let Some(runtime) = self.interfaces.runtime.get(&id) else {
            return Ok(false);
        };
        let logical = runtime.logical_id;
        let child = runtime.child_index;
        let item = captured_item.unwrap_or(runtime.item);
        let context = crate::script_event::Context {
            widget: Some((logical, child)),
            operation: operation as i32,
            extra: sub_operation.map_or(0, |v| v as i32 + 1),
            operation_base: Some(base.map_or_else(||self.interfaces.widgets[&id].data_text.clone(),str::to_owned)),
            ..Default::default()
        };
        if let Some((args, _)) = self.interfaces.listener(id, 1409) {
            // eo reports callback script failures but ds.fi still dispatches the operation.
            if let Err(error) = self.run_listener(&args, &context) {
                if self.script_failures.len() == 64 { self.script_failures.pop_front(); }
                self.script_failures.push_back(error);
            }
        }
        // ds.fi keeps its original object even if the callback deletes/replaces it.
        let mounted = self.interfaces.mount_widget_object(&object);
        let result = (|| {
        let Some(widget) = self.interfaces.widgets.get(&id) else {
            return Ok(false);
        };
        if !self.content_action(widget.content)? {
            return Ok(false);
        }
        if self.interfaces.operation_mask(id) & (1 << (operation - 1)) == 0 {
            return Ok(false);
        }
        self.outbound.push_back(crate::interface_action::operation(
            logical,
            child,
            item,
            operation,
            sub_operation,
        )?);
        Ok(true)
        })();
        if let Some(mounted) = mounted { self.interfaces.unmount_selected_widget(mounted); }
        result
    }
    /// Dispatch the same default entry used by the native interface menu.
    pub fn primary_interface_action(&mut self, point: [i32; 2]) -> Result<bool> {
        let Some(entry) = self.interface_menu(point)?.last().cloned() else {
            return Ok(false);
        };
        self.menu_action(entry.action)?;
        Ok(true)
    }
    pub fn begin_widget_drag(
        &mut self,
        point: [i32; 2],
        entries: &[crate::context_menu::Entry<crate::context_menu::Action>],
    ) -> Result<bool> {
        if self.widget_drag.is_some() || self.menu.is_some() {
            return Ok(false);
        }
        let layout = self.interfaces.layout(self.canvas)?;
        for id in layout.hit_test(point) {
            let widget = &self.interfaces.widgets[&id];
            let origin = layout.widgets[&id].absolute;
            if let Some(drag) = crate::script_drag::start(
                &self.interfaces, Some(id),
                std::array::from_fn(|axis| point[axis].wrapping_sub(origin[axis])),
                origin, false, entries,
            ) {
                self.widget_drag = Some(drag);
                return Ok(true);
            }
            if widget.no_click_through {
                break;
            }
        }
        Ok(false)
    }
    pub fn tick_widget_drag(
        &mut self,
    ) -> Result<
        Option<(
            [i32; 2],
            Vec<crate::context_menu::Entry<crate::context_menu::Action>>,
        )>,
    > {
        if self.widget_drag.is_none() { return Ok(None); }
        // A layout error must not consume an otherwise valid pending drag.
        let layout = self.interfaces.layout(self.canvas)?;
        let Some(mut drag) = self.widget_drag.take() else {
            return Ok(None);
        };
        self.interfaces.drag_visual = None;
        let source_current = self.interfaces.runtime.get(&drag.widget).is_some_and(|r| r.instance == drag.instances[0]);
        let parent_current = self.interfaces.runtime.get(&drag.parent).is_some_and(|r| r.instance == drag.instances[1]);
        let source = layout.widgets.get(&drag.widget).filter(|_| source_current);
        let parent = layout.widgets.get(&drag.parent).filter(|_| parent_current);
        let visible = [
            source.is_some_and(|p| !p.hidden),
            parent.is_some_and(|p| !p.hidden),
        ];
        // uw.ae resets placement each tick; ps visits refresh it and ps.am reads live thresholds.
        drag.motion.origin = source.filter(|p| !p.hidden).map_or([-1; 2], |p| p.absolute);
        if let Some(widget) = self.interfaces.widgets.get(&drag.widget).filter(|_| source_current) {
            drag.motion.dead_zone = widget.drag_zone;
            drag.motion.dead_time = widget.drag_threshold;
        }
        let size = source.map_or([0; 2], |p| p.geometry.size);
        let bounds = parent.map_or([0; 4], |p| {
            [
                p.absolute[0],
                p.absolute[1],
                p.geometry.size[0],
                p.geometry.size[1],
            ]
        });
        let scroll = self
            .interfaces
            .runtime
            .get(&drag.parent)
            .map_or([0; 2], |r| r.scroll);
        let target = layout.hit_test(self.input.pointer).find(|&id| {
            id != drag.widget && self.interfaces.interaction_flags(id) & (1 << 20) != 0
        });
        let frame = drag.motion.step(
            self.input.pointer,
            size,
            bounds,
            scroll,
            visible,
            self.input.buttons & if self.preferences.mouse_camera { 1 } else { 5 } != 0,
            !drag.entries.is_empty(),
        );
        if let Some(runtime) = self.interfaces.runtime.get(&drag.widget).filter(|_| source_current) {
            let identity = (runtime.logical_id, runtime.child_index, runtime.item);
            let source_object = self.interfaces.retain_drag_object(drag.widget);
            let target_object = target.and_then(|id| self.interfaces.retain_drag_object(id));
            let context = crate::script_event::Context {
                widget: Some((identity.0, identity.1)),
                mouse: frame.mouse,
                drag_target: target.and_then(|id| {
                    self.interfaces
                        .runtime
                        .get(&id)
                        .map(|r| (r.logical_id, r.child_index))
                }),
                ..Default::default()
            };
            if drag.motion.dragging && visible.iter().all(|v| *v) {
                if let Some((args, _)) = self.interfaces.listener(drag.widget, 1405) {
                    self.run_listener(&args, &context)?;
                }
                if drag.motion.active {
                    self.interfaces.drag_visual = Some((
                        drag.widget,
                        frame.position,
                        [
                            bounds[0],
                            bounds[1],
                            bounds[0] + bounds[2],
                            bounds[1] + bounds[3],
                        ],
                    ));
                }
            }
            if frame.outcome == 2 {
                if let Some(args) = source_object.as_ref()
                    .and_then(|object| self.interfaces.drag_snapshot(object))
                    .and_then(|snapshot| snapshot.complete_listener) {
                    self.run_listener(&args, &context)?;
                }
                let source = source_object.as_ref().and_then(|object| self.interfaces.drag_snapshot(object));
                let target = target_object.as_ref().and_then(|object| self.interfaces.drag_snapshot(object));
                if let (Some(source), Some(target)) = (source, target) {
                    if self.interfaces.retained_drag_ancestor(&source).is_some() {
                        self.outbound.push_back(crate::interface_action::drag(source.identity, target.identity));
                    }
                }
            }
        }
        if matches!(frame.outcome, 1 | 3) {
            return Ok(Some((frame.mouse, drag.entries)));
        }
        if drag.motion.active {
            self.widget_drag = Some(drag);
        }
        Ok(None)
    }
    pub fn perform_default_menu(
        &mut self,
        point: [i32; 2],
        entries: Vec<crate::context_menu::Entry<crate::context_menu::Action>>,
        font: &crate::font::Font,
    ) -> Result<()> {
        self.perform_default_menu_with_icons(point,entries,font,Default::default())
    }
    pub fn perform_default_menu_with_icons(
        &mut self,
        point: [i32; 2],
        entries: Vec<crate::context_menu::Entry<crate::context_menu::Action>>,
        font: &crate::font::Font,
        icons: std::sync::Arc<Vec<crate::sprite::SpriteSheet>>,
    ) -> Result<()> {
        if self.default_menu_opens(&entries) {
            let mut choices = vec![crate::context_menu::Entry {
                option: "Cancel".into(),
                target: String::new(),
                action: crate::context_menu::Action::Cancel,
                submenu: false,
                children: Vec::new(),
            }];
            choices.extend(entries);
            self.prepare_scene_menu(point);
            self.menu = Some(crate::context_menu::Menu::open_with_icons(
                choices,
                point,
                self.canvas,
                font,
                icons,
            ));
        } else if let Some(entry) = entries.last() {
            self.menu_action_at(entry.action,point)?;
        }
        Ok(())
    }
    pub fn default_menu_opens(
        &self,
        entries: &[crate::context_menu::Entry<crate::context_menu::Action>],
    ) -> bool {
        let (opcode, forced) = entries
            .last()
            .map_or((0, false), |entry| match entry.action {
                crate::context_menu::Action::Interface { id, operation, .. } => {
                    let high =
                        operation as i32 - 1 > self.interfaces.widgets[&id].operation_priority;
                    (
                        if high { 1007 } else { 57 },
                        !high
                            && self
                                .interfaces
                                .runtime
                                .get(&id)
                                .is_some_and(|r| r.prioritize_menu),
                    )
                }
                _ => (0, false),
            });
        crate::menu_settings::opens_on_left(
            self.menu_settings.left_click_menu,
            entries.len() + 1,
            opcode,
            forced,
        )
    }
    pub fn interface_menu(&self, point: [i32; 2]) -> Result<Vec<crate::context_menu::Entry<crate::context_menu::Action>>> {
        self.interface_menu_at(point, point)
    }
    /// Native menu construction keeps the press location separate from the current pointer.
    pub fn interface_menu_at(
        &self,
        point: [i32; 2],
        pointer: [i32; 2],
    ) -> Result<Vec<crate::context_menu::Entry<crate::context_menu::Action>>> {
        self.interface_menu_with_world(point, pointer, &[], &[])
    }
    /// Merge scene actions at the viewport's traversal position before final priority sorting.
    pub fn interface_menu_with_world(
        &self,
        point: [i32; 2],
        pointer: [i32; 2],
        world: &[crate::context_menu::Entry<crate::context_menu::Action>],
        low_priority: &[crate::context_menu::Action],
    ) -> Result<Vec<crate::context_menu::Entry<crate::context_menu::Action>>> {
        self.interface_menu_with_world_groups(point, pointer, world, low_priority, &[])
    }
    pub fn interface_menu_with_world_groups(
        &self,
        point: [i32; 2],
        pointer: [i32; 2],
        world: &[crate::context_menu::Entry<crate::context_menu::Action>],
        low_priority: &[crate::context_menu::Action],
        actors: &[crate::scene_menu::ActorContribution],
    ) -> Result<Vec<crate::context_menu::Entry<crate::context_menu::Action>>> {
        use crate::context_menu::{Action, Entry};
        let layout = self.interfaces.layout(self.canvas)?;
        let mut entries = Vec::new();
        for id in crate::interface_menu_order::widgets(&self.interfaces, &layout, point, pointer) {
            let w = &self.interfaces.widgets[&id];
            // ps.ap routes viewport/minimap content without ordinary widget actions.
            if w.content == 1337 { crate::scene_menu::append(&mut entries, world, actors); continue; }
            if w.content == 1338 { continue; }
            if w.content == 1400 {
                let placed=&layout.widgets[&id];
                for entry in self.world_map.menu([placed.absolute[0],placed.absolute[1],placed.geometry.size[0],placed.geometry.size[1]],point)? {
                    entries.push(Entry { option:entry.option,target:entry.target,action:Action::Map{opcode:entry.opcode,event:entry.event},submenu:false,children:Vec::new() });
                }
            }
            if w.content == 1400 { continue; }
            let legacy = match w.button {
                1 | 4 | 5 => Some((
                    w.button_text.clone(),
                    String::new(),
                    Action::Button { id, kind: w.button },
                )),
                2 if self.selected_spell.is_none()
                    && self.interfaces.interaction_flags(id) >> 11 & 63 != 0
                    && crate::menu_label::present(&w.spell_action) =>
                {
                    Some((
                        w.spell_action.clone(),
                        format!("<col=ff00>{}", w.spell_name),
                        Action::SelectSpell(id),
                    ))
                }
                3 => Some(("Close".into(), String::new(), Action::Close)),
                6 if self.interfaces.continue_widget.is_none() => {
                    Some((w.button_text.clone(), String::new(), Action::Continue(id)))
                }
                _ => None,
            };
            if let Some((option, target, action)) = legacy {
                entries.push(Entry {
                    option,
                    target,
                    action,
                    submenu: false,
                    children: Vec::new(),
                });
            }
            if w.if3 && self.selected_spell.is_some() {
                let selection = self.selected_spell.as_ref().unwrap();
                if self.interfaces.interaction_flags(id) & (1 << 21) != 0
                    && selection.mask & 32 != 0
                {
                    entries.push(Entry {
                        option: selection.option.clone(),
                        target: format!("{} -> {}", selection.target, w.data_text),
                        action: Action::CastInterface(id),
                        submenu: false,
                        children: Vec::new(),
                    });
                }
            } else if w.if3 {
                let mask = self.interfaces.operation_mask(id);
                let listener = self.interfaces.listener(id, 1409).is_some();
                for i in (0..32).rev() {
                    if i as i32 == w.operation_priority
                        && self.interfaces.interaction_flags(id) >> 11 & 63 != 0
                        && crate::menu_label::present(&w.spell_action)
                    {
                        entries.push(Entry {
                            option: w.spell_action.clone(),
                            target: w.data_text.clone(),
                            action: Action::SelectSpell(id),
                            submenu: false,
                            children: Vec::new(),
                        });
                    }
                    let Some(option) = w.actions.get(i) else {
                        continue;
                    };
                    if crate::menu_label::present(option) && (listener || mask & (1 << i) != 0) {
                        entries.push(Entry {
                            option: option.clone(),
                            target: w.data_text.clone(),
                            action: Action::Interface {
                                id,
                                operation: (i + 1) as u8,
                                sub: None,
                            },
                            submenu: false,
                            children: w
                                .sub_actions
                                .get(i)
                                .and_then(|row| row.as_ref())
                                .map(|row| {
                                    row.iter()
                                        .enumerate()
                                        .rev()
                                        .filter(|(_, s)| !s.is_empty())
                                        .map(|(sub, text)| Entry {
                                            option: text.clone(),
                                            target: String::new(),
                                            action: Action::Interface {
                                                id,
                                                operation: (i + 1) as u8,
                                                sub: Some(sub as u8),
                                            },
                                            submenu: false,
                                            children: Vec::new(),
                                        })
                                        .collect()
                                })
                                .unwrap_or_default(),
                        });
                    }
                }
            }
            if w.if3
                && self.selected_spell.is_none()
                && self.interfaces.interaction_flags(id) & 1 != 0
            {
                entries.push(Entry {
                    option: "Continue".into(),
                    target: String::new(),
                    action: Action::Continue(id),
                    submenu: false,
                    children: Vec::new(),
                });
            }
        }
        crate::menu_limits::limit_main(&mut entries);
        self.prioritize_menu(&mut entries, low_priority);
        crate::menu_limits::limit_submenus(&mut entries);
        Ok(entries)
    }
    pub fn prioritize_menu(
        &self,
        entries: &mut [crate::context_menu::Entry<crate::context_menu::Action>],
        low_priority: &[crate::context_menu::Action],
    ) {
        crate::menu_priority::sort(entries, &self.interfaces, low_priority);
    }
    pub fn clear_spell(&mut self) -> Result<()> {
        if let Some(selection) = self.selected_spell.clone() {
            // ft resolves ll.cq at cancellation time: inventory rebuilds can replace
            // the runtime object without changing the selected logical child.
            let listener = self.interfaces.script_child(selection.logical, selection.child)
                .and_then(|id| self.interfaces.listener(id, 1406));
            if let Some((args, _)) = listener {
                if let Err(error) = self.run_listener(
                    &args,
                    &crate::script_event::Context {
                        widget: Some((selection.logical, selection.child)),
                        ..Default::default()
                    },
                ) {
                    // Original script failures are reported; ft still clears nc/nm.
                    if self.script_failures.len() == 64 { self.script_failures.pop_front(); }
                    self.script_failures.push_back(error);
                }
            }
            self.selected_spell = None;
        }
        Ok(())
    }
    pub fn select_spell(&mut self, id: u32) -> Result<()> {
        let Some(widget) = self.interfaces.widgets.get(&id).cloned() else {
            return Ok(());
        };
        let Some(runtime) = self.interfaces.runtime.get(&id).cloned() else {
            return Ok(());
        };
        self.clear_spell()?;
        let selection = crate::spell_action::Selection::from_widget(
            id,
            runtime.logical_id,
            runtime.child_index,
            runtime.item,
            self.interfaces.interaction_flags(id),
            &widget,
        );
        if let Some((args, _)) = self.interfaces.listener(id, 1416) {
            if let Err(error) = self.run_listener(
                &args,
                &crate::script_event::Context {
                    widget: Some((selection.logical, selection.child)),
                    ..Default::default()
                },
            ) {
                // az.ff selects after ji.ag, even when the script reports a failure.
                if self.script_failures.len() == 64 { self.script_failures.pop_front(); }
                self.script_failures.push_back(error);
            }
        }
        self.selected_spell = Some(selection);
        Ok(())
    }
    pub fn view_entity(&self, view: u16) -> Option<&crate::world_entity::Entity> {
        crate::world_entity::registered_entity(view, &self.world_entities, &self.detached_world_entities)
    }
    pub fn view_npc(&self, view: u16, index: u16) -> Option<&crate::npc_info::Npc> {
        if view == 0 { self.npc_state.npcs.get(&index) }
        else { self.view_entity(view)?.npcs.npcs.get(&index) }
    }
    /// Original GameClient.xq / hr.fp, including the entity config's gates.
    pub fn scene_action_allowed(&self, action: crate::context_menu::Action) -> Result<bool> {
        let (view, action) = action.scene();
        let local = self.player_state.as_ref().map_or(0, |s| s.local_view) == view;
        // GameClient.xq resolves the entity from the root world's table.
        // Protocol/camera lookup through view_entity retains registered-ID semantics.
        let entity = if view == 0 { None } else {
            self.world_entities.get(&0).and_then(|world| world.entities.get(&view))
        };
        if !local {
            if let Some(entity) = entity {
                let Some(def) = &entity.definition else { return Ok(false); };
                if !crate::interaction_policy::entity_action(def.collision_type, local, action) {
                    return Ok(false);
                }
            }
        }
        self.interaction_policy.action(view as i16, action, entity.is_some_and(|e| e.owner_type != 0), local)
    }
    pub fn prepare_scene_menu(&mut self, _point: [i32; 2]) {
        self.scene_walk_commands.push_back(crate::scene_walk::Command::Arm {instance:(0,0),walk:false});
        for world in self.world_entities.values().chain(self.detached_world_entities.iter()) {
            for (&id, entity) in &world.entities {
                if entity.registered {self.scene_walk_commands.push_back(crate::scene_walk::Command::Arm {
                    instance:(id,entity.spawn_generation),walk:false});}
            }
        }
    }
    pub fn menu_action(&mut self, action: crate::context_menu::Action) -> Result<()> {
        let point=if self.input.pressed_buttons!=0 {self.input.press_position}else{self.input.pointer};
        self.menu_action_context(action,false,point)
    }
    pub fn menu_action_at(&mut self, action: crate::context_menu::Action, point: [i32; 2]) -> Result<()> {
        self.menu_action_context(action,false,point)
    }
    pub fn menu_action_from_open_menu(&mut self, action: crate::context_menu::Action) -> Result<()> {
        self.menu_action_context(action,true,self.input.press_position)
    }
    fn menu_action_context(&mut self, action: crate::context_menu::Action, menu_open: bool, point: [i32;2]) -> Result<()> {
        use crate::{context_menu::Action, world_action};
        if !self.scene_action_allowed(action)? { return Ok(()); }
        let (view, action) = action.scene();
        // GameFloatProjection.fu resolves the registered view after policy filtering.
        // A missing root entity does not itself reject the policy check.
        if view != 0 && self.view_entity(view).is_none() { return Ok(()); }
        let control = self.input.held.contains(&82);
        let packet = match action {
            Action::Map { opcode, event } => {
                if !(1008..=1012).contains(&opcode) { return Err(Error::Invalid("map menu opcode")); }
                self.run_map_event(opcode as i32 - 998, event, None)?;
                return self.clear_spell();
            }
            Action::InView { .. } => unreachable!("scene() unwraps world actions"),
            Action::WorldEntity { id, operation } => {
                world_action::world_entity(operation, id, control)?
            }
            Action::ExamineNpc(index) => {
                // Java sets the cross before resolving even a vanished NPC/morph.
                self.click_cross.show(crate::click_cross::Kind::Action, point);
                let Some(npc) = self.view_npc(view, index) else {
                    return self.clear_spell();
                };
                let Some(def) = self.npcs.get(&(npc.id as u32)) else {
                    return self.clear_spell();
                };
                let bit = if let Some(id) = def.transform_varbit {
                    use crate::script_runtime::Source;
                    let bit = self.scripts.source.varbit(id as i32)?;
                    bit.get(self.scripts.variables.player(bit.base))
                } else {
                    -1
                };
                let varp = def
                    .transform_varp
                    .map_or(-1, |id| self.scripts.variables.player(id));
                let Some(id) = def.transformed_id(bit, varp) else {
                    return self.clear_spell();
                };
                world_action::examine(0, id as u16, [0; 2])?
            }
            Action::Examine { kind, id, tile } => world_action::examine(kind, id, tile)?,
            Action::Player { index, operation } => {
                let Some(player) = self
                    .player_state
                    .as_ref()
                    .and_then(|s| s.view_actor(view, index))
                else {
                    return self.clear_spell();
                };
                self.map_flag =
                    Some(((player.position[1] as u32) << 14) | player.position[2] as u32);
                world_action::player(operation, index, control)?
            }
            Action::Continue(id) => {
                if self.interfaces.continue_widget.is_none() {
                    if let Some(runtime) = self.interfaces.runtime.get(&id) {
                        self.outbound.push_back(crate::interface_action::resume(
                            runtime.logical_id,
                            runtime.child_index,
                        ));
                        self.interfaces.set_continue_widget(Some(id));
                    }
                }
                return self.clear_spell();
            }
            Action::Close => {
                self.close_interfaces()?;
                return self.clear_spell();
            }
            Action::Button { id, kind } => {
                let Some(widget) = self.interfaces.widgets.get(&id).cloned() else {
                    return Ok(());
                };
                let logical = self.interfaces.runtime[&id].logical_id;
                if kind == 1 && widget.content > 0 {
                    if !self.content_action(widget.content)? {
                        return self.clear_spell();
                    }
                }
                self.outbound
                    .push_back(crate::interface_action::button(logical));
                if matches!(kind, 4 | 5) {
                    if let Some(expression) =
                        widget.cs1.first().filter(|v| v.len() > 1 && v[0] == 5)
                    {
                        let varp = expression[1] as u16;
                        let current = self.scripts.variables.player(varp);
                        let next = if kind == 4 {
                            1i32.wrapping_sub(current)
                        } else {
                            widget.comparisons.first().map_or(current, |v| v.1 as i32)
                        };
                        if kind == 4 || current != next {
                            self.scripts.variables.set_player(varp, next);
                            self.apply_menu_varp(varp);
                        }
                    }
                }
                return self.clear_spell();
            }
            Action::SelectSpell(id) => return self.select_spell(id),
            Action::CastWorld { kind, id, tile } => {
                if (kind == 1 && self.view_npc(view, id).is_none())
                    || (kind == 2
                        && !self.player_state.as_ref().is_some_and(|s| {
                            s.view_actor(view, id).is_some()
                        }))
                {
                    return self.clear_spell();
                }
                let Some(selection) = &self.selected_spell else {
                    return Ok(());
                };
                self.map_flag = Some((tile[0] as u32) << 14 | tile[1] as u32);
                crate::spell_action::world(kind, id, tile, control, selection.source())?
            }
            Action::CastInterface(id) => {
                let Some(selection) = self.selected_spell.clone() else {
                    return Ok(());
                };
                let Some(runtime) = self.interfaces.runtime.get(&id) else {
                    return self.clear_spell();
                };
                let (logical, child, item) =
                    (runtime.logical_id, runtime.child_index, runtime.item);
                if let Some((args, _)) = self.interfaces.listener(id, 1435) {
                    if let Err(error) = self.run_listener(
                        &args,
                        &crate::script_event::Context {
                            widget: Some((logical, child)),
                            operation_base: Some(self.interfaces.widgets[&id].data_text.clone()),
                            ..Default::default()
                        },
                    ) {
                        // GameFloatProjection.fu invokes fx through ji.ag before
                        // sending opcode 58; a script failure does not abort it.
                        if self.script_failures.len() == 64 { self.script_failures.pop_front(); }
                        self.script_failures.push_back(error);
                    }
                }
                crate::spell_action::interface(
                    selection.source(),
                    logical,
                    child as u16,
                    item as u16,
                )
            }
            Action::Cancel => return self.clear_spell(),
            Action::Interface { id, operation, sub } => {
                self.interface_operation(id, operation, sub)?;
                return self.clear_spell();
            }
            Action::Npc {
                index,
                operation,
                sub,
            } => {
                let Some(npc) = self.view_npc(view, index) else {
                    return self.clear_spell();
                };
                self.map_flag = Some(((npc.position[0] as u32) << 14) | npc.position[1] as u32);
                world_action::npc(operation, index, sub, control)?
            }
            Action::Ground {
                id,
                tile,
                operation,
                sub,
            } => {
                self.map_flag = Some(((tile[0] as u32) << 14) | tile[1] as u32);
                world_action::ground_item(operation, id, tile[0], tile[1], sub, control)?
            }
            Action::Object {
                id,
                tile,
                operation,
                sub,
            } => {
                self.map_flag = Some(((tile[0] as u32) << 14) | tile[1] as u32);
                world_action::object(operation, id, tile[0], tile[1], sub, control)?
            }
            Action::Heading(direction) => {
                let packet = self.steering.request(direction)?;
                self.click_cross.show(crate::click_cross::Kind::Walk, point);
                packet
            }
            Action::Walk(_) => {
                let instance=(view,if view==0 {0}else{self.view_entity(view).unwrap().spawn_generation});
                self.scene_walk_commands.push_back(if menu_open {
                    crate::scene_walk::Command::Accept {instance}
                } else {crate::scene_walk::Command::Arm {instance,walk:true}});
                return self.clear_spell();
            }
        };
        if matches!(action, Action::WorldEntity { .. } | Action::Examine { .. }
            | Action::Player { .. } | Action::Npc { .. } | Action::Ground { .. }
            | Action::Object { .. } | Action::CastWorld { .. }) {
            self.click_cross.show(crate::click_cross::Kind::Action, point);
        }
        self.outbound.push_back(packet);
        self.clear_spell()
    }
    pub fn local_fine(&self) -> [i32; 2] {
        self.player_state
            .as_ref()
            .and_then(|s| s.root_actor(s.local))
            .map_or(
                [
                    self.data.position[1] * 128 + 64,
                    self.data.position[2] * 128 + 64,
                ],
                |p| p.path.position,
            )
    }
    /// ka.ec plus oe.bj: minimap tracks the selected view's local actor.
    pub fn local_minimap_fine(&self) -> [i32; 2] {
        let Some(state) = &self.player_state else { return self.local_fine(); };
        let Some(player) = state.local_actor() else { return self.local_fine(); };
        if state.local_view == 0 { return player.path.position; }
        let Some(view) = state.local_view_state() else { return self.local_fine(); };
        let local = [player.path.position[0] - view.base[0]*128,
            player.path.position[1] - view.base[1]*128];
        let projected = self.world_entities.get(&0).and_then(|w| w.entities.get(&state.local_view))
            .map(|e| { let p=e.transform_horizontal(local); [p[0] as i32,p[2] as i32] })
            .unwrap_or(local);
        [projected[0] + self.world_base[0]*128,projected[1] + self.world_base[1]*128]
    }
    /// Original po.ez uses selected-view actor projection, then subtracts the root base.
    pub fn local_map_coordinate(&self) -> [i32; 3] {
        let actor = self.player_state.as_ref().and_then(|state| {
            let player = state.local_actor()?;
            let base = if state.local_view == 0 { self.world_base } else { state.local_view_state()?.base };
            Some(crate::camera_focus::Focus { view: state.local_view,
                fine: std::array::from_fn(|i| player.path.position[i].wrapping_sub(base[i].wrapping_mul(128))),
                plane: player.position[0] as u8 })
        });
        crate::map_local::coordinate(self.world_base, actor, self.world_entities.get(&0))
    }
    pub fn camera_focus_local(&self) -> Option<crate::camera_focus::Focus> {
        let (kind, index) = match self.camera.follow_target {
            Some(target) => target,
            None => match &self.player_state {
                Some(state) => (0, state.local as i32),
                None => return Some(crate::camera_focus::Focus { view: 0,
                    fine: [self.local_fine()[0] - self.world_base[0]*128, self.local_fine()[1] - self.world_base[1]*128],
                    plane: self.data.position[0] as u8 }),
            },
        };
        if kind == 0 {
            return crate::camera_focus::resolve_registered_player(self.world_base,
                &self.view_registry, self.player_state.as_ref()?, &self.world_entities,
                &self.detached_world_entities, u16::try_from(index).ok()?);
        }
        if kind == 1 {
            return crate::camera_focus::resolve_registered_npc(self.world_base,
                self.data.position[0] as u8, &self.view_registry, &self.npc_state,
                &self.world_entities, &self.detached_world_entities, u16::try_from(index).ok()?);
        }
        if kind == 3 {
            return Some(crate::camera_focus::resolve_registered_tile(self.world_base,
                &self.view_registry, &self.world_entities, &self.detached_world_entities, index));
        }
        let views = self.view_registry.ordered().into_iter().filter_map(|id| {
            if id == 0 {
                Some(crate::projectile_world::View { id, base: self.world_base, dimensions: [104; 2] })
            } else if let Some(view) = self.player_state.as_ref().and_then(|state| state.views.get(&id)) {
                Some(crate::projectile_world::View { id, base: view.base, dimensions: view.dimensions })
            } else {
                let entity = self.world_entities.values().chain(self.detached_world_entities.iter()).find_map(|world| world.entities.get(&id))?;
                Some(crate::projectile_world::View { id,
                    base: entity.rebuild.as_ref().map_or([0; 2], |r| r.base),
                    dimensions: entity.dimensions })
            }
        }).collect::<Vec<_>>();
        crate::camera_focus::resolve_local(&views, self.world_entities.get(&0),
            self.data.position[0] as u8, kind, index, |view, kind, index| {
                let base = views.iter().find(|v| v.id == view)?.base;
                let (position, plane) = if kind == 0 {
                    let player = self.player_state.as_ref()?.view_actor(view, index)?;
                    (player.path.position, player.position[0] as u8)
                } else if view == 0 {
                    (self.npc_state.npcs.get(&index)?.path.position, self.data.position[0] as u8)
                } else {
                    let entity = self.world_entities.values().chain(self.detached_world_entities.iter()).find_map(|w| w.entities.get(&view))?;
                    (entity.npcs.npcs.get(&index)?.path.position, entity.plane)
                };
                Some(([position[0] - base[0]*128, position[1] - base[1]*128], plane))
            })
    }
    pub fn camera_focus(&self) -> Option<([i32; 2], usize)> {
        let focus = self.camera_focus_local()?.projected(self.world_entities.get(&0));
        Some(([focus.fine[0] + self.world_base[0]*128, focus.fine[1] + self.world_base[1]*128], focus.plane as usize))
    }
    pub(crate) fn enqueue_sequence_sounds(&mut self, sound_events: Vec<(u16,[i32;2],bool,u32,usize)>) -> Result<()> {
        if sound_events.is_empty() {return Ok(());}
        let sequences = self.interface_sequences.as_ref().unwrap();
        for (world, fine, local, id, frame) in sound_events {
            let sounds = &sequences[&id].sounds[&(frame as u16)];
            let draw = if sounds.len() > 1 {
                let mut bytes = [0; 4];
                loop {
                    getrandom::getrandom(&mut bytes)
                        .map_err(|e| Error::Io(format!("animation sound random: {e}")))?;
                    let value = u32::from_ne_bytes(bytes);
                    if value < u32::MAX - u32::MAX % 100 {
                        break value % 100 + 1;
                    }
                }
            } else {
                1
            };
            let Some(sound) = crate::sequence_sound::select(sounds, draw) else {
                continue;
            };
            if let Some(request) =
                crate::sequence_sound::request(sound, local, fine, self.world_base)
            {
                self.audio
                    .request_world_sequence(world, sequences[&id].flag19, request, &self.preferences.device)?;
            }
        }
        Ok(())
    }
    /// Advance root graphics before planning late child admission. Sound requests
    /// are enqueued after early children, matching pj.cb's observable phase order.
    pub(crate) fn advance_root_graphics(&mut self, ticks: u32) -> Result<Vec<(u16,[i32;2],bool,u32,usize)>> {
        if self.world_spots.is_empty() {return Ok(Vec::new());}
        if self.interface_sequences.is_none() {
            self.interface_sequences=Some(crate::animation::Sequence::load_all(&mut self.scripts.source.cache)?);
        }
        Ok(crate::world_spot::draw(&mut self.world_spots,self.data.position[0],
            self.actor_cycle.wrapping_sub(1),ticks,true,false,&self.spots,self.interface_sequences.as_ref().unwrap())?
            .into_iter().map(|(fine,id,frame)|(0,fine,false,id,frame)).collect())
    }
    /// Execute a root graphics draw phase for callers assembling scene passes directly.
    pub fn draw_root_graphics(&mut self, ticks: u32) -> Result<()> {
        let events=self.advance_root_graphics(ticks)?;
        self.enqueue_sequence_sounds(events)
    }
    /// Draw-phase child effects; skipped scenes retain their clocks without catch-up.
    pub fn draw_child_graphics(&mut self, world: u16, ticks: u32, admitted: bool, hidden: bool) -> Result<()> {
        if !admitted || hidden { return Ok(()); }
        if self.interface_sequences.is_none() {
            self.interface_sequences=Some(crate::animation::Sequence::load_all(&mut self.scripts.source.cache)?);
        }
        let Some(entity)=self.world_entities.values_mut().find_map(|w|w.entities.get_mut(&world)) else { return Ok(()); };
        let base=entity.rebuild.as_ref().map_or([0;2],|r|r.base);
        let events=crate::world_spot::draw(&mut entity.world_spots,entity.plane as i32,
            self.actor_cycle.wrapping_sub(1),ticks,admitted,hidden,&self.spots,self.interface_sequences.as_ref().unwrap())?;
        let events=events.into_iter().map(|(fine,id,frame)| (world,[
            fine[0].wrapping_sub(base[0].wrapping_mul(128)).wrapping_add(self.world_base[0].wrapping_mul(128)),
            fine[1].wrapping_sub(base[1].wrapping_mul(128)).wrapping_add(self.world_base[1].wrapping_mul(128)),
        ],false,id,frame)).collect();
        self.enqueue_sequence_sounds(events)
    }
    pub fn tick_actor_sequences(&mut self) -> Result<()> {
        if self.interface_sequences.is_none() {
            if self.player_state.is_none()
                && self.npc_state.npcs.is_empty()
                && self
                    .world_entities
                    .values()
                    .chain(self.detached_world_entities.iter())
                    .all(|world| world.entities.is_empty())
                && self.world_spots.is_empty()
                && self.projectiles.is_empty()
                && self.scripted_projectiles.effects.is_empty()
                && self.scripted_events.is_empty()
            {
                return Ok(());
            }
            self.interface_sequences = Some(crate::animation::Sequence::load_all(
                &mut self.scripts.source.cache,
            )?);
        }
        let sequences = self.interface_sequences.as_ref().unwrap();
        let mut ready=|id:i32| sequences.get(&(id as u32)).is_some_and(|sequence|
            self.animation_readiness.ready(&mut self.scripts.source.cache,sequence));
        let local_player = self.player_state.as_ref().map(|state| state.local);
        let high = self.player_state.as_ref().map_or_else(Vec::new, |state| state.high.clone());
        let root_base = self.world_base;
        let mut views = self.view_registry.ordered();
        // Retain support for hand-assembled hosts without registered view metadata.
        let mut additional = self.world_entities.values()
            .chain(self.detached_world_entities.iter())
            .flat_map(|world| world.entities.iter().filter(|(_, entity)| entity.registered).map(|(&id, _)| id))
            .filter(|id| !views.contains(id)).collect::<Vec<_>>();
        additional.sort_unstable(); additional.dedup(); views.extend(additional);
        let mut sound_events = Vec::new();
        let mut capture = |view: u16, generation: u64, base: [i32;2], events: &mut std::collections::VecDeque<(bool,u16,u32,usize)>, npcs: &crate::npc_info::State, players: Option<&crate::player_info::State>| {
            while let Some((player,index,id,frame)) = events.pop_front() {
                let fine = if player {
                    let Some(actor) = players.and_then(|p| p.instance_actor(view,generation,index)) else { continue; };
                    actor.path.position
                } else {
                    let Some(actor) = npcs.npcs.get(&index) else { continue; };
                    actor.path.position
                };
                let point = std::array::from_fn(|axis| fine[axis]
                    .wrapping_sub(base[axis].wrapping_mul(128))
                    .wrapping_add(root_base[axis].wrapping_mul(128)));
                let local = local_player.is_some_and(|local| crate::sequence_sound::local_actor(index,local));
                sound_events.push((view,point,local,id,frame));
            }
        };
        // GameClient.iw visits every player instance before any NPC instance.
        let player_views=crate::player_info::instance_descriptors(&views,self.view_registry.ordered().len(),root_base,
            &self.world_entities,&self.detached_world_entities);
        for (view_id,generation,base,dimensions) in player_views {
            let (base,dimensions)=if view_id==0 {(base,dimensions)}else{self.player_state.as_ref().and_then(|s|s.instance_view(view_id,generation)).map_or((base,dimensions),|view|(view.base,view.dimensions))};
            let targets=crate::actor_target::collect_instance(crate::actor_target::Observer{view:view_id,base,generation},
                root_base,&self.view_registry,&self.npc_state,self.player_state.as_ref(),&self.world_entities,&self.detached_world_entities);
            if view_id==0 {
                if let Some(state)=&mut self.player_state {
                    let local=state.local;let players=crate::player_runtime::ordered(&high,state.root_iter_mut());
                    crate::player_runtime::tick_with_targets(players.into_iter(),&self.npcs,sequences,&self.spots,targets,
                        self.actor_cycle,base,dimensions,Some(local),&mut self.actor_events,&mut ready)?;
                }
                capture(0,0,base,&mut self.actor_events,&self.npc_state,self.player_state.as_ref());
            }else{
                let mut events=std::collections::VecDeque::new();
                if let Some(view)=self.player_state.as_mut().and_then(|s|s.instance_view_mut(view_id,generation)) {
                    let players=crate::player_runtime::ordered(&high,view.players.iter_mut().map(|(&i,p)|(i,p)));
                    crate::player_runtime::tick_with_targets(players.into_iter(),&self.npcs,sequences,&self.spots,targets,
                        self.actor_cycle,base,dimensions,None,&mut events,&mut ready)?;
                }
                capture(view_id,generation,base,&mut events,&self.npc_state,self.player_state.as_ref());
            }
        }
        // Registry nodes, not numeric IDs, determine NPC simulation order.
        // Store owner addresses so each phase mutates its original entity.
        #[derive(Clone,Copy)]
        enum NpcOwner { Root, Attached(u16), Detached(usize) }
        let registered_count=self.view_registry.ordered().len();
        let mut npc_views=Vec::new();
        for (rank,&view_id) in views.iter().enumerate(){
            let bucket=if rank<registered_count {(view_id&15) as usize}else{16};
            if view_id==0 {npc_views.push((bucket,0,rank,view_id,NpcOwner::Root));continue;}
            for (&parent,world) in &self.world_entities {
                if let Some(e)=world.entities.get(&view_id).filter(|e|e.registered){
                    npc_views.push((bucket,e.spawn_generation,rank,view_id,NpcOwner::Attached(parent)));
                }
            }
            for (parent,world) in self.detached_world_entities.iter().enumerate(){
                if let Some(e)=world.entities.get(&view_id).filter(|e|e.registered){
                    npc_views.push((bucket,e.spawn_generation,rank,view_id,NpcOwner::Detached(parent)));
                }
            }
        }
        npc_views.sort_by_key(|&(bucket,generation,rank,..)|(bucket,generation,rank));
        for (_,generation,_,view_id,owner) in npc_views {
            let (base,dimensions)=match owner{
                NpcOwner::Root=>(root_base,[104;2]),
                NpcOwner::Attached(parent)=>{let e=&self.world_entities[&parent].entities[&view_id];(e.rebuild.as_ref().map_or([0;2],|r|r.base),e.dimensions)},
                NpcOwner::Detached(parent)=>{let e=&self.detached_world_entities[parent].entities[&view_id];(e.rebuild.as_ref().map_or([0;2],|r|r.base),e.dimensions)},
            };
            // Earlier instances have already advanced; take a fresh target view.
            let targets=crate::actor_target::collect_instance(crate::actor_target::Observer{view:view_id,base,generation},
                root_base,&self.view_registry,&self.npc_state,self.player_state.as_ref(),&self.world_entities,&self.detached_world_entities);
            if matches!(owner,NpcOwner::Root){
                crate::npc_runtime::tick_in_view_with_targets(0,&mut self.npc_state,
                    &self.npcs,sequences,&self.spots,targets,self.actor_cycle,
                    base,dimensions,local_player,&mut self.actor_events,&mut ready)?;
                capture(0,0,base,&mut self.actor_events,&self.npc_state,self.player_state.as_ref());
            }else{
                let entity=match owner{
                    NpcOwner::Attached(parent)=>self.world_entities.get_mut(&parent).unwrap().entities.get_mut(&view_id).unwrap(),
                    NpcOwner::Detached(parent)=>self.detached_world_entities[parent].entities.get_mut(&view_id).unwrap(),
                    NpcOwner::Root=>unreachable!(),
                };
                let mut events=std::collections::VecDeque::new();
                crate::npc_runtime::tick_in_view_with_targets(view_id,&mut entity.npcs,
                    &self.npcs,sequences,&self.spots,targets,self.actor_cycle,
                    base,dimensions,None,&mut events,&mut ready)?;
                capture(view_id,generation,base,&mut events,&entity.npcs,self.player_state.as_ref());
            }
        }
        for (parent,id) in crate::world_entity::simulation_order(&self.view_registry,&self.world_entities) {
            let entity=self.world_entities.get_mut(&parent).unwrap().entities.get_mut(&id).unwrap();
            for (sequence,frame) in entity.tick_animations_with_readiness(sequences,&mut ready)? {
                if sequences[&sequence].sounds.contains_key(&(frame as u16)) {
                    sound_events.push((parent,[
                        entity.pose.x.wrapping_add(root_base[0].wrapping_mul(128)),
                        entity.pose.z.wrapping_add(root_base[1].wrapping_mul(128)),
                    ],false,sequence,frame));
                }
            }
        }
        for world in self.world_entities.values_mut().chain(self.detached_world_entities.iter_mut()) {
            for (&view_id, entity) in &mut world.entities {
                if !entity.registered { continue; }
                if let Some(view) = self.player_state.as_mut().and_then(|s| s.instance_view_mut(view_id,entity.spawn_generation)) {
                    for p in view.players.values_mut() { p.overlays.speech.tick(); }
                }
                for npc in entity.npcs.npcs.values_mut() { npc.overlays.speech.tick(); }

            }
        }
        self.enqueue_sequence_sounds(sound_events)?;
        for npc in self.npc_state.npcs.values_mut() {
            npc.overlays.speech.tick();
        }
        if let Some(state) = &mut self.player_state {
            for (_, player) in state.root_iter_mut() {
                player.overlays.speech.tick();
            }
        }
        self.actor_cycle = self.actor_cycle.wrapping_add(1);
        Ok(())
    }
    fn projectile_curve(&mut self, key: i32, progress: i32) -> Result<i32> {
        if let Some(script) = self.scripts.source.event_script(86, key, -1)? {
            let mut vm = Vm::new(script, vec![progress], vec![], vec![], 500000)?
                .with_int_slots(Rc::clone(&self.script_int_slots));
            loop {
                let result = self.step_vm(&mut vm, 10000);
                self.script_slot0 = vm.stacks.int_slot0();
                match result {
                    Ok(vm::Status::Finished) => break,
                    Ok(_) => {}
                    Err(error) => {
                        if self.script_failures.len() == 64 {
                            self.script_failures.pop_front();
                        }
                        self.script_failures
                            .push_back(Error::Io(format!("projectile curve {key}: {error}")));
                        break;
                    }
                }
            }
        }
        Ok(self.script_slot0)
    }
    fn projectile_endpoint(
        &self,
        scene: &crate::scene::Scene,
        children: &std::collections::HashMap<u16, &crate::scene::Scene>,
        tile: [i32; 3],
        actor: i32,
        scripted: bool,
    ) -> [f32; 3] {
        let roots = self.world_entities.get(&0);
        let resolved = crate::projectile_world::resolve_registered(
            crate::projectile_world::View { id: 0, base: self.world_base,
                dimensions: [scene.width as i32, scene.height as i32] },
            &self.view_registry, self.player_state.as_ref(), &self.npc_state,
            &self.world_entities, &self.detached_world_entities,
            [tile[1], tile[2]], actor, scripted,
        );
        let terrain = if resolved.view == 0 {
            Some(scene)
        } else {
            children.get(&resolved.view).copied()
        };
        let mut elevation = terrain
            .and_then(|s| {
                s.elevation(
                    tile[0] as usize,
                    resolved.fine[0] as i32,
                    resolved.fine[1] as i32,
                )
            })
            .unwrap_or(0);
        // tt.cd adds the root terrain under a root-owned entity to the selected
        // view's sample, which receives the already transformed coordinates.
        if let Some(entity) = roots.and_then(|r| r.entities.get(&resolved.view)) {
            elevation += scene
                .elevation(self.data.position[0] as usize, entity.pose.x, entity.pose.z)
                .unwrap_or(0);
        }
        [resolved.fine[0], elevation as f32, resolved.fine[1]]
    }
    /// Resolve creation sources at the packet boundary, before actor simulation.
    pub fn apply_scripted_projectile_events(
        &mut self,
        scene: &crate::scene::Scene,
        children: &std::collections::HashMap<u16, &crate::scene::Scene>,
    ) -> Result<()> {
        if self.scripted_events.is_empty() {return Ok(());}
        let mut points=std::collections::HashMap::new();
        for (_,event) in &self.scripted_events {
            if let crate::scripted_projectile::Event::Create {definition,..}=event {
                let endpoint=&definition.source;
                points.entry((endpoint.entity,endpoint.tile)).or_insert_with(||
                    self.projectile_endpoint(scene,children,endpoint.tile,endpoint.entity,true));
            }
        }
        let mut state=std::mem::take(&mut self.scripted_projectiles);
        let result=(|| -> Result<()> {
            while let Some((cycle,event))=self.scripted_events.pop_front() {
                state.apply(event,cycle,|endpoint| {
                    let p=points.get(&(endpoint.entity,endpoint.tile))
                        .ok_or(Error::Invalid("scripted creation endpoint snapshot"))?;
                    Ok([p[0]+endpoint.offset[0] as f32,p[1]-endpoint.offset[1] as f32,p[2]+endpoint.offset[2] as f32])
                },|spot|self.spots.get(&(spot as u32)).map(|s|s.sequence)
                    .ok_or(Error::Invalid("scripted projectile spot")))?;
            }
            Ok(())
        })();
        self.scripted_projectiles=state;
        result
    }
    fn advance_scripted_projectiles(
        &mut self,
        scene: &crate::scene::Scene,
        children: &std::collections::HashMap<u16, &crate::scene::Scene>,
        ticks: u32,
        sound_events: &mut Vec<(u16,[i32;2],bool,u32,usize)>,
    ) -> Result<()> {
        if self.scripted_projectiles.effects.is_empty() && self.scripted_events.is_empty() {
            return Ok(());
        }
        self.apply_scripted_projectile_events(scene,children)?;
        if self.interface_sequences.is_none() {
            self.interface_sequences = Some(crate::animation::Sequence::load_all(
                &mut self.scripts.source.cache,
            )?);
        }
        let mut endpoints = vec![
            &self.scripted_projectiles.definition.source,
            &self.scripted_projectiles.definition.target,
            &self.scripted_projectiles.redirect,
        ];
        for effect in self.scripted_projectiles.effects.values() {
            if let Some(pending) = &effect.motion.pending {
                endpoints.push(pending);
            }
        }
        let mut points = std::collections::HashMap::new();
        for endpoint in endpoints {
            points
                .entry((endpoint.entity, endpoint.tile))
                .or_insert_with(|| {
                    self.projectile_endpoint(scene, children, endpoint.tile, endpoint.entity, true)
                });
        }
        let point = |endpoint: &crate::scripted_projectile::Endpoint| -> Result<[f32; 3]> {
            let p = points
                .get(&(endpoint.entity, endpoint.tile))
                .ok_or(Error::Invalid("scripted projectile endpoint snapshot"))?;
            Ok([
                p[0] + endpoint.offset[0] as f32,
                p[1] - endpoint.offset[1] as f32,
                p[2] + endpoint.offset[2] as f32,
            ])
        };
        let mut state = std::mem::take(&mut self.scripted_projectiles);
        let result = (|| -> Result<()> {
            let cycle = self.actor_cycle.wrapping_sub(1);
            for key in state.ordered_keys() {
                if cycle > state.definition.end {
                    state.effects.remove(&key);
                    continue;
                }
                let effect = state.effects.get_mut(&key).unwrap();
                effect.visible = cycle >= state.definition.start;
                if !effect.visible {
                    continue;
                }
                if effect.pending_shared {
                    effect.motion.pending = Some(state.redirect.clone());
                }
                let target = point(&state.definition.target)?;
                let advanced =
                    effect
                        .motion
                        .advance(&mut state.definition, target, cycle, ticks as i32, |id, p| {
                            self.projectile_curve(id, p)
                        })?;
                effect.pending_shared = effect.motion.pending.is_some();
                if advanced && effect.sequence >= 0 {
                    let sequence = self
                        .interface_sequences
                        .as_ref()
                        .unwrap()
                        .get(&(effect.sequence as u32))
                        .ok_or(Error::Invalid("scripted projectile sequence"))?;
                    let (_, frames) = effect.clock.advance(sequence, ticks)?;
                    for frame in frames {
                        if sequence.sounds.contains_key(&(frame as u16)) {
                            sound_events.push((0, [
                                effect.motion.position[0] as i32 + self.world_base[0] * 128,
                                effect.motion.position[2] as i32 + self.world_base[1] * 128,
                            ], false, effect.sequence as u32, frame));
                        }
                    }
                }
            }
            Ok(())
        })();
        self.scripted_projectiles = state;
        result
    }
    pub fn tick_projectiles(&mut self, scene: &crate::scene::Scene) -> Result<()> {
        self.tick_projectiles_in_views(scene, &std::collections::HashMap::new())
    }
    pub fn tick_projectiles_in_views(
        &mut self,
        scene: &crate::scene::Scene,
        children: &std::collections::HashMap<u16, &crate::scene::Scene>,
    ) -> Result<()> {
        self.draw_projectiles_in_views(scene, children, 1)
    }
    /// Draw-time projectile motion; direct mesh hosts can supply the accumulated delta.
    pub fn draw_projectiles_in_views(
        &mut self,
        scene: &crate::scene::Scene,
        children: &std::collections::HashMap<u16, &crate::scene::Scene>,
        ticks: u32,
    ) -> Result<()> {
        let events=self.advance_projectiles_in_views(scene,children,ticks)?;
        self.enqueue_sequence_sounds(events)
    }
    pub(crate) fn advance_projectiles_in_views(
        &mut self,
        scene: &crate::scene::Scene,
        children: &std::collections::HashMap<u16, &crate::scene::Scene>,
        ticks: u32,
    ) -> Result<Vec<(u16,[i32;2],bool,u32,usize)>> {
        let mut events=Vec::new();
        // pj.cb: ordinary projectiles, scripted projectiles, then root graphics.
        self.advance_ordinary_projectiles(scene,children,ticks,&mut events)?;
        self.advance_scripted_projectiles(scene,children,ticks,&mut events)?;
        Ok(events)
    }
    fn advance_ordinary_projectiles(
        &mut self,
        scene: &crate::scene::Scene,
        children: &std::collections::HashMap<u16, &crate::scene::Scene>,
        ticks: u32,
        sound_events: &mut Vec<(u16,[i32;2],bool,u32,usize)>,
    ) -> Result<()> {
        if self.projectiles.is_empty() {
            return Ok(());
        }
        if self.interface_sequences.is_none() {
            self.interface_sequences = Some(crate::animation::Sequence::load_all(
                &mut self.scripts.source.cache,
            )?);
        }
        let sequences = self.interface_sequences.as_ref().unwrap();
        let cycle = self.actor_cycle.wrapping_sub(1);
        self.projectiles.retain(|p| cycle <= p.request.end);
        let points = self
            .projectiles
            .iter()
            .map(|p| {
                [
                    self.projectile_endpoint(
                        scene,
                        children,
                        p.request.source,
                        p.request.entities[0],
                        false,
                    ),
                    self.projectile_endpoint(
                        scene,
                        children,
                        p.request.target,
                        p.request.entities[1],
                        false,
                    ),
                ]
            })
            .collect::<Vec<_>>();
        let point = |p: [f32; 3], height: u16| {
            [
                p[0] as i32 + self.world_base[0] * 128,
                p[2] as i32 + self.world_base[1] * 128,
                p[1] as i32 - height as i32,
            ]
        };
        for (projectile, endpoints) in self.projectiles.iter_mut().zip(points) {
            let q = &projectile.request;
            if cycle < q.start {
                continue;
            }
            projectile.motion.advance(
                point(endpoints[0], q.heights[0]),
                point(endpoints[1], q.heights[1]),
                q.start,
                q.end,
                cycle,
                ticks as i32,
                q.slope as i32,
                q.offset as i32,
            );
            let definition = self
                .spots
                .get(&(q.id as u32))
                .ok_or(Error::Invalid("projectile spot definition"))?;
            if definition.sequence >= 0 {
                let sequence = sequences
                    .get(&(definition.sequence as u32))
                    .ok_or(Error::Invalid("projectile sequence"))?;
                let (_, frames) = projectile.clock.advance(sequence, ticks)?;
                for frame in frames {
                    if sequence.sounds.contains_key(&(frame as u16)) {
                        sound_events.push((0, [
                            projectile.motion.position[0] as i32,
                            projectile.motion.position[1] as i32,
                        ], false, definition.sequence as u32, frame));
                    }
                }
            }
        }
        Ok(())
    }
    /// Consume pending input once, then retain only held state for catch-up ticks.
    /// Restore render input and the current cycle even when a tick fails.
    pub fn tick_interfaces_elapsed(
        &mut self,
        ticks: u32,
        pending: &mut crate::input::Input,
        suppress_pointer: bool,
    ) -> Result<u64> {
        self.tick_interfaces_elapsed_with(ticks, pending, suppress_pointer, |_| Ok(()))
    }
    /// Run post-interface work before consuming this tick's input and restoring
    /// the render snapshot. GameClient's game-cycle path calls ps.am here.
    pub fn tick_interfaces_elapsed_with(
        &mut self,
        ticks: u32,
        pending: &mut crate::input::Input,
        suppress_pointer: bool,
        mut after_tick: impl FnMut(&mut Self) -> Result<()>,
    ) -> Result<u64> {
        let final_cycle = self.cycle;
        std::mem::swap(&mut self.input, pending);
        let mut total = Ok(0u64);
        for tick in 0..ticks {
            self.cycle = final_cycle.wrapping_sub(ticks as i32).wrapping_add(tick as i32 + 1);
            self.input.advance_key_events();
            let result = self.tick_interfaces(suppress_pointer || self.menu.is_some() || self.widget_drag.is_some())
                .and_then(|count| after_tick(self).map(|()| count));
            self.input.begin_cycle();
            match result {
                Ok(count) => *total.as_mut().unwrap() += count,
                Err(error) => { total = Err(error); break; }
            }
        }
        self.cycle = final_cycle;
        std::mem::swap(&mut self.input, pending);
        total
    }
    pub fn tick_interfaces(&mut self, suppress_pointer: bool) -> Result<u64> {
        #[cfg(feature="native-profiling")]
        let mut ui_profile=self.ui_profile.as_mut().and_then(|p|p.begin(self.cycle,self.interfaces.widgets.len()));
        self.remote.crm.poll()?;
        if self.friends_chat_changed {
            if let Some(c) = &mut self.friends_chat {
                c.sort_members(self.data.world)?;
            }
            self.friends_chat_changed = false;
        }
        self.chat_delivery.restricted = self.player_state.is_some()
            && crate::chat_packet::restricted_area(self.data.position[1], self.data.position[2]);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| Error::Invalid("social clock"))?
            .as_secs() as i64;
        for text in self.social.drain_notifications(now) {
            self.apply(game_packet::Update::Message(game_packet::GameMessage {
                kind: 5,
                sender: None,
                text,
            }))?;
        }
        if self.data.reboot_timer > 1 {
            self.data.reboot_timer -= 1;
        }
        self.steering.tick();
        if self.logout_timer > 0 {
            self.logout_timer -= 1;
        }
        let generation = self.scripts.variables.generation;
        let count = generation.wrapping_sub(self.observed_varps);
        if count > 32 {
            // More than a full history invalidates every trigger filter.
            for _ in 0..33 {
                self.interface_events.changed(1407, -1);
            }
        } else {
            let history = self.scripts.variables.changed().collect::<Vec<_>>();
            for &id in history
                .iter()
                .skip(history.len().saturating_sub(count as usize))
            {
                self.interface_events.changed(1407, id as i32);
            }
        }
        self.observed_varps = generation;
        let mut layout = self.interfaces.preview_layout(self.canvas)?;
        self.interfaces.prepare_editors(&mut self.scripts.source.cache,&layout)?;
        if crate::remote_url::poll_visible(&mut self.interfaces, &layout, &mut self.remote)? {
            layout=self.interfaces.layout(self.canvas)?;
        } else {
            self.interfaces.record_layout_geometry(&layout);
        }
        if self.interfaces.tick_tooltip(&layout, self.input.pointer, suppress_pointer) {
            // A legacy redirect can expose a container and its IF3 descendants this tick.
            layout = self.interfaces.layout(self.canvas)?;
        }
        #[cfg(feature="native-profiling")]
        crate::ui_profile::Sample::mark(&mut ui_profile);
        // Keyboard operations are independent of pointer/menu suppression.
        for &id in &layout.order {
            if !layout.widgets.get(&id).is_some_and(|p|!p.hidden) {continue;}
            let Some(runtime)=self.interfaces.runtime.get(&id) else{continue;};
            if !runtime.shortcuts.active() || !self.interfaces.shortcut_scope.allows(runtime.logical_id) {continue;}
            let instance=runtime.instance;
            for index in 0..11 {
                let Some(runtime)=self.interfaces.runtime.get_mut(&id).filter(|r|r.instance==instance) else{break;};
                if !runtime.shortcuts.poll(index,self.cycle,|key|self.input.held.contains(&key),|key|self.input.key_events.iter().any(|&(code,_)|code==key)){continue;}
                let result=if index==10 {self.select_spell(id)}else{self.interface_operation_with_base(id,index as u8+1,None,Some("")).map(|_|())};
                if let Err(error)=result {
                    if self.script_failures.len()==64 {self.script_failures.pop_front();}
                    self.script_failures.push_back(Error::Io(format!("widget shortcut {id}:{}: {error}",index+1)));
                    break;
                }
                if let Some(runtime)=self.interfaces.runtime.get_mut(&id).filter(|r|r.instance==instance) {
                    runtime.shortcuts.dispatched(index,self.cycle);
                }
            }
        }
        #[cfg(feature="native-profiling")]
        crate::ui_profile::Sample::mark(&mut ui_profile);
        let mut map_instructions=0;
        let map_widgets=layout.order.iter().copied().filter(|id|layout.widgets.get(id).is_some_and(|p|!p.hidden)&&self.interfaces.widgets.get(id).is_some_and(|w|w.content==1400)).collect::<Vec<_>>();
        if !map_widgets.is_empty() {
            let point=if self.input.pressed_buttons!=0 {self.input.press_position}else{self.input.pointer};
            let mut allowed=std::collections::HashSet::new();
            for id in layout.hit_test(point) {allowed.insert(id);if self.interfaces.widgets[&id].no_click_through {break;}}
            let buttons=if self.preferences.mouse_camera {1}else{5};
            let admin=self.data.staff_level>=2&&self.input.held.contains(&81)&&self.input.held.contains(&82);
            let now=crate::map_pointer::millis();
            for id in map_widgets {
                self.world_map.load(&mut self.scripts.source.cache)?;
                self.world_map.advance(self.heatmap.map.is_some());
                let placed=&layout.widgets[&id];
                let active=point[0]>=placed.clip[0]&&point[1]>=placed.clip[1]&&point[0]<placed.clip[2]&&point[1]<placed.clip[3];
                let rect=[placed.absolute[0],placed.absolute[1],placed.geometry.size[0],placed.geometry.size[1]];
                for (kind,event) in self.world_map.hover_events(active,rect,point)? {
                    match self.run_map_event(kind,event,Some(point)) {
                        Ok(count)=>map_instructions+=count,
                        Err(error)=>{if self.script_failures.len()==64{self.script_failures.pop_front();}self.script_failures.push_back(Error::Io(format!("map hover event {kind}: {error}")));}
                    }
                }
                let over=allowed.contains(&id)&&!suppress_pointer;
                if let Some(packet)=self.world_map.pointer_input(point,over&&self.input.buttons&buttons!=0,over&&self.input.pressed_buttons&buttons!=0,self.widget_drag.is_some(),admin,now){self.outbound.push_back(packet);}
            }
        }
        let pointer_events=self.capture_widget_pointer(&layout);
        let events = self.interface_events.collect(
            &self.interfaces,
            &layout,
            &self.input,
            self.preferences.mouse_camera,
            suppress_pointer,
        );
        #[cfg(feature="native-profiling")]
        crate::ui_profile::Sample::mark(&mut ui_profile);
        let instructions = self.dispatch_interface_events(events);
        #[cfg(feature="native-profiling")]
        crate::ui_profile::Sample::mark(&mut ui_profile);
        self.dispatch_widget_pointer(pointer_events)?;
        let layout = self.interfaces.layout(self.canvas)?;
        for &id in &layout.order {
            let w = &self.interfaces.widgets[&id];
            if w.cs1.is_empty() && w.comparisons.is_empty() {
                continue;
            }
            let context = crate::cs1::Context {
                data: &self.data,
                variables: &self.scripts.variables,
                inventories: &self.inventories,
                items: &self.items,
                combat: self.combat_level,
            };
            let values = (0..w.cs1.len())
                .map(|i| {
                    context.evaluate(w, i, |id| {
                        use crate::script_runtime::Source;
                        let b = self.scripts.source.varbit(id)?;
                        Ok(b.get(self.scripts.variables.player(b.base)))
                    })
                })
                .collect::<Vec<_>>();
            let runtime = self.interfaces.runtime.get_mut(&id).unwrap();
            runtime.condition = crate::cs1::condition(w, &values);
            runtime.cs1_values = values;
        }
        #[cfg(feature="native-profiling")]
        crate::ui_profile::Sample::mark(&mut ui_profile);
        if layout
            .order
            .iter()
            .any(|id| self.interfaces.widgets.get(id).is_some_and(|w| w.kind == 6))
        {
            if self.interface_sequences.is_none() {
                self.interface_sequences = Some(crate::animation::Sequence::load_all(
                    &mut self.scripts.source.cache,
                )?);
            }
            crate::interface_animation::tick(
                &mut self.interfaces,
                &layout,
                self.interface_sequences.as_ref().unwrap(),
                1,
            )?;
        }
        #[cfg(feature="native-profiling")]
        crate::ui_profile::Sample::mark(&mut ui_profile);
        #[cfg(feature="native-profiling")]
        if let Some(error)=crate::ui_profile::Profile::complete(&mut self.ui_profile,ui_profile,instructions+map_instructions) {
            eprintln!("UI profiling disabled after output error: {error}");
        }
        Ok(instructions + map_instructions)
    }
    pub fn apply(&mut self, u: game_packet::Update) -> Result<()> {
        match u {
            game_packet::Update::OpenUrl(url) => self.remote.pending_urls.push_back(url),
            game_packet::Update::SiteSettings(value) => self.site_settings = value,
            game_packet::Update::Uid192(value) => self.uid192.receive(value),
            game_packet::Update::AmbientStart { id, fade } => {
                let cache = &mut self.scripts.source.cache;
                let definition = crate::ambient_sound::Definition::load(cache, id as u32)?;
                self.audio.ambient_start(definition, fade, self.preferences.device.option22, cache)?;
            }
            game_packet::Update::AmbientStop { fade } => self.audio.ambient_stop(fade),
            game_packet::Update::Heatmap(enabled) => self.heatmap.enabled(enabled),
            game_packet::Update::Skybox(id) => self.skybox.request(id),
            game_packet::Update::GroupFull(bytes) => {
                self.groups.load_definitions(&mut self.scripts.source.cache)?;
                for entry in self.groups.decode_full(&bytes)? {
                    if let Some(event) = self.groups.removal(&entry) { self.group_event(event); }
                    if let Some(event) = self.groups.apply_full(entry) { self.group_event(event); }
                }
            }
            game_packet::Update::GroupVariables(deltas) => {
                for delta in deltas { self.groups.apply_delta(delta)?; }
            }
            game_packet::Update::InteractionPolicy(update) => self.interaction_policy.apply(update),
            game_packet::Update::ServerString(text) => self.data.server_string = text,
            game_packet::Update::MarketOffer { slot, offer } => {
                *self.market.offers.get_mut(slot as usize).ok_or(Error::Invalid("exchange offer slot"))? = offer;
                self.interface_events.changed(1425, 0);
            }
            game_packet::Update::TradingPost(snapshot) => {
                self.market.replace(snapshot, crate::market::now());
                self.interface_events.changed(1426, 0);
            }
            game_packet::Update::Logout(reason) => self.logout_request = Some(reason),
            game_packet::Update::WorldHop(target) => self.world_hop = Some(target),
            game_packet::Update::Music(request) => request.apply(&mut self.audio.music,self.preferences.device.get_game(7)?),
            game_packet::Update::Camera(update) => self.camera.apply(update),
            game_packet::Update::Ping(challenge) => {
                // Native Rust has no Java garbage collector pause time to report.
                self.outbound.push_back(crate::client_metrics::ping_reply(
                    challenge,
                    self.frame_rate.fps,
                    0,
                ));
            }
            game_packet::Update::HideOperations { kind, hidden } => {
                self.hidden_operations[kind as usize] = hidden
            }
            game_packet::Update::PlayerOperation {
                slot,
                low_priority,
                label,
            } => {
                if (1..=8).contains(&slot) {
                    self.player_operations[slot as usize - 1] = label;
                    self.player_operation_priority[slot as usize - 1] = low_priority;
                }
            }

            game_packet::Update::ChatMessage(packet) => {
                if self.huffman.is_none() {
                    let lengths = self.scripts.source.cache.named_file(10, "huffman", "")?;
                    self.huffman = Some(crate::huffman::Huffman::new(&lengths)?);
                }
                if let Some(message) = self.chat_delivery.receive(
                    packet,
                    &self.social,
                    &self.clan_channels,
                    self.huffman.as_ref().unwrap(),
                )? {
                    self.chat.add(
                        message.kind as i32,
                        self.cycle,
                        message.sender.clone(),
                        message.prefix,
                        message.text.clone(),
                    );
                    self.chat.refresh_social(&self.social);
                    self.interface_events.changed(1418, 0);
                    if self.game_messages.len() == 100 {
                        self.game_messages.pop_front();
                    }
                    self.game_messages.push_back(game_packet::GameMessage {
                        kind: message.kind,
                        sender: Some(message.sender),
                        text: message.text,
                    });
                }
            }
            game_packet::Update::ClanProfileReset(present) => self.clan_profile.reset(present),
            game_packet::Update::ClanProfileValue(payload) => {
                self.clan_profile
                    .update(&payload, &mut self.scripts.source.cache)?;
            }
            game_packet::Update::ClanSettingsDelta { slot, delta } => {
                if let Some(packet) = self.clans.delta(slot, delta)? {
                    self.outbound.push_back(packet);
                }
                self.interface_events.changed(1428, 0);
            }
            game_packet::Update::ClanChannelDelta { slot, delta } => {
                if let Some(packet) = self.clan_channels.delta(slot, delta)? {
                    self.outbound.push_back(packet);
                }
                self.interface_events.changed(1429, 0);
            }
            game_packet::Update::ClanChannel { slot, channel } => {
                self.clan_channels.apply(slot, channel)?;
                self.interface_events.changed(1429, 0);
            }
            game_packet::Update::ClanSettings { slot, settings } => {
                self.clans.apply(slot, settings)?;
                self.interface_events.changed(1428, 0);
            }
            game_packet::Update::FriendsChat { snapshot, payload } => {
                let local = self
                    .player_state
                    .as_ref()
                    .and_then(|s| s.root_actor(s.local))
                    .and_then(|p| p.appearance.as_ref())
                    .map_or("", |a| a.name.as_str());
                if snapshot && payload.is_empty() {
                    self.friends_chat = None;
                } else if snapshot {
                    self.friends_chat
                        .get_or_insert_with(Default::default)
                        .snapshot(&payload, local)?;
                } else if let Some(channel) = &mut self.friends_chat {
                    channel.delta(&payload, local)?;
                }
                self.friends_chat_changed = true;
                self.interface_events.changed(1421, 0);
            }
            game_packet::Update::FriendLoading => {
                self.social.state = 1;
                self.interface_events.changed(1420, 0);
            }
            game_packet::Update::Friends(updates) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|_| Error::Invalid("social clock"))?
                    .as_secs() as i64;
                self.social.apply_friends(updates, self.data.world, now)?;
                self.chat.refresh_social(&self.social);
                self.interface_events.changed(1420, 0);
            }
            game_packet::Update::Ignores(updates) => {
                self.social.apply_ignores(updates)?;
                self.chat.refresh_social(&self.social);
                self.interface_events.changed(1420, 0);
            }
            game_packet::Update::SynthSound { id, loops, delay } => {
                self.audio.request(
                    id as u32,
                    loops as i32,
                    delay as i32,
                    self.preferences.device.get_game(8)?,
                );
            }

            game_packet::Update::ChatFilters { public, trade } => {
                self.chat.public_filter = public as i32;
                self.chat.trade_filter = trade as i32;
            }
            game_packet::Update::PrivateChatFilter(value) => {
                self.chat.private_filter = (value <= 2).then_some(value as i32)
            }
            game_packet::Update::ActiveWorld { id, plane } => {
                if id != 0 {
                    self.world_entities
                        .get_mut(&0)
                        .and_then(|world| world.entities.get_mut(&id))
                        .ok_or(Error::Invalid("active world before spawn"))?
                        .plane = plane;
                }
                // GameClient.ib selects sn.la only after the entity lookup succeeds.
                self.active_world = id;
                self.active_plane = plane;
            }
            game_packet::Update::WorldEntities(payload) => {
                let bounds = if self.active_world == 0 {
                    [104, 104]
                } else {
                    self.world_entities
                        .get(&0)
                        .and_then(|world| world.entities.get(&self.active_world))
                        .ok_or(Error::Invalid("world entity parent view"))?
                        .dimensions
                };
                if self.interface_sequences.is_none() {
                    self.interface_sequences = Some(crate::animation::Sequence::load_all(
                        &mut self.scripts.source.cache,
                    )?);
                }
                let removed = self
                    .world_entities
                    .entry(self.active_world)
                    .or_default()
                    .read_packet_removed(
                        &payload,
                        bounds,
                        self.actor_cycle,
                        self.interface_sequences.as_ref().unwrap(),
                    )?;
                for (id, removed_entity) in removed {
                    // up.ev unregisters the oldest global view. bs.ag separately
                    // unlinks this parent's entity, which may own a different view.
                    let victim = self.world_entities.values().chain(self.detached_world_entities.iter())
                        .filter_map(|world| world.entities.get(&id))
                        .filter(|entity| entity.spawn_generation != 0)
                        .chain(removed_entity.iter())
                        .filter(|entity| entity.registered)
                        .map(|entity| entity.spawn_generation).min();
                    if let Some(generation) = victim {
                        for entity in self.world_entities.values_mut().chain(self.detached_world_entities.iter_mut())
                            .filter_map(|world| world.entities.get_mut(&id)) {
                            if entity.spawn_generation == generation { entity.registered = false; }
                        }
                    }
                    // Orphan global views have no parent-visible entity list.
                    // Release them once their independent registration is removed.
                    for retained in &mut self.detached_world_entities {
                        if retained.visible.is_empty() { retained.entities.retain(|_, entity| entity.registered); }
                    }
                    self.detached_world_entities.retain(|world| !world.entities.is_empty());
                    if let Some(entity) = removed_entity {
                        if entity.registered && Some(entity.spawn_generation) != victim {
                            let mut retained = crate::world_entity::State::default();
                            retained.entities.insert(id, entity);
                            self.detached_world_entities.push(retained);
                        }
                    }
                    // cc.aa unregisters only this view, not its registered descendants.
                    // An equal-ID child is not the active parent's entity table.
                    if id != self.active_world {
                        if let Some(children) = self.world_entities.remove(&id) {
                            if !children.entities.is_empty() { self.detached_world_entities.push(children); }
                        }
                    }
                    self.interaction_policy.remove(id as i16);
                    if self.view_entity(id).is_none() { self.view_registry.remove(id); }
                    let keep = victim.is_some_and(|generation| self.world_entities.values()
                        .chain(self.detached_world_entities.iter())
                        .any(|world| world.entities.get(&id).is_some_and(|e| e.spawn_generation == generation)));
                    if let Some(players) = &mut self.player_state { players.unregister_referenced_instance(id, victim, keep); }
                }
                let references = self.world_entities.values().chain(self.detached_world_entities.iter())
                    .flat_map(|world| world.entities.iter().map(|(&id,e)| (id,e.spawn_generation))).collect();
                if let Some(players) = &mut self.player_state { players.retain_view_references(&references); }
                let state = self.world_entities.get_mut(&self.active_world).unwrap();
                for &id in &state.visible { if state.entities[&id].registered { self.view_registry.insert(id); } }
                // Registration order follows the packet, not HashMap iteration.
                for &id in &state.visible {
                    let entity = state.entities.get_mut(&id).unwrap();
                    if entity.definition.is_some() { continue; }
                    // cy/bs creates a new GameWorldView before any rebuild packet.
                    // A reused numeric ID must not inherit the prior scene cache.
                    self.world_rebuild_generation = self.world_rebuild_generation.wrapping_add(1);
                    entity.spawn_generation = self.world_rebuild_generation;
                    let definition = crate::world_entity::load_definition(
                        &mut self.scripts.source.cache, &mut self.world_entity_definitions, entity.config
                    )?.clone();
                    if definition.default_animation >= 0 {
                        entity.default_animation.set(
                            definition.default_animation,
                            0,
                            0,
                            self.interface_sequences.as_ref().unwrap(),
                        )?;
                    }
                    entity.definition = Some(definition);
                }
            }
            game_packet::Update::WorldEntityRebuild(payload) => {
                let entity = self
                    .world_entities
                    .get_mut(&0)
                    .and_then(|world| world.entities.get_mut(&self.active_world))
                    .ok_or(Error::Invalid("world entity rebuild before spawn"))?;
                let rebuild = crate::rebuild::WorldEntity::decode(
                    &payload,
                    entity.dimensions.map(|v| v as usize),
                )?;
                self.world_rebuild_generation = self.world_rebuild_generation.wrapping_add(1);
                entity.rebuild_generation = self.world_rebuild_generation;
                entity.rebuild = Some(rebuild);
                entity.zone.scene_allocations.clear();
            }
            game_packet::Update::NpcOrigin(origin) => {
                self.npc_origin_offset = Some(origin);
                self.npc_origin = Some([
                    self.world_base[0] + origin[0] as i32,
                    self.world_base[1] + origin[1] as i32,
                ]);
            }
            game_packet::Update::Message(message) => {
                if message
                    .sender
                    .as_ref()
                    .is_some_and(|name| self.social.is_ignored(name))
                {
                    return Ok(());
                }
                self.chat.add(
                    message.kind as i32,
                    self.cycle,
                    message.sender.clone().unwrap_or_default(),
                    String::new(),
                    message.text.clone(),
                );
                self.chat.refresh_social(&self.social);
                if self.game_messages.len() == 100 {
                    self.game_messages.pop_front();
                }
                self.game_messages.push_back(message);
                self.interface_events.changed(1418, 0);
            }
            game_packet::Update::Stat {
                index,
                boosted,
                invisible,
                experience,
            } => {
                self.data.stat(index, boosted, invisible, experience)?;
                self.interface_events.changed(1415, index as i32);
            }
            game_packet::Update::RunEnergy(v) => self.data.energy = v as i32,
            game_packet::Update::RunWeight(v) => self.data.weight = v as i32,
            game_packet::Update::Reboot { message, cycles } => {
                self.data.reboot(message, cycles);
                self.interface_events.changed(1422, 0);
            }
            game_packet::Update::ServerTick => self.server_tick = self.server_tick.wrapping_add(1),
            game_packet::Update::DialogAbort => self.interfaces.queue_lifecycle(1423)?,
            game_packet::Update::LocationAnimation(request) => {
                let (base, dimensions, events) = if self.active_world == 0 {
                    (self.world_base, [104;2], &mut self.zone.locations)
                } else {
                    let entity = self.world_entities.get_mut(&0)
                        .and_then(|state| state.entities.get_mut(&self.active_world))
                        .ok_or(Error::Invalid("Location animation world before spawn"))?;
                    (entity.rebuild.as_ref().map_or([0;2], |r| r.base), entity.dimensions, &mut entity.zone.locations)
                };
                if let Some(event) = request.event(base, dimensions, self.active_plane as i32) { events.push_back(event); }
            }
            game_packet::Update::NpcHeadIcon { index, slot, group, image } => {
                let state = if self.active_world == 0 { &mut self.npc_state } else {
                    &mut self.world_entities.get_mut(&0)
                        .and_then(|world| world.entities.get_mut(&self.active_world))
                        .ok_or(Error::Invalid("NPC head icon world before spawn"))?.npcs
                };
                if let Some(npc) = state.npcs.get_mut(&index) {
                    let definition = self.npcs.get(&(npc.id as u32)).ok_or(Error::Invalid("NPC head icon definition"))?;
                    npc.appearance.set_head_icon(&definition.head_icons,slot,group,image)?;
                }
            }
            game_packet::Update::PlayerSequence { id, delay } => {
                // GameClient.ib -> ka.ec selects the local view, not active_world.
                if let Some(player) = self.player_state.as_mut().and_then(|s| s.local_actor_mut()) {
                    if self.interface_sequences.is_none() {
                        self.interface_sequences = Some(crate::animation::Sequence::load_all(
                            &mut self.scripts.source.cache,
                        )?);
                    }
                    player.sequence.set(id, delay as i32, player.path.len as i32,
                        self.interface_sequences.as_ref().unwrap())?;
                    player.path.path_at_start = player.sequence.path_at_start;
                }
            }
            game_packet::Update::NpcSequence { index, id, delay } => {
                if self.interface_sequences.is_none() {
                    self.interface_sequences = Some(crate::animation::Sequence::load_all(
                        &mut self.scripts.source.cache,
                    )?);
                }
                let state = if self.active_world == 0 { &mut self.npc_state } else {
                    &mut self.world_entities.get_mut(&0)
                        .and_then(|world| world.entities.get_mut(&self.active_world))
                        .ok_or(Error::Invalid("NPC animation world before spawn"))?.npcs
                };
                if let Some(npc) = state.npcs.get_mut(&index) {
                    npc.sequence.set(id, delay as i32, npc.path.len as i32,
                        self.interface_sequences.as_ref().unwrap())?;
                    npc.path.path_at_start = npc.sequence.path_at_start;
                }
            }
            game_packet::Update::ActorSpot(update) => {
                if update.player {
                    // Packet47 selects the root entity's view, independently of cc.tt.
                    let generation = self.world_entities.get(&0)
                        .and_then(|world| world.entities.get(&self.active_world))
                        .map(|entity| entity.spawn_generation);
                    if let Some(p) = self.player_state.as_mut().and_then(|s| {
                        if self.active_world == 0 { s.root_actor_mut(update.index) }
                        else { s.referenced_view_mut(self.active_world,generation?).and_then(|v| v.players.get_mut(&update.index)) }
                    })
                    {
                        p.spots.apply(
                            update.slot,
                            update.id,
                            update.height,
                            update.delay,
                            self.actor_cycle,
                        );
                    }
                } else {
                    let state = if self.active_world == 0 {
                        &mut self.npc_state
                    } else {
                        &mut self
                            .world_entities
                            .get_mut(&0)
                            .and_then(|world| world.entities.get_mut(&self.active_world))
                            .ok_or(Error::Invalid("NPC spot world before spawn"))?
                            .npcs
                    };
                    if let Some(npc) = state.npcs.get_mut(&update.index) {
                        npc.spots.apply(
                            update.slot,
                            update.id,
                            update.height,
                            update.delay,
                            self.actor_cycle,
                        );
                    }
                }
            }
            game_packet::Update::WorldSpot(mut request) => {
                let (dimensions, base, effects) = if self.active_world == 0 {
                    ([104, 104], self.world_base, &mut self.world_spots)
                } else {
                    let entity = self
                        .world_entities
                        .get_mut(&0)
                        .and_then(|state| state.entities.get_mut(&self.active_world))
                        .ok_or(Error::Invalid("world spot view missing"))?;
                    (
                        entity.dimensions,
                        entity.rebuild.as_ref().map_or([0; 2], |r| r.base),
                        &mut entity.world_spots,
                    )
                };
                if (0..dimensions[0]).contains(&request.position[1])
                    && (0..dimensions[1]).contains(&request.position[2])
                {
                    request.position = [
                        self.active_plane as i32,
                        request.position[1] + base[0],
                        request.position[2] + base[1],
                    ];
                    effects.push(request.start(self.actor_cycle));
                }
            }
            game_packet::Update::Projectile(request) => {
                if request.id != 65535 {
                    self.projectiles.push(request.start(self.actor_cycle));
                }
            }
            game_packet::Update::Zone(packet) => {
                // Absolute ground packets use cc.ap's coordinate lookup, not sn.la.
                let target_world = packet.absolute.map_or(self.active_world, |position| {
                    self.view_registry.containing([position[1], position[2]], |id| {
                        let entity = self.view_entity(id)?;
                        Some((entity.rebuild.as_ref().map_or([0; 2], |r| r.base), entity.dimensions))
                    })
                });
                if target_world != 0 {
                    let entity = if packet.absolute.is_some() {
                        crate::world_entity::registered_entity_mut(target_world,
                            &mut self.world_entities, &mut self.detached_world_entities)
                    } else {
                        self.world_entities.get_mut(&0)
                            .and_then(|state| state.entities.get_mut(&target_world))
                    }.ok_or(Error::Invalid("zone world missing"))?;
                    let base = entity.rebuild.as_ref().map_or([0; 2], |r| r.base);
                    entity.zone.origin = self.zone.origin;
                    let result = entity.zone.apply_in_view(
                        packet,
                        base,
                        self.active_plane as i32,
                        self.server_tick,
                        &self.items,
                        entity.dimensions,
                    );
                    self.zone.origin = entity.zone.origin;
                    result?;
                    while let Some(request) = entity.zone.graphics.pop_front() {
                        entity.world_spots.push(request.start(self.actor_cycle));
                    }
                    while let Some(sound) = entity.zone.sounds.pop_front() {
                        self.audio.request_world_area(target_world, sound, &self.preferences.device)?;
                    }
                    while let Some(request) = entity.zone.projectiles.pop_front() {
                        self.projectiles.push(request.start(self.actor_cycle));
                    }
                    while let Some(event) = entity.zone.scripted_projectiles.pop_front() {
                        self.scripted_events.push_back((self.actor_cycle, event));
                    }
                    return Ok(());
                }
                let result = self.zone.apply(
                    packet,
                    self.world_base,
                    self.active_plane as i32,
                    self.server_tick,
                    &self.items,
                );
                while let Some(sound) = self.zone.sounds.pop_front() {
                    self.audio.request_area(sound, &self.preferences.device)?;
                }
                while let Some(request) = self.zone.projectiles.pop_front() {
                    self.projectiles.push(request.start(self.actor_cycle));
                }
                while let Some(event) = self.zone.scripted_projectiles.pop_front() {
                    self.scripted_events.push_back((self.actor_cycle, event));
                }
                while let Some(request) = self.zone.graphics.pop_front() {
                    self.world_spots.push(request.start(self.actor_cycle));
                }
                result?;
            }
            game_packet::Update::Hint(raw) => {
                self.hint.apply(&raw, self.data.position[0])?;
            }
            game_packet::Update::Minimap(v) => self.minimap_state = v,
            game_packet::Update::MapFlag(v) => self.map_flag = (v != u32::MAX).then_some(v),
            game_packet::Update::ResetAnimations => {
                if self.active_world == 0 {
                    if let Some(state) = &mut self.player_state {
                        for (_, p) in state.root_iter_mut() {
                            p.sequence.reset();
                        }
                    }
                    for npc in self.npc_state.npcs.values_mut() {
                        npc.sequence.reset();
                    }
                } else {
                    let generation = self.world_entities.get(&0)
                        .and_then(|world| world.entities.get(&self.active_world))
                        .map(|entity| entity.spawn_generation);
                    if let Some(view) = self.player_state.as_mut().and_then(|s| s.referenced_view_mut(self.active_world,generation?)) {
                        for player in view.players.values_mut() { player.sequence.reset(); }
                    }
                    let entity = self
                        .world_entities
                        .get_mut(&0)
                        .and_then(|world| world.entities.get_mut(&self.active_world))
                        .ok_or(Error::Invalid("reset animations world missing"))?;
                    for npc in entity.npcs.npcs.values_mut() {
                        npc.sequence.reset();
                    }
                }
            }
            game_packet::Update::PlayerInfo(payload) => {
                if self.interface_sequences.is_none() {
                    self.interface_sequences = Some(crate::animation::Sequence::load_all(
                        &mut self.scripts.source.cache,
                    )?);
                }
                let state = self
                    .player_state
                    .as_mut()
                    .ok_or(Error::Invalid("player info before initialization"))?;
                let huffman = self
                    .huffman
                    .as_ref()
                    .ok_or(Error::Invalid("player Huffman table missing"))?;
                for id in crate::world_entity::registration_order(&self.world_entities, &self.detached_world_entities) {
                    self.view_registry.insert(id);
                }
                let ids=self.view_registry.ordered();
                let views=crate::player_info::instance_descriptors(&ids,ids.len(),self.world_base,
                    &self.world_entities,&self.detached_world_entities);
                state.decode_in_instances(
                    &payload,
                    &self.items,
                    huffman,
                    self.interface_sequences.as_ref(),
                    self.actor_cycle,
                    &views,
                )?;
                self.heatmap.refresh(self.world_base, state.root_iter().map(|(_,p)|p.path.position), self.cycle);
                for player in state.players.iter_mut().flatten().chain(
                    state.views.values_mut().chain(state.duplicate_views.values_mut()).flat_map(|v| v.players.values_mut())) {
                    for update in &player.blocks {
                        if let crate::player_update::Update::Actor(block) = update {
                            player.overlays.apply(
                                block,
                                self.actor_cycle,
                                &self.overlay_definitions,
                                |bit, id| {
                                    use crate::script_runtime::Source;
                                    if bit {
                                        let b = self.scripts.source.varbit(id)?;
                                        Ok(b.get(self.scripts.variables.player(b.base)))
                                    } else {
                                        Ok(self.scripts.variables.player(id as u16))
                                    }
                                },
                            )?;
                        }
                    }
                }
                let player = state.players[state.local as usize].as_ref().unwrap();
                self.data.position = player.position;
                if self.map_flag.is_some_and(|flag| {
                    [((flag >> 14) & 16383) as i32, (flag & 16383) as i32]
                        == [player.position[1], player.position[2]]
                }) {
                    self.map_flag = None;
                }

                if let Some(appearance) = &player.appearance {
                    self.combat_level = appearance.combat as i32;
                }
                let local = state.local;
                let events = std::mem::take(&mut state.chat_events);
                for (index, name, hidden, event) in events {
                    let message = match event {
                        crate::player_update::Update::Chat(chat) => {
                            let message = crate::chat_packet::player_message(
                                &chat,
                                &name,
                                hidden,
                                &self.social,
                                self.chat_delivery.restricted,
                            )?;
                            if let (Some(message), Some(player)) =
                                (&message, state.players[index as usize].as_mut())
                            {
                                let speech = &mut player.overlays.speech;
                                speech.text = Some(
                                    message
                                        .text
                                        .trim_matches(|c: char| (c as u32) <= 32)
                                        .to_owned(),
                                );
                                speech.remaining = 150;
                                speech.color = chat.color;
                                speech.effect = chat.effect;
                                speech.automatic = chat.automatic;
                                speech.pattern = crate::overhead::pattern(&chat.pattern);
                                speech.filtered = index != local
                                    && crate::chat_packet::player_type(chat.icon)?.2
                                    && !self.chat.filter.is_empty()
                                    && !message.text.to_lowercase().contains(&self.chat.filter);
                                let speech = speech.clone();
                                for view in state.views.values_mut().chain(state.duplicate_views.values_mut()) {
                                    if let Some(copy) = view.players.get_mut(&index) { copy.overlays.speech = speech.clone(); }
                                }
                            }
                            message
                        }
                        crate::player_update::Update::Actor(crate::actor_update::Block::Say(
                            text,
                        )) => {
                            if let Some(player) = state.players[index as usize].as_mut() {
                                player.overlays.speech.say(&text, true);
                            }
                            if let Some(text) = text.strip_prefix('~') {
                                Some(crate::chat_packet::Message {
                                    kind: 2,
                                    sender: name,
                                    prefix: String::new(),
                                    text: text.into(),
                                })
                            } else if index == local {
                                Some(crate::chat_packet::Message {
                                    kind: 2,
                                    sender: name,
                                    prefix: String::new(),
                                    text,
                                })
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };
                    if let Some(message) = message {
                        self.chat.add(
                            message.kind as i32,
                            self.cycle,
                            message.sender,
                            message.prefix,
                            message.text,
                        );
                        self.chat.refresh_social(&self.social);
                        self.interface_events.changed(1418, 0);
                    }
                }
            }
            game_packet::Update::NpcInfo { large, payload } => {
                if self.interface_sequences.is_none() {
                    self.interface_sequences = Some(crate::animation::Sequence::load_all(
                        &mut self.scripts.source.cache,
                    )?);
                }
                let (state, origin) = if self.active_world == 0 {
                    (
                        &mut self.npc_state,
                        self.npc_origin
                            .unwrap_or([self.data.position[1], self.data.position[2]]),
                    )
                } else {
                    let entity = self
                        .world_entities
                        .get_mut(&0)
                        .and_then(|world| world.entities.get_mut(&self.active_world))
                        .ok_or(Error::Invalid("NPC world before spawn"))?;
                    let base = entity.rebuild.as_ref().map_or([0; 2], |r| r.base);
                    let offset = self.npc_origin_offset.unwrap_or([0; 2]);
                    (
                        &mut entity.npcs,
                        [base[0] + offset[0] as i32, base[1] + offset[1] as i32],
                    )
                };
                state.decode_with_sequences(
                    &payload,
                    large,
                    origin,
                    &self.npcs,
                    self.interface_sequences.as_ref(),
                    self.actor_cycle,
                )?;
                for npc in state.npcs.values_mut() {
                    for block in &npc.blocks {
                        npc.overlays.apply(
                            block,
                            self.actor_cycle,
                            &self.overlay_definitions,
                            |bit, id| {
                                use crate::script_runtime::Source;
                                if bit {
                                    let b = self.scripts.source.varbit(id)?;
                                    Ok(b.get(self.scripts.variables.player(b.base)))
                                } else {
                                    Ok(self.scripts.variables.player(id as u16))
                                }
                            },
                        )?;
                    }
                }
            }
            game_packet::Update::Interface(u) => self.interface(u)?,
            game_packet::Update::Inventory(u) => {
                let id = match &u {
                    crate::inventory::Update::Full { container, .. }
                    | crate::inventory::Update::Partial { container, .. } => *container & 32767,
                    crate::inventory::Update::Stop(id) => *id,
                };
                self.inventories.apply(u);
                self.interface_events.changed(1414, id as i32);
            }
            game_packet::Update::Varp { id, value } => {
                self.scripts.variables.from_server(id, value);
                self.apply_menu_varp(id);
            }
            game_packet::Update::VarpReset => self
                .scripts
                .variables
                .reset_server(self.varp_ids.iter().copied()),
            game_packet::Update::VarpSync => {
                let changed = self.scripts.variables.server_differences();
                self.scripts.variables.synchronize();
                for id in changed {
                    self.apply_menu_varp(id);
                }
            }
            game_packet::Update::Script(call) => {
                let (id, ints, objects, longs) = call.split();
                let script = self.script(id)?;
                let mut vm = Vm::new(script, ints, objects, longs, 5000000)?
                    .with_int_slots(Rc::clone(&self.script_int_slots));
                loop {
                    let result = self.step_vm(&mut vm, 10000);
                    self.script_slot0 = vm.stacks.int_slot0();
                    if result? == vm::Status::Finished {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn world_base(&mut self, base: (i32, i32)) {
        // Rebuild packets establish a new scene allocation lifetime before later zone events.
        self.zone.scene_allocations.clear();
        self.camera.rebase(self.world_base, [base.0, base.1]);
        let delta = [
            self.world_base[0].wrapping_sub(base.0).wrapping_mul(128),
            self.world_base[1].wrapping_sub(base.1).wrapping_mul(128),
        ];
        if let Some(world) = self.world_entities.get_mut(&0) {
            for entity in world.entities.values_mut() {
                entity.shift(delta);
            }
        }
        self.world_base = [base.0, base.1];
        self.zone.piles.retain(|p, _| {
            (base.0..base.0 + 104).contains(&p[1]) && (base.1..base.1 + 104).contains(&p[2])
        });

        self.npc_origin = None;
    }
    pub fn reconnect_players(&mut self, data: &[u8], local_index: u16) -> Result<()> {
        let (mut state, consumed) = crate::player_info::State::initialize(data, local_index)?;
        state.set_npc_definitions(&self.npcs);
        if consumed != data.len() {
            return Err(Error::Invalid("reconnect player snapshot length"));
        }
        self.data.position = state.players[local_index as usize]
            .as_ref()
            .unwrap()
            .position;
        self.player_state = Some(state);
        for npc in self.npc_state.npcs.values_mut() {
            npc.facing = None;
        }
        self.inventories.containers.clear();
        self.data.reboot_timer = 0;
        self.data.reboot_message = None;
        self.minimap_state = 0;
        self.map_flag = None;
        self.outbound.clear();
        self.scene_walk_commands.clear();
        self.window.game_state = 30;
        self.queue_window_status();
        Ok(())
    }
    pub fn resize_canvas(&mut self, canvas: [i32; 2]) -> Result<()> {
        if self.canvas != canvas {
            self.interfaces.resize_canvas(canvas, true)?;
            self.canvas = canvas;
        }
        Ok(())
    }
    pub fn set_window_mode(&mut self, mode: i32) {
        if std::env::var_os("OSRS_SMOKE_RESIZE").is_some() {
            println!("Native window mode request {mode}, state {}, canvas {:?}", self.window.game_state, self.canvas);
        }
        self.window.set_mode(mode);
        self.preferences.window_mode = self.window.mode;
        if self.window.game_state >= 25 { self.queue_window_status(); }
    }
    pub fn native_layout(&mut self, physical: [i32; 2], now: u64, game_state: i32) -> Result<crate::canvas_transform::Fit> {
        let (geometry, resized, status) = self.window.frame(physical, now, game_state);
        if resized {
            self.interfaces.resize_canvas(geometry.canvas, true)?;
            self.canvas = geometry.canvas;
        }
        if status { self.queue_window_status(); }
        Ok(crate::canvas_transform::Fit { origin: geometry.origin, size: geometry.canvas, logical: geometry.canvas })
    }
    pub fn queue_window_status(&mut self) {
        let mut w = crate::writer::Writer::new();
        w.u8(self.preferences.window_mode as u8);
        w.u16(self.canvas[0] as u16);
        w.u16(self.canvas[1] as u16);
        self.outbound.push_back(crate::protocol::Packet {
            opcode: 10,
            payload: w.into_vec(),
        });
    }
    pub fn initialize_players(&mut self, data: &[u8], local_index: u16) -> Result<usize> {
        let (mut state, consumed) = crate::player_info::State::initialize(data, local_index)?;
        self.groups.clear();
        state.set_npc_definitions(&self.npcs);
        if self.huffman.is_none() {
            let lengths = self.scripts.source.cache.named_file(10, "huffman", "")?;
            self.huffman = Some(crate::huffman::Huffman::new(&lengths)?);
        }
        self.data.position = state.players[local_index as usize]
            .as_ref()
            .unwrap()
            .position;
        self.player_state = Some(state);
        self.npc_state = Default::default();
        Ok(consumed)
    }
    pub fn run_event(&mut self, event: ScriptEvent) -> Result<u64> {
        let mut execution = self.begin_event(event)?;
        while execution.step(self, 10000)? != vm::Status::Finished {}
        Ok(execution.vm.executed())
    }
    pub fn run_map_event(&mut self, kind:i32, event:crate::map_event::Event, mouse:Option<[i32;2]>) -> Result<u64> {
        self.world_map.event=Some(event);
        if self.world_map.elements.is_none() { self.world_map.elements=Some(crate::map_element::Definition::load_all(&mut self.scripts.source.cache)?); }
        let category=self.world_map.elements.as_ref().unwrap().get(&(event.element as u32)).map_or(-1,|d|d.category);
        let Some(script)=self.scripts.source.event_script(kind,event.element,category)? else {return Ok(0);};
        let mut vm=Vm::new(script,mouse.map_or_else(Vec::new,|p|p.to_vec()),vec![],vec![],500000)?.with_int_slots(Rc::clone(&self.script_int_slots));
        loop {let result=self.step_vm(&mut vm,10000);self.script_slot0=vm.stacks.int_slot0();if result?==vm::Status::Finished {return Ok(vm.executed());}}
    }
    fn group_event(&mut self, event: crate::group_state::Event) {
        let result = (|| -> Result<()> {
            if let Some(script) = self.scripts.source.event_script(85, event.definition as i32, -1)? {
                let mut vm = Vm::new(script, vec![event.slot as i32], vec![], vec![event.identity], 500000)?
                    .with_int_slots(Rc::clone(&self.script_int_slots));
                loop {
                    let result = self.step_vm(&mut vm, 10000);
                    self.script_slot0 = vm.stacks.int_slot0();
                    if result? == vm::Status::Finished { break; }
                }
            }
            Ok(())
        })();
        if let Err(error) = result {
            if self.script_failures.len() == 64 { self.script_failures.pop_front(); }
            self.script_failures.push_back(Error::Io(format!("group event {}: {error}", event.definition)));
        }
    }
    pub fn begin_event(&mut self, event: ScriptEvent) -> Result<EventExecution> {
        let context = event.context;
        let args = crate::script_event::bind(&event.arguments, &context)?;
        let id = args.script;
        let script = self.script(id)?;
        Ok(EventExecution {
            resize_depth: context.resize_depth,
            vm: Vm::new(script, args.ints, args.objects, args.longs, 5000000)?
                .with_int_slots(Rc::clone(&self.script_int_slots)),
            id,
            widget: context.widget,
        })
    }
    pub fn run_listener(
        &mut self,
        values: &[Value],
        context: &crate::script_event::Context,
    ) -> Result<u64> {
        let args = crate::script_event::bind(values, context)?;
        let id = args.script;
        let script = self.script(id)?;
        let mut vm = Vm::new(script, args.ints, args.objects, args.longs, 5000000)?
            .with_int_slots(Rc::clone(&self.script_int_slots));
        loop {
            let previous_depth = std::mem::replace(&mut self.resize_depth, context.resize_depth);
            let result = self.step_vm(&mut vm, 10000);
            self.resize_depth = previous_depth;
            self.script_slot0 = vm.stacks.int_slot0();
            if result.map_err(|e| {
                Error::Io(format!(
                    "interface script {id} widget {:?}: {e}",
                    context.widget
                ))
            })? == vm::Status::Finished
            {
                break;
            }
        }
        Ok(vm.executed())
    }
    pub fn dispatch_interface_events(&mut self, events: Vec<crate::interface_events::Queued>) -> u64 {
        use crate::interface_events::Phase;
        enum Pending {
            Ordinary(crate::interface_events::Queued),
            Resize(crate::widget_resize::Queued),
        }
        // Resizes already queued before collection precede ordinary general events.
        let mut general = self.interfaces.resize_events.borrow_mut().drain(..)
            .map(Pending::Resize).collect::<std::collections::VecDeque<_>>();
        let mut early = Vec::new();
        for event in events {
            if event.phase == Phase::General {general.push_back(Pending::Ordinary(event));}
            else {early.push(event);}
        }
        let (mut instructions, mut errors) = crate::interface_events::dispatch(early, |event| {
            if event.valid(&self.interfaces) {self.run_listener(&event.arguments, &event.context)} else {Ok(0)}
        });
        // Timer/release callbacks append to the general queue after its existing tail.
        general.extend(self.interfaces.resize_events.borrow_mut().drain(..).map(Pending::Resize));
        crate::interface_events::drain_prefetched(&mut general, |event, general| {
            let result = match event {
                Pending::Ordinary(event) => if event.valid(&self.interfaces) {
                    self.run_listener(&event.arguments, &event.context)
                } else {Ok(0)},
                Pending::Resize(event) => if event.valid(&self.interfaces) {
                    self.run_listener(&event.event.arguments, &event.event.context)
                } else {Ok(0)},
            };
            match result {Ok(count)=>instructions+=count,Err(error)=>errors.push(error)}
            general.extend(self.interfaces.resize_events.borrow_mut().drain(..).map(Pending::Resize));
        });
        for event in general {
            match event {
                Pending::Resize(event) => self.interfaces.resize_events.borrow_mut().push_back(event),
                Pending::Ordinary(_) => unreachable!("only callback-generated resizes remain after tail prefetch"),
            }
        }
        for error in errors {
            if self.script_failures.len() == 64 {self.script_failures.pop_front();}
            self.script_failures.push_back(error);
        }
        instructions
    }
    pub fn drain_resize_events(&mut self) -> u64 {
        self.dispatch_interface_events(Vec::new())
    }

}
impl Host for ClientState {
    // bj.bo is interpreter-global: a nested callback resets the parent's request.
    fn begin_script(&mut self) { self.script_close_pending = false; }
    fn request_close(&mut self) { self.script_close_pending = true; }
    fn take_close_request(&mut self) -> bool { std::mem::take(&mut self.script_close_pending) }
    fn finish_script(&mut self) -> Result<()> {
        while let Some(operation) = self.deferred_widget_operations.pop_front() {
            self.interfaces.load_group(&mut self.scripts.source.cache, (operation.widget >> 16) as u16)?;
            if let Some(id) = self.interfaces.script_child(operation.widget, operation.child) {
                self.interface_operation_with_item(id, operation.operation, None, Some(""), Some(operation.item))?;
            }
        }
        Ok(())
    }
    fn close_interfaces(&mut self) -> Result<()> {
        self.outbound.push_back(crate::protocol::Packet { opcode:95, payload:Vec::new() });
        self.interfaces.close_modal()
    }
    fn script(&mut self, id: i32) -> Result<Rc<Script>> {
        self.scripts.script(id)
    }
    fn read(&mut self, kind: Variable, id: i32) -> Result<Value> {
        match kind {
            Variable::ClanSetting => return self.clans.parameter(id).map(Value::Int),
            Variable::Clan => {
                return self
                    .clan_profile
                    .read(id, &mut self.scripts.source.cache)
                    .map(Value::Int)
            }
            _ => {}
        }
        self.scripts.read(kind, id)
    }
    fn write(&mut self, kind: Variable, id: i32, value: Value) -> Result<()> {
        self.scripts.write(kind, id, value)?;
        if kind == Variable::Player {
            self.apply_menu_varp(id as u16);
        }
        Ok(())
    }
    fn extended(&mut self, opcode: u16, dot: bool, stacks: &mut Stacks) -> Result<Control> {
        if opcode == 3100 {
            let text = interface_script::string(stacks)?;
            self.apply(game_packet::Update::Message(game_packet::GameMessage { kind:0, sender:None, text }))?;
            return Ok(Control::Continue);
        }
        if matches!(opcode, 3104..=3106) {
            let text = interface_script::string(stacks)?;
            self.outbound.push_back(crate::dialog_resume::response(opcode, &text)?);
            return Ok(Control::Continue);
        }
        if opcode == 3107 {
            let operation = stacks.pop_int()?;
            let name = interface_script::string(stacks)?;
            let outcome = crate::named_player_action::resolve(self.player_state.as_ref(), operation, &name)?;
            if let Some(packet) = outcome.packet { self.outbound.push_back(packet); }
            if let Some(text) = outcome.message {
                self.apply(game_packet::Update::Message(game_packet::GameMessage { kind:4, sender:None, text }))?;
            }
            return Ok(Control::Continue);
        }
        if matches!(opcode, 3108 | 3109) {
            let explicit = if opcode == 3108 { Some(stacks.pop_int()? as u32) } else { None };
            let y = stacks.pop_int()?;
            let x = stacks.pop_int()?;
            let widget = if let Some(id) = explicit {
                self.interfaces.load_group(&mut self.scripts.source.cache, (id >> 16) as u16)?;
                self.interfaces.widgets.contains_key(&id).then_some(id)
            } else { self.active_widgets[usize::from(dot)] };
            if self.menu.is_none() && self.widget_drag.is_none() {
                self.widget_drag = if opcode==3109 {
                    crate::script_drag::start_selected(&mut self.interfaces,&self.active_widgets,dot,[x,y],[-1;2],&self.pointer_menu)
                }else{crate::script_drag::start(&self.interfaces, widget, [x,y], [-1; 2], false, &self.pointer_menu)};
            }
            return Ok(Control::Continue);
        }
        if opcode == 3103 {
            return Ok(if self.interfaces.closing_modal { Control::Continue } else { Control::RequestClose });
        }
        if self.world_entity_limit.execute(opcode,stacks)? { return Ok(Control::Continue); }
        if matches!(opcode, 8560 | 8570) {
            self.groups.load_definitions(&mut self.scripts.source.cache)?;
        }
        if self.groups.execute(opcode, stacks)? { return Ok(Control::Continue); }
        if matches!(opcode, 1121 | 2121) {
            let id = if opcode == 2121 {
                let id = stacks.pop_int()? as u32;
                self.interfaces
                    .load_group(&mut self.scripts.source.cache, (id >> 16) as u16)?;
                id
            } else {
                self.active_widgets[usize::from(dot)].ok_or(Error::Invalid("active widget"))?
            };
            let identity = if opcode == 1121 {
                self.interfaces.sync_selected_widgets(&self.active_widgets);
                self.interfaces.selected_identity(usize::from(dot))
            } else {
                self.interfaces.runtime.get(&id).map(|runtime| (runtime.logical_id, runtime.child_index))
            }.ok_or(Error::Invalid("continue widget"))?;
            self.outbound.push_back(crate::interface_action::resume(identity.0, identity.1));
            if opcode == 1121 {
                self.interfaces.set_selected_continue_widget(usize::from(dot));
            } else {
                self.interfaces.set_continue_widget(Some(id));
            }
            return Ok(Control::Continue);
        }
        if crate::script_array::execute(opcode, stacks)? {
            return Ok(Control::Continue);
        }
        if opcode == 5001 {
            let [public, private, trade] = interface_script::ints(stacks)?;
            self.outbound
                .push_back(self.chat.set_filters(public, private, trade));
            return Ok(Control::Continue);
        }
        if matches!(opcode, 5008 | 5009 | 5010) {
            if self.huffman.is_none() {
                let lengths = self.scripts.source.cache.named_file(10, "huffman", "")?;
                self.huffman = Some(crate::huffman::Huffman::new(&lengths)?);
            }
            let codec = self.huffman.as_ref().unwrap();
            let packet = if opcode == 5009 {
                let text = interface_script::string(stacks)?;
                let recipient = interface_script::string(stacks)?;
                crate::chat_packet::private_message(&recipient, &text, codec)?
            } else {
                let text = interface_script::string(stacks)?;
                let (kind, channel) = if opcode == 5010 {
                    let [kind, channel] = interface_script::ints(stacks)?;
                    (kind, channel)
                } else {
                    (stacks.pop_int()?, -1)
                };
                crate::chat_packet::public_message(kind, &text, channel, codec)?
            };
            self.outbound.push_back(packet);
            return Ok(Control::Continue);
        }
        if (3628..=3657).contains(&opcode) {
            match opcode {
                3628 => self.social.friend_sort.clear(),
                3639 => self.social.sort_friends(self.data.world)?,
                3640 => self.social.ignore_sort.clear(),
                3643 => self.social.sort_ignores()?,
                3644 => {
                    if let Some(c) = &mut self.friends_chat {
                        c.sort.clear()
                    }
                }
                3655 => {
                    if let Some(c) = &mut self.friends_chat {
                        c.sort_members(self.data.world)?
                    }
                }
                _ => {
                    let ascending = stacks.pop_int()? == 1;
                    match opcode {
                        3629..=3638 => self
                            .social
                            .friend_sort
                            .add((opcode - 3629) as u8, ascending),
                        3641..=3642 => self
                            .social
                            .ignore_sort
                            .add((opcode - 3641) as u8, ascending),
                        3645..=3654 => {
                            if let Some(c) = &mut self.friends_chat {
                                c.sort.add((opcode - 3645) as u8, ascending)
                            }
                        }
                        3656 => self.social.friend_sort.add(10, ascending),
                        3657 => {
                            if let Some(c) = &mut self.friends_chat {
                                c.sort.add(10, ascending)
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            }
            return Ok(Control::Continue);
        }
        if (3604..=3608).contains(&opcode) {
            let name = interface_script::string(stacks)?;
            let rank = if opcode == 3604 { stacks.pop_int()? } else { 0 };
            let local = self
                .player_state
                .as_ref()
                .and_then(|s| s.root_actor(s.local))
                .and_then(|p| p.appearance.as_ref())
                .map_or("", |a| a.name.as_str());
            let command = self
                .social
                .command(opcode, &name, rank, local, self.data.account_members)?;
            if let Some(packet) = command.packet {
                self.outbound.push_back(packet);
            }
            if let Some(text) = command.message {
                self.apply(game_packet::Update::Message(game_packet::GameMessage {
                    kind: 30,
                    sender: None,
                    text,
                }))?;
            }
            if command.changed {
                self.chat.refresh_social(&self.social);
                self.interface_events.changed(1420, 0);
            }
            return Ok(Control::Continue);
        }
        if matches!(opcode, 3617 | 3619 | 3620) {
            let name = if opcode == 3620 {
                String::new()
            } else {
                interface_script::string(stacks)?
            };
            if let Some(packet) =
                crate::friends_chat::command(opcode, &name, self.friends_chat.is_some())?
            {
                self.outbound.push_back(packet);
            }
            return Ok(Control::Continue);
        }
        if (3600..=3627).contains(&opcode) {
            let name = self
                .player_state
                .as_ref()
                .and_then(|state| state.local_actor())
                .and_then(|player| player.appearance.as_ref())
                .map_or("", |appearance| appearance.name.as_str());
            if crate::friends_chat::execute(
                self.friends_chat.as_ref(),
                opcode,
                stacks,
                name,
                &self.social,
            )? {
                return Ok(Control::Continue);
            }
            if self.social.execute(opcode, stacks, name)? {
                return Ok(Control::Continue);
            }
        }
        if (3903..=3926).contains(&opcode) && self.market.execute(opcode, stacks, self.data.world, crate::market::now())? {
            return Ok(Control::Continue);
        }
        if matches!(opcode, 3819 | 3821 | 3859) {
            let (index, slot, muted) = match opcode {
                3819 => {
                    let [index, slot] = interface_script::ints(stacks)?;
                    (index, slot, false)
                }
                3821 => {
                    let [index, muted, slot] = interface_script::ints(stacks)?;
                    (index, slot, muted == 1)
                }
                _ => (stacks.pop_int()?, self.clan_channels.selected_slot, false),
            };
            if let Some(packet) = self.clan_channels.management(opcode, slot, index, muted)? {
                self.outbound.push_back(packet);
            }
            if opcode == 3821 {
                return Err(Error::Invalid(
                    "reference clan mute opcode returns unhandled after sending",
                ));
            }
            return Ok(Control::Continue);
        }
        if opcode == 3890 {
            stacks.push_int(i32::from(self.clan_profile.present))?;
            return Ok(Control::Continue);
        }
        if self.clan_channels.execute(opcode, stacks)? {
            return Ok(Control::Continue);
        }
        if self.clans.execute(opcode, stacks)? {
            return Ok(Control::Continue);
        }
        if self.chat.execute(opcode, stacks)? {
            return Ok(Control::Continue);
        }
        // bj's 5015 branch queries GameClient.je: absent local appearance is "".
        if opcode == 5015 {
            let name = self
                .player_state
                .as_ref()
                .and_then(|state| state.local_actor())
                .and_then(|player| player.appearance.as_ref())
                .map_or("", |appearance| appearance.name.as_str());
            stacks.push_object(Object::String(name.to_owned()))?;
            return Ok(Control::Continue);
        }
        if self
            .audio
            .execute(opcode, stacks, &self.preferences.device)?
        {
            return Ok(Control::Continue);
        }
        if self
            .database
            .execute(opcode, stacks, &mut self.scripts.source.cache)?
        {
            return Ok(Control::Continue);
        }
        if self.params.execute(
            opcode,
            stacks,
            &mut self.scripts.source.cache,
            &self.items,
            &self.npcs,
        )? {
            return Ok(Control::Continue);
        }
        if self.item_search.execute(opcode, stacks, &self.items)? {
            return Ok(Control::Continue);
        }
        if matches!(opcode, 1207 | 1210 | 2207 | 2210) { self.refresh_widget_player(); }
        if crate::widget_argument_operation::execute(opcode, stacks, &mut self.scripts.source.cache,
            &mut self.interfaces, &mut self.outbound)? {
            return Ok(Control::Continue);
        }
        if crate::widget_deferred::execute(opcode, dot, stacks, &mut self.scripts.source.cache,
            &mut self.interfaces, &self.active_widgets, &mut self.deferred_widget_operations)? {
            return Ok(Control::Continue);
        }
        if let Some(control) = crate::widget_resize::execute(opcode, dot, self.resize_depth,
            stacks, &mut self.scripts.source.cache, &mut self.interfaces, &self.active_widgets)? {
            return Ok(control);
        }
        if crate::remote_url::execute(opcode, dot, stacks, &mut self.scripts.source.cache,
            &mut self.interfaces, &self.active_widgets, &mut self.remote)? {
            return Ok(Control::Continue);
        }
        if interface_script::execute(
            opcode,
            dot,
            stacks,
            &mut self.scripts.source.cache,
            &mut self.interfaces,
            &mut self.active_widgets,
            self.canvas,
            &self.items,
        )? {
            return Ok(Control::Continue);
        }
        if matches!(opcode, 6600 | 6619 | 6620) {
            self.world_map.load(&mut self.scripts.source.cache)?;
            let local = self.local_map_coordinate();
            if opcode == 6600 {
                if local[0] != -1 { self.world_map.open_at(local, true); }
            } else {
                let packed = stacks.pop_int()?;
                let id = stacks.pop_int()?;
                let target = if packed == -1 { [-1, 0, 0] } else { [(packed >> 28) & 3, (packed >> 14) & 16383, packed & 16383] };
                self.world_map.open_target(id, local, target, opcode == 6620);
            }
            return Ok(Control::Continue);
        }
        if matches!(opcode, 6697..=6699) {
            self.world_map.event.ok_or(Error::Invalid("missing map script event"))?.query(opcode,stacks)?;
            return Ok(Control::Continue);
        }
        if self.world_map.execute(opcode, stacks, &mut self.scripts.source.cache)? { return Ok(Control::Continue); }
        if crate::world_entity::script_query(opcode, stacks, |id| {
            Ok(crate::world_entity::load_definition(
                &mut self.scripts.source.cache, &mut self.world_entity_definitions, id
            )?.name.clone())
        })? { return Ok(Control::Continue); }
        if self.data.execute(
            opcode,
            stacks,
            &mut self.scripts.source.cache,
            &self.inventories,
            self.cycle,
        )? || crate::script_string::execute_appearance(opcode, stacks,
            self.player_state.as_ref().and_then(|state| state.local_actor())
                .and_then(|player| player.appearance.as_ref()).map(|update| update.appearance.extra))?
            || crate::script_string::execute(opcode, stacks)?
            || crate::script_item::execute(opcode, stacks, &self.items)?
            || crate::script_npc::execute(opcode, stacks, &self.npcs)?
        {
            return Ok(Control::Continue);
        }
        if self
            .viewport
            .execute(opcode, stacks, &self.interfaces, self.canvas)?
        {
            return Ok(Control::Continue);
        }
        if crate::script_logout::execute(opcode, &mut self.logout_timer) {
            return Ok(Control::Continue);
        }
        if opcode == 5307 {
            let mode = stacks.pop_int()?;
            if matches!(mode, 1 | 2) { self.set_window_mode(mode); }
            return Ok(Control::Continue);
        }
        if self.preferences.execute(opcode, stacks, self.canvas)? {
            return Ok(Control::Continue);
        }
        if self.camera.execute(opcode, stacks)? {
            return Ok(Control::Continue);
        }
        if self.input.execute(opcode, stacks)? {
            return Ok(Control::Continue);
        }
        self.scripts.extended(opcode, dot, stacks).map_err(|e| {
            if matches!(e, Error::Unsupported(_)) {
                Error::Io(format!("unported client script opcode {opcode}"))
            } else {
                e
            }
        })
    }
}
