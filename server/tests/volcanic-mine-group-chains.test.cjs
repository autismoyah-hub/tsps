// Run after yarn build: node --test tests/volcanic-mine-group-chains.test.cjs
// Real cache, Player, instance, registered object handlers and TaskManager ticks.
// Skills, paid access and standing tiles are supplied; no network/rendering/walking claim.
process.env.DISABLE_PLAYER_BOTS = '1';
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after, test } = require('node:test');
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager: PM } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');
const { SkillManager } = require('../dist/game/content/skill/SkillManager');
const { PathFinder } = require('../dist/game/model/movement/path/PathFinder');
let core, Common, Game;
let serial = 0;
const errors = [];
const originalError = console.error;
before(async () => {
  console.error = (...args) => { errors.push(args); originalError(...args); };
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  core = PM.getCoreApi();
  core.ObjectDefinition.init();
  core.RegionManager.init();
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins'));
  PM.emitServerStartup({});
  Common = require('../plugins/minigames/volcanicmine/Common.VolcanicMine');
  Game = require('../plugins/minigames/volcanicmine/Game.VolcanicMine');
  core.RegionManager.loadMapFiles(...Common.data.tiles.entrance.slice(0, 2));
});
after(() => {
  try { PM.emitServerShutdown({}); }
  finally { console.error = originalError; assert.deepEqual(errors, [], 'no caught plugin/runtime errors'); }
});
function ticks(n) { for (let i = 0; i < n; i++) core.TaskManager.process(); }
function click(p, object, option) {
  const definition = core.ObjectDefinition.forPlayer(object.getId(), p);
  const clickType = definition.getInteractions().findIndex(s => s?.toLowerCase() === option.toLowerCase()) + 1;
  assert.ok(clickType > 0, `active cache exposes ${option}`);
  const at = object.getLocation();
  const location = { x: at.getX(), y: at.getY(), z: at.getZ() };
  const source = p.getPrivateArea()?.templateOf?.(at);
  const sourceLocation = source ? { x: source[0], y: source[1], z: source[2] } : { ...location };
  assert.equal(PM.emitObjectInteraction({ player: p, object, objectId: definition.id, definition,
    clickType, option: option.toLowerCase(), location, sourceLocation, handled: false }), true);
}
function group(t, count) {
  const players = [];
  let game;
  t.after(() => {
    try {
      if (game) {
        for (const p of players) if (game.players.has(p)) game.remove(p);
        game.end();
        ticks(6);
        assert.equal(game.placed.size, 0, 'all placed instance objects removed');
        assert.equal(game.area.isDestroyed(), true, 'instance disposed after its real delay');
        assert.equal(game.area.spawned.length, 0, 'owned NPC list disposed');
      }
    } finally {
      for (const p of players) {
        p.closeInterruptibleInterfaces();
        core.TaskManager.cancelTasks(p);
        core.TaskManager.cancelTasks(p.getIndex());
        if (p.isRegistered()) core.World.getPlayers().remove(p);
        assert.equal(p.isRegistered(), false);
        assert.equal(Game.gameOf(p), null);
        assert.equal(core.TaskManager.hasActiveTask(p), false);
      }
      assert.deepEqual(errors, []);
    }
  });
  for (let i = 0; i < count; i++) {
    const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => true,
      send: () => true, close() {}, onData() {}, onError() {}, onClose() {} };
    const p = new Player(new PlayerSession(channel));
    players.push(p);
    const name = `vmgroup${++serial}`;
    p.setUsername(name); p.setLongUsername(core.Misc.stringToLongBigInt(name));
    p.setAttribute('bot-skip-persistence', true);
    assert.equal(core.World.getPlayers().add(p), true);
    p.getSession().setPlayer(p);
    p.getSkillManager().setMaxLevels(core.Skill.MINING, 99, false)
      .setExperience(core.Skill.MINING, SkillManager.getExperienceForLevel(99))
      .setCurrentLevels(core.Skill.MINING, 99, false);
    p.getInventory().adds(core.ItemIdentifiers.DRAGON_PICKAXE, 1);
    p.setAttribute(Common.data.attributes.paid, true);
    p.moveTo(new core.Location(3813, 3806, 0)); p.setNeedsPlacement(false);
    const entrance = core.World.findCacheObject(p, Common.data.locs.entrance, new core.Location(...Common.data.tiles.entrance));
    assert.ok(entrance, 'real entrance exists');
    click(p, entrance, 'Climb-down');
    game ??= Game.gameOf(p);
    assert.ok(game, 'registered entrance creates a game');
    assert.equal(Game.gameOf(p), game, 'all players join the same instance');
    assert.equal(game.phase, 'lobby');
    assert.ok(p.getInventory().contains(Common.data.items.vessel));
  }
  assert.equal(game.players.size, count);
  // Suppress unrelated random hazards using the existing fixture seam, as the central suite does.
  game.quiet = true;
  ticks(Common.data.timing.lobby);
  assert.equal(game.phase, 'running', 'real lobby timer starts mining');
  assert.equal(game.players.size, count);
  return { players, game };
}
function row(game, p, object) {
  const definition = core.ObjectDefinition.forPlayer(object.getId(), p);
  return { rawId: object.getId(), activeId: definition?.id ?? null,
    tile: game.area.templateOf(object.getLocation()), shape: object.getType(), face: object.getFace(),
    size: definition ? [definition.getSizeX(), definition.getSizeY()] : null,
    actions: definition?.getInteractions() ?? [] };
}
function scene(game, p, report = false) {
  const raw = [];
  for (let cx = 0; cx < Common.data.instance.widthChunks; cx++) {
    for (let cy = 0; cy < Common.data.instance.heightChunks; cy++) raw.push(...game.area.getChunkObjects(1, cx, cy));
  }
  const ids = new Set([...Common.data.locs.boulders, Common.data.locs.whirlpool]);
  const candidates = [...raw, ...game.placed.values()].filter(o => ids.has(o.getId()));
  const effective = new Map();
  for (const object of candidates) {
    const active = core.ObjectDefinition.forPlayer(object.getId(), p);
    if (!active) continue;
    const resolved = core.World.findCacheObject(p, active.id, object.getLocation());
    if (resolved) effective.set(`${resolved.getId()}:${game.area.templateOf(resolved.getLocation()).join(',')}`, resolved);
  }
  if (report) {
    const lava = raw.filter(o => o.getId() === Common.data.locs.lava).map(o => game.area.templateOf(o.getLocation()));
    const lavaRows = [];
    for (let y = Common.data.instance.source[1]; y < Common.data.instance.source[1] + Common.data.instance.heightChunks * 8; y++) {
      const xs = [...new Set(lava.filter(tile => tile[1] === y).map(tile => tile[0]))].sort((a, b) => a - b);
      const runs = [];
      for (const x of xs) {
        if (runs.length && runs.at(-1)[1] === x - 1) runs.at(-1)[1] = x;
        else runs.push([x, x]);
      }
      if (runs.length) lavaRows.push({ y, runs });
    }
    console.log(JSON.stringify({ event: 'volcanic-mine-template-map', target: 'osrs-242_2026-11-03',
      source: Common.data.instance.source, plane: 1, participants: game.players.size,
      rawObjects: raw.length, rawBoulders: raw.filter(o => ids.has(o.getId())).map(o => row(game, p, o)),
      placedBoulders: [...game.placed.values()].filter(o => ids.has(o.getId())).map(o => row(game, p, o)),
      resolvedCandidates: [...effective.values()].map(o => row(game, p, o)), lavaRows,
      limitation: 'Cache fallback can expose template candidates without independent runtime chains; not observed retail side routing or timing' }));
  }
  return [...effective.values()].filter(o => o.getId() === Common.data.locs.boulders[0]);
}
function stand(p, object) {
  const def = core.ObjectDefinition.forPlayer(object.getId(), p), at = object.getLocation();
  for (let dx = -1; dx <= def.getSizeX(); dx++) for (let dy = -1; dy <= def.getSizeY(); dy++) {
    if (dx !== -1 && dy !== -1 && dx !== def.getSizeX() && dy !== def.getSizeY()) continue;
    const tile = new core.Location(at.getX() + dx, at.getY() + dy, at.getZ());
    if (core.RegionManager.blocked(tile, p.getPrivateArea())) continue;
    p.moveTo(tile); p.setNeedsPlacement(false);
    if (PathFinder.reachedObject(p, at.getX(), at.getY(), def.getSizeX(), def.getSizeY(), object.getFace(), 10, def.getBlockingMask())) return;
  }
  assert.fail('cache supplies a walkable adjacent mining tile');
}
test('a solo lobby reaches a mineable central boulder through registered native handlers', t => {
  const { players: [p], game } = group(t, 1);
  const boulders = [...game.placed.values()].filter(o => o.getId() === Common.data.locs.boulders[0]);
  assert.equal(boulders.length, 1, 'solo positive control has one active attached boulder');
  stand(p, boulders[0]);
  const xp = p.getSkillManager().getExperience(core.Skill.MINING);
  click(p, boulders[0], 'Mine');
  ticks(8);
  assert.ok(p.getSkillManager().getExperience(core.Skill.MINING) > xp, 'registered mining action chips the existing boulder');
  assert.ok(game.points.get(p) > 0, 'chip contributes points');
});
test('twenty participants can address all three boulders and mine a side without advancing the central chain', t => {
  const { players, game } = group(t, 20);
  const attached = scene(game, players[0], true).sort((a, b) => a.getLocation().getX() - b.getLocation().getX());
  // Both Wiki15392920 and Boulder15329620 agree on all three channels at20+.
  assert.equal(attached.length, 3, 'feature-absence target: twenty players can address three attached boulders');
  const [west, central] = attached;
  assert.ok(west.getLocation().getX() < central.getLocation().getX());
  // A template fallback must not resurrect a runtime boulder after its removal.
  // Remember which candidates were actual placed objects before any mining action.
  const placedAt = object => [...game.placed.values()].some(o => o.getId() === object.getId()
    && o.getLocation().equals(object.getLocation()) && o.getType() === object.getType());
  const originallyPlaced = new Set(attached.filter(placedAt));
  const stillAttached = object => originallyPlaced.has(object) ? placedAt(object)
    : Boolean(core.World.findCacheObject(players[0], object.getId(), object.getLocation()));
  for (const p of players) { stand(p, west); click(p, west, 'Mine'); }
  let elapsed = 0;
  while (stillAttached(west) && stillAttached(central) && elapsed++ < 400) ticks(1);
  console.log(JSON.stringify({ event: 'volcanic-mine-group-mining-boundary', elapsed,
    phase: game.phase, collapsing: game.collapsing, ended: game.ended,
    participants: game.players.size, westStillAttached: stillAttached(west), centralStillAttached: stillAttached(central) }));
  assert.equal(game.phase, 'running', 'game must still be running at the mining boundary');
  assert.equal(game.collapsing, false, 'a collapse is not evidence of independent chain advancement');
  assert.equal(game.ended, false);
  assert.equal(game.players.size, 20);
  assert.equal(stillAttached(central), true, 'mining west must not consume the central boulder');
  assert.equal(stillAttached(west), false, 'ordinary group mining advances its selected side');
});
