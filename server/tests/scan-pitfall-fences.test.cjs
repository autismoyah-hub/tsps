// Throwaway read-only scan for issue #5736 (Hunter pitfalls, a teased creature crossing a fenced side). Nothing here asserts
// behaviour. For every pit base loc in the Feldip Hills larupia area it prints the footprint and, for each tile within the
// plugin's own near() reach (Chebyshev distance <= 5 from the pit centre, Context.Hunter.js distance()), the straight run
// moveTowards/exactMove would make to the pit's coord (one sign-step at a time, no collision check): clear, blocked because an
// endpoint tile is itself blocked ('endpoint'), or blocked by a fence/wall between two free tiles ('fence'), for a size-1 and a
// larupia-size creature. Run after `yarn build`: node --test tests/scan-pitfall-fences.test.cjs
process.env.DISABLE_PLAYER_BOTS = "1";
const assert = require("node:assert/strict");
const path = require("node:path");
const { before, after, test } = require("node:test");
require("../dist/Server").Server.installProductionPathResolver();
const { CachePipeline } = require("../dist/game/cache/CachePipeline");
const { PluginManager: PM } = require("../dist/plugins/PluginManager");
const { ObjectDefinition } = require("../dist/game/definition/ObjectDefinition");

let core;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, ".."));
  PM.loadFromDirectory(path.resolve(__dirname, "../plugins"));
  core = PM.getCoreApi();
  core.RegionManager.init(); // as tests/leechfin-fishing.test.cjs
  ObjectDefinition.init();
  PM.emitServerStartup({});
});
after(() => PM.emitServerShutdown({}));

const L = (x, y, z = 0) => new core.Location(x, y, z);
const guarded = (fn) => { try { return fn(); } catch (error) { return `ERR ${error?.message}`; } };

/** Pitfalls.Hunter.js footprint(): the base loc's tiles, turned with its face. */
function footprint(base) {
  const def = core.CacheDefinitions.getObject(base.getId()), at = base.getLocation();
  const turned = base.getFace() % 2 === 1, sizeX = def?.sizeX ?? 2, sizeY = def?.sizeY ?? 2;
  const width = turned ? sizeY : sizeX, length = turned ? sizeX : sizeY;
  return { x0: at.getX(), y0: at.getY(), x1: at.getX() + width - 1, y1: at.getY() + length - 1, z: at.getZ() };
}

/** Context.Hunter.js distance(): Chebyshev. center(): Pitfalls.Hunter.js center(trap). */
const distance = (a, b) => Math.max(Math.abs(a.getX() - b.getX()), Math.abs(a.getY() - b.getY()));
const center = (f) => L((f.x0 + f.x1) >> 1, (f.y0 + f.y1) >> 1, f.z);

/**
 * A straight run as moveTowards(oneTile) makes it (one sign-step at a time, size-`size` NPC): the first refused step, classed
 * 'endpoint' when either tile is itself blocked (a pit tile, a rock) and 'fence' when both tiles are free but the step is refused
 * (a fence or wall between them).
 */
function run(from, to, size) {
  let at = from, steps = 0;
  while (!(at.getX() === to.getX() && at.getY() === to.getY()) && steps++ < 12) {
    const next = L(at.getX() + Math.sign(to.getX() - at.getX()), at.getY() + Math.sign(to.getY() - at.getY()), at.getZ());
    if (!core.RegionManager.canMovestart(at, next, size, size, null)) {
      const endpoint = core.RegionManager.blocked(at, null) || core.RegionManager.blocked(next, null);
      return { kind: endpoint ? "endpoint" : "fence", at: [at.getX(), at.getY()], next: [next.getX(), next.getY()] };
    }
    at = next;
  }
  return { kind: "clear" };
}

test("scan: pits of the Feldip Hills larupia area, and every creature tile the plugin would accept", () => {
  // Spined larupia spawns in npc-spawns.json: x 2544..2573, y 2883..2910 (regions 39,45 and 40,45).
  for (const [rx, ry] of [[39, 45], [40, 45]]) core.RegionManager.loadMapFiles(rx * 64 + 8, ry * 64 + 8, false);
  const collapsed = new Set(Object.keys(core.ObjectIdentifiers).filter(k => k.startsWith("COLLAPSED_TRAP")).map(k => core.ObjectIdentifiers[k]));
  const larupiaSize = guarded(() => core.CacheDefinitions.getNpc(core.NpcIdentifiers.SPINED_LARUPIA).size) ?? "?";
  let objects = 0; const bases = [];
  for (let x = 2530; x <= 2590; x++) for (let y = 2870; y <= 2925; y++) {
    for (const object of core.MapObjects.getObjectsAt(L(x, y), null)) {
      objects++;
      const def = guarded(() => core.CacheDefinitions.getObject(object.getId()));
      if (def && Array.isArray(def.transforms) && def.transforms.some(t => collapsed.has(t))) bases.push(object);
    }
  }
  console.log(`PITSCAN collapsed-trap ids ${JSON.stringify([...collapsed])} larupia size ${larupiaSize} objects scanned ${objects} pit base locs ${bases.length}`);
  console.log(`PITSCAN pits ${JSON.stringify(bases.map(b => [b.getId(), guarded(() => core.CacheDefinitions.getObject(b.getId()).name), b.getLocation().getX(), b.getLocation().getY(), b.getFace()]))}`);
  assert.ok(objects > 0, "the scan loaded no objects: the map did not load");
  assert.ok(bases.length > 0, `no pit base loc found among ${objects} objects: the pit id predicate or the area is wrong, nothing can be said about fences`);
  for (const base of bases) {
    const f = footprint(base), c = center(f), at = base.getLocation();
    // The pit's own tiles, apart from the fence question.
    const pitTiles = []; for (let x = f.x0; x <= f.x1; x++) for (let y = f.y0; y <= f.y1; y++) pitTiles.push(guarded(() => core.RegionManager.blocked(L(x, y), null)));
    for (const size of [1, larupiaSize === "?" ? 2 : larupiaSize]) {
      const counts = { tiles: 0, blockedTile: 0, clear: 0, endpoint: 0, fence: 0 }, samples = [];
      // Every tile within the plugin's own near() reach (distance <= 5 from the pit centre) a creature could stand on.
      for (let x = c.getX() - 5; x <= c.getX() + 5; x++) for (let y = c.getY() - 5; y <= c.getY() + 5; y++) {
        const t = L(x, y, f.z);
        if (distance(t, c) > 5) continue;
        counts.tiles++;
        if (core.RegionManager.blocked(t, null)) { counts.blockedTile++; continue; }
        const r = run(t, L(f.x0, f.y0, f.z), size);
        counts[r.kind]++;
        if (r.kind === "fence" && samples.length < 6) samples.push([[x, y], r.at, r.next]);
      }
      console.log(`PIT ${JSON.stringify({ id: base.getId(), at: [at.getX(), at.getY()], face: base.getFace(), fp: [f.x0, f.y0, f.x1, f.y1], centre: [c.getX(), c.getY()], pitTilesBlocked: pitTiles, size, counts, fenceSamples_from_step_next: samples })}`);
    }
    // What stands in the way, for the first fence sample's neighbourhood: objects within 1 tile of the pit's footprint.
    const around = [];
    for (let x = f.x0 - 1; x <= f.x1 + 1; x++) for (let y = f.y0 - 1; y <= f.y1 + 1; y++) for (const o of guarded(() => core.MapObjects.getObjectsAt(L(x, y), null)) || [])
      if (o !== base && around.length < 14) around.push([o.getId(), guarded(() => core.CacheDefinitions.getObject(o.getId()).name), o.getType(), o.getFace(), x, y]);
    console.log(`PITOBJ ${JSON.stringify({ id: base.getId(), around })}`);
  }
});
