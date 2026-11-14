// RSPSApp/tsps#5157 Parts 2+3: CACHE FACTS (report-only; no behaviour asserted). Test-only, unrun locally.
// Run after `yarn build`, from server/: node --test tests/client-of-kourend-cache-facts.test.cjs
//
// These are CACHE facts (what the definitions contain), not retail behaviour. Each line says what it is.
//
// PART 2 (NPC transforms): for the Veos NPC ids 1063, 8484, 10723, 10724, 8630, 2850, 8632, print from the real cache
// definitions (CacheDefinitions.getNpc, the NpcType the server's npc.getCurrentDefinition uses; NpcType.ts fields
// actions[], transforms[] (child ids, the LAST entry is the default), transformVarbit, transformVarp):
//   name, actions[], transforms[], transformVarbit, transformVarp, and for each child id its name and actions[].
// Then resolve the definition the runtime would show (CacheDefinitions.resolveNpc(id, vars)) for simulated varbit/varp
// values: for each state index 0..n, the chosen child and its actions. The vars object is a plain getVarbit/getVarp map
// (NpcType.transform reads only those two calls; NpcType.ts:612-632). The states use the values the Client of Kourend quest
// writes (ClientOfKourend.Quest.js: VARP veos_quest 1566, VEOS_PROGRESS_VARBIT 5619, house varbits 5620-5625 written as 1).
// Fields the TS class does not expose print as "absent" (never a guess).
//
// PART 3 (sequence names): CacheDefinitions.getSeqNames() (CacheDefinitions.ts:336) matched against /orb|crystal|scan|hold|raise/i,
// and the names of ids 2721 (Wanted: wanted_crystal_ball_scan, SlugMenace.Quest.js) and 4839 (mayor scan, the sourced Wanted
// crystal-ball scan) and 454/799 (the sourced gfx ids, if the cache names them).
//
// Output is capped (MAX lines per section); absent properties print as "absent"; the only assertion is that the cache loads.
const assert = require("node:assert/strict");
const path = require("node:path");
const { before, test } = require("node:test");
const { Server } = require("../dist/Server");
Server.installProductionPathResolver();
const { CachePipeline } = require("../dist/game/cache/CachePipeline");
const { CacheDefinitions } = require("../dist/game/cache/CacheDefinitions");

const VEOS_IDS = [1063, 8484, 10723, 10724, 8630, 2850, 8632];
const MAX = 60;

before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, ".."));
});

/** A property read that never throws: present values as JSON, absent ones as "absent". */
function field(obj, name) {
  try {
    const value = obj?.[name];
    if (value === undefined) return "absent";
    return JSON.parse(JSON.stringify(value ?? null));
  } catch (error) {
    return `unreadable: ${error?.message ?? error}`;
  }
}

/** The 5 action slots, with explicit nulls for holes: the slot index is the right-click position (NpcType.ts:115, 203-205). */
function actionsOf(npc) {
  try {
    const list = npc?.actions;
    if (!list) return "absent";
    return Array.from({ length: 5 }, (_, i) => (list[i] === undefined || list[i] === null ? null : list[i]));
  } catch (error) {
    return `unreadable: ${error?.message ?? error}`;
  }
}

function nameOf(npc) {
  try {
    return typeof npc?.getName === "function" ? npc.getName() : field(npc, "name");
  } catch (error) {
    return `unreadable: ${error?.message ?? error}`;
  }
}

/** vars with plain getVarbit/getVarp maps (NpcType.transform reads only these two). */
function vars(varbits = {}, varps = {}) {
  return {
    getVarbit: (id) => (varbits[id] ?? 0),
    getVarp: (id) => (varps[id] ?? 0),
  };
}

test("CACHE LOADS (the only assertion): the Veos NPC definitions resolve", () => {
  const npc = CacheDefinitions.getNpc(8484);
  assert.ok(npc, "NpcType for 8484 exists in the cache");
  console.log(`# DEF cache-loads ok: getNpc(8484) name=${JSON.stringify(nameOf(npc))}`);
});

test("PART 2 (cache facts): each Veos NPC's own actions, transform data, and its children", () => {
  let lines = 0;
  for (const id of VEOS_IDS) {
    if (lines++ > MAX) break;
    let npc = null;
    try { npc = CacheDefinitions.getNpc(id); } catch (error) { console.log(`# DEF npc=${id} getNpc threw: ${error?.message ?? error}`); continue; }
    if (!npc) { console.log(`# DEF npc=${id} NOT IN CACHE`); continue; }
    console.log(`# DEF npc=${id} name=${JSON.stringify(nameOf(npc))} actions=${JSON.stringify(actionsOf(npc))} transforms=${JSON.stringify(field(npc, "transforms"))} transformVarbit=${JSON.stringify(field(npc, "transformVarbit"))} transformVarp=${JSON.stringify(field(npc, "transformVarp"))}`);
    const children = Array.isArray(npc.transforms) ? npc.transforms.filter((c) => c !== -1 && c !== undefined) : [];
    for (const child of children) {
      let c = null;
      try { c = CacheDefinitions.getNpc(child); } catch (error) { console.log(`# DEF   child=${child} getNpc threw: ${error?.message ?? error}`); continue; }
      console.log(`# DEF   child=${child} name=${JSON.stringify(nameOf(c))} actions=${JSON.stringify(actionsOf(c))} transforms=${JSON.stringify(field(c, "transforms"))}`);
    }
  }
});

test("PART 2 (cache facts): the REAL transform variable of each Veos id, enumerated 0..transforms.length-2 with resolveNpc", () => {
  for (const id of VEOS_IDS) {
    let npc = null;
    try { npc = CacheDefinitions.getNpc(id); } catch (error) { console.log(`# DEF enum npc=${id} getNpc threw: ${error?.message ?? error}`); continue; }
    if (!npc) { console.log(`# DEF enum npc=${id} NOT IN CACHE`); continue; }
    const varbit = typeof npc.transformVarbit === "number" ? npc.transformVarbit : -1;
    const varp = typeof npc.transformVarp === "number" ? npc.transformVarp : -1;
    const transforms = Array.isArray(npc.transforms) ? npc.transforms : null;
    const kind = varbit !== -1 ? "varbit" : varp !== -1 ? "varp" : "none";
    const ref = kind === "varbit" ? varbit : varp;
    console.log(`# DEF enum npc=${id} transformVarbit=${varbit} transformVarp=${varp} kind=${kind} ref=${ref} transforms=${JSON.stringify(transforms)}`);
    if (!transforms || kind === "none") continue;
    // The last transforms[] entry is the default; indices 0..length-2 select a child (NpcType.ts transform logic).
    for (let value = 0; value <= transforms.length - 2; value++) {
      const v = kind === "varbit" ? vars({ [ref]: value }, {}) : vars({}, { [ref]: value });
      let resolved = null;
      try { resolved = CacheDefinitions.resolveNpc(id, v); } catch (error) { console.log(`# DEF enum npc=${id} ${kind}=${ref} value=${value} threw: ${error?.message ?? error}`); continue; }
      console.log(`# DEF enum npc=${id} ${kind}=${ref} value=${value} -> id=${resolved?.id ?? "undefined"} name=${JSON.stringify(nameOf(resolved))} actions=${JSON.stringify(actionsOf(resolved))}`);
    }
  }
});

test("PART 2 (cache facts): the definition resolveNpc chooses per simulated state for 8484 and 10724", () => {
  // The states the quest writes: progress 5619 (0, 4 orb, 5 activated, 9 complete), the house varbits 5620-5625 (0 or 1), veos_quest varp 1566 (0..n).
  const states = [
    { label: "none", varbits: {}, varps: {} },
    { label: "progress-5619=4", varbits: { 5619: 4 }, varps: {} },
    { label: "progress-5619=9", varbits: { 5619: 9 }, varps: {} },
    { label: "reveal-5625=1", varbits: { 5625: 1 }, varps: {} },
    { label: "varp-1566=1", varbits: {}, varps: { 1566: 1 } },
    { label: "varp-1566=2", varbits: {}, varps: { 1566: 2 } },
  ];
  for (const id of [8484, 10724]) {
    let base = null;
    try { base = CacheDefinitions.getNpc(id); } catch (error) { console.log(`# DEF resolve npc=${id} getNpc threw: ${error?.message ?? error}`); continue; }
    if (!base) { console.log(`# DEF resolve npc=${id} NOT IN CACHE`); continue; }
    for (const state of states) {
      let resolved = null;
      try { resolved = CacheDefinitions.resolveNpc(id, vars(state.varbits, state.varps)); } catch (error) { console.log(`# DEF resolve npc=${id} state=${state.label} threw: ${error?.message ?? error}`); continue; }
      console.log(`# DEF resolve npc=${id} state=${state.label} -> id=${resolved?.id ?? "undefined"} name=${JSON.stringify(nameOf(resolved))} actions=${JSON.stringify(actionsOf(resolved))}`);
    }
  }
});

test("PART 3 (cache facts): sequence names matching orb/crystal/scan/hold/raise, and the sourced Commorb-scan ids", () => {
  let names = null;
  try { names = CacheDefinitions.getSeqNames(); } catch (error) { console.log(`# DEF seq getSeqNames threw: ${error?.message ?? error}`); return; }
  const entries = [...(names ?? new Map()).entries()];
  console.log(`# DEF seq total=${entries.length}`);
  const hits = entries.filter(([, name]) => /orb|crystal|scan|hold|raise/i.test(String(name))).slice(0, MAX);
  for (const [id, name] of hits) console.log(`# DEF seq match id=${id} name=${JSON.stringify(name)}`);
  console.log(`# DEF seq match count (capped at ${MAX}) = ${hits.length}`);
  for (const id of [2721, 4839]) console.log(`# DEF seq id=${id} name=${JSON.stringify(names?.get?.(id) ?? "absent")}`);
});
