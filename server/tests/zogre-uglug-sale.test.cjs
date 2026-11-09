// Run from server/ after the baseline build: node --test tests/zogre-uglug-sale.test.cjs
// Declared quest state and immediate movement arrival isolate the native packet, transcript,
// inventory and ShopManager paths. This does not prove physical navigation to Uglug.
process.env.DISABLE_PLAYER_BOTS = '1';
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after, test } = require('node:test');
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager: PM } = require('../dist/plugins/PluginManager');
const { ShopDefinitionLoader } = require('../dist/game/definition/loader/impl/ShopDefinitionLoader');
const { NpcDefinitionLoader } = require('../dist/game/definition/loader/impl/NpcDefinitionLoader');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');
const { PlayerSave } = require('../dist/game/entity/impl/player/persistence/PlayerSave');
const { UseItemPacketListener } = require('../dist/net/packet/impl/UseItemPacketListener');
const { World } = require('../dist/game/World');
const { MAX_STACK_AMOUNT } = require('../dist/game/model/container/StackType');
let core, next = 2200;
const SHOP = 1392;
const UGLUG = 861;
const STAGE_KEY = 'quest.zogre_flesh_eaters.stage';

before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  new NpcDefinitionLoader().load();
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins'));
  core = PM.getCoreApi();
  core.RegionManager.init();
  core.RegionManager.loadMapFiles(2442, 3049);
  PM.emitServerStartup({});
  assert.equal(new ShopDefinitionLoader().load(), true);
  core.ShopManager.initialize();
});
after(() => PM.emitServerShutdown({}));

function actor(t) {
  const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => true, send: () => true, close() {}, onData() {}, onError() {}, onClose() {} };
  const p = new Player(new PlayerSession(channel));
  p.setIndex(next++); p.setUsername(`uglug${next}`); p.setRegistered(true);
  p.setLocation(new core.Location(2442, 3049, 0)); p.needsPlacement = false;
  p.setAttribute(STAGE_KEY, 8);
  const messages = [];
  t.mock.method(p.getPacketSender(), 'sendMessage', text => { messages.push(text); return p.getPacketSender(); });
  t.mock.method(p.getMovementQueue(), 'walkToEntity', (_npc, arrive) => arrive());
  t.after(() => { core.ShopManager.close(p); PM.emitPlayerLogout({ player: p, username: p.getUsername() }); p.setRegistered(false); core.TaskManager.cancelTasks(p.getIndex()); });
  return { p, messages };
}
function target() {
  const def = core.NpcDefinition.forId(UGLUG);
  return { getDefinition: () => def, getCurrentDefinition: () => def, getContentId: () => UGLUG,
    getId: () => UGLUG, getLocation: () => new core.Location(2442, 3049, 0) };
}
function use(t, p, slot) {
  const npc = target(), npcs = World.getNpcs(), old = npcs.get;
  const item = p.getInventory().forSlot(slot);
  assert.ok(item && item.getId() > 0);
  npcs.get = () => npc;
  try { UseItemPacketListener.itemOnNpc(p, 3214, 1, item.getId(), slot); }
  finally { npcs.get = old; }
  assert.ok(p.getDialogueManager().getCurrent(), 'item use starts the real transcript');
}
function drain(p) {
  const dm = p.getDialogueManager();
  let limit = 80;
  while (dm.getCurrent()) { assert.ok(limit-- > 0, 'dialogue stalled'); dm.advance(); }
}
function open(p) {
  core.ShopManager.close(p);
  assert.equal(core.ShopManager.open(p, SHOP), true);
  return core.ShopManager.getOpenShop(p);
}
function restore(t, p) {
  PM.emitPlayerLogout({ player: p, username: p.getUsername() });
  const other = actor(t);
  PlayerSave.fromPlayer(p).applyToPlayer(other.p);
  PM.emitPlayerLogin({ player: other.p, username: other.p.getUsername(), isNewAccount: false });
  other.p.needsPlacement = false;
  return other;
}

test('the store refuses trade before the first balm sale', t => {
  const { p } = actor(t);
  assert.equal(open(p), null, 'starting the quest does not open the store');
});

for (const [name, price] of [['RELICYMS_BALM_1_', 100], ['RELICYMS_BALM_2_', 300], ['RELICYMS_BALM_3_', 650], ['RELICYMS_BALM_4_', 1000]]) {
  test(`first Uglug sale of ${name} pays ${price} and unlocks trade across relog`, t => {
    const { p, messages } = actor(t), inv = p.getInventory();
    const id = core.ItemIdentifiers[name];
    inv.setItem(0, new core.Item(id, 1));
    use(t, p, 0); drain(p);
    assert.equal(inv.getAmount(id), 0);
    assert.equal(inv.getAmount(core.ItemIdentifiers.COINS), price);
    assert.ok(messages.some(text => text.includes(`get ${price} coins in return`)), messages.join(' | '));
    assert.ok(open(p), 'trade opens after the first sale');
    const restored = restore(t, p);
    assert.ok(open(restored.p), 'the unlock survives a save and login');
  });
}

test('the offered slot is consumed, not a different held dose', t => {
  const { p } = actor(t), inv = p.getInventory();
  const one = core.ItemIdentifiers.RELICYMS_BALM_1_, four = core.ItemIdentifiers.RELICYMS_BALM_4_;
  inv.setItem(0, new core.Item(one, 1)); inv.setItem(1, new core.Item(four, 1));
  use(t, p, 0); drain(p);
  assert.equal(inv.getAmount(one), 0); assert.equal(inv.getAmount(four), 1);
  assert.equal(inv.getAmount(core.ItemIdentifiers.COINS), 100);
});

test('cancel, swap, remove, and relog before commit leave the sale recoverable', async t => {
  for (const change of ['cancel', 'swap', 'remove', 'relog']) await t.test(change, child => {
    let { p } = actor(child), inv = p.getInventory();
    const one = core.ItemIdentifiers.RELICYMS_BALM_1_, four = core.ItemIdentifiers.RELICYMS_BALM_4_;
    inv.setItem(0, new core.Item(one, 1)); inv.setItem(1, new core.Item(four, 1));
    use(child, p, 0);
    if (change === 'cancel') p.getDialogueManager().reset();
    if (change === 'swap') inv.swap(0, 1);
    if (change === 'remove') inv.deleteAtSlot(0, 1, false);
    if (change === 'relog') { p = restore(child, p).p; inv = p.getInventory(); }
    drain(p);
    assert.equal(inv.getAmount(core.ItemIdentifiers.COINS), 0);
    assert.equal(inv.getAmount(four), 1);
    assert.equal(open(p), null, 'no sale means no store unlock');
  });
});

test('a full coin stack does not consume the balm or unlock the shop', t => {
  const { p } = actor(t), inv = p.getInventory(), id = core.ItemIdentifiers.RELICYMS_BALM_1_;
  inv.setItem(0, new core.Item(id, 1)); inv.setItem(1, new core.Item(core.ItemIdentifiers.COINS, MAX_STACK_AMOUNT));
  use(t, p, 0); drain(p);
  assert.equal(inv.getAmount(id), 1); assert.equal(inv.getAmount(core.ItemIdentifiers.COINS), MAX_STACK_AMOUNT);
  assert.equal(open(p), null);
});

test('a later offered dose replaces the abandoned dialogue without double payment', t => {
  const { p } = actor(t), inv = p.getInventory();
  const one = core.ItemIdentifiers.RELICYMS_BALM_1_, four = core.ItemIdentifiers.RELICYMS_BALM_4_;
  inv.setItem(0, new core.Item(one, 1)); inv.setItem(1, new core.Item(four, 1));
  use(t, p, 0); use(t, p, 1); drain(p);
  assert.equal(inv.getAmount(one), 1); assert.equal(inv.getAmount(four), 0);
  assert.equal(inv.getAmount(core.ItemIdentifiers.COINS), 1000);
  assert.ok(open(p));
});

test('an unrelated started-quest shop retains its own refusal', t => {
  const { p } = actor(t);
  // Oziach's shop is gated by Dragon Slayer I, which this actor has not completed.
  const shop = core.ShopDefinition.all().find(entry => entry.getName() === 'Oziach (shop)');
  assert.ok(shop, 'canonical Oziach shop loaded');
  assert.equal(core.ShopManager.open(p, shop.getId()), true);
  assert.equal(core.ShopManager.getOpenShop(p), null);
});
