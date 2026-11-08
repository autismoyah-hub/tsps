// Native shop, currency and inventory path. Requires the built server and its cache.
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
let core, serial = 0;
const oldBots = process.env.DISABLE_PLAYER_BOTS;
const oldError = console.error;
const hookErrors = [];
before(async () => {
  console.error = (...args) => { hookErrors.push(args.map(String).join(" ")); oldError.apply(console, args); };
  process.env.DISABLE_PLAYER_BOTS = "1";
  await CachePipeline.initialize(path.resolve(__dirname, ".."));
  PM.loadFromDirectory(path.resolve(__dirname, "../plugins"));
  core = PM.getCoreApi();
  core.RegionManager.init();
  core.RegionManager.loadMapFiles(3222, 3218);
  PM.emitServerStartup({});
  assert.equal(new ShopDefinitionLoader().load(), true);
  core.ShopManager.initialize();
});
after(() => {
  const failures = [];
  try {
    try { PM.emitServerShutdown({}); } catch (error) { failures.push(error); }
    try { assert.deepEqual(hookErrors, [], "no swallowed plugin or engine errors"); } catch (error) { failures.push(error); }
  } finally {
    console.error = oldError;
    if (oldBots === undefined) delete process.env.DISABLE_PLAYER_BOTS;
    else process.env.DISABLE_PLAYER_BOTS = oldBots;
  }
  if (failures.length) throw new AggregateError(failures, "shutdown failed");
});
function actor(t) {
  const channel = { kind: "test", remoteAddress: "127.0.0.1", isOpen: () => true, send: () => true, close() {}, onData() {}, onError() {}, onClose() {} };
  const session = new PlayerSession(channel);
  const p = new Player(session);
  let worldIndex = -1;
  // Install cleanup before registration, mocks or other setup can fail.
  t.after(() => {
    const failures = [];
    const attempt = fn => { try { fn(); } catch (error) { failures.push(error); } };
    attempt(() => core.ShopManager.close(p));
    attempt(() => p.closeInterruptibleInterfaces());
    attempt(() => PM.emitPlayerLogout({ player: p, username: p.getUsername() }));
    attempt(() => core.GameConstants.PLAYER_PERSISTENCE.cancelRequestedSave(p));
    attempt(() => p.getMovementQueue().reset());
    attempt(() => p.getMovementQueue().walkToReset());
    attempt(() => core.TaskManager.cancelTasks(p));
    attempt(() => { if (worldIndex >= 0) core.TaskManager.cancelTasks(worldIndex); });
    attempt(() => { if (worldIndex >= 0 && core.World.getPlayers().get(worldIndex) === p) core.World.getPlayers().remove(p); });
    attempt(() => { if (worldIndex >= 0) assert.notEqual(core.World.getPlayers().get(worldIndex), p, "actor removed"); });
    attempt(() => assert.equal(core.TaskManager.hasActiveTask(p), false, "player tasks cancelled"));
    attempt(() => { if (worldIndex >= 0) assert.equal(core.TaskManager.hasActiveTask(worldIndex), false, "index tasks cancelled"); });
    if (failures.length) throw new AggregateError(failures, "actor cleanup failed");
  });
  p.setAttribute("bot-skip-persistence", true);
  session.setPlayer(p);
  p.setUsername(`minerals${++serial}`);
  p.setLongUsername(core.Misc.stringToLongBigInt(p.getUsername()));
  assert.notEqual(p.getLongUsername(), 0n);
  assert.equal(core.World.getPlayers().add(p), true);
  worldIndex = p.getIndex();
  assert.equal(core.World.getPlayers().get(worldIndex), p);
  p.setLocation(new core.Location(3222, 3218, 0));
  p.setNeedsPlacement(false);
  assert.ok(p.getHitpoints() > 0);
  assert.notEqual(p.isDyingReturn(), true);
  assert.equal(p.busy(), false);
  const connection = new ClientConnection(channel); connection.player = p;
  const messages = [];
  t.mock.method(p.getPacketSender(), "sendMessage", m => { messages.push(m); return p.getPacketSender(); });
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
  const glove = core.ItemIdentifiers.MINING_GLOVES, other = core.ItemIdentifiers.BRONZE_AXE;
  inv.add(new core.Item(minerals, MAX_STACK_AMOUNT), false);
  const ordinary = new core.Item(other);
  assert.equal(ordinary.isSellable(), true, "non-stock control is normally sellable");
  inv.add(ordinary, false);
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
