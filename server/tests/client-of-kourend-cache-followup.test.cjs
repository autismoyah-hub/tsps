// RSPSApp/tsps#5157 cache follow-up (report-only; no behaviour asserted). Test-only. Run after `yarn build`, from server/:
//   node --test tests/client-of-kourend-cache-followup.test.cjs
//
// CACHE facts only (names and definitions), NOT retail behaviour. Absent or throwing reads print as "absent"/"unreadable"; the
// only assertion is that the cache loads. Output capped per section.
//
// (a) VARBIT BACKING: the VarBitType of 12148 (X Marks SARIM_VISIBILITY, XMarksTheSpot.Quest.js:217) and 12151 (LUMBRIDGE_VISIBILITY,
//     :216): baseVar (the varp it lives in), startBit, endBit (VarBitType.ts fields). Then the decoded value of the bit range for
//     the three values the quest writes (0,1,3) and the two the Veos options need (2,4), so a corpus varp change can be decoded.
// (b) NAMES: sequence and spotanim names with id above 3651 matching a narrower regex, capped at 80; id-neighbours (±3) of
//     3651, 3257, 2721 and 2722; spotanim names near 454 and 799 (the sourced Wanted/mayor scan graphics).
const assert = require("node:assert/strict");
const path = require("node:path");
const { before, test } = require("node:test");
const { Server } = require("../dist/Server");
Server.installProductionPathResolver();
const { CachePipeline } = require("../dist/game/cache/CachePipeline");
const { CacheDefinitions } = require("../dist/game/cache/CacheDefinitions");

const MAX = 80;

before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, ".."));
});

function read(fn) {
  try { return fn(); } catch (error) { return `unreadable: ${error?.message ?? error}`; }
}

/** The bit range value of a varp for a given value of the varbit: (varp >> startBit) & mask. */
function bitsFor(startBit, endBit, value) {
  const width = endBit - startBit + 1;
  const mask = (1 << width) - 1;
  return (value >> startBit) & mask;
}

test("CACHE LOADS (the only assertion)", () => {
  assert.ok(CacheDefinitions.getVarbit(12148) !== undefined, "the cache loads varbit definitions");
});

test("(a) VARBIT BACKING for 12148 and 12151 (VarBitType: baseVar, startBit, endBit)", () => {
  for (const id of [12148, 12151]) {
    const v = read(() => CacheDefinitions.getVarbit(id));
    if (!v || typeof v === "string") { console.log(`# DEF varbit ${id} ${typeof v === "string" ? v : "NOT IN CACHE"}`); continue; }
    const baseVar = read(() => v.baseVar) ?? "absent";
    const startBit = read(() => v.startBit) ?? "absent";
    const endBit = read(() => v.endBit) ?? "absent";
    console.log(`# DEF varbit ${id} baseVar=${baseVar} startBit=${startBit} endBit=${endBit}`);
    if (typeof startBit === "number" && typeof endBit === "number") {
      for (const value of [0, 1, 2, 3, 4]) {
        console.log(`# DEF varbit ${id} decodes varp value ${value} -> bits=${bitsFor(startBit, endBit, value)} (varp ${baseVar})`);
      }
    }
  }
});

test("(b) NAMES above 3651 matching a narrower regex (capped 80), id neighbours, spotanim names near 454 and 799", () => {
  const narrow = /orb|crystal_?ball|scan|activate|hold_?up|raise_?orb|veos|kourend|arceuus|dark_?altar/i;
  const seqs = read(() => CacheDefinitions.getSeqNames()) ?? new Map();
  const above = [...seqs.entries()].filter(([id, name]) => id > 3651 && narrow.test(String(name))).slice(0, MAX);
  console.log(`# DEF seq above-3651 narrow matches (capped ${MAX}) = ${above.length}`);
  for (const [id, name] of above) console.log(`# DEF seq id=${id} name=${JSON.stringify(name)}`);
  for (const center of [3651, 3257, 2721, 2722]) {
    for (let id = center - 3; id <= center + 3; id++) {
      console.log(`# DEF seq neighbour of ${center}: id=${id} name=${JSON.stringify(seqs.get?.(id) ?? "absent")}`);
    }
  }
  const spots = read(() => CacheDefinitions.getSpotanimNames()) ?? new Map();
  for (const center of [454, 799]) {
    for (let id = center - 2; id <= center + 2; id++) {
      console.log(`# DEF spotanim near ${center}: id=${id} name=${JSON.stringify(spots.get?.(id) ?? "absent")}`);
    }
  }
});
