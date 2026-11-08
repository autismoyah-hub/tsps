// Native shop, currency and inventory path. Requires the built server and its cache.
process.env.DISABLE_PLAYER_BOTS = "1";
const assert = require("node:assert/strict");
const path = require("node:path");
const { before, after, test } = require("node:test");
require("../dist/Server").Server.installProductionPathResolver();
const { CachePipeline } = require("../dist/game/cache/CachePipeline");
const { PluginManager: PM } = require("../dist/plugins/PluginManager");
const { ShopDefinitionLoader } = require("../dist/game/definition/loader/impl/ShopDefinitionLoader");
const { Player } = require("../dist/game/entity/impl/player/Player");
const { PlayerSession } = require("../dist/net/PlayerSession");
const { ClientConnection } = require("../dist/net/NetworkBuilder");
const { MAX_STACK_AMOUNT } = require("../dist/game/model/container/StackType");
let core, index = 7800;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, ".."));
  PM.loadFromDirectory(path.resolve(__dirname, "../plugins"));
  core = PM.getCoreApi();
  core.RegionManager.init();
  core.RegionManager.loadMapFiles(3222, 3218);
  PM.emitServerStartup({});
  assert.equal(new ShopDefinitionLoader().load(), true);
  core.ShopManager.initialize();
});
after(() => PM.emitServerShutdown({}));
function actor(t) {
  const channel = { kind: "test", remoteAddress: "127.0.0.1", isOpen: () => true, send: () => true, close() {}, onData() {}, onError() {}, onClose() {} };
  const p = new Player(new PlayerSession(channel));
  p.setIndex(index++); p.setUsername(`minerals${index}`); p.setRegistered(true);
  p.setLocation(new core.Location(3222, 3218, 0)); p.needsPlacement = false;
  const connection = new ClientConnection(channel); connection.player = p;
  const messages = [];
  t.mock.method(p.getPacketSender(), "sendMessage", m => { messages.push(m); return p.getPacketSender(); });
  t.after(() => { core.ShopManager.close(p); PM.emitPlayerLogout({ player: p, username: p.getUsername() }); p.setRegistered(false); core.TaskManager.cancelTasks(p.getIndex()); });
  return { p, connection, messages };
}
async function action(f, groupId, childId, buttonNum, slot, itemId) {
  await f.connection.dispatch([{ type: "widget_action", widgetId: (groupId << 16) | childId, groupId, childId, buttonNum, slot, itemId }]);
  f.connection.processTurnPackets();
}
async function sell(f, id, button = 2) {
  const slot = f.p.getInventory().getSlotForItemId(id);
  assert.ok(slot >= 0);
  await action(f, 301, 0, button, slot, id);
}
function open(f, id) { assert.equal(core.ShopManager.open(f.p, id), true); }
function stock(f, id) { return core.ShopManager.getOpenShop(f.p).stock.find(entry => entry.itemId === id)?.amount; }

test("Belona's actual exchange charges and returns the Wiki mineral prices", async t => {
  const f = actor(t), inv = f.p.getInventory(), minerals = core.ItemIdentifiers.UNIDENTIFIED_MINERALS;
  const entries = [[21343, 40, 32], [21345, 100, 80], [24851, 10, 8], [24853, 20, 16]];
  inv.add(new core.Item(minerals, 1000), false);
  open(f, 1454);
  for (const [id, cost, refund] of entries) {
    const before = inv.getAmount(minerals), beforeStock = stock(f, id);
    const slot = core.ShopManager.getOpenShop(f.p).stock.findIndex(entry => entry.itemId === id);
    await action(f, 300, 16, 2, slot + 1, id);
    assert.equal(inv.getAmount(id), 1);
    assert.equal(inv.getAmount(minerals), before - cost);
    assert.equal(stock(f, id), beforeStock - 1);
    await sell(f, id, 1);
    assert.ok(f.messages.at(-1).includes(`shop will buy for ${refund} `));
    await sell(f, id);
    assert.equal(inv.getAmount(id), 0);
    assert.equal(inv.getAmount(minerals), before - cost + refund);
    assert.equal(stock(f, id), beforeStock);
  }
});

test("Belona refuses other stock and preserves locked items and a full currency stack", async t => {
  const f = actor(t), inv = f.p.getInventory(), minerals = core.ItemIdentifiers.UNIDENTIFIED_MINERALS;
  const glove = 21343, other = 12009;
  inv.add(new core.Item(minerals, MAX_STACK_AMOUNT), false);
  inv.add(new core.Item(other), false);
  inv.add(new core.Item(glove, 1, { [core.Item.UNTRADEABLE_META]: true }), false);
  open(f, 1454);
  await sell(f, other);
  await sell(f, glove);
  assert.equal(inv.getAmount(other), 1);
  assert.equal(inv.getAmount(glove), 1);
  assert.equal(inv.getAmount(minerals), MAX_STACK_AMOUNT);
  inv.deleteNumber(glove, 1);
  inv.add(new core.Item(glove), false);
  const before = stock(f, glove);
  await sell(f, glove);
  assert.equal(inv.getAmount(glove), 1);
  assert.equal(inv.getAmount(minerals), MAX_STACK_AMOUNT);
  assert.equal(stock(f, glove), before);
});

test("another reward shop still refuses Belona's stock", async t => {
  const f = actor(t), inv = f.p.getInventory(), glove = 21343;
  inv.add(new core.Item(glove), false);
  const shop = core.ShopDefinition.all().find(s => s.getName() === "Dusuri's Star Shop");
  assert.ok(shop);
  open(f, shop.getId());
  await sell(f, glove);
  assert.equal(inv.getAmount(glove), 1);
  assert.equal(inv.getAmount(core.ItemIdentifiers.STARDUST), 0);
});
