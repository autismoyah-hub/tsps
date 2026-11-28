//! Cache-backed script lookup and client variables used by the interpreter.
use crate::{
    buffer::Reader,
    disk_cache::DiskCache,
    script::Script,
    script_math,
    vm::{Control, Host, Object, Stacks, Value, Variable},
    Error, Result,
};
use std::{
    collections::{HashMap, VecDeque},
    rc::Rc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Varbit {
    pub base: u16,
    pub low: u8,
    pub high: u8,
}
impl Varbit {
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(data);
        let mut v = Self {
            base: 0,
            low: 0,
            high: 0,
        };
        loop {
            match r.u8()? {
                0 => break,
                1 => {
                    v.base = r.u16()?;
                    v.low = r.u8()?;
                    v.high = r.u8()?;
                }
                _ => return Err(Error::Unsupported("varbit definition opcode")),
            }
        }
        if r.remaining() != 0 || v.low > v.high || v.high > 31 {
            return Err(Error::Invalid("varbit definition"));
        }
        Ok(v)
    }
    fn mask(self) -> i32 {
        (u32::MAX >> (31 - (self.high - self.low))) as i32
    }
    pub fn get(self, value: i32) -> i32 {
        (value >> self.low) & self.mask()
    }
    pub fn set(self, base: i32, value: i32) -> i32 {
        let mask = self.mask();
        let value = if value < 0 || value > mask { 0 } else { value };
        let shifted = mask.wrapping_shl(self.low as u32);
        (base & !shifted) | (value.wrapping_shl(self.low as u32) & shifted)
    }
}
pub trait Source {
    fn script(&mut self, id: i32) -> Result<Rc<Script>>;
    fn varbit(&mut self, id: i32) -> Result<Varbit>;
}
pub struct CacheSource {
    pub cache: DiskCache,
    scripts: HashMap<i32, Rc<Script>>,
    order: VecDeque<i32>,
    varbits: HashMap<i32, Varbit>,
    events: HashMap<i32, Option<u32>>,
}
impl CacheSource {
    pub fn new(cache: DiskCache) -> Self {
        Self {
            cache,
            scripts: HashMap::new(),
            order: VecDeque::new(),
            varbits: HashMap::new(),
            events: HashMap::new(),
        }
    }
    pub fn event_script(
        &mut self,
        kind: i32,
        key: i32,
        category: i32,
    ) -> Result<Option<Rc<Script>>> {
        for name in [
            kind.wrapping_add(key.wrapping_shl(8)),
            kind.wrapping_add((-3i32).wrapping_sub(category).wrapping_shl(8)),
            kind.wrapping_sub(512),
        ] {
            // The original shifts in 32 bits before widening to the shared cache key.
            let cache_key = name.wrapping_shl(16);
            if let Some(script) = self.cached(cache_key) {
                return Ok(Some(script));
            }
            let group = if let Some(group) = self.events.get(&name) {
                *group
            } else {
                let hash = crate::disk_cache::name_hash(&name.to_string());
                let group = self
                    .cache
                    .reference(12)?
                    .groups
                    .iter()
                    .find(|g| g.name_hash == Some(hash))
                    .map(|g| g.id);
                self.events.insert(name, group);
                group
            };
            let Some(group) = group else {
                continue;
            };
            let index = self.cache.reference(12)?;
            let single_group = index.groups.len() == 1 && index.groups[0].id == 0;
            let (archive_group, file) = if single_group {
                (0, group)
            } else {
                let info = index.groups.iter().find(|g| g.id == group).unwrap();
                if info.files.len() != 1 || info.files[0].id != 0 {
                    return Err(Error::Invalid("non-flat event script group"));
                }
                (group, 0)
            };
            let bytes = self.cache.file(12, archive_group, file, [0; 4])?;
            if bytes.len() <= 1 {
                continue;
            }
            let script = Rc::new(Script::decode(&bytes, 1_000_000)?);
            self.remember(cache_key, script.clone());
            return Ok(Some(script));
        }
        Ok(None)
    }
    fn cached(&mut self, key: i32) -> Option<Rc<Script>> {
        let script = self.scripts.get(&key)?.clone();
        self.order.retain(|&n| n != key);
        self.order.push_back(key);
        Some(script)
    }
    fn remember(&mut self, key: i32, script: Rc<Script>) {
        if self.order.len() == 128 {
            let old = self.order.pop_front().unwrap();
            self.scripts.remove(&old);
        }
        self.order.push_back(key);
        self.scripts.insert(key, script);
    }
}
impl Source for CacheSource {
    fn script(&mut self, id: i32) -> Result<Rc<Script>> {
        if let Some(script) = self.cached(id) {
            return Ok(script);
        }
        if id < 0 {
            return Err(Error::Invalid("negative script id"));
        }
        let script = Rc::new(Script::decode(
            &self.cache.file(12, id as u32, 0, [0; 4])?,
            1_000_000,
        )?);
        self.remember(id, script.clone());
        Ok(script)
    }

    fn varbit(&mut self, id: i32) -> Result<Varbit> {
        if id < 0 {
            return Err(Error::Invalid("negative varbit id"));
        }
        if let Some(v) = self.varbits.get(&id) {
            return Ok(*v);
        }
        let varbit = Varbit::decode(&self.cache.file(2, 14, id as u32, [0; 4])?)?;
        // Definitions are tiny; bound misses against malicious script operands.
        if self.varbits.len() >= 65536 {
            self.varbits.clear();
        }
        self.varbits.insert(id, varbit);
        Ok(varbit)
    }
}
pub struct Variables {
    player: Vec<i32>,
    server: Vec<i32>,
    client: HashMap<i32, Value>,
    persistent: Vec<bool>,
    varcs_dirty: bool,
    varcs_saved: Option<std::time::Duration>,
    pub clan_settings: HashMap<i32, i32>,
    pub clan: HashMap<i32, i32>,
    changed: VecDeque<u16>,
    pub generation: u64,
}
/// Revision 239 og(BinaryBuffer), including its ignored tags and tag-3 u16.
pub fn varc_persistent(bytes: &[u8]) -> Result<bool> {
    let mut r = Reader::new(bytes);
    let mut persistent = false;
    loop {
        match r.u8()? {
            0 => return Ok(persistent),
            2 => persistent = true,
            3 => {
                r.u16()?;
            }
            _ => {}
        }
    }
}
impl Default for Variables {
    fn default() -> Self {
        Self {
            player: vec![0; 65536],
            server: vec![0; 65536],
            client: HashMap::new(),
            persistent: Vec::new(),
            varcs_dirty: false,
            varcs_saved: None,
            clan_settings: HashMap::new(),
            clan: HashMap::new(),
            changed: VecDeque::new(),
            generation: 0,
        }
    }
}
impl Variables {
    pub fn set_persistence(&mut self, persistent: Vec<bool>) {
        self.persistent = persistent;
    }
    pub fn load_varcs(&mut self, bytes: &[u8]) -> Result<()> {
        let result = crate::saved_varcs::read(bytes, &self.persistent, &mut self.client);
        self.varcs_dirty = false;
        result
    }
    pub fn encode_varcs(&self) -> Result<Vec<u8>> {
        crate::saved_varcs::encode(&self.persistent, &self.client)
    }
    pub fn varcs_dirty(&self) -> bool {
        self.varcs_dirty
    }
    pub fn varcs_due(&self, now: std::time::Duration) -> bool {
        self.varcs_dirty
            && self
                .varcs_saved
                .is_none_or(|last| now.saturating_sub(last) > std::time::Duration::from_secs(60))
    }
    pub fn varcs_saved(&mut self, now: std::time::Duration) {
        self.varcs_dirty = false;
        self.varcs_saved = Some(now);
    }
    /// dl.br removes only defined transient varcs; out-of-range keys survive.
    pub fn take_for_login(&mut self, persistent: &[bool]) -> Self {
        let mut client = std::mem::take(&mut self.client);
        client.retain(|&id, _| id < 0 || persistent.get(id as usize).copied().unwrap_or(true));
        Self {
            client,
            persistent: persistent.to_vec(),
            varcs_dirty: self.varcs_dirty,
            varcs_saved: self.varcs_saved,
            ..Default::default()
        }
    }
    pub fn player_values(&self) -> &[i32] {
        &self.player
    }
    pub fn player(&self, id: u16) -> i32 {
        self.player[id as usize]
    }
    pub fn set_player(&mut self, id: u16, value: i32) {
        self.player[id as usize] = value;
        self.generation = self.generation.wrapping_add(1);
        if self.changed.len() == 32 {
            self.changed.pop_front();
        }
        self.changed.push_back(id);
    }
    pub fn changed(&self) -> impl Iterator<Item = u16> + '_ {
        self.changed.iter().copied()
    }
    pub fn from_server(&mut self, id: u16, value: i32) {
        self.server[id as usize] = value;
        self.set_player(id, value);
    }
    pub fn server_differences(&self) -> Vec<u16> {
        self.player
            .iter()
            .zip(&self.server)
            .enumerate()
            .filter_map(|(id, (a, b))| (a != b).then_some(id as u16))
            .collect()
    }
    pub fn synchronize(&mut self) {
        for id in 0..self.player.len() {
            if self.player[id] != self.server[id] {
                self.set_player(id as u16, self.server[id]);
            }
        }
    }
    pub fn reset_server(&mut self, ids: impl IntoIterator<Item = u16>) {
        for id in ids {
            self.server[id as usize] = 0;
            self.set_player(id, 0);
        }
    }
}
pub struct Runtime<S> {
    pub source: S,
    pub variables: Variables,
    random: u64,
}
impl<S> Runtime<S> {
    pub fn new(source: S, seed: u64) -> Self {
        Self {
            source,
            variables: Variables::default(),
            random: (seed ^ 0x5deece66d) & ((1 << 48) - 1),
        }
    }
}
fn random_double(seed: &mut u64) -> f64 {
    *seed = seed.wrapping_mul(0x5deece66d).wrapping_add(11) & ((1 << 48) - 1);
    let high = *seed >> (48 - 26);
    *seed = seed.wrapping_mul(0x5deece66d).wrapping_add(11) & ((1 << 48) - 1);
    let low = *seed >> (48 - 27);
    ((high << 27) | low) as f64 / (1u64 << 53) as f64
}
impl<S: Source> Host for Runtime<S> {
    fn script(&mut self, id: i32) -> Result<Rc<Script>> {
        self.source.script(id)
    }
    fn read(&mut self, kind: Variable, id: i32) -> Result<Value> {
        Ok(match kind {
            Variable::Player => Value::Int(
                self.variables
                    .player(u16::try_from(id).map_err(|_| Error::Invalid("player variable id"))?),
            ),
            Variable::Bit => {
                let bit = self.source.varbit(id)?;
                Value::Int(bit.get(self.variables.player(bit.base)))
            }
            Variable::ClientInt => Value::Int(match self.variables.client.get(&id) {
                Some(Value::Int(v)) => *v,
                _ => -1,
            }),
            Variable::ClientLong => Value::Long(match self.variables.client.get(&id) {
                Some(Value::Long(v)) => *v,
                Some(Value::Int(v)) => *v as i64,
                _ => -1,
            }),
            Variable::ClientString => {
                Value::Object(Object::String(match self.variables.client.get(&id) {
                    Some(Value::Object(Object::String(s))) => s.clone(),
                    _ => String::new(),
                }))
            }
            Variable::ClanSetting => {
                Value::Int(*self.variables.clan_settings.get(&id).unwrap_or(&-1))
            }
            Variable::Clan => Value::Int(*self.variables.clan.get(&id).unwrap_or(&-1)),
        })
    }
    fn write(&mut self, kind: Variable, id: i32, value: Value) -> Result<()> {
        match (kind, &value) {
            (Variable::Player, Value::Int(v)) => self.variables.set_player(
                u16::try_from(id).map_err(|_| Error::Invalid("player variable id"))?,
                *v,
            ),
            (Variable::Bit, Value::Int(v)) => {
                let bit = self.source.varbit(id)?;
                let next = bit.set(self.variables.player(bit.base), *v);
                self.variables.set_player(bit.base, next);
            }
            (Variable::ClientInt, Value::Int(_))
            | (Variable::ClientLong, Value::Long(_))
            | (Variable::ClientString, Value::Object(Object::String(_) | Object::Null)) => {
                if kind != Variable::ClientString
                    && self
                        .variables
                        .persistent
                        .get(id as usize)
                        .copied()
                        .unwrap_or(false)
                {
                    self.variables.varcs_dirty = true;
                }
                self.variables.client.insert(id, value);
            }
            _ => return Err(Error::Invalid("script variable write type")),
        }
        Ok(())
    }
    fn extended(&mut self, opcode: u16, _dot_widget: bool, stacks: &mut Stacks) -> Result<Control> {
        if script_math::execute(opcode, stacks, || random_double(&mut self.random))? {
            return Ok(Control::Continue);
        }
        Err(Error::Unsupported("script host opcode"))
    }
}
