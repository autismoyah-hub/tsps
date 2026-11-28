//! Opt-in dedicated-account native input acceptance checks. Runs only with --smoke-actions.
use crate::{client_state::ClientState, live_runtime::Connection, Error, Result};
#[derive(Default)]
pub struct Actions {
    pub right_click: bool,
    pub map_capture: bool,
    map_enabled: bool,
    map_shortcuts: bool,
    map_phase: u8,
    map_start: [i32;2],
    map_point: [i32;2],
    pub drag_input: u8,
    drag_phase: u8,
    drag_seed_requested: bool,
    drag_target: Option<([i32; 2], usize)>,
    inventory_done: bool,
    spell_phase: u8,
    drop_menu: Option<i32>,
    camera_start: Option<[i32; 2]>,
    camera_stopped: bool,
    camera_ready: bool,
    setup_requested: bool,
    setup_ready: bool,
    target: Option<[i32; 2]>,
    moved: bool,
    minimap_target: Option<[i32; 2]>,
    minimap_moved: bool,
    clicked_npc: bool,
    tanner_scanning: bool,
    message_start: usize,
    pub complete: bool,
    inventory: bool,
    inventory_item: i32,
    talked: bool,
    inventory_clicked: bool,
    inventory_equipped: bool,
    ground_item: Option<i32>,
    ground_before: i32,
    ground_clicked: bool,
    ground_step: Option<[i32; 2]>,
    drop_target: Option<[i32; 2]>,
    drop_observed: Option<([i32; 3], std::time::Instant)>,
    ground_diagnostic: String,

    inventory_before_click: i32,
    observed_inventory: Vec<crate::inventory::Item>,
    observed_message_count: usize,
    reported_item: bool,
    inventory_tab_clicked: bool,
}
impl Actions {
    pub fn next_keys(&mut self, client: &ClientState, live: &Connection) -> Vec<(u32, bool)> {
        if self.map_shortcuts && self.map_phase==4 && client.world_map.pointer.drag_center==[-1,-1] {
            self.map_phase=6;
            println!("Posting Win32 Escape for the map's script-configured shortcut");
            return vec![(0x1b,true)];
        }
        if self.map_phase==6 {
            if client.interfaces.layout(client.canvas).is_ok_and(|layout| !layout.order.iter().any(|id|!layout.widgets[id].hidden && client.interfaces.widgets[id].content==1400)) {
                self.map_phase=7;
                println!("Win32 map Escape shortcut closed the interface; releasing key");
                return vec![(0x1b,false)];
            }
            return Vec::new();
        }
        if self.minimap_moved && !self.clicked_npc {
            let visible = live.pick_where(|tag| {
                tag & 0x4000000000000000 != 0
                    && client.npc_state.npcs.get(&(tag as u16))
                        .is_some_and(|npc| npc.id == 16299)
            }).is_some();
            if visible && self.tanner_scanning {
                self.tanner_scanning = false;
                println!("Native acceptance tanner visible; releasing camera scan key");
                return vec![(0x27, false)];
            }
            if !visible && !self.tanner_scanning {
                self.tanner_scanning = true;
                let target = client.npc_state.npcs.values().find(|npc| npc.id == 16299).map(|npc| npc.position);
                println!("Native acceptance scanning camera for tanner at {target:?}, player {:?}", client.data.position);
                return vec![(0x27, true)];
            }
            return Vec::new();
        }
        if live.world.is_none() || self.camera_ready {
            return Vec::new();
        }
        let angles = [client.camera.pitch, client.camera.yaw];
        let Some(start) = self.camera_start else {
            self.camera_start = Some(angles);
            println!("Posting Win32 camera arrow keys at {angles:?}");
            return vec![(0x26, true), (0x27, true)];
        };
        if !self.camera_stopped
            && angles[0] - start[0] >= 512
            && (angles[1] - start[1]) & 16383 >= 1024
        {
            self.camera_stopped = true;
            println!("Win32 camera rotation confirmed at {angles:?}; releasing keys");
            return vec![(0x26, false), (0x27, false)];
        }
        if self.camera_stopped && client.camera.velocity == [0, 0] {
            self.camera_ready = true;
            println!("Win32 camera inertia settled at {angles:?}");
        }
        Vec::new()
    }
    pub fn new(inventory: bool) -> Self {
        Self {
            inventory,
            map_enabled: std::env::var_os("OSRS_SMOKE_MAP").is_some(),
            map_shortcuts: std::env::var_os("OSRS_SMOKE_MAP_KEYS").is_some(),
            inventory_item: 1277,
            ..Default::default()
        }
    }

    fn map_click(&mut self, client:&mut ClientState) -> Result<Option<[i32;2]>> {
        let layout=client.interfaces.layout(client.canvas)?;
        let visible=layout.order.iter().find_map(|id| {
            let p=&layout.widgets[id];
            (!p.hidden&&client.interfaces.widgets[id].content==1400).then_some(p)
        });
        match self.map_phase {
            0 => {
                for id in &layout.order {
                    let p=&layout.widgets[id];let w=&client.interfaces.widgets[id];
                    if !p.hidden&&w.actions.iter().any(|a|a.to_ascii_lowercase().contains("world map")) {
                        let point=[p.absolute[0]+p.geometry.size[0]/2,p.absolute[1]+p.geometry.size[1]/2];
                        if layout.hit_test(point).any(|hit|hit==*id) {
                            println!("Posting Win32 World Map click {point:?}, widget {id}");self.map_phase=1;return Ok(Some(point));
                        }
                    }
                }
            }
            1 => if let Some(p)=visible {
                if client.world_map.renderer.as_ref().is_some_and(|r|client.world_map.current.is_some_and(|id|r.ready(id))) {
                    self.map_start=client.world_map.center;self.map_point=[p.absolute[0]+p.geometry.size[0]/2,p.absolute[1]+p.geometry.size[1]/2];
                    self.map_capture=true;self.map_phase=2;self.drag_input=1;
                    println!("Native map rendered; pressing drag at {:?}, center {:?}",self.map_point,self.map_start);return Ok(Some(self.map_point));
                }
            },
            2 => if client.world_map.pointer.drag_center!=[-1,-1] {
                self.map_phase=3;self.drag_input=2;return Ok(Some([self.map_point[0]+32,self.map_point[1]+16]));
            },
            3 => if client.world_map.center!=self.map_start {
                println!("Win32 map drag confirmed: {:?} -> {:?}",self.map_start,client.world_map.center);
                self.map_phase=4;self.drag_input=3;return Ok(Some([self.map_point[0]+32,self.map_point[1]+16]));
            },
            4 => if !self.map_shortcuts && client.world_map.pointer.drag_center==[-1,-1] {
                for id in layout.order.iter().rev() {
                    let p=&layout.widgets[id];if p.hidden {continue;}
                    let point=[p.absolute[0]+p.geometry.size[0]/2,p.absolute[1]+p.geometry.size[1]/2];
                    if client.interface_menu(point)?.iter().any(|e|e.option.eq_ignore_ascii_case("Close")) {
                        println!("Posting Win32 map Close click {point:?}");self.map_phase=5;return Ok(Some(point));
                    }
                }
            },
            5 | 7 => if visible.is_none() {println!("Win32 map close confirmed");self.complete=true;},
            _=>{}
        }
        Ok(None)
    }
    fn spell_click(
        &mut self,
        client: &mut ClientState,
        live: &Connection,
        fit: crate::canvas_transform::Fit,
    ) -> Result<Option<[i32; 2]>> {
        use crate::context_menu::Action;
        let layout = client.interfaces.layout(client.canvas)?;
        if self.spell_phase >= 6 {return self.map_click(client);}
        if self.spell_phase == 0 {
            for &id in layout.order.iter().rev() {
                let placed = &layout.widgets[&id];
                let widget = &client.interfaces.widgets[&id];
                if !placed.hidden && widget.actions.iter().any(|a| a == "Magic") {
                    let point = [
                        placed.absolute[0] + placed.geometry.size[0] / 2,
                        placed.absolute[1] + placed.geometry.size[1] / 2,
                    ];
                    if layout.hit_test(point).any(|hit| hit == id) {
                        self.spell_phase = 1;
                        println!("Posting Win32 Magic tab click {point:?}");
                        return Ok(Some(point));
                    }
                }
            }
        } else if self.spell_phase == 1 {
            for &id in layout.order.iter().rev() {
                let placed = &layout.widgets[&id];
                if placed.hidden {
                    continue;
                }
                let widget = &client.interfaces.widgets[&id];
                if widget.spell_action.trim().is_empty()
                    || client.interfaces.interaction_flags(id) >> 11 & 63 == 0
                {
                    continue;
                }
                let point = [
                    placed.absolute[0] + placed.geometry.size[0] / 2,
                    placed.absolute[1] + placed.geometry.size[1] / 2,
                ];
                if client
                    .interface_menu(point)?
                    .iter()
                    .any(|e| matches!(e.action, Action::SelectSpell(_)))
                {
                    self.right_click = true;
                    self.spell_phase = 2;
                    println!("Posting Win32 spell right-click {point:?}, widget {id}");
                    return Ok(Some(point));
                }
            }
        } else if matches!(self.spell_phase, 2 | 4) {
            if let Some(menu) = &client.menu {
                if let Some(index) = menu.entries.iter().position(|e| {
                    if self.spell_phase == 2 {
                        matches!(e.action, Action::SelectSpell(_))
                    } else {
                        matches!(e.action, Action::Cancel)
                    }
                }) {
                    let point = [menu.bounds[0] + menu.bounds[2] / 2, menu.row_y(index) - 5];
                    println!(
                        "Posting Win32 spell menu action {:?} at {point:?}",
                        menu.entries[index].action
                    );
                    self.spell_phase += 1;
                    return Ok(Some(point));
                }
            }
        } else if self.spell_phase == 3 {
            if let Some(selection) = &client.selected_spell {
                if let Some((point, _)) = live.pick_visible(fit, |tag| tag >> 63 == 1) {
                    println!(
                        "Win32 spell selection confirmed: {} {} mask {}",
                        selection.option, selection.target, selection.mask
                    );
                    self.right_click = true;
                    self.spell_phase = 4;
                    return Ok(Some(point));
                }
            }
        } else if self.spell_phase == 5 && client.selected_spell.is_none() && client.menu.is_none()
        {
            println!("Win32 spell Cancel confirmed; selection cleared");
            if self.map_enabled {self.spell_phase=6;} else {self.complete=true;}
        }
        Ok(None)
    }
    pub fn next_click(
        &mut self,
        client: &mut ClientState,
        live: &Connection,
        fit: crate::canvas_transform::Fit,
    ) -> Result<Option<[i32; 2]>> {
        if !self.camera_ready {
            return Ok(None);
        }
        if self.inventory {
            let items = client
                .inventories
                .containers
                .get(&93)
                .cloned()
                .unwrap_or_default();
            if items != self.observed_inventory {
                println!("Inventory acceptance update: {items:?}");
                self.observed_inventory = items;
                self.reported_item = false;
            }
            for message in client
                .game_messages
                .iter()
                .skip(self.observed_message_count)
            {
                println!("Inventory acceptance message: {}", message.text);
            }
            self.observed_message_count = client.game_messages.len();
        }
        if self.complete {
            return Ok(None);
        }
        if live.elapsed().as_secs() > 180 {
            return Err(Error::Io("native mouse action acceptance timed out".into()));
        }
        if live.scripts_busy() || !client.interfaces.scripts.is_empty() {
            return Ok(None);
        }
        if self.inventory_done {
            return self.spell_click(client, live, fit);
        }
        let Some((scene, _)) = &live.world else {
            return Ok(None);
        };
        let position = client.data.position;
        if self.inventory && !self.setup_ready {
            if position == [0, 10428, 122] {
                self.setup_ready = true;
                println!("Acceptance setup ready: dedicated hub position confirmed");
            } else {
                if !self.setup_requested {
                    self.setup_requested = true;
                    println!("Acceptance setup required: dedicated hub position");
                }
                return Ok(None);
            }
        }
        if let Some(target) = self.target {
            if !self.moved && [position[1], position[2]] == target {
                println!("Win32 mouse walk confirmed by server at {target:?}");
                self.moved = true;
            }
        } else if let Some((point, tag)) = live.pick_where(|tag| {
            if tag >> 63 != 1 {
                return false;
            }
            let x = ((tag >> 14) & 16383) as i32;
            let z = (tag & 16383) as i32;
            // Keep the dedicated hub probe south of its northern boundary.
            if z + scene.base_y >= 127 {
                return false;
            }
            let p = [position[1] - scene.base_x, position[2] - scene.base_y];
            let delta = [x - p[0], z - p[1]];
            crate::route::can_step(&scene.collision[position[0] as usize], p, 1, delta)
        }) {
            let target = [
                ((tag >> 14) & 16383) as i32 + scene.base_x,
                (tag & 16383) as i32 + scene.base_y,
            ];
            self.target = Some(target);
            println!("Posting Win32 click {point:?} for walk target {target:?}");
            return Ok(Some(point));
        }
        if self.moved && !self.minimap_moved {
            if let Some(target) = self.minimap_target {
                if [position[1], position[2]] == target {
                    println!("Win32 minimap walk confirmed by server at {target:?}");
                    self.minimap_moved = true;
                }
            } else if let Some(view) = live.minimap_view(client) {
                if client
                    .player_state
                    .as_ref()
                    .and_then(|s| s.local_actor())
                    .is_some_and(|p| p.path.len != 0)
                {
                    return Ok(None);
                }
                let layout = client.interfaces.layout(client.canvas)?;
                for &id in &layout.order {
                    let placement = &layout.widgets[&id];
                    let widget = &client.interfaces.widgets[&id];
                    if widget.content != 1338 || placement.hidden {
                        continue;
                    }
                    let Some(mask) = crate::minimap::widget_mask(
                        &mut client.scripts.source.cache,
                        widget,
                        true,
                    )?
                    else {
                        continue;
                    };
                    let local = [position[1] - scene.base_x, position[2] - scene.base_y];
                    let fine = [
                        client.local_fine()[0] - scene.base_x * 128,
                        client.local_fine()[1] - scene.base_y * 128,
                    ];
                    for dy in -8..=8 {
                        for dx in -8..=8 {
                            let point = [
                                mask.width as i32 / 2 + dx,
                                mask.starts.len() as i32 / 2 + dy,
                            ];
                            let click = [
                                point[0] + placement.absolute[0],
                                point[1] + placement.absolute[1],
                            ];
                            let delivered = fit.to_logical(fit.to_physical(click));
                            let point = [
                                delivered[0] - placement.absolute[0],
                                delivered[1] - placement.absolute[1],
                            ];
                            let Some((target, _)) =
                                crate::minimap::click_target(&mask, point, fine, view.angle, 4.0)?
                            else {
                                continue;
                            };
                            let delta = [target[0] - local[0], target[1] - local[1]];
                            if target[1] + scene.base_y >= 127 {
                                continue;
                            }
                            if delta == [0, 0]
                                || !crate::route::can_step(
                                    &scene.collision[position[0] as usize],
                                    local,
                                    1,
                                    delta,
                                )
                            {
                                continue;
                            }
                            let target = [target[0] + scene.base_x, target[1] + scene.base_y];
                            self.minimap_target = Some(target);
                            println!(
                                "Posting Win32 minimap click {click:?} for walk target {target:?}"
                            );
                            return Ok(Some(click));
                        }
                    }
                }
            }
        }
        if self.minimap_moved && !self.clicked_npc && !self.tanner_scanning
            && client.camera.velocity == [0, 0]
        {
            if let Some((point, _)) = live.pick_visible(fit, |tag| {
                tag & 0x4000000000000000 != 0
                    && client
                        .npc_state
                        .npcs
                        .get(&(tag as u16))
                        .is_some_and(|npc| npc.id == 16299)
            }) {
                self.clicked_npc = true;
                self.message_start = client.game_messages.len();
                println!("Posting Win32 click {point:?} for tanner Talk-to");
                return Ok(Some(point));
            }
        }
        if !self.talked
            && self.clicked_npc
            && client
                .game_messages
                .iter()
                .skip(self.message_start)
                .any(|m| m.text == "Bring cowhides. The tanner tans; he does not keep a shop.")
        {
            self.talked = true;
            self.inventory_item = if client.inventories.total(94, self.inventory_item) > 0 {
                1279
            } else {
                1277
            };
            println!("Acceptance seed item: {}", self.inventory_item);
            println!(
                "Acceptance seed required: {}",
                client.inventories.total(93, self.inventory_item) == 0
            );
            self.complete = !self.inventory;
            println!("Win32 mouse Talk-to confirmed by expected server response");
            let layout = client.interfaces.layout(client.canvas)?;
            let mut containers = client.inventories.containers.iter().collect::<Vec<_>>();
            containers.sort_by_key(|(id, _)| **id);
            for (id, items) in containers {
                println!("Acceptance inventory {id}: {items:?}");
            }
            for &id in &layout.order {
                let placement = &layout.widgets[&id];
                let runtime = &client.interfaces.runtime[&id];
                if !placement.hidden && runtime.item >= 0 {
                    println!("Acceptance item widget {}:{} item {} at {:?} size {:?} actions {:?} mask {:08x}",runtime.logical_id,runtime.child_index,runtime.item,placement.absolute,placement.geometry.size,client.interfaces.widgets[&id].actions,client.interfaces.operation_mask(id));
                }
            }
        }
        if self.inventory && self.talked && !self.inventory_equipped {
            if self.inventory_clicked {
                if client.inventories.total(94, self.inventory_item) > 0
                    && client.inventories.total(93, self.inventory_item)
                        == self.inventory_before_click - 1
                {
                    println!("Win32 inventory Wield confirmed by server: test sword moved from inventory to equipment");
                    self.inventory_equipped = true;
                }
            } else if client.inventories.total(93, self.inventory_item) > 0 {
                let layout = client.interfaces.layout(client.canvas)?;
                if !self.reported_item {
                    for (&id, r) in &client.interfaces.runtime {
                        if r.item == self.inventory_item {
                            println!("Inventory acceptance candidate {id} {}:{} layout {:?}, actions {:?}, mask {:08x}",r.logical_id,r.child_index,layout.widgets.get(&id),client.interfaces.widgets[&id].actions,client.interfaces.operation_mask(id));
                        }
                    }
                    self.reported_item = true;
                }
                if !self.inventory_tab_clicked {
                    for &id in layout.order.iter().rev() {
                        let p = &layout.widgets[&id];
                        let w = &client.interfaces.widgets[&id];
                        if !p.hidden && w.actions.iter().any(|a| a == "Inventory") {
                            let point = [
                                p.absolute[0] + p.geometry.size[0] / 2,
                                p.absolute[1] + p.geometry.size[1] / 2,
                            ];
                            if layout.hit_test(point).any(|hit| hit == id) {
                                self.inventory_tab_clicked = true;
                                println!(
                                    "Posting Win32 Inventory tab click {point:?}, widget {id}"
                                );
                                return Ok(Some(point));
                            }
                        }
                    }
                }
                for &id in layout.order.iter().rev() {
                    let p = &layout.widgets[&id];
                    let r = &client.interfaces.runtime[&id];
                    let w = &client.interfaces.widgets[&id];
                    if p.hidden
                        || r.item != self.inventory_item
                        || p.geometry.size[0] <= 0
                        || p.geometry.size[1] <= 0
                    {
                        continue;
                    }
                    let mask = client.interfaces.operation_mask(id);
                    let action = w.actions.iter().enumerate().find(|(i, a)| {
                        !a.trim().is_empty()
                            && (client.interfaces.listener(id, 1409).is_some()
                                || mask & (1u32 << i) != 0)
                    });
                    if !action.is_some_and(|(_, a)| a == "Wield") {
                        continue;
                    }
                    let point = [
                        p.absolute[0] + p.geometry.size[0] / 2,
                        p.absolute[1] + p.geometry.size[1] / 2,
                    ];
                    if !layout.hit_test(point).any(|hit| hit == id) {
                        continue;
                    }
                    if self.drag_phase == 0 {
                        if client.interfaces.drag_parent(id).is_none() {
                            return Err(Error::Invalid("acceptance item has no drag parent"));
                        }
                        let target = layout
                            .order
                            .iter()
                            .filter_map(|target| {
                                let rt = &client.interfaces.runtime[target];
                                let p = &layout.widgets[target];
                                (rt.logical_id == r.logical_id
                                    && rt.child_index >= 0
                                    && rt.child_index != r.child_index
                                    && !p.hidden
                                    && client.interfaces.interaction_flags(*target) & (1 << 20)
                                        != 0)
                                    .then_some((
                                        [
                                            p.absolute[0] + p.geometry.size[0] / 2,
                                            p.absolute[1] + p.geometry.size[1] / 2,
                                        ],
                                        rt.child_index as usize,
                                    ))
                            })
                            .next();
                        let Some(target) = target else {
                            if !self.drag_seed_requested {
                                self.drag_seed_requested = true;
                                for &sibling in &layout.order {
                                    let rt = &client.interfaces.runtime[&sibling];
                                    if rt.logical_id == r.logical_id {
                                        println!("Acceptance drag sibling {}:{} item {} flags {:08x} hidden {}", rt.logical_id, rt.child_index, rt.item, client.interfaces.interaction_flags(sibling), layout.widgets[&sibling].hidden);
                                    }
                                }
                                println!("Acceptance drag target seed required: 1205");
                            }
                            return Ok(None);
                        };
                        self.drag_target = Some(target);
                        self.drag_phase = 1;
                        self.drag_input = 1;
                        println!(
                            "Posting Win32 inventory drag press {point:?}, target slot {} at {:?}",
                            target.1, target.0
                        );
                        return Ok(Some(point));
                    }
                    if self.drag_phase == 1 {
                        self.drag_phase = 2;
                        self.drag_input = 2;
                        return Ok(Some(self.drag_target.unwrap().0));
                    }
                    if self.drag_phase == 2 {
                        if client
                            .widget_drag
                            .as_ref()
                            .is_some_and(|d| d.motion.dragging && d.motion.age > 5)
                        {
                            self.drag_phase = 3;
                            self.drag_input = 3;
                            println!(
                                "Posting Win32 inventory drag release {:?}",
                                self.drag_target.unwrap().0
                            );
                            return Ok(Some(self.drag_target.unwrap().0));
                        }
                        return Ok(None);
                    }
                    if self.drag_phase == 3 {
                        let slot = self.drag_target.unwrap().1;
                        if client
                            .inventories
                            .containers
                            .get(&93)
                            .and_then(|items| items.get(slot))
                            .is_some_and(|item| item.id == self.inventory_item)
                        {
                            self.drag_phase = 4;
                            println!("Win32 inventory drag confirmed by server: item {} now in slot {slot}",self.inventory_item);
                        } else {
                            return Ok(None);
                        }
                    }
                    self.inventory_before_click = client.inventories.total(93, self.inventory_item);
                    self.inventory_clicked = true;
                    println!(
                        "Posting Win32 inventory Wield click {point:?}, widget {}:{}",
                        r.logical_id, r.child_index
                    );
                    return Ok(Some(point));
                }
            }
        }
        if self.inventory_equipped {
            if let Some(item) = self.drop_menu {
                let Some(menu) = &client.menu else {
                    return Ok(None);
                };
                let Some(index) = menu.entries.iter().position(|e| e.option == "Drop") else {
                    return Err(Error::Invalid("inventory context menu missing Drop"));
                };
                let point = [menu.bounds[0] + menu.bounds[2] / 2, menu.row_y(index) - 5];
                self.drop_menu = None;
                self.ground_before = client.inventories.total(93, item);
                self.ground_item = Some(item);
                println!("Posting Win32 context-menu Drop click {point:?}, item {item}");
                return Ok(Some(point));
            }
            if let Some(item) = self.ground_item {
                let candidate = live.pick_visible(fit, |tag| {
                    tag & 0x1000000000000000 != 0 && ((tag >> 32) as u16 as i32) == item
                });
                let piles = client
                    .zone
                    .piles
                    .iter()
                    .filter(|(_, pile)| pile.iter().any(|i| i.id as i32 == item))
                    .map(|(p, items)| {
                        (
                            *p,
                            items
                                .iter()
                                .filter(|i| i.id as i32 == item)
                                .map(|i| i.quantity)
                                .collect::<Vec<_>>(),
                        )
                    })
                    .collect::<Vec<_>>();
                let diagnostic=format!("Ground acceptance item {item}: inventory {}, piles {piles:?}, player {position:?}, visible pick {candidate:?}",client.inventories.total(93,item));
                if diagnostic != self.ground_diagnostic {
                    println!("{diagnostic}");
                    self.ground_diagnostic = diagnostic;
                }

                if self.ground_clicked {
                    if client.inventories.total(93, item) == self.ground_before {
                        println!("Win32 ground-item Take confirmed by server: dropped test sword returned to inventory");
                        self.inventory_done = true;
                    }
                } else if client.inventories.total(93, item) == self.ground_before - 1
                    && client
                        .zone
                        .piles
                        .values()
                        .any(|pile| pile.iter().any(|i| i.id as i32 == item))
                {
                    // The local player can occlude a small item dropped under its feet.
                    // Move aside through the same native input path before picking it up.
                    if self.ground_step.is_none() {
                        if let Some((point, tag)) = live.pick_visible(fit, |tag| {
                            if tag >> 63 != 1 {
                                return false;
                            }
                            let target = [((tag >> 14) & 16383) as i32, (tag & 16383) as i32];
                            let local = [position[1] - scene.base_x, position[2] - scene.base_y];
                            let delta = [target[0] - local[0], target[1] - local[1]];
                            delta != [0, 0]
                                && crate::route::can_step(
                                    &scene.collision[position[0] as usize],
                                    local,
                                    1,
                                    delta,
                                )
                        }) {
                            let target = [
                                ((tag >> 14) & 16383) as i32 + scene.base_x,
                                (tag & 16383) as i32 + scene.base_y,
                            ];
                            self.ground_step = Some(target);
                            println!("Posting Win32 step-away click {point:?}, target {target:?}");
                            return Ok(Some(point));
                        }
                        return Ok(None);
                    }
                    if self.ground_step != Some([position[1], position[2]]) {
                        return Ok(None);
                    }
                    if let Some((point, _)) = candidate {
                        self.ground_clicked = true;
                        println!("Posting Win32 ground-item Take click {point:?}, item {item}");
                        return Ok(Some(point));
                    }
                }
            } else {
                if let Some(target) = self.drop_target {
                    if (target[0] - position[1])
                        .abs()
                        .max((target[1] - position[2]).abs())
                        > 1
                    {
                        return Ok(None);
                    }
                    if self
                        .drop_observed
                        .as_ref()
                        .is_none_or(|(p, _)| *p != position)
                    {
                        self.drop_observed = Some((position, std::time::Instant::now()));
                        println!("Ground preparation reached server position {position:?}");
                        return Ok(None);
                    }
                    if self.drop_observed.as_ref().unwrap().1.elapsed().as_millis() < 1000 {
                        return Ok(None);
                    }
                } else {
                    let local = [position[1] - scene.base_x, position[2] - scene.base_y];
                    let mut reachable = std::collections::HashMap::new();
                    if let Some((point, tag)) = live.pick_where(|tag| {
                        if tag >> 63 != 1 {
                            return false;
                        }
                        let t = [((tag >> 14) & 16383) as i32, (tag & 16383) as i32];
                        if t[1] + scene.base_y >= 127 {
                            return false;
                        }
                        let distance = (t[0] - local[0]).abs().max((t[1] - local[1]).abs());
                        if !(4..=6).contains(&distance) {
                            return false;
                        }
                        *reachable.entry(t).or_insert_with(|| {
                            crate::route::Search::default()
                                .find(&scene.collision[position[0] as usize], local, 1, t)
                                .ok()
                                .flatten()
                                .is_some_and(|r| r.reached)
                        })
                    }) {
                        let target = [
                            ((tag >> 14) & 16383) as i32 + scene.base_x,
                            (tag & 16383) as i32 + scene.base_y,
                        ];
                        self.drop_target = Some(target);
                        println!("Posting Win32 open-ground preparation click {point:?}, target {target:?}");
                        return Ok(Some(point));
                    }
                    return Ok(None);
                }
                let layout = client.interfaces.layout(client.canvas)?;
                for &id in layout.order.iter().rev() {
                    let r = &client.interfaces.runtime[&id];
                    let w = &client.interfaces.widgets[&id];
                    if layout.widgets[&id].hidden
                        || r.logical_id != 9764864
                        || !matches!(r.item, 1277 | 1279)
                    {
                        continue;
                    }
                    let Some(_) = w.actions.iter().enumerate().find(|(i, a)| {
                        *a == "Drop" && client.interfaces.operation_mask(id) & (1 << i) != 0
                    }) else {
                        continue;
                    };
                    let item = r.item;
                    let p = &layout.widgets[&id];
                    let point = [
                        p.absolute[0] + p.geometry.size[0] / 2,
                        p.absolute[1] + p.geometry.size[1] / 2,
                    ];
                    self.drop_menu = Some(item);
                    self.right_click = true;
                    println!("Posting Win32 inventory right-click {point:?}, item {item}");
                    return Ok(Some(point));
                }
            }
        }
        Ok(None)
    }
}
