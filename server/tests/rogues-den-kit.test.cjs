// After yarn build: node --test tests/rogues-den-kit.test.cjs
// The Wiki describes a stackable kit consumed to make one item. Full-bag replacement is
// inferred from inventory conservation, not a captured OSRS interaction.
'use strict';
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, test } = require('node:test');
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');

const KIT = 5558, BONES = 526;
const choices = [['Spade', 952], ['Chisel', 1755], ['Rope', 954], ['Tinderbox', 590]];
let core, index = 8000;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  core = PluginManager.getCoreApi();
  require('../plugins/minigames/RoguesDen.plugin').register(PluginManager.createApi('RoguesDenKitRegression'));
  assert.ok(core.CacheDefinitions.getItem(KIT).inventoryActions.includes('Build'));
  assert.equal(core.ItemDefinition.forId(KIT).isStackable(), true);
});
function actor() {
  const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => true, send: () => true,
    close() {}, onData() {}, onError() {}, onClose() {} };
  const player = new Player(new PlayerSession(channel));
  player.setIndex(index++);
  player.setRegistered(true);
  return player;
}
function fullBag(player, count) {
  const inv = player.getInventory();
  inv.add(new core.Item(KIT, count), false);
  for (let i = 0; i < 27; i++) inv.add(new core.Item(BONES, 1), false);
  assert.equal(inv.getFreeSlots(), 0);
  assert.equal(inv.getAmount(KIT), count);
  assert.equal(inv.getAmount(BONES), 27);
}
function build(player) {
  const inv = player.getInventory();
  const slot = inv.getItems().findIndex(item => item?.getId() === KIT);
  const item = inv.getItems()[slot];
  const clickType = core.CacheDefinitions.getItem(KIT).inventoryActions.indexOf('Build') + 1;
  assert.equal(PluginManager.emitItemAction({ player, item, itemId: KIT, slot,
    interfaceId: core.Inventory.INTERFACE_ID, clickType, option: 'Build', handled: false }), true);
  assert.deepEqual(core.MultiChatboxPrompt.getPending(player).options, choices.map(([name]) => name));
}
function answer(player, n) {
  assert.equal(PluginManager.emitInterfaceActionClick({ player, buttonId: core.MultiChatboxPrompt.OPTIONS_WIDGET_ID,
    groupId: 219, slot: n, action: n }), true);
}
for (const [i, [name, id]] of choices.entries()) {
  test(`a full bag can replace its final kit with a ${name}`, () => {
    const player = actor(); fullBag(player, 1); build(player); answer(player, i + 1);
    const inv = player.getInventory();
    assert.equal(inv.getAmount(KIT), 0); assert.equal(inv.getAmount(id), 1);
    assert.equal(inv.getAmount(BONES), 27); assert.equal(inv.getFreeSlots(), 0);
  });
}
test('a full bag with a kit stack retains it when no output slot is available', () => {
  const player = actor(); fullBag(player, 2); build(player); answer(player, 1);
  const inv = player.getInventory();
  assert.equal(inv.getAmount(KIT), 2); assert.equal(inv.getAmount(952), 0);
  assert.equal(inv.getAmount(BONES), 27); assert.equal(inv.getFreeSlots(), 0);
});
test('a delayed choice cannot create an item after the kit is removed', () => {
  const player = actor(), other = actor(); fullBag(player, 1); fullBag(other, 1);
  build(player); player.getInventory().delete(KIT, 1); answer(player, 1);
  assert.equal(player.getInventory().getAmount(952), 0);
  assert.equal(player.getInventory().getAmount(BONES), 27);
  assert.equal(other.getInventory().getAmount(KIT), 1);
  assert.equal(other.getInventory().getAmount(952), 0);
});
