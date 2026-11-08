// Forestry kit pouch integration. Run after `yarn build`: node --test tests/forestry-pouches.test.cjs
// The pouch and kit are seeded as already owned; their acquisition is outside this fixture.
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after, beforeEach, afterEach, test } = require('node:test');
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');
const { EquipPacketListener } = require('../dist/net/packet/impl/EquipPacketListener');
const { PlayerSave } = require('../dist/game/entity/impl/player/persistence/PlayerSave');
const Storage = require('../plugins/items/storage/Common.StorageItems');

let core;
let next = 8100;
let hookErrors = [];
const originalError = console.error;
const originalWarn = console.warn;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  PluginManager.loadFromDirectory(path.resolve(__dirname, '../plugins'));
  core = PluginManager.getCoreApi();
  assert.equal(core.CacheDefinitions.getObjId('forestry_kit'), core.ItemIdentifiers.FORESTRY_KIT);
  assert.equal(core.CacheDefinitions.getObjId('skillcape_woodcutting'), core.ItemIdentifiers.WOODCUTTING_CAPE);
  assert.ok(Number.isInteger(core.CacheDefinitions.getObjId('skillcape_woodcutting_trimmed')));
});

beforeEach(() => {
  hookErrors = [];
  console.error = (...args) => { if (/\[plugins\].*(?:failed|error)/i.test(String(args[0]))) hookErrors.push(args.map(String).join(' ')); originalError(...args); };
  console.warn = (...args) => { if (/hook threw/i.test(String(args[0]))) hookErrors.push(args.map(String).join(' ')); originalWarn(...args); };
});
afterEach(() => {
  console.error = originalError;
  console.warn = originalWarn;
  assert.deepEqual(hookErrors, [], 'no dispatched plugin hook failed');
});
after(() => PluginManager.emitServerShutdown({}));

function actor(t, level) {
  const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => true, send: () => true,
    close() {}, onData() {}, onError() {}, onClose() {} };
  const player = new Player(new PlayerSession(channel));
  player.setIndex(next++);
  player.setUsername(`pouch${player.getIndex()}`);
  player.setRegistered(true);
  t.after(() => {
    try { PluginManager.emitPlayerLogout({ player, username: player.getUsername() }); }
    finally {
      player.closeInterruptibleInterfaces();
      core.TaskManager.cancelTasks(player);
      core.TaskManager.cancelTasks(player.getIndex());
      player.setRegistered(false);
    }
    assert.equal(player.isRegistered(), false);
    assert.equal(core.TaskManager.hasActiveTask(player), false);
    assert.equal(core.TaskManager.hasActiveTask(player.getIndex()), false);
  });
  player.setLocation(new core.Location(3234, 3222, 0));
  player.needsPlacement = false;
  player.getSkillManager().setMaxLevels(core.Skill.WOODCUTTING, level, false);
  player.getSkillManager().setCurrentLevel(core.Skill.WOODCUTTING, level, false);
  player.setHitpoints(10);
  assert.equal(player.getSkillManager().getMaxLevel(core.Skill.WOODCUTTING), level);
  assert.equal(player.getSkillManager().getCurrentLevel(core.Skill.WOODCUTTING), level);
  assert.ok(player.getHitpoints() > 0);
  assert.ok(player.getEquipment().getItems()[core.Equipment.CAPE_SLOT].getId() < 0);
  return player;
}
function give(player, ...ids) {
  for (const id of ids) player.getInventory().add(new core.Item(id, 1), false);
}
function item(player, id) {
  const slot = player.getInventory().getItems().findIndex(entry => entry?.getId() === id);
  assert.ok(slot >= 0, `item ${id} must be carried`);
  return { slot, item: player.getInventory().getItems()[slot] };
}
function use(player, usedId, targetId) {
  const used = item(player, usedId), target = item(player, targetId);
  return PluginManager.emitItemOnItem({ player, usedItem: used.item, usedItemId: usedId, usedItemSlot: used.slot,
    usedWithItem: target.item, usedWithItemId: targetId, usedWithItemSlot: target.slot, handled: false });
}
function option(player, id, option) {
  const held = item(player, id);
  const index = core.CacheDefinitions.getItem(id).inventoryActions.indexOf(option);
  assert.ok(index >= 0, `${option} must be a cache inventory action`);
  return PluginManager.emitItemAction({ player, interfaceId: core.Inventory.INTERFACE_ID, item: held.item,
    itemId: id, slot: held.slot, clickType: index + 1, option, handled: false });
}
function perk(player) {
  const request = { player, skill: 'WOODCUTTING', result: false };
  PluginManager.emitCustomEvent('skillcape:has-perk', request);
  return request.result;
}

test('cape pouch requires 99 Woodcutting and does not consume the pouch at 98', (t) => {
  const p = actor(t, 98), I = core.ItemIdentifiers;
  give(p, I.FORESTRY_KIT, I.CAPE_POUCH);
  const messages = [];
  t.mock.method(p.getPacketSender(), 'sendMessage', message => { messages.push(message); return p.getPacketSender(); });
  assert.equal(use(p, I.CAPE_POUCH, I.FORESTRY_KIT), true, 'the registered item-on-item handler consumed the interaction');
  assert.ok(messages.includes('You need level 99 Woodcutting to expand your Forestry kit/basket with this.'));
  assert.equal(p.getInventory().getAmount(I.CAPE_POUCH), 1);
  assert.equal(p.getInventory().getAmount(I.FORESTRY_KIT), 1);
});

test('a stored Woodcutting cape grants its perk with the kit worn, not carried', (t) => {
  const p = actor(t, 99), I = core.ItemIdentifiers;
  give(p, I.FORESTRY_KIT, I.CAPE_POUCH, I.WOODCUTTING_CAPE, I.LOGS);
  assert.equal(use(p, I.CAPE_POUCH, I.FORESTRY_KIT), true);
  assert.equal(p.getInventory().getAmount(I.CAPE_POUCH), 0);
  option(p, I.FORESTRY_KIT, 'Fill');
  assert.equal(p.getInventory().getAmount(I.WOODCUTTING_CAPE), 0);
  assert.equal(Storage.amountOf(p, Storage.FAMILIES.get('forestry-kit'), I.WOODCUTTING_CAPE), 1);
  assert.equal(p.getInventory().getAmount(I.LOGS), 1);
  assert.equal(perk(p), false, 'carrying the kit does not grant the cape perk');
  assert.ok(core.CacheDefinitions.getItem(I.FORESTRY_KIT).inventoryActions.includes('Wear'));
  EquipPacketListener.equip(p, I.FORESTRY_KIT, item(p, I.FORESTRY_KIT).slot, core.Inventory.INTERFACE_ID);
  assert.equal(p.getEquipment().getItems()[core.Equipment.CAPE_SLOT].getId(), I.FORESTRY_KIT);
  assert.equal(perk(p), true, 'wearing the kit grants the stored cape perk');
});


test('the kit refuses a cape before the upgrade and the upgraded store survives PlayerSave', (t) => {
  const p = actor(t, 99), I = core.ItemIdentifiers;
  give(p, I.FORESTRY_KIT, I.CAPE_POUCH, I.WOODCUTTING_CAPE);
  option(p, I.FORESTRY_KIT, 'Fill');
  assert.equal(p.getInventory().getAmount(I.WOODCUTTING_CAPE), 1);
  use(p, I.CAPE_POUCH, I.FORESTRY_KIT);
  option(p, I.FORESTRY_KIT, 'Fill');
  assert.equal(p.getInventory().getAmount(I.WOODCUTTING_CAPE), 0);
  const other = actor(t, 99);
  give(other, I.FORESTRY_KIT, I.WOODCUTTING_CAPE);
  option(other, I.FORESTRY_KIT, 'Fill');
  assert.equal(other.getInventory().getAmount(I.WOODCUTTING_CAPE), 1, 'another player does not inherit the upgrade or store');
  const restored = actor(t, 99);
  PlayerSave.fromPlayer(p).applyToPlayer(restored);
  assert.equal(restored.getInventory().getAmount(I.FORESTRY_KIT), 1);
  option(restored, I.FORESTRY_KIT, 'View');
  PluginManager.emitInterfaceActionClick({ player: restored, buttonId: (823 << 16) | 21,
    groupId: 823, childId: 21, action: 2, slot: 0, itemId: I.WOODCUTTING_CAPE, handled: false });
  assert.equal(restored.getInventory().getAmount(I.WOODCUTTING_CAPE), 1);
  assert.equal(Storage.amountOf(restored, Storage.FAMILIES.get('forestry-kit'), I.WOODCUTTING_CAPE), 0);
  EquipPacketListener.equip(restored, I.FORESTRY_KIT, item(restored, I.FORESTRY_KIT).slot, core.Inventory.INTERFACE_ID);
  assert.equal(restored.getEquipment().getItems()[core.Equipment.CAPE_SLOT].getId(), I.FORESTRY_KIT);
  assert.equal(perk(restored), false, 'wearing the now-empty kit does not grant the perk');
  assert.equal(restored.getInventory().getAmount(I.WOODCUTTING_CAPE), 1, 'withdrawal conserves the cape');
});
