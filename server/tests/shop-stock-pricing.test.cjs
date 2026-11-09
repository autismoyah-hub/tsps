// Native regression for ordinary stock-sensitive prices; the actor and starting items are seeded.
process.env.DISABLE_PLAYER_BOTS = '1';
const assert = require('node:assert/strict');
const path = require('node:path');
const {before, after, test} = require('node:test');
require('../dist/Server').Server.installProductionPathResolver();
const {CachePipeline} = require('../dist/game/cache/CachePipeline');
const {PluginManager: PM} = require('../dist/plugins/PluginManager');
const {ShopDefinitionLoader} = require('../dist/game/definition/loader/impl/ShopDefinitionLoader');
const {Player} = require('../dist/game/entity/impl/player/Player');
const {PlayerSession} = require('../dist/net/PlayerSession');
const {ClientConnection} = require('../dist/net/NetworkBuilder');
let core, next = 2300;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins'));
  core = PM.getCoreApi();
  core.RegionManager.init(); core.RegionManager.loadMapFiles(3230, 3203);
  PM.emitServerStartup({});
  assert.equal(new ShopDefinitionLoader().load(), true);
  core.ShopManager.initialize();
});
after(() => PM.emitServerShutdown({}));

async function openBob(t, coins, axes) {
  core.ShopManager.reload();
  const channel = {kind:'test', remoteAddress:'127.0.0.1', isOpen:()=>true, send:()=>true, close(){}, onData(){}, onError(){}, onClose(){}};
  const player = new Player(new PlayerSession(channel));
  let npc = null, npcAdded = false, loggedIn = false;
  t.after(() => {
    core.ShopManager.close(player);
    if (loggedIn) { PM.emitPlayerDisconnect({player, username:player.getUsername()}); PM.emitPlayerLogout({player, username:player.getUsername()}); }
    core.TaskManager.cancelTasks(player); core.TaskManager.cancelTasks(player.getIndex());
    core.World.unregisterPlayerPosition(player); player.setRegistered(false);
    if (npcAdded) core.World.getNpcs().remove(npc);
  });
  player.setIndex(next++); player.setUsername(`shopprice${next}`); player.setRegistered(true);
  player.setLocation(new core.Location(3230, 3203, 0)); player.needsPlacement = false;
  const connection = new ClientConnection(channel); connection.player = player;
  const messages = [];
  t.mock.method(player.getPacketSender(), 'sendMessage', message => { messages.push(message); return player.getPacketSender(); });
  loggedIn = true; PM.emitPlayerLogin({player, username:player.getUsername()}); player.closeInterruptibleInterfaces();
  const inv = player.getInventory(), axe = core.ItemIdentifiers.STEEL_AXE, money = core.ItemIdentifiers.COINS;
  assert.equal(axe, 1353); assert.equal(core.ItemDefinition.forId(axe).getValue(), 200);
  inv.add(new core.Item(money, coins), false); inv.add(new core.Item(axe, axes), false);
  npc = core.NPC.create(10619, new core.Location(3231, 3203, 0));
  npcAdded = core.World.getNpcs().add(npc); assert.equal(npcAdded, true);
  const definition = npc.getCurrentDefinition(player);
  const trade = definition.getActions().indexOf('Trade') + 1;
  assert.equal(trade, 3);
  let turns = 0;
  function turn() {
    assert.ok(++turns <= 20); player.resetUpdating(); core.TaskManager.process(); connection.processTurnPackets();
    const q = player.getMovementQueue(); q.beginCycle(); q.processEntityPursuit(); if (q.hasPendingWork()) q.process();
    core.TaskManager.processWalkTo(player.getIndex());
  }
  async function send(packet) { await connection.dispatch([packet]); turn(); }
  await send({type:'npc_option', index:npc.getIndex(), clickType:trade});
  for (let n = 0; n < 4 && !core.ShopManager.getOpenShop(player); n++) turn();
  const view = () => core.ShopManager.getOpenShop(player);
  assert.equal(view()?.name, "Bob's Brilliant Axes.");
  assert.equal(view().stock.find(row => row.itemId === axe)?.amount, 3);
  async function sell() {
    const slot = inv.getSlotForItemId(axe); assert.ok(slot >= 0);
    await send({type:'widget_action', widgetId:301<<16, groupId:301, childId:0, buttonNum:2, slot, itemId:axe});
  }
  async function buy() {
    const slot = view().stock.findIndex(row => row.itemId === axe); assert.ok(slot >= 0);
    await send({type:'widget_action', widgetId:(300<<16)|16, groupId:300, childId:16, buttonNum:2, slot:slot+1, itemId:axe});
  }
  const stock = () => view().stock.find(row => row.itemId === axe)?.amount;
  return {inv, axe, money, messages, stock, sell, buy};
}

// Wiki Shop revision 15329432: Bob buys a steel axe for 120 coins at stock 3,
// then 116 at stock 4, and sells it for 196 at stock 4.
test('Bob refuses an unaffordable purchase and pays the stock-sensitive sale price', async t => {
  const f = await openBob(t, 100, 2);
  f.messages.length = 0;
  await f.buy();
  assert.equal(f.inv.getAmount(f.money), 100); assert.equal(f.inv.getAmount(f.axe), 2); assert.equal(f.stock(), 3);
  assert.ok(f.messages.some(message => /enough coins/i.test(message)));
  await f.sell();
  assert.equal(f.inv.getAmount(f.money), 220); assert.equal(f.inv.getAmount(f.axe), 1); assert.equal(f.stock(), 4);
  await f.sell();
  assert.equal(f.inv.getAmount(f.money), 336); assert.equal(f.inv.getAmount(f.axe), 0); assert.equal(f.stock(), 5);
});

test('Bob charges the stock-sensitive price after a player sells an axe', async t => {
  const f = await openBob(t, 500, 1);
  await f.sell();
  assert.equal(f.inv.getAmount(f.axe), 0); assert.equal(f.stock(), 4);
  const before = f.inv.getAmount(f.money);
  await f.buy();
  assert.equal(before - f.inv.getAmount(f.money), 196);
  assert.equal(f.inv.getAmount(f.axe), 1); assert.equal(f.stock(), 3);
});

test('Bob sells a steel axe at its normal stock price', async t => {
  const f = await openBob(t, 500, 1);
  await f.buy();
  assert.equal(f.inv.getAmount(f.money), 300);
  assert.equal(f.inv.getAmount(f.axe), 2); assert.equal(f.stock(), 2);
});
