const assert = require('node:assert/strict');
const path = require('node:path');
const { test } = require('node:test');
const { Server } = require('../dist/Server');
Server.installProductionPathResolver();
const { PluginManager } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');
const { PlayerSave } = require('../dist/game/entity/impl/player/persistence/PlayerSave');
const runtime = require('../plugins/quests/QuestRuntime');

test('temple sanctity resets on logout and stays zero through saved login', async t => {
  await require('../dist/game/cache/CachePipeline').CachePipeline.initialize(path.resolve(__dirname, '..'));
  const core = PluginManager.getCoreApi();
  const { World, TaskManager, ItemIdentifiers: I, ObjectIdentifiers: O } = core;
  // Register only this quest: loading the full plugin tree changes the fixture's scope.
  require('../plugins/quests/quests/ShadesOfMortton.Quest')(PluginManager.createApi('ShadesLogoutTest'));
  const quest = runtime.getRegisteredQuests().find(q => q.key === 'shades_of_mortton');
  assert.ok(quest);
  const actors = [];
  t.after(() => {
    for (const { player } of actors) {
      TaskManager.cancelTasks(player);
      if (player.isRegistered()) World.getPlayers().removes(player);
    }
  });
  function actor(name) {
    const channel = { kind:'test', remoteAddress:'127.0.0.1', isOpen:()=>true,
      send:()=>true, close(){}, onData(){}, onError(){}, onClose(){} };
    const player = new Player(new PlayerSession(channel));
    const configs = new Map();
    const sender = player.getPacketSender(), sendConfig = sender.sendConfig.bind(sender);
    sender.sendConfig = (id, value) => { configs.set(id, value); return sendConfig(id, value); };
    player.setUsername(name);
    player.setAttribute('bot-skip-persistence', true);
    player.setLocation(new core.Location(3506, 3315, 0));
    actors.push({ player, configs });
    assert.equal(World.getPlayers().add(player), true);
    player.needsPlacement = false;
    return { player, configs };
  }
  function login(a) {
    PluginManager.emitPlayerLogin({ player:a.player, username:a.player.getUsername(), isNewAccount:false });
  }
  function repair(a) {
    const inventory = a.player.getInventory();
    for (const [id, count] of [[I.LIMESTONE_BRICK, 1], [I.PLANK, 1], [I.SWAMP_PASTE, 5]]) {
      inventory.add(new core.Item(id, count), true);
    }
    const location = new core.Location(3506, 3316, 0);
    const wall = new core.GameObject(O.BROKEN_WALL, location, 10, 0, null);
    assert.equal(PluginManager.emitObjectInteraction({ player:a.player, object:wall, objectId:wall.getId(),
      clickType:1, location, handled:false }), true);
  }
  const a = actor('shadeslogouta'), b = actor('shadeslogoutb');
  quest.setStage(a.player, 85);
  quest.setStage(b.player, 55);
  a.player.getInventory().add(new core.Item(I.HAMMER, 1), true);
  b.player.getInventory().add(new core.Item(I.HAMMER, 1), true);
  login(a);
  login(b);
  assert.equal(a.configs.get(345), 20, 'unmarked legacy quest state retains its fallback');
  repair(a);
  repair(b);
  assert.ok(a.configs.get(345) > 20, 'the real wall interaction earned sanctity');
  const peerSanctity = b.configs.get(345);
  const sharedRepair = b.configs.get(343);
  assert.equal(World.getPlayers().removes(a.player), true, 'real removal invokes logout and cancels pending work');
  const saved = PlayerSave.fromPlayer(a.player);
  for (let i = 0; i < 5; i++) World.process();
  assert.equal(a.player.getAttribute('shades-of-mortton:temple-sanctity'), 0, 'pending repair cannot credit after logout');
  assert.ok(b.configs.get(345) >= peerSanctity, 'the other player keeps their sanctity');
  assert.equal(b.configs.get(343), sharedRepair, 'logout does not reset world repair');
  const restored = actor('shadeslogoutrestored');
  saved.applyToPlayer(restored.player);
  login(restored);
  assert.equal(restored.configs.get(345), 0, 'saved zero must not trigger the legacy fallback');
  repair(restored);
  assert.ok(restored.configs.get(345) > 0, 'a new repair earns sanctity after login');
});
