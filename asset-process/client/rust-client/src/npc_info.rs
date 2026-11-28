//! Transactional revision 239 NPC synchronization (small/large V5 and qz.eo).
use crate::{
    actor_update::{Block, Customisation, Face, FaceTarget, Headbar, Hit},
    buffer::{BitReader, Reader, Transform},
    npc::Definition,
    Error, Result,
};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speed {
    Crawl,
    Walk,
    Run,
}
#[derive(Clone, Debug)]
pub struct Npc {
    pub visual: crate::actor_visual::State,
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
    pub id: u16,
    pub position: [i32; 2],
    pub angle: u16,
    pub spawn_cycle: Option<u32>,
    pub steps: Vec<(u8, Speed)>,
    pub jump: bool,
    pub blocks: Vec<Block>,
    pub appearance: Appearance,
    pub animations: [i32; 15],
}
/// Persistent overrides survive ticks that contain no extension blocks.
#[derive(Clone, Debug)]
pub struct Appearance {
    pub body: Option<Customisation>,
    pub head: Option<Customisation>,
    pub name: String,
    pub combat: i32,
    pub operations: u8,
    pub head_icons: Option<[(i32, i32); 8]>,
}
impl Default for Appearance {
    fn default() -> Self {
        Self {
            body: None,
            head: None,
            name: String::new(),
            combat: -1,
            operations: 31,
            head_icons: None,
        }
    }
}
impl Appearance {
    pub fn set_head_icon(
        &mut self,
        base: &[(i32, i16)],
        slot: u8,
        group: i32,
        image: i16,
    ) -> Result<()> {
        if slot >= 8 {
            return Err(Error::Invalid("NPC head icon slot"));
        }
        let icons = self.head_icons.get_or_insert_with(|| {
            std::array::from_fn(|i| base.get(i).map_or((-1, -1), |&(g, n)| (g, n as i32)))
        });
        icons[slot as usize] = (group, image as i32);
        Ok(())
    }
    fn apply(&mut self, block: &Block) {
        match block {
            Block::Body(v) => self.body = v.clone(),
            Block::Head(v) => self.head = v.clone(),
            Block::Name(v) => self.name = v.clone(),
            Block::Combat(v) => self.combat = *v,
            Block::Ops(v) => self.operations = *v,
            Block::HeadIcons(v) => self.head_icons = *v,
            _ => {}
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub indices: Vec<u16>,
    pub npcs: HashMap<u16, Npc>,
    /// xc node insertion order, independent of the packet's visible list.
    pub insertion_order: Vec<u16>,
}
const ANGLES: [u16; 8] = [768, 1024, 1280, 512, 1536, 256, 0, 1792];
const STEPS: [[i32; 2]; 8] = [
    [-1, 1],
    [0, 1],
    [1, 1],
    [-1, 0],
    [1, 0],
    [-1, -1],
    [0, -1],
    [1, -1],
];
impl State {
    /// ht.bt traverses GameWorldView.ac: 128 buckets in root, 8 in children.
    /// Existing nodes retain their position when reused within one NPC packet.
    pub fn iteration_order(&self, view: u16) -> Vec<u16> {
        let mut order = Vec::with_capacity(self.npcs.len());
        let mut seen = HashSet::new();
        for &index in self.insertion_order.iter().chain(&self.indices) {
            if self.npcs.contains_key(&index) && seen.insert(index) { order.push(index); }
        }
        // Deterministic fallback for hosts that insert directly into the public map.
        let mut missing = self.npcs.keys().copied().filter(|id| !seen.contains(id)).collect::<Vec<_>>();
        missing.sort_unstable();
        order.extend(missing);
        let mask = if view == 0 { 127 } else { 7 };
        order.sort_by_key(|index| *index & mask);
        order
    }
    pub fn decode(
        &mut self,
        data: &[u8],
        large: bool,
        base: [i32; 2],
        defs: &HashMap<u32, Definition>,
    ) -> Result<Vec<u16>> {
        self.decode_with_sequences(data, large, base, defs, None, 0)
    }
    pub fn decode_with_sequences(
        &mut self,
        data: &[u8],
        large: bool,
        base: [i32; 2],
        defs: &HashMap<u32, Definition>,
        sequences: Option<&HashMap<u32, crate::animation::Sequence>>,
        cycle: i32,
    ) -> Result<Vec<u16>> {
        let mut next = self.clone();
        let removed = next.decode_inner(data, large, base, defs, sequences, cycle)?;
        *self = next;
        Ok(removed)
    }
    fn decode_inner(
        &mut self,
        data: &[u8],
        large: bool,
        base: [i32; 2],
        defs: &HashMap<u32, Definition>,
        sequences: Option<&HashMap<u32, crate::animation::Sequence>>,
        cycle: i32,
    ) -> Result<Vec<u16>> {
        let mut bits = BitReader::new(data);
        let count = bits.bits(8)? as usize;
        if count > self.indices.len() {
            return Err(Error::Invalid("NPC retained count"));
        }
        let old = std::mem::take(&mut self.indices);
        let mut extended = vec![];
        let mut used = HashSet::new();
        for &index in old.iter().take(count) {
            let npc = self
                .npcs
                .get_mut(&index)
                .ok_or(Error::Invalid("NPC retained index"))?;
            npc.steps.clear();
            npc.blocks.clear();
            npc.jump = false;
            if bits.bits(1)? == 0 {
                self.indices.push(index);
                used.insert(index);
                continue;
            }
            match bits.bits(2)? {
                0 => extended.push(index),
                1 => {
                    step(npc, bits.bits(3)? as u8, Speed::Walk, sequences);
                    if bits.bits(1)? != 0 {
                        extended.push(index);
                    }
                }
                2 => {
                    if bits.bits(1)? != 0 {
                        step(npc, bits.bits(3)? as u8, Speed::Run, sequences);
                        step(npc, bits.bits(3)? as u8, Speed::Run, sequences);
                    } else {
                        step(npc, bits.bits(3)? as u8, Speed::Crawl, sequences);
                    }
                    if bits.bits(1)? != 0 {
                        extended.push(index);
                    }
                }
                _ => continue,
            }
            self.indices.push(index);
            used.insert(index);
        }
        while bits.remaining_bits() >= 28 {
            let index = bits.bits(16)? as u16;
            if index == 65535 {
                break;
            }
            if !used.insert(index) {
                return Err(Error::Invalid("duplicate NPC index"));
            }
            let spawn_cycle = if bits.bits(1)? != 0 {
                Some(bits.bits(32)?)
            } else {
                None
            };
            let update = bits.bits(1)? != 0;
            let width = if large { 8 } else { 6 };
            let x = delta(&mut bits, width)?;
            let angle = ANGLES[bits.bits(3)? as usize];
            let z = delta(&mut bits, width)?;
            let jump = bits.bits(1)? != 0;
            let id = bits.bits(14)? as u16;
            let d = defs
                .get(&(id as u32))
                .ok_or(Error::Invalid("NPC spawn definition"))?;
            let is_new = !self.npcs.contains_key(&index);
            if is_new { self.insertion_order.push(index); }
            let npc = self.npcs.entry(index).or_insert(Npc {
                visual: Default::default(),
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
                id,
                position: base,
                angle,
                spawn_cycle: None,
                steps: vec![],
                jump: true,
                blocks: vec![],
                appearance: Appearance::default(),
                animations: d.animations,
            });
            if is_new {
                npc.turn.angle = angle;
                npc.turn.target = angle;
            }
            npc.id = id;
            npc.animations = d.animations;
            npc.position = [base[0].wrapping_add(x), base[1].wrapping_add(z)];
            npc.cancel_on_move(sequences);
            let delta = [
                npc.position[0] - npc.path.tiles[0][0],
                npc.position[1] - npc.path.tiles[0][1],
            ];
            if is_new || jump || delta.iter().any(|v| !(-8..=8).contains(v)) {
                npc.path.teleport(npc.position, d.size as i32);
            } else {
                npc.path.push(npc.position, 1);
            }
            npc.jump = jump;
            npc.steps.clear();
            npc.blocks.clear();
            if let Some(cycle) = spawn_cycle {
                npc.spawn_cycle = Some(cycle);
            }
            if d.turn_speed == 0 {
                npc.angle = 0;
                npc.turn.angle = 0;
            }
            self.indices.push(index);
            if update {
                extended.push(index);
            }
        }
        let mut r = Reader::new(&data[bits.bytes_consumed()..]);
        for index in extended {
            let npc = self.npcs.get_mut(&index).unwrap();
            let d = defs
                .get(&(npc.id as u32))
                .ok_or(Error::Invalid("NPC update definition"))?;
            npc.blocks = decode_blocks(&mut r, d)?;
            for block in &npc.blocks {
                npc.appearance.apply(block);
                npc.visual.apply(block, cycle);
                if let Block::Say(text) = block {
                    npc.overlays.speech.say(text, false);
                }
                if let Block::SpotAnimations(spots) = block {
                    for &(slot, id, height, delay) in spots {
                        npc.spots.apply(slot, id, height, delay, cycle);
                    }
                }
                if let Block::Face(face) = block {
                    npc.facing = Some(face.clone());
                }
                if let Block::BaseAnimations(poses) = block {
                    let defaults = defs[&(npc.id as u32)].animations;
                    npc.animations = std::array::from_fn(|i| {
                        poses[i].map_or(defaults[i], |id| if id == 65535 { -1 } else { id as i32 })
                    });
                }
                if let Block::ExactMove {
                    offsets,
                    delays,
                    direction,
                } = block
                {
                    let at = npc.path.tiles[0];
                    npc.path.clear();
                    npc.exact = Some(crate::actor_exact::Exact {
                        start: cycle.wrapping_add(delays[0] as i32),
                        end: cycle.wrapping_add(delays[1] as i32),
                        from: [at[0] + offsets[0] as i32, at[1] + offsets[1] as i32],
                        to: [at[0] + offsets[2] as i32, at[1] + offsets[3] as i32],
                        direction: *direction,
                    });
                }
                if let Block::Freeze {
                    delay,
                    duration,
                    cancel_sequence,
                } = block
                {
                    npc.freezes.apply(
                        *delay,
                        *duration,
                        *cancel_sequence,
                        cycle,
                        &mut npc.sequence,
                    );
                }
                if let (Block::Sequence { id, delay }, Some(sequences)) = (block, sequences) {
                    npc.sequence.set(
                        if *id == 65535 { -1 } else { *id as i32 },
                        *delay as i32,
                        npc.path.len as i32,
                        sequences,
                    )?;
                    npc.path.path_at_start = npc.sequence.path_at_start;
                }
                if let Block::Transform(id) = block {
                    if !defs.contains_key(&(*id as u32)) {
                        return Err(Error::Invalid("NPC transform definition"));
                    }
                    npc.id = *id;
                    npc.animations = defs[&(*id as u32)].animations;
                }
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("NPC info trailing bytes"));
        }
        let removed = old
            .into_iter()
            .filter(|id| !used.contains(id))
            .collect::<Vec<_>>();
        for id in &removed {
            self.npcs.remove(id);
        }
        let npcs = &self.npcs;
        self.insertion_order.retain(|id| npcs.contains_key(id));
        Ok(removed)
    }
}
impl Npc {
    fn cancel_on_move(&mut self, sequences: Option<&HashMap<u32, crate::animation::Sequence>>) {
        if sequences
            .and_then(|s| s.get(&(self.sequence.id as u32)))
            .is_some_and(|s| s.priority_moving == 1)
        {
            self.sequence.reset();
        }
    }
}
fn step(
    npc: &mut Npc,
    direction: u8,
    speed: Speed,
    sequences: Option<&HashMap<u32, crate::animation::Sequence>>,
) {
    let d = STEPS[direction as usize];
    npc.position[0] = npc.position[0].wrapping_add(d[0]);
    npc.position[1] = npc.position[1].wrapping_add(d[1]);
    npc.steps.push((direction, speed));
    npc.cancel_on_move(sequences);
    npc.path.push(
        npc.position,
        match speed {
            Speed::Crawl => 0,
            Speed::Walk => 1,
            Speed::Run => 2,
        },
    );
}
fn delta(r: &mut BitReader<'_>, width: u8) -> Result<i32> {
    let v = r.bits(width)? as i32;
    Ok(if v & (1 << (width - 1)) != 0 {
        v - (1 << width)
    } else {
        v
    })
}
fn byte(r: &mut Reader<'_>, alt: u8) -> Result<u8> {
    r.transformed(match alt {
        1 => Transform::Add,
        2 => Transform::Neg,
        3 => Transform::Sub,
        _ => Transform::None,
    })
}
fn short(r: &mut Reader<'_>, alt: u8) -> Result<u16> {
    match alt {
        1 => r.u16_le(),
        2 => r.u16_add(false),
        3 => r.u16_add(true),
        _ => r.u16(),
    }
}
fn nullable_large(r: &mut Reader<'_>) -> Result<i32> {
    let wide = r.clone().u8()? & 128 != 0;
    let v = r.large_smart()?;
    Ok(if !wide && v == 32767 { -1 } else { v as i32 })
}
fn custom(r: &mut Reader<'_>, d: &Definition, kind: u8) -> Result<Option<Customisation>> {
    let flags = byte(
        r,
        match kind {
            3 => 2,
            1 => 1,
            _ => 0,
        },
    )?;
    if flags & 1 != 0 {
        return Ok(None);
    }
    let mut c = Customisation::default();
    if flags & 2 != 0 {
        let count = byte(
            r,
            match kind {
                3 => 1,
                1 => 3,
                _ => 0,
            },
        )?;
        let mut ids = vec![];
        for _ in 0..count {
            ids.push(match kind {
                3 => r.u32()?,
                1 => r.u32_inverse_middle()?,
                _ => r.u32_le()?,
            });
        }
        c.models = Some(ids);
    }
    if flags & 4 != 0 {
        let count = if kind == 3 {
            byte(r, 2)? as usize
        } else {
            d.recolor.len()
        };
        let mut values = vec![];
        for _ in 0..count {
            values.push(short(
                r,
                match kind {
                    3 => 1,
                    1 => 2,
                    _ => 0,
                },
            )?);
        }
        c.recolor = Some(values);
    }
    if flags & 8 != 0 {
        let count = if kind == 3 {
            byte(r, 2)? as usize
        } else {
            d.retexture.len()
        };
        let mut values = vec![];
        for _ in 0..count {
            values.push(short(
                r,
                match kind {
                    3 => 3,
                    1 => 2,
                    _ => 1,
                },
            )?);
        }
        c.retexture = Some(values);
    }
    if flags & 16 != 0 {
        c.use_player = kind == 3 || byte(r, if kind == 1 { 2 } else { 0 })? == 1;
    }
    if kind == 3 && flags & 32 != 0 {
        let gender = byte(r, 1)?;
        let count = byte(r, 1)?;
        let mut kits = vec![];
        for _ in 0..count {
            kits.push(r.u16()?);
        }
        c.composition = Some((gender, kits));
    }
    Ok(Some(c))
}
pub fn decode_blocks(r: &mut Reader<'_>, d: &Definition) -> Result<Vec<Block>> {
    let mut mask = r.u8()? as u32;
    if mask & 64 != 0 {
        mask |= (r.u8()? as u32) << 8;
    }
    if mask & 2048 != 0 {
        mask |= (r.u8()? as u32) << 16;
    }
    if mask & 0x200000 != 0 {
        mask |= (r.u8()? as u32) << 24;
    }
    if mask & 0xfc000000 != 0 {
        return Err(Error::Invalid("unknown NPC extension mask"));
    }
    let mut out = vec![];
    if mask & 0x800000 != 0 {
        out.push(Block::Body(custom(r, d, 2)?));
    }
    if mask & 0x8000 != 0 {
        out.push(Block::Combat(r.u32_le()? as i32));
    }
    if mask & 0x400000 != 0 {
        let flag = byte(r, 3)?;
        let mut icons = [(-1, -1); 8];
        for i in 0..8 {
            if flag & (1 << i) != 0 {
                icons[i] = (nullable_large(r)?, r.unsigned_smart()? as i32 - 1);
            }
        }
        out.push(Block::HeadIcons((flag != 0).then_some(icons)));
    }
    if mask & 0x1000 != 0 {
        out.push(Block::Ops(byte(r, 2)?));
    }
    if mask & 0x80000 != 0 {
        let count = byte(r, 1)?;
        let mut hits = vec![];
        for _ in 0..count {
            hits.push(Hit {
                kind: r.unsigned_smart()?,
                value: r.unsigned_smart()?,
                delay: r.unsigned_smart()?,
                limit: r.unsigned_smart()?,
            });
        }
        out.push(Block::Hits(hits));
    }
    if mask & 0x200 != 0 {
        out.push(Block::ExactMove {
            offsets: [
                byte(r, 3)? as i8,
                r.u8()? as i8,
                byte(r, 1)? as i8,
                r.u8()? as i8,
            ],
            delays: [short(r, 3)?, short(r, 2)?],
            direction: short(r, 2)?,
        });
    }
    if mask & 0x2000000 != 0 {
        out.push(Block::Body(custom(r, d, 3)?));
    }
    if mask & 0x1000000 != 0 {
        let count = byte(r, 2)?;
        let mut bars = vec![];
        for _ in 0..count {
            let kind = r.unsigned_smart()?;
            let end = r.unsigned_smart()?;
            let (timing, fill) = if end == 32767 {
                (None, [0, 0])
            } else {
                let start = r.unsigned_smart()?;
                let a = byte(r, 1)?;
                let b = if end > 0 { byte(r, 3)? } else { a };
                (Some([start, end]), [a, b])
            };
            bars.push(Headbar { kind, timing, fill });
        }
        out.push(Block::Headbars(bars));
    }
    if mask & 128 != 0 {
        out.push(Block::Sequence {
            id: r.u16()?,
            delay: byte(r, 2)?,
        });
    }
    if mask & 2 != 0 {
        out.push(Block::Say(r.string()?));
    }
    if mask & 256 != 0 {
        out.push(Block::Tint {
            delays: [short(r, 1)?, short(r, 2)?],
            hsl: [r.u8()?, byte(r, 1)?, byte(r, 1)?],
            weight: byte(r, 1)?,
        });
    }
    if mask & 1024 != 0 {
        out.push(Block::Transparency {
            delays: [short(r, 3)? as i16, r.u16()? as i16],
            alpha: [byte(r, 1)? as i8, byte(r, 2)? as i8],
            use_start: byte(r, 2)? != 0,
        });
    }
    if mask & 0x100000 != 0 {
        let flags = r.u32_le()?;
        let mut animations = [None; 15];
        for (i, alt) in [0, 3, 3, 1, 0, 3, 3, 3, 2, 2, 0, 1, 2, 1, 0]
            .into_iter()
            .enumerate()
        {
            if flags & (1 << i) != 0 {
                animations[i] = Some(short(r, alt)?);
            }
        }
        out.push(Block::BaseAnimations(animations));
    }
    if mask & 32 != 0 {
        out.push(Block::Freeze {
            delay: r.u16()?,
            duration: r.u16()?,
            cancel_sequence: byte(r, 2)? != 0,
        });
    }
    for (flag, size) in [(65536, 3), (8192, 9)] {
        if mask & flag != 0 {
            out.push(Block::Discarded {
                mask: flag,
                bytes: r.bytes(size)?.to_vec(),
            });
        }
    }
    if mask & 0x20000 != 0 {
        out.push(Block::Head(custom(r, d, 1)?));
    }
    if mask & 0x40000 != 0 {
        let count = byte(r, 2)?;
        let mut spots = vec![];
        for _ in 0..count {
            let slot = r.u8()?;
            let id = r.u16()?;
            let packed = r.u32_inverse_middle()?;
            spots.push((slot, id, (packed >> 16) as u16, packed as u16));
        }
        out.push(Block::SpotAnimations(spots));
    }
    if mask & 16 != 0 {
        out.push(Block::Discarded {
            mask: 16,
            bytes: r.bytes(3)?.to_vec(),
        });
    }
    if mask & 0x4000 != 0 {
        out.push(Block::Name(r.string()?));
    }
    if mask & 1 != 0 {
        out.push(Block::Transform(short(r, 3)?));
    }
    if mask & 8 != 0 {
        out.push(Block::Face(face(r)?));
    }
    if mask & 4 != 0 {
        out.push(Block::Discarded {
            mask: 4,
            bytes: r.bytes(4)?.to_vec(),
        });
    }
    Ok(out)
}
pub fn face(r: &mut Reader<'_>) -> Result<Face> {
    let flags = byte(r, 1)?;
    face_flags(r, flags)
}
pub fn face_flags(r: &mut Reader<'_>, flags: u8) -> Result<Face> {
    let walk = flags & 7;
    if walk > 1 {
        return Err(Error::Invalid("actor facing walk mode"));
    }
    let target = match (flags >> 3) & 7 {
        0 => {
            let kind = r.unsigned_smart()?;
            if kind > 3 {
                return Err(Error::Invalid("actor facing entity kind"));
            }
            FaceTarget::Entity {
                kind,
                index: nullable_large(r)?,
                fallback: r.unsigned_smart()?,
            }
        }
        1 => {
            let x = r.unsigned_smart()?;
            let z = r.unsigned_smart()?;
            let size = r.unsigned_smart()?;
            FaceTarget::Tile {
                x,
                z,
                size: [(size & 15) as u8, (size >> 4 & 15) as u8],
            }
        }
        2 => FaceTarget::Angle(r.unsigned_smart()?),
        3 => FaceTarget::Reset,
        _ => return Err(Error::Invalid("actor facing kind")),
    };
    Ok(Face {
        target,
        turn_on_walk: walk == 1,
        instant: flags & 64 != 0,
    })
}
