//! #181 (partial): passive client observation journal core, schema `client-journal/1`.
//!
//! Pure and std-only: no filesystem, process or network. Wired only into the private test
//! adapter (feature `private-test-adapter`, #181 W1); the encoding below is unchanged by that.
//! A recorder is sized once at construction (caps checked, then fallible preallocation);
//! the observation methods never allocate, never return an error to the game and never
//! panic. Running out of room, or any counter saturating, seals the journal as invalid
//! instead of dropping silently. See `docs/harness/client-journal.md`.
//!
//! What a record means, and what it does not:
//! - `Dequeue` is the client popping an event from its outbound queue and the outcome of
//!   the session staging call. It is not a socket write, delivery or server receipt.
//! - `Decoded`/`Applied` are the client's own decode and apply results.
//! - NPC facts carry the client index, hit splat values and headbar fill bytes as decoded.
//!   Absolute hitpoints and server actor generation are unavailable; index reuse is not a
//!   generation. `client_counted_server_ticks` is the client's own ServerTick packet counter.
use std::io::Write;

pub const SCHEMA: &str = "client-journal/1";
pub const MIN_RECORDS: u32 = 3;
pub const MAX_RECORDS: u32 = 65_536;
pub const MAX_NPC_FACTS: u32 = 262_144;
pub const MAX_EXPORT_BYTES: u64 = 64 << 20;
pub const MAX_LABEL: usize = 32;
/// Upper bounds on one encoded line and on one encoded NPC fact (tests check them).
pub const MAX_LINE_BYTES: u64 = 512;
pub const MAX_FACT_BYTES: u64 = 80;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    max_records: u32,
    max_npc_facts: u32,
    max_export_bytes: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LimitError {
    RecordsBelowMinimum,
    RecordsAboveCap,
    NpcFactsAboveCap,
    ExportAboveCap,
    ExportBudgetTooSmall { required: u64 },
}
impl Limits {
    /// Checks every cap and the worst-case export size before anything is allocated.
    pub fn new(max_records: u32, max_npc_facts: u32, max_export_bytes: u64) -> Result<Self, LimitError> {
        if max_records < MIN_RECORDS { return Err(LimitError::RecordsBelowMinimum); }
        if max_records > MAX_RECORDS { return Err(LimitError::RecordsAboveCap); }
        if max_npc_facts > MAX_NPC_FACTS { return Err(LimitError::NpcFactsAboveCap); }
        if max_export_bytes > MAX_EXPORT_BYTES { return Err(LimitError::ExportAboveCap); }
        let required = worst_case_bytes(max_records, max_npc_facts);
        if required > max_export_bytes { return Err(LimitError::ExportBudgetTooSmall { required }); }
        Ok(Self { max_records, max_npc_facts, max_export_bytes })
    }
    pub fn max_records(&self) -> u32 { self.max_records }
    pub fn max_npc_facts(&self) -> u32 { self.max_npc_facts }
    pub fn max_export_bytes(&self) -> u64 { self.max_export_bytes }
}
pub fn worst_case_bytes(max_records: u32, max_npc_facts: u32) -> u64 {
    max_records as u64 * MAX_LINE_BYTES + max_npc_facts as u64 * MAX_FACT_BYTES
}

/// A phase: fresh journal per phase, label limited to `[A-Za-z0-9_-]{1,32}`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Phase {
    ordinal: u32,
    label: [u8; MAX_LABEL],
    len: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LabelError;
impl Phase {
    pub fn new(ordinal: u32, label: &str) -> Result<Self, LabelError> {
        let bytes = label.as_bytes();
        if bytes.is_empty() || bytes.len() > MAX_LABEL
            || !bytes.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'_' || *b == b'-') {
            return Err(LabelError);
        }
        let mut stored = [0; MAX_LABEL];
        stored[..bytes.len()].copy_from_slice(bytes);
        Ok(Self { ordinal, label: stored, len: bytes.len() as u8 })
    }
    pub fn ordinal(&self) -> u32 { self.ordinal }
    pub fn label(&self) -> &str {
        // Only validated ASCII is ever stored.
        std::str::from_utf8(&self.label[..self.len as usize]).unwrap_or("")
    }
}

/// A saturating counter. Reaching `u32::MAX` sticks and marks the journal invalid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Count(u32);
impl Count {
    pub const ZERO: Count = Count(0);
    pub fn from_raw(value: u32) -> Self { Self(value) }
    pub fn get(&self) -> u32 { self.0 }
    pub fn saturated(&self) -> bool { self.0 == u32::MAX }
    /// Returns false once saturated; the value never wraps.
    pub fn bump(&mut self) -> bool { self.bump_by(1) }
    pub fn bump_by(&mut self, n: u32) -> bool {
        match self.0.checked_add(n) {
            Some(v) if v != u32::MAX => { self.0 = v; true }
            _ => { self.0 = u32::MAX; false }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DequeueEvent { Packet { opcode: u16, payload_len: u32 }, DiscardCipher }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StageOutcome { Staged, RefusedNotEstablished, RefusedQueueFull, RefusedEncode, RefusedOther }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeOutcome { Decoded, Unhandled, Malformed }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyOutcome { Applied, Deferred, Failed }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpcFactKind {
    Removed,
    /// Hit splat as decoded (`actor_update::Hit`): a splat value, not remaining hitpoints.
    Hit { kind: u16, value: u16 },
    /// Headbar fill bytes as decoded (`actor_update::Headbar::fill`); the denominator is
    /// the headbar definition's width, which this journal does not know.
    Headbar { kind: u16, fill: [u8; 2] },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpcFact { pub client_index: u16, pub kind: NpcFactKind }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientClocks { pub actor_cycle: i32, pub client_counted_server_ticks: i32 }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CallCounts { pub dequeues: Count, pub decoded: Count, pub applied: Count, pub npc_infos: Count, pub dropped: Count }
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Totals {
    pub records: Count, pub dequeues: Count, pub decoded: Count, pub applied: Count,
    pub npc_infos: Count, pub npc_facts: Count, pub calls: Count, pub dropped: Count,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Invalidation { RecordOverflow, NpcFactOverflow, CounterSaturated }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status { Valid, Invalid { reason: Invalidation, at_ordinal: u32 } }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactRange { start: u32, len: u32 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fact {
    Initial { phase: Phase, limits: Limits },
    Dequeue { event: DequeueEvent, outcome: StageOutcome },
    Decoded { opcode: u16, payload_len: u32, outcome: DecodeOutcome },
    Applied { opcode: u16, outcome: ApplyOutcome },
    NpcInfo { view: u16, large: bool, count: u16, facts: FactRange },
    Call { call: u32, counts: CallCounts, clocks: ClientClocks },
    Final { status: Status, totals: Totals },
}
/// Immutable once recorded; its ordinal is its index in the journal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record { ordinal: u32, fact: Fact }
impl Record {
    pub fn ordinal(&self) -> u32 { self.ordinal }
    pub fn fact(&self) -> &Fact { &self.fact }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConstructError { Alloc }

#[derive(Clone, Copy)]
enum Kind { Dequeue, Decoded, Applied, NpcInfo }

pub struct Recorder {
    limits: Limits,
    records: Vec<Record>,
    facts: Vec<NpcFact>,
    status: Status,
    totals: Totals,
    call: CallCounts,
}
impl Recorder {
    /// Preallocates every record and NPC fact slot, then writes `Initial`.
    pub fn new(limits: Limits, phase: Phase) -> Result<Self, ConstructError> {
        let mut records = Vec::new();
        records.try_reserve_exact(limits.max_records as usize).map_err(|_| ConstructError::Alloc)?;
        let mut facts = Vec::new();
        facts.try_reserve_exact(limits.max_npc_facts as usize).map_err(|_| ConstructError::Alloc)?;
        let mut recorder = Self { limits, records, facts, status: Status::Valid, totals: Totals::default(), call: CallCounts::default() };
        recorder.push(Fact::Initial { phase, limits });
        Ok(recorder)
    }
    pub fn status(&self) -> Status { self.status }
    pub fn records(&self) -> &[Record] { &self.records }

    pub fn dequeue(&mut self, event: DequeueEvent, outcome: StageOutcome) {
        if self.observe(Fact::Dequeue { event, outcome }) {
            self.count(Kind::Dequeue);
        }
    }
    pub fn decoded(&mut self, opcode: u16, payload_len: u32, outcome: DecodeOutcome) {
        if self.observe(Fact::Decoded { opcode, payload_len, outcome }) {
            self.count(Kind::Decoded);
        }
    }
    pub fn applied(&mut self, opcode: u16, outcome: ApplyOutcome) {
        if self.observe(Fact::Applied { opcode, outcome }) {
            self.count(Kind::Applied);
        }
    }
    pub fn npc_info(&mut self, view: u16, large: bool, count: u16, facts: &[NpcFact]) {
        if self.status != Status::Valid { return self.drop_one(); }
        let room = self.limits.max_npc_facts as usize - self.facts.len();
        if facts.len() > room { return self.invalidate(Invalidation::NpcFactOverflow); }
        // Check the record slot before committing facts, so a refused record leaves no facts.
        if !self.has_room() { return self.invalidate(Invalidation::RecordOverflow); }
        let range = FactRange { start: self.facts.len() as u32, len: facts.len() as u32 };
        self.facts.extend_from_slice(facts);
        self.push(Fact::NpcInfo { view, large, count, facts: range });
        if !self.totals.npc_facts.bump_by(facts.len() as u32) { self.invalidate(Invalidation::CounterSaturated); }
        self.count(Kind::NpcInfo);
    }
    /// End of one client poll call. An empty call is still recorded, with zero counts.
    pub fn call_end(&mut self, clocks: ClientClocks) {
        let call = self.totals.calls.get();
        let counts = self.call;
        if self.observe(Fact::Call { call, counts, clocks }) {
            self.call = CallCounts::default();
            if !self.totals.calls.bump() { self.invalidate(Invalidation::CounterSaturated); }
        }
    }
    /// Writes `Final` into the slot reserved for it. Always succeeds.
    pub fn finish(mut self) -> Journal {
        self.totals.records = Count::from_raw(self.records.len() as u32 + 1);
        let status = self.status;
        let totals = self.totals;
        let ordinal = self.records.len() as u32;
        self.records.push(Record { ordinal, fact: Fact::Final { status, totals } });
        Journal { records: self.records, facts: self.facts, status }
    }

    fn has_room(&self) -> bool {
        // The last slot is reserved for Final.
        self.records.len() + 1 < self.limits.max_records as usize
    }
    fn observe(&mut self, fact: Fact) -> bool {
        if self.status != Status::Valid { self.drop_one(); return false; }
        if !self.has_room() { self.invalidate(Invalidation::RecordOverflow); return false; }
        self.push(fact);
        true
    }
    fn push(&mut self, fact: Fact) {
        let ordinal = self.records.len() as u32;
        self.records.push(Record { ordinal, fact });
    }
    fn count(&mut self, kind: Kind) {
        let (t, c) = (&mut self.totals, &mut self.call);
        let (total, call) = match kind {
            Kind::Dequeue => (&mut t.dequeues, &mut c.dequeues),
            Kind::Decoded => (&mut t.decoded, &mut c.decoded),
            Kind::Applied => (&mut t.applied, &mut c.applied),
            Kind::NpcInfo => (&mut t.npc_infos, &mut c.npc_infos),
        };
        let ok = total.bump() & call.bump();
        if !ok { self.invalidate(Invalidation::CounterSaturated); }
    }
    fn invalidate(&mut self, reason: Invalidation) {
        if self.status == Status::Valid {
            self.status = Status::Invalid { reason, at_ordinal: self.records.len() as u32 };
        }
        if reason != Invalidation::CounterSaturated { self.drop_one(); }
    }
    fn drop_one(&mut self) {
        // Saturation of the drop counters is already covered by the invalid status.
        self.totals.dropped.bump();
        self.call.dropped.bump();
    }
}

pub struct Journal { records: Vec<Record>, facts: Vec<NpcFact>, status: Status }
#[derive(Debug, PartialEq, Eq)]
pub enum ExportError { Alloc, TooLarge }
impl Journal {
    pub fn status(&self) -> Status { self.status }
    pub fn records(&self) -> &[Record] { &self.records }
    /// NPC facts of an `npc_info` record, only for a reference borrowed from this journal's
    /// own `records()`. A record from another journal, or a copy, yields `None`.
    pub fn npc_facts(&self, record: &Record) -> Option<&[NpcFact]> {
        let own = self.records.get(record.ordinal as usize)?;
        if !std::ptr::eq(own, record) { return None; }
        match own.fact { Fact::NpcInfo { facts, .. } => self.fact_slice(&facts), _ => None }
    }
    fn fact_slice(&self, range: &FactRange) -> Option<&[NpcFact]> {
        let start = range.start as usize;
        self.facts.get(start..start.checked_add(range.len as usize)?)
    }
    /// Canonical NDJSON, one record per line. All or nothing.
    pub fn encode(&self) -> Result<Vec<u8>, ExportError> {
        let Some(Fact::Initial { limits, .. }) = self.records.first().map(|r| r.fact) else { return Err(ExportError::TooLarge) };
        let budget = worst_case_bytes(self.records.len() as u32, self.facts.len() as u32).min(limits.max_export_bytes);
        let mut out = Vec::new();
        out.try_reserve_exact(budget as usize).map_err(|_| ExportError::Alloc)?;
        for record in &self.records {
            let before = out.len();
            self.line(&mut out, record).map_err(|_| ExportError::TooLarge)?;
            if (out.len() - before) as u64 > MAX_LINE_BYTES + self.fact_bytes(record) {
                return Err(ExportError::TooLarge);
            }
        }
        if out.len() as u64 > limits.max_export_bytes { return Err(ExportError::TooLarge); }
        Ok(out)
    }
    fn fact_bytes(&self, record: &Record) -> u64 {
        match record.fact { Fact::NpcInfo { facts, .. } => facts.len as u64 * MAX_FACT_BYTES, _ => 0 }
    }
    fn line(&self, o: &mut Vec<u8>, record: &Record) -> std::io::Result<()> {
        write!(o, "{{\"o\":{},", record.ordinal)?;
        match record.fact {
            Fact::Initial { phase, limits } => write!(o,
                "\"k\":\"initial\",\"schema\":\"{SCHEMA}\",\"phase\":{},\"label\":\"{}\",\"max_records\":{},\"max_npc_facts\":{},\"max_export_bytes\":{},\"wire\":\"unavailable\",\"absolute_hp\":\"unavailable\",\"server_generation\":\"unavailable\"",
                phase.ordinal, phase.label(), limits.max_records, limits.max_npc_facts, limits.max_export_bytes)?,
            Fact::Dequeue { event, outcome } => {
                match event {
                    DequeueEvent::Packet { opcode, payload_len } =>
                        write!(o, "\"k\":\"dequeue\",\"event\":\"packet\",\"opcode\":{opcode},\"payload_len\":{payload_len}")?,
                    DequeueEvent::DiscardCipher =>
                        write!(o, "\"k\":\"dequeue\",\"event\":\"discard_cipher\",\"opcode\":null,\"payload_len\":null")?,
                }
                write!(o, ",\"outcome\":\"{}\"", match outcome {
                    StageOutcome::Staged => "staged",
                    StageOutcome::RefusedNotEstablished => "refused_not_established",
                    StageOutcome::RefusedQueueFull => "refused_queue_full",
                    StageOutcome::RefusedEncode => "refused_encode",
                    StageOutcome::RefusedOther => "refused_other",
                })?
            }
            Fact::Decoded { opcode, payload_len, outcome } => write!(o,
                "\"k\":\"decoded\",\"opcode\":{opcode},\"payload_len\":{payload_len},\"outcome\":\"{}\"",
                match outcome { DecodeOutcome::Decoded => "decoded", DecodeOutcome::Unhandled => "unhandled", DecodeOutcome::Malformed => "malformed" })?,
            Fact::Applied { opcode, outcome } => write!(o, "\"k\":\"applied\",\"opcode\":{opcode},\"outcome\":\"{}\"",
                match outcome { ApplyOutcome::Applied => "applied", ApplyOutcome::Deferred => "deferred", ApplyOutcome::Failed => "failed" })?,
            Fact::NpcInfo { view, large, count, facts } => {
                write!(o, "\"k\":\"npc_info\",\"view\":{view},\"large\":{large},\"count\":{count},\"facts\":[")?;
                // Ranges are only ever created by this journal's recorder.
                let facts = self.fact_slice(&facts).ok_or(std::io::ErrorKind::InvalidData)?;
                for (i, fact) in facts.iter().enumerate() {
                    if i != 0 { o.push(b','); }
                    write!(o, "{{\"client_index\":{},", fact.client_index)?;
                    match fact.kind {
                        NpcFactKind::Removed => write!(o, "\"f\":\"removed\"}}")?,
                        NpcFactKind::Hit { kind, value } => write!(o, "\"f\":\"hit\",\"kind\":{kind},\"value\":{value}}}")?,
                        NpcFactKind::Headbar { kind, fill } => write!(o, "\"f\":\"headbar\",\"kind\":{kind},\"fill\":[{},{}]}}", fill[0], fill[1])?,
                    }
                }
                write!(o, "],\"absolute_hp\":\"unavailable\",\"server_generation\":\"unavailable\"")?
            }
            Fact::Call { call, counts, clocks } => write!(o,
                "\"k\":\"call\",\"call\":{call},\"dequeues\":{},\"decoded\":{},\"applied\":{},\"npc_infos\":{},\"dropped\":{},\"actor_cycle\":{},\"client_counted_server_ticks\":{}",
                counts.dequeues.get(), counts.decoded.get(), counts.applied.get(), counts.npc_infos.get(), counts.dropped.get(),
                clocks.actor_cycle, clocks.client_counted_server_ticks)?,
            Fact::Final { status, totals } => {
                match status {
                    Status::Valid => write!(o, "\"k\":\"final\",\"status\":\"valid\",\"reason\":null,\"invalid_at\":null")?,
                    Status::Invalid { reason, at_ordinal } => write!(o, "\"k\":\"final\",\"status\":\"invalid\",\"reason\":\"{}\",\"invalid_at\":{at_ordinal}",
                        match reason {
                            Invalidation::RecordOverflow => "record_overflow",
                            Invalidation::NpcFactOverflow => "npc_fact_overflow",
                            Invalidation::CounterSaturated => "counter_saturated",
                        })?,
                }
                write!(o, ",\"records\":{},\"dequeues\":{},\"decoded\":{},\"applied\":{},\"npc_infos\":{},\"npc_facts\":{},\"calls\":{},\"dropped\":{}",
                    totals.records.get(), totals.dequeues.get(), totals.decoded.get(), totals.applied.get(),
                    totals.npc_infos.get(), totals.npc_facts.get(), totals.calls.get(), totals.dropped.get())?
            }
        }
        o.extend_from_slice(b"}\n");
        Ok(())
    }
}


/// Narrow check of an exported journal against the emitted canonical framing. It is NOT a
/// JSON parser and NOT semantic validation. It checks: the byte and line caps (before any
/// scan), `\n` framing, a printable non-space ASCII allowlist, each line's canonical
/// `{"o":<ordinal = line index>,"k":"<known kind>"` prefix and closing `}`, Initial first with
/// the exact schema, exactly one Final and last, and the Final line against its exact emitted
/// grammar. Fields of the other lines are not read.
#[derive(Debug, PartialEq, Eq)]
pub enum FramingError {
    ExportTooLarge,
    MaxLinesAboveCap,
    Empty,
    PartialEof { complete_lines: u32 },
    TooManyLines,
    ForbiddenByte { line: u32 },
    BadPrefix { line: u32 },
    OrdinalGap { line: u32 },
    UnknownKind { line: u32 },
    MissingInitial,
    MissingFinal,
    MisplacedBoundary { line: u32 },
    WrongSchema,
    FinalGrammar,
    RecordCountMismatch,
}
/// What the Final line claims, read from its exact grammar. Only `records` is checked
/// (against the line count); the status and totals are claims, not verified facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FinalClaims {
    pub status: Status,
    pub records: u32, pub dequeues: u32, pub decoded: u32, pub applied: u32,
    pub npc_infos: u32, pub npc_facts: u32, pub calls: u32, pub dropped: u32,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Framing { pub lines: u32, pub final_claims: FinalClaims }
const KINDS: [&str; 7] = ["initial", "dequeue", "decoded", "applied", "npc_info", "call", "final"];
const SCHEMA_FIELD: &[u8] = b",\"schema\":\"client-journal/1\",";

pub fn check_framing(bytes: &[u8], max_lines: u32) -> Result<Framing, FramingError> {
    if bytes.len() as u64 > MAX_EXPORT_BYTES { return Err(FramingError::ExportTooLarge); }
    if max_lines > MAX_RECORDS { return Err(FramingError::MaxLinesAboveCap); }
    if bytes.is_empty() { return Err(FramingError::Empty); }
    // At most MAX_EXPORT_BYTES (2^26) newlines, so the count fits u32.
    let complete = bytes.iter().filter(|b| **b == b'\n').count() as u32;
    if bytes.last() != Some(&b'\n') { return Err(FramingError::PartialEof { complete_lines: complete }); }
    if complete > max_lines { return Err(FramingError::TooManyLines); }
    let mut last_kind = "";
    let mut claims = None;
    for (index, line) in bytes[..bytes.len() - 1].split(|b| *b == b'\n').enumerate() {
        let n = index as u32;
        if line.iter().any(|b| !(0x21..=0x7e).contains(b)) { return Err(FramingError::ForbiddenByte { line: n }); }
        let mut c = Cursor(line);
        if !c.lit(b"{\"o\":") { return Err(FramingError::BadPrefix { line: n }); }
        let ordinal = c.uint().ok_or(FramingError::BadPrefix { line: n })?;
        if ordinal != n { return Err(FramingError::OrdinalGap { line: n }); }
        if !c.lit(b",\"k\":\"") || line.last() != Some(&b'}') { return Err(FramingError::BadPrefix { line: n }); }
        let kind = *KINDS.iter().find(|k| c.0.starts_with(k.as_bytes()) && c.0.get(k.len()) == Some(&b'"'))
            .ok_or(FramingError::UnknownKind { line: n })?;
        c.0 = &c.0[kind.len() + 1..];
        match (n, kind) {
            (0, "initial") => if !c.lit(SCHEMA_FIELD) { return Err(FramingError::WrongSchema) },
            (0, _) => return Err(FramingError::MissingInitial),
            (_, "initial") => return Err(FramingError::MisplacedBoundary { line: n }),
            _ if last_kind == "final" => return Err(FramingError::MisplacedBoundary { line: n }),
            (_, "final") => claims = Some(final_claims(c).ok_or(FramingError::FinalGrammar)?),
            _ => {}
        }
        last_kind = kind;
    }
    let Some(final_claims) = claims else { return Err(FramingError::MissingFinal) };
    if last_kind != "final" { return Err(FramingError::MissingFinal); }
    if final_claims.records != complete { return Err(FramingError::RecordCountMismatch); }
    Ok(Framing { lines: complete, final_claims })
}

/// Exactly the emitted Final grammar after `"k":"final"`; anything else is refused.
fn final_claims(mut c: Cursor<'_>) -> Option<FinalClaims> {
    let status = if c.lit(b",\"status\":\"valid\",\"reason\":null,\"invalid_at\":null") {
        Status::Valid
    } else if c.lit(b",\"status\":\"invalid\",\"reason\":\"") {
        let reason = if c.lit(b"record_overflow\"") { Invalidation::RecordOverflow }
            else if c.lit(b"npc_fact_overflow\"") { Invalidation::NpcFactOverflow }
            else if c.lit(b"counter_saturated\"") { Invalidation::CounterSaturated }
            else { return None };
        Status::Invalid { reason, at_ordinal: c.field(b",\"invalid_at\":")? }
    } else {
        return None;
    };
    let claims = FinalClaims {
        status,
        records: c.field(b",\"records\":")?,
        dequeues: c.field(b",\"dequeues\":")?,
        decoded: c.field(b",\"decoded\":")?,
        applied: c.field(b",\"applied\":")?,
        npc_infos: c.field(b",\"npc_infos\":")?,
        npc_facts: c.field(b",\"npc_facts\":")?,
        calls: c.field(b",\"calls\":")?,
        dropped: c.field(b",\"dropped\":")?,
    };
    (c.0 == b"}").then_some(claims)
}

struct Cursor<'a>(&'a [u8]);
impl Cursor<'_> {
    fn lit(&mut self, literal: &[u8]) -> bool {
        match self.0.strip_prefix(literal) { Some(rest) => { self.0 = rest; true } None => false }
    }
    /// Canonical u32 decimal: 1-10 digits, no leading zero, at most u32::MAX.
    fn uint(&mut self) -> Option<u32> {
        let digits = self.0.iter().take_while(|b| b.is_ascii_digit()).count();
        if digits == 0 || digits > 10 || (digits > 1 && self.0[0] == b'0') { return None; }
        let value = self.0[..digits].iter().fold(0u64, |v, b| v * 10 + (b - b'0') as u64);
        let value = u32::try_from(value).ok()?;
        self.0 = &self.0[digits..];
        Some(value)
    }
    fn field(&mut self, name: &[u8]) -> Option<u32> {
        if self.lit(name) { self.uint() } else { None }
    }
}
