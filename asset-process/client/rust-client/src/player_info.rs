//! Four-pass player visibility and movement synchronization.
use crate::{
    buffer::{BitReader, Reader},
    huffman::Huffman,
    item::Definitions,
    player_appearance,
    player_update::{self, Update},
    Error, Result,
};
#[derive(Clone, Debug)]
pub struct Player {
    /// Composition ar: local to this actor/view, preserved across appearance updates.
    pub last_model: std::cell::Cell<u64>,
    pub visual: crate::actor_visual::State,
    pub attachment: Option<crate::location_merge::Attachment>,
    pub path: crate::actor_path::Path,
    pub sequence: crate::actor_sequence::State,
    pub pose: crate::actor_pose::Pose,
    pub idle_fallback: crate::actor_pose::Pose,
    pub spots: crate::actor_spot::Effects,
    pub overlays: crate::actor_overlay::State,
    pub turn: crate::actor_turn::State,
    pub facing: Option<crate::actor_update::Face>,
    pub freezes: crate::actor_freeze::Freezes,
    pub exact: Option<crate::actor_exact::Exact>,
    pub position: [i32; 3],
    pub queued_move: bool,
    pub moved: bool,
    pub speed: i8,
    pub appearance: Option<player_appearance::Update>,
    pub blocks: Vec<Update>,
}
#[derive(Clone, Debug)]
pub struct View {
    pub base: [i32; 2],
    pub dimensions: [i32; 2],
    pub players: std::collections::BTreeMap<u16, Player>,
}
impl View {
    pub fn contains(&self, position: [i32; 3]) -> bool {
        (0..2).all(|axis| {
            position[axis + 1] > self.base[axis]
                && position[axis + 1] < self.base[axis].wrapping_add(self.dimensions[axis])
        })
    }
}
#[derive(Clone)]
pub struct State {
    /// Pending composition hash evictions, retained even when an actor leaves a view.
    pub model_invalidations: std::collections::HashSet<u64>,
    pub local: u16,
    pub local_view: u16,
    pub view_order: Vec<u16>,
    pub views: std::collections::BTreeMap<u16, View>,
    pub(crate) duplicate_views: std::collections::BTreeMap<(u16,u64), View>,
    retired_views: std::collections::BTreeMap<(u16,u64), View>,
    instance_generations: std::collections::BTreeMap<u16,u64>,
    instance_order: Vec<(u16,u64)>,
    faces: Vec<Option<crate::actor_update::Face>>,
    visuals: Vec<crate::actor_visual::State>,
    npc_sizes: std::sync::Arc<std::collections::HashMap<u16, u8>>,
    /// Logical wire records. Scene consumers use root_actor/root_iter or views.
    pub players: Vec<Option<Player>>,
    pub high: Vec<u16>,
    pub low: Vec<u16>,
    /// Captured in wire order before subsequent appearance blocks mutate identity.
    pub chat_events: Vec<(u16, String, bool, Update)>,
    regions: Vec<i32>,
    flags: Vec<u8>,
    appearances: Vec<Option<player_appearance::Update>>,
    speeds: Vec<i8>,
}
const WALK: [[i32; 2]; 8] = [
    [-1, -1],
    [0, -1],
    [1, -1],
    [-1, 0],
    [1, 0],
    [-1, 1],
    [0, 1],
    [1, 1],
];
const RUN: [[i32; 2]; 16] = [
    [-2, -2],
    [-1, -2],
    [0, -2],
    [1, -2],
    [2, -2],
    [-2, -1],
    [2, -1],
    [-2, 0],
    [2, 0],
    [-2, 1],
    [2, 1],
    [-2, 2],
    [-1, 2],
    [0, 2],
    [1, 2],
    [2, 2],
];
fn appearance_size(
    appearance: Option<&player_appearance::Update>,
    sizes: &std::collections::HashMap<u16, u8>,
) -> i32 {
    appearance
        .and_then(|a| a.appearance.npc)
        .and_then(|id| sizes.get(&id))
        .map_or(1, |&size| size as i32)
}
impl State {
    pub fn set_npc_definitions(
        &mut self,
        definitions: &std::collections::HashMap<u32, crate::npc::Definition>,
    ) {
        self.npc_sizes = std::sync::Arc::new(
            definitions
                .iter()
                .filter_map(|(&id, d)| u16::try_from(id).ok().map(|id| (id, d.size)))
                .collect(),
        );
    }

    pub fn root_actor(&self, index: u16) -> Option<&Player> {
        if let Some(view) = self.views.get(&0) {
            view.players.get(&index)
        } else {
            self.players.get(index as usize)?.as_ref()
        }
    }
    pub fn view_actor(&self, view: u16, index: u16) -> Option<&Player> {
        if view == 0 { self.root_actor(index) }
        else { self.views.get(&view)?.players.get(&index) }
    }
    pub fn root_actor_mut(&mut self, index: u16) -> Option<&mut Player> {
        if let Some(view) = self.views.get_mut(&0) {
            view.players.get_mut(&index)
        } else {
            self.players.get_mut(index as usize)?.as_mut()
        }
    }
    pub fn root_iter(&self) -> impl Iterator<Item = (u16, &Player)> {
        self.views
            .get(&0)
            .into_iter()
            .flat_map(|v| v.players.iter().map(|(&i, p)| (i, p)))
            .chain(
                (!self.views.contains_key(&0))
                    .then_some(&self.players)
                    .into_iter()
                    .flat_map(|v| {
                        v.iter()
                            .enumerate()
                            .filter_map(|(i, p)| p.as_ref().map(|p| (i as u16, p)))
                    }),
            )
    }
    pub fn root_iter_mut(&mut self) -> impl Iterator<Item = (u16, &mut Player)> {
        let fallback = !self.views.contains_key(&0);
        self.views
            .get_mut(&0)
            .into_iter()
            .flat_map(|v| v.players.iter_mut().map(|(&i, p)| (i, p)))
            .chain(
                fallback
                    .then_some(&mut self.players)
                    .into_iter()
                    .flat_map(|v| {
                        v.iter_mut()
                            .enumerate()
                            .filter_map(|(i, p)| p.as_mut().map(|p| (i as u16, p)))
                    }),
            )
    }
    pub fn local_actor(&self) -> Option<&Player> {
        if self.local_view == 0 {
            self.root_actor(self.local)
        } else {
            self.views.get(&self.local_view)?.players.get(&self.local)
        }
    }
    pub fn local_actor_mut(&mut self) -> Option<&mut Player> {
        if self.local_view == 0 {
            self.root_actor_mut(self.local)
        } else {
            self.views.get_mut(&self.local_view)?.players.get_mut(&self.local)
        }
    }
    /// Returns bytes consumed so the caller can decode the following region rebuild.
    pub fn initialize(data: &[u8], local: u16) -> Result<(Self, usize)> {
        if local == 0 || local >= 2048 {
            return Err(Error::Invalid("local player index"));
        }
        let mut b = BitReader::new(data);
        let packed = b.bits(30)?;
        let mut s = Self {
            model_invalidations: Default::default(),
            local,
            local_view: 0,
            view_order: Vec::new(),
            views: Default::default(),
            duplicate_views: Default::default(),
            retired_views: Default::default(),
            instance_generations: Default::default(),
            instance_order: Default::default(),
            faces: vec![None; 2048],
            visuals: vec![Default::default(); 2048],
            npc_sizes: Default::default(),
            players: vec![None; 2048],
            high: vec![local],
            low: vec![],
            chat_events: vec![],
            regions: vec![0; 2048],
            flags: vec![0; 2048],
            appearances: vec![None; 2048],
            speeds: vec![1; 2048],
        };
        s.players[local as usize] = Some(Player {
            last_model: Default::default(),
            visual: Default::default(),
            attachment: None,
            path: Default::default(),
            sequence: Default::default(),
            pose: Default::default(),
            idle_fallback: Default::default(),
            spots: Default::default(),
            overlays: Default::default(),
            turn: Default::default(),
            facing: None,
            freezes: Default::default(),
            exact: None,
            position: [
                (packed >> 28) as i32,
                (packed >> 14 & 16383) as i32,
                (packed & 16383) as i32,
            ],
            queued_move: false,
            moved: false,
            speed: 1,
            appearance: None,
            blocks: vec![],
        });
        {
            let p = s.players[local as usize].as_mut().unwrap();
            p.path.teleport([p.position[1], p.position[2]], 1);
        }
        for i in 1..2048 {
            if i != local as usize {
                let v = b.bits(18)? as i32;
                s.regions[i] = ((v >> 16) << 28) | ((v >> 8 & 255) << 14) | (v & 255);
                s.low.push(i as u16);
            }
        }
        Ok((s, b.bytes_consumed()))
    }
    pub fn decode(&mut self, data: &[u8], items: &Definitions, huffman: &Huffman) -> Result<()> {
        self.decode_with_sequences(data, items, huffman, None, 0)
    }
    pub fn decode_with_sequences(
        &mut self,
        data: &[u8],
        items: &Definitions,
        huffman: &Huffman,
        sequences: Option<&std::collections::HashMap<u32, crate::animation::Sequence>>,
        cycle: i32,
    ) -> Result<()> {
        let mut next = self.clone();
        next.decode_inner(data, items, huffman, sequences, cycle)?;
        *self = next;
        Ok(())
    }
    /// Apply view geometry and player synchronization as a single transaction.
    pub fn decode_in_views(
        &mut self,
        data: &[u8],
        items: &Definitions,
        huffman: &Huffman,
        sequences: Option<&std::collections::HashMap<u32, crate::animation::Sequence>>,
        cycle: i32,
        views: &[(u16, [i32; 2], [i32; 2])],
    ) -> Result<()> {
        let descriptors=views.iter().map(|&(id,base,size)|(id,0,base,size)).collect::<Vec<_>>();
        self.decode_in_instances(data,items,huffman,sequences,cycle,&descriptors)
    }
    /// Synchronize every registered scene instance while preserving surviving actor state.
    pub fn decode_in_instances(
        &mut self,data:&[u8],items:&Definitions,huffman:&Huffman,
        sequences:Option<&std::collections::HashMap<u32,crate::animation::Sequence>>,
        cycle:i32,views:&[(u16,u64,[i32;2],[i32;2])],
    )->Result<()> {
        let mut next=self.clone();next.reconcile_instances(views);
        next.decode_inner(data,items,huffman,sequences,cycle)?;*self=next;Ok(())
    }
    /// dy.ao membership runs after logical movement and before extension blocks.
    pub fn sync_views(&mut self) {
        for view in self.views.values_mut().chain(self.duplicate_views.values_mut()) {
            let base = view.base;
            let dimensions = view.dimensions;
            let inside = |position: [i32; 3]| {
                (0..2).all(|axis| {
                    position[axis + 1] > base[axis]
                        && position[axis + 1] < base[axis].wrapping_add(dimensions[axis])
                })
            };
            view.players.retain(|&index, _| {
                self.players[index as usize]
                    .as_ref()
                    .is_some_and(|p| inside(p.position))
            });
            for &index in &self.high {
                let Some(logical) = &self.players[index as usize] else {
                    continue;
                };
                if !inside(logical.position) {
                    continue;
                }
                let copy = view.players.entry(index).or_insert_with(|| {
                    let mut actor = Player::at(
                        logical.position,
                        self.appearances[index as usize].clone(),
                        self.speeds[index as usize],
                        self.faces[index as usize].clone(),
                    );
                    actor.path.teleport(
                        [logical.position[1], logical.position[2]],
                        appearance_size(actor.appearance.as_ref(), &self.npc_sizes),
                    );
                    actor.visual = self.visuals[index as usize].view_copy();
                    actor
                });
                copy.position = logical.position;
                copy.moved = logical.moved;
                copy.queued_move = logical.queued_move;
                copy.blocks.clear();
            }
        }
        self.select_local_instance();
    }
    fn decode_inner(
        &mut self,
        data: &[u8],
        items: &Definitions,
        huffman: &Huffman,
        sequences: Option<&std::collections::HashMap<u32, crate::animation::Sequence>>,
        cycle: i32,
    ) -> Result<()> {
        self.chat_events.clear();
        for p in self.players.iter_mut().flatten() {
            p.blocks.clear();
            p.moved = false;
            p.queued_move = false;
        }
        let mut offset = 0;
        let mut extended = vec![];
        for (high, inactive) in [(true, false), (true, true), (false, true), (false, false)] {
            let indices = if high {
                self.high.clone()
            } else {
                self.low.clone()
            };
            let mut b = BitReader::new(&data[offset..]);
            let mut skip = 0;
            for index in indices {
                let i = index as usize;
                if (self.flags[i] & 1 != 0) != inactive {
                    continue;
                }
                if skip > 0 {
                    skip -= 1;
                    self.flags[i] |= 2;
                } else if b.bits(1)? == 0 {
                    skip = match b.bits(2)? {
                        0 => 0,
                        1 => b.bits(5)?,
                        2 => b.bits(8)?,
                        _ => b.bits(11)?,
                    };
                    self.flags[i] |= 2;
                } else if high {
                    self.high_move(&mut b, i, &mut extended)?;
                } else if self.low_move(&mut b, i, &mut extended, 0)? {
                    self.flags[i] |= 2;
                }
            }
            if skip != 0 {
                return Err(Error::Invalid("player skip exceeds pass"));
            }
            offset += b.bytes_consumed();
        }
        self.high.clear();
        self.low.clear();
        for i in 1..2048 {
            self.flags[i] >>= 1;
            if self.players[i].is_some() {
                self.high.push(i as u16);
            } else {
                self.low.push(i as u16);
            }
        }
        self.sync_views();
        for p in self
            .players
            .iter_mut()
            .flatten()
            .chain(self.views.values_mut().chain(self.duplicate_views.values_mut()).flat_map(|v| v.players.values_mut()))
        {
            if p.moved && !p.queued_move {
                p.move_path(p.speed, sequences);
            }
        }
        let mut r = Reader::new(&data[offset..]);
        for i in extended {
            let blocks = player_update::decode(&mut r, items, huffman)?;
            // Evict before any actor receives its new composition. This queue is
            // transactional with packet decoding and survives actor removals.
            for block in &blocks {
                if let Update::Appearance(next) = block {
                    for actor in self.players[i].iter().chain(self.views.values().chain(self.duplicate_views.values()).filter_map(|v| v.players.get(&(i as u16)))) {
                        if let Some(old) = &actor.appearance {
                            if next.force_refresh || old.appearance.render_key() != next.appearance.render_key() {
                                self.model_invalidations.insert(old.appearance.render_key());
                            }
                        }
                    }
                }
            }

            for block in &blocks {
                if matches!(block, Update::Reset(_)) {
                    self.speeds[i] = 1;
                    self.appearances[i] = None;
                    self.faces[i] = None;
                    self.visuals[i] = Default::default();
                }
            }
            for block in &blocks {
                if let Update::Actor(block) = block {
                    self.visuals[i].apply(block, cycle);
                }
                match block {
                    Update::Appearance(a) => self.appearances[i] = Some(a.clone()),
                    Update::MoveSpeed(v) => self.speeds[i] = *v,
                    Update::Actor(crate::actor_update::Block::Face(face)) => {
                        self.faces[i] = Some(face.clone())
                    }
                    _ => {}
                }
            }
            let p = self.players[i]
                .as_mut()
                .ok_or(Error::Invalid("player extension index"))?;
            self.chat_events.extend(
                p.apply_updates(&blocks, sequences, cycle, &self.npc_sizes)?
                    .into_iter()
                    .map(|(name, hidden, block)| (i as u16, name, hidden, block)),
            );
            for view in self.views.values_mut().chain(self.duplicate_views.values_mut()) {
                if let Some(copy) = view.players.get_mut(&(i as u16)) {
                    copy.apply_updates(&blocks, sequences, cycle, &self.npc_sizes)?;
                }
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("player info trailing bytes"));
        }
        Ok(())
    }
    fn high_move(&mut self, b: &mut BitReader<'_>, i: usize, ext: &mut Vec<usize>) -> Result<()> {
        let extended = b.bits(1)? != 0;
        if extended {
            ext.push(i);
        }
        let kind = b.bits(2)?;
        let p = self.players[i]
            .as_mut()
            .ok_or(Error::Invalid("high resolution player missing"))?;
        if kind == 0 {
            if extended {
                return Ok(());
            }
            if i == self.local as usize {
                return Err(Error::Invalid("local player removal"));
            }
            self.regions[i] =
                (p.position[0] << 28) | (p.position[1] >> 13 << 14) | (p.position[2] >> 13);
            self.players[i] = None;
            if b.bits(1)? != 0 {
                self.low_move(b, i, ext, 0)?;
            }
            return Ok(());
        }
        p.moved = true;
        p.queued_move = extended;
        if kind == 1 || kind == 2 {
            let d = if kind == 1 {
                WALK[b.bits(3)? as usize]
            } else {
                RUN[b.bits(4)? as usize]
            };
            p.position[1] += d[0];
            p.position[2] += d[1];
        } else if b.bits(1)? == 0 {
            let v = b.bits(12)? as i32;
            p.position[0] = (p.position[0] + (v >> 10)) & 3;
            let x = (v >> 5) & 31;
            let z = v & 31;
            p.position[1] += if x > 15 { x - 32 } else { x };
            p.position[2] += if z > 15 { z - 32 } else { z };
        } else {
            let v = b.bits(30)? as i32;
            p.position[0] = (p.position[0] + (v >> 28)) & 3;
            p.position[1] = (p.position[1] + (v >> 14 & 16383)) & 16383;
            p.position[2] = (p.position[2] + (v & 16383)) & 16383;
        }
        Ok(())
    }
    fn low_move(
        &mut self,
        b: &mut BitReader<'_>,
        i: usize,
        ext: &mut Vec<usize>,
        depth: usize,
    ) -> Result<bool> {
        if depth >= 32 {
            return Err(Error::Invalid("player region recursion"));
        }
        let kind = b.bits(2)?;
        if kind == 0 {
            if b.bits(1)? != 0 {
                self.low_move(b, i, ext, depth + 1)?;
            }
            let x = b.bits(13)? as i32;
            let z = b.bits(13)? as i32;
            if b.bits(1)? != 0 {
                ext.push(i);
            }
            if self.players[i].is_some() {
                return Err(Error::Invalid("duplicate visible player"));
            }
            let v = self.regions[i];
            self.players[i] = Some(Player {
            last_model: Default::default(),
                visual: self.visuals[i].view_copy(),
                attachment: None,
                path: Default::default(),
                sequence: Default::default(),
                pose: Default::default(),
            idle_fallback: Default::default(),
                spots: Default::default(),
                overlays: Default::default(),
                turn: Default::default(),
                facing: self.faces[i].clone(),
                freezes: Default::default(),
                exact: None,
                position: [v >> 28, ((v >> 14 & 255) << 13) + x, ((v & 255) << 13) + z],
                queued_move: false,
                moved: false,
                speed: self.speeds[i],
                appearance: self.appearances[i].clone(),
                blocks: vec![],
            });
            {
                let p = self.players[i].as_mut().unwrap();
                p.path.teleport(
                    [p.position[1], p.position[2]],
                    appearance_size(p.appearance.as_ref(), &self.npc_sizes),
                );
            }
            return Ok(true);
        }
        let v = self.regions[i];
        let level = v >> 28;
        let x = v >> 14 & 255;
        let z = v & 255;
        self.regions[i] = match kind {
            1 => (((level + b.bits(2)? as i32) & 3) << 28) | (v & 0xfffffff),
            2 => {
                let n = b.bits(5)? as i32;
                let d = WALK[(n & 7) as usize];
                (((level + (n >> 3)) & 3) << 28)
                    .wrapping_add((x + d[0]) << 14)
                    .wrapping_add(z + d[1])
            }
            _ => {
                let n = b.bits(18)? as i32;
                (((level + (n >> 16)) & 3) << 28)
                    | (((x + (n >> 8 & 255)) & 255) << 14)
                    | ((z + (n & 255)) & 255)
            }
        };
        Ok(false)
    }
}

impl Player {
    pub fn at(
        position: [i32; 3],
        appearance: Option<player_appearance::Update>,
        speed: i8,
        facing: Option<crate::actor_update::Face>,
    ) -> Self {
        let mut path = crate::actor_path::Path::default();
        path.teleport([position[1], position[2]], 1);
        path.speeds[0] = speed;
        Self {
            last_model: Default::default(),
            visual: Default::default(),
            attachment: None,
            path,
            sequence: Default::default(),
            pose: Default::default(),
            idle_fallback: Default::default(),
            spots: Default::default(),
            overlays: Default::default(),
            turn: Default::default(),
            facing,
            freezes: Default::default(),
            exact: None,
            position,
            queued_move: false,
            moved: false,
            speed,
            appearance,
            blocks: Vec::new(),
        }
    }
    fn apply_updates(
        &mut self,
        blocks: &[Update],
        sequences: Option<&std::collections::HashMap<u32, crate::animation::Sequence>>,
        cycle: i32,
        npc_sizes: &std::collections::HashMap<u16, u8>,
    ) -> Result<Vec<(String, bool, Update)>> {
        let p = self;
        let mut chat = Vec::new();
        // The original resets state before processing any other flagged fields,
        // although its reset byte occurs later in wire order.
        if blocks.iter().any(|b| matches!(b, Update::Reset(_))) {
            // dk.xd drops the logical appearance cache, but an existing
            // actor retains its current composition until another update.
            p.visual = Default::default();
            p.speed = 1;
            p.path.speeds[0] = 1;
            p.facing = None;
            // This is li.bd(-1, 0), not the lighter boundary reset: it
            // normalizes the animation counters and captures the path length.
            p.sequence
                .set(-1, 0, p.path.len as i32, &Default::default())?;
            p.path.path_at_start = p.sequence.path_at_start;
            if let Some(a) = &mut p.appearance {
                a.actions = Default::default();
            }
        }
        let mut temporary_speed = -1;
        for block in blocks {
            if let Update::Actor(block) = block {
                p.visual.apply(block, cycle);
            }
            match block {
                Update::Actor(crate::actor_update::Block::SpotAnimations(spots)) => {
                    for &(slot, id, height, delay) in spots {
                        p.spots.apply(slot, id, height, delay, cycle);
                    }
                }
                Update::Actor(crate::actor_update::Block::Face(face)) => {
                    p.facing = Some(face.clone())
                }
                Update::Appearance(a) => {
                    p.appearance = Some(a.clone());
                }
                Update::Actions(a) => {
                    if let Some(v) = &mut p.appearance {
                        v.actions = a.clone();
                    }
                }
                Update::Chat(_) | Update::Actor(crate::actor_update::Block::Say(_)) => {
                    if let Update::Actor(crate::actor_update::Block::Say(text)) = block {
                        p.overlays.speech.say(text, true);
                    }
                    if let Some(a) = &p.appearance {
                        chat.push((a.name.clone(), a.hidden, block.clone()));
                    }
                }
                Update::Actor(crate::actor_update::Block::Freeze {
                    delay,
                    duration,
                    cancel_sequence,
                }) => p
                    .freezes
                    .apply(*delay, *duration, *cancel_sequence, cycle, &mut p.sequence),
                Update::Actor(crate::actor_update::Block::ExactMove {
                    offsets,
                    delays,
                    direction,
                }) => {
                    let at = [p.position[1], p.position[2]];
                    let to = [at[0] + offsets[2] as i32, at[1] + offsets[3] as i32];
                    p.path.clear();
                    p.path.tiles[0] = to;
                    p.queued_move = false;
                    p.exact = Some(crate::actor_exact::Exact {
                        start: cycle.wrapping_add(delays[0] as i32),
                        end: cycle.wrapping_add(delays[1] as i32),
                        from: [at[0] + offsets[0] as i32, at[1] + offsets[1] as i32],
                        to,
                        direction: *direction,
                    });
                }
                Update::TemporarySpeed(v) => temporary_speed = *v,
                Update::Actor(crate::actor_update::Block::Sequence { id, delay }) => {
                    if let Some(sequences) = sequences {
                        p.sequence.set(
                            if *id == 65535 { -1 } else { *id as i32 },
                            *delay as i32,
                            p.path.len as i32,
                            sequences,
                        )?;
                        p.path.path_at_start = p.sequence.path_at_start;
                    }
                }
                Update::MoveSpeed(v) => {
                    p.speed = *v;
                }
                _ => {}
            }
        }
        p.blocks = blocks.to_vec();
        if p.queued_move {
            if temporary_speed == 127 {
                p.path.teleport(
                    [p.position[1], p.position[2]],
                    appearance_size(p.appearance.as_ref(), npc_sizes),
                );
            } else {
                p.move_path(
                    if temporary_speed == -1 {
                        p.speed
                    } else {
                        temporary_speed
                    },
                    sequences,
                );
            }
        }

        Ok(chat)
    }
    fn move_path(
        &mut self,
        speed: i8,
        sequences: Option<&std::collections::HashMap<u32, crate::animation::Sequence>>,
    ) {
        if sequences
            .and_then(|s| s.get(&(self.sequence.id as u32)))
            .is_some_and(|s| s.priority_moving == 1)
        {
            self.sequence.reset();
        }
        self.path.push([self.position[1], self.position[2]], speed);
    }
}

#[cfg(test)]
mod reset_tests {
    use super::*;
    #[test]
    fn reset_drops_cached_appearance_but_keeps_visible_composition() {
        let (mut state, _) = State::initialize(&[0; 4608], 1).unwrap();
        let appearance = player_appearance::Update {
            appearance: Default::default(),
            skull: -1,
            overhead: -1,
            animations: [-1; 7],
            name: "Retained".into(),
            combat: 3,
            skill: 0,
            hidden: false,
            force_refresh: false,
            actions: ["a".into(), "b".into(), "c".into()],
            team: 0,
        };
        state.appearances[1] = Some(appearance.clone());
        state.players[1].as_mut().unwrap().appearance = Some(appearance);
        let items = Definitions::from_records(vec![], false).unwrap();
        let huffman = Huffman::new(&[8; 256]).unwrap();
        // Local update/no movement, then skip the other 2046 indices, then reset.
        state
            .decode(&[0xc0, 0x7f, 0xf4, 2, 0], &items, &huffman)
            .unwrap();
        assert!(state.appearances[1].is_none());
        let visible = state.players[1]
            .as_ref()
            .unwrap()
            .appearance
            .as_ref()
            .unwrap();
        assert_eq!(visible.name, "Retained");
        assert!(visible.actions.iter().all(String::is_empty));
    }
}

#[path = "player_view_instances.rs"]
mod view_instances;
pub use view_instances::instance_descriptors;
