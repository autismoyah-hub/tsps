// RSPSApp/tsps#5275 reachability remainder: can Telekinetic Grab on the Elid ancestral key (3432,2928,0)
// resolve from a natural approach? Test-only diagnostic on test/5275-reach-diagnostic (off af254fcfd).
// Run after `yarn build`, from server/: node --test tests/spirits-elid-reachability.test.cjs
//
// Real code on the path: real Player with a real PlayerSession on a stub channel (messages recorded via
// the real PacketSender wrapped to a list), real login hook (PluginManager.emitPlayerLogin -> the quest's
// handleLogin -> ensureAncestralKey), real MagicOnItemPacketListener.castGroundItem, real MovementQueue.walkToTile
// driven by the real movement tick and TaskManager.process (unpatched), real ItemOnGroundManager, real inventory
// runes, real magic and hitpoints levels, real RegionManager.canMove collision for the frontier.
// Seeded and disclosed: stage 20 via quest.setStage (the ballad handout is not replayed).
// Not exercised: the spell_on_ground client packet (resolveClientCombatSpellId's widget mapping for Telekinetic
// Grab was not verified in this pass, so castGroundItem is called directly with spell id 1168). Other plugins'
// magic:can-telekinetic-grab listeners (Prayer, Ironman, MisthalinMystery) are not loaded.
const assert = require("node:assert/strict");
const path = require("node:path");
const { before, test } = require("node:test");
const { Server } = require("../dist/Server");
Server.installProductionPathResolver();
const { CachePipeline } = require("../dist/game/cache/CachePipeline");
const { NpcDefinitionLoader } = require("../dist/game/definition/loader/impl/NpcDefinitionLoader");
const { PluginManager } = require("../dist/plugins/PluginManager");
const { Player } = require("../dist/game/entity/impl/player/Player");
const { PlayerSession } = require("../dist/net/PlayerSession");
const { Location } = require("../dist/game/model/Location");
const { Item } = require("../dist/game/model/Item");
const { ItemIdentifiers } = require("../dist/util/ItemIdentifiers");
const { RegionManager } = require("../dist/game/collision/RegionManager");
const { World } = require("../dist/game/World");
const { TaskManager } = require("../dist/game/task/TaskManager");
const { ItemOnGroundManager } = require("../dist/game/entity/impl/grounditem/ItemOnGroundManager");
const { MagicOnItemPacketListener } = require("../dist/net/packet/impl/MagicOnItemPacketListener");
const { CombatRange } = require("../dist/game/content/combat/CombatRange");
const { PathFinder } = require("../dist/game/model/movement/path/PathFinder");
const { MapObjects } = require("../dist/game/entity/impl/object/MapObjects");
const { ObjectDefinition } = require("../dist/game/definition/ObjectDefinition");
const runtime = require("../plugins/quests/QuestRuntime");

runtime.startTranscript = () => true;

const KEY = 6792;
const KEY_TILE = new Location(3432, 2928, 0);
const TELEKINETIC_GRAB = 1168;
const GRAB_RANGE = 10; // MagicOnItemPacketListener.ts:54
const STAGE_BALLAD = 20;
const STAGE_KEY = 25;
const KEY_LIFETIME = 100000; // SpiritsOfTheElid.Quest.js ANCESTRAL_KEY_DESPAWN_TICKS
const TICK_LIMIT = 150; // walk (<=60 steps) + landing (ceil(80/30)=3 ticks, MagicOnItemPacketListener.ts) + margin
const IN_RANGE_SEED = new Location(3430, 2928, 0); // Chebyshev 2 from the key; the BFS seed
const GHASLOR_SPAWN = new Location(3441, 2933, 0); // data/definitions/npc-spawns.json: id 4745, wanderRadius 2
const SHOES_TILE = new Location(3442, 2917, 0); // SpiritsOfTheElid.Quest.js:197 (drop tile, sourced sample)
const RING_MIN = 11;
const RING_MAX = 15;
const SAMPLE_CAP = 20;

let quest = null;
let index = 4000;
let cycle = 5000;
const actors = [];

before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, ".."));
  new NpcDefinitionLoader().load();
  ObjectDefinition.init(); // object names for the blocker report
  // Collision is loaded here. Without init() the region map is empty and every canMove is open, so
  // the diagnostic would find no block by construction (the same init + loadMapFiles the #5289 test uses).
  PluginManager.getCoreApi().RegionManager.init();
  PluginManager.getCoreApi().RegionManager.loadMapFiles(KEY_TILE.getX(), KEY_TILE.getY());
  const count = runtime.getRegisteredQuests().length;
  require("../plugins/quests/quests/SpiritsOfTheElid.Quest")(PluginManager.createApi("ElidReachability"));
  quest = runtime.getRegisteredQuests().slice(count).at(-1);
  assert.ok(quest, "the Elid quest registers through the real plugin API");
});

function chebyshev(a, b) { return Math.max(Math.abs(a.getX() - b.getX()), Math.abs(a.getY() - b.getY())); }
const tileKey = (t) => `${t.getX()},${t.getY()}`;

/** One real world tick, in the order the other native tests use: tasks, ground items, timers, movement. */
/**
 * One world cycle in World.process order: tasks (WALK_TO tasks are skipped here), ground items, each
 * player's owner turn (Player.process: timers, movement, then walk-to), then the walk-to sweep.
 * Combat, hit queue, aggression and areas are not run: none of these starts is in combat.
 * Evidence: TaskManager.ts:37-43 skips WALK_TO in process(); Player.ts:536 runs processWalkTo;
 * World.ts:1389 runs processRemainingWalkTo. The walk task only runs through these.
 */
function tick(player) {
  cycle++;
  World.processCycle = cycle;
  TaskManager.process();
  ItemOnGroundManager.process();
  if (player) ownerTurn(player);
  TaskManager.processRemainingWalkTo();
}

function ownerTurn(player) {
  player.getTimers().process();
  const mq = player.getMovementQueue();
  mq.beginCycle();
  mq.processEntityPursuit();
  if (mq.hasPendingWork()) mq.process();
  TaskManager.processWalkTo(player.getIndex());
  mq.dropPendingClickRunOverride();
  mq.endClickRunOverrideWhenIdle();
}

/** A real player: real PlayerSession on a stub channel; sendMessage recorded through the real PacketSender. */
function actor(start, label) {
  const username = `elid-reach-${label}`;
  const channel = { kind: "test", remoteAddress: "127.0.0.1", isOpen: () => true, send: () => true,
    close() {}, onData() {}, onError() {}, onClose() {} };
  const player = new Player(new PlayerSession(channel), new Location(start.getX(), start.getY(), start.getZ()));
  player.setIndex(index++);
  player.setRegistered(true);
  player.setNeedsPlacement(false);
  player.setUsername(username);
  const sender = player.getPacketSender();
  const messages = [];
  for (const name of ["sendMessage", "sendTypedMessage"]) {
    const original = sender[name].bind(sender);
    sender[name] = (...args) => { messages.push(String(args.find((a) => typeof a === "string") ?? "")); return original(...args); };
  }
  const core = PluginManager.getCoreApi();
  // castGroundItem needs magic 33, live hitpoints and law + air runes in the real inventory.
  player.getSkillManager().setCurrentLevel(core.Skill.MAGIC, 33);
  player.getSkillManager().setMaxLevel(core.Skill.MAGIC, 99);
  player.getSkillManager().setCurrentLevel(core.Skill.HITPOINTS, 10);
  player.getSkillManager().setMaxLevel(core.Skill.HITPOINTS, 10);
  player.getInventory().adds(ItemIdentifiers.LAW_RUNE, 10);
  player.getInventory().adds(ItemIdentifiers.AIR_RUNE, 10);
  actors.push(player);
  return { player, messages, username };
}

/** Stage 20 seeded through the quest API, then the REAL login hook creates the key (ensureAncestralKey). */
function loginWithKey(player, username) {
  quest.setStage(player, STAGE_BALLAD);
  PluginManager.emitPlayerLogin({ player, username, isNewAccount: false });
  return ItemOnGroundManager.getGroundItem(username, KEY, KEY_TILE, player.getPrivateArea() ?? null);
}

/** Single-step collision from the real RegionManager. */
function stepOk(from, dx, dy) {
  return RegionManager.canMove(from.getX(), from.getY(), from.getX() + dx, from.getY() + dy, from.getZ(), 1, 1, null);
}

/** Frontier: BFS of real single steps from the in-range seed, to Chebyshev RING_MAX. */
function explore() {
  const seen = new Map([[tileKey(IN_RANGE_SEED), IN_RANGE_SEED]]);
  const queue = [IN_RANGE_SEED];
  const dirs = [[1, 0], [-1, 0], [0, 1], [0, -1]];
  for (let head = 0; head < queue.length; head++) {
    const here = queue[head];
    for (const [dx, dy] of dirs) {
      const next = new Location(here.getX() + dx, here.getY() + dy, 0);
      if (seen.has(tileKey(next)) || chebyshev(next, KEY_TILE) > RING_MAX) continue;
      if (!stepOk(here, dx, dy)) continue;
      seen.set(tileKey(next), next);
      queue.push(next);
    }
  }
  const ring = [...seen.values()].filter((t) => chebyshev(t, KEY_TILE) >= RING_MIN);
  ring.sort((a, b) => chebyshev(a, KEY_TILE) - chebyshev(b, KEY_TILE) || a.getX() - b.getX() || a.getY() - b.getY());
  return { seen, ring };
}

/** Real cast from the start, then real ticks until the key is in the inventory or the tick limit. */
function castAndDrive(player, messages, username, start) {
  const listener = new MagicOnItemPacketListener();
  const pre = {
    keyOnGround: !!ItemOnGroundManager.getGroundItem(username, KEY, KEY_TILE, player.getPrivateArea() ?? null),
    sameFloor: player.getLocation().getZ() === KEY_TILE.getZ(),
    privateArea: player.getPrivateArea() ?? null,
    runesLaw: player.getInventory().getAmount(ItemIdentifiers.LAW_RUNE),
    runesAir: player.getInventory().getAmount(ItemIdentifiers.AIR_RUNE),
    hitpoints: player.getHitpoints(),
    clickDelayOk: player.getClickDelay().elapsedTime(500),
    stage: quest.getStage(player),
  };
  // The predicate at the start, evaluated by the same CombatRange call the cast uses (not Chebyshev).
  const predicateAtStart = CombatRange.withinApproachDistance(player, KEY_TILE, GRAB_RANGE);
  listener.castGroundItem(player, KEY, KEY_TILE.getX(), KEY_TILE.getY(), TELEKINETIC_GRAB);
  // Predicate-based arrival: the first tick at which the predicate holds (null if it never does).
  let ticksToArrival = predicateAtStart ? 0 : null;
  const messagesAtCast = messages.length;
  for (let n = 0; n < TICK_LIMIT; n++) {
    if (player.getInventory().getAmount(KEY) > 0) break;
    tick(player);
    if (ticksToArrival === null && CombatRange.withinApproachDistance(player, KEY_TILE, GRAB_RANGE)) ticksToArrival = n + 1;
  }
  return {
    start: tileKey(start), chebyshev: chebyshev(start, KEY_TILE), predicateAtStart, pre,
    ticksToArrival, finalTile: tileKey(player.getLocation()),
    predicateAtFinal: CombatRange.withinApproachDistance(player, KEY_TILE, GRAB_RANGE),
    messages: messages.slice(messagesAtCast), stage: quest.getStage(player),
    keyInInventory: player.getInventory().getAmount(KEY) > 0,
  };
}

/** Cleanup per actor: cancel its tasks, remove its key pile, so one start cannot leak into the next. */
function cleanup() {
  for (const p of actors.splice(0)) {
    TaskManager.cancelTasks(p.getIndex());
    const pile = ItemOnGroundManager.getGroundItem(p.getUsername(), KEY, KEY_TILE, p.getPrivateArea() ?? null);
    if (pile) ItemOnGroundManager.deregister(pile);
    p.setRegistered(false);
  }
}

/** Collision is live: the region exists, the box around the key has blocked edges, and the key's neighbours are reported. */
function collisionFacts() {
  const core = PluginManager.getCoreApi();
  const region = core.RegionManager.getRegion(KEY_TILE.getX(), KEY_TILE.getY());
  let blocked = 0, total = 0;
  for (let dx = -15; dx <= 15; dx++) for (let dy = -15; dy <= 15; dy++) {
    total++;
    if (core.RegionManager.getClipping(KEY_TILE.getX() + dx, KEY_TILE.getY() + dy, 0, null) !== 0) blocked++;
  }
  const neighbours = [[1, 0, "E"], [-1, 0, "W"], [0, 1, "N"], [0, -1, "S"]].map(([dx, dy, name]) => {
    const open = stepOk(KEY_TILE, dx, dy);
    return `${name}=${open ? "open" : "blocked"}`;
  });
  const keyClip = core.RegionManager.getClipping(KEY_TILE.getX(), KEY_TILE.getY(), 0, null);
  return { region: !!region, loaded: !!region, blocked, total, keyClip, neighbours };
}

test("PRECONDITION: collision is live (region loaded, box has blocked edges, key neighbours reported)", () => {
  const facts = collisionFacts();
  console.log(`# REACH collision region loaded=${facts.loaded} box31x31 blockedEdges=${facts.blocked}/${facts.total} keyClip=${facts.keyClip} neighbours ${facts.neighbours.join(" ")}`);
  assert.equal(facts.loaded, true, "precondition: the key's region is loaded (RegionManager.init + loadMapFiles)");
  assert.ok(facts.blocked > 0, "precondition: the 31x31 box has blocked edges, so canMove is not vacuously open");
});

/** Hard-assert starts: a missing key, a dead click delay or a bad level is a harness precondition failure. */
function assertPrecondition(out) {
  assert.equal(out.pre.keyOnGround, true, `precondition: key on the ground: ${JSON.stringify(out.pre)}`);
  assert.equal(out.pre.clickDelayOk, true, `precondition: click delay elapsed: ${JSON.stringify(out.pre)}`);
  assert.equal(out.pre.sameFloor, true, "precondition: same floor");
  assert.ok(out.pre.runesLaw > 0 && out.pre.runesAir > 0, `precondition: law and air runes: ${JSON.stringify(out.pre)}`);
  assert.ok(out.pre.hitpoints > 0, "precondition: live hitpoints");
}

test("REACH: frontier precondition and Ghaslor conversation-tile range (real RegionManager)", () => {
  const { seen, ring } = explore();
  console.log(`# REACH explored ${seen.size} tiles from seed ${tileKey(IN_RANGE_SEED)}; ring ${RING_MIN}-${RING_MAX} size ${ring.length}`);
  assert.ok(ring.length > 0, "precondition: the ring 11-15 is non-empty, so the frontier test is not vacuous");
  console.log(`# REACH shoes ${tileKey(SHOES_TILE)} connected to seed: ${seen.has(tileKey(SHOES_TILE))}`);
  const ghaslorTiles = [];
  for (let dx = -2; dx <= 2; dx++) for (let dy = -2; dy <= 2; dy++) {
    ghaslorTiles.push(new Location(GHASLOR_SPAWN.getX() + dx, GHASLOR_SPAWN.getY() + dy, 0));
  }
  // The conversation tile: the player is adjacent to Ghaslor's current tile, so the distance spans NPC tile +/- 1.
  const distances = ghaslorTiles.flatMap((t) => [-1, 0, 1].flatMap((dx) => [-1, 0, 1].map((dy) =>
    chebyshev(new Location(t.getX() + dx, t.getY() + dy, 0), KEY_TILE))));
  console.log(`# REACH Ghaslor conversation-tile Chebyshev range ${Math.min(...distances)}..${Math.max(...distances)}`);
});

/** The approach predicate itself, evaluated for a player standing on `tile` (not Chebyshev). */
function approachAt(probe, tile) {
  probe.setLocation(new Location(tile.getX(), tile.getY(), 0));
  return CombatRange.withinApproachDistance(probe, KEY_TILE, GRAB_RANGE);
}

test("harness proof: a start outside the predicate walks to the predicate and lands; the walk task must run", () => {
  const { seen, ring } = explore();
  const { player: probe } = actor(IN_RANGE_SEED, "probe");
  // Route end: where a walk that is never ticked would stop. The proof only discriminates if this tile fails the predicate.
  const routeEnd = new Location(3433, 2928, 0);
  assert.equal(approachAt(probe, routeEnd), false,
    "precondition: the route end fails withinApproachDistance, so a walk that is never ticked fails this proof");
  const dirs = [[1, 0], [-1, 0], [0, 1], [0, -1]];
  // Start: outside the predicate, with a real open step to a tile the predicate accepts (chosen by the predicate).
  let pick = null;
  for (const start of ring.filter((t) => chebyshev(t, KEY_TILE) <= 12)) {
    if (approachAt(probe, start)) continue;
    for (const [dx, dy] of dirs) {
      if (!stepOk(start, dx, dy)) continue;
      const mid = new Location(start.getX() + dx, start.getY() + dy, 0);
      if (approachAt(probe, mid)) { pick = { start, mid }; break; }
    }
    if (pick) break;
  }
  assert.ok(pick, "precondition: a start outside withinApproachDistance with an open step into it exists");
  assert.ok(seen.has(tileKey(pick.start)), "the proof start is connected to the seed");
  cleanup();
  const { player, messages, username } = actor(pick.start, "walk-proof");
  assert.ok(loginWithKey(player, username), "precondition: the real login hook registered the key for this player");
  const out = castAndDrive(player, messages, username, pick.start);
  const satisfiedAtEnd = approachAt(player, player.getLocation());
  console.log(`# REACH walk-proof mid=${tileKey(pick.mid)} ${JSON.stringify(out)} predicateAtFinal=${satisfiedAtEnd}`);
  assertPrecondition(out);
  assert.equal(out.keyInInventory, true, `walk proof must land: ${JSON.stringify(out)}`);
  assert.equal(out.stage, STAGE_KEY, "stage 20 -> 25");
  assert.equal(satisfiedAtEnd, true, "the walk stopped on a tile that satisfies the predicate, not the route end");
  cleanup();
});

test("positive: an in-range start (no walk) grabs the key and advances 20 -> 25", () => {
  assert.ok(chebyshev(IN_RANGE_SEED, KEY_TILE) <= GRAB_RANGE, "the start is inside the grab range");
  const { player, messages, username } = actor(IN_RANGE_SEED, "in-range");
  assert.ok(loginWithKey(player, username));
  const out = castAndDrive(player, messages, username, IN_RANGE_SEED);
  console.log(`# REACH in-range ${JSON.stringify(out)}`);
  assert.equal(out.predicateAtStart, true, "precondition: the in-range seed satisfies withinApproachDistance");
  assertPrecondition(out);
  assert.equal(out.keyInInventory, true, `in-range must land: ${JSON.stringify(out)}`);
  assert.equal(out.stage, STAGE_KEY);
  cleanup();
});

test("positive: Ghaslor's spawn tile (in range, no walk) grabs the key", () => {
  const { player, messages, username } = actor(GHASLOR_SPAWN, "ghaslor");
  assert.ok(loginWithKey(player, username));
  const out = castAndDrive(player, messages, username, GHASLOR_SPAWN);
  console.log(`# REACH ghaslor-spawn ${JSON.stringify(out)}`);
  // Hard only when the predicate holds at the spawn; otherwise the row is report-only (STOP rule).
  if (out.predicateAtStart) {
    assertPrecondition(out);
    assert.equal(out.keyInInventory, true, `ghaslor spawn (predicate holds) must land: ${JSON.stringify(out)}`);
  } else {
    console.log(`# REACH ghaslor-spawn report-only: predicateAtStart=false, not a hard positive`);
  }
  cleanup();
});

test("natural approach: sampled frontier starts are reported, not asserted (STOP rule)", () => {
  const { ring } = explore();
  const stride = Math.max(1, Math.floor(ring.length / SAMPLE_CAP));
  const sampled = ring.filter((_, i) => i % stride === 0).slice(0, SAMPLE_CAP);
  const rows = [];
  sampled.forEach((t, i) => {
    const { player, messages, username } = actor(t, `frontier-${i}`);
    loginWithKey(player, username);
    rows.push(castAndDrive(player, messages, username, t));
    cleanup();
  });
  for (const r of rows) console.log(`# REACH frontier ${JSON.stringify(r)}`);
  const notLanded = rows.filter((r) => !r.keyInInventory);
  console.log(`# REACH frontier summary: ${rows.length} sampled, ${rows.length - notLanded.length} landed, ${notLanded.length} not landed`);
  for (const r of notLanded) console.log(`# REACH NOT-LANDED ${JSON.stringify(r)}`);
});

test("sourced out-of-range sample: SHOES_TILE, reported", () => {
  const { player, messages, username } = actor(SHOES_TILE, "shoes");
  loginWithKey(player, username);
  const out = castAndDrive(player, messages, username, SHOES_TILE);
  console.log(`# REACH shoes-tile ${JSON.stringify(out)}`);
  cleanup();
});


// ---------------------------------------------------------------------------------------------------------
// Report-only extension (3a sight set, 3a' blocker identification, 3b Ghaslor conversation tiles).
// Retail datum used as a SOURCED START (private RSProx corpus rsprox-corpus-v001, decoded header revision 231.1,
// recording 3018, transcript SHA256 a827c61063d46f68e5f93f5314b32dcc3bb29b5a382b93610c26bcdfc9322612,
// lines 2400-2640; decoder build and capture date UNKNOWN): the player walked to (3428,2929) (MoveGameClick lines
// 2554-2560; local PlayerInfo coord 3428,2929 at line 2576), stood idle for three ticks, then sent
// OpObjT(id=6792, x=3432, z=2928, interface 218 component 27) at line 2621; within the same tick the server sent
// SetMapFlag (57,48) = (3433,2928) with build base (3376,2880) from RebuildNormalV1 zone 428/366 (line 2400),
// cleared it (line 2631), and played Spotanim 142 / MapProjAnimV2 143 / MapAnim 144 on the key tile.
// So retail resolved this cast at once from (3428,2929), and retail's destination flag was the same east tile
// (3433,2928) that the closest-tile ground-item route picks here. No recording in the snapshot shows a cast on this
// key (or any item) started outside sight.
// ---------------------------------------------------------------------------------------------------------
const RETAIL_ORIGIN = new Location(3428, 2929, 0);
const PROJ_FLAG_NAMES = Object.getOwnPropertyNames(RegionManager)
  .filter((k) => /^PROJECTILE_|^UNKNOWN$|^UNLOADED_TILE$/.test(k) && typeof RegionManager[k] === "number")
  .map((k) => [k, RegionManager[k]]);
const flagNames = (clip) => PROJ_FLAG_NAMES.filter(([, v]) => (clip & v) !== 0).map(([k]) => k);

function objectsAt(x, y) {
  return MapObjects.getObjectsAt(new Location(x, y, 0), null).map((o) => ({
    id: o.getId(), name: ObjectDefinition.forId(o.getId())?.getName?.() ?? "?", type: o.getType(), face: o.getFace(),
  }));
}

/** Exact route cost the combat route helper uses (PathFinder.calculateRouteToAny): exact-tile findRoute, no moveNear. */
function routeCost(from, to) {
  const route = PathFinder.rsmodRouteFinding.findRoute({
    level: 0, srcX: from.getX(), srcY: from.getY(), srcSize: 1, destX: to.getX(), destY: to.getY(),
    destWidth: 1, destLength: 1, locShape: -1, moveNear: false, privateArea: null,
  });
  if (!route.success) return null;
  let cost = 0, fx = from.getX(), fy = from.getY();
  for (const w of route.waypoints) { cost += Math.max(Math.abs(w.x - fx), Math.abs(w.y - fy)); fx = w.x; fy = w.y; }
  return cost;
}

/** A port of RegionManager.canProjectileRay (RegionManager.ts:657-710) that records every blocking check instead of stopping at the first. */
function rayTrace(sx, sy, ex, ey) {
  const hits = [];
  if (sx === ex && sy === ey) return { blocked: false, hits };
  const R = RegionManager;
  const dx = ex - sx, dy = ey - sy, east = dx >= 0, north = dy >= 0;
  const xFlags = R.UNLOADED_TILE | R.UNKNOWN | R.PROJECTILE_TILE_BLOCKED | (east ? R.PROJECTILE_WEST_BLOCKED : R.PROJECTILE_EAST_BLOCKED);
  const yFlags = R.UNLOADED_TILE | R.UNKNOWN | R.PROJECTILE_TILE_BLOCKED | (north ? R.PROJECTILE_SOUTH_BLOCKED : R.PROJECTILE_NORTH_BLOCKED);
  const check = (x, y, flags, atEnd, axis) => {
    const mask = atEnd ? flags & ~R.PROJECTILE_TILE_BLOCKED : flags;
    const clip = R.getProjectileClipping(x, y, 0, null);
    if ((clip & mask) !== 0) {
      hits.push({ tile: `${x},${y}`, axis, clip: `0x${clip.toString(16)}`, blockedBy: flagNames(clip & mask), objects: objectsAt(x, y) });
    }
  };
  const scale = 65536, half = scale / 2;
  if (Math.abs(dx) > Math.abs(dy)) {
    const off = east ? 1 : -1;
    let sc = sy * scale + half + (north ? 0 : -1);
    const tan = Math.trunc(dy * scale / Math.abs(dx));
    for (let x = sx; x !== ex;) {
      x += off;
      const y = Math.floor(sc / scale);
      check(x, y, xFlags, x === ex && y === ey, "x");
      sc += tan;
      const ny = Math.floor(sc / scale);
      if (ny !== y) check(x, ny, yFlags, x === ex && ny === ey, "y");
    }
  } else {
    const off = north ? 1 : -1;
    let sc = sx * scale + half + (east ? 0 : -1);
    const tan = Math.trunc(dx * scale / Math.abs(dy));
    for (let y = sy; y !== ey;) {
      y += off;
      const x = Math.floor(sc / scale);
      check(x, y, yFlags, x === ex && y === ey, "y");
      sc += tan;
      const nx = Math.floor(sc / scale);
      if (nx !== x) check(nx, y, xFlags, nx === ex && y === ey, "x");
    }
  }
  return { blocked: hits.length > 0, hits };
}

test("REACH 3a: the sight set (tiles within range that satisfy the approach predicate), its map and route costs", () => {
  const { seen } = explore();
  const { player: probe } = actor(IN_RANGE_SEED, "probe-3a");
  const sight = [];
  for (let dx = -GRAB_RANGE; dx <= GRAB_RANGE; dx++) for (let dy = -GRAB_RANGE; dy <= GRAB_RANGE; dy++) {
    const t = new Location(KEY_TILE.getX() + dx, KEY_TILE.getY() + dy, 0);
    if (approachAt(probe, t)) sight.push(t);
  }
  const connected = sight.filter((t) => seen.has(tileKey(t)));
  const xs = sight.map((t) => t.getX()), ys = sight.map((t) => t.getY());
  console.log(`# REACH 3a sightSet count=${sight.length} connectedToSeed=${connected.length} bbox x ${Math.min(...xs)}..${Math.max(...xs)} y ${Math.min(...ys)}..${Math.max(...ys)}`);
  assert.ok(sight.length > 0, "precondition: the sight set is non-empty");
  // A picture of the box: K key, S sight tile connected to the seed, s sight tile not connected, . walkable connected tile, ' ' not walkable/not reached.
  const sightKeys = new Set(sight.map(tileKey));
  for (let y = KEY_TILE.getY() + GRAB_RANGE; y >= KEY_TILE.getY() - GRAB_RANGE; y--) {
    let row = "";
    for (let x = KEY_TILE.getX() - GRAB_RANGE; x <= KEY_TILE.getX() + GRAB_RANGE; x++) {
      const k = `${x},${y}`;
      row += x === KEY_TILE.getX() && y === KEY_TILE.getY() ? "K" : sightKeys.has(k) ? (seen.has(k) ? "S" : "s") : seen.has(k) ? "." : " ";
    }
    console.log(`# REACH 3a map y=${y} x=${KEY_TILE.getX() - GRAB_RANGE}..${KEY_TILE.getX() + GRAB_RANGE} |${row}|`);
  }
  const starts = [["ghaslor-spawn", GHASLOR_SPAWN], ["shoes", SHOES_TILE], ["east-route-end", new Location(3433, 2928, 0)], ["retail-origin", RETAIL_ORIGIN], ["walk-proof-start", new Location(3421, 2927, 0)]];
  for (const [label, from] of starts) {
    const costs = sight.map((t) => ({ t, c: routeCost(from, t) })).filter((e) => e.c !== null).sort((a, b) => a.c - b.c || a.t.getX() - b.t.getX() || a.t.getY() - b.t.getY());
    console.log(`# REACH 3a routeToSight ${label} from=${tileKey(from)} reachableSightTiles=${costs.length}/${sight.length} nearest=${JSON.stringify(costs.slice(0, 4).map((e) => ({ tile: tileKey(e.t), cost: e.c, chebyshevToKey: chebyshev(e.t, KEY_TILE) })))}`);
  }
});

test("REACH 3a': what blocks the projectile line from each start to the key (port of the ray, agreement with production asserted)", () => {
  const starts = [["ghaslor-spawn", GHASLOR_SPAWN], ["shoes", SHOES_TILE], ["east-route-end", new Location(3433, 2928, 0)],
    ["retail-origin", RETAIL_ORIGIN], ["in-range-seed", IN_RANGE_SEED], ["walk-proof-start", new Location(3421, 2927, 0)]];
  console.log(`# REACH 3a' keyTile clip=0x${RegionManager.getProjectileClipping(KEY_TILE.getX(), KEY_TILE.getY(), 0, null).toString(16)} blockedBy=${JSON.stringify(flagNames(RegionManager.getProjectileClipping(KEY_TILE.getX(), KEY_TILE.getY(), 0, null)))} objects=${JSON.stringify(objectsAt(KEY_TILE.getX(), KEY_TILE.getY()))}`);
  for (const [label, from] of starts) {
    const port = rayTrace(from.getX(), from.getY(), KEY_TILE.getX(), KEY_TILE.getY());
    const production = RegionManager.canProjectileAttackBounds(from, KEY_TILE, 1, 1, 1, 1, null);
    console.log(`# REACH 3a' ray ${label} from=${tileKey(from)} portBlocked=${port.blocked} productionClear=${production} hits=${JSON.stringify(port.hits.slice(0, 4))}`);
    assert.equal(!port.blocked, production, `the ray port agrees with RegionManager.canProjectileAttackBounds for ${label}; otherwise the blocker report is unreliable`);
  }
});

test("REACH 3b: the Ghaslor conversation tiles (player adjacent to each of his 25 wander tiles) as report-only starts", () => {
  const { seen } = explore();
  const { player: probe } = actor(IN_RANGE_SEED, "probe-3b");
  const starts = new Map();
  for (let nx = -2; nx <= 2; nx++) for (let ny = -2; ny <= 2; ny++) {
    const npcTile = new Location(GHASLOR_SPAWN.getX() + nx, GHASLOR_SPAWN.getY() + ny, 0);
    if (!seen.has(tileKey(npcTile))) continue; // Ghaslor must be able to stand there
    for (let dx = -1; dx <= 1; dx++) for (let dy = -1; dy <= 1; dy++) {
      if (!dx && !dy) continue;
      const t = new Location(npcTile.getX() + dx, npcTile.getY() + dy, 0);
      if (seen.has(tileKey(t))) starts.set(tileKey(t), t);
    }
  }
  const all = [...starts.values()].sort((a, b) => a.getX() - b.getX() || a.getY() - b.getY());
  const withPredicate = all.map((t) => ({ t, p: approachAt(probe, t) }));
  const yes = withPredicate.filter((e) => e.p).length;
  const d = all.map((t) => chebyshev(t, KEY_TILE));
  console.log(`# REACH 3b ghaslorConversationTiles total=${all.length} predicateTrue=${yes} predicateFalse=${all.length - yes} chebyshev ${Math.min(...d)}..${Math.max(...d)}`);
  assert.ok(all.length > 0, "precondition: Ghaslor conversation tiles exist");
  const cap = 30, stride = Math.max(1, Math.floor(all.length / cap));
  const sample = withPredicate.filter((_, i) => i % stride === 0).slice(0, cap);
  const rows = [];
  sample.forEach((e, i) => {
    const { player, messages, username } = actor(e.t, `ghaslor-conv-${i}`);
    loginWithKey(player, username);
    rows.push(castAndDrive(player, messages, username, e.t));
    cleanup();
  });
  for (const r of rows) console.log(`# REACH 3b conv ${JSON.stringify(r)}`);
  const byPred = (v) => rows.filter((r) => r.predicateAtStart === v);
  console.log(`# REACH 3b summary sampled=${rows.length} predicateTrue landed=${byPred(true).filter((r) => r.keyInInventory).length}/${byPred(true).length} predicateFalse landed=${byPred(false).filter((r) => r.keyInInventory).length}/${byPred(false).length}`);
  for (const r of rows.filter((r) => !r.keyInInventory)) console.log(`# REACH 3b NOT-LANDED ${JSON.stringify(r)}`);
});

test("REACH retail-observed start (3428,2929): the tile a retail player cast the key grab from (recording 3018), report-only", () => {
  const { player, messages, username } = actor(RETAIL_ORIGIN, "retail-origin");
  assert.ok(loginWithKey(player, username));
  const out = castAndDrive(player, messages, username, RETAIL_ORIGIN);
  console.log(`# REACH retail-origin ${JSON.stringify(out)}`);
  console.log(`# REACH retail-origin note: retail resolved the cast at once from here; predicateAtStart=${out.predicateAtStart} here (false would mean this server's clipping differs from retail at this tile)`);
  cleanup();
});
