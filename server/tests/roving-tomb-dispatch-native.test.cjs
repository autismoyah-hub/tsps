// Run after cache setup and build: node --test tests/roving-tomb-dispatch-native.test.cjs
// Exercises the actual PluginManager registration wrappers in quest order. Seeded
// quest stages and inventory are a bounded server-dispatch fixture, not retail proof.
process.env.DISABLE_PLAYER_BOTS = '1';
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after, test } = require('node:test');
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager: PM } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');
const { Inventory } = require('../dist/game/model/container/impl/Inventory');
const { ClientConnection } = require('../dist/net/NetworkBuilder');
const { PlayerSave } = require('../dist/game/entity/impl/player/persistence/PlayerSave');
let core;
const errors = [], originalError = console.error, originalWarn = console.warn;
console.error = (...args) => { errors.push(['error',...args]); originalError(...args); };
console.warn = (...args) => { if (!String(args[0] ?? '').startsWith('[task] slow_task_tick ')) errors.push(['warn',...args]); originalWarn(...args); };
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins'));
  // This finite fixture must not start the administrative HTTP listener.
  const adminHooks = PM.serverStartupHooks.filter(h => h.pluginName === 'WorldAdmin');
  assert.equal(adminHooks.length, 1);
  PM.serverStartupHooks = PM.serverStartupHooks.filter(h => h.pluginName !== 'WorldAdmin');
  core = PM.getCoreApi();
  core.RegionManager.init(); core.RegionManager.loadMapFiles(2558, 3444);
  PM.emitServerStartup({});
});
after(() => { try { PM.emitServerShutdown({}); } finally { console.error = originalError; console.warn = originalWarn; assert.deepEqual(errors, []); } });
let serial = 1;
function actor(t, stage, restricted, worn = false) {
  const channel = { kind:'test', remoteAddress:'127.0.0.1', isOpen:()=>true, send:()=>true, close(){}, onData(){}, onError(){}, onClose(){} };
  const p = new Player(new PlayerSession(channel));
  const name = `tombroute${serial++}`;
  p.setUsername(name); p.setLongUsername(core.Misc.stringToLongBigInt(name));
  p.setAttribute('bot-skip-persistence', true);
  assert.equal(core.World.getPlayers().add(p), true);
  p.getSession().setPlayer(p);
  const connection = new ClientConnection(channel); connection.player = p;
  p.getSession().setTurnPacketProcessor(() => connection.processTurnPackets());
  p.fixtureConnection = connection;
  p.fixtureMessages = [];
  const sender = p.getPacketSender(), sendMessage = sender.sendMessage.bind(sender);
  sender.sendMessage = message => { p.fixtureMessages.push(message); return sendMessage(message); };
  p.moveTo(new core.Location(2558, 3443, 0)); p.setNeedsPlacement(false);
  p.setAttribute('quest.waterfall_quest.stage', 10);
  p.setAttribute('quest.roving_elves.stage', stage);
  p.getInventory().adds(core.ItemIdentifiers.GLARIALS_PEBBLE, 1);
  if (restricted && worn) p.getEquipment().setItem(core.Equipment.WEAPON_SLOT, new core.Item(restricted, 1));
  else if (restricted) p.getInventory().adds(restricted, 1);
  t.after(() => {
    p.closeInterruptibleInterfaces(); core.TaskManager.cancelTasks(p); core.TaskManager.cancelTasks(p.getIndex());
    if (p.isRegistered()) core.World.getPlayers().remove(p);
    assert.equal(p.isRegistered(), false);
  });
  return p;
}
async function usePebble(p) {
  const id = core.ItemIdentifiers.GLARIALS_PEBBLE;
  const objectId = core.ObjectIdentifiers.GLARIALS_TOMBSTONE;
  const object = core.World.findCacheObject(p, objectId, new core.Location(2558, 3444, 0));
  assert.ok(object, 'cache-placed tombstone');
  const itemSlot = p.getInventory().getItems().findIndex(item => item?.getId() === id);
  const item = p.getInventory().getItems()[itemSlot]; assert.ok(item);
  let calls = 0, handled = false, observedSlot, observedObject;
  const original = PM.emitItemOnObject;
  PM.emitItemOnObject = function(event) {
    const result = original.call(this, event);
    if (event.player === p && event.itemId === id && event.objectId === objectId) {
      calls++; handled = result; observedSlot = event.itemSlot; observedObject = event.object;
    }
    return result;
  };
  try {
    await p.fixtureConnection.dispatch([{type:'item_on_object', widgetId:Inventory.INTERFACE_ID,
      objectId, itemId:id, slot:itemSlot, x:2558, y:3444}]);
    for (let i=0; i<20 && !calls; i++) core.World.process();
    assert.equal(calls, 1, 'packet reached tombstone consumer'); assert.equal(handled, true);
    assert.equal(observedSlot, itemSlot); assert.equal(observedObject, object);
  } finally { PM.emitItemOnObject = original; }

}
function snapshot(p) {
  const l = p.getLocation();
  return { location:[l.getX(),l.getY(),l.getZ()], pebble:p.getInventory().getAmount(core.ItemIdentifiers.GLARIALS_PEBBLE),
    opened:Number(p.getAttribute('waterfall-quest:tombstone-open')) || 0,
    waterfall:Number(p.getAttribute('quest.waterfall_quest.stage')) };
}
async function advance(p) {
  for (let i=0; i<20; i++) {
    core.World.process();
    if (!p.getDialogueManager().isActive()) continue;
    const group = p.getPacketSender().getChatboxGroupId();
    const child = ({231:5, 217:5, 193:0, 229:2})[group];
    assert.notEqual(child, undefined, `known travel dialogue group ${group}`);
    await p.fixtureConnection.dispatch([{type:'dialogue_continue',widgetId:(group<<16)|child,childIndex:-1}]);
    p.fixtureConnection.processTurnPackets();
  }
  assert.equal(p.getDialogueManager().isActive(), false, 'travel dialogue completed');
  assert.equal(p.getMovementQueue().isMovementBlocked(), false, 'travel released movement');
}

test('active Roving refuses the same carried weapon, worn weapon and rune as canonical Waterfall', async t => {
  const I = core.ItemIdentifiers;
  for (const [item,worn] of [[I.BRONZE_SWORD,false],[I.BRONZE_SWORD,true],[I.AIR_RUNE,false]]) {
    const control = actor(t, 0, item, worn), active = actor(t, 2, item, worn);
    await usePebble(control); await usePebble(active); await advance(control); await advance(active);
    assert.deepEqual(snapshot(active), snapshot(control));
    assert.ok(active.fixtureMessages.includes("You place the pebble in the gravestone's small indent but nothing happens."));
    assert.deepEqual(snapshot(active), {location:[2558,3443,0],pebble:1,opened:0,waterfall:10});
  }
});
test('unrestricted Roving entry follows canonical Waterfall success state', async t => {
  const control = actor(t, 0), active = actor(t, 2);
  await usePebble(control); await usePebble(active); await advance(control); await advance(active);
  assert.deepEqual(snapshot(active), snapshot(control));
  assert.deepEqual(snapshot(active), {location:[2555,9844,0],pebble:0,opened:1,waterfall:10});
});

test('refusal retains pebble and same player can retry after removing weapon', async t => {
  const p = actor(t, 2, core.ItemIdentifiers.BRONZE_SWORD);
  await usePebble(p); await advance(p);
  assert.deepEqual(snapshot(p), {location:[2558,3443,0],pebble:1,opened:0,waterfall:10});
  p.getInventory().deleteNumber(core.ItemIdentifiers.BRONZE_SWORD, 1);
  await usePebble(p); await advance(p);
  assert.deepEqual(snapshot(p), {location:[2555,9844,0],pebble:0,opened:1,waterfall:10});
});
test('completed Roving delegates to canonical Waterfall handler', async t => {
  const control = actor(t, 0), complete = actor(t, 6);
  await usePebble(control); await usePebble(complete); await advance(control); await advance(complete);
  assert.deepEqual(snapshot(complete), snapshot(control));
});
test('existing open state survives save/login and rechecks restriction before seeded seed pickup', async t => {
  const p = actor(t, 2);
  await usePebble(p); await advance(p);
  assert.equal(snapshot(p).opened, 1);
  PM.emitPlayerLogout({player:p, username:p.getUsername()});
  const saved = PlayerSave.fromPlayer(p), restored = actor(t, 0);
  saved.applyToPlayer(restored);
  PM.emitPlayerLogin({player:restored, username:restored.getUsername(), isNewAccount:false});
  assert.equal(Number(restored.getAttribute('waterfall-quest:tombstone-open')), 1);
  restored.moveTo(new core.Location(2558,3443,0)); restored.setNeedsPlacement(false);
  restored.getInventory().adds(core.ItemIdentifiers.BRONZE_SWORD, 1);
  const objectId = core.ObjectIdentifiers.GLARIALS_TOMBSTONE;
  const object = core.World.findCacheObject(restored, objectId, new core.Location(2558,3444,0));
  assert.ok(object);
  const interact = () => PM.emitObjectInteraction({player:restored,object,objectId,
    definition:object.getDefinition(), option:'read', clickType:1,
    location:{x:2558,y:3444,z:0},handled:false});
  interact(); assert.deepEqual(snapshot(restored).location, [2558,3443,0]);
  restored.getInventory().deleteNumber(core.ItemIdentifiers.BRONZE_SWORD,1);
  interact(); assert.deepEqual(snapshot(restored).location, [2555,9844,0]);
  // Seed is fixture-placed: this covers the registered pickup consumer, not a guardian kill/drop.
  const seed = core.ItemIdentifiers.CONSECRATION_SEED;
  const ground = core.ItemOnGroundManager.registerLocation(restored, new core.Item(seed,1), restored.getLocation().clone(), null);
  assert.ok(ground);
  t.after(() => { const items=core.World.getItems(), i=items.indexOf(ground); if(i>=0) items.splice(i,1); });
  core.ItemOnGroundManager.pickup(restored, ground);
  assert.equal(restored.getInventory().getAmount(seed), 1);
  assert.equal(Number(restored.getAttribute('quest.roving_elves.stage')), 3);
});

test('pre-existing saved open state enforces restriction after login', t => {
  const p = actor(t, 2);
  p.setAttribute('waterfall-quest:tombstone-open', 1);
  PM.emitPlayerLogout({player:p, username:p.getUsername()});
  const restored = actor(t, 0); PlayerSave.fromPlayer(p).applyToPlayer(restored);
  PM.emitPlayerLogin({player:restored,username:restored.getUsername(),isNewAccount:false});
  restored.moveTo(new core.Location(2558,3443,0)); restored.setNeedsPlacement(false);
  restored.getInventory().adds(core.ItemIdentifiers.AIR_RUNE,1);
  const objectId = core.ObjectIdentifiers.GLARIALS_TOMBSTONE;
  const object = core.World.findCacheObject(restored, objectId, new core.Location(2558,3444,0));
  assert.ok(object);
  PM.emitObjectInteraction({player:restored,object,objectId,definition:object.getDefinition(),
    option:'read',clickType:1,location:{x:2558,y:3444,z:0},handled:false});
  assert.deepEqual(snapshot(restored).location,[2558,3443,0]);
  assert.ok(restored.fixtureMessages.includes('The tomb refuses you while you carry restricted items.'));
});
