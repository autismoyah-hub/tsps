//! Actor mesh placement for the live world. Animation/overheads are tracked separately.
use crate::{
    client_state::ClientState,
    identity_kit::Kit,
    item_model::Models,
    model::Model,
    render3d::{texture_uv, Triangle},
    scene::Scene,
    Error, Result,
};
use std::collections::HashMap;
pub struct Actors {
    palette: Vec<u32>,
    pub overlap_bounds: Vec<crate::world_overlap::Bounds>,
    models: Models,
    pub(crate) location_models: crate::object_model::Models,
    pub(crate) animations: crate::animation_runtime::Animations,
    kits: HashMap<u32, Kit>,
    ground: HashMap<(u32, Option<crate::zone::Custom>), (Model, Vec<[i32; 3]>)>,
    players: crate::player_model_cache::Cache<(Model, Vec<[i32; 3]>)>,
    npcs: HashMap<(u32, Option<crate::actor_update::Customisation>), (Model, Vec<[i32; 3]>)>,
    pub visible_npcs: HashMap<u16, u32>,
    pub heights: HashMap<u64, i32>,
    target_varp: Option<u16>,
    spots: HashMap<u16, (Model, Vec<[i32; 3]>)>,
    steering_models: [Option<(Model, Vec<[i32; 3]>)>; 2],
}
impl Actors {
    pub fn load(cache: &mut crate::disk_cache::DiskCache) -> Result<Self> {
        Ok(Self {
            palette: crate::floor::palette(0.8)?,
            overlap_bounds: Vec::new(),
            models: Default::default(),
            location_models: Default::default(),
            animations: crate::animation_runtime::Animations::load(cache)?,
            kits: Kit::load_all(cache)?,
            ground: HashMap::new(),
            players: Default::default(),
            npcs: HashMap::new(),
            visible_npcs: HashMap::new(),
            heights: HashMap::new(),
            target_varp: crate::overhead::overlay_varps(cache)?[1],
            spots: HashMap::new(),
            steering_models: crate::steering_scene::load(cache)?,
        })
    }
    /// Prepare root-only arrows before all scene occupancy planning.
    pub fn prepare_steering(&self, client: &mut ClientState, shape: crate::viewport_shape::Shape, hovered: Option<u8>) {
        client.steering.scene = crate::steering_scene::placements(shape.size[1], shape.focal,
            [client.camera.tracked[0], client.camera.tracked_height, client.camera.tracked[1]],
            client.steering, hovered, self.steering_models.each_ref().map(|m| m.is_some()));
    }
    fn animation_height(&self,client:&ClientState,world:Option<u16>,tag:u64)->i32 {
        let sequences=&self.animations.sequences;
        match tag>>60 {
            2=>client.player_state.as_ref().and_then(|s|s.root_owned_actor(world.unwrap_or(0),tag as u16,&client.world_entities))
                .map_or(0,|p|crate::actor_height::player_ready(sequences,p,&client.animation_readiness)),
            4=>{let state=if let Some(id)=world{client.world_entities.get(&0).and_then(|w|w.entities.get(&id)).map(|e|&e.npcs)}else{Some(&client.npc_state)};
                state.and_then(|s|s.npcs.get(&(tag as u16))).map_or(0,|n|crate::actor_height::npc_ready(sequences,n,&client.animation_readiness))},
            _=>0,
        }
    }
    /// GameActor.ee / GameNPC.ee expose overlay heights separately from model bounds.
    pub fn overlay_heights(&self,client:&ClientState,world:Option<u16>)->HashMap<u64,i32> {
        self.heights.iter().map(|(&tag,&height)| {
            let base=if tag>>60==4 {
                let state=if let Some(id)=world{client.world_entities.get(&0).and_then(|w|w.entities.get(&id)).map(|e|&e.npcs)}else{Some(&client.npc_state)};
                state.and_then(|s|s.npcs.get(&(tag as u16))).and_then(|n|client.npcs.get(&(n.id as u32)))
                    .map_or(height,|d|if d.overlay_height==-1{height}else{d.overlay_height})
            }else{height};
            (tag,base.wrapping_add(self.animation_height(client,world,tag)))
        }).collect()
    }
    fn shift_actor_heights(&self,client:&ClientState,world:Option<u16>,triangles:&mut [Triangle]) {
        let mut offsets=HashMap::new();
        for triangle in triangles {
            let Some(order)=triangle.model_order.as_mut() else{continue;};
            if order.scene_height_offset!=0 {continue;}
            let tag=order.actor_spot.unwrap_or(triangle.tag);
            if !matches!(tag>>60,2|4){continue;}
            let offset=*offsets.entry(tag).or_insert_with(||self.animation_height(client,world,tag));
            if offset==0{continue;}
            order.scene_height_offset=offset;
            for v in &mut triangle.vertices{v[1]-=offset as f32;}
            if let Some(mapping)=&mut triangle.texture_mapping {for v in &mut mapping.vertices{v[1]-=offset as f32;}}
            if let Some(bounds)=&mut triangle.model_bounds{bounds.position[1]=bounds.position[1].wrapping_sub(offset);}
        }
    }
    fn invalidate_players(&mut self, client: &mut ClientState) {
        if let Some(state) = &mut client.player_state {
            for key in state.model_invalidations.drain() { self.players.remove(key); }
        }
    }
    fn player_model_key(
        &mut self,
        cache: &mut crate::disk_cache::DiskCache,
        items: &crate::item::Definitions,
        appearance: &crate::player_model::Appearance,
        last: &std::cell::Cell<u64>,
    ) -> Result<Option<u64>> {
        use crate::player_model_cache::Choice;
        let key = appearance.render_key();
        let ready = if self.players.contains(key) { true } else {
            appearance.models_ready(&self.kits, items, |id| {
                self.models.available(cache, id)
            })?
        };
        match self.players.choose(key, last.get(), ready) {
            Choice::Cached(key) => Ok(Some(key)),
            Choice::Unavailable => Ok(None),
            Choice::Build => {
                let model = appearance.mesh(cache, &mut self.models, &self.kits, items, false)?;
                let colors = model.light(64, 850, [-30, -50, -30])?;
                self.players.insert(key, (model, colors));
                last.set(key);
                Ok(Some(key))
            }
        }
    }
    /// Retain original scene insertion order across separately assembled meshes.
    pub fn register_submission_order(&self,frame:&mut crate::render3d::Frame,client:&mut ClientState,scene:&Scene,world:Option<u16>)->Result<()> {
        let target=self.target_varp.map(|id|client.scripts.variables.player(id)).filter(|&v|v!=-1).map(|v|(v&2047) as u16);
        for (ordinal,key) in submission_plan(client,scene,world,target)?.into_iter().enumerate(){
            let tag=(if key.npc{0x4000000000000000}else{0x2000000000000000})|key.index as u64;
            frame.temporary_source(world.unwrap_or(0),tag,ordinal+if world.is_none(){2}else{0});
        }
        Ok(())
    }
    /// Root child-scene admission uses the same occupancy as actor/effect rendering.
    pub fn root_child_submissions(&self,client:&mut ClientState,scene:&Scene)->Result<std::collections::HashSet<u16>> {
        let target=self.target_varp.map(|id|client.scripts.variables.player(id)).filter(|&v|v!=-1).map(|v|(v&2047) as u16);
        root_child_submissions(client,scene,target)
    }
    /// Scene references used by actor, effect and child-scene submission this frame.
    pub fn temporary_objects(&self,client:&mut ClientState,scene:&Scene,world:Option<u16>)->Result<Vec<(usize,crate::object_traversal::Object)>> {
        Ok(self.temporary_draw_objects(client,scene,world)?.0)
    }
    /// Actor body tags retain their temporary input identity independently of allocation IDs.
    pub fn temporary_draw_objects(&self,client:&mut ClientState,scene:&Scene,world:Option<u16>)->Result<(Vec<(usize,crate::object_traversal::Object)>,Vec<Option<u64>>)> {
        let (objects,tags,_)=self.temporary_scene_objects(client,scene,world)?;
        Ok((objects,tags))
    }
    /// Parent insertion identities distinguish root-owned child scenes from anonymous effects.
    pub fn temporary_scene_objects(&self,client:&mut ClientState,scene:&Scene,world:Option<u16>)->Result<(Vec<(usize,crate::object_traversal::Object)>,Vec<Option<u64>>,Vec<(usize,u16)>)> {
        let target=self.target_varp.map(|id|client.scripts.variables.player(id)).filter(|&v|v!=-1).map(|v|(v&2047) as u16);
        let objects=temporary_submission_state(client,scene,world,target)?.1;
        let mut records=Vec::new();let mut tags=Vec::new();let mut children=Vec::new();
        for (index,(plane,object,tag,child)) in objects.into_iter().enumerate() {
            records.push((plane,object));tags.push(tag);if let Some(view)=child {children.push((index,view));}
        }
        Ok((records,tags,children))
    }
    /// sf.cs inserts started attached effects after every admitted actor body.
    pub fn attached_spot_objects(&self,client:&ClientState,world:Option<u16>,
        objects:&[(usize,crate::object_traversal::Object)],tags:&[Option<u64>]) -> (Vec<(usize,[i32;2])>,Vec<(u64,u8)>) {
        let view=world.unwrap_or(0);let mut records=Vec::new();let mut keys=Vec::new();
        for ((plane,object),tag) in objects.iter().zip(tags) {
            let Some(owner)=*tag else {continue;};let index=owner as u16;
            let effects=if owner>>60==4 {
                let state=if let Some(id)=world {client.world_entities.get(&0).and_then(|w|w.entities.get(&id)).map(|e|&e.npcs)}else{Some(&client.npc_state)};
                state.and_then(|state|state.npcs.get(&index)).map(|npc|&npc.spots)
            }else {client.player_state.as_ref().and_then(|state|state.root_owned_actor(view,index,&client.world_entities)).map(|player|&player.spots)};
            if let Some(effects)=effects {for slot in effects.ordered_slots() {
                if client.actor_cycle>=effects.slots[&slot].start {records.push((*plane,object.center));keys.push((owner,slot));}
            }}
        }
        (records,keys)
    }
    pub fn set_palette(&mut self,palette:&[u32]) {
        self.palette.clear();self.palette.extend_from_slice(palette);
    }
    /// Render NPC bodies in a world view using the same model and animation path.
    /// Root rendering clears the visibility/height maps before publishing overlays.
    pub fn npc_triangles(
        &mut self,
        client: &mut ClientState,
        scene: &Scene,
        world: Option<u16>,
    ) -> Result<(Vec<Triangle>, Vec<String>)> {
        use crate::script_runtime::Source;
        self.invalidate_players(client);
        let target=self.target_varp.map(|id|client.scripts.variables.player(id)).filter(|&v|v!=-1).map(|v|(v&2047) as u16);
        let (accepted,mut temporary_slots,_,objects)=submission_state_with_objects(client,scene,world,target)?;
        let mut temporary_id=objects.len();
        let submission_order=accepted.into_iter().filter_map(|key|key.npc.then_some(key.index)).collect::<Vec<_>>();
        self.visible_npcs.clear();
        if world.is_some() {
            self.heights.clear();
        }
        let (state, plane, world_spots) = if let Some(id) = world {
            let entity = client
                .world_entities
                .get_mut(&0)
                .and_then(|state| state.entities.get_mut(&id))
                .ok_or(Error::Invalid("NPC render world missing"))?;
            (&entity.npcs, entity.plane, Some(&mut entity.world_spots))
        } else {
            (&client.npc_state, client.data.position[0] as u8, None)
        };
        let collapsed: HashMap<u16, u16> = state
            .npcs
            .iter()
            .filter_map(|(&index, npc)| {
                let def = client.npcs.get(&(npc.id as u32))?;
                crate::world_overlap::npc_obscured(
                    def.hide_overlap,
                    [
                        npc.path.position[0] - scene.base_x * 128,
                        npc.path.position[1] - scene.base_y * 128,
                    ],
                    def.size as i32,
                    &self.overlap_bounds,
                )
                .then_some((index, def.overlap))
            })
            .collect();
        let mut out = Vec::new();
        let mut pending = Vec::new();

        let mut effect_npcs=std::collections::HashSet::new();
        for &index in &submission_order {
            let npc=&state.npcs[&index];
            let overlap = collapsed.get(&index).copied();
            let idle = npc.animations[0];
            let Some(d) = crate::npc_model::resolve(&client.npcs, npc.id as u32, |bit, id| {
                if bit {
                    let b = client.scripts.source.varbit(id as i32)?;
                    Ok(b.get(client.scripts.variables.player(b.base)))
                } else {
                    Ok(client.scripts.variables.player(id))
                }
            })?
            else {
                continue;
            };
            effect_npcs.insert(index);
            let Some((action, pose)) = crate::actor_pose::select(
                &npc.freezes, &npc.sequence, &npc.pose.state,
                &npc.idle_fallback.state, idle, false, |id| self.animations.sequences.get(&(id as u32)).is_some_and(|sequence| client.animation_readiness.ready(&mut client.scripts.source.cache,sequence)),
            ) else { self.heights.remove(&(0x4000000000000000 | index as u64)); continue; };
            self.visible_npcs.insert(index, d.id);

            let body = npc.appearance.body.as_ref();
            if body.is_some_and(|b| b.use_player) {
                let Some(player) = client.player_state.as_ref().and_then(|s| s.local_actor()) else { continue; };
                let Some(a) = player.appearance.as_ref() else { continue; };
                if let Some(id) = a.appearance.npc {
                    let Some(def) =
                        crate::npc_model::resolve(&client.npcs, id as u32, |bit, id| {
                            if bit {
                                let b = client.scripts.source.varbit(id as i32)?;
                                Ok(b.get(client.scripts.variables.player(b.base)))
                            } else {
                                Ok(client.scripts.variables.player(id))
                            }
                        })?
                    else {
                        continue;
                    };
                    let key = (def.id, None);
                    if !self.npcs.contains_key(&key) {
                        let Some(model) = crate::npc_model::mesh(
                            &mut client.scripts.source.cache,
                            &mut self.models,
                            def,
                            false,
                            None,
                        )?
                        else {
                            continue;
                        };
                        let colors =
                            model.light(64 + def.ambient, 850 + def.contrast, [-30, -50, -30])?;

                        if self.npcs.len() >= 256 {
                            self.npcs.clear();
                        }
                        self.npcs.insert(key.clone(), (model, colors));
                    }
                    let (model, colors) = &self.npcs[&key];
                    let mut model = model.clone();
                    self.animations.apply_actor(
                        &mut client.scripts.source.cache,
                        &mut model,
                        action,
                        pose,
                        idle,
                    )?;
                    crate::npc_model::resize(&mut model, def);
                    let x = npc.path.position[0] - scene.base_x * 128;
                    let z = npc.path.position[1] - scene.base_y * 128;
                    if let Some(h) = scene.elevation(plane as usize, x, z) {
                        self.heights.insert(
                            0x4000000000000000 | index as u64,
                            model
                                .vertices
                                .iter()
                                .map(|v| -(v[1] as i32))
                                .max()
                                .unwrap_or(0)
                                .max(0),
                        );
                        place_actor(
                            &mut out,
                            &model,
                            colors,
                            [x, h, z],
                            npc.angle,
                            0x4000000000000000 | index as u64,
                            client.npcs.get(&(npc.id as u32)).is_some_and(|d| d.size == 1),
                            &npc.visual,
                            client.actor_cycle,
                            true,
                            overlap,
                            &self.palette,
                            crate::model_order::Mode::actor(false,client.npcs.get(&(npc.id as u32)).is_none_or(|d|d.z_buffer),body.is_some_and(|b|b.use_player)),
)?;
                    }
                    continue;
                }
                let Some(hash) = self.player_model_key(
                    &mut client.scripts.source.cache, &client.items, &a.appearance, &player.last_model,
                )? else { continue; };
                let (model, colors) = self.players.get(hash);
                let mut model = model.clone();
                self.animations.apply_actor(
                    &mut client.scripts.source.cache,
                    &mut model,
                    action,
                    pose,
                    idle,
                )?;
                let x = npc.path.position[0] - scene.base_x * 128;
                let z = npc.path.position[1] - scene.base_y * 128;
                if let Some(h) = scene.elevation(plane as usize, x, z) {
                    self.heights.insert(
                        0x4000000000000000 | index as u64,
                        model
                            .vertices
                            .iter()
                            .map(|v| -(v[1] as i32))
                            .max()
                            .unwrap_or(0)
                            .max(0),
                    );
                    place_actor(
                        &mut out,
                        &model,
                        colors,
                        [x, h, z],
                        npc.angle,
                        0x4000000000000000 | index as u64,
                        client.npcs.get(&(npc.id as u32)).is_some_and(|d| d.size == 1),
                        &npc.visual,
                        client.actor_cycle,
                        true,
                        overlap,
                        &self.palette,
                        crate::model_order::Mode::actor(false,client.npcs.get(&(npc.id as u32)).is_none_or(|d|d.z_buffer),body.is_some_and(|b|b.use_player)),
)?;
                }
                continue;
            }
            let key = (d.id, body.cloned());
            if let Some(custom) = body.filter(|b| b.composition.is_some()) {
                if !self.npcs.contains_key(&key) {
                    let gender = custom.composition.as_ref().unwrap().0;
                    let mut appearance =
                        crate::player_model::Appearance::standard(&self.kits, gender);
                    if let Some(equipment) = &custom.models {
                        let Ok(slots) = <[u32; 12]>::try_from(equipment.as_slice()) else {
                            pending.push("NPC player composition equipment array size".into());
                            continue;
                        };
                        appearance.equipment = slots;
                    }
                    let colors = if let Some(colors) = &custom.recolor {
                        if colors.len() < 5 {
                            return Err(Error::Invalid("NPC player composition palette length"));
                        }
                        std::array::from_fn(|i| colors[i] as i16 as i32)
                    } else {
                        [0; 5]
                    };
                    let model = appearance.mesh_with_colors(
                        &mut client.scripts.source.cache,
                        &mut self.models,
                        &self.kits,
                        &client.items,
                        false,
                        colors,
                    )?;
                    let colors = model.light(64, 850, [-30, -50, -30])?;
                    if self.npcs.len() >= 256 {
                        self.npcs.clear();
                    }
                    self.npcs.insert(key.clone(), (model, colors));
                }
                let (model, colors) = &self.npcs[&key];
                let mut model = model.clone();
                self.animations.apply_actor(
                    &mut client.scripts.source.cache,
                    &mut model,
                    action,
                    pose,
                    idle,
                )?;
                let x = npc.path.position[0] - scene.base_x * 128;
                let z = npc.path.position[1] - scene.base_y * 128;
                if let Some(h) = scene.elevation(plane as usize, x, z) {
                    self.heights.insert(
                        0x4000000000000000 | index as u64,
                        model
                            .vertices
                            .iter()
                            .map(|v| -(v[1] as i32))
                            .max()
                            .unwrap_or(0)
                            .max(0),
                    );
                    place_actor(
                        &mut out,
                        &model,
                        colors,
                        [x, h, z],
                        npc.angle,
                        0x4000000000000000 | index as u64,
                        client.npcs.get(&(npc.id as u32)).is_some_and(|d| d.size == 1),
                        &npc.visual,
                        client.actor_cycle,
                        true,
                        overlap,
                        &self.palette,
                        crate::model_order::Mode::actor(false,client.npcs.get(&(npc.id as u32)).is_none_or(|d|d.z_buffer),body.is_some_and(|b|b.use_player)),
)?;
                }
                continue;
            }
            if !self.npcs.contains_key(&key) {
                let custom = body.map(|c| crate::npc_model::Overrides {
                    models: c.models.clone(),
                    recolor: c.recolor.clone(),
                    retexture: c.retexture.clone(),
                    use_player: false,
                });
                let Some(model) = crate::npc_model::mesh(
                    &mut client.scripts.source.cache,
                    &mut self.models,
                    d,
                    false,
                    custom.as_ref(),
                )?
                else {
                    pending.push(format!("NPC {} model unavailable", npc.id));
                    continue;
                };
                let colors = model.light(64 + d.ambient, 850 + d.contrast, [-30, -50, -30])?;

                if self.npcs.len() >= 256 {
                    self.npcs.clear();
                }
                self.npcs.insert(key.clone(), (model, colors));
            }
            let (model, colors) = &self.npcs[&key];
            let mut model = model.clone();
            self.animations.apply_actor(
                &mut client.scripts.source.cache,
                &mut model,
                action,
                pose,
                idle,
            )?;
            crate::npc_model::resize(&mut model, d);
            let x = npc.path.position[0] - scene.base_x * 128;
            let z = npc.path.position[1] - scene.base_y * 128;
            if let Some(h) = scene.elevation(plane as usize, x, z) {
                self.heights.insert(
                    0x4000000000000000 | index as u64,
                    model
                        .vertices
                        .iter()
                        .map(|v| -(v[1] as i32))
                        .max()
                        .unwrap_or(0)
                        .max(0),
                );
                place_actor(
                    &mut out,
                    &model,
                    colors,
                    [x, h, z],
                    npc.angle,
                    0x4000000000000000 | index as u64,
                    client.npcs.get(&(npc.id as u32)).is_some_and(|d| d.size == 1),
                    &npc.visual,
                    client.actor_cycle,
                    true,
                    overlap,
                    &self.palette,
                    crate::model_order::Mode::actor(false,client.npcs.get(&(npc.id as u32)).is_none_or(|d|d.z_buffer),body.is_some_and(|b|b.use_player)),
)?;
            }
        }
        for &index in &submission_order {
            let npc=&state.npcs[&index];
            let overlap = collapsed.get(&index).copied();
            if !effect_npcs.contains(&index) {
                continue;
            }
            let x = npc.path.position[0] - scene.base_x * 128;
            let z = npc.path.position[1] - scene.base_y * 128;
            let Some(h) = scene.elevation(plane as usize, x, z) else {
                continue;
            };
            for slot in npc.spots.ordered_slots() {
                let effect=&npc.spots.slots[&slot];
                if client.actor_cycle < effect.start {
                    continue;
                }
                let d = client
                    .spots
                    .get(&(effect.id as u32))
                    .ok_or(Error::Invalid("render NPC spot definition"))?;
                let spot_start=out.len();
                self.place_spot(
                    &mut client.scripts.source.cache,
                    d,
                    effect,
                    [x, h, z],
                    npc.angle,
                    0x4000000000000000 | index as u64,
                    &mut out,
                    Some(&npc.visual),
                    client.actor_cycle,
                    overlap,
                )?;
                mark_spot_slot(&mut out[spot_start..],Some(slot));
            }
        }
        if let Some(effects) = world_spots {
            for effect in effects {
                if effect.position[0] != plane as i32 || client.actor_cycle.wrapping_sub(1) < effect.animation.start
                {
                    continue;
                }
                let x = (effect.position[1] - scene.base_x) * 128 + 64;
                let z = (effect.position[2] - scene.base_y) * 128 + 64;
                if !reserve_effect(&mut temporary_slots,plane as usize,[x,z]){continue;}
                let effect_id=temporary_id;temporary_id+=1;
                if effect.elevation.is_none() {
                    effect.elevation = scene.elevation(plane as usize, x, z);
                }
                if let Some(h) = effect.elevation {
                    let definition = client
                        .spots
                        .get(&(effect.animation.id as u32))
                        .ok_or(Error::Invalid("render child world spot definition"))?;
                    let effect_start=out.len();
                    self.place_spot(
                        &mut client.scripts.source.cache,
                        definition,
                        &effect.animation,
                        [x, h, z],
                        0,
                        0,
                        &mut out,
                        None,
                        client.actor_cycle,
                        None,
                    )?;
                    mark_temporary(&mut out[effect_start..],Some(effect_id));
                }
            }
        }
        self.shift_actor_heights(client,world,&mut out);
        Ok((out, pending))
    }
    fn place_spot(
        &mut self,
        cache: &mut crate::disk_cache::DiskCache,
        d: &crate::spot_animation::Definition,
        effect: &crate::actor_spot::Effect,
        position: [i32; 3],
        angle: u16,
        tag: u64,
        out: &mut Vec<Triangle>,
        visual: Option<&crate::actor_visual::State>,
        cycle: i32,
        overlap: Option<u16>,
    ) -> Result<()> {
        if !self.spots.contains_key(&effect.id) {
            let Some(model) = d.base_model(cache, &mut self.models)? else {
                return Ok(());
            };
            let colors = model.light(64 + d.ambient, 850 + d.contrast, [-30, -50, -30])?;
            if self.spots.len() >= 256 {
                self.spots.clear();
            }
            self.spots.insert(effect.id, (model, colors));
        }
        let (base, colors) = &self.spots[&effect.id];
        let mut model = base.clone();
        if d.sequence >= 0 {
            self.animations
                .apply(cache, &mut model, d.sequence as u32, &effect.clock)?;
        }
        model.scale([d.scale[0], d.scale[1], d.scale[0]]);
        if matches!(d.rotation, 90 | 180 | 270) {
            model.rotate_quarter((d.rotation / 90) as u8);
        }
        model.translate([0, -(effect.height as i32), 0]);
        let start=out.len();
        if overlap.is_some() {
            let default_visual = crate::actor_visual::State::default();
            place_actor(
                out,
                &model,
                colors,
                position,
                angle,
                if d.flag { tag } else { 0 },
                false,
                visual.unwrap_or(&default_visual),
                cycle,
                true,
                overlap,
                &self.palette,
                crate::model_order::Mode::Auto,
)?;
        } else {
        place(
            out,
            &model,
            colors,
            position,
            angle,
            if d.flag { tag } else { 0 },
            false,
        )?;
        if let Some(visual) = visual {
            apply_alpha(out, start, visual.alpha(cycle));
        }
        }
        if tag!=0 {for face in &mut out[start..] {if let Some(order)=&mut face.model_order {order.actor_spot=Some(tag);}}}
        Ok(())
    }
    /// Player bodies use per-view actor clocks; meshes remain shared by composition.
    pub fn player_triangles(
        &mut self,
        client: &mut ClientState,
        scene: &Scene,
        world: Option<u16>,
    ) -> Result<Vec<Triangle>> {
        use crate::script_runtime::Source;
        self.invalidate_players(client);
        let target = self
            .target_varp
            .map(|id| client.scripts.variables.player(id))
            .filter(|&v| v != -1)
            .map(|v| (v & 2047) as u16);
        let plane = if let Some(id) = world {
            client
                .world_entities
                .get(&0)
                .and_then(|w| w.entities.get(&id))
                .ok_or(Error::Invalid("player render world missing"))?
                .plane as i32
        } else {
            client.data.position[0]
        };
        let local = client.player_state.as_ref().map_or(0, |s| s.local);
        let accepted=submission_plan(client,scene,world,target)?;
        let players: Vec<_> = client.player_state.as_ref().map_or_else(Vec::new, |s| {
            let view=world.unwrap_or(0);
            accepted.iter().filter(|key|!key.npc)
                .filter_map(|key|s.root_owned_actor(view,key.index,&client.world_entities).map(|p|(key.index,p))).collect()
        });
        let mut out = Vec::new();
        let mut bodies = HashMap::new();
        {
            for &(index, player) in &players {
                if index == local && !client.preferences.render_self && Some(index) != target {
                    continue;
                }
                let idle = player.appearance.as_ref().map_or(-1, |a| a.animations[0]);
                // Native login currently requests high detail; the 200-player rule still applies.
                let low_pose = crate::player_preview::crowded_pose(
                    client.player_state.as_ref().map_or(0, |s| s.high.len()), false,
                    index != local && Some(index) != target, player.pose.state.id, idle,
                    player.attachment.as_ref().is_some_and(|a| a.placement.active(client.actor_cycle)),
                );
                let Some((action, pose)) = crate::player_preview::select(
                    &player.freezes, &player.sequence, &player.pose.state,
                    &player.idle_fallback.state, idle, low_pose, |id| self.animations.sequences.get(&(id as u32)).is_some_and(|sequence| client.animation_readiness.ready(&mut client.scripts.source.cache,sequence)),
                ) else { self.heights.remove(&(0x2000000000000000 | index as u64)); continue; };
                let Some(a) = &player.appearance else {
                    continue;
                };
                if a.hidden || player.position[0] != plane {
                    continue;
                }
                if let Some(npc_id) = a.appearance.npc {
                    let Some(d) =
                        crate::npc_model::resolve(&client.npcs, npc_id as u32, |bit, id| {
                            if bit {
                                let b = client.scripts.source.varbit(id as i32)?;
                                Ok(b.get(client.scripts.variables.player(b.base)))
                            } else {
                                Ok(client.scripts.variables.player(id))
                            }
                        })?
                    else {
                        continue;
                    };
                    let key = (d.id, None);
                    if !self.npcs.contains_key(&key) {
                        let Some(model) = crate::npc_model::mesh(
                            &mut client.scripts.source.cache,
                            &mut self.models,
                            d,
                            false,
                            None,
                        )?
                        else {
                            continue;
                        };
                        let colors =
                            model.light(64 + d.ambient, 850 + d.contrast, [-30, -50, -30])?;

                        self.npcs.insert(key.clone(), (model, colors));
                    }
                    let x = player.path.position[0] - scene.base_x * 128;
                    let z = player.path.position[1] - scene.base_y * 128;
                    if let Some(h) = scene.elevation(player.position[0] as usize, x, z) {
                        let (model, colors) = &self.npcs[&key];
                        let mut model = model.clone();
                        self.animations.apply_actor(
                            &mut client.scripts.source.cache,
                            &mut model,
                            action,
                            pose,
                            idle,
                        )?;
                        crate::npc_model::resize(&mut model, d);
                        self.heights.insert(
                            0x2000000000000000 | index as u64,
                            model
                                .vertices
                                .iter()
                                .map(|v| -(v[1] as i32))
                                .max()
                                .unwrap_or(0)
                                .max(0),
                        );
                        let body_start = out.len();
                        place_actor(
                            &mut out,
                            &model,
                            colors,
                            [x, h, z],
                            player.turn.angle,
                            0x2000000000000000 | index as u64,
                            true,
                            &player.visual,
                            client.actor_cycle,
                            true,
                            None,
                            &self.palette,
                            crate::model_order::Mode::Priority,
)?;
                        bodies.insert(index, (crate::model_bounds::Bounds::new(&model.vertices, [x,h,z]).with_selection(&model.vertices, player.turn.angle, true), body_start..out.len()));
                    }
                    continue;
                }
                let appearance = if let Some(action) = action {
                    a.appearance
                        .sequence_equipment(&self.animations.sequences[&(action.id as u32)])
                } else {
                    a.appearance.clone()
                };
                let Some(hash) = self.player_model_key(
                    &mut client.scripts.source.cache, &client.items, &appearance, &player.last_model,
                )? else {
                    self.heights.remove(&(0x2000000000000000 | index as u64));
                    continue;
                };
                let (model, colors) = self.players.get(hash);
                let mut model = model.clone();
                self.animations.apply_actor(
                    &mut client.scripts.source.cache,
                    &mut model,
                    action,
                    pose,
                    idle,
                )?;
                let x = player.path.position[0] - scene.base_x * 128;
                let z = player.path.position[1] - scene.base_y * 128;
                if let Some(h) = scene.elevation(player.position[0] as usize, x, z) {
                    self.heights.insert(
                        0x2000000000000000 | index as u64,
                        model
                            .vertices
                            .iter()
                            .map(|v| -(v[1] as i32))
                            .max()
                            .unwrap_or(0)
                            .max(0),
                    );
                    let body_start = out.len();
                    place_actor(
                        &mut out,
                        &model,
                        colors,
                        [x, h, z],
                        player.turn.angle,
                        0x2000000000000000 | index as u64,
                        true,
                        &player.visual,
                        client.actor_cycle,
                        true,
                            None,
                        &self.palette,
                        crate::model_order::Mode::Priority,
)?;
                    bodies.insert(index, (crate::model_bounds::Bounds::new(&model.vertices, [x,h,z]).with_selection(&model.vertices, player.turn.angle, true), body_start..out.len()));
                }
            }
        }
        {
            for &(index, player) in &players {
                if index == local && !client.preferences.render_self && Some(index) != target {
                    continue;
                }
                let tag = 0x2000000000000000 | index as u64;
                if !self.heights.contains_key(&tag) {
                    continue;
                }
                let Some(attachment) = player
                    .attachment
                    .as_ref()
                    .filter(|a| a.placement.active(client.actor_cycle))
                else {
                    continue;
                };
                let x = player.path.position[0] - scene.base_x * 128;
                let z = player.path.position[1] - scene.base_y * 128;
                if let Some(height) = scene.elevation(player.position[0] as usize, x, z) {
                    let position = [player.path.position[0], height, player.path.position[1]];
                    let model = attachment.relative_model(position, player.turn.target);
                    let attachment_start = out.len();
                    place_actor(
                        &mut out,
                        &model,
                        &attachment.colors,
                        [x, height, z],
                        player.turn.angle,
                        tag,
                        true,
                        &player.visual,
                        client.actor_cycle,
                        false,
                        None,
                        &self.palette,
                        crate::model_order::Mode::Priority,
)?;
                    if let Some((body, range)) = bodies.get(&index) {
                        let attachment = crate::model_bounds::Bounds::new(&model.vertices, [x,height,z]).with_selection(&model.vertices, player.turn.angle, true);
                        if let Some(combined) = body.union(attachment) {
                            let (body_faces, attachment_faces) = out.split_at_mut(attachment_start);
                            for face in body_faces[range.clone()].iter_mut().chain(attachment_faces) {
                                face.model_bounds = Some(combined);
                                if let Some(order)=&mut face.model_order {order.batch=range.start;}
                            }
                            // The combined body already publishes these bounds.
                            // An empty/invisible attachment needs no second marker.
                            if !range.is_empty() && out[attachment_start..].iter()
                                .all(|t| t.local_vertices.is_none() && t.alpha == 255) {
                                out.truncate(attachment_start);
                            }
                        }
                    }
                }
            }
        }

        if world.is_some() {
            for &(index, player) in &players {
                // Scene insertion precedes model loading: a missing body does
                // not suppress independently available attached effects.
                let x = player.path.position[0] - scene.base_x * 128;
                let z = player.path.position[1] - scene.base_y * 128;
                if let Some(h) = scene.elevation(plane as usize, x, z) {
                    for slot in player.spots.ordered_slots() {
                        let effect=&player.spots.slots[&slot];
                        if client.actor_cycle < effect.start {
                            continue;
                        }
                        let definition = client
                            .spots
                            .get(&(effect.id as u32))
                            .ok_or(Error::Invalid("child player spot definition"))?;
                        let spot_start=out.len();
                self.place_spot(
                            &mut client.scripts.source.cache,
                            definition,
                            effect,
                            [x, h, z],
                            player.turn.angle,
                            0x2000000000000000 | index as u64,
                            &mut out,
                            Some(&player.visual),
                            client.actor_cycle,
                    None,
                        )?;
                mark_spot_slot(&mut out[spot_start..],Some(slot));
                    }
                }
            }
        }
        self.shift_actor_heights(client,world,&mut out);
        Ok(out)
    }
    pub fn triangles(
        &mut self,
        client: &mut ClientState,
        scene: &Scene,
        item_support: &HashMap<[u8; 3], i32>,
    ) -> Result<(Vec<Triangle>, Vec<String>)> {
        self.visible_npcs.clear();
        self.heights.clear();
        let target = self
            .target_varp
            .map(|id| client.scripts.variables.player(id))
            .filter(|&v| v != -1)
            .map(|v| (v & 2047) as u16);
        let mut out = Vec::new();
        let mut pending = Vec::new();
        let mut arrow_slots = scene.temporary_slots();
        let mut arrow_input=0;
        for (ordinal, placement) in client.steering.scene.into_iter().enumerate() {
            let Some(p) = placement else { continue; };
            let Some((model, colors)) = &self.steering_models[p.active as usize] else { continue; };
            if !reserve_effect(&mut arrow_slots, client.data.position[0] as usize, [p.position[0], p.position[2]]) { continue; }
            let start = out.len();
            place(&mut out, model, colors, p.position, p.angle, 0, false)?;
            for triangle in &mut out[start..] {
                let order=triangle.model_order.as_mut().unwrap();
                order.source_ordinal=Some(ordinal);order.temporary_id=Some(arrow_input);
            }
            arrow_input+=1;
        }
        crate::model_order::append(&mut out, self.player_triangles(client, scene, None)?);
        let (npc_triangles, npc_pending) = self.npc_triangles(client, scene, None)?;
        crate::model_order::append(&mut out, npc_triangles);
        pending.extend(npc_pending);
        let (accepted,mut temporary_slots,_,objects)=submission_state_with_objects(client,scene,None,target)?;
        let mut temporary_id=objects.len();
        let mut projectiles = client
            .projectiles
            .iter()
            .filter(|p| p.motion.moving && p.request.source[0] == client.data.position[0])
            .map(|p| {
                (
                    p.request.id,
                    &p.clock,
                    [
                        p.motion.position[0],
                        p.motion.position[2],
                        p.motion.position[1],
                    ],
                    p.motion.yaw,
                    p.motion.pitch,
                )
            })
            .collect::<Vec<_>>();
        if client.scripted_projectiles.definition.source.tile[0] == client.data.position[0] {
            for key in client.scripted_projectiles.ordered_keys() {
                let p = &client.scripted_projectiles.effects[&key];
                if p.visible {
                    projectiles.push((
                        client.scripted_projectiles.definition.spot,
                        &p.clock,
                        [
                            p.motion.position[0] as i32 as f64 + scene.base_x as f64 * 128.0,
                            p.motion.position[1] as f64,
                            p.motion.position[2] as i32 as f64 + scene.base_y as f64 * 128.0,
                        ],
                        p.motion.yaw,
                        p.motion.pitch,
                    ));
                }
            }
        }
        for (id, clock, position, yaw, pitch) in projectiles {
            let fine=[position[0] as i32-scene.base_x*128,position[2] as i32-scene.base_y*128];
            if !reserve_effect(&mut temporary_slots,client.data.position[0] as usize,fine){continue;}
            let effect_id=temporary_id;temporary_id+=1;
            let d = client
                .spots
                .get(&(id as u32))
                .ok_or(Error::Invalid("render projectile definition"))?;
            if !self.spots.contains_key(&id) {
                let Some(model) = d.base_model(&mut client.scripts.source.cache, &mut self.models)? else {
                    continue;
                };
                let colors = model.light(64 + d.ambient, 850 + d.contrast, [-30, -50, -30])?;
                self.spots.insert(id, (model, colors));
            }
            let (base, colors) = &self.spots[&id];
            let mut model = base.clone();
            if d.sequence >= 0 {
                self.animations.apply(
                    &mut client.scripts.source.cache,
                    &mut model,
                    d.sequence as u32,
                    clock,
                )?;
            }
            model.scale([d.scale[0], d.scale[1], d.scale[0]]);
            if matches!(d.rotation, 90 | 180 | 270) {
                model.rotate_quarter((d.rotation / 90) as u8);
            }
            let radians = pitch as f64 * 0.0030679615;
            let sin = (radians.sin() * 65536.0) as i32;
            let cos = (radians.cos() * 65536.0) as i32;
            for v in &mut model.vertices {
                let y = v[1] as i32;
                let z = v[2] as i32;
                v[1] = (y.wrapping_mul(cos).wrapping_sub(z.wrapping_mul(sin)) >> 16) as f32;
                v[2] = (y.wrapping_mul(sin).wrapping_add(z.wrapping_mul(cos)) >> 16) as f32;
            }
            let p = position;
            let effect_start=out.len();
            place(
                &mut out,
                &model,
                colors,
                [
                    p[0] as i32 - scene.base_x * 128,
                    p[1] as i32,
                    p[2] as i32 - scene.base_y * 128,
                ],
                yaw,
                0,
                false,
            )?;
            mark_temporary(&mut out[effect_start..],Some(effect_id));
        }
        let mut effects = Vec::new();
        if let Some(state) = &client.player_state {
            for index in accepted.iter().filter_map(|key|(!key.npc).then_some(key.index)) {
                if index == state.local && !client.preferences.render_self && Some(index) != target
                {
                    continue;
                }
                if let Some(p) = state.root_actor(index) {
                    if p.position[0] == client.data.position[0]
                        && p.appearance.as_ref().is_some_and(|a| !a.hidden)
                    {
                        for slot in p.spots.ordered_slots() {
                            let effect=&p.spots.slots[&slot];
                            effects.push((
                                p.path.position,
                                p.turn.angle,
                                0x2000000000000000 | index as u64,
                                effect,
                                None,
                                Some(&p.visual),
                                Some(slot),
                                None,
                            ));
                        }
                    }
                }
            }
        }
        for effect in &mut client.world_spots {
            if effect.position[0] == client.data.position[0] && client.actor_cycle.wrapping_sub(1)>=effect.animation.start {
                let fine=[(effect.position[1]-scene.base_x)*128+64,(effect.position[2]-scene.base_y)*128+64];
                if !reserve_effect(&mut temporary_slots,client.data.position[0] as usize,fine){continue;}
            let effect_id=temporary_id;temporary_id+=1;
                if effect.elevation.is_none() {
                    effect.elevation = scene.elevation(
                        effect.position[0] as usize,
                        (effect.position[1] - scene.base_x) * 128 + 64,
                        (effect.position[2] - scene.base_y) * 128 + 64,
                    );
                }
                effects.push((
                    [effect.position[1] * 128 + 64, effect.position[2] * 128 + 64],
                    0,
                    0,
                    &effect.animation,
                    effect.elevation,
                    None,
                    None,
                    Some(effect_id),
                ));
            }
        }
        for (fine, angle, tag, effect, elevation, visual, slot, effect_id) in effects {
            if client.actor_cycle < effect.start {
                continue;
            }
            let d = client
                .spots
                .get(&(effect.id as u32))
                .ok_or(Error::Invalid("render spot definition"))?;
            let x = fine[0] - scene.base_x * 128;
            let z = fine[1] - scene.base_y * 128;
            if let Some(h) =
                elevation.or_else(|| scene.elevation(client.data.position[0] as usize, x, z))
            {
                let spot_start=out.len();
                self.place_spot(
                    &mut client.scripts.source.cache,
                    d,
                    effect,
                    [x, h, z],
                    angle,
                    tag,
                    &mut out,
                    visual,
                    client.actor_cycle,
                    None,
                )?;
                mark_spot_slot(&mut out[spot_start..],slot);
                mark_temporary(&mut out[spot_start..],effect_id);
            }
        }
        let (ground, ground_pending) = self.ground_triangles(client, scene, item_support, None)?;
        crate::model_order::append(&mut out, ground);
        pending.extend(ground_pending);
        self.shift_actor_heights(client,None,&mut out);
        Ok((out, pending))
    }
    /// Apply scene object changes in a child view before drawing its mesh.
    pub fn child_locations(
        &mut self,
        client: &mut ClientState,
        scene: &mut Scene,
        mesh: &mut crate::world_mesh::WorldMesh,
        id: u16,
        ticks: u32,
        definitions: &HashMap<u32, crate::object::Definition>,
    ) -> Result<()> {
        let entity = client
            .world_entities
            .get_mut(&0)
            .and_then(|state| state.entities.get_mut(&id))
            .ok_or(Error::Invalid("location world missing"))?;
        let mut animated = false;
        while let Some(event) = entity.zone.locations.pop_front() {
            use crate::location_update::Event;
            match event {
                Event::Change(origin, change) => entity
                    .locations
                    .apply(origin, change, |key| scene.location(key))?,
                Event::Clear(origin) => entity.locations.clear(origin),
                Event::Animate(origin, request) => {
                    let key = request.key(origin)?;
                    let applied = scene.animate_location(key, request, definitions)?;
                    animated |= applied;
                    if let Some(pending) = entity.locations.entries.get_mut(&key) {
                        let forced = definitions
                            .get(&(pending.replacement.id as u32))
                            .is_some_and(|d| d.flag90);
                        if forced || !applied {
                            pending.animation = request.sequence;
                        }
                    }
                }
                Event::Merge(origin, request) => {
                    let Some(player) = client
                        .player_state
                        .as_mut()
                        .and_then(|s| s.referenced_view_mut(id, entity.spawn_generation))
                        .and_then(|v| v.players.get_mut(&request.player))
                    else {
                        continue;
                    };
                    let def = definitions
                        .get(&(request.object as u32))
                        .ok_or(Error::Invalid("location merge definition"))?;
                    let Some(model) = self.location_models.object(
                        &mut client.scripts.source.cache,
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
                    entity
                        .locations
                        .merge(key, request.start, request.end, |key| scene.location(key));
                    // The event is received at this cycle, after any elapsed cycles.
                    // Earlier ticks in this batch must not consume its lifetime.
                    let pending = entity.locations.entries.get_mut(&key).unwrap();
                    pending.delay = request.start as i32 + ticks as i32;
                    pending.lifetime = request.end as i32 + ticks as i32;
                    player.attachment = Some(attachment);
                }
            }
        }
        let changed = entity.locations.advance(
            [scene.base_x, scene.base_y],
            ticks,
            |key, object, animation| {
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
        if changed > 0
            || animated
            || mesh.transforms_changed(client.scripts.variables.player_values())
        {
            let mut next = crate::world_mesh::WorldMesh::build_with_varps(
                scene,
                &mut client.scripts.source.cache,
                entity.plane as usize,
                client.scripts.variables.player_values(),
            )?;
            next.retain_clocks(mesh, &self.animations);
            *mesh = next;
        }
        mesh.advance(
            scene,
            &mut client.scripts.source.cache,
            &mut self.animations,
            ticks,
        )
    }

    pub fn ground_triangles(
        &mut self,
        client: &mut ClientState,
        scene: &Scene,
        item_support: &HashMap<[u8; 3], i32>,
        world: Option<u16>,
    ) -> Result<(Vec<Triangle>, Vec<String>)> {
        let zone = if let Some(id) = world {
            let entity = client
                .world_entities
                .get(&0)
                .and_then(|state| state.entities.get(&id))
                .ok_or(Error::Invalid("ground item world missing"))?;
            &entity.zone
        } else {
            &client.zone
        };
        let mut out = Vec::new();
        let mut pending = Vec::new();
        for (position, pile) in &zone.piles {
            let Some(tx)=position[1].checked_sub(scene.base_x) else {continue;};
            let Some(tz)=position[2].checked_sub(scene.base_y) else {continue;};
            if !(0..4).contains(&position[0]) || tx<0 || tz<0 || tx as usize>=scene.width || tz as usize>=scene.height {continue;}
            let x = tx * 128 + 64;
            let z = tz * 128 + 64;
            let Some(h) = scene.elevation(position[0] as usize, x, z) else {
                continue;
            };
            let mut ids = Vec::new();
            for item in pile {
                if ids.contains(&item.id) {
                    continue;
                }
                ids.push(item.id);
                if ids.len() > 3 {
                    break;
                }
                let Some(def) = client.items.items.get(&(item.id as u32)) else {
                    pending.push(format!("ground item {} definition", item.id));
                    continue;
                };
                let quantity = if item.custom.is_some() {
                    1
                } else {
                    item.quantity
                };
                let variant = def.count_variant(quantity);
                let key = (variant, item.custom.clone());
                if !self.ground.contains_key(&key) {
                    let custom =
                        item.custom
                            .as_ref()
                            .map(|c| crate::item_model::AppearanceOverride {
                                inventory_model: Some(c.model),
                                recolor: Some(c.recolor.clone()),
                                retexture: Some(c.retexture.clone()),
                                ..Default::default()
                            });
                    match self.models.inventory(
                        &mut client.scripts.source.cache,
                        &client.items,
                        item.id as u32,
                        quantity,
                        custom.as_ref(),
                    ) {
                        Ok(model) => {
                            let def = &client.items.items[&variant];
                            let colors = model.light(
                                64 + def.ambient,
                                768 + def.contrast,
                                [-50, -10, -50],
                            )?;
                            if self.ground.len() >= 256 {
                                self.ground.clear();
                            }
                            if std::env::var_os("OSRS_GROUND_DIAGNOSTIC").is_some() {
                                eprintln!("Ground mesh {} at {:?}, scene base [{}, {}], height {h}, vertices {}, faces {}, bounds {:?}", item.id, position, scene.base_x, scene.base_y, model.vertices.len(), model.faces.len(), (0..3).map(|axis| model.vertices.iter().fold([f32::INFINITY,f32::NEG_INFINITY], |b,v| [b[0].min(v[axis]),b[1].max(v[axis])])).collect::<Vec<_>>());
                            }
                            self.ground.insert(key.clone(), (model, colors));
                        }
                        Err(e) => {
                            pending.push(format!("ground item {} mesh: {e}", item.id));
                            continue;
                        }
                    }
                }
                let (model, colors) = &self.ground[&key];
                let tag = crate::item_submission::tag(item.id,position[0] as usize,[tx as usize,tz as usize]);
                let mut tilted = model.clone();
                let (tx, ty) = ((x >> 7) as usize, (z >> 7) as usize);
                let mut plane = position[0] as usize;
                if plane < 3 && scene.tile(1, tx, ty).is_some_and(|t| t.flags & 2 != 0) {
                    plane += 1;
                }
                let heights = [
                    scene.height(plane, tx, ty),
                    scene.height(plane, tx + 1, ty),
                    scene.height(plane, tx + 1, ty + 1),
                    scene.height(plane, tx, ty + 1),
                ];
                let support = item_support
                    .get(&[position[0] as u8, tx as u8, ty as u8])
                    .copied()
                    .unwrap_or(0);
                if support == 0 {
                    if let [Some(a), Some(b), Some(c), Some(d)] = heights {
                        crate::ground_model::align(
                            &mut tilted.vertices,
                            crate::ground_model::normal([a, b, c, d]),
                        );
                    }
                }
                place(&mut out, &tilted, colors, [x, h - support, z], 0, tag, true)?;
            }
        }
        Ok((out, pending))
    }
}
fn place(
    out: &mut Vec<Triangle>,
    model: &Model,
    colors: &[[i32; 3]],
    position: [i32; 3],
    angle: u16,
    tag: u64,
    single_tile: bool,
) -> Result<()> {
    // Ground-pile slope alignment has already run, but its output is still
    // quantized by GameModel.by just like animated actor vertices.
    let start = out.len();
    let vertices = crate::model_draw::vertices(&model.vertices, angle, position)?;
    let bounds = crate::model_bounds::Bounds::new(&model.vertices, position).with_selection(&model.vertices, angle, single_tile);
    for (i, face) in model.faces.iter().enumerate() {
        if colors[i][2] == -2 {
            continue;
        }
        let color = if colors[i][2] == -1 {
            [colors[i][0]; 3]
        } else {
            colors[i]
        };
        let positions = face.map(|v| vertices[v as usize]);
        let texture = model
            .face_textures
            .as_ref()
            .and_then(|t| (t[i] != -1).then_some(t[i] as u16));
        let map = model
            .texture_coords
            .as_ref()
            .and_then(|t| {
                if t[i] == -1 {
                    None
                } else {
                    model.texture_triangles.get(t[i] as u8 as usize)
                }
            })
            .unwrap_or(face);
        out.push(Triangle {
            #[cfg(feature="enhanced-renderer")] material_hsl:Some(model.colors[i]), texture_mapping: Some(crate::render3d::TextureMapping { vertices: map.map(|v| vertices[v as usize]), local: Some(map.map(|v| model.vertices[v as usize])) }),
            local_vertices: Some(face.map(|v| model.vertices[v as usize])),
            model_bounds: Some(bounds),
                model_order: Some(crate::model_order::Submission::new(model,i,start)), depth_bias: model.face_depth_bias(i),
            view: 0,
            vertices: positions,
            colors: color,
                        flat: colors[i][2] == -1,
            uv: texture_uv(map.map(|v| vertices[v as usize]), positions),
            texture,
            alpha: model.face_alpha(i),
            near_alpha: Some(model.alpha.as_ref().map_or(0, |a| a[i])),
            tag,
        });
    }
    if out.len() == start && single_tile && tag != 0 {
        let mut marker = Triangle::selection_only(bounds, tag);
        marker.model_order = Some(crate::model_order::Submission {
            actor_spot: None, source_ordinal: None, temporary_id: None, scene_height_offset: 0,
            mode: crate::model_order::Mode::Auto, priority: model.priority,
            has_priorities: false, has_alpha: model.alpha.is_some(), batch: start,
        });
        out.push(marker);
    }
    Ok(())
}

/// The override applies only to original body faces, before attached models.
fn place_actor(
    out: &mut Vec<Triangle>,
    model: &Model,
    colors: &[[i32; 3]],
    position: [i32; 3],
    angle: u16,
    tag: u64,
    single_tile: bool,
    visual: &crate::actor_visual::State,
    cycle: i32,
    body: bool,
    overlap: Option<u16>,
    palette: &[u32],
    mode: crate::model_order::Mode,
) -> Result<()> {
    let collapsed = overlap.map(|_| {
        let mut m = model.clone();
        crate::world_overlap::collapse_npc_model(&mut m);
        m
    });
    let model = collapsed.as_ref().unwrap_or(model);
    let tint = overlap
        .map(|color| crate::actor_visual::Tint {
            hsl: [
                (color >> 10 & 63) as u8,
                (color >> 7 & 7) as u8,
                (color & 127) as u8,
            ],
            weight: 127,
            ..Default::default()
        })
        .unwrap_or(visual.tint);
    let start = out.len();
    let tinted;
    let colors = if body && (overlap.is_some() || tint.active(cycle)) {
        // Original flat faces feed the current RGB palette entry back through
        // the packed-HSL override; brightness therefore affects this path too.
        tinted = colors
            .iter()
            .enumerate()
            .map(|(i, c)| {
                if i as i32 >= model.faces.len() as i16 as i32
                    || c[2] == -2
                    || model.face_textures.as_ref().is_some_and(|t| t[i] != -1)
                {
                    return *c;
                }
                if c[2] == -1 {
                    [tint.color(palette[c[0] as usize] as i32), c[1], -1]
                } else {
                    c.map(|value| tint.color(value))
                }
            })
            .collect::<Vec<_>>();
        &tinted
    } else {
        colors
    };
    place(out, model, colors, position, angle, tag, single_tile)?;
    #[cfg(feature="enhanced-renderer")]
    if body && (overlap.is_some() || tint.active(cycle)) {
        let limit=model.faces.len() as i16 as i32;
        let admitted=model.faces.iter().enumerate().filter(|(i,_)|colors[*i][2]!=-2);
        for (triangle,(face,_)) in out[start..].iter_mut().zip(admitted) {
            if (face as i32)<limit && triangle.texture.is_none() {
                triangle.material_hsl=triangle.material_hsl.map(|c|tint.color(c as i32) as u16);
            }
        }
    }
    for t in &mut out[start..] {if let Some(order)=&mut t.model_order {order.mode=mode;}}
    apply_alpha(out, start, visual.alpha(cycle));
    Ok(())
}

fn apply_alpha(out: &mut Vec<Triangle>, start: usize, alpha: u8) {
    if alpha == 255 {
        if let Some(first) = out.get(start).copied().filter(|t| t.model_bounds
            .and_then(|b| b.selection).is_some_and(|s| s.single_tile)) {
            out.truncate(start);
            let mut marker = Triangle::selection_only(first.model_bounds.unwrap(), first.tag);
            marker.model_order = first.model_order;
            out.push(marker);
        } else {
            // Non-single-tile models still need face regions for original picking.
            for triangle in &mut out[start..] { triangle.alpha = 255; }
        }
    } else {
        for triangle in &mut out[start..] {
            // Selection markers are not encoded model face alpha bytes.
            if triangle.local_vertices.is_none() { continue; }
            triangle.alpha = crate::actor_visual::face_alpha(triangle.alpha, alpha);
        }
    }
}

#[cfg(test)]
mod selection_tests {
    #[test]
    #[cfg(feature="enhanced-renderer")]
    fn actor_material_tint_tracks_visible_untextured_faces_only() {
        use super::*;
        let mut model=Model::decode(&[0;18]).unwrap();
        model.vertices=vec![[-30.,-20.,0.],[0.,30.,0.],[30.,-20.,0.]];
        model.faces=vec![[0,1,2];3]; model.colors=vec![65400,32040,12000];
        model.face_textures=Some(vec![-1,-1,1]);
        let colors=vec![[10,20,-2],[32000,32020,32040],[64;3]];
        let palette=crate::floor::palette(0.8).unwrap();
        let mut visual=crate::actor_visual::State::default();
        visual.tint=crate::actor_visual::Tint{start:0,end:10,hsl:[19,4,32],weight:127};
        let mut faces=Vec::new();
        place_actor(&mut faces,&model,&colors,[0,0,300],0,1,false,&visual,1,true,None,&palette,crate::model_order::Mode::Auto).unwrap();
        assert_eq!(faces.len(),2);
        assert_eq!(faces[0].material_hsl,Some(visual.tint.color(32040) as u16));
        assert_eq!(faces[1].material_hsl,Some(12000));
        let mut untinted=Vec::new();
        place_actor(&mut untinted,&model,&colors,[0,0,300],0,1,false,&visual,11,true,None,&palette,crate::model_order::Mode::Auto).unwrap();
        assert_eq!(untinted[0].material_hsl,Some(32040));
        assert_eq!(untinted[0].colors,colors[1]);
    }
    #[test]
    fn selection_only_empty_model_survives_actor_alpha_schedule() {
        use super::*;
        use crate::{render3d::{Camera, Frame}, viewport_steering::Ray};
        let model = Model::decode(&[0; 18]).unwrap();
        let mut faces = Vec::new();
        place(&mut faces, &model, &[], [0, 0, 300], 0, 1, true).unwrap();
        for alpha in [0, 128, 255, 0] {
            apply_alpha(&mut faces, 0, alpha);
            assert_eq!(faces.len(), 1);
            assert_eq!(faces[0].alpha, 255);
            assert_eq!(faces[0].model_order.unwrap().batch, 0);
            let mut frame = Frame::new(200, 200).unwrap();
            let camera = Camera::angles([0; 3], 0, 0, 200.).unwrap();
            frame.draw(&camera, &faces, &[], &Default::default());
            assert!(frame.tags.iter().all(|&tag| tag == 0));
            let ray = Ray { far: 3500, size: [200; 2], focal: 200, angles: [0; 2], eye: [0; 3], focus: [0; 3] };
            // Original empty ct model: near slab at260 plus32 selection padding.
            assert_eq!(frame.model_hits([100; 2], ray), [(1, 292)]);
        }
    }
}

/// Register root temporary renderables after the caller assigns global batch IDs.
/// Ground piles are static scene submissions and retain their existing slots.
pub fn register_root_order(frame:&mut crate::render3d::Frame,camera:&crate::render3d::Camera,client:&ClientState,scene:&Scene,triangles:&[Triangle]) {
    register_view_order(frame,camera,client,scene,triangles.iter(),0);
}
/// Child renderables retain local bounds and use their inverse world camera.
pub fn register_view_order<'a>(frame:&mut crate::render3d::Frame,camera:&crate::render3d::Camera,client:&ClientState,scene:&Scene,triangles:impl IntoIterator<Item=&'a Triangle>,view:u16) {
    use crate::scene_footprint::Footprint;
    let dimensions=[scene.width as i32,scene.height as i32];
    if dimensions.iter().any(|&n|n<=0){return;}
    let eye=if view==0 {
        let eye=[camera.position[0] as i32,camera.position[2] as i32];
        std::array::from_fn(|i|eye[i].clamp(0,dimensions[i]*128-1))
    }else {let Some(eye)=frame.child_eye(view) else{return;};[eye[0],eye[2]]};
    let npcs=if view==0 {Some(&client.npc_state)}else{client.world_entities.get(&0).and_then(|w|w.entities.get(&view)).map(|e|&e.npcs)};
    // Ordering only: retain current visibility until full opaque/alpha zone traversal.
    let ranks=crate::scene_zone::ordered(dimensions,eye,0)
        .into_iter().enumerate().map(|(rank,zone)|(zone,rank)).collect::<HashMap<_,_>>();
    let mut seen=std::collections::HashSet::new();
    for triangle in triangles {
        if triangle.view!=view{continue;}
        let (Some(order),Some(bounds))=(triangle.model_order,triangle.model_bounds) else{continue;};
        if !seen.insert((triangle.tag,order.batch)){continue;}
        let fine=[bounds.position[0],bounds.position[2]];
        let (footprint,local)=if order.actor_spot.is_some() {
            (Footprint::spot(fine),false)
        }else{match triangle.tag>>60 {
            2=>{
                let Some(state)=client.player_state.as_ref() else{continue;};
                let index=triangle.tag as u16;
                let Some(player)=state.root_owned_actor(view,index,&client.world_entities) else{continue;};
                let footprint=if let Some(attachment)=player.attachment.as_ref().filter(|a|a.placement.active(client.actor_cycle)) {
                    let [x,y,right,bottom]=attachment.placement.bounds;
                    Footprint{origin:[x-scene.base_x,y-scene.base_y],size:[right-x+1,bottom-y+1]}
                }else{Footprint::actor(fine,60,player.turn.angle as i32,player.sequence.stretch)};
                (footprint,state.local_view==view&&state.local==index)
            },
            4=>{
                let Some(npc)=npcs.and_then(|state|state.npcs.get(&(triangle.tag as u16))) else{continue;};
                let Some(definition)=client.npcs.get(&(npc.id as u32)) else{continue;};
                (Footprint::actor(fine,definition.size as i32*64-4,npc.turn.angle as i32,npc.sequence.stretch),false)
            },
            0 if triangle.tag==0=>(Footprint::actor(fine,60,bounds.angle as i32,false),false),
            _=>continue,
        }};
        if let Some(&rank)=ranks.get(&footprint.zone(0)) {frame.temporary_model(view,triangle.tag,order.batch,rank,local);}
    }
}

/// Per-view actor insertion eligibility before model loading. The original stamps
/// centered tiles even if later model loading or scene-capacity insertion fails.
/// Root early child scenes reserve slots after priority players and before NPCs.
pub fn submission_plan(client:&mut ClientState,scene:&Scene,world:Option<u16>,target:Option<u16>)->Result<Vec<crate::actor_submission::Key>> {
    Ok(submission_state(client,scene,world,target)?.0)
}
fn submission_state(client:&mut ClientState,scene:&Scene,world:Option<u16>,target:Option<u16>)->Result<(Vec<crate::actor_submission::Key>,crate::scene_slots::Slots,Vec<u16>)> {
    let (actors,slots,children,_)=submission_state_with_objects(client,scene,world,target)?;
    Ok((actors,slots,children))
}
type TemporaryObjects=Vec<(usize,crate::object_traversal::Object,Option<u64>,Option<u16>)>;
fn submission_state_with_objects(client:&mut ClientState,scene:&Scene,world:Option<u16>,target:Option<u16>)->Result<(Vec<crate::actor_submission::Key>,crate::scene_slots::Slots,Vec<u16>,TemporaryObjects)> {
    use crate::{actor_submission::{self,Actor,Key},script_runtime::Source};
    let view=world.unwrap_or(0);
    let (npcs,plane)=if let Some(id)=world {
        let entity=client.world_entities.get(&0).and_then(|w|w.entities.get(&id)).ok_or(Error::Invalid("actor submission view missing"))?;
        (&entity.npcs,entity.plane as i32)
    }else{(&client.npc_state,client.data.position[0])};
    let base=[scene.base_x.wrapping_mul(128),scene.base_y.wrapping_mul(128)];
    let mut eligible=HashMap::new();let mut footprints=HashMap::new();let mut npc_order=Vec::new();
    for &index in &npcs.indices {
        let Some(npc)=npcs.npcs.get(&index) else{continue;};
        let definition=client.npcs.get(&(npc.id as u32)).ok_or(Error::Invalid("NPC definition missing"))?;
        npc_order.push((index,definition.priority));
        if definition.transforms.is_some() {
            let bit=if let Some(id)=definition.transform_varbit {let bit=client.scripts.source.varbit(id as i32)?;bit.get(client.scripts.variables.player(bit.base))}else{-1};
            let varp=if definition.transform_varbit.is_none(){definition.transform_varp.map_or(-1,|id|client.scripts.variables.player(id))}else{-1};
            if definition.transformed_id(bit,varp).is_none(){continue;}
        }
        eligible.insert(Key{npc:true,index},Actor{fine:std::array::from_fn(|i|npc.path.position[i].wrapping_sub(base[i])),center_dedup:definition.size==1});
        footprints.insert(Key{npc:true,index},crate::scene_footprint::Footprint::actor(std::array::from_fn(|i|npc.path.position[i].wrapping_sub(base[i])),definition.size as i32*64-4,npc.turn.angle as i32,npc.sequence.stretch));
    }
    let (high,local)=if let Some(state)=&client.player_state {
        let entries=if view==0 {state.root_iter().collect::<Vec<_>>()}else{state.root_owned_view(view,&client.world_entities).map_or_else(Vec::new,|v|v.players.iter().map(|(&i,p)|(i,p)).collect())};
        for (index,player) in entries {
            if player.position[0]!=plane||!player.appearance.as_ref().is_some_and(|a|!a.hidden){continue;}
            eligible.insert(Key{npc:false,index},Actor{fine:std::array::from_fn(|i|player.path.position[i].wrapping_sub(base[i])),center_dedup:!player.attachment.as_ref().is_some_and(|a|a.placement.active(client.actor_cycle))});
            let footprint=if let Some(a)=player.attachment.as_ref().filter(|a|a.placement.active(client.actor_cycle)) {
                let [x,y,right,bottom]=a.placement.bounds;
                crate::scene_footprint::Footprint{origin:[x-scene.base_x,y-scene.base_y],size:[right-x+1,bottom-y+1]}
            }else{crate::scene_footprint::Footprint::actor(std::array::from_fn(|i|player.path.position[i].wrapping_sub(base[i])),60,player.turn.angle as i32,player.sequence.stretch)};
            footprints.insert(Key{npc:false,index},footprint);
        }
        (state.high.as_slice(),state.local)
    }else{(&[][..],0)};
    let order=actor_submission::ordered(high,local,target,client.preferences.render_self,&npc_order);
    let retained=actor_submission::retained([scene.width as i32,scene.height as i32],&order,|key|eligible.get(&key).copied());
    let mut slots=scene.temporary_slots();let mut objects=Vec::new();
    if view==0 { for p in client.steering.scene.into_iter().flatten() {
        reserve_temporary(&mut slots,&mut objects,plane as usize,[p.position[0],p.position[2]]);
    }}
    let children=if view==0 {root_children(client)}else{Vec::new()};
    let mut accepted_children=Vec::new();let mut early_done=false;let mut accepted=Vec::new();
    let reserve_children=|slots:&mut crate::scene_slots::Slots,accepted:&mut Vec<u16>,objects:&mut TemporaryObjects| {
        for &(id,fine,late) in &children {if !late&&reserve_temporary(slots,objects,plane as usize,fine){objects.last_mut().unwrap().3=Some(id);accepted.push(id);}}
    };
    for key in retained {
        let priority_player=!key.npc&&(key.index==local||Some(key.index)==target);
        if !early_done&&!priority_player {reserve_children(&mut slots,&mut accepted_children,&mut objects);early_done=true;}
        let footprint=footprints[&key];
        if footprint.origin.iter().any(|&v|v<0)||footprint.size.iter().any(|&v|v<0){continue;}
        if slots.insert_object(plane as usize,footprint.origin[0] as usize,footprint.origin[1] as usize,footprint.size.map(|v|v as usize),true){accepted.push(key);objects.push((plane as usize,traversal_object(footprint,eligible[&key].fine),Some((if key.npc {0x4000000000000000}else{0x2000000000000000})|key.index as u64),None));}
    }
    if !early_done {reserve_children(&mut slots,&mut accepted_children,&mut objects);}
    Ok((accepted,slots,accepted_children,objects))
}
fn root_children(client:&ClientState)->Vec<(u16,[i32;2],bool)> {
    let local=client.player_state.as_ref().map_or(0,|p|p.local_view);
    client.world_entities.get(&0).map_or_else(Vec::new,|state|{
        crate::world_overlap::submission_order(state,local,client.world_entity_limit.get()).into_iter().map(|id|{
            let e=&state.entities[&id];(id,[e.pose.x,e.pose.z],id!=local&&e.owner_type==1)
        }).collect()
    })
}
fn root_child_submissions(client:&mut ClientState,scene:&Scene,target:Option<u16>)->Result<std::collections::HashSet<u16>> {
    if root_children(client).is_empty(){return Ok(std::collections::HashSet::new());}
    Ok(temporary_submission_state(client,scene,None,target)?.0)
}
fn temporary_submission_state(client:&mut ClientState,scene:&Scene,world:Option<u16>,target:Option<u16>)->Result<(std::collections::HashSet<u16>,TemporaryObjects)> {
    let (_,mut slots,early,mut objects)=submission_state_with_objects(client,scene,world,target)?;
    let mut accepted=early.into_iter().collect::<std::collections::HashSet<_>>();
    if let Some(id)=world {
        let entity=client.world_entities.get(&0).and_then(|w|w.entities.get(&id)).ok_or(Error::Invalid("temporary submission view missing"))?;
        for effect in &entity.world_spots {
            if effect.position[0]==entity.plane as i32 && client.actor_cycle.wrapping_sub(1)>=effect.animation.start {
                reserve_temporary(&mut slots,&mut objects,entity.plane as usize,
                    [(effect.position[1]-scene.base_x)*128+64,(effect.position[2]-scene.base_y)*128+64]);
            }
        }
        return Ok((accepted,objects));
    }
    let children=root_children(client);
    let plane=client.data.position[0];
    // Late child scenes follow ordinary and scripted projectiles, then world graphics.
    for p in &client.projectiles {if p.motion.moving&&p.request.source[0]==plane {
        reserve_temporary(&mut slots,&mut objects,plane as usize,[p.motion.position[0] as i32-scene.base_x*128,p.motion.position[1] as i32-scene.base_y*128]);
    }}
    if client.scripted_projectiles.definition.source.tile[0]==plane {
        for key in client.scripted_projectiles.ordered_keys(){let p=&client.scripted_projectiles.effects[&key];if p.visible {
            reserve_temporary(&mut slots,&mut objects,plane as usize,[p.motion.position[0] as i32,p.motion.position[2] as i32]);
        }}
    }
    for p in &client.world_spots {if p.position[0]==plane&&client.actor_cycle.wrapping_sub(1)>=p.animation.start {
        reserve_temporary(&mut slots,&mut objects,plane as usize,[(p.position[1]-scene.base_x)*128+64,(p.position[2]-scene.base_y)*128+64]);
    }}
    for (id,fine,late) in children{if late&&reserve_temporary(&mut slots,&mut objects,plane as usize,fine){objects.last_mut().unwrap().3=Some(id);accepted.insert(id);}}
    Ok((accepted,objects))
}
fn traversal_object(footprint:crate::scene_footprint::Footprint,fine:[i32;2])->crate::object_traversal::Object {
    crate::object_traversal::Object {start:footprint.origin,
        end:std::array::from_fn(|axis|footprint.origin[axis]+footprint.size[axis]-1),center:fine}
}
fn reserve_temporary(slots:&mut crate::scene_slots::Slots,objects:&mut TemporaryObjects,plane:usize,fine:[i32;2])->bool {
    if !reserve_effect(slots,plane,fine){return false;}
    objects.push((plane,traversal_object(crate::scene_footprint::Footprint::actor(fine,60,0,false),fine),None,None));
    true
}

fn reserve_effect(slots:&mut crate::scene_slots::Slots,plane:usize,fine:[i32;2])->bool {
    let footprint=crate::scene_footprint::Footprint::actor(fine,60,0,false);
    slots.insert_object(plane,footprint.origin[0] as usize,footprint.origin[1] as usize,footprint.size.map(|n|n as usize),true)
}

fn mark_spot_slot(faces:&mut [Triangle],slot:Option<u8>) {
    if let Some(slot)=slot {for face in faces {if let Some(order)=&mut face.model_order {
        order.source_ordinal=Some(slot as usize);
    }}}
}

fn mark_temporary(faces:&mut [Triangle],id:Option<usize>) {
    for face in faces {if let Some(order)=&mut face.model_order {order.temporary_id=id;}}
}
