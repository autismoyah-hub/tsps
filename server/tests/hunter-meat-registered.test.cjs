// Seeded actors and manually sequenced world phases: no client rendering or natural login.
// Run from server after a matching build and cache: node --test tests/hunter-meat-registered.test.cjs
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after, test } = require('node:test');
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager: PM } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');
const { ClientConnection } = require('../dist/net/NetworkBuilder');
const Food = require('../plugins/items/Food.plugin');
const Potions = require('../plugins/items/Potions.plugin');
let core, nextActor = 1, initialCycle, setupCompromised = false;
const errors = [], originalError = console.error;

before(async () => {
  console.error = (...args) => { errors.push(args.map(String).join(' ')); originalError(...args); };
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  core = PM.getCoreApi(); initialCycle = core.World.processCycle;
  Food.register(PM.createApi('HunterFixtureFood'));
  Potions.register(PM.createApi('HunterFixturePotions'));
});
after(() => {
  if (core && initialCycle !== undefined) core.World.processCycle = initialCycle;
  console.error = originalError;
  assert.deepEqual(errors, [], 'registration, task and hook errors must remain visible');
});
function actor(t, hp = 10, energy = 50) {
  assert.equal(setupCompromised, false, 'prior partial player registration: do not continue');
  const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => true,
    send: () => true, close() {}, onData() {}, onError() {}, onClose() {} };
  const session = new PlayerSession(channel), p = new Player(session);
  session.setPlayer(p); p.setUsername('hunterfixture' + nextActor++);
  p.setAttribute('bot-skip-persistence', true);
  p.setLocation(new core.Location(3222, 3218, 0));
  // A throwing onAdd can allocate a slot before MobileList increments size.
  // Do not remove that partial entry or run subsequent cases in a corrupted list.
  let added = false, addStarted = false;
  t.after(() => {
    if (addStarted && !added) {
      setupCompromised = true;
      assert.fail('player add did not complete; partial world state retained; process is unusable');
    }
    if (!added) return;
    setupCompromised = true;
    const index = p.getIndex();
    core.TaskManager.cancelTasks(p); core.TaskManager.cancelTasks(index);
    if (core.World.getPlayers().get(index) === p) core.World.getPlayers().remove(p);
    assert.notEqual(core.World.getPlayers().get(index), p, 'allocated player slot released');
    assert.equal(p.isRegistered(), false);
    const remaining = [...core.TaskManager.pendingTasks.toArray(), ...core.TaskManager.activeTasks];
    assert.equal(remaining.some(task => (task.key === p || task.key === index) && task.isRunning()), false,
      'actor tasks stopped (queue drain is not asserted)');
    setupCompromised = false;
  });
  addStarted = true;
  const accepted = core.World.getPlayers().add(p);
  if (!accepted) { addStarted = false; assert.fail('world rejected player registration'); }
  added = true;
  assert.ok(p.getIndex() >= 0 && p.getIndex() < 2048);
  assert.equal(core.World.getPlayers().get(p.getIndex()), p);
  assert.equal(p.isRegistered(), true); p.needsPlacement = false;
  const skills = p.getSkillManager();
  skills.setMaxLevel(core.Skill.HITPOINTS, 99); skills.setCurrentLevel(core.Skill.HITPOINTS, hp, false);
  p.setRunEnergy(energy);
  const c = new ClientConnection(channel); c.player = p;
  const observer = 'HunterFixtureObserver' + p.getIndex(), events = [];
  t.after(() => {
    for (const name of ['food:eaten', 'food:delayed-heal', 'potions:drunk']) {
      const hooks = PM.customEventHooks.get(name) ?? [];
      PM.customEventHooks.set(name, hooks.filter(h => h.pluginName !== observer));
    }
  });
  for (const name of ['food:eaten', 'food:delayed-heal', 'potions:drunk']) {
    PM.createApi(observer).onCustomEvent(name, e => {
      if (e.player === p) events.push({ name, cycle: core.World.getProcessCycle(), itemId: e.itemId, meta: e.meta, heal: e.heal });
    });
  }
  return { p, c, inv: p.getInventory(), events };
}
function put(f, slot, id, meta) { f.inv.setItem(slot, new core.Item(id, 1, meta)); }
async function click(f, slot, option) {
  const item = f.inv.getItems()[slot]; assert.ok(item, 'clicked item exists');
  const id = item.getId(), definition = core.CacheDefinitions.getItem(id);
  assert.equal(definition.inventoryActions[0], option, 'cache first action matches click');
  const widget = core.CacheDefinitions.getComponentId('inventory:items'); assert.ok(widget > 0);
  await f.c.dispatch([{ type: 'widget_action', widgetId: widget, groupId: widget >>> 16,
    childId: widget & 65535, slot, itemId: id, buttonNum: 2, opId: 2 }]);
}
// World processes tasks first; Player processes input and then its real timers.
function turn(f) {
  f.p.resetUpdating(); core.World.processCycle = (core.World.processCycle + 1) & 0x7fffffff;
  core.TaskManager.process(); f.c.processTurnPackets(); f.p.getTimers().process();
}
function events(f, name) { return f.events.filter(e => e.name === name); }
function assertBite(f, item, meta, hp) {
  assert.equal(f.p.getHitpoints(), hp);
  const e = events(f, 'food:eaten'); assert.equal(e.length, 1); assert.equal(e[0].itemId, item);
  assert.deepEqual(e[0].meta, meta); assert.equal(f.p.getAttribute('food:eaten'), 1);
  assert.equal(f.p.getTimers().left(core.TimerKey.FOOD), 2);
}
async function start(f) {
  const id = core.ItemIdentifiers.COOKED_DASHING_KEBBIT, meta = { preset: 'hunter-test' };
  put(f, 0, id, meta); await click(f, 0, 'Eat'); turn(f);
  assert.equal(f.inv.getAmount(id), 0); assertBite(f, id, meta, 23);
  assert.equal(f.p.getRunEnergy(), 50); assert.equal(events(f, 'food:delayed-heal').length, 0);
  return { id, cycle0: core.World.getProcessCycle() };
}
function advance(f, n) { for (let i = 0; i < n; i++) turn(f); }
function assertDelayed(f, id, cycle, hp, energy) {
  const e = events(f, 'food:delayed-heal'); assert.equal(e.length, 1);
  assert.equal(e[0].itemId, id); assert.equal(e[0].cycle, cycle);
  assert.equal(f.p.getHitpoints(), hp); assert.equal(f.p.getRunEnergy(), energy);
}

test('dashing kebbit fires once on registered tick seven', async t => {
  const f = actor(t), { id, cycle0 } = await start(f);
  advance(f, 6); assert.equal(f.p.getHitpoints(), 23); assert.equal(events(f, 'food:delayed-heal').length, 0);
  turn(f); assertDelayed(f, id, cycle0 + 7, 33, 60);
  turn(f); assertDelayed(f, id, cycle0 + 7, 33, 60);
  assert.equal(events(f, 'food:eaten').length, 1); assert.equal(f.p.getAttribute('food:eaten'), 1);
});

test('ordinary food preserves the pending effect after a timer-admitted click', async t => {
  const f = actor(t), { id, cycle0 } = await start(f), trout = core.ItemIdentifiers.TROUT;
  put(f, 1, trout, { preset: 'trout-test' }); advance(f, 2); await click(f, 1, 'Eat'); turn(f);
  assert.equal(f.inv.getAmount(trout), 0); assert.equal(f.p.getHitpoints(), 30);
  assert.equal(events(f, 'food:eaten').length, 2); assert.deepEqual(events(f, 'food:eaten')[1].meta, { preset: 'trout-test' });
  assert.equal(f.p.getTimers().left(core.TimerKey.FOOD), 2);
  advance(f, 3); assert.equal(events(f, 'food:delayed-heal').length, 0);
  turn(f); assertDelayed(f, id, cycle0 + 7, 40, 60); turn(f);
  assert.equal(events(f, 'food:delayed-heal').length, 1); assert.equal(f.p.getAttribute('food:eaten'), 2);
});

test('ordinary potion preserves the pending effect and its dose metadata', async t => {
  const f = actor(t), { id, cycle0 } = await start(f), ids = core.ItemIdentifiers;
  put(f, 1, ids.ATTACK_POTION_4_, { preset: 'potion-test' }); advance(f, 2); await click(f, 1, 'Drink'); turn(f);
  assert.equal(f.inv.getItems()[1].getId(), ids.ATTACK_POTION_3_);
  assert.deepEqual(f.inv.getItems()[1].getMeta(), { preset: 'potion-test' });
  assert.equal(events(f, 'potions:drunk').length, 1); assert.equal(f.p.getTimers().left(core.TimerKey.FOOD), 2);
  advance(f, 3); assert.equal(events(f, 'food:delayed-heal').length, 0);
  turn(f); assertDelayed(f, id, cycle0 + 7, 33, 60); turn(f);
  assert.equal(events(f, 'food:delayed-heal').length, 1); assert.equal(f.p.getAttribute('food:eaten'), 1);
});

test('a later admitted hunter meal replaces the first effect', async t => {
  const f = actor(t), { cycle0 } = await start(f), graahk = core.ItemIdentifiers.COOKED_GRAAHK;
  put(f, 1, graahk, { preset: 'graahk-test' }); advance(f, 2); await click(f, 1, 'Eat'); turn(f);
  assert.equal(f.inv.getAmount(graahk), 0); assert.equal(f.p.getHitpoints(), 31);
  assert.equal(events(f, 'food:eaten').length, 2); assert.deepEqual(events(f, 'food:eaten')[1].meta, { preset: 'graahk-test' });
  assert.equal(f.p.getTimers().left(core.TimerKey.FOOD), 2);
  advance(f, 4); assert.equal(core.World.getProcessCycle(), cycle0 + 7);
  assert.equal(f.p.getHitpoints(), 31); assert.equal(events(f, 'food:delayed-heal').length, 0);
  advance(f, 3); assertDelayed(f, graahk, cycle0 + 10, 37, 50); turn(f);
  assert.equal(events(f, 'food:delayed-heal').length, 1); assert.equal(f.p.getAttribute('food:eaten'), 2);
});

test('secondary heal and energy respect caps', async t => {
  const f = actor(t, 85, 95), id = core.ItemIdentifiers.COOKED_DASHING_KEBBIT;
  put(f, 0, id, { preset: 'cap-test' }); await click(f, 0, 'Eat'); turn(f);
  assert.equal(f.inv.getAmount(id), 0); assertBite(f, id, { preset: 'cap-test' }, 98);
  const cycle0 = core.World.getProcessCycle(); advance(f, 6);
  assert.equal(f.p.getHitpoints(), 98); assert.equal(f.p.getRunEnergy(), 95);
  turn(f); assertDelayed(f, id, cycle0 + 7, 99, 100); turn(f);
  assert.equal(events(f, 'food:delayed-heal').length, 1); assert.equal(f.p.getAttribute('food:eaten'), 1);
});
