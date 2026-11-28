//! Revision 239 hit-splat and health-bar definitions (px, GameHealthBarConfig).
use crate::{buffer::Reader, disk_cache::DiskCache, Error, Result};
use std::collections::HashMap;
fn resource(r: &mut Reader<'_>) -> Result<i32> {
    let wide = r.clone().u8()? & 128 != 0;
    let id = r.large_smart()?;
    Ok(if !wide && id == 32767 { -1 } else { id as i32 })
}
fn nullable(r: &mut Reader<'_>) -> Result<i32> {
    let id = r.u16()?;
    Ok(if id == 65535 { -1 } else { id as i32 })
}
#[derive(Clone, Debug)]
pub struct HealthBar {
    pub id: u32,
    pub order: u8,
    pub priority: u8,
    pub fade: i32,
    pub step: i32,
    pub duration: u16,
    pub sprites: [i32; 2],
    pub width: u8,
    pub padding: u8,
}
impl HealthBar {
    pub fn decode(id: u32, data: &[u8]) -> Result<Self> {
        let mut d = Self {
            id,
            order: 255,
            priority: 255,
            fade: -1,
            step: 1,
            duration: 70,
            sprites: [-1; 2],
            width: 30,
            padding: 0,
        };
        let mut r = Reader::new(data);
        loop {
            match r.u8()? {
                0 => break,
                1 => {
                    r.u16()?;
                }
                2 => d.order = r.u8()?,
                3 => d.priority = r.u8()?,
                4 => d.fade = 0,
                5 => d.duration = r.u16()?,
                6 => {
                    r.u8()?;
                }
                7 => d.sprites[0] = resource(&mut r)?,
                8 => d.sprites[1] = resource(&mut r)?,
                11 => d.fade = r.u16()? as i32,
                14 => d.width = r.u8()?,
                15 => d.padding = r.u8()?,
                _ => return Err(Error::Invalid("health bar opcode")),
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("health bar trailing bytes"));
        }
        Ok(d)
    }
    pub fn load_all(cache: &mut DiskCache) -> Result<HashMap<u32, Self>> {
        cache
            .group(2, 33, [0; 4])?
            .into_iter()
            .map(|(id, bytes)| Ok((id, Self::decode(id, &bytes)?)))
            .collect()
    }
}
#[derive(Clone, Debug)]
pub struct Hitsplat {
    pub id: u32,
    pub font: i32,
    pub color: u32,
    pub duration: u16,
    pub sprites: [i32; 4],
    pub offset: [i16; 2],
    pub fade: i32,
    pub text: String,
    pub policy: i32,
    pub baseline: i16,
    pub varbit: i32,
    pub varp: i32,
    pub transforms: Option<Vec<i32>>,
}
impl Hitsplat {
    pub fn decode(id: u32, data: &[u8]) -> Result<Self> {
        let mut d = Self {
            id,
            font: -1,
            color: 0xffffff,
            duration: 70,
            sprites: [-1; 4],
            offset: [0; 2],
            fade: -1,
            text: String::new(),
            policy: -1,
            baseline: 0,
            varbit: -1,
            varp: -1,
            transforms: None,
        };
        let mut r = Reader::new(data);
        loop {
            match r.u8()? {
                0 => break,
                1 => d.font = resource(&mut r)?,
                2 => d.color = r.u24()?,
                op @ 3..=6 => {
                    d.sprites[match op {
                        3 => 0,
                        4 => 2,
                        5 => 1,
                        _ => 3,
                    }] = resource(&mut r)?
                }
                7 => d.offset[0] = r.u16()? as i16,
                8 => {
                    if r.u8()? != 0 {
                        return Err(Error::Invalid("hitsplat string prefix"));
                    }
                    d.text = r.string()?;
                }
                9 => d.duration = r.u16()?,
                10 => d.offset[1] = r.u16()? as i16,
                11 => d.fade = 0,
                12 => d.policy = r.u8()? as i32,
                13 => d.baseline = r.u16()? as i16,
                14 => d.fade = r.u16()? as i32,
                op @ (17 | 18) => {
                    d.varbit = nullable(&mut r)?;
                    d.varp = nullable(&mut r)?;
                    let fallback = if op == 18 { nullable(&mut r)? } else { -1 };
                    let count = r.u8()?;
                    let mut values = Vec::new();
                    for _ in 0..=count {
                        values.push(nullable(&mut r)?);
                    }
                    values.push(fallback);
                    d.transforms = Some(values);
                }
                _ => return Err(Error::Invalid("hitsplat opcode")),
            }
        }
        if r.remaining() != 0 {
            return Err(Error::Invalid("hitsplat trailing bytes"));
        }
        Ok(d)
    }
    pub fn load_all(cache: &mut DiskCache) -> Result<HashMap<u32, Self>> {
        cache
            .group(2, 32, [0; 4])?
            .into_iter()
            .map(|(id, bytes)| Ok((id, Self::decode(id, &bytes)?)))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Hit {
    pub kind: i32,
    pub value: i32,
    pub end: i32,
}
#[derive(Clone, Debug, Default)]
pub struct Hits {
    pub slots: Vec<Hit>,
}
impl Hits {
    /// The definition is already resolved through its varbit/varp transform.
    pub fn add(
        &mut self,
        kind: i32,
        value: i32,
        cycle: i32,
        delay: i32,
        limit: usize,
        definition: Option<&Hitsplat>,
    ) {
        if kind < 0 || limit == 0 {
            return;
        }
        let Some(def) = definition else {
            return;
        };
        let mut full = self.slots.len() >= limit;
        let mut next = 0;
        for (i, hit) in self.slots.iter().enumerate() {
            if hit.end > cycle {
                next = (i + 1) % limit;
            } else {
                full = false;
            }
        }
        let mut selected = None;
        if full {
            if def.policy == -1 {
                return;
            }
            let mut minimum = 0;
            for (i, hit) in self.slots.iter().enumerate() {
                let n = if def.policy == 0 {
                    hit.end
                } else if def.policy == 1 {
                    hit.value
                } else {
                    continue;
                };
                if i == 0 || n < minimum {
                    minimum = n;
                    selected = Some(i);
                }
            }
            if def.policy == 1 && minimum >= value {
                return;
            }
        } else {
            if limit > 4 {
                next = 0;
            }
            for _ in 0..limit {
                let slot = next;
                next = (next + 1) % limit;
                if slot >= self.slots.len() {
                    self.slots.push(Hit {
                        kind: -1,
                        value: 0,
                        end: 0,
                    });
                    selected = Some(self.slots.len() - 1);
                    break;
                }
                if self.slots[slot].end <= cycle {
                    selected = Some(slot);
                    break;
                }
            }
        }
        if let Some(slot) = selected {
            self.slots[slot] = Hit {
                kind,
                value,
                end: cycle.wrapping_add(def.duration as i32).wrapping_add(delay),
            };
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BarUpdate {
    pub start: i32,
    pub fill: [i32; 2],
    pub interpolation: i32,
}
#[derive(Clone, Debug)]
pub struct Bar {
    pub definition: HealthBar,
    /// Original deque iteration order, oldest update first.
    pub updates: Vec<BarUpdate>,
}
impl Bar {
    pub fn add(&mut self, update: BarUpdate) {
        if let Some(old) = self
            .updates
            .iter_mut()
            .find(|old| old.start == update.start)
        {
            *old = update;
            return;
        }
        let insertion = self
            .updates
            .iter()
            .rposition(|old| old.start <= update.start);
        if let Some(index) = insertion {
            self.updates.insert(index + 1, update);
            if self.updates.len() > 4 {
                self.updates.remove(0);
            }
        } else if self.updates.len() < 4 {
            self.updates.insert(0, update);
        }
    }
    /// Selecting a visible update consumes superseded/expired entries, like ch.af.
    pub fn current(&mut self, cycle: i32) -> Option<&BarUpdate> {
        if self.updates.first()?.start > cycle {
            return None;
        }
        while self.updates.get(1).is_some_and(|next| next.start <= cycle) {
            self.updates.remove(0);
        }
        let first = &self.updates[0];
        if first
            .start
            .wrapping_add(first.interpolation)
            .wrapping_add(self.definition.duration as i32)
            > cycle
        {
            Some(&self.updates[0])
        } else {
            self.updates.remove(0);
            None
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Bars {
    pub entries: Vec<Bar>,
}
impl Bars {
    pub fn add(&mut self, definition: &HealthBar, update: BarUpdate) {
        if let Some(bar) = self
            .entries
            .iter_mut()
            .find(|bar| bar.definition.id == definition.id)
        {
            bar.add(update);
            return;
        }
        let mut eviction = None;
        let mut priority = definition.priority;
        let mut insertion = 0;
        for (index, bar) in self.entries.iter().enumerate() {
            if bar.definition.order <= definition.order {
                insertion = index + 1;
            }
            if bar.definition.priority > priority {
                priority = bar.definition.priority;
                eviction = Some(index);
            }
        }
        let full = self.entries.len() >= 4;
        if full && eviction.is_none() {
            return;
        }
        let mut bar = Bar {
            definition: definition.clone(),
            updates: Vec::new(),
        };
        bar.add(update);
        self.entries.insert(insertion, bar);
        if full {
            let index = eviction.unwrap();
            self.entries.remove(index + usize::from(index >= insertion));
        }
    }
    pub fn remove(&mut self, id: u32) {
        self.entries.retain(|bar| bar.definition.id != id);
    }
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub speech: crate::overhead::Speech,
    pub hits: Hits,
    pub bars: Bars,
}
pub struct Definitions {
    pub hits: HashMap<u32, Hitsplat>,
    pub bars: HashMap<u32, HealthBar>,
}
impl Definitions {
    pub fn load(cache: &mut DiskCache) -> Result<Self> {
        Ok(Self {
            hits: Hitsplat::load_all(cache)?,
            bars: HealthBar::load_all(cache)?,
        })
    }
    pub fn hit(
        &self,
        id: i32,
        mut variable: impl FnMut(bool, i32) -> Result<i32>,
    ) -> Result<Option<&Hitsplat>> {
        if id < 0 {
            return Ok(None);
        }
        let def = self
            .hits
            .get(&(id as u32))
            .ok_or(Error::Invalid("hit-splat definition missing"))?;
        let Some(transforms) = &def.transforms else {
            return Ok(Some(def));
        };
        let value = if def.varbit >= 0 {
            variable(true, def.varbit)?
        } else if def.varp >= 0 {
            variable(false, def.varp)?
        } else {
            -1
        };
        let next = if value >= 0 && (value as usize) < transforms.len() - 1 {
            transforms[value as usize]
        } else {
            *transforms.last().unwrap()
        };
        if next < 0 {
            Ok(None)
        } else {
            Ok(Some(self.hits.get(&(next as u32)).ok_or(
                Error::Invalid("transformed hit-splat definition missing"),
            )?))
        }
    }
}
impl State {
    pub fn apply(
        &mut self,
        block: &crate::actor_update::Block,
        cycle: i32,
        definitions: &Definitions,
        mut variable: impl FnMut(bool, i32) -> Result<i32>,
    ) -> Result<()> {
        use crate::actor_update::Block;
        match block {
            Block::Hits(hits) => {
                for hit in hits {
                    let kind = if hit.kind == 65535 {
                        -1
                    } else {
                        hit.kind as i32
                    };
                    let def = definitions.hit(kind, &mut variable)?;
                    self.hits.add(
                        kind,
                        hit.value as i32,
                        cycle,
                        hit.delay as i32,
                        hit.limit as usize,
                        def,
                    );
                }
            }
            Block::Headbars(bars) => {
                for bar in bars {
                    if let Some([delay, interpolation]) = bar.timing {
                        let definition = definitions
                            .bars
                            .get(&(bar.kind as u32))
                            .ok_or(Error::Invalid("health-bar definition missing"))?;
                        self.bars.add(
                            definition,
                            BarUpdate {
                                start: cycle.wrapping_add(delay as i32),
                                fill: bar.fill.map(i32::from),
                                interpolation: interpolation as i32,
                            },
                        );
                    } else {
                        self.bars.remove(bar.kind as u32);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}
