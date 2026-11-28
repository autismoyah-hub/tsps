//! Live transport/world bridge shared with the native window. Unsupported behavior is reported.
use crate::{
    client_state::ClientState,
    login::{self, Authentication, Credentials, Platform, Request},
    raster::Raster,
    render3d::{Camera, Frame},
    rsa::RsaPublicKey,
    scene::Scene,
    session::{Event, Session},
    texture::{Definition, Texture},
    world_mesh::WorldMesh,
    Error, Result,
};
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};
type Rebuild = std::result::Result<crate::rebuild::Normal, crate::rebuild::Instance>;
/// Deferred packets wait while a packet-synchronous interface event (onLoad,
/// lifecycle, structural continuation) is queued or running, so later packets
/// observe its effects. End-of-cycle editor callbacks (`PendingEvent::Deferred`,
/// Java's ct class) do not hold packets: GameClient.aft applies its packet
/// batch (ex) before draining ct (cq.ac), and ex never consults ct.
fn deferred_open(pending: &Option<crate::client_state::EventExecution>, client: &ClientState) -> bool {
    pending.is_none() && client.interfaces.scripts.iter().all(crate::interface_state::PendingEvent::is_end_of_cycle)
}
/// Prefix of the report for an end-of-cycle callback dropped by the live drain.
const STALE_CALLBACK: &str = "stale end-of-cycle callback";
/// One live drain: VM steps taken and entries left for the next poll.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Drain {
    steps: u32,
    deferred: usize,
}
/// Runs one generation of queued interface events to completion each poll.
/// GameClient.aft drains its queue once per cycle (ps.ac -> ji.ag ->
/// af(event, 500000, 475000)), bounded per event by instruction fuel, never
/// by elapsed time. The generation is the work queued when the drain starts,
/// with the structural expansions and mutation continuations it produces;
/// events appended by callbacks while it runs (editor listeners, for
/// example) wait for the next poll. That is a deliberate divergence from
/// Java's four deques, which can run some appended events in the same pass
/// and leave mutually requeuing chains unbounded. Here a generation is
/// finite: a fixed set of entries, each bounded by its VM fuel, plus
/// structural traversal. Arrays already captured by the walk keep their
/// length, but callbacks can create descendant arrays the walk has not yet
/// captured (up to depth 128), so a generation may still be large.
/// Deferred packets keep waiting while a generation leaves packet-synchronous
/// entries; a self-sustaining end-of-cycle editor chain advances one link per
/// poll without holding them (see `deferred_open`).
/// An end-of-cycle callback whose producing widget instance is no longer
/// current (unloaded, replaced, or a detached dynamic child) is dropped before
/// any VM step and reported (`Interfaces::deferred_current`; a bounded
/// fail-closed decision that differs from Java, which would run it).
/// Stops early when an event's script is still a PendingAsset; that event is
/// requeued at the front with its class. Event failures are appended to
/// `failures` even when a queue error is returned.
fn drain_interface_events(
    pending: &mut Option<crate::client_state::EventExecution>,
    client: &mut ClientState,
    failures: &mut Vec<String>,
) -> Result<Drain> {
    client.interfaces.begin_generation();
    let result = drain_generation(pending, client, failures);
    client.interfaces.end_generation();
    let steps = result?;
    Ok(Drain { steps, deferred: client.interfaces.scripts.len() })
}
fn drain_generation(
    pending: &mut Option<crate::client_state::EventExecution>,
    client: &mut ClientState,
    failures: &mut Vec<String>,
) -> Result<u32> {
    let mut steps = 0u32;
    loop {
        if pending.is_none() {
            let Some((event, instance)) = client.next_interface_event_tagged()? else {
                break;
            };
            if let Some(instance) = instance {
                if !client.interfaces.deferred_current(&event, instance) {
                    failures.push(format!("{STALE_CALLBACK}: widget {} instance {instance}", event.widget));
                    continue;
                }
            }
            match client.begin_event(event.clone()) {
                Ok(execution) => *pending = Some(execution),
                Err(Error::PendingAsset { .. }) => {
                    client.interfaces.scripts.push_front(crate::interface_state::PendingEvent::requeued(event, instance));
                    break;
                }
                Err(e) => {
                    failures.push(e.to_string());
                    continue;
                }
            }
        }
        let mut execution = pending.take().unwrap();
        steps = steps.saturating_add(1);
        match execution.step(client, 100) {
            Ok(crate::vm::Status::Finished) => {}
            Ok(_) => *pending = Some(execution),
            Err(e) => failures.push(e.to_string()),
        }
    }
    Ok(steps)
}
type MapIcons = Vec<([i32; 2], std::sync::Arc<crate::sprite_pixels::Pixels>)>;
fn connect_host(host: String, port: u16) -> std::thread::JoinHandle<Result<Session>> {
    std::thread::spawn(move || {
        use std::net::ToSocketAddrs;
        let addresses = (host.as_str(), port).to_socket_addrs()?;
        let mut error = Error::Invalid("world host has no addresses");
        for address in addresses {
            match Session::connect(address, Duration::from_secs(30)) {
                Ok(session) => return Ok(session),
                Err(e) => error = e,
            }
        }
        Err(error)
    })
}
#[cfg(not(feature = "custom-assets"))]
type WorldJob = std::thread::JoinHandle<
    Result<(
        Scene,
        WorldMesh,
        std::sync::Arc<crate::sprite_pixels::Pixels>,
        MapIcons,
    )>,
>;
#[cfg(feature = "custom-assets")]
type WorldJob = std::thread::JoinHandle<
    Result<(
        Scene,
        WorldMesh,
        std::sync::Arc<crate::sprite_pixels::Pixels>,
        MapIcons,
        Option<Vec<u32>>,
    )>,
>;
pub struct Connection {
    #[cfg(feature="enhanced-renderer")]
    enhanced: Option<crate::enhanced::Renderer>,
    entity_overlap_hidden: HashSet<u16>,
    terrain_visibility: crate::terrain_submission::Cache,
    entity_submitted_cycle: HashMap<u16, i32>,
    entity_sprites: HashMap<u32, std::sync::Arc<crate::sprite_pixels::Pixels>>,
    entity_npc_definitions: HashMap<u16, HashMap<u16, u32>>,
    entity_worlds: crate::world_scene_instances::Store<((u64, u64), Scene, WorldMesh, i32)>,
    pending_script: Option<crate::client_state::EventExecution>,
    deferred_events: std::collections::VecDeque<Event>,
    world_job: Option<WorldJob>,
    rebuild_walk: Option<crate::scene_walk::Selection>,
    next_rebuild: Option<Rebuild>,
    pub session: Session,
    address: std::net::SocketAddr,
    endpoint_host: String,
    hopping: bool,
    login_seed: Option<[u32; 4]>,
    login_retries: u8,
    logged_out: Option<u8>,
    pub title: Option<crate::title_input::State>,
    title_assets: Option<crate::title::Assets>,
    pub world_selector: Option<crate::world_select::State>,
    world_selector_assets: Option<crate::world_select::Assets>,
    world_selector_loading: bool,
    mouse_recorder: Option<crate::mouse_worker::Worker>,
    mouse_pending_click: Option<crate::mouse_click::Press>,
    mouse_click_state: crate::mouse_click::State,
    pub mouse_click_packets: u64,
    keyboard_input: crate::keyboard_input::Input,
    keyboard_report: crate::keyboard_report::State,
    pub keyboard_packets: u64,
    input_reports: crate::auxiliary_input::State,
    pending_wheel: i32,
    window_focused: bool,
    pub auxiliary_packets: u64,
    title_music_started: bool,
    transfer_delay: Option<login::TransferDelay>,
    remember_username: bool,
    authenticator: Option<(u32, bool)>,
    dob_in_client: bool,
    date_of_birth: Option<u32>,
    resume_seed: Option<[u32; 4]>,
    reconnect_job: Option<std::thread::JoinHandle<Result<Session>>>,
    token_login: bool,
    token_config: Option<crate::token_auth::Config>,
    token_secret: Option<String>,
    token_job: Option<std::thread::JoinHandle<(crate::token_auth::Config, Result<String>)>>,
    username: String,
    password: String,
    key: RsaPublicKey,
    crcs: [u32; 23],
    local: Option<u16>,
    started: Instant,
    /// #181: passive client journal (private test adapter only); `Off` leaves the poll unchanged.
    #[cfg(feature="private-test-adapter")]
    journal: JournalSlot,
    #[cfg(feature="native-profiling")]
    pub scene_profile:Option<crate::ui_profile::Profile>,
    #[cfg(feature="native-profiling")]
    readiness_trace:readiness_trace::Trace,
    #[cfg(feature="native-profiling")]
    pub scene_filter:crate::benchmark_scene::Mode,
    #[cfg(feature="native-profiling")]
    pub scene_filter_report:crate::benchmark_scene::Report,
    #[cfg(all(feature="native-profiling",feature="enhanced-renderer"))]
    pub enhanced_trace:Option<crate::enhanced_trace::Trace>,
    pub world: Option<(Scene, WorldMesh)>,
    minimap: Option<std::sync::Arc<crate::sprite_pixels::Pixels>>,
    #[cfg(feature = "custom-assets")]
    minimap_geography: Option<Vec<u32>>,
    minimap_icons: MapIcons,
    minimap_icons_pending: bool,
    minimap_assets: std::sync::Arc<crate::minimap::Assets>,
    textures: HashMap<u16, Texture>,
    palette: Vec<u32>,
    brightness: f64,
    pick: Option<(Frame, [i32; 2], crate::viewport_steering::Ray, [i32; 3])>,
    /// #175: passive draw/scene-epoch custody for the private test adapter.
    #[cfg(feature="private-test-adapter")]
    pub(crate) draw_custody: crate::draw_custody::Custody,
    actors: crate::world_actors::Actors,
    overlays: crate::overlay_render::Assets,
    locations: crate::location_update::Queue,
    world_cycle: i32,
    graphics_cycle: i32,
    location_definitions: Option<HashMap<u32, crate::object::Definition>>,
    reported: HashSet<String>,
    pub packets: usize,
}
impl Connection {
    #[cfg(feature="enhanced-renderer")]
    pub fn set_enhanced(&mut self, settings: crate::enhanced::Settings) -> Result<()> {
        settings.validate().map_err(Error::Io)?;
        self.enhanced = if settings.mode == crate::enhanced::Mode::Classic {None}
            else {Some(crate::enhanced::Renderer::new(settings).map_err(Error::Io)?)};
        Ok(())
    }
    pub fn start_mouse_recorder(&mut self) -> Result<()> {
        if self.mouse_recorder.is_none() { self.mouse_recorder = Some(crate::mouse_worker::Worker::start()?); }
        Ok(())
    }
    pub fn mouse_wheel(&mut self, delta: i32) { self.pending_wheel = self.pending_wheel.wrapping_add(delta); }
    pub fn window_focus(&mut self, focused: bool) { self.window_focused = focused; if !focused { self.keyboard_focus_lost(); } }
    pub fn keyboard_key(&mut self, key: i32, down: bool) { self.keyboard_input.key(key, down); }
    pub fn keyboard_focus_lost(&mut self) { self.keyboard_input.lose_focus(); }
    pub fn mouse_pointer(&self, point: [i32; 2]) {
        if let Some(worker) = &self.mouse_recorder { worker.pointer(point); }
    }
    pub fn mouse_click(&mut self, button: u8, point: [i32; 2]) {
        if let Some(worker) = &self.mouse_recorder { self.mouse_pending_click = Some(crate::mouse_click::Press { button, point, millis: worker.millis() }); }
    }
    pub fn mouse_stats(&self) -> Option<crate::mouse_worker::Stats> {
        self.mouse_recorder.as_ref().map(|worker| worker.stats())
    }
    pub fn player_preview(&self, client: &ClientState) -> Option<crate::player_preview::Snapshot> {
        let state = client.player_state.as_ref()?;
        let player = state.local_actor()?;
        let scene = if state.local_view == 0 { self.world.as_ref().map(|w| &w.0) }
            else { self.entity_worlds.get(&state.local_view).map(|w| &w.1) };
        let height = scene.and_then(|scene| scene.elevation(player.position[0] as usize,
            player.path.position[0] - scene.base_x * 128, player.path.position[1] - scene.base_y * 128));
        crate::player_preview::Snapshot::capture(player, client.actor_cycle, height)
    }
    pub fn camera_follow_target(&self, client: &ClientState) -> Option<([i32; 2], usize, i32)> {
        let target = client.camera_focus_local()?;
        let entities = client.world_entities.get(&0);
        let height = target.ground_height(entities, client.data.position[0] as u8, |view, plane, point| {
            let scene = if view == 0 { self.world.as_ref().map(|w| &w.0) }
                else { self.entity_worlds.get(&view).map(|w| &w.1) };
            scene.and_then(|s| s.elevation(plane as usize, point[0], point[1])).unwrap_or(0)
        });
        let projected = target.projected(entities);
        Some(([projected.fine[0] + client.world_base[0]*128, projected.fine[1] + client.world_base[1]*128], projected.plane as usize, height))
    }
    fn prune_entity_worlds(&mut self, client: &ClientState) {
        let mut retired=Vec::new();
        self.entity_worlds.retain(|&(id, instance), (generation, _, _, _)| {
            let keep=client.world_entities.values()
                .chain(client.detached_world_entities.iter())
                .any(|world| world.entities.get(&id)
                    .is_some_and(|entity| entity.spawn_generation == instance && instance == generation.0));
            if !keep {retired.push(id);}
            keep
        });
        self.select_registered_scenes(client);
        for id in retired {
            // GameScene.be/ba belong to the spawned scene, not its numeric ID.
            self.entity_submitted_cycle.remove(&id);
            self.entity_overlap_hidden.remove(&id);
        }
    }
    fn select_registered_scenes(&mut self, client: &ClientState) {
        self.entity_worlds.select_registered(client.world_entities.values()
            .chain(client.detached_world_entities.iter())
            .flat_map(|world|world.entities.iter().filter(|(_,e)|e.registered)
                .map(|(&id,e)|(id,e.spawn_generation))));
    }
    /// Resource access through a concrete world-view reference.
    pub fn instance_scene(&self, id: u16, generation: u64) -> Option<&Scene> {
        self.entity_worlds.get_instance(id,generation).map(|entry|&entry.1)
    }
    pub fn projectile_scenes(&self) -> HashMap<u16, &Scene> {
        self.entity_worlds.iter().map(|(&id, (_, scene, _, _))| (id, scene)).collect()
    }
    pub fn connect(
        client: &mut ClientState,
        address: &str,
        username: &str,
        password_file: &str,
        public_key: &str,
    ) -> Result<Self> {
        Self::connect_credentials(client,address,username,Some(password_file),None,public_key)
    }
    pub fn connect_token(client: &mut ClientState, address: &str, username: &str,
        config: crate::token_auth::Config, public_key: &str) -> Result<Self> {
        Self::connect_credentials(client,address,username,None,Some(config),public_key)
    }
    fn connect_credentials(client: &mut ClientState,address: &str,username: &str,
        password_file: Option<&str>,token_config: Option<crate::token_auth::Config>,public_key: &str) -> Result<Self> {
        client.uid192.configure_storage(std::path::PathBuf::from("client-data/uid192.dat"));
        let text = std::fs::read_to_string(public_key)?;
        let modulus = text
            .lines()
            .find_map(|s| s.strip_prefix("Modulus: "))
            .ok_or(Error::Invalid("RSA public modulus missing"))?;
        let key = RsaPublicKey::from_hex(modulus.trim(), 65537)?;
        let password=match password_file {Some(path)=>std::fs::read_to_string(path)?.trim().to_owned(),None=>String::new()};
        let crcs = client.scripts.source.cache.login_crcs()?;
        let brightness=client.preferences.device.brightness;
        let palette=crate::floor::palette(brightness)?;
        let mut textures = HashMap::new();
        for (id, d) in Definition::load_all(&mut client.scripts.source.cache)? {
            textures.insert(
                id,
                Texture::load(&mut client.scripts.source.cache, d, 128, brightness)?,
            );
        }
        let mut actors = crate::world_actors::Actors::load(&mut client.scripts.source.cache)?;
        actors.set_palette(&palette);
        let overlays = crate::overlay_render::Assets::load(&mut client.scripts.source.cache)?;
        let address = address
            .parse()
            .map_err(|_| Error::Invalid("server address"))?;
        let minimap_assets=std::sync::Arc::new(crate::minimap::Assets::load(&mut client.scripts.source.cache)?);
        let session = Session::disconnected();
        let token_login=token_config.is_some();
        let policy_pending=client.preferences.saved.accepted_eula < client.preferences.required_eula;
        let mut retained_token_config=None;
        let (token_job,connecting)=if policy_pending {
            retained_token_config=token_config;
            (None,None)
        } else if let Some(config)=token_config {
            (Some(std::thread::spawn(move || crate::token_auth::acquire(config))),None)
        } else {
            (None,Some(std::thread::spawn(move || Session::connect(address, Duration::from_secs(120)))))
        };
        Ok(Self {
            entity_sprites: HashMap::new(),
            entity_npc_definitions: HashMap::new(),
            entity_worlds: Default::default(),
            entity_overlap_hidden: HashSet::new(),
            terrain_visibility: crate::terrain_submission::Cache::default(),
            entity_submitted_cycle: HashMap::new(),
            pending_script: None,
            deferred_events: Default::default(),
            world_job: None,
            rebuild_walk: None,
            next_rebuild: None,
            session,
            address,
            endpoint_host: address.ip().to_string(),
            hopping: false,
            login_seed: None,
            login_retries: 0,
            logged_out: policy_pending.then_some(0),
            title: Some(crate::title_input::State {
                token_ready: token_login,
                required_eula: client.preferences.required_eula,
                accepted_eula: client.preferences.saved.accepted_eula,
                screen: if policy_pending { crate::title_input::Screen::Legal } else { crate::title_input::Screen::Connecting },
                username: username.into(),
                password: password.clone(),
                hide: client.preferences.device.hide_username,
                remember: client.preferences.saved.remembered_username.is_some(),
                lines: ["".into(), "Connecting to server...".into(), "".into()],
                ..Default::default()
            }),
            title_assets: None,
            world_selector: None,
            world_selector_assets: None,
            world_selector_loading: false,
            mouse_recorder: None,
            mouse_pending_click: None,
            mouse_click_state: Default::default(),
            mouse_click_packets: 0,
            keyboard_input: Default::default(),
            keyboard_report: Default::default(),
            keyboard_packets: 0,
            input_reports: Default::default(),
            pending_wheel: 0,
            window_focused: true,
            auxiliary_packets: 0,
            title_music_started: false,
            transfer_delay: None,
            remember_username: client.preferences.saved.remembered_username.is_some(),
            authenticator: None,
            dob_in_client: false,
            date_of_birth: None,
            resume_seed: None,
            reconnect_job: connecting,
            token_login,
            token_config: retained_token_config,
            token_secret: None,
            token_job,
            username: username.into(),
            password,
            key,
            crcs,
            local: None,
            started: Instant::now(),
            #[cfg(feature="private-test-adapter")]
            journal: JournalSlot::Off,
            #[cfg(feature="native-profiling")]
            scene_profile:None,
            #[cfg(feature="native-profiling")]
            readiness_trace:Default::default(),
            #[cfg(feature="native-profiling")]
            scene_filter:Default::default(),
            #[cfg(feature="native-profiling")]
            scene_filter_report:Default::default(),
            #[cfg(all(feature="native-profiling",feature="enhanced-renderer"))]
            enhanced_trace:None,
            #[cfg(feature="enhanced-renderer")]
            enhanced:None,
            world: None,
            minimap: None,
            #[cfg(feature = "custom-assets")]
            minimap_geography: None,
            minimap_icons: Vec::new(),
            minimap_icons_pending: false,
            minimap_assets,
            textures,
            palette,
            brightness,
            pick: None,
            #[cfg(feature="private-test-adapter")]
            draw_custody: crate::draw_custody::Custody::new(),
            actors,
            overlays,
            locations: Default::default(),
            world_cycle: 0,
            graphics_cycle: client.actor_cycle,
            location_definitions: None,
            reported: HashSet::new(),
            packets: 0,
        })
    }
    /// Last completed world draw, which may precede a later UI-only capture.
    #[cfg(any(feature="private-test-adapter",feature="native-profiling"))]
    pub fn capture_camera(&self)->Option<(crate::viewport_steering::Ray,[i32;3])> {
        self.pick.as_ref().map(|(_,_,ray,root_eye)|(*ray,*root_eye))
    }
    pub fn minimap_view(&self, client: &ClientState) -> Option<crate::minimap::View> {
        let (scene, _) = self.world.as_ref()?;
        let terrain = self.minimap.as_ref()?.clone();
        let local = client
            .player_state
            .as_ref()
            .and_then(|s| s.local_actor());
        let local_fine = client.local_minimap_fine();
        let local_pixels = local_fine.map(|v| v / 32);
        let team = local.and_then(|p| p.appearance.as_ref()).map_or(0, |a| a.team);
        let player_dot = |a: &crate::player_appearance::Update| {
            if client.social.is_friend_online(&a.name) { 3 }
            else if team != 0 && team == a.team { 4 }
            else if client.friends_chat.as_ref().is_some_and(|c| c.contains(&a.name)) { 5 }
            else if client.clans.actor_member(&a.name) { 6 } else { 2 }
        };
        let angle = client.camera.yaw as u16;
        let mut markers = Vec::new();
        let mut players = Vec::new();
        let delta = |fine| crate::minimap::offset(fine, local_fine, 0.03125);
        for (position, pile) in &client.zone.piles {
            if position[0] == client.data.position[0] && !pile.is_empty() {
                markers.push((delta([position[1] * 128 + 64, position[2] * 128 + 64]), 0));
            }
        }

        for (&index, npc) in &client.npc_state.npcs {
            if self
                .actors
                .visible_npcs
                .get(&index)
                .and_then(|id| client.npcs.get(id))
                .is_some_and(|d| d.minimap && d.interactable)
            {
                markers.push((delta(npc.path.position), 1));
            }
        }
        if let Some(state) = &client.player_state {
            for &i in &state.high {
                if i == state.local && state.local_view == 0 {
                    continue;
                }
                let Some(player) = state.root_actor(i) else {
                    continue;
                };
                let Some(a) = &player.appearance else {
                    continue;
                };
                if a.hidden {
                    continue;
                }
                let dot = player_dot(a);
                players.push((delta(player.path.position), dot));
            }
        }
        let hint = if client.hint.flashes(client.actor_cycle) {
            client.hint.minimap_fine(client).map(delta)
        } else { None };
        let destination = client.map_flag.and_then(|flag| {
            let x = ((flag >> 14) & 16383) as i32;
            let y = (flag & 16383) as i32;
            (x != scene.base_x).then_some(delta([x * 128 + 64, y * 128 + 64]))
        });
        let local_root = [
            local_fine[0] - scene.base_x * 128,
            local_fine[1] - scene.base_y * 128,
        ];
        let entities = client
            .world_entities
            .get(&0)
            .into_iter()
            .flat_map(|world| {
                world
                    .visible
                    .iter()
                    .filter_map(|id| world.entities.get(id).map(|entity| (*id, entity)))
            })
            .filter(|(id, _)| !self.entity_overlap_hidden.contains(id))
            .map(|(id, entity)| {
                let base = entity.rebuild.as_ref().map_or([0; 2], |r| r.base);
                let ground_markers = entity.zone.piles.iter().filter_map(|(position, pile)| {
                    if pile.is_empty() || position[0] != entity.plane as i32 {
                        return None;
                    }
                    let point = crate::minimap::ground_position(
                        [position[1] - base[0], position[2] - base[1]], true, Some(entity));
                    Some((
                        crate::minimap::offset(
                            point,
                            local_root,
                            0.03125,
                        ),
                        0,
                    ))
                });
                let npc_markers = entity.npcs.indices.iter().filter_map(|index| {
                    let npc = entity.npcs.npcs.get(index)?;
                    let definition = self
                        .entity_npc_definitions
                        .get(&id)?
                        .get(index)
                        .and_then(|id| client.npcs.get(id))?;
                    if !definition.minimap || !definition.interactable {
                        return None;
                    }
                    let point = entity.transform_horizontal([
                        npc.path.position[0] - base[0] * 128,
                        npc.path.position[1] - base[1] * 128,
                    ]);
                    Some((
                        crate::minimap::offset(
                            [point[0] as i32, point[2] as i32],
                            local_root,
                            0.03125,
                        ),
                        1,
                    ))
                });
                let player_markers = client.player_state.as_ref().into_iter().flat_map(|s| {
                    s.referenced_view(id, entity.spawn_generation).into_iter().flat_map(move |v| v.players.iter().filter_map(move |(&index,p)| {
                        if index == s.local && id == s.local_view { return None; }
                        let a = p.appearance.as_ref()?;
                        if a.hidden { return None; }
                        let point = entity.transform_horizontal([
                            p.path.position[0] - base[0]*128,p.path.position[1] - base[1]*128]);
                        Some((crate::minimap::offset([point[0] as i32,point[2] as i32],local_root,0.03125),player_dot(a)))
                    }))
                });
                let markers = ground_markers.chain(npc_markers).chain(player_markers).collect();
                crate::minimap::EntityView {
                    delta: crate::minimap::offset(
                        [entity.pose.x, entity.pose.z],
                        local_root,
                        0.03125,
                    ),
                    yaw: entity.pose.angle as u16,
                    sprite: entity
                        .definition
                        .as_ref()
                        .and_then(|d| self.entity_sprites.get(&(d.sprite as u32)))
                        .cloned(),
                    markers,
                }
            })
            .collect();
        Some(crate::minimap::View {
            steering: (client.interaction_policy.get(-1)[0]==2).then_some((client.input.pointer,client.steering)),
            terrain,
            assets: self.minimap_assets.clone(),
            markers,
            players,
            entities,
            icons: self
                .minimap_icons
                .iter()
                .map(|(p, s)| (delta([p[0] * 128 + 64, p[1] * 128 + 64]), s.clone()))
                .collect(),
            hint,
            destination,
            compass_disabled: client.minimap_state >= 3,
            source: [
                local_pixels[0] - scene.base_x * 4,
                crate::scene::SIZE as i32 * 4 - (local_pixels[1] - scene.base_y * 4),
            ],
            angle,
            disabled: matches!(client.minimap_state, 2 | 5),
            local_visible: local
                .and_then(|p| p.appearance.as_ref())
                .is_some_and(|a| !a.hidden),
        })
    }
    pub fn report(&mut self, message: String) {
        if self.reported.insert(message.clone()) {
            eprintln!("Pending: {message}");
        }
    }
    pub fn logged_out(&self) -> Option<u8> {
        self.logged_out
    }
    fn ensure_title_assets(&mut self, client: &mut ClientState) -> Result<bool> {
        let load = (|| -> Result<()> {
        if self.title_assets.is_none() {
            self.title_assets = Some(crate::title::Assets::load(
                &mut client.scripts.source.cache,
                client.data.world_flags as u32,
            )?);
        }
        if !self.title_music_started {
            client.audio.music.title(&mut client.scripts.source.cache,client.preferences.device.title_music_disabled)?;
            self.title_music_started=true;
        }
        if self.world_selector.is_some() && self.world_selector_assets.is_none() {
            self.world_selector_assets = Some(crate::world_select::Assets::load(&mut client.scripts.source.cache)?);
        }
        Ok(())
        })();
        match load {
            Ok(()) => Ok(true),
            Err(Error::PendingAsset { .. }) => Ok(false),
            Err(error) => Err(error),
        }
    }
    pub fn draw_title(&mut self, client: &mut ClientState, raster: &mut Raster) -> Result<bool> {
        let drawn = self.draw_title_screen(client, raster)?;
        // #175: a title frame clears this frame's raster, whatever paint put there.
        #[cfg(feature="private-test-adapter")]
        if drawn { self.draw_custody.title_overwrote(); }
        Ok(drawn)
    }
    fn draw_title_screen(&mut self, client: &mut ClientState, raster: &mut Raster) -> Result<bool> {
        if self.title.is_none() || (self.hopping && self.transfer_delay.is_none()) {
            return Ok(false);
        }
        if !self.ensure_title_assets(client)? {
            raster.clear(0xff000000);
            return Ok(true);
        }
        raster.clear(0xff000000);
        if let Some(state) = &mut self.world_selector {
            if self.world_selector_assets.is_none() {
                self.world_selector_assets = Some(crate::world_select::Assets::load(&mut client.scripts.source.cache)?);
            }
            self.world_selector_assets.as_ref().unwrap().draw(raster, &client.data.worlds, state, client.input.pointer)?;
            return Ok(true);
        }
        self.title.as_mut().unwrap().world_list_loading = client.data.worlds.loading();
        self.title_assets.as_ref().unwrap().draw(
            raster,
            self.title.as_ref().unwrap(),
            client.cycle,
        )?;
        self.title_assets
            .as_mut()
            .unwrap()
            .draw_fire(raster, client.cycle);
        self.title_assets.as_ref().unwrap().draw_mute(raster,client.preferences.device.title_music_disabled)?;
        if client.data.worlds.configured() {
            self.title_assets.as_ref().unwrap().draw_world_button(raster, client.data.world, self.world_selector_loading)?;
        }
        Ok(true)
    }
    fn title_input(&mut self, client: &mut ClientState) -> Result<()> {
        if self.title.is_none() {
            return Ok(());
        }
        if !self.ensure_title_assets(client)? { return Ok(()); }
        if let Some(state) = &mut self.world_selector {
            if self.world_selector_assets.is_none() {
                self.world_selector_assets = Some(crate::world_select::Assets::load(&mut client.scripts.source.cache)?);
            }
            let action = self.world_selector_assets.as_ref().unwrap().input(state, &client.input, client.canvas, client.preferences.mouse_camera);
            match action {
                Some(crate::world_select::Action::Close) => self.world_selector = None,
                Some(crate::world_select::Action::Sort(column, direction)) => client.data.worlds.sort_column(column, direction),
                Some(crate::world_select::Action::Select(index)) => {
                    if let Some(world) = client.data.worlds.worlds.get(index).cloned() {
                        // Custom endpoints retain their explicitly configured port, as server-directed hops do.
                        self.title_assets.as_mut().unwrap().set_world_logo(&mut client.scripts.source.cache, world.properties as u32)?;
                        self.endpoint_host = world.host;
                        client.data.world = world.id;
                        client.data.world_flags = world.properties;
                        client.set_members(world.properties & 1 != 0)?;
                        self.world_selector = None;
                    }
                }
                None => {}
            }
            if self.world_selector.is_none() {
                let title = self.title.as_mut().unwrap();
                if title.screen == crate::title_input::Screen::MembersRequired {
                    title.existing_user(0, client.preferences.saved.remembered_username.as_deref());
                }
            }
            return Ok(());
        }
        let [x,y]=client.input.press_position;
        let left=(client.canvas[0]-765)/2;
        let clicked = client.input.pressed_buttons&1!=0 || !client.preferences.mouse_camera&&client.input.pressed_buttons&4!=0;
        if client.data.worlds.configured() && (self.world_selector_loading || clicked && (left+5..=left+105).contains(&x) && (463..=498).contains(&y)) {
            let ready = client.data.worlds.poll_load();
            self.world_selector_loading = client.data.worlds.loading();
            if ready {
                self.world_selector = Some(Default::default());
                return Ok(());
            }
            if clicked && (left+5..=left+105).contains(&x) && (463..=498).contains(&y) { return Ok(()); }
        }
        if (client.input.pressed_buttons&1!=0 || !client.preferences.mouse_camera&&client.input.pressed_buttons&4!=0) && x>=left+715 && x<left+765 && (453..503).contains(&y) {
            client.preferences.device.title_music_disabled=!client.preferences.device.title_music_disabled;
            client.audio.music.title(&mut client.scripts.source.cache,client.preferences.device.title_music_disabled)?;
        }
        let (radio, help) = self.title_assets.as_ref().unwrap().input_dimensions()?;
        let state = self.title.as_mut().unwrap();
        state.required_eula = client.preferences.required_eula;
        state.accepted_eula = client.preferences.saved.accepted_eula;
        let legal_link = if state.screen == crate::title_input::Screen::Legal && clicked { self.title_assets.as_ref().unwrap().legal_link(client.canvas[0], [x,y]) } else { None };
        let mut action = state.input_for_world(
            &client.input,
            client.canvas[0],
            radio,
            help,
            client.preferences.mouse_camera,
            client.data.world_flags as u32,
            client.preferences.saved.remembered_username.as_deref(),
            crate::clipboard::read,
        );
        if let Some(url) = legal_link { action = Some(crate::title_input::Action::OpenLegalDocument(url)); }
        if matches!(action, Some(crate::title_input::Action::SubmitDateOfBirth)) {
            action = if let Some(today) = crate::date_of_birth::local_today() {
                state.submit_date_of_birth(today, crate::date_of_birth::local_noon)
            } else {
                state.lines = ["".into(), "Local date conversion is unavailable.".into(), "".into()];
                None
            };
        }
        client.preferences.device.hide_username = state.hide;
        self.remember_username = state.remember;
        if !state.remember
            || !state.hide
                && client.preferences.saved.remembered_username.as_deref()
                    != Some(state.username.as_str())
        {
            client.preferences.saved.remembered_username = None;
        }
        match action {
            Some(crate::title_input::Action::AcceptTerms) => { client.preferences.saved.accepted_eula = state.accepted_eula; }
            Some(crate::title_input::Action::OpenLegalDocument(url)) => { client.remote.pending_urls.push_back(url.into()); }
            Some(crate::title_input::Action::SelectWorld) => {
                if client.data.worlds.configured() {
                    if client.data.worlds.poll_load() { self.world_selector = Some(Default::default()); }
                    self.world_selector_loading = client.data.worlds.loading();
                } else {
                    self.report("world selection requires OSRS_WORLD_LIST_FILE or OSRS_WORLD_LIST_URL".into());
                }
            }
            Some(crate::title_input::Action::Login) => {
                client.prepare_login()?;
                self.username = state.username.clone();
                self.password = state.password.clone();
                self.date_of_birth = state.submitted_date_of_birth;
                self.authenticator = state
                    .submitted_authenticator
                    .map(|code| (code, state.trust_computer));
                self.logged_out = None;
                self.resume_seed = None;
                self.login_seed = None;
                self.login_retries = 0;
                self.local = None;
                self.started = Instant::now();
                self.begin_login_connection();
            }
            Some(crate::title_input::Action::CreateAccount | crate::title_input::Action::Help) => {
                state.notification([
                    "This client is using a custom world.".into(),
                    "Use that world's account and support services.".into(),
                    "".into(),
                ]);
                self.report("custom-world account/support navigation is not configured".into());
            }
            None | Some(crate::title_input::Action::SubmitDateOfBirth) => {}
        }
        Ok(())
    }
    fn login_message(&mut self, lines: [String; 3]) {
        self.hopping = false;
        self.transfer_delay = None;
        self.session.close();
        self.logged_out = Some(0);
        self.resume_seed = None;
        self.reconnect_job = None;
        self.deferred_events.clear();
        let title = self.title.get_or_insert_with(Default::default);
        title.token_ready = self.token_login;
        title.notification(lines);
    }
    fn login_rejected(&mut self, code: i32) {
        eprintln!("Native login response: {code}");
        let response = crate::login_response::response(code, false);
        let lines = response
            .lines
            .map(|v| v.map(str::to_owned))
            .unwrap_or_else(|| {
                self.title
                    .as_ref()
                    .map(|v| v.lines.clone())
                    .unwrap_or_default()
            });
        self.login_message(lines);
        let title = self.title.as_mut().unwrap();
        match response.index {
            7 => title.screen = if self.dob_in_client { crate::title_input::Screen::InlineDateOfBirth } else { crate::title_input::Screen::DateOfBirth },
            9 => title.screen = crate::title_input::Screen::Retry,
            6 => title.screen = crate::title_input::Screen::Message,
            8 => title.screen = crate::title_input::Screen::Privacy,
            33 => title.screen = crate::title_input::Screen::LauncherRequired,
            34 => title.screen = crate::title_input::Screen::MembersRequired,
            32 => title.screen = crate::title_input::Screen::DateOfBirth,
            3 => title.screen = crate::title_input::Screen::InvalidCredentials,
            4 => title.screen = crate::title_input::Screen::Authenticator,
            14 => title.screen = crate::title_input::Screen::AccountStatus(response.ban.unwrap_or(0) as u8),
            index => {
                if response.lines.is_none() {
                    title.lines = [
                        format!("Login response {code}."),
                        format!("Required login screen: {index}."),
                        "This flow is not implemented yet.".into(),
                    ];
                }
                self.report(format!("login response {code}: title screen {index} requires additional implementation"));
            }
        }
    }
    pub fn status_lines(&self) -> Option<Vec<&'static str>> {
        if self.hopping {
            Some(vec!["Please wait..."])
        } else if let Some(reason) = self.logged_out {
            Some(match reason {
                1 => vec!["You were disconnected from the server."],
                2 => vec![
                    "The game servers are currently being updated.",
                    "Please wait a few minutes and try again.",
                ],
                _ => vec!["Logged out."],
            })
        } else if self.is_reconnecting() {
            Some(vec![
                "Connection lost",
                "Please wait - attempting to reestablish",
            ])
        } else {
            None
        }
    }
    fn clear_world(&mut self) {
        self.world = None;
        self.rebuild_walk = None;
        self.entity_worlds.clear();
        self.entity_overlap_hidden.clear();
        self.entity_submitted_cycle.clear();
        self.entity_sprites.clear();
        self.entity_npc_definitions.clear();
        self.world_job = None;
        self.next_rebuild = None;
        self.minimap = None;
        #[cfg(feature = "custom-assets")]
        {self.minimap_geography=None;}
        self.minimap_icons.clear();
        self.minimap_icons_pending=false;
        self.pick = None;
        self.locations = Default::default();
        #[cfg(feature="private-test-adapter")]
        self.draw_custody.boundary(crate::draw_custody::Boundary::Reset, false);
    }
    fn log_out(&mut self, client: &mut ClientState, reason: u8) {
        self.hopping = false;
        self.transfer_delay = None;
        self.session.close();
        self.logged_out = Some(reason);
        let mut title = crate::title_input::State {
            token_ready: self.token_login,
            required_eula: client.preferences.required_eula,
            accepted_eula: client.preferences.saved.accepted_eula,
            screen: if client.preferences.saved.accepted_eula < client.preferences.required_eula { crate::title_input::Screen::Legal } else if self.token_login { crate::title_input::Screen::Token } else { crate::title_input::Screen::Welcome },
            username: if self.remember_username {
                self.username.clone()
            } else {
                String::new()
            },
            remember: self.remember_username,
            hide: client.preferences.device.hide_username,
            ..Default::default()
        };
        match reason {
            1 => title.notification([
                "".into(),
                "You were disconnected from the server.".into(),
                "".into(),
            ]),
            2 => title.notification([
                "The game servers are currently being updated.".into(),
                "Please wait a few minutes and try again.".into(),
                "".into(),
            ]),
            _ => {}
        }
        self.title = Some(title);
        self.resume_seed = None;
        self.login_seed = None;
        self.local = None;
        self.pending_script = None;
        self.deferred_events.clear();
        self.clear_world();
        client.outbound.clear();
        client.scene_walk_commands.clear();
        client.player_state = None;
        client.npc_state = Default::default();
        client.zone = Default::default();
        client.projectiles.clear();
        client.scripted_projectiles = Default::default();
        client.scripted_events.clear();
        client.groups.clear();
        client.world_spots.clear();
        client.world_entities.clear();
        client.detached_world_entities.clear();
        client.view_registry = Default::default();
        client.audio.prepare_logout();
        self.title_music_started=false;
        client.interfaces.scripts.clear();
        client.interfaces.closing_modal = false;
        client.pointer_menu.clear();
        println!("Live native logout, reason {reason}");
    }
    pub fn is_reconnecting(&self) -> bool {
        self.resume_seed.is_some()
    }
    fn begin_login_connection(&mut self) {
        self.token_secret=None;
        if self.token_login {
            if let Some(config)=self.token_config.take() {
                self.token_job=Some(std::thread::spawn(move ||crate::token_auth::acquire(config)));
                self.reconnect_job=None;
            } else { self.login_rejected(65); }
        } else {
            self.reconnect_job=Some(connect_host(self.endpoint_host.clone(),self.address.port()));
        }
    }
    fn begin_hop(&mut self, client: &mut ClientState, target: crate::game_packet::WorldHop) -> Result<()> {
        // Native endpoints are explicitly configured, so keep their port when
        // the server supplies a new host. DNS and connect run off the UI thread.
        self.endpoint_host = target.host;
        self.started = Instant::now();
        self.session.close();
        self.hopping = true;
        self.logged_out = None;
        self.transfer_delay = None;
        self.resume_seed = None;
        self.login_seed = None;
        self.login_retries = 0;
        self.local = None;
        self.pending_script = None;
        self.deferred_events.clear();
        self.clear_world();
        client.outbound.clear();
        client.scene_walk_commands.clear();
        client.data.world = target.world as i32;
        client.data.world_flags = target.flags as i32;
        client.set_members(target.flags & 1 != 0)?;
        self.title = Some(crate::title_input::State {
            token_ready: self.token_login,
            screen: crate::title_input::Screen::Connecting,
            lines: [String::new(), "Please wait...".into(), String::new()],
            username: self.username.clone(),
            password: self.password.clone(),
            ..Default::default()
        });
        self.begin_login_connection();
        Ok(())
    }
    fn packet_failure(&mut self, client:&mut ClientState,error:Error)->Result<()> {
        match crate::packet_failure::action(&error,client.logout_timer) {
            crate::packet_failure::Action::Reconnect=>self.begin_reconnect(client),
            crate::packet_failure::Action::Logout=>{self.report(format!("packet failure: {error}"));self.log_out(client,0);Ok(())}
        }
    }
    fn begin_reconnect(&mut self, client: &mut ClientState) -> Result<()> {
        if client.logout_timer > 0 {
            self.log_out(client, 0);
            return Ok(());
        }
        if self.resume_seed.is_some() {
            return Err(Error::Io("session disconnected".into()));
        }
        if self.local.is_none() {
            return Err(Error::Invalid("connection lost before login success"));
        }
        self.login_retries = 0;
        self.resume_seed = Some(
            self.login_seed
                .ok_or(Error::Invalid("connection lost before login"))?,
        );
        self.session.close();
        self.pending_script = None;
        self.deferred_events.clear();
        client.outbound.clear();
        client.scene_walk_commands.clear();
        self.begin_login_connection();
        println!("Connection lost; reconnecting to the same world");
        Ok(())
    }
    fn advance_transfer(&mut self, ticks: u32) {
        let (lines, finished) = self.transfer_delay.as_mut().unwrap().advance_ticks(ticks);
        self.title.get_or_insert_with(Default::default).lines = lines;
        if finished {
            self.transfer_delay = None;
            self.begin_login_connection();
        }
    }
    pub fn poll(&mut self, client: &mut ClientState) -> Result<()> {
        self.poll_elapsed(client, 1)
    }
    /// Poll transport once and advance login timers by elapsed game ticks.
    pub fn poll_elapsed(&mut self, client: &mut ClientState, ticks: u32) -> Result<()> {
        // #181: once the journal is exported, a further recorded call is a defect; refuse it before any work.
        #[cfg(feature="private-test-adapter")]
        if matches!(self.journal, JournalSlot::Exported) {
            return Err(Error::Invalid("client journal: recorded seam after final"));
        }
        let result = self.poll_elapsed_inner(client, ticks);
        // One call record per poll, on every exit including errors.
        #[cfg(feature="private-test-adapter")]
        if let JournalSlot::Recording(recorder) = &mut self.journal {
            recorder.call_end(crate::client_journal::ClientClocks {
                actor_cycle: client.actor_cycle,
                client_counted_server_ticks: client.server_tick,
            });
        }
        result
    }
    fn poll_elapsed_inner(&mut self, client: &mut ClientState, ticks: u32) -> Result<()> {
        while let Some(url)=client.remote.pending_urls.pop_front() {
            if let Err(error)=crate::remote_input::open_web_url(&url) {self.report(format!("external URL: {error}"));}
        }
        if ticks != 0 { if let Some(worker) = &self.mouse_recorder { worker.latch_pointer(); } }
        let mut wheel = if ticks != 0 { std::mem::take(&mut self.pending_wheel) } else { 0 };
        let mut keyboard_cycle = self.keyboard_input.advance(ticks);
        let mouse_click = if ticks != 0 { self.mouse_pending_click.take() } else { None };
        if let Some(error) = client.data.worlds.last_error.take() { self.report(format!("world list: {error}")); }
        if let Some(target) = client.world_hop.take() {
            self.begin_hop(client, target)?;
        }
        if self.transfer_delay.is_some() {
            self.advance_transfer(ticks);
            return Ok(());
        }
        if self.logged_out.is_some() {
            self.title_input(client)?;
        }
        if self.logged_out.is_some() {
            return Ok(());
        }
        let mut rebuild = None;
        if let Some(job)=&self.token_job {
            if !job.is_finished() { return Ok(()); }
            match self.token_job.take().unwrap().join() {
                Ok((config,result))=>{
                    self.token_config=Some(config);
                    match result {
                        Ok(token)=>{
                            self.token_secret=Some(token);
                            self.reconnect_job=Some(connect_host(self.endpoint_host.clone(),self.address.port()));
                        }
                        Err(_)=>{self.login_rejected(65);return Ok(());}
                    }
                }
                Err(_)=>{self.login_rejected(65);return Ok(());}
            }
        }
        if let Some(job) = &self.reconnect_job {
            if !job.is_finished() {
                return Ok(());
            }
            let session = self
                .reconnect_job
                .take()
                .unwrap()
                .join()
                .map_err(|_| Error::Io("reconnect worker panicked".into()))?;
            match session {
                Ok(session) => self.session = session,
                Err(error) => {
                    self.report(error.to_string());
                    self.login_rejected(-2);
                    return Ok(());
                }
            }
        }
        let established = self.session.is_established();
        #[cfg(feature="native-profiling")]
        {
            let now=self.started.elapsed();
            if self.observation_ready(client) {if let Some(line)=self.readiness_trace.milestone(readiness_trace::READY,now) {eprintln!("{line}");}}
            if let Some(line)=self.readiness_trace.cycle(self.deferred_events.len(),client.interfaces.scripts.len(),self.pending_script.is_some(),now) {eprintln!("{line}");}
        }
        match self.session.poll() {
            #[cfg(feature="native-profiling")]
            Ok(events) => {
                let now=self.started.elapsed();
                for event in &events {
                    if let Event::Packet(packet)=event {for line in self.readiness_trace.received(packet.opcode,now) {eprintln!("{line}");}}
                }
                self.deferred_events.extend(events)
            }
            #[cfg(not(feature="native-profiling"))]
            Ok(events) => self.deferred_events.extend(events),
            Err(error) if established => {
                self.packet_failure(client,error)?;
                return Ok(());
            }
            Err(error) => {
                self.report(error.to_string());
                self.login_rejected(-2);
                return Ok(());
            }
        }
        if self.deferred_events.len() > 16384 {
            return Err(Error::Invalid(
                "live event backlog exceeded during script execution",
            ));
        }
        while deferred_open(&self.pending_script, client) {
            let Some(event) = self.deferred_events.pop_front() else {
                break;
            };
            match event {
                Event::Login(login::Event::DateOfBirth { in_client }) => {
                    self.dob_in_client = in_client;
                    self.login_rejected(61);
                }
                Event::Login(login::Event::ServerSeed(_)) => {
                    let platform = Platform {
                        os: 1,
                        is_64_bit: true,
                        java_vendor: 4,
                        java_version: [21, 0, 0],
                        memory_mb: 512,
                        processors: 2,
                        client_name: "Rust239".into(),
                        ..Default::default()
                    };
                    let request = Request {
                        username: &self.username,
                        credentials: if let Some(previous_seed) = self.resume_seed {
                            Credentials::Reconnect { previous_seed }
                        } else {
                            Credentials::Login {
                                authentication: self.date_of_birth.map_or_else(|| self.authenticator.map_or(
                                    client.preferences.saved.token(&self.username).map_or(
                                        Authentication::Password,
                                        Authentication::Remembered,
                                    ),
                                    |(code, remember)| Authentication::Authenticator {
                                        code,
                                        remember,
                                    },
                                ), Authentication::DateOfBirth),
                                secret: if self.token_login { self.token_secret.as_deref().ok_or(Error::Invalid("play token unavailable"))? } else { &self.password },
                                token: self.token_login,
                            }
                        },
                        client_build: i32::MAX as u32,
                        client_type: 1,
                        platform_type: 0,
                        resizable: client.preferences.window_mode == 2,
                        low_detail: false,
                        width: client.canvas[0] as u16,
                        height: client.canvas[1] as u16,
                        uuid: client.uid192.login_bytes(),
                        site_settings: &client.site_settings,
                        affiliate: 0,
                        deep_links: &[],
                        platform: &platform,
                        archive_crcs: &self.crcs,
                    };
                    self.login_seed = Some(self.session.submit(&request, &self.key)?);
                }
                Event::Login(login::Event::Success(account)) => {
                    if let Some(worker) = &self.mouse_recorder { worker.reset_queue(); }
                    self.mouse_click_state = Default::default();
                    self.keyboard_report = Default::default();
                    self.input_reports = Default::default();
                    self.pending_wheel = 0;
                    wheel = 0;
                    self.keyboard_input.clear_pending();
                    keyboard_cycle = None;
                    self.mouse_pending_click = None;
                    self.date_of_birth = None;
                    let hopping = std::mem::take(&mut self.hopping);
                    if self.resume_seed.take().is_some() || hopping {
                        // Response 2 follows the full GameNPCComposition.ad reset even
                        // when the request was a reconnect (response 15 preserves state).
                        client.prepare_login()?;
                        self.clear_world();
                        self.pending_script = None;
                    }
                    self.local = Some(account.player_index);
                    if let Some(token) = account.remembered_token {
                        client
                            .preferences
                            .saved
                            .remember_token(&self.username, token);
                    }
                    client.preferences.saved.remembered_username =
                        self.remember_username.then(|| self.username.clone());
                    self.title = None;
                    self.title_assets=None;
                    self.title_music_started=false;
                    client.audio.music.stop(0,100);
                    // Login callbacks can select a mode before the next native frame.
                    client.window.game_state = 25;
                    client.queue_window_status();
                    client.remote.user_id = account.user_id as i64;
                    client.remote.user_hash = account.user_hash as i64;
                    client.data.account_members = account.members;
                    client.data.staff_level = account.staff_level as i32;
                    client.data.moderator = account.player_moderator;
                    println!(
                        "Live native login accepted, player {}",
                        account.player_index
                    );
                    #[cfg(feature="native-profiling")]
                    if let Some(line)=self.readiness_trace.milestone(readiness_trace::LOGIN,self.started.elapsed()) {eprintln!("{line}");}
                }
                Event::Packet(packet) => {
                    self.packets += 1;
                    macro_rules! packet_try {
                        ($value:expr) => {match $value {Ok(value)=>value,Err(error)=>{self.packet_failure(client,error)?;return Ok(());}}};
                    }

                    if packet.opcode == 49 {
                        let index = if client.player_state.is_none() {
                            self.local
                        } else {
                            None
                        };
                        let normal = packet_try!(crate::rebuild::Normal::decode(&packet.payload, index));
                        if let Some(index) = index {
                            packet_try!(client.initialize_players(&packet.payload, index));
                        }
                        client.world_base(normal.base());
                        rebuild = Some(Ok(normal));
                    } else if packet.opcode == 125 {
                        let instance = packet_try!(crate::rebuild::Instance::decode(&packet.payload));
                        client.world_base(instance.base());
                        rebuild = Some(Err(instance));
                    } else {
                        match crate::game_packet::decode(&packet) {
                            Ok(Some(u)) => {
                                // Interpreter failures are caught inside Java's script runner.
                                let script_packet=matches!(&u,crate::game_packet::Update::Script(_));
                                let scene_identity_packet=matches!(&u,crate::game_packet::Update::WorldEntities(_));
                                match client.prepare_packet_assets(&u) {
                                    Ok(())=>(),
                                    Err(Error::PendingAsset { .. })=>{
                                        #[cfg(feature="native-profiling")]
                                        self.readiness_trace.pending_asset();
                                        self.deferred_events.push_front(Event::Packet(packet));break;
                                    }
                                    Err(error) if script_packet=>{self.report(format!("packet script: {error}"));continue;}
                                    Err(error)=>{self.packet_failure(client,error)?;return Ok(());}
                                }
                                if let Err(error)=client.apply(u) {
                                    if script_packet {self.report(format!("packet script: {error}"));}
                                    else {self.packet_failure(client,error)?;return Ok(());}
                                }
                                #[cfg(feature="native-profiling")]
                                if let Some(line)=self.readiness_trace.applied(packet.opcode,self.started.elapsed()) {eprintln!("{line}");}
                                // Later packets in this poll must not sample a replaced scene.
                                if scene_identity_packet {self.prune_entity_worlds(client);}
                                if let Some((scene, _)) = &self.world {
                                    if !client.scripted_events.is_empty() {
                                        let children=self.projectile_scenes();
                                        packet_try!(client.apply_scripted_projectile_events(scene,&children));
                                    }
                                    client
                                        .camera
                                        .resolve_snaps(scene, client.data.position[0] as usize);
                                }
                                if let Some(reason) = client.logout_request.take() {
                                    self.log_out(client, reason);
                                    return Ok(());
                                }
                                if let Some(target) = client.world_hop.take() {
                                    self.begin_hop(client, target)?;
                                    return Ok(());
                                }
                            }
                            Ok(None) => self.report(format!(
                                "packet {} ({})",
                                packet.opcode,
                                crate::protocol239::SERVER_PACKETS
                                    .iter()
                                    .find(|s| s.opcode as u16 == packet.opcode)
                                    .map_or("unknown", |s| s.name)
                            )),
                            Err(error)=>{self.packet_failure(client,error)?;return Ok(());}
                        }
                    }
                }
                Event::Login(login::Event::Rejected(code)) => {
                    // GameClient.iz: response 23 restarts login once on the
                    // same endpoint. The transfer countdown does not reset iw.
                    if code == 23 && self.login_retries == 0 {
                        self.login_retries += 1;
                        self.session.close();
                        self.deferred_events.clear();
                        self.begin_login_connection();
                        return Ok(());
                    }
                    self.login_rejected(code as i32);
                    return Ok(());
                }
                Event::Login(login::Event::Message(lines)) => {
                    self.login_message(lines);
                    return Ok(());
                }
                Event::Disconnected => {
                    if self.local.is_none() {
                        self.login_message([
                            "Connection lost.".into(),
                            "Please try again.".into(),
                            "".into(),
                        ]);
                    } else {
                        self.begin_reconnect(client)?;
                    }
                    return Ok(());
                }
                Event::Login(login::Event::Reconnected(bytes)) => {
                    let local = self.local.ok_or(Error::Invalid("reconnect local player"))?;
                    client.reconnect_players(&bytes, local)?;
                    self.resume_seed = None;
                    self.pick = None;
                    #[cfg(feature="private-test-adapter")]
                    self.draw_custody.boundary(crate::draw_custody::Boundary::Reconnect, self.world.is_some());
                    println!("Live native session reconnected");
                }
                Event::Login(login::Event::HopDelay { cycles }) => {
                    self.deferred_events.clear();
                    self.login_seed = None;
                    self.transfer_delay = Some(login::TransferDelay { remaining: cycles as i32 });
                    self.advance_transfer(ticks);
                    return Ok(());
                }
                _ => {}
            }
        }
        if self.session.is_established() {
            if let Err(error) = self.session.advance_receive_timeout(ticks, self.world.is_some() && rebuild.is_none()) {
                self.packet_failure(client, error)?;
                return Ok(());
            }
            if ticks != 0 && self.world.is_some() && rebuild.is_none() {
                if let Some(worker) = &self.mouse_recorder {
                    let (packet, point) = worker.send(mouse_click.is_some());
                    client.data.mouse_reported = Some(point);
                    if let Some(packet) = packet { self.session.send(packet.opcode, &packet.payload)?; }
                }
                if let Some(press) = mouse_click {
                    if let Some(packet) = self.mouse_click_state.send(press.button, client.preferences.mouse_camera, press.point, client.canvas, press.millis) {
                        self.session.send(packet.opcode, &packet.payload)?;
                        self.mouse_click_packets += 1;
                    }
                }
            }
            if ticks != 0 && self.world.is_some() && rebuild.is_none() {
                if self.mouse_recorder.is_some() {
                    if let Some(packet) = crate::auxiliary_input::State::wheel(wheel) {
                        self.session.send(packet.opcode, &packet.payload)?;
                        self.auxiliary_packets += 1;
                    }
                }
                if let (Some((age, keys)), Some(worker)) = (keyboard_cycle, &self.mouse_recorder) {
                    if let Some(packet) = self.keyboard_report.send(age, &keys, worker.millis()) {
                        self.session.send(packet.opcode, &packet.payload)?;
                        self.keyboard_packets += 1;
                    }
                }
            }
            if ticks != 0 && self.world.is_some() && rebuild.is_none() && self.mouse_recorder.is_some() {
                if let Some(packet) = self.input_reports.focus(self.window_focused) {
                    self.session.send(packet.opcode, &packet.payload)?;
                    self.auxiliary_packets += 1;
                }
            }
            #[cfg(feature="native-profiling")]
            let scripts_started = Instant::now();
            let mut failures = Vec::new();
            let drained = drain_interface_events(&mut self.pending_script, client, &mut failures);
            // Failures before a queue error are still reported, in order.
            for failure in failures {self.report(failure);}
            let drained = drained?;
            #[cfg(feature="native-profiling")]
            if drained.steps != 0 {self.readiness_trace.slice(drained.steps,scripts_started.elapsed(),drained.deferred);}
            #[cfg(not(feature="native-profiling"))]
            let _ = drained;
            if ticks!=0 && rebuild.is_none() {self.dispatch_scene_walks(client)?;}
            while let Some(event) = client.outbound.pop_event() {
                // #181: copy the observed fields before the event moves into the session.
                #[cfg(feature="private-test-adapter")]
                let observed = match &event {
                    crate::outbound::Event::Packet(packet) => crate::client_journal::DequeueEvent::Packet {
                        opcode: packet.opcode,
                        payload_len: u32::try_from(packet.payload.len()).unwrap_or(u32::MAX),
                    },
                    crate::outbound::Event::DiscardCipher => crate::client_journal::DequeueEvent::DiscardCipher,
                };
                let staged = self.session.send_event(event);
                #[cfg(feature="private-test-adapter")]
                if let JournalSlot::Recording(recorder) = &mut self.journal {
                    use crate::client_journal::StageOutcome;
                    recorder.dequeue(observed, if staged.is_ok() { StageOutcome::Staged } else { StageOutcome::RefusedOther });
                }
                staged?;
            }
            if let Err(error) = self.session.advance_keepalive(ticks, self.world.is_some() && rebuild.is_none()) {
                self.packet_failure(client, error)?;
                return Ok(());
            }
        }
        if let Some(rebuild) = rebuild {
            self.next_rebuild = Some(rebuild);
            // GameScene.ai clears geometry inside the existing root scene.
            // Keep its selection through loading, including a superseding rebuild.
            if let Some((scene, _)) = self.world.take() {
                self.rebuild_walk = Some(scene.walk);
            }
            self.minimap = None;
        #[cfg(feature = "custom-assets")]
        {self.minimap_geography=None;}
            self.minimap_icons.clear();
        self.minimap_icons_pending=false;
            self.pick = None;
            #[cfg(feature="private-test-adapter")]
            self.draw_custody.boundary(crate::draw_custody::Boundary::Rebuild, false);
        }
        if self.world_job.as_ref().is_some_and(|job| job.is_finished()) {
            let result = self
                .world_job
                .take()
                .unwrap()
                .join()
                .map_err(|_| Error::Io("world loader thread panicked".into()))?;
            if self.next_rebuild.is_none() {
                #[cfg(not(feature = "custom-assets"))]
                let (mut scene, mesh, minimap, icons) = result?;
                #[cfg(feature = "custom-assets")]
                let (mut scene, mesh, minimap, icons, geography) = result?;
                #[cfg(feature = "custom-assets")]
                {self.minimap_geography=geography;}
                if let Some(walk) = self.rebuild_walk.take() {scene.walk = walk;}
                println!(
                    "Live world built: {} objects, {} triangles, {} missing regions",
                    mesh.object_instances,
                    mesh.triangles.len(),
                    scene.missing_regions.len()
                );
                self.locations.rebuild([scene.base_x, scene.base_y]);
                self.world = Some((scene, mesh));
                self.world_cycle = client.actor_cycle;
                #[cfg(feature="private-test-adapter")]
                self.draw_custody.boundary(crate::draw_custody::Boundary::Installed, true);
                self.minimap = Some(minimap);
                self.minimap_icons = icons;
                self.minimap_icons_pending = false;
            }
        }
        if self.world_job.is_none() {
            if let Some(rebuild) = self.next_rebuild.take() {
                let mut cache = client.scripts.source.cache.reopen()?;
                cache.wait_for_recovery(true);
                let plane = client.data.position[0] as usize;
                let assets = self.minimap_assets.clone();
                let varps = client.scripts.variables.player_values().to_vec();
                self.world_job = Some(std::thread::spawn(move || {
                    let start = Instant::now();
                    let scene = match rebuild {
                        Ok(normal) => Scene::load_normal(&mut cache, &normal)?,
                        Err(instance) => Scene::load_instance(&mut cache, &instance)?,
                    };
                    let mesh = WorldMesh::build_with_varps(&scene, &mut cache, 3, &varps)?;
                    let floors = crate::floor::Floors::load(&mut cache)?;
                    let textures = Definition::load_all(&mut cache)?;
                    let mut image =
                        crate::minimap::terrain(&scene, &floors, &textures, plane, 0, 0)?;
                    #[cfg(feature = "custom-assets")]
                    let geography=crate::environment_minimap::scene_geography(&scene,&mut cache)?;
                    #[cfg(feature = "custom-assets")]
                    if plane==0 {crate::environment_minimap::compose_geography(&mut image,geography.as_deref())?;}
                    let defs = crate::object::Definition::load_all(&mut cache)?;
                    crate::minimap::objects(&scene, &defs, &assets.scenes, plane, &mut image)?;
                    let minimap = std::sync::Arc::new(crate::sprite_pixels::Pixels {
                        canvas: [image.width(), image.height()],
                        offset: [0, 0],
                        size: [image.width(), image.height()],
                        pixels: image.pixels().to_vec(),
                    });
                    println!(
                        "World background load: {:.3}s",
                        start.elapsed().as_secs_f64()
                    );
                    let icons = crate::minimap::icons(&scene, &defs, &mut cache, plane)?;
                    #[cfg(feature = "custom-assets")]
                    let icons={let mut icons=icons;crate::environment_minimap::append_map_icons(&scene,&mut cache,plane,&mut icons)?;icons};
                    #[cfg(feature = "custom-assets")]
                    {Ok((scene,mesh,minimap,icons,geography))}
                    #[cfg(not(feature = "custom-assets"))]
                    {Ok((scene,mesh,minimap,icons))}
                }));
            }
        }
        self.apply_locations(client)?;
        Ok(())
    }
    fn apply_locations(&mut self, client: &mut ClientState) -> Result<()> {
        let Some((scene, mesh)) = &mut self.world else {
            return Ok(());
        };
        #[cfg(feature = "custom-assets")]
        mesh.environment_motion_active(self.window_focused);
        let cache = &mut client.scripts.source.cache;
        let ticks = client.actor_cycle.wrapping_sub(self.world_cycle).max(0) as u32;
        self.world_cycle = client.actor_cycle;
        let transformed = mesh.pending_assets || mesh.transforms_changed(client.scripts.variables.player_values());
        if client.zone.locations.is_empty() && self.locations.entries.is_empty() && !transformed && !self.minimap_icons_pending {
            return mesh.advance(scene, cache, &mut self.actors.animations, ticks);
        }
        if self.location_definitions.is_none() {
            self.location_definitions = Some(crate::object::Definition::load_all(cache)?);
        }
        let definitions = self.location_definitions.as_ref().unwrap();
        let renderer=scene.definition_renderer();
        renderer.validate(definitions)?;
        let location_models=&mut self.actors.location_models;
        let mut animated = false;
        while let Some(event) = client.zone.locations.pop_front() {
            match event {
                crate::location_update::Event::Merge(origin, request) => {
                    let Some(player) = client
                        .player_state
                        .as_mut()
                        .and_then(|s| s.root_actor_mut(request.player))
                    else {
                        continue;
                    };
                    let def = definitions
                        .get(&(request.object as u32))
                        .ok_or(Error::Invalid("location merge definition"))?;
                    let key=request.key(origin)?;
                    let selected=renderer.get(key[0] as usize,[key[1]-scene.base_x,key[2]-scene.base_y],def);
                    let def=selected.as_ref();
                    let Some(model) = location_models.object(
                        cache,
                        def,
                        request.shape,
                        request.rotation,
                    )?
                    else {
                        continue;
                    };
                    let attachment = crate::location_merge::Attachment::prepare(
                        request,
                        origin,
                        client.actor_cycle,
                        scene,
                        def,
                        (*model).clone(),
                    )?;
                    let key = request.key(origin)?;
                    self.locations
                        .merge(key, request.start, request.end, |key| scene.location(key));
                    // The event is received at this cycle, after any elapsed cycles.
                    // Earlier ticks in this batch must not consume its lifetime.
                    let pending = self.locations.entries.get_mut(&key).unwrap();
                    pending.delay = request.start as i32 + ticks as i32;
                    pending.lifetime = request.end as i32 + ticks as i32;
                    player.attachment = Some(attachment);
                }
                crate::location_update::Event::Animate(origin, request) => {
                    let key = request.key(origin)?;
                    let applied = scene.animate_location(key, request, definitions)?;
                    animated |= applied;
                    if let Some(pending) = self.locations.entries.get_mut(&key) {
                        let forced = definitions
                            .get(&(pending.replacement.id as u32))
                            .is_some_and(|d| d.flag90);
                        if forced || !applied {
                            pending.animation = request.sequence;
                        }
                    }
                }
                crate::location_update::Event::Change(origin, change) => {
                    self.locations
                        .apply(origin, change, |key| scene.location(key))?
                }
                crate::location_update::Event::Clear(origin) => self.locations.clear(origin),
            }
        }
        let changed = self.locations.advance(
            [scene.base_x, scene.base_y],
            ticks,
            |key, object, animation| {
                if object.id >= 0 {
                    let def = definitions.get(&(object.id as u32)).ok_or(Error::Invalid("pending location definition"))?;
                    let selected=renderer.get(key[0] as usize,[key[1]-scene.base_x,key[2]-scene.base_y],def);
                    if !crate::object_model::ready(cache, &selected, object.shape)? { return Ok(false); }
                }
                scene.replace_location(key, object, definitions)?;
                if animation != -1 {
                    scene.animate_location(
                        key,
                        crate::location_animation::Request {
                            at: 0,
                            shape: object.shape,
                            rotation: object.rotation,
                            sequence: animation,
                        },
                        definitions,
                    )?;
                }
                Ok(true)
            },
        )?;
        if changed > 0 || animated || transformed {
            let mut next = WorldMesh::build_with_varps(
                scene,
                cache,
                3,
                client.scripts.variables.player_values(),
            )?;
            next.retain_clocks(mesh, &self.actors.animations);
            *mesh = next;
        }
        mesh.advance(scene, cache, &mut self.actors.animations, ticks)?;
        if changed > 0 {
            let plane = client.data.position[0] as usize;
            let floors = crate::floor::Floors::load(cache)?;
            let textures = Definition::load_all(cache)?;
            let mut image = crate::minimap::terrain(scene, &floors, &textures, plane, 0, 0)?;
            #[cfg(feature = "custom-assets")]
            if plane==0 {crate::environment_minimap::compose_geography(&mut image,self.minimap_geography.as_deref())?;}
            crate::minimap::objects(
                scene,
                definitions,
                &self.minimap_assets.scenes,
                plane,
                &mut image,
            )?;
            self.minimap = Some(std::sync::Arc::new(crate::sprite_pixels::Pixels {
                canvas: [image.width(), image.height()],
                offset: [0, 0],
                size: [image.width(), image.height()],
                pixels: image.pixels().to_vec(),
            }));
            self.pick = None;
            #[cfg(feature="private-test-adapter")]
            self.draw_custody.locations_applied();
            println!(
                "Applied {changed} dynamic location changes; {} live scene objects",
                scene.objects.len()
            );
        }
        if changed > 0 || self.minimap_icons_pending {
            let next=(|| {
                let icons=crate::minimap::icons(scene,definitions,cache,client.data.position[0] as usize)?;
                #[cfg(feature = "custom-assets")]
                let icons={let mut icons=icons;crate::environment_minimap::append_map_icons(scene,cache,client.data.position[0] as usize,&mut icons)?;icons};
                Ok(icons)
            })();
            self.minimap_icons_pending=!crate::minimap::commit_icons(&mut self.minimap_icons,next)?;
        }
        Ok(())
    }
    fn refresh_brightness(&mut self,client:&mut ClientState)->Result<()> {
        let brightness=client.preferences.device.brightness;
        if self.brightness.to_bits()==brightness.to_bits() {return Ok(());}
        let palette=crate::floor::palette(brightness)?;
        let textures=self.textures.iter().map(|(&id,t)|Ok((id,Texture::load(&mut client.scripts.source.cache,t.definition.clone(),t.size,brightness)?))).collect::<Result<HashMap<_,_>>>()?;
        self.textures=textures;self.palette=palette;self.actors.set_palette(&self.palette);self.brightness=brightness;
        Ok(())
    }
    fn walk_instances(client: &ClientState) -> Vec<(u16,u64)> {
        let mut instances=vec![(0,0)];
        instances.extend(client.world_entities.values().chain(client.detached_world_entities.iter())
            .flat_map(|world|world.entities.iter()).filter(|(_,entity)|entity.registered)
            .map(|(&id,entity)|(id,entity.spawn_generation)));
        instances.sort_by_key(|&(id,generation)|(id&15,generation));
        instances
    }
    fn walk_scene_mut(&mut self, instance: (u16,u64)) -> Option<&mut Scene> {
        if instance==(0,0) {self.world.as_mut().map(|(scene,_)|scene)}
        else {self.entity_worlds.get_instance_mut(instance.0,instance.1).map(|(_,scene,_,_)|scene)}
    }
    fn apply_scene_walk_commands(&mut self, client: &mut ClientState) {
        let mut waiting=std::collections::VecDeque::new();
        while let Some(command)=client.scene_walk_commands.pop_front() {
            let instance=command.instance();
            if let Some(scene)=self.walk_scene_mut(instance) {scene.walk.apply(command);}
            else if instance==(0,0) || client.world_entities.values().chain(client.detached_world_entities.iter())
                .any(|world|world.entities.get(&instance.0).is_some_and(|e|e.spawn_generation==instance.1 && e.registered)) {
                // A newly registered view may not have prepared its native scene yet.
                waiting.push_back(command);
            }
        }
        client.scene_walk_commands=waiting;
    }
    /// GameClient's registered-scene movement phase, after a scene draw resolves selection.
    pub fn dispatch_scene_walks(&mut self, client: &mut ClientState) -> Result<usize> {
        // The original registered-scene movement phase is inside game state 30.
        if self.world.is_none() {return Ok(0);}
        self.apply_scene_walk_commands(client);
        let modifier=if client.input.held.contains(&82) {if client.input.held.contains(&81) {2}else{1}}else{0};
        let mut count=0;
        for instance in Self::walk_instances(client) {
            let Some(scene)=self.walk_scene_mut(instance) else {continue;};
            if !scene.walk.ready() {continue;}
            let tile=[scene.walk.selected[0].wrapping_add(scene.base_x) as u16,
                scene.walk.selected[1].wrapping_add(scene.base_y) as u16];
            let packet=crate::world_action::walk(tile[0],tile[1],modifier)?;
            scene.walk.clear();
            client.click_cross.show(crate::click_cross::Kind::Walk, client.input.press_position);
            client.map_flag=Some((tile[0] as u32)<<14|tile[1] as u32);
            #[cfg(feature="private-test-adapter")]
            eprintln!("Private scene walk queued: cycle={} view={:?} tile={tile:?} point={:?}",client.cycle,instance,client.input.press_position);
            client.outbound.push_back(packet);count+=1;
        }
        Ok(count)
    }
    pub fn paint(
        &mut self,
        client: &mut ClientState,
        width: usize,
        height: usize,
    ) -> Result<Raster> {
        // #175: every call is recorded; each return below states what its raster holds.
        #[cfg(feature="private-test-adapter")]
        self.draw_custody.paint_begin();
        #[cfg(feature="native-profiling")]
        let mut scene_profile=self.scene_profile.as_mut().and_then(|p|p.begin(client.cycle,client.interfaces.widgets.len()));
        let graphics_ticks=client.actor_cycle.wrapping_sub(self.graphics_cycle).max(0) as u32;
        self.graphics_cycle=client.actor_cycle;
        self.refresh_brightness(client)?;
        self.prune_entity_worlds(client);
        for world in client.world_entities.values().chain(client.detached_world_entities.iter()) {
            for (&id, entity) in &world.entities {
                if let Some(sprite) = entity
                    .definition
                    .as_ref()
                    .map(|d| d.sprite)
                    .filter(|&id| id >= 0)
                {
                    let sprite = sprite as u32;
                    if !self.entity_sprites.contains_key(&sprite) {
                        self.entity_sprites.insert(
                            sprite,
                            std::sync::Arc::new(crate::sprite_pixels::Pixels::load(
                                &mut client.scripts.source.cache,
                                sprite,
                            )?),
                        );
                    }
                }

                if self
                    .entity_worlds
                    .get_instance(id, entity.spawn_generation)
                    .is_none_or(|(generation, _, _, _)| *generation != (entity.spawn_generation, entity.rebuild_generation))
                {
                    self.entity_worlds.try_build((id, entity.spawn_generation), |previous| {
                        let mut scene = if let Some(rebuild) = &entity.rebuild {
                            Scene::load_world_entity(&mut client.scripts.source.cache, rebuild)?
                        } else {
                            Scene::empty_sized(0, 0, entity.dimensions[0] as usize, entity.dimensions[1] as usize)?
                        };
                        // Geometry rebuilds retain the owning view's GameScene state.
                        // An ID reused by a different spawn has no matching entry.
                        if let Some((_, previous, _, _)) = previous {
                            scene.walk = previous.walk.clone();
                        }
                        let mesh = WorldMesh::build_with_varps(
                            &scene,
                            &mut client.scripts.source.cache,
                            3,
                            client.scripts.variables.player_values(),
                        )?;
                        // Models::object can report unavailable models without an error.
                        // Do not publish that partial mesh as a completed rebuild.
                        if mesh.pending_assets { return Ok(None); }
                        Ok(Some(((entity.spawn_generation, entity.rebuild_generation), scene, mesh, client.actor_cycle)))
                    })?;
                }
            }
        }
        self.select_registered_scenes(client);
        self.apply_scene_walk_commands(client);
        // Java retains the open menu's top entry; otherwise rebuild under the pointer.
        let top = if let Some(menu) = &client.menu { menu.entries.last().map(|e| e.action) } else if !client.view_registry.ordered().into_iter().any(|view| client.interaction_policy.get(view as i16)[0] == 2) {
            None
        } else {
            let point = client.input.pointer;
            let mut entries = client.interface_menu(point)?;
            if entries.is_empty() { entries = self.menu_entries(client, point)?; }
            entries.last().map(|e| e.action)
        };
        let hovered = match top.map(|a| a.scene().1) { Some(crate::context_menu::Action::Heading(d)) => Some(d), _ => None };
        if self.world.is_none() {
            #[cfg(feature="private-test-adapter")]
            self.draw_custody.paint_blank(crate::draw_custody::Outcome::NoWorld);
            return Raster::new(width, height, 16 << 20);
        }
        let layout = client.interfaces.layout(client.canvas)?;
        let Some(view) = layout.order.iter().find_map(|id| {
            let w = &client.interfaces.widgets[id];
            let p = &layout.widgets[id];
            (w.content == 1337 && !p.hidden && p.geometry.size[0] > 0 && p.geometry.size[1] > 0)
                .then_some(p)
        }) else {
            #[cfg(feature="private-test-adapter")]
            self.draw_custody.paint_blank(crate::draw_custody::Outcome::NoViewport);
            return Raster::new(width, height, 16 << 20);
        };
        let shape = client.viewport.shape(view.geometry.size);
        self.actors.prepare_steering(client, shape, hovered);
        let origin = [
            view.absolute[0] + shape.offset[0],
            view.absolute[1] + shape.offset[1],
        ];
        // cm samples every registered owning view before scene traversal.
        let pointer = [client.input.pointer[0] - origin[0], client.input.pointer[1] - origin[1]];
        for instance in Self::walk_instances(client) {
            let plane = if instance == (0,0) {Some(client.data.position[0])} else {
                client.world_entities.values().chain(client.detached_world_entities.iter())
                    .find_map(|world|world.entities.get(&instance.0)
                        .filter(|entity|entity.registered && entity.spawn_generation==instance.1)
                        .map(|entity|entity.plane as i32))
            };
            if let (Some(plane),Some(scene)) = (plane,self.walk_scene_mut(instance)) {
                scene.walk.sample(plane,pointer,shape.size);
            }
        }
        let (scene,mesh) = self.world.as_mut().unwrap();
        #[cfg(feature="native-profiling")]
        let profiled_scene_triangles=mesh.triangles.len() as u64;
        client
            .camera
            .view_pose(shape.size[1], client.viewport.zoom.map(i32::from));
        let roof_plane = client.camera.roof_plane(
            scene, client.data.position[0] as u8, client.preferences.device.roofs_hidden,
        );
        let mut entropy = [0; 40];
        if client.camera.shake.iter().any(|shake| shake.requested) {
            getrandom::getrandom(&mut entropy).map_err(|e| Error::Io(e.to_string()))?;
        }
        let mut samples = entropy.chunks_exact(8).map(|bytes| {
            (u64::from_le_bytes(bytes.try_into().unwrap()) >> 11) as f64 / (1u64 << 53) as f64
        });
        let (eye, [pitch, yaw]) = client.camera.shaken_pose(|| samples.next().unwrap());
        let mut camera = Camera::root_scene(eye, pitch, yaw, shape.focal as f32, [scene.width, scene.height])?;
        camera.far = client.preferences.device.far_clip();
        #[cfg(feature = "custom-assets")]
        mesh.update_environment_cutaway(&camera,[client.camera.tracked[0]as f32,client.camera.tracked_height as f32,client.camera.tracked[1]as f32]);
        let mut frame = Frame::new(shape.size[0] as usize, shape.size[1] as usize)?;
        #[cfg(feature="native-profiling")]
        crate::ui_profile::Sample::mark(&mut scene_profile);
        let mut child_object_calls = Vec::new();
        let mut child_visibility = crate::scene_visibility::Visibility::new(shape.size,
            client.viewport.zoom.map(i32::from),client.preferences.device.draw_distance.clamp(25,90),
            [((pitch.clamp(1,4160)-1)/256) as usize,((yaw&16383)/1024) as usize])?;
        let mut drawn_walk_views=vec![((0,0),client.data.position[0])];
        client.skybox.load(&mut client.scripts.source.cache)?;
        if let Some(triangles) = &client.skybox.triangles {
            frame.draw_background(&camera, triangles, &self.palette, &self.textures);
        }
        #[cfg(feature="enhanced-renderer")]
        let enhanced_background = (client.skybox.triangles.is_some()
            && self.enhanced.as_ref().is_some_and(|renderer|renderer.enabled()))
            .then(||frame.pixels.clone());
        #[cfg(feature="enhanced-renderer")]
        if self.enhanced.as_ref().is_some_and(|renderer|renderer.enabled()) {
            frame.begin_presentation_capture();
        }
        #[cfg(feature="enhanced-renderer")]
        let child_shadow_light=self.enhanced.as_ref().filter(|r|r.enabled()&&r.settings().shadows>0).map(|r|r.settings().light);
        #[cfg(feature="enhanced-renderer")]
        let mut child_shadow_triangles=Vec::new();
        #[cfg(feature="enhanced-renderer")]
        let mut child_shadow_error=None;
        let mut scene_triangles = Vec::new();
        self.entity_npc_definitions.clear();
        let mut child_overlays = Vec::new();
        let local_view = client.player_state.as_ref().map_or(0, |s| s.local_view);
        let child_ids = crate::world_overlap::frame_order(&client.world_entities, local_view, client.world_entity_limit.get())?;
        let child_scenes=self.entity_worlds.iter().map(|(&id, (_, scene, _, _))| (id, scene)).collect::<HashMap<_,_>>();
        let mut root_events=client.advance_projectiles_in_views(scene,&child_scenes,graphics_ticks)?;
        root_events.extend(client.advance_root_graphics(graphics_ticks)?);
        let mut root_graphic_sounds=Some(root_events);
        let root_items=crate::item_submission::sources(&client.zone,scene,&mesh.item_support);
        scene.sync_item_allocations(client.zone.scene_allocations.iter().copied());
        scene.sync_item_piles(&root_items.iter().map(|source|source.pile).collect::<Vec<_>>());
        let root_item_keys=crate::item_submission::keys(scene,&root_items);
        let root_children=self.actors.root_child_submissions(client,scene)?;
        let mut late_entities = HashSet::new();
        let mut late_triangles = Vec::new();
        let mut root_actor_bounds = None;
        self.entity_overlap_hidden.retain(|id| {
            client
                .world_entities
                .values()
                .any(|s| s.entities.contains_key(id))
        });
        self.entity_submitted_cycle.retain(|id, cycle| {
            *cycle == client.actor_cycle
                && client
                    .world_entities
                    .values()
                    .any(|s| s.entities.contains_key(id))
        });
        let parent_actors: HashMap<_, _> = client
            .world_entities
            .keys()
            .map(|&parent| (parent, crate::world_overlap::parent_actors(client, parent)))
            .collect();
        let mut transforms: HashMap<u16, (crate::world_entity::SceneTransform, u8)> =
            HashMap::new();
        {
            for (parent, id) in &child_ids {
                let late = (*parent == 0 && *id != local_view && client.world_entities[parent].entities[id].owner_type == 1)
                    || late_entities.contains(parent);
                if late {
                    if let Some(events)=root_graphic_sounds.take() {client.enqueue_sequence_sounds(events)?;}
                    late_entities.insert(*id);
                    if *parent == 0 && root_actor_bounds.is_none() {
                        root_actor_bounds = Some(crate::world_overlap::visible_bounds(
                            client.world_entities.get(&0),
                            client.actor_cycle,
                            &self.entity_submitted_cycle,
                            &self.entity_overlap_hidden,
                        ));
                    }
                }
                let entity = &client.world_entities[parent].entities[id];
                let (parent_scene, parent_transform, plane) = if *parent == 0 {
                    (
                        &*scene,
                        crate::world_entity::SceneTransform::default(),
                        client.data.position[0] as u8,
                    )
                } else {
                    let Some((transform, plane)) = transforms.get(parent) else {
                        continue;
                    };
                    let Some((_, scene, _, _)) = self.entity_worlds.get(parent) else {
                        continue;
                    };
                    (scene, transform.clone(), *plane)
                };
                let ground = parent_scene
                    .elevation(plane as usize, entity.pose.x, entity.pose.z)
                    .unwrap_or(0);
                let overlap_color = entity.definition.as_ref().map_or(39188, |d| d.color);
                let bounds = entity.definition.as_ref().map_or([0; 4], |d| d.bounds);
                let position = [entity.pose.x, entity.pose.z];
                let hidden = *id != local_view
                    && entity.owner_type == 1
                    && crate::world_overlap::obscured(
                        bounds,
                        position,
                        entity.pose.angle,
                        parent_actors[parent].iter().copied(),
                        crate::world_overlap::submitted_boxes(
                            &client.world_entities[parent],
                            *id,
                            client.actor_cycle,
                            &self.entity_submitted_cycle,
                        ),
                    );
                if hidden {
                    self.entity_overlap_hidden.insert(*id);
                } else {
                    self.entity_overlap_hidden.remove(id);
                }
                self.entity_submitted_cycle.insert(*id, client.actor_cycle);
                // cf.cu stamps the child before parent Scene.bs can reject it.
                let admitted=*parent!=0||root_children.contains(id);
                let transform = entity
                    .geometry_transform(ground)
                    .with_overlap_hidden(hidden);
                let animation = self
                    .actors
                    .animations
                    .world_entity_matrix(&mut client.scripts.source.cache, entity)?;
                let mut transform =
                    crate::world_entity::SceneTransform(vec![(transform, animation)]);
                transform.0.extend(parent_transform.0);
                let child_plane=entity.plane;
                let walk_instance=(*id,entity.spawn_generation);
                #[cfg(feature="private-test-adapter")]
                let child_rebuild_generation=entity.rebuild_generation;
                let Some((_, child_scene, child_mesh, cycle)) = self.entity_worlds.get_instance_mut(*id, entity.spawn_generation)
                else {
                    continue;
                };
                let ticks = client.actor_cycle.wrapping_sub(*cycle).max(0) as u32;
                *cycle = client.actor_cycle;
                if self.location_definitions.is_none() {
                    self.location_definitions = Some(crate::object::Definition::load_all(
                        &mut client.scripts.source.cache,
                    )?);
                }
                self.actors.child_locations(
                    client,
                    child_scene,
                    child_mesh,
                    *id,
                    ticks,
                    self.location_definitions.as_ref().unwrap(),
                )?;
                let item_sources=client.world_entities.get(&0).and_then(|world|world.entities.get(id))
                    .map(|entity|crate::item_submission::sources(&entity.zone,child_scene,&child_mesh.item_support)).unwrap_or_default();
                if let Some(entity)=client.world_entities.get(&0).and_then(|world|world.entities.get(id)) {
                    child_scene.sync_item_allocations(entity.zone.scene_allocations.iter().copied());
                }
                child_scene.sync_item_piles(&item_sources.iter().map(|source|source.pile).collect::<Vec<_>>());
                let item_keys=crate::item_submission::keys(child_scene,&item_sources);
                if !admitted {continue;}
                drawn_walk_views.push((walk_instance,child_plane as i32));
                // #175: admission/projection of a child view, not proof of its pixels.
                #[cfg(feature="private-test-adapter")]
                self.draw_custody.child_view_admitted(*id,walk_instance.1,child_rebuild_generation);
                frame.child_projection(*id, transform.matrix(), eye, [pitch, yaw]);
                let child_object_batch_base=if late {late_triangles.len()}else{scene_triangles.len()};
                transforms.insert(*id, (transform.clone(), child_plane));
                #[cfg(feature="enhanced-renderer")]
                if child_shadow_error.is_none() {if let Some(light)=child_shadow_light {
                    child_shadow_error=crate::enhanced::append_transformed_shadow_casters(&mut child_shadow_triangles,
                        child_mesh.visible_triangles(child_plane),&transform,
                        [client.camera.tracked[0]as f32,client.camera.tracked_height as f32,client.camera.tracked[1]as f32],light).err();
                }}
                let triangles = child_mesh
                    .triangles
                    .iter()
                    .map(|triangle| {
                        let mut transformed = *triangle;
                        transformed.view = *id;
                        transformed.vertices =
                            triangle.vertices.map(|point| transform.point(point));
                        if hidden {
                            transformed.scene_tint(overlap_color, &self.palette);
                        }
                        transformed
                    })
                    .collect::<Vec<_>>();
                if late {
                    crate::model_order::append(&mut late_triangles, triangles);
                } else {
                    crate::model_order::append(&mut scene_triangles, triangles);
                }
                self.actors.overlap_bounds = crate::world_overlap::visible_bounds(
                    client.world_entities.get(&0),
                    client.actor_cycle,
                    &self.entity_submitted_cycle,
                    &self.entity_overlap_hidden,
                );
                client.draw_child_graphics(*id,graphics_ticks,admitted,hidden)?;
                if let Some(local_eye)=frame.child_eye(*id) {
                    let child_view=crate::terrain_submission::View {
                        root:false,size:[child_scene.width as i32,child_scene.height as i32],
                        camera:[local_eye[0]>>7,local_eye[2]>>7],eye_y:local_eye[1],
                        radius:client.preferences.device.draw_distance.clamp(25,90),maximum_plane:child_plane as usize,
                    };
                    let (temporary,actor_tags)=if hidden {(Vec::new(),Vec::new())}else{self.actors.temporary_draw_objects(client,child_scene,Some(*id))?};
                    let (spots,spot_keys)=self.actors.attached_spot_objects(client,Some(*id),&temporary,&actor_tags);
                    child_scene.sync_terrain_calls(&child_mesh.terrain_submissions);
                    if let Some(calls)=child_scene.draw_render_calls_with_spots(&child_view,&mut child_visibility,[local_eye[0],local_eye[2]],&temporary,&spots) {
                        let base=child_object_batch_base;
                        let models=child_mesh.object_submissions.iter().filter_map(|submitted|
                            submitted.traversal_id.map(|identity|(identity,child_mesh.triangles[submitted.batch].tag,submitted.batch+base)))
                            .collect::<Vec<_>>();
                        let boundaries=child_mesh.boundary_submissions.iter().map(|submitted|
                            (submitted.identity,child_mesh.triangles[submitted.batch].tag,submitted.batch+base)).collect::<Vec<_>>();
                        let decorations=child_mesh.decoration_submissions.iter().map(|submitted|
                            (submitted.identity,child_mesh.triangles[submitted.batch].tag,submitted.batch+base)).collect::<Vec<_>>();
                        let ground=child_mesh.ground_submissions.iter().map(|submitted|
                            (submitted.identity,child_mesh.triangles[submitted.batch].tag,submitted.batch+base)).collect::<Vec<_>>();
                        let terrain=child_mesh.terrain_submissions.iter().map(|&s|(s.identity([child_scene.width,child_scene.height]),s.tag())).collect::<Vec<_>>();
                        child_object_calls.push((*id,late,models,boundaries,decorations,ground,terrain,item_keys,actor_tags,spot_keys,calls));
                    }
                }
                let (mut child_actors, pending) = if hidden {
                    (Vec::new(), Vec::new())
                } else {
                    self.actors.npc_triangles(client, child_scene, Some(*id))?
                };
                if !hidden {
                    crate::model_order::append(&mut child_actors, self.actors.player_triangles(
                        client,
                        child_scene,
                        Some(*id),
                    )?);
                }
                let (ground, ground_pending) = self.actors.ground_triangles(
                    client,
                    child_scene,
                    &child_mesh.item_support,
                    Some(*id),
                )?;
                crate::model_order::append(&mut child_actors, ground);
                for warning in pending.into_iter().chain(ground_pending) {
                    eprintln!("Child world {id}: {warning}");
                }
                for triangle in &mut child_actors {
                    triangle.view = *id;
                    triangle.vertices = triangle.vertices.map(|point| transform.point(point));
                    if hidden {
                        triangle.scene_tint(overlap_color, &self.palette);
                    }
                }
                #[cfg(feature="enhanced-renderer")]
                if child_shadow_error.is_none() {if let Some(light)=child_shadow_light {
                    // Actor vertices above already use the composed child-to-root transform.
                    child_shadow_error=crate::enhanced::append_transformed_shadow_casters(&mut child_shadow_triangles,
                        child_actors.iter(),&crate::world_entity::SceneTransform::default(),
                        [client.camera.tracked[0]as f32,client.camera.tracked_height as f32,client.camera.tracked[1]as f32],light).err();
                }}
                if late {
                    crate::model_order::append(&mut late_triangles, child_actors);
                } else {
                    crate::model_order::append(&mut scene_triangles, child_actors);
                }
                if !hidden {
                    child_overlays.push((*id, transform, self.actors.overlay_heights(client,Some(*id))));
                    self.entity_npc_definitions
                        .insert(*id, self.actors.visible_npcs.clone());
                }
            }
        }
        if let Some(events)=root_graphic_sounds.take() {client.enqueue_sequence_sounds(events)?;}
        self.actors.overlap_bounds = root_actor_bounds.unwrap_or_else(|| {
            crate::world_overlap::visible_bounds(
                client.world_entities.get(&0),
                client.actor_cycle,
                &self.entity_submitted_cycle,
                &self.entity_overlap_hidden,
            )
        });
        let (actors, pending) = self.actors.triangles(client, scene, &mesh.item_support)?;
        #[cfg(feature="native-profiling")]
        crate::ui_profile::Sample::mark(&mut scene_profile);
        // GameScene completes phase zero across the scene before its alpha pass.
        // Keep each view's submission order while retaining actors for diagnostics.
        let mut actors = actors;
        let root_batch_count = mesh.triangles.len();
        crate::model_order::rebase(&mut scene_triangles, root_batch_count);
        let actor_batch_base = root_batch_count + scene_triangles.len();
        crate::model_order::rebase(&mut actors, actor_batch_base);
        let late_batch_base = actor_batch_base + actors.len();
        crate::model_order::rebase(&mut late_triangles, late_batch_base);
        for (view,late,models,boundaries,decorations,ground,terrain,item_keys,actor_tags,spot_keys,calls) in child_object_calls {
            let bodies=crate::object_submission::actor_body_batches(scene_triangles.iter().chain(late_triangles.iter()),view);
            crate::object_submission::register_calls_with_terrain(&mut frame,view,models,boundaries,decorations,ground,
                crate::item_submission::batches(&item_keys,scene_triangles.iter().chain(late_triangles.iter()),view),Some(terrain),bodies,
                crate::object_submission::actor_spot_batches(scene_triangles.iter().chain(late_triangles.iter()),view),
                crate::object_submission::temporary_effect_batches(scene_triangles.iter().chain(late_triangles.iter()),view),&actor_tags,&spot_keys,&[],&calls,
                if late {late_batch_base}else{root_batch_count});
        }
        crate::world_actors::register_root_order(&mut frame,&camera,client,scene,&actors);
        self.actors.register_submission_order(&mut frame,client,scene,None)?;
        let drawn_instances=client.world_entities.get(&0).into_iter()
            .flat_map(|world|world.entities.iter().map(|(&id,entity)|(id,entity.spawn_generation))).collect::<Vec<_>>();
        for (id,generation) in drawn_instances {
            let Some((_,child_scene,_,_))=self.entity_worlds.get_instance(id,generation) else {continue;};
            if frame.child_eye(id).is_some() {
                crate::world_actors::register_view_order(&mut frame,&camera,client,child_scene,
                    scene_triangles.iter().chain(late_triangles.iter()),id);
                self.actors.register_submission_order(&mut frame,client,child_scene,Some(id))?;
            }
        }
        let terrain_view = crate::terrain_submission::View {
            root: true, size: [scene.width as i32, scene.height as i32],
            camera: [camera.position[0] as i32 >> 7, camera.position[2] as i32 >> 7],
            eye_y: camera.position[1] as i32,
            radius: client.preferences.device.draw_distance.clamp(25, 90),
            maximum_plane: roof_plane as usize,
        };
        let visibility = self.terrain_visibility.prepare(shape.size,
            client.viewport.zoom.map(i32::from), terrain_view.radius,
            [((pitch.clamp(1, 4160) - 1) / 256) as usize, ((yaw & 16383) / 1024) as usize])?;
        let (temporary,actor_tags,child_views)=self.actors.temporary_scene_objects(client,scene,None)?;
        let (spots,spot_keys)=self.actors.attached_spot_objects(client,None,&temporary,&actor_tags);
        scene.sync_terrain_calls(&mesh.terrain_submissions);
        let object_calls = scene.draw_render_calls_with_spots(&terrain_view,visibility,
            [camera.position[0] as i32,camera.position[2] as i32],&temporary,&spots);
        if let Some(calls)=&object_calls {
            crate::object_submission::register_calls_with_terrain(&mut frame,0,
                mesh.object_submissions.iter().filter_map(|submitted|
                    submitted.traversal_id.map(|id|(id,mesh.triangles[submitted.batch].tag,submitted.batch))),
                mesh.boundary_submissions.iter().map(|submitted|
                    (submitted.identity,mesh.triangles[submitted.batch].tag,submitted.batch)),
                mesh.decoration_submissions.iter().map(|submitted|
                    (submitted.identity,mesh.triangles[submitted.batch].tag,submitted.batch)),
                mesh.ground_submissions.iter().map(|submitted|
                    (submitted.identity,mesh.triangles[submitted.batch].tag,submitted.batch)),
                crate::item_submission::batches(&root_item_keys,actors.iter(),0),
                Some(mesh.terrain_submissions.iter().map(|&s|(s.identity([scene.width,scene.height]),s.tag())).collect()),
                crate::object_submission::actor_body_batches(actors.iter(),0),
                crate::object_submission::actor_spot_batches(actors.iter(),0),
                crate::object_submission::temporary_effect_batches(actors.iter(),0),&actor_tags,&spot_keys,&child_views,calls,0);
        }
        let mut object_batches = HashSet::new();
        for submitted in &mesh.object_submissions {
            let first=&mesh.triangles[submitted.batch];
            if object_calls.is_none() || submitted.traversal_id.is_none() {
                let phases=crate::object_submission::scene_object(&terrain_view,visibility,scene,&submitted.object,submitted.size);
                frame.model_admission(0,first.tag,submitted.batch,phases);
            }
            let object=&submitted.object;
            let tile=[object.x as usize,object.y as usize];
            let bridge=scene.tile(1,tile[0],tile[1]).is_some_and(|tile|tile.flags&2!=0);
            frame.model_opaque_order(0,first.tag,submitted.batch,
                crate::object_scene_order::key(object.plane as usize,tile,bridge,object.runtime));
            object_batches.insert(submitted.batch);
        }
        object_batches.extend(mesh.boundary_submissions.iter().map(|submitted|submitted.batch));
        object_batches.extend(mesh.decoration_submissions.iter().map(|submitted|submitted.batch));
        object_batches.extend(mesh.ground_submissions.iter().map(|submitted|submitted.batch));
        #[cfg(feature="custom-assets")]
        mesh.register_authored_art(&mut frame);
        #[cfg(all(feature="native-profiling",feature="enhanced-renderer"))]
        if self.enhanced_trace.is_some() { frame.begin_submission_profile(); }
        let mut terrain_admission = HashMap::new();
        #[cfg(feature="native-profiling")]
        let scene_filter={
            #[cfg(feature="custom-assets")]
            let candidates=scene.objects.iter().chain(scene.retained_objects().iter().map(|old|&old.object)).map(|object| {
                let tag=((object.id as u64+1)<<32)|((object.plane as u64)<<28)|((object.x as u64)<<14)|object.y as u64;
                (object.id,tag,scene.authored_diagnostic_placement(object))
            });
            #[cfg(not(feature="custom-assets"))]
            let candidates=std::iter::empty();
            let filter=crate::benchmark_scene::Filter::new(self.scene_filter,candidates);
            self.scene_filter_report=if self.scene_filter==crate::benchmark_scene::Mode::Full {
                crate::benchmark_scene::Report::full(mesh.triangles.len())
            }else{filter.report(mesh.triangles.iter().map(|t|(t.view,t.tag)))};
            filter
        };
        #[cfg(feature="native-profiling")]
        let excluded_draw_faces=std::cell::Cell::new(0);
        frame.draw_iter(&camera, mesh.visible_triangles_or(roof_plane, |triangle| (object_calls.is_some() && triangle.tag & (1<<63)!=0) || triangle.model_order.is_some_and(|order|object_batches.contains(&order.batch))).filter(|triangle| {
            #[cfg(feature="native-profiling")]
            if !scene_filter.allows(triangle.view,triangle.tag) {excluded_draw_faces.set(excluded_draw_faces.get()+1);return false;}
            if triangle.tag & (1 << 63) == 0 || object_calls.is_some() { return true; }
            *terrain_admission.entry(triangle.tag).or_insert_with(||
                terrain_view.terrain_tag(visibility, scene, triangle.tag))
        }).chain(scene_triangles.iter()).chain(actors.iter()).chain(late_triangles.iter()),
            &self.palette, &self.textures);
        #[cfg(feature="native-profiling")]
        {self.scene_filter_report.excluded_draw_faces=excluded_draw_faces.get();}
        #[cfg(feature="enhanced-renderer")]
        if let Some(renderer)=self.enhanced.as_mut().filter(|renderer|renderer.enabled()) {
            #[cfg(feature="native-profiling")]
            crate::ui_profile::Sample::mark(&mut scene_profile);
            let mut retained_faces=0;
            let result=frame.take_presentation_geometry().map_err(|e|format!("{e:?}"))
                .and_then(|geometry| {
                    if let Some(error)=child_shadow_error {return Err(error);}
                    let geometry=geometry.ok_or("Missing Classic presentation capture")?;
                    let triangles=geometry.iter().collect::<Vec<_>>();
                    let scenery=scene.objects.iter().chain(scene.retained_objects().iter().map(|old|&old.object))
                        .map(|obj|(((obj.id as u64+1)<<32)|((obj.plane as u64)<<28)|((obj.x as u64)<<14)|obj.y as u64,0)).collect();
                    let stable_count=crate::enhanced::opaque_scenery_prefix(&triangles,&scenery);
                    retained_faces=stable_count;
                    let casters=if renderer.settings().shadows>0 {
                        crate::enhanced::root_shadow_casters(mesh.visible_triangles(roof_plane)
                            .filter(|triangle| {
                                #[cfg(feature="native-profiling")]
                                {
                                    let allowed=scene_filter.allows(triangle.view,triangle.tag);
                                    if !allowed {self.scene_filter_report.excluded_shadow_candidates+=1;}
                                    return allowed;
                                }
                                #[cfg(not(feature="native-profiling"))]
                                {let _=triangle;true}
                            })
                            .chain(scene_triangles.iter()).chain(actors.iter()).chain(late_triangles.iter()).chain(child_shadow_triangles.iter()))
                    } else {Vec::new()};
                    renderer.render_with_shadow_casters(crate::enhanced::SceneFrame {
                        camera:&camera, triangles:&triangles, stable_count, stable_revision:None,
                        albedo:None, palette:&self.palette, textures:&self.textures,
                        size:[frame.width as u32,frame.height as u32],
                        light_focus:[client.camera.tracked[0]as f32,client.camera.tracked_height as f32,client.camera.tracked[1]as f32],
                        ambient:64, contrast:768,
                    }, enhanced_background.as_deref(), &casters)
                });
            match result {
                Ok(pixels) if pixels.len()==frame.pixels.len() => {
                    // Only display pixels change. All Classic pick/depth/model
                    // and terrain-hit data remain authoritative for dispatch.
                    frame.pixels=pixels;
                    #[cfg(feature="native-profiling")]
                    if let Some(error)=crate::enhanced_trace::Trace::complete(&mut self.enhanced_trace,crate::enhanced_trace::Sample {
                        cycle:client.cycle, eye:camera.position,
                        focus:[client.camera.tracked[0]as f32,client.camera.tracked_height as f32,client.camera.tracked[1]as f32],
                        child_casters:child_shadow_triangles.len(), stats:&renderer.stats,
                        submission:frame.submission_profile(),
                    }) {eprintln!("Enhanced trace disabled: {error}");}
                    if renderer.stats.frames==1 || renderer.stats.frames%300==0 {
                        eprintln!("Explicit Enhanced world display: {} scenery_prefix={retained_faces}",renderer.stats.json());
                    }
                }
                Ok(_) => renderer.disable("Enhanced display dimensions changed".into()),
                Err(reason) => renderer.disable(reason),
            }
            #[cfg(feature="native-profiling")]
            crate::ui_profile::Sample::mark(&mut scene_profile);
        }
        if !client.zone.piles.is_empty() && std::env::var_os("OSRS_GROUND_DIAGNOSTIC").is_some() {
            let path = std::path::Path::new("work/ground-scene-diagnostic.bmp");
            if !path.exists() {
                std::fs::write(
                    path,
                    Raster::from_pixels(frame.width, frame.height, frame.pixels.clone())?.bmp()?,
                )?;
                let ground = actors
                    .iter()
                    .filter(|t| t.tag & 0x1000000000000000 != 0)
                    .copied()
                    .collect::<Vec<_>>();
                let mut isolated = Frame::new(frame.width, frame.height)?;
                isolated.draw(&camera, &ground, &self.palette, &self.textures);
                eprintln!(
                    "Ground diagnostic: {} triangles, {} visible pixels, {} isolated pixels",
                    ground.len(),
                    frame
                        .tags
                        .iter()
                        .filter(|t| **t & 0x1000000000000000 != 0)
                        .count(),
                    isolated.tags.iter().filter(|t| **t != 0).count()
                );
                std::fs::write(
                    "work/ground-isolated-diagnostic.bmp",
                    Raster::from_pixels(isolated.width, isolated.height, isolated.pixels)?.bmp()?,
                )?;
            }
        }
        let mut overlays =
            Raster::from_pixels(frame.width, frame.height, std::mem::take(&mut frame.pixels))?;
        let child_views = child_overlays
            .iter()
            .filter_map(|(id, transform, heights)| {
                let generation=client.world_entities.get(&0)?.entities.get(id)?.spawn_generation;
                self.entity_worlds.get_instance(*id,generation).map(|(_, scene, _, _)| {
                    crate::overlay_render::ChildView {
                        id: *id,
                        scene,
                        transform,
                        heights,
                    }
                })
            })
            .collect::<Vec<_>>();
        let overlay_heights=self.actors.overlay_heights(client,None);
        self.overlays.draw_at_views(
            client,
            scene,
            &camera,
            &overlay_heights,
            &mut overlays,
            origin[1],
            &child_views,
        )?;
        if matches!(client.hint.kind,2|4) && client.hint.flashes(client.actor_cycle) {
            let views=client.view_registry.ordered().into_iter().filter_map(|id| {
                let s=if id==0 {&*scene} else {&self.entity_worlds.get(&id)?.1};
                Some(crate::projectile_world::View{id,base:[s.base_x,s.base_y],dimensions:[s.width as i32,s.height as i32]})
            }).collect::<Vec<_>>();
            let entities=client.world_entities.get(&0);
            let point=client.hint.scene_point(&views,entities,|id,p| {
                let (s,plane)=if id==0 {(&*scene,client.data.position[0] as usize)} else {
                    let s=&self.entity_worlds.get(&id)?.1;
                    let entity=entities?.entities.get(&id)?;
                    (s,entity.plane as usize)
                };
                Some(s.elevation(plane,p[0],p[1]).unwrap_or(0))
            });
            if let Some(point)=point.and_then(|p|crate::hint::project_scene(p,[scene.width as i32,scene.height as i32],eye,[pitch,yaw],shape.size,shape.focal)) {
                self.overlays.draw_scene_hint(&mut overlays,point)?;
            }
        }
        frame.pixels = overlays.pixels().to_vec();
        for message in pending {
            self.report(message);
        }

        let mut pixels = vec![0xff000000; width * height];
        for y in 0..frame.height {
            for x in 0..frame.width {
                let dx = origin[0] + x as i32;
                let dy = origin[1] + y as i32;
                if dx >= view.clip[0]
                    && dy >= view.clip[1]
                    && dx < view.clip[2]
                    && dy < view.clip[3]
                    && dx >= 0
                    && dy >= 0
                    && (dx as usize) < width
                    && (dy as usize) < height
                {
                    pixels[dy as usize * width + dx as usize] = frame.pixels[y * frame.width + x];
                }
            }
        }
        let root_base = self.world.as_ref().map(|(scene, _)| [scene.base_x, scene.base_y]).unwrap_or([0; 2]);
        let player = client.player_state.as_ref().and_then(|state| state.local_actor().map(|actor| {
            (state.local_view, [(actor.path.position[0] >> 7).wrapping_sub(root_base[0]),
                (actor.path.position[1] >> 7).wrapping_sub(root_base[1])])
        }));
        for (instance, level) in drawn_walk_views {
            let Some(scene)=self.walk_scene_mut(instance).filter(|scene|scene.walk.pending_pick) else {continue;};
            let hits=frame.terrain_hits_where(scene.walk.point, |view, tag| {
                view==instance.0 && scene.tile_render_plane(((tag>>28)&3) as usize,
                    ((tag>>14)&16383) as usize,(tag&16383) as usize).is_some_and(|plane|plane as i32<=level)
            });
            let tile=hits.get(&instance.0).map_or([-1;2],|&(tag,_)|crate::scene_walk::terrain_target(
                [((tag>>14)&16383) as i32,(tag&16383) as i32],instance==(0,0),player));
            scene.walk.finish_draw(tile);
        }
        // Steering retains the global eye; GameIntProjection model selection uses the root clamp.
        self.pick = Some((frame, origin, crate::viewport_steering::Ray {
            size: shape.size, focal: shape.focal, far: camera.far, angles: [pitch, yaw], eye,
            focus: [client.camera.tracked[0], client.camera.tracked_height, client.camera.tracked[1]],
        }, camera.position.map(|v| v as i32)));
        #[cfg(feature="native-profiling")]
        if let Some(error)=crate::ui_profile::Profile::complete(&mut self.scene_profile,scene_profile,profiled_scene_triangles) {
            eprintln!("Scene profiling disabled after output error: {error}");
        }
        let raster = Raster::from_pixels(width, height, pixels)?;
        // #175: only now does this call return world pixels (pick was set above).
        #[cfg(feature="private-test-adapter")]
        self.draw_custody.paint_drawn(client.actor_cycle);
        Ok(raster)
    }
    pub fn menu_entries(
        &mut self,
        client: &mut ClientState,
        point: [i32; 2],
    ) -> Result<Vec<crate::context_menu::Entry<crate::context_menu::Action>>> {
        let (world, low_priority, actors) = self.ranked_menu_entries(client, point)?;
        let mut entries = Vec::new();
        crate::scene_menu::append(&mut entries, &world, &actors);
        crate::menu_limits::limit_main(&mut entries);
        client.prioritize_menu(&mut entries, &low_priority);
        crate::menu_limits::limit_submenus(&mut entries);
        Ok(entries)
    }
    pub fn context_menu(
        &mut self,
        client: &mut ClientState,
        point: [i32; 2],
    ) -> Result<Vec<crate::context_menu::Entry<crate::context_menu::Action>>> {
        let (world, low_priority, actors) = self.ranked_menu_entries(client, point)?;
        client.interface_menu_with_world_groups(point, client.input.pointer, &world, &low_priority, &actors)
    }
    fn ranked_menu_entries(
        &mut self,
        client: &mut ClientState,
        point: [i32; 2],
    ) -> Result<(Vec<crate::context_menu::Entry<crate::context_menu::Action>>, Vec<crate::context_menu::Action>, Vec<crate::scene_menu::ActorContribution>)> {
        fn filter(entries: &mut Vec<crate::context_menu::Entry<crate::context_menu::Action>>, client: &ClientState, view: u16) -> Result<()> {
            use crate::context_menu::Action;
            for entry in entries.iter_mut() {
                if !matches!(entry.action, Action::Walk(_) | Action::Heading(_) | Action::InView { .. }) {
                    entry.action = entry.action.in_view(view);
                }
                filter(&mut entry.children, client, view)?;
            }
            let mut result=Vec::with_capacity(entries.len());
            for entry in entries.drain(..) { if client.scene_action_allowed(entry.action)? {result.push(entry);} }
            *entries=result;
            Ok(())
        }
        let layout = client.interfaces.layout(client.canvas)?;
        if crate::world_pointer::target(&client.interfaces, &layout, point, client.input.pointer, 1337).is_none() {
            return Ok((Vec::new(), Vec::new(), Vec::new()));
        }
        let mut candidates = Vec::new();
        if let Some((frame, origin, ray, model_eye)) = &self.pick {
            let local = [point[0] - origin[0], point[1] - origin[1]];
            candidates.extend(frame.scene_model_hits(local, crate::viewport_steering::Ray { eye: *model_eye, ..*ray }).into_iter().map(|(tag, view, _)| (tag, view)));
            if local[0] >= 0 && local[1] >= 0 && local[0] < frame.width as i32 && local[1] < frame.height as i32 {
                let pixel = local[1] as usize * frame.width + local[0] as usize;
                let visible = (frame.tags[pixel], frame.views[pixel]);
                // Preserve picks from submissions without model metadata.
                if !frame.has_model_selection(visible.1, visible.0) {
                    if !candidates.contains(&visible) { candidates.push(visible); }
                }
            }
        }
        let mut resolved = Vec::new();
        for candidate in candidates {
            if let Some(candidate) = self.resolve_menu_candidate(client,candidate)? { resolved.push(candidate); }
        }
        let mut candidates = resolved;
        if candidates.is_empty() { candidates.push((0,0)); }
        let views = candidates.iter().filter(|(tag,_)| tag & 0x6000000000000000 != 0)
            .map(|(_,view)|*view).collect::<std::collections::HashSet<_>>().into_iter()
            .map(|view|(view,crate::actor_menu::actors(client,view))).collect();
        let mut candidates = crate::actor_menu::expand(&candidates,&views,client.menu_settings.combat_target);
        crate::ground_menu::deduplicate_piles(&mut candidates);
        let mut low_priority = Vec::new();
        let mut movement = Vec::new();
        let mut entries = Vec::new();
        let mut actors = Vec::new();
        for (index, candidate) in candidates.into_iter().enumerate() {
            let mut view = 0;
            let mut extra_movement = Vec::new();
            let mut actor = false;
            let ground = candidate.0 & 0x1000000000000000 != 0;
            let mut selected = self.unfiltered_menu_entries(client, point, &mut view,
                if index == 0 { &mut movement } else { &mut extra_movement }, candidate, &mut low_priority, &mut actor)?;
            filter(&mut selected, client, view)?;
            let walk_target = selected.iter().find(|e| matches!(e.action.scene().1, crate::context_menu::Action::Walk(_)) && !e.target.is_empty()).map(|e|e.target.clone());
            selected.retain(|e| !matches!(e.action.scene().1, crate::context_menu::Action::Walk(_)));
            let start = entries.len();
            for entry in selected {
                if actor || ground || !entries.iter().any(|e: &crate::context_menu::Entry<crate::context_menu::Action>| e.action == entry.action && e.option == entry.option) {
                    entries.push(entry);
                }
            }
            if actor { actors.push(crate::scene_menu::ActorContribution { entries: start..entries.len(), walk_target }); }
        }
        filter(&mut movement, client, 0)?;
        for actor in &mut actors {
            actor.entries.start += movement.len();
            actor.entries.end += movement.len();
        }
        entries.splice(0..0, movement);
        Ok((entries, low_priority, actors))
    }
    fn resolve_menu_candidate(&mut self, client:&mut ClientState, candidate:(u64,u16))->Result<Option<(u64,u16)>> {
        let (mut tag, mut view) = candidate;
        if view != 0 && tag != 0 && tag >> 60 == 0 && self.location_definitions.is_none() {
            self.location_definitions = Some(crate::object::Definition::load_all(&mut client.scripts.source.cache)?);
        }
        if view != 0 {
            let Some((parent_view, entity)) = client.world_entities.get(&0)
                .and_then(|world| world.entities.get(&view)).map(|e| (0u16, e)) else {
                return Ok(None);
            };
            let Some(def) = &entity.definition else { return Ok(None); };
            let parent = crate::world_pick::parent_tag(def.picking_type,
                client.player_state.as_ref().map_or(0, |s| s.local_view) == view,
                def.interactable, self.entity_overlap_hidden.contains(&view),
                crate::world_pick::tag(entity.plane as i32, entity.pose.x >> 7,
                    entity.pose.z >> 7, 4, false, view as u32, parent_view as i32));
            let selectable = if tag & 0x4000000000000000 != 0 {
                entity.npcs.npcs.get(&(tag as u16)).and_then(|n| client.npcs.get(&(n.id as u32)))
                    .is_some_and(|d| d.interactable)
            } else if tag != 0 && tag >> 60 == 0 {
                self.location_definitions.as_ref().and_then(|defs| defs.get(&(((tag >> 32) - 1) as u32)))
                    .is_some_and(|d| d.interactive != 0)
            } else { tag != 0 && tag & 0x8000000000000000 == 0 };
            let source = crate::world_pick::tag(0, 0, 0, 0, !selectable, 1, view as i32);
            let Some(selected) = crate::world_pick::native_target(parent, source, tag, view, view, parent_view)
                else { return Ok(None); };
            (tag, view) = selected;
        }
        Ok(Some((tag,view)))
    }
    fn unfiltered_menu_entries(
        &mut self,
        client: &mut ClientState,
        point: [i32; 2],
        target_view: &mut u16,
        movement: &mut Vec<crate::context_menu::Entry<crate::context_menu::Action>>,
        candidate: (u64, u16),
        low_priority: &mut Vec<crate::context_menu::Action>,
        actor: &mut bool,
    ) -> Result<Vec<crate::context_menu::Entry<crate::context_menu::Action>>> {
        use crate::context_menu::{Action, Entry};
        let mut entries = Vec::new();
        let Some((frame, origin, ray, _)) = &self.pick else {
            return Ok(entries);
        };
        let x = point[0] - origin[0];
        let y = point[1] - origin[1];
        if x < 0 || y < 0 || x >= frame.width as i32 || y >= frame.height as i32 {
            return Ok(entries);
        }
        if client.selected_spell.is_none() {
            let hits = frame.terrain_hits_where([x, y], |view, tag| {
                let (scene, level) = if view==0 {
                    let Some((scene,_))=&self.world else {return false;};
                    (scene,client.data.position[0])
                } else {
                    let Some(entity)=client.view_entity(view) else {return false;};
                    let Some((_,scene,_,_))=self.entity_worlds.get_instance(view,entity.spawn_generation) else {return false;};
                    (scene,entity.plane as i32)
                };
                scene.tile_render_plane(((tag>>28)&3) as usize,((tag>>14)&16383) as usize,
                    (tag&16383) as usize).is_some_and(|plane|plane as i32<=level)
            });
            let views = client.view_registry.ordered().into_iter().map(|view| crate::viewport_steering::ViewHit {
                view, policy: client.interaction_policy.get(view as i16)[0],
                depth: hits.get(&view).map_or(-1.0, |h| h.1),
            }).collect::<Vec<_>>();
            for hit in crate::viewport_steering::movement_views(&views) {
                let (option, action) = if hit.policy == 2 {
                    let Some(direction) = ray.heading([x, y]) else { continue; };
                    ("Set heading", Action::Heading(direction))
                } else {
                    let Some(&(tag, _)) = hits.get(&hit.view) else { continue; };
                    let scene = if hit.view == 0 { self.world.as_ref().map(|(s, _)| s) }
                        else { self.entity_worlds.get(&hit.view).map(|(_, s, _, _)| s) };
                    let Some(scene) = scene else { continue; };
                    let tile = [((tag >> 14) & 16383) as i32 + scene.base_x, (tag & 16383) as i32 + scene.base_y];
                    if tile.iter().any(|v| !(0..16384).contains(v)) { continue; }
                    ("Walk here", Action::Walk(tile.map(|v| v as u16)))
                };
                movement.push(Entry { option: option.into(), target: String::new(), action: action.in_view(hit.view), submenu: false, children: Vec::new() });
            }
        }
        let pixel = y as usize * frame.width + x as usize;
        let (tag, view) = candidate;
        *target_view = view;
        let Some(plane) = crate::world_pick::menu_plane(client.data.position[0],
            client.player_state.as_ref().map_or(0, |s| s.local_view), view, &client.world_entities)
            else { return Ok(entries); };
        let scene = if view == 0 { self.world.as_ref().map(|(scene, _)| scene) }
            else { client.world_entities.get(&0).and_then(|world| world.entities.get(&view))
                .and_then(|entity| self.entity_worlds.get_instance(view, entity.spawn_generation))
                .map(|(_, scene, _, _)| scene) };
        let Some(scene) = scene else {
            return Ok(entries);
        };
        let ground_tag = frame.ground_tags[pixel];
        let ground_view = frame.ground_views[pixel];
        let ground_scene = if ground_view == 0 { self.world.as_ref().map(|(s, _)| s) }
            else { self.entity_worlds.get(&ground_view).map(|(_, s, _, _)| s) };
        let ground_tile = ground_scene.filter(|_| ground_tag != 0).map(|scene| {
            [
                (((ground_tag >> 14) & 16383) as i32 + scene.base_x) as u16,
                ((ground_tag & 16383) as i32 + scene.base_y) as u16,
            ]
        });
        if tag == 0 {
            return Ok(entries);
        }
        if tag & 0x0800_0000_0000_0000 != 0 {
            if client.selected_spell.is_none() {
                if let Some(tile) = ground_tile {
                    entries.push(Entry {
                        option: "Walk here".into(),
                        target: String::new(),
                        action: Action::Walk(tile).in_view(ground_view),
                        submenu: false,
                        children: Vec::new(),
                    });
                }
                let id = tag as u16;
                if let Some(entity) = client
                    .world_entities
                    .get(&0)
                    .and_then(|world| world.entities.get(&id))
                {
                    if let Some(def) = entity
                        .definition
                        .as_ref()
                        .filter(|def| {
                            let parent = crate::world_pick::parent_tag(
                                def.picking_type,
                                client.player_state.as_ref().map_or(0, |s| s.local_view) == id,
                                def.interactable,
                                self.entity_overlap_hidden.contains(&id),
                                crate::world_pick::tag(
                                    entity.plane as i32, entity.pose.x >> 7,
                                    entity.pose.z >> 7, 4, false, id as u32, 0,
                                ),
                            );
                            // A zero parent preserves child picks; kind 5 is the
                            // original suppression sentinel, never an entity menu.
                            parent != 0 && (parent >> 16) & 7 == 4
                        })
                    {
                        entries.extend(crate::world_entity_menu::entries(id, entity, def));
                    }
                }
            }
            return Ok(entries);
        }
        if tag & 0x4000000000000000 != 0 {
            *actor = true;
            let index = tag as u16;
            let npc = if view == 0 { client.npc_state.npcs.get(&index) }
                else { client.world_entities.get(&0).and_then(|w| w.entities.get(&view)).and_then(|e| e.npcs.npcs.get(&index)) };
            let Some(npc) = npc.cloned() else {
                return Ok(entries);
            };
            let Some(resolved) = crate::actor_menu::npc_definition(client,npc.id as u32)? else {
                return Ok(entries);
            };
            let Some(def) = client.npcs.get(&resolved) else {
                return Ok(entries);
            };
            if !def.interactable
                || (def.follower
                    && !client
                        .menu_settings
                        .follower_varps
                        .iter()
                        .any(|&id| client.scripts.variables.player(id) as u16 == index))
            {
                return Ok(entries);
            }
            let local_level = client
                .player_state
                .as_ref()
                .and_then(|s| s.local_actor())
                .and_then(|p| p.appearance.as_ref())
                .map_or(0, |p| p.combat as i32);
            let target = crate::menu_settings::npc_target(
                if npc.appearance.name.is_empty() {
                    &def.name
                } else {
                    &npc.appearance.name
                },
                def.combat,
                npc.appearance.combat,
                local_level,
            );
            let examine = Entry {
                option: "Examine".into(),
                target: target.clone(),
                action: Action::ExamineNpc(index),
                submenu: false,
                children: Vec::new(),
            };
            let follower_low = def.low_priority && client.preferences.follower_ops_low_priority;
            if follower_low {
                entries.push(examine.clone());
            }
            if let Some(selection) = &client.selected_spell {
                if let Some(entry) =
                    (!client.hidden_operations[2]).then(|| selection.world_entry(1, index, npc.position.map(|v| v as u16), &target)).flatten()
                {
                    entries.push(entry);
                }
                return Ok(entries);
            }

            for attacks in [false, true] {
                for i in (0..5).rev() {
                    if client.hidden_operations[2] || npc.appearance.operations & (1 << i) == 0 {
                        continue;
                    }
                    if let Some(label) = crate::entity_ops::label(
                        &def.operations,
                        i,
                        None,
                        &client.scripts.variables,
                        &mut client.scripts.source,
                    )? {
                        let attack = label.eq_ignore_ascii_case("Attack");
                        if attack != attacks {
                            continue;
                        }
                        let Some(low) = client.menu_settings.npc_priority(
                            attack,
                            follower_low,
                            local_level,
                            def.combat,
                        ) else {
                            continue;
                        };
                        if low { low_priority.push(Action::Npc { index, operation: i + 1, sub: 0 }.in_view(view)); }
                        entries.push(Entry {
                            option: label.to_owned(),
                            target: target.clone(),
                            action: Action::Npc {
                                index,
                                operation: i + 1,
                                sub: 0,
                            },
                            submenu: false,
                            children: crate::entity_ops::children(
                                &def.operations,
                                i,
                                &client.scripts.variables,
                                &mut client.scripts.source,
                            )?
                            .into_iter()
                            .map(|(sub, option)| Entry {
                                option,
                                target: target.clone(),
                                action: Action::Npc {
                                    index,
                                    operation: i + 1,
                                    sub,
                                },
                                submenu: false,
                                children: Vec::new(),
                            })
                            .collect(),
                        });
                    }
                }
            }
            if !follower_low {
                entries.push(examine);
            }
            if let Some(tile) = ground_tile {
                entries.push(Entry {
                    option: "Walk here".into(),
                    target: String::new(),
                    action: Action::Walk(tile).in_view(ground_view),
                    submenu: false,
                    children: Vec::new(),
                });
            }

            return Ok(entries);
        }
        if tag & 0x2000000000000000 != 0 {
            let index = tag as u16;
            if let Some(state) = &client.player_state {
                if index == state.local {
                    if client.selected_spell.is_none() {
                        if let Some(tile) = ground_tile {
                            entries.push(Entry {
                                option: "Walk here".into(),
                                target: String::new(),
                                action: Action::Walk(tile).in_view(ground_view),
                                submenu: false,
                                children: Vec::new(),
                            });
                        }
                    }
                    return Ok(entries);
                }
                if let Some(player) = state.root_owned_actor(view, index, &client.world_entities) {
                    if let Some(appearance) = &player.appearance {
                        *actor = true;
                        let local = state.local_actor()
                            .and_then(|p| p.appearance.as_ref());
                        let local_level = local.map_or(0, |p| p.combat as i32);
                        let local_team = local.map_or(0, |p| p.team);
                        let clan = client.clans.settings.iter().enumerate().any(|(i, c)| {
                            i != 2
                                && c.as_ref().is_some_and(|c| {
                                    c.borrow().members.iter().any(|m| {
                                        m.name.as_ref().is_some_and(|n| {
                                            n.eq_ignore_ascii_case(&appearance.name)
                                        })
                                    })
                                })
                        });
                        let level = if appearance.skill == 0 {
                            format!(
                                "<col={:x}>  (level-{})",
                                crate::menu_settings::combat_color(
                                    appearance.combat as i32,
                                    local_level
                                ),
                                appearance.combat
                            )
                        } else {
                            format!("  (skill-{})", appearance.skill)
                        };
                        let target = format!(
                            "<col=ffffff>{}{}{}{}{}",
                            appearance.actions[0],
                            appearance.name,
                            appearance.actions[1],
                            level,
                            appearance.actions[2]
                        );
                        if let Some(selection) = &client.selected_spell {
                            if let Some(entry) = selection.world_entry(
                                2,
                                index,
                                [player.position[1] as u16, player.position[2] as u16],
                                &target,
                            ) {
                                entries.push(entry);
                            }
                            return Ok(entries);
                        }
            
                        for i in (0..8).rev() {
                            if let Some(option) = client.player_operations[i]
                                .as_ref()
                                .filter(|v| !v.is_empty())
                            {
                                let Some(low) = client.menu_settings.player_priority(
                                    option.eq_ignore_ascii_case("Attack"),
                                    client.player_operation_priority[i],
                                    local_level,
                                    appearance.combat as i32,
                                    [local_team, appearance.team],
                                    clan,
                                ) else {
                                    continue;
                                };
                                if low { low_priority.push(Action::Player { index, operation: (i + 1) as u8 }.in_view(view)); }
                                entries.push(Entry {
                                    option: option.clone(),
                                    target: target.clone(),
                                    action: Action::Player {
                                        index,
                                        operation: (i + 1) as u8,
                                    },
                                    submenu: false,
                                    children: Vec::new(),
                                });
                            }
                        }
                        if let Some(tile) = ground_tile {
                            entries.push(Entry {
                                option: "Walk here".into(),
                                target: target.clone(),
                                action: Action::Walk(tile).in_view(ground_view),
                                submenu: false,
                                children: Vec::new(),
                            });
                        }
            
                    }
                }
            }
            return Ok(entries);
        }
        let tile = [
            ((tag >> 14) & 16383) as i32 + scene.base_x,
            (tag & 16383) as i32 + scene.base_y,
        ];
        if tile.iter().any(|v| !(0..16384).contains(v)) {
            return Ok(entries);
        }
        let tile = tile.map(|v| v as u16);
        if client.selected_spell.is_none() {
            if let Some(ground_tile) = ground_tile {
                entries.push(Entry {
                    option: "Walk here".into(),
                    target: String::new(),
                    action: Action::Walk(ground_tile).in_view(ground_view),
                    submenu: false,
                    children: Vec::new(),
                });
            }
        }
        if tag & 0x8000000000000000 != 0 {
            return Ok(entries);
        }
        if tag & 0x1000000000000000 != 0 {
            let zone = if view == 0 { Some(&client.zone) }
                else { client.world_entities.get(&0)
                    .and_then(|world| world.entities.get(&view)).map(|e| &e.zone) };
            let pile = zone.and_then(|z| z.piles.get(&[plane,tile[0] as i32,tile[1] as i32])).cloned().unwrap_or_default();
            entries.extend(crate::ground_menu::entries(&pile,tile,&client.items,client.selected_spell.as_ref(),client.hidden_operations[1],&client.scripts.variables,&mut client.scripts.source)?);
        } else {
            let id = ((tag >> 32) - 1) as u16;
            if !crate::object_menu::present(scene,plane,tile,id) { return Ok(entries); }
            #[cfg(feature = "custom-assets")]
            if view==0 && self.world.as_ref().is_some_and(|(scene,_)|scene.visual_only_id(u32::from(id))) {return Ok(entries);}
            if self.location_definitions.is_none() {
                self.location_definitions = Some(crate::object::Definition::load_all(
                    &mut client.scripts.source.cache,
                )?);
            }
            let Some(mut def) = self
                .location_definitions
                .as_ref()
                .unwrap()
                .get(&(id as u32))
            else {
                return Ok(entries);
            };
            if def.transforms.is_some() {
                use crate::script_runtime::Source;
                let bit = if let Some(id) = def.transform_varbit {
                    let bit = client.scripts.source.varbit(id as i32)?;
                    bit.get(client.scripts.variables.player(bit.base))
                } else {
                    0
                };
                let varp = def
                    .transform_varp
                    .map_or(0, |id| client.scripts.variables.player(id));
                let Some(resolved) = def.transformed_id(bit, varp) else {
                    return Ok(entries);
                };
                let Some(resolved) = self.location_definitions.as_ref().unwrap().get(&resolved)
                else {
                    return Ok(entries);
                };
                def = resolved;
            }
            let state = crate::object_menu::queue(&self.locations,&client.world_entities,view).and_then(|queue|queue.menu_state(plane,tile,id));
            entries.extend(crate::object_menu::entries(id,tile,def,state,client.selected_spell.as_ref(),client.hidden_operations[0],&client.scripts.variables,&mut client.scripts.source)?);
        }
        Ok(entries)
    }
    pub fn click(&mut self, client: &mut ClientState, point: [i32; 2]) -> Result<bool> {
        let layout = client.interfaces.layout(client.canvas)?;
        if matches!(client.minimap_state, 0 | 3) {
            if let Some(id) = layout
                .hit_test(point)
                .find(|id| client.interfaces.widgets[id].content == 1338)
            {
                if let (Some((scene, _)), Some(view)) = (&self.world, self.minimap_view(client)) {
                    let widget = &client.interfaces.widgets[&id];
                    if let Some(mask) =
                        crate::minimap::widget_mask(&mut client.scripts.source.cache, widget, true)?
                    {
                        let origin = layout.widgets[&id].absolute;
                        // The injected minimap handler resolves the legacy -1 view policy.
                        let mode=client.interaction_policy.get(-1)[0];
                        if mode!=1 {
                            if mode==2 {
                                if let Some(direction)=crate::steering::minimap_click(&mask,
                                    [point[0]-origin[0],point[1]-origin[1]],
                                    [client.input.pointer[0]-origin[0],client.input.pointer[1]-origin[1]],view.angle) {
                                    client.outbound.push_back(client.steering.request(direction)?);
                                    return Ok(true);
                                }
                            }
                            return Ok(false);
                        }
                        let fine = [
                            client.local_fine()[0] - scene.base_x * 128,
                            client.local_fine()[1] - scene.base_y * 128,
                        ];
                        if let Some((target, centered)) = crate::minimap::click_target(
                            &mask,
                            [point[0] - origin[0], point[1] - origin[1]],
                            fine,
                            view.angle,
                            4.0,
                        )? {
                            let keys = if client.input.held.contains(&82) {
                                if client.input.held.contains(&81) {
                                    2
                                } else {
                                    1
                                }
                            } else {
                                0
                            };
                            client.outbound.push_back(crate::world_action::minimap_walk(
                                [target[0] + scene.base_x, target[1] + scene.base_y],
                                keys,
                                centered,
                                view.angle,
                                fine,
                            )?);
                            client.map_flag = Some(
                                (((target[0] + scene.base_x) as u32 & 16383) << 14)
                                    | ((target[1] + scene.base_y) as u32 & 16383),
                            );
                            return Ok(true);
                        }
                    }
                }
            }
        }
        let entries = self.menu_entries(client, point)?;
        if let Some(entry) = entries.last() {
            client.menu_action_at(entry.action,point)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    /// Find a visible pick pixel that survives the actual window/canvas coordinate round trip.
    pub fn pick_visible(
        &self,
        fit: crate::canvas_transform::Fit,
        mut test: impl FnMut(u64) -> bool,
    ) -> Option<([i32; 2], u64)> {
        let (frame, origin, _, _) = self.pick.as_ref()?;
        let mut best = None;
        let mut best_score = 0;
        for y in 0..frame.height {
            for x in 0..frame.width {
                let tag = frame.tags[y * frame.width + x];
                if tag == 0 || !test(tag) {
                    continue;
                }
                let point = [origin[0] + x as i32, origin[1] + y as i32];
                let mapped = fit.to_logical(fit.to_physical(point));
                let mx = mapped[0] - origin[0];
                let my = mapped[1] - origin[1];
                if mx < 0
                    || my < 0
                    || mx >= frame.width as i32
                    || my >= frame.height as i32
                    || frame.tags[my as usize * frame.width + mx as usize] != tag
                {
                    continue;
                }
                let mut score = 0;
                for dy in -2..=2 {
                    for dx in -2..=2 {
                        let xx = mx + dx;
                        let yy = my + dy;
                        if xx >= 0
                            && yy >= 0
                            && xx < frame.width as i32
                            && yy < frame.height as i32
                            && frame.tags[yy as usize * frame.width + xx as usize] == tag
                        {
                            score += 1;
                        }
                    }
                }
                if score > best_score {
                    best_score = score;
                    best = Some((point, tag));
                }
            }
        }
        best
    }
    pub fn pick_where(&self, mut test: impl FnMut(u64) -> bool) -> Option<([i32; 2], u64)> {
        self.pick_where_view(|_, tag| test(tag))
    }
    pub fn pick_where_view(&self, test: impl FnMut(u16, u64) -> bool) -> Option<([i32; 2], u64)> {
        let (frame, origin, _, _) = self.pick.as_ref()?;
        // Probe the deepest actual same-tag/view region, rather than the first
        // one-pixel inset which can leave the tile as the camera follows movement.
        crate::pick_probe::interior([frame.width,frame.height],&frame.tags,&frame.views,test)
            .map(|([x,y],tag)|([origin[0]+x as i32,origin[1]+y as i32],tag))
    }
    /// Observation must wait for received work queued behind a resumed listener.
    pub fn observation_ready(&self,client:&ClientState)->bool {
        !self.scripts_busy() && self.deferred_events.is_empty()
            && client.interfaces.scripts.is_empty() && self.world_job.is_none() && self.world.is_some()
    }
    #[cfg(feature="private-test-adapter")]
    pub fn private_action_ready(&self,client:&ClientState)->bool {self.observation_ready(client)}
    /// Last world draw's pick frame and its canvas origin, read-only.
    #[cfg(feature="private-test-adapter")]
    pub fn private_pick_frame(&self)->Option<(&Frame,[i32;2])> {self.pick.as_ref().map(|(frame,origin,_,_)|(frame,*origin))}
    pub fn scripts_busy(&self) -> bool {
        self.pending_script.is_some()
            || self.is_reconnecting()
            || self.logged_out.is_some()
            || self.title.is_some()
    }
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
}

/// #181: the connection's journal slot. `Recording` starts at adapter attach (before the
/// first main-loop poll); `Exported` is terminal and refuses every later poll.
#[cfg(feature="private-test-adapter")]
pub(crate) enum JournalSlot {
    Off,
    Recording(crate::client_journal::Recorder),
    Exported,
}
#[cfg(feature="private-test-adapter")]
impl Connection {
    /// Attach a fresh recorder once. A connection that has ever recorded refuses.
    pub(crate) fn attach_journal(&mut self, recorder: crate::client_journal::Recorder) -> Result<()> {
        if !matches!(self.journal, JournalSlot::Off) {
            return Err(Error::Invalid("client journal already attached"));
        }
        self.journal = JournalSlot::Recording(recorder);
        Ok(())
    }
    /// The one-shot export boundary: take the recorder and mark the slot exported.
    pub(crate) fn take_journal(&mut self) -> Result<crate::client_journal::Recorder> {
        match std::mem::replace(&mut self.journal, JournalSlot::Exported) {
            JournalSlot::Recording(recorder) => Ok(recorder),
            other => {
                self.journal = other;
                Err(Error::Invalid("client journal not recording"))
            }
        }
    }
}

#[cfg(test)]
#[path = "live_runtime_drain_tests.rs"]
mod drain_tests;

/// #88 test-only construction (see live_runtime_packet_tests.rs). Differs from
/// connect_credentials only where noted; poll_elapsed, packet decode/apply,
/// the deferred-packet gate and the generation drain are the production paths.
#[cfg(test)]
impl Connection {
    /// `session` is a real loopback Session whose login request the test
    /// already submitted with `Session::submit_with_seed(.., login_seed)`.
    /// Omitted or replaced relative to connect_credentials, all off the
    /// packet/apply/drain path:
    /// - the ServerSeed branch that builds and submits the login Request
    ///   (the test submitted it, so ISAAC seeds are known); `crcs`, `key` and
    ///   `password` are therefore unused and set to inert values;
    /// - the connect worker thread (`reconnect_job`) and token/EULA flows
    ///   (no token login; policy treated as accepted);
    /// - `client.uid192` storage configuration (no client-data files written);
    /// - overlay and minimap render assets: built from public parts (one
    ///   synthetic font, empty sprite sets) because they are only read when
    ///   painting. Textures, actors and the palette use the production loaders.
    ///
    /// Every other field mirrors connect_credentials' initial value (the
    /// literal is copied from it), including `started: Instant::now()`,
    /// `graphics_cycle: client.actor_cycle`, the title state (Connecting
    /// screen, credentials, remember flags), keyboard/mouse/input report
    /// defaults, `world: None` and `packets: 0`. Intentional differences:
    /// `reconnect_job`, `token_job` and `token_config` are None; `logged_out`
    /// is None (EULA treated as accepted); `login_seed` is the submitted seeds;
    /// `crcs` are zero, `key` is a placeholder and `password` is "test" (all
    /// only read by the skipped request-building branch); overlay/minimap
    /// assets are synthetic. If connect_credentials changes an initial value,
    /// update this list and the literal together.
    pub(crate) fn for_packet_tests(
        client: &mut ClientState,
        session: Session,
        address: std::net::SocketAddr,
        login_seed: [u32; 4],
    ) -> Result<Self> {
        let key = RsaPublicKey::from_hex(&"f".repeat(256), 65537)?;
        let password = String::from("test");
        let username = "fixture";
        let crcs = [0u32; 23];
        let brightness=client.preferences.device.brightness;
        let palette=crate::floor::palette(brightness)?;
        let mut textures = HashMap::new();
        for (id, d) in Definition::load_all(&mut client.scripts.source.cache)? {
            textures.insert(
                id,
                Texture::load(&mut client.scripts.source.cache, d, 128, brightness)?,
            );
        }
        let mut actors = crate::world_actors::Actors::load(&mut client.scripts.source.cache)?;
        actors.set_palette(&palette);
        let font = crate::font::Font {
            metrics: crate::font::Metrics { advances: [6; 256], ascent: 12, kerning: None },
            glyphs: crate::sprite::SpriteSheet {
                width: 1, height: 1, palette: vec![0, 0xffffff],
                sprites: (0..256).map(|_| crate::sprite::Sprite { x: 0, y: 0, width: 1, height: 1, indices: vec![1] }).collect(),
            },
            max_ascent: 12,
            max_descent: 0,
        };
        let overlays = crate::overlay_render::Assets::from_assets(0, HashMap::from([(0, font)]), HashMap::new())?;
        let pixel = || crate::sprite_pixels::Pixels { canvas: [1, 1], offset: [0, 0], size: [1, 1], pixels: vec![0] };
        let minimap_assets = std::sync::Arc::new(crate::minimap::Assets {
            compass: pixel(), dots: Vec::new(), scenes: Vec::new(), flags: Vec::new(), edge: pixel(),
        });
        let token_login = false;
        Ok(Self {
            entity_sprites: HashMap::new(),
            entity_npc_definitions: HashMap::new(),
            entity_worlds: Default::default(),
            entity_overlap_hidden: HashSet::new(),
            terrain_visibility: crate::terrain_submission::Cache::default(),
            entity_submitted_cycle: HashMap::new(),
            pending_script: None,
            deferred_events: Default::default(),
            world_job: None,
            rebuild_walk: None,
            next_rebuild: None,
            session,
            address,
            endpoint_host: address.ip().to_string(),
            hopping: false,
            login_seed: Some(login_seed),
            login_retries: 0,
            logged_out: None,
            title: Some(crate::title_input::State {
                token_ready: token_login,
                required_eula: client.preferences.required_eula,
                accepted_eula: client.preferences.saved.accepted_eula,
                screen: crate::title_input::Screen::Connecting,
                username: username.into(),
                password: password.clone(),
                hide: client.preferences.device.hide_username,
                remember: client.preferences.saved.remembered_username.is_some(),
                lines: ["".into(), "Connecting to server...".into(), "".into()],
                ..Default::default()
            }),
            title_assets: None,
            world_selector: None,
            world_selector_assets: None,
            world_selector_loading: false,
            mouse_recorder: None,
            mouse_pending_click: None,
            mouse_click_state: Default::default(),
            mouse_click_packets: 0,
            keyboard_input: Default::default(),
            keyboard_report: Default::default(),
            keyboard_packets: 0,
            input_reports: Default::default(),
            pending_wheel: 0,
            window_focused: true,
            auxiliary_packets: 0,
            title_music_started: false,
            transfer_delay: None,
            remember_username: client.preferences.saved.remembered_username.is_some(),
            authenticator: None,
            dob_in_client: false,
            date_of_birth: None,
            resume_seed: None,
            reconnect_job: None,
            token_login,
            token_config: None,
            token_secret: None,
            token_job: None,
            username: username.into(),
            password,
            key,
            crcs,
            local: None,
            started: Instant::now(),
            #[cfg(feature="private-test-adapter")]
            journal: JournalSlot::Off,
            #[cfg(feature="native-profiling")]
            scene_profile:None,
            #[cfg(feature="native-profiling")]
            readiness_trace:Default::default(),
            #[cfg(feature="native-profiling")]
            scene_filter:Default::default(),
            #[cfg(feature="native-profiling")]
            scene_filter_report:Default::default(),
            #[cfg(all(feature="native-profiling",feature="enhanced-renderer"))]
            enhanced_trace:None,
            #[cfg(feature="enhanced-renderer")]
            enhanced:None,
            world: None,
            minimap: None,
            #[cfg(feature = "custom-assets")]
            minimap_geography: None,
            minimap_icons: Vec::new(),
            minimap_icons_pending: false,
            minimap_assets,
            textures,
            palette,
            brightness,
            pick: None,
            #[cfg(feature="private-test-adapter")]
            draw_custody: crate::draw_custody::Custody::new(),
            actors,
            overlays,
            locations: Default::default(),
            world_cycle: 0,
            graphics_cycle: client.actor_cycle,
            location_definitions: None,
            reported: HashSet::new(),
            packets: 0,
        })
    }
}

#[cfg(test)]
#[path = "live_runtime_packet_tests.rs"]
mod packet_tests;

/// Profiling-only readiness timeline: when SERVER_TICK and UPDATE_STAT packets
/// are received versus applied, and what the live loop waits on meanwhile.
/// "Received" is the time session.poll returns the decoded packet event to this
/// loop, not wire arrival: it proves the packet had arrived by then, but cannot
/// by itself separate an earlier socket backlog from a late server send.
/// Observation only; dispatch, ordering and VM limits are unchanged.
#[cfg(feature="native-profiling")]
mod readiness_trace {
    use std::time::Duration;
    pub const SERVER_TICK: u16 = 83;
    pub const UPDATE_STAT: u16 = 46;
    const MILESTONES: [&str; 7] = [
        "login accepted", "first packet received", "first SERVER_TICK received", "first SERVER_TICK applied",
        "first UPDATE_STAT received", "first UPDATE_STAT applied", "first observation_ready",
    ];
    pub const LOGIN: usize = 0;
    pub const READY: usize = 6;
    /// Summaries stop this long after the first observation_ready.
    const TAIL: Duration = Duration::from_secs(10);
    #[derive(Default, Debug, PartialEq)]
    struct Window {
        cycles: u32, deferred: usize, max_deferred: usize, scripts: usize, max_scripts: usize,
        script_pending_cycles: u32, slices: u32, steps: u32, slice_time: Duration,
        pending_asset_breaks: u32, received: u32, applied: u32,
        deferring_drains: u32, max_deferred_entries: usize,
    }
    #[derive(Default)]
    pub struct Trace { first: [Option<Duration>; 7], window: Window, window_start: Duration }
    impl Trace {
        /// Returns the line to log the first time a milestone is reached.
        pub fn milestone(&mut self, index: usize, now: Duration) -> Option<String> {
            if self.first[index].is_some() { return None; }
            self.first[index] = Some(now);
            Some(format!("Native readiness trace: {} at {:.3}s", MILESTONES[index], now.as_secs_f64()))
        }
        pub fn received(&mut self, opcode: u16, now: Duration) -> Vec<String> {
            self.window.received += 1;
            let mut lines: Vec<_> = self.milestone(1, now).into_iter().collect();
            if opcode == SERVER_TICK { lines.extend(self.milestone(2, now)); }
            if opcode == UPDATE_STAT { lines.extend(self.milestone(4, now)); }
            lines
        }
        pub fn applied(&mut self, opcode: u16, now: Duration) -> Option<String> {
            self.window.applied += 1;
            match opcode { SERVER_TICK => self.milestone(3, now), UPDATE_STAT => self.milestone(5, now), _ => None }
        }
        pub fn pending_asset(&mut self) { self.window.pending_asset_breaks += 1; }
        /// `deferred`: entries left for the next poll by the generation boundary.
        pub fn slice(&mut self, steps: u32, time: Duration, deferred: usize) {
            self.window.slices += 1;
            self.window.steps += steps;
            self.window.slice_time += time;
            if deferred != 0 {
                self.window.deferring_drains += 1;
                self.window.max_deferred_entries = self.window.max_deferred_entries.max(deferred);
            }
        }
        /// Called once per poll with the state it starts from. Returns a summary
        /// line about once per second until TAIL after readiness, then nothing.
        pub fn cycle(&mut self, deferred: usize, scripts: usize, script_pending: bool, now: Duration) -> Option<String> {
            if self.first[LOGIN].is_none() || self.first[READY].is_some_and(|ready| now > ready + TAIL) { return None; }
            let w = &mut self.window;
            w.cycles += 1;
            w.deferred = deferred; w.max_deferred = w.max_deferred.max(deferred);
            w.scripts = scripts; w.max_scripts = w.max_scripts.max(scripts);
            w.script_pending_cycles += u32::from(script_pending);
            if now < self.window_start + Duration::from_secs(1) { return None; }
            let w = std::mem::take(&mut self.window);
            let line = format!(
                "Native readiness window {:.3}s: cycles {}; deferred {} (max {}); queued scripts {} (max {}); script-pending cycles {}; slices {} steps {} in {:.3}s; PendingAsset breaks {}; packets received {} applied {}; generation-deferring drains {} (max entries {})",
                now.as_secs_f64(), w.cycles, w.deferred, w.max_deferred, w.scripts, w.max_scripts,
                w.script_pending_cycles, w.slices, w.steps, w.slice_time.as_secs_f64(), w.pending_asset_breaks, w.received, w.applied,
                w.deferring_drains, w.max_deferred_entries,
            );
            self.window_start = now;
            Some(line)
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        fn s(ms: u64) -> Duration { Duration::from_millis(ms) }
        #[test]
        fn received_and_applied_milestones_are_recorded_once_and_separately() {
            let mut t = Trace::default();
            assert!(t.milestone(LOGIN, s(0)).is_some());
            let lines = t.received(SERVER_TICK, s(100));
            assert_eq!(lines.len(), 2, "first packet and first tick received");
            assert!(lines[1].contains("first SERVER_TICK received at 0.100s"));
            assert!(t.received(SERVER_TICK, s(150)).is_empty());
            assert_eq!(t.received(UPDATE_STAT, s(200)).len(), 1);
            assert!(t.applied(UPDATE_STAT, s(4000)).unwrap().contains("first UPDATE_STAT applied at 4.000s"));
            assert!(t.applied(SERVER_TICK, s(5000)).unwrap().contains("first SERVER_TICK applied at 5.000s"));
            assert!(t.applied(SERVER_TICK, s(5100)).is_none());
            assert!(t.applied(7, s(5200)).is_none());
        }
        #[test]
        fn windows_summarise_waiting_state_and_stop_after_readiness_tail() {
            let mut t = Trace::default();
            assert!(t.cycle(5, 1, true, s(0)).is_none(), "nothing before login");
            t.milestone(LOGIN, s(0));
            assert!(t.cycle(40, 3, true, s(100)).is_none());
            t.slice(50, s(5), 3); t.pending_asset();
            let line = t.cycle(30, 2, false, s(1000)).unwrap();
            assert!(line.contains("cycles 2; deferred 30 (max 40); queued scripts 2 (max 3); script-pending cycles 1; slices 1 steps 50"), "{line}");
            assert!(line.contains("PendingAsset breaks 1"));
            assert!(line.contains("generation-deferring drains 1 (max entries 3)"), "{line}");
            assert_eq!(t.window, Window::default(), "window resets after a summary");
            t.milestone(READY, s(2000));
            assert!(t.cycle(0, 0, false, s(12000)).is_some(), "still within the tail");
            assert!(t.cycle(0, 0, false, s(12001)).is_none());
            assert!(t.cycle(0, 0, false, s(20000)).is_none());
        }
    }
}
