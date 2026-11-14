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
