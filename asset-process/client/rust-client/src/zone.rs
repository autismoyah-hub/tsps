//! Zone envelopes and ground-item mutations from GameClient.ib/GameSceneTilePaint.dc.
use crate::{
    buffer::{Reader, Transform},
    item::Definitions,
    Error, Result,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Custom {
    pub model: u32,
    pub recolor: Vec<u16>,
    pub retexture: Vec<u16>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    pub id: u16,
    pub quantity: i32,
    pub operations: u8,
    pub visible: i32,
    pub despawn: i32,
    pub ownership: u8,
    pub private: bool,
    pub custom: Option<Custom>,
}
#[derive(Clone, Debug)]
pub enum Record {
    Customise {
        at: u8,
        id: u16,
        quantity: i32,
        model: u16,
        color_index: i16,
        color: u16,
        texture_index: i16,
        texture: u16,
    },
    Add {
        at: u8,
        item: Item,
    },
    Quantity {
        at: u8,
        id: u16,
        old: i32,
        new: i32,
    },
    Remove {
        at: u8,
        id: u16,
        quantity: i32,
    },
    Operations {
        at: u8,
        id: u16,
        operations: u8,
    },
    Uncustomise {
        at: u8,
        id: u16,
        quantity: i32,
    },
    Pending {
        kind: u8,
        bytes: Vec<u8>,
    },
}
#[derive(Clone, Debug)]
pub struct Packet {
    pub absolute: Option<[i32; 3]>,
    pub origin: Option<[i32; 3]>,
    pub clear: bool,
    pub records: Vec<Record>,
}
impl Record {
    pub fn decode(kind: u8, r: &mut Reader<'_>) -> Result<Self> {
        Ok(match kind {
            3 => {
                let model = r.u16()?;
                let color_index = r.u16_le()? as i16;
                let id = r.u16_le()?;
                let at = r.transformed(Transform::Sub)?;
                let texture = r.u16_add(false)?;
                let quantity = r.u32_middle()? as i32;
                let color = r.u16_add(true)?;
                let texture_index = r.u16_add(false)? as i16;
                Self::Customise {
                    at,
                    id,
                    quantity,
                    model,
                    color_index,
                    color,
                    texture_index,
                    texture,
                }
            }
            8 => {
                let at = r.transformed(Transform::Add)?;
                let visible = r.u16()? as i32;
                let ownership = r.u8()?;
                let id = r.u16_add(false)?;
                let private = r.u8()? == 1;
                let operations = r.transformed(Transform::Neg)?;
                let despawn = r.u16_add(true)? as i32;
                let quantity = r.u32_le()? as i32;
                Self::Add {
                    at,
                    item: Item {
                        id,
                        quantity,
                        operations,
                        visible,
                        despawn,
                        ownership,
                        private,
                        custom: None,
                    },
                }
            }
            4 => {
                let old = r.u32_middle()? as i32;
                let at = r.transformed(Transform::Add)?;
                let new = r.u32_le()? as i32;
                let id = r.u16_add(true)?;
                Self::Quantity { at, id, old, new }
            }
            10 => {
                let id = r.u16()?;
                let at = r.u8()?;
                let quantity = r.u32_inverse_middle()? as i32;
                Self::Remove { at, id, quantity }
            }
            11 => {
                let at = r.transformed(Transform::Add)?;
                let id = r.u16_add(true)?;
                let operations = r.u8()?;
                Self::Operations { at, id, operations }
            }
            6 => {
                let id = r.u16()?;
                let quantity = r.u32()? as i32;
                let at = r.transformed(Transform::Add)?;
                Self::Uncustomise { at, id, quantity }
            }
            7 => {
                // This variable-length loc-add record has optional operation labels.
                let mut bytes = Vec::new();
                let count = r.transformed(Transform::Add)?;
                bytes.push(count.wrapping_add(128));
                for _ in 0..count {
                    bytes.push(r.u8()?);
                    loop {
                        let b = r.u8()?;
                        bytes.push(b);
                        if b == 0 {
                            break;
                        }
                    }
                }
                bytes.extend_from_slice(r.bytes(5)?);
                Self::Pending { kind, bytes }
            }
            _ => {
                let size = match kind {
                    0 => 2,
                    1 => 18,
                    2 => 14,
                    3 => 17,
                    5 => 6,
                    9 => 4,
                    12 => 37,
                    13 => 24,
                    14 => 7,
                    _ => return Err(Error::Invalid("zone record kind")),
                };
                Self::Pending {
                    kind,
                    bytes: r.bytes(size)?.to_vec(),
                }
            }
        })
    }
}
impl Packet {
    pub fn decode(opcode: u16, raw: &[u8]) -> Result<Self> {
        let mut r = Reader::new(raw);
        let mut records = Vec::new();
        let mut clear = false;
        let mut absolute = None;
        let origin = match opcode {
            10 | 33 | 34 | 86 | 110 => {
                let kind = match opcode {
                    10 => 0,
                    33 => 7,
                    34 => 2,
                    86 => 5,
                    _ => 9,
                };
                records.push(Record::decode(kind, &mut r)?);
                None
            }
            54 => {
                let y = r.transformed(Transform::Sub)?;
                let x = r.transformed(Transform::Neg)?;
                let plane = r.u8()?;
                clear = true;
                Some([plane as i32, x as i32, y as i32])
            }
            105 => {
                let plane = r.transformed(Transform::Neg)?;
                let y = r.transformed(Transform::Add)?;
                let x = r.transformed(Transform::Add)?;
                while r.remaining() > 0 {
                    let kind = r.u8()?;
                    records.push(Record::decode(kind, &mut r)?);
                }
                Some([plane as i32, x as i32, y as i32])
            }
            9 => {
                let y = r.transformed(Transform::Add)?;
                let x = r.u8()?;
                let plane = r.transformed(Transform::Neg)?;
                Some([plane as i32, x as i32, y as i32])
            }
            13 | 41 | 52 | 70 | 78 | 104 => {
                let (coordinate, record) = match opcode {
                    78 => {
                        let visible = r.u16()? as i32;
                        let id = r.u16_le()?;
                        let despawn = r.u16_le()? as i32;
                        let private = r.transformed(Transform::Sub)? == 1;
                        let ownership = r.u8()?;
                        let operations = r.transformed(Transform::Neg)?;
                        let coord = r.u32_inverse_middle()?;
                        let quantity = r.u32_middle()? as i32;
                        (
                            coord,
                            Record::Add {
                                at: 0,
                                item: Item {
                                    id,
                                    quantity,
                                    operations,
                                    visible,
                                    despawn,
                                    ownership,
                                    private,
                                    custom: None,
                                },
                            },
                        )
                    }
                    13 => {
                        let old = r.u32_le()? as i32;
                        let new = r.u32()? as i32;
                        let coord = r.u32()?;
                        let id = r.u16()?;
                        (
                            coord,
                            Record::Quantity {
                                at: 0,
                                id,
                                old,
                                new,
                            },
                        )
                    }
                    41 => {
                        let id = r.u16_add(false)?;
                        let quantity = r.u32()? as i32;
                        let coord = r.u32()?;
                        (
                            coord,
                            Record::Remove {
                                at: 0,
                                id,
                                quantity,
                            },
                        )
                    }
                    70 => {
                        let id = r.u16_add(false)?;
                        let operations = r.transformed(Transform::Sub)?;
                        let coord = r.u32_inverse_middle()?;
                        (
                            coord,
                            Record::Operations {
                                at: 0,
                                id,
                                operations,
                            },
                        )
                    }
                    104 => {
                        let coord = r.u32()?;
                        let quantity = r.u32_middle()? as i32;
                        let id = r.u16()?;
                        (
                            coord,
                            Record::Uncustomise {
                                at: 0,
                                id,
                                quantity,
                            },
                        )
                    }
                    52 => {
                        let model = r.u16_add(false)?;
                        let texture = r.u16()?;
                        let texture_index = r.u16_add(false)? as i16;
                        let quantity = r.u32_middle()? as i32;
                        let coord = r.u32()?;
                        let color = r.u16_add(false)?;
                        let id = r.u16()?;
                        let color_index = r.u16_add(true)? as i16;
                        (
                            coord,
                            Record::Customise {
                                at: 0,
                                id,
                                quantity,
                                model,
                                color_index,
                                color,
                                texture_index,
                                texture,
                            },
                        )
                    }
                    _ => unreachable!(),
                };
                absolute = Some([
                    (coordinate as i32) >> 28,
                    ((coordinate >> 14) & 16383) as i32,
                    (coordinate & 16383) as i32,
                ]);
                records.push(record);
                None
            }
            _ => return Err(Error::Invalid("zone envelope opcode")),
        };
        if r.remaining() != 0 {
            return Err(Error::Invalid("zone trailing bytes"));
        }
        Ok(Self {
            absolute,
            origin,
            clear,
            records,
        })
    }
}
#[derive(Clone, Debug, Default)]
pub struct State {
    pub origin: [i32; 3],
    /// Absolute coordinates keep piles stable when the scene base changes.
    pub piles: BTreeMap<[i32; 3], Vec<Item>>,
    /// Logical tiles allocated by item mutations since the owning scene rebuild.
    /// Removal and zone clear retain these, including add/remove between paints.
    pub scene_allocations: BTreeSet<[i32; 3]>,
    pub pending: VecDeque<([i32; 3], Record)>,
    pub locations: VecDeque<crate::location_update::Event>,
    pub sounds: VecDeque<crate::area_sound::Request>,
    pub graphics: VecDeque<crate::world_spot::Request>,
    pub projectiles: VecDeque<crate::projectile::Request>,
    pub scripted_projectiles: VecDeque<crate::scripted_projectile::Event>,
}
impl State {
    pub fn apply(
        &mut self,
        packet: Packet,
        base: [i32; 2],
        plane: i32,
        tick: i32,
        defs: &Definitions,
    ) -> Result<()> {
        self.apply_in_view(packet, base, plane, tick, defs, [104, 104])
    }
    pub fn apply_in_view(
        &mut self,
        packet: Packet,
        base: [i32; 2],
        plane: i32,
        tick: i32,
        defs: &Definitions,
        dimensions: [i32; 2],
    ) -> Result<()> {
        if let Some(origin) = packet.origin {
            self.origin = origin;
        }
        let local_origin = packet
            .absolute
            .map_or(self.origin, |p| [p[0], p[1] - base[0], p[2] - base[1]]);
        if packet.absolute.is_some() && !(0..4).contains(&local_origin[0]) {
            return Ok(());
        }
        let level = if local_origin[0] == -1 {
            plane
        } else {
            local_origin[0]
        };
        if !(0..4).contains(&level) {
            return Err(Error::Invalid("zone plane"));
        }
        let origin = [level, base[0] + local_origin[1], base[1] + local_origin[2]];
        if packet.clear {
            self.piles.retain(|p, _| {
                p[0] != level
                    || p[1] < origin[1]
                    || p[1] >= origin[1] + 8
                    || p[2] < origin[2]
                    || p[2] >= origin[2] + 8
            });
            self.locations
                .push_back(crate::location_update::Event::Clear(origin));
        }
        let mut unsupported = false;
        for record in packet.records {
            let at = match &record {
                Record::Add { at, .. }
                | Record::Quantity { at, .. }
                | Record::Remove { at, .. }
                | Record::Operations { at, .. }
                | Record::Uncustomise { at, .. }
                | Record::Customise { at, .. } => *at,
                Record::Pending { kind: 13, bytes } => {
                    let request = crate::projectile::Request::decode(origin, 0, bytes)?;
                    if request.id != 65535 {
                        if self.projectiles.len() >= 16384 {
                            return Err(Error::Invalid("projectile queue limit"));
                        }
                        self.projectiles.push_back(request);
                    }
                    continue;
                }
                Record::Pending { kind, bytes } if matches!(kind, 1 | 12) => {
                    let event =
                        crate::scripted_projectile::Event::decode(*kind, origin, plane, bytes)?;
                    if self.scripted_projectiles.len() >= 16384 {
                        return Err(Error::Invalid("scripted projectile queue limit"));
                    }
                    self.scripted_projectiles.push_back(event);
                    continue;
                }
                Record::Pending { kind: 9, bytes } => {
                    let request = crate::location_animation::Request::decode(bytes)?;
                    let key = request.key(origin)?;
                    if (0..dimensions[0] - 1).contains(&(key[1] - base[0]))
                        && (0..dimensions[1] - 1).contains(&(key[2] - base[1]))
                    {
                        if self.locations.len() >= 16384 {
                            return Err(Error::Invalid("pending location limit"));
                        }
                        self.locations
                            .push_back(crate::location_update::Event::Animate(origin, request));
                    }
                    continue;
                }
                Record::Pending { kind: 2, bytes } => {
                    let request = crate::location_merge::Request::decode(bytes)?;
                    if self.locations.len() >= 16384 {
                        return Err(Error::Invalid("pending location limit"));
                    }
                    self.locations
                        .push_back(crate::location_update::Event::Merge(origin, request));
                    continue;
                }
                Record::Pending { kind: 5, bytes } => {
                    let request = crate::world_spot::Request::decode(origin, bytes)?;
                    if (0..dimensions[0]).contains(&(request.position[1] - base[0]))
                        && (0..dimensions[1]).contains(&(request.position[2] - base[1]))
                    {
                        if self.graphics.len() >= 16384 {
                            return Err(Error::Invalid("world graphics queue limit"));
                        }
                        self.graphics.push_back(request);
                    }
                    continue;
                }
                Record::Pending { kind: 14, bytes } => {
                    let sound = crate::area_sound::Request::decode(local_origin, base, bytes)?;
                    if (0..dimensions[0]).contains(&(sound.tile[0] - base[0]))
                        && (0..dimensions[1]).contains(&(sound.tile[1] - base[1]))
                    {
                        if self.sounds.len() >= 16384 {
                            return Err(Error::Invalid("pending area sound limit"));
                        }
                        self.sounds.push_back(sound);
                    }
                    continue;
                }
                Record::Pending { kind, bytes } if matches!(kind, 0 | 7) => {
                    let change = crate::location_update::Change::decode(*kind, bytes)?;
                    let x = local_origin[1] + ((change.at >> 4) & 7) as i32;
                    let y = local_origin[2] + (change.at & 7) as i32;
                    if (0..dimensions[0]).contains(&x) && (0..dimensions[1]).contains(&y) {
                        if self.locations.len() >= 16384 {
                            return Err(Error::Invalid("pending location limit"));
                        }
                        self.locations
                            .push_back(crate::location_update::Event::Change(origin, change));
                    }
                    continue;
                }
                Record::Pending { .. } => {
                    if self.pending.len() >= 16384 {
                        return Err(Error::Invalid("pending zone record limit"));
                    }
                    self.pending.push_back((origin, record));
                    unsupported = true;
                    continue;
                }
            };
            let x = local_origin[1] + ((at >> 4) & 7) as i32;
            let y = local_origin[2] + (at & 7) as i32;
            if !(0..dimensions[0]).contains(&x) || !(0..dimensions[1]).contains(&y) {
                continue;
            }
            let key = [level, base[0] + x, base[1] + y];
            let mut reorder = true;
            match record {
                Record::Add { mut item, .. } => {
                    item.visible = item.visible.wrapping_add(tick);
                    item.despawn = item.despawn.wrapping_add(tick);
                    self.piles.entry(key).or_default().push(item);
                }
                Record::Quantity { id, old, new, .. } => {
                    if let Some(pile) = self.piles.get_mut(&key) {
                        if let Some(item) =
                            pile.iter_mut().find(|i| i.id == id && i.quantity == old)
                        {
                            item.quantity = new;
                        }
                    }
                }
                Record::Remove { id, quantity, .. } => {
                    if let Some(pile) = self.piles.get_mut(&key) {
                        if let Some(index) = pile
                            .iter()
                            .position(|i| i.id == id && i.quantity == quantity)
                        {
                            pile.remove(index);
                        }
                        if pile.is_empty() {
                            self.piles.remove(&key);
                        }
                    }
                }
                Record::Operations { id, operations, .. } => {
                    if let Some(pile) = self.piles.get_mut(&key) {
                        if let Some(item) = pile.iter_mut().find(|i| i.id == id) {
                            item.operations = operations;
                        }
                    }
                    reorder = false;
                }
                Record::Uncustomise { id, quantity, .. } => {
                    if let Some(pile) = self.piles.get_mut(&key) {
                        if let Some(item) = pile
                            .iter_mut()
                            .find(|i| i.id == id && i.quantity == quantity)
                        {
                            item.custom = None;
                        }
                    }
                }
                Record::Customise {
                    id,
                    quantity,
                    model,
                    color_index,
                    color,
                    texture_index,
                    texture,
                    ..
                } => {
                    if let Some(pile) = self.piles.get_mut(&key) {
                        if let Some(item) = pile
                            .iter_mut()
                            .find(|i| i.id == id && i.quantity == quantity)
                        {
                            let def = defs
                                .items
                                .get(&(id as u32))
                                .ok_or(Error::Invalid("ground customisation definition"))?;
                            let custom = item.custom.get_or_insert_with(|| Custom {
                                model: model as u32,
                                recolor: def.recolor.iter().map(|v| v.1).collect(),
                                retexture: def.retexture.iter().map(|v| v.1).collect(),
                            });
                            if let Some(v) = custom.recolor.get_mut(color_index as usize) {
                                *v = color;
                            }
                            if let Some(v) = custom.retexture.get_mut(texture_index as usize) {
                                *v = texture;
                            }
                            custom.model = model as u32;
                        }
                    }
                }
                Record::Pending { .. } => unreachable!(),
            }
            if reorder {
                if let Some(pile) = self.piles.get_mut(&key) {
                    if Self::reorder(pile, defs)? {
                        self.scene_allocations.insert(key);
                    }
                }
            }
        }
        if unsupported {
            return Err(Error::Unsupported(
                "zone object/effect records queued for implementation",
            ));
        }
        Ok(())
    }
    fn reorder(pile: &mut Vec<Item>, defs: &Definitions) -> Result<bool> {
        let mut best = -99999999i64;
        let mut top = None;
        for (index, item) in pile.iter().enumerate() {
            let def = defs
                .items
                .get(&(item.id as u32))
                .ok_or(Error::Invalid("ground item definition"))?;
            let mut value = def.price as i64;
            if def.stackable == 1 {
                value *= if item.quantity < i32::MAX {
                    item.quantity.wrapping_add(1)
                } else {
                    item.quantity
                } as i64;
            }
            if value > best {
                best = value;
                top = Some(index);
            }
        }
        if let Some(index) = top {
            let item = pile.remove(index);
            pile.insert(0, item);
        }
        Ok(top.is_some())
    }
}
