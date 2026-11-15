// #5693 measurement (not for commit): real-route repair of the Myreque bridge rungs.
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after, test } = require('node:test');
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager: PM } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');

const STAGE = 'quest.in_search_of_the_myreque.stage';
const BRIDGE = 'quest.in_search_of_the_myreque.bridge';
const REACHED_HOLLOWS = 25, SHOWN_WAY_OUT = 90, IN_INN_BASEMENT = 95;
const WALL = 5052, TREE = 5003, RUNG_BOTTOM = 26245, RUNG_TOP = 26247;
let core, next = 2800;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins')); core = PM.getCoreApi();
  PM.emitServerStartup({});
});
after(() => PM.emitServerShutdown({}));

function actor(t, stage, x, y, bridge = 0) {
  const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => true, send: () => true, close() {}, onData() {}, onError() {}, onClose() {}, flush() {} };
  const p = new Player(new PlayerSession(channel)); p.setIndex(next++); p.setUsername('myrequebw' + next);
  p.setRegistered(true); p.setLocation(new core.Location(x, y, 0)); p.needsPlacement = false;
  p.setAttribute(STAGE, stage);
  if (bridge) p.setAttribute(BRIDGE, bridge);
  p.messages = [];
  p.sendMessage = (text) => { p.messages.push(text); };
  t.after(() => { p.setRegistered(false); core.TaskManager.cancelTasks(p.getIndex()); });
  return p;
}
const at = (p) => { const l = p.getLocation(); return [l.getX(), l.getY(), l.getZ()]; };
function click(p, id, x, y, type, face) {
  const object = new core.GameObject(id, new core.Location(x, y, 0), type, face, null);
  const event = { player: p, object, objectId: id, location: object.getLocation(), clickType: 1 };
  PM.emitObjectInteraction(event);
  return event;
}
const wall = (p) => click(p, WALL, 3480, 9837, 0, 3);
const tree = (p, y, face) => click(p, TREE, 3502, y, 10, face);
const rung = (p, id, y) => click(p, id, 3502, y, 22, 3);
const tick = (n) => { for (let i = 0; i < n; i++) core.TaskManager.process(); };

test('the wall passes from the south side to the north side (rotation 3 wall on the (3480,9837) south edge)', (t) => {
  const p = actor(t, SHOWN_WAY_OUT, 3480, 9836);
  const event = wall(p);
  assert.ok(event.handled);
  assert.equal(p.getAttribute(STAGE), IN_INN_BASEMENT, 'stage 90 moves to 95');
  tick(6);
  assert.deepEqual(at(p), [3480, 9837, 0]);
  assert.ok(p.messages.includes('You search the wall and find the door which Veliaf told you about.'), p.messages.join('|'));
  assert.ok(p.messages.includes('You walk through.'), p.messages.join('|'));
});

const { ObjectDefinition } = require('../dist/game/definition/ObjectDefinition');
const { MapObjects } = require('../dist/game/entity/impl/object/MapObjects');
const { RegionManager } = require('../dist/game/collision/RegionManager');
const { CacheDefinitions } = require('../dist/game/cache/CacheDefinitions');

function step(p) { core.World.processCycle++; core.TaskManager.process(); const q = p.getMovementQueue(); q.beginCycle?.(); if (q.hasPendingWork?.()) q.process(); core.TaskManager.processWalkTo(p.getIndex()); }
function objectsAt(x, y) { return MapObjects.getObjectsAt(new core.Location(x, y, 0), null); }

test('M0 real objects around the bridge (placed ids, cache transforms)', (t) => {
  RegionManager.init();
  const p = actor(t, REACHED_HOLLOWS, 3502, 3427);
  for (let x = 3502; x <= 3503; x++) for (let y = 3424; y <= 3432; y++) {
    for (const o of objectsAt(x, y)) {
      const raw = CacheDefinitions.getObject(o.getId());
      const d = ObjectDefinition.forPlayer(o.getId(), p);
      console.log(`MEASURE obj (${x},${y}) placed=${o.getId()} type=${o.getType()} face=${o.getFace?.()} rawName=${raw?.name} varbit=${raw?.transformVarbit} transforms=${JSON.stringify(raw?.transforms)} resolved=${d?.getId?.()}/${d?.getName?.()} acts=${JSON.stringify(d?.getActions?.())}`);
    }
  }
});

function routedObject(p, object) {
  const d = () => ObjectDefinition.forPlayer(object.getId(), p);
  const acts = d()?.getActions?.() ?? [];
  const clickType = Math.max(1, acts.findIndex(a => a === 'Repair') + 1);
  p.getMovementQueue().walkToObject(object, { execute: () => {
    const event = { player: p, object, objectId: object.getId(), location: object.getLocation(), clickType, definition: d(), handled: false };
    PM.emitObjectInteraction(event);
    console.log(`MEASURE dispatch placed=${object.getId()} resolved=${d()?.getId?.()} clickType=${clickType} handled=${event.handled}`);
  } }, clickType);
}

test('M1 repair the rungs through the real walk-to-object route from the landing', (t) => {
  RegionManager.init();
  const p = actor(t, REACHED_HOLLOWS, 3502, 3427);
  const { ItemIdentifiers: I } = core;
  p.getInventory().adds(I.PLANK, 3); p.getInventory().adds(I.STEEL_NAILS, 225); p.getInventory().adds(I.HAMMER, 1);
  for (const y of [3428, 3429, 3430]) {
    const object = objectsAt(3502, y).find(o => /26|50/.test(String(o.getId())));
    console.log(`MEASURE pick (3502,${y}) ->`, object?.getId?.());
    if (!object) continue;
    routedObject(p, object);
    for (let i = 0; i < 12; i++) step(p);
    console.log(`MEASURE after (3502,${y}): at=${JSON.stringify(at(p))} bridge=${p.getAttribute(BRIDGE)} msgs=${JSON.stringify(p.messages.slice(-2))}`);
  }
});

test('M2 control and route diagnostics', (t) => {
  RegionManager.init();
  const p = actor(t, REACHED_HOLLOWS, 3502, 3427);
  const q = p.getMovementQueue();
  const run = (label, object, clickType) => {
    let executed = 0;
    q.walkToObject(object, { execute: () => { executed++; } }, clickType);
    const info = [`route=${q.hasRoute?.()}`, `path=${q.pathX},${q.pathY}`, `pending=${q.hasPendingWork?.()}`];
    for (let i = 0; i < 10; i++) step(p);
    console.log(`MEASURE diag ${label}: ${info.join(' ')} executed=${executed} at=${JSON.stringify(at(p))}`);
  };
  const tree = objectsAt(3502, 3426).find(o => o.getId() === 5003);
  run('control tree 5003 (3502,3426) Climb', tree, 2);
  const rung1 = objectsAt(3502, 3428).find(o => o.getId() === 26245);
  run('rung 26245 (3502,3428) Repair', rung1, 1);
  const bridge = objectsAt(3502, 3427).find(o => o.getId() === 5002);
  run('bridge 5002 (3502,3427) Cross', bridge, 1);
});
