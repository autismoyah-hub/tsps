// Proposed native regression. Run only after the fixture and its execution profile are reviewed.
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after, test } = require('node:test');
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');
const { ClientConnection } = require('../dist/net/NetworkBuilder');
const { SkillManager } = require('../dist/game/content/skill/SkillManager');
const { Task } = require('../dist/game/task/Task');
const Cooking = require('../plugins/skills/Cooking.plugin');

let core, range, runner;
let nextIndex = 2800;
let raw, cooked, burnt, chicken, cookedChicken, spare;

before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  core = PluginManager.getCoreApi();
  core.ObjectDefinition.init();
  core.RegionManager.init();
  core.RegionManager.loadMapFiles(3222, 3218);
  Cooking.register(PluginManager.createApi('CookingDropInterruption'));
  [raw, cooked, burnt, chicken, cookedChicken, spare] =
    ['raw_beef', 'cooked_meat', 'burnt_meat', 'raw_chicken', 'cooked_chicken', 'pot_empty']
      .map(name => core.CacheDefinitions.getObjId(name));
  for (const id of [raw, cooked, burnt, chicken, cookedChicken, spare]) {
    assert.ok(Number.isInteger(id) && id > 0, 'active cache resolves each ordinary item');
    assert.equal(core.ItemDefinition.forId(id).isNoted(), false);
    assert.equal(core.ItemDefinition.forId(id).isStackable(), false);
  }
  assert.ok(new core.Item(spare, 1).isDropable(), 'the unrelated pot is actually droppable');
  const rangeId = core.CacheDefinitions.getLocId('fai_varrock_range');
  range = new core.GameObject(rangeId, new core.Location(3222, 3218, 0), 10, 0);
  assert.equal(range.getDefinition().getName(), 'Range');
  assert.ok(range.getDefinition().getActions().includes('Cook'));
  assert.notEqual(rangeId, core.ObjectIdentifiers.COOKING_RANGE, 'ordinary non-quest range');
  core.MapObjects.add(range);
});

after(() => {
  assert.equal(Cooking._test.activeSessions.size, 0, 'all actors retired');
  if (runner) runner.stop();
  if (range) core.MapObjects.remove(range);
});

function actor(t) {
  const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => true,
    send: () => true, close() {}, onData() {}, onError() {}, onClose() {} };
  const player = new Player(new PlayerSession(channel));
  player.setIndex(nextIndex++).setUsername(`cookdrop${nextIndex}`).setRegistered(true);
  player.setLocation(new core.Location(3221, 3218, 0));
  player.setNeedsPlacement(false);
  const connection = new ClientConnection(channel);
  connection.player = player;
  const skills = player.getSkillManager();
  skills.setCurrentLevel(core.Skill.COOKING, 50, false).setMaxLevels(core.Skill.COOKING, 50, false)
    .setExperience(core.Skill.COOKING, SkillManager.getExperienceForLevel(50));
  const inventory = player.getInventory();
  inventory.adds(raw, 2);
  inventory.adds(spare, 1);
  const initialXp = skills.getExperience(core.Skill.COOKING);
  const ground = () => core.World.getItems().filter(g => !g.isPendingRemoval()
    && g.getOwner() === player.getUsername() && g.getItem().getId() === spare);
  t.after(() => {
    PluginManager.emitPlayerDisconnect({ player, username: player.getUsername() });
    assert.equal(Cooking.isCookingActive(player), false);
    player.setRegistered(false);
    core.TaskManager.cancelTasks(player.getIndex());
    for (const item of core.World.getItems().filter(g => !g.isPendingRemoval()
      && g.getOwner() === player.getUsername())) core.ItemOnGroundManager.deregister(item);
  });
  const state = () => ({ raw: inventory.getAmount(raw), cooked: inventory.getAmount(cooked),
    burnt: inventory.getAmount(burnt), xp: skills.getExperience(core.Skill.COOKING) - initialXp });
  return { player, connection, inventory, ground, state };
}

function admit(f, itemId = raw, amount = 2) {
  const slot = f.inventory.getSlotForItemId(itemId);
  const item = f.inventory.getItems()[slot];
  const event = { player: f.player, object: range, objectId: range.getId(), itemId,
    item, itemSlot: slot, slot, handled: false };
  assert.equal(PluginManager.emitItemOnObject(event), true, 'registered item-on-range handler accepts raw food');
  assert.equal(event.handled, true);
  const menu = f.player.getCreationMenu();
  assert.ok(menu, 'two raw foods expose the native CreationMenu');
  assert.deepEqual(menu.getItems(), [itemId]);
  const pendingBefore = new Set(core.TaskManager.pendingTasks.toArray());
  menu.execute(itemId, amount);
  assert.equal(Cooking.isCookingActive(f.player), true);
  if (!runner) {
    const submitted = core.TaskManager.pendingTasks.toArray().filter(task => !pendingBefore.has(task));
    assert.equal(submitted.length, 1, 'the first menu submission adds only the cooking runner');
    [runner] = submitted;
    assert.ok(Number.isInteger(runner.cycle), 'submitted cooking runner exposes its cycle');
    assert.equal(runner.key, Task.DEFAULT_KEY, 'shared runner is not keyed to the actor');
    assert.notEqual(runner.key, f.player.getIndex());
  }
}

async function turn(f, input) {
  f.player.resetUpdating();
  core.World.processCycle = (core.World.processCycle + 1) & 0x7fffffff;
  const cycle = runner.cycle;
  const errors = [];
  const originalError = console.error;
  console.error = (...args) => { errors.push(args); originalError(...args); };
  try { core.TaskManager.process(); } finally { console.error = originalError; }
  assert.deepEqual(errors, [], 'task manager did not catch an execution error');
  assert.equal(runner.cycle, cycle + 1);
  if (input) await input();
  f.connection.processTurnPackets();
  const movement = f.player.getMovementQueue();
  movement.beginCycle();
  movement.processEntityPursuit();
  if (movement.hasPendingWork()) movement.process();
  core.TaskManager.processWalkTo(f.player.getIndex());
}

async function drop(f) {
  const slot = f.inventory.getSlotForItemId(spare);
  assert.ok(slot >= 0);
  const cacheItem = core.CacheDefinitions.getItem(spare);
  assert.ok(cacheItem.inventoryActions.some(action => action?.toLowerCase() === 'drop'), 'cache exposes Drop');
  await f.connection.dispatch([{ type: 'inventory_action', widgetId: core.Inventory.INTERFACE_ID,
    slot, itemId: spare, option: 'Drop', optionIndex: 5 }]);
}

const first = () => ({ raw: 1, cooked: 1, burnt: 0, xp: 30 * core.GameConstants.EXPERIENCE_MULTIPLIER });
const second = () => ({ raw: 0, cooked: 2, burnt: 0, xp: 60 * core.GameConstants.EXPERIENCE_MULTIPLIER });
async function ticks(f, count) { for (let i = 0; i < count; i++) await turn(f); }
function assertDropped(f) {
  assert.equal(f.inventory.getAmount(spare), 0, 'native drop removed the unrelated item');
  assert.equal(f.ground().length, 1, 'native drop placed exactly one owned item on ground');
  assert.equal(f.ground()[0].getItem().getAmount(), 1);
  assert.equal(f.player.getMovementQueue().size(), 0);
}

for (const mode of ['no-drop', 'before-first', 'after-first', 'due-tick', 'walk', 'replacement', 'missing-food']) {
  test(`ordinary cooking: ${mode}`, async t => {
    const f = actor(t);
    if (mode === 'replacement') f.inventory.adds(chicken, 2);
    admit(f);
    assert.deepEqual(f.state(), { raw: 2, cooked: 0, burnt: 0, xp: 0 });
    if (mode === 'before-first') {
      await turn(f, () => drop(f));
      assertDropped(f);
      await ticks(f, 6);
      assert.deepEqual(f.state(), { raw: 2, cooked: 0, burnt: 0, xp: 0 });
      assert.equal(Cooking.isCookingActive(f.player), false);
      return;
    }
    await ticks(f, 2);
    assert.deepEqual(f.state(), first(), 'first normal cook has completed before the intervention');
    if (mode === 'after-first') {
      await turn(f, () => drop(f));
      assertDropped(f);
    } else if (mode === 'walk') {
      await turn(f, () => f.player.getMovementQueue().requestWalk(new core.Location(3220, 3218, 0), false));
      assert.equal(f.player.getMovementQueue().isMovings(), true);
      assert.equal(range.isWithinInteractionDistance(f.player.getLocation()), true);
    } else if (mode === 'replacement') {
      await turn(f, () => admit(f, chicken, 2));
    } else if (mode === 'missing-food') {
      // A valid Drop of the sole remaining raw beef removes the input, independently of the pot case.
      const slot = f.inventory.getSlotForItemId(raw);
      await turn(f, () => f.connection.dispatch([{ type: 'inventory_action', widgetId: core.Inventory.INTERFACE_ID,
        slot, itemId: raw, option: 'Drop', optionIndex: 5 }]));
      assert.equal(f.inventory.getAmount(raw), 0);
      assert.equal(core.World.getItems().filter(g => !g.isPendingRemoval()
        && g.getOwner() === f.player.getUsername() && g.getItem().getId() === raw).length, 1);
    } else if (mode === 'due-tick') {
      await ticks(f, 2);
      assert.deepEqual(f.state(), first());
      await turn(f, () => drop(f)); // task runs before this input
      assertDropped(f);
      assert.deepEqual(f.state(), second(), 'already due cook precedes drop input');
    }
    await ticks(f, 6);
    if (mode === 'no-drop' || mode === 'due-tick') assert.deepEqual(f.state(), second());
    if (mode === 'after-first' || mode === 'walk') assert.deepEqual(f.state(), first());
    if (mode === 'missing-food') assert.deepEqual(f.state(), { ...first(), raw: 0 });
    if (mode === 'replacement') {
      assert.equal(f.inventory.getAmount(raw), 1, 'replacement leaves the other raw beef');
      assert.equal(f.inventory.getAmount(cooked), 1);
      assert.equal(f.inventory.getAmount(chicken), 0);
      assert.equal(f.inventory.getAmount(cookedChicken), 2);
      assert.equal(f.state().xp, 90 * core.GameConstants.EXPERIENCE_MULTIPLIER);
    }
    assert.equal(Cooking.isCookingActive(f.player), false);
  });
}

test('dropping an item stops only its owner while another player keeps cooking', async t => {
  const dropping = actor(t), other = actor(t);
  admit(dropping);
  admit(other);
  // Both actors share a runner; each turn advances the same global tick once.
  await ticks(dropping, 2);
  assert.deepEqual(dropping.state(), first());
  assert.deepEqual(other.state(), first());
  await turn(dropping, () => drop(dropping));
  assertDropped(dropping);
  await ticks(other, 6);
  assert.deepEqual(dropping.state(), first());
  assert.deepEqual(other.state(), second());
  assert.equal(Cooking.isCookingActive(dropping.player), false);
  assert.equal(Cooking.isCookingActive(other.player), false);
});
