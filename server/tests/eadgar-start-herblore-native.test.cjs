// Seeded house and prerequisite states; real drink/house exit and generated dialogue, synthetic travel and NPC.
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after, test } = require('node:test');
process.env.DISABLE_PLAYER_BOTS = '1';
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager: PM } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');
const { ClientConnection } = require('../dist/net/NetworkBuilder');
const { ObjectDefinition } = require('../dist/game/definition/ObjectDefinition');
const { MultiChatboxPrompt } = require('../dist/game/model/menu/MultiChatboxPrompt');
let core, House, Instance, nextName = 0;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  core = PM.getCoreApi(); core.RegionManager.init(); ObjectDefinition.init();
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins')); PM.emitServerStartup({});
  House = require('../plugins/skills/construction/House.Construction');
  Instance = require('../plugins/skills/construction/Instance.Construction');
});
after(() => PM.emitServerShutdown({}));
function registered(key) {
  const result = require('../plugins/quests/QuestRuntime').getRegisteredQuests().find(q => q.key === key);
  assert.ok(result, `registered quest ${key}`); return result;
}
function actor(t, level, missing) {
  const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => true,
    send: () => true, close() {}, onData() {}, onError() {}, onClose() {} };
  const player = new Player(new PlayerSession(channel));
  const errors = [], warnings = [];
  const originalError = console.error.bind(console), originalWarn = console.warn.bind(console);
  t.mock.method(console, 'error', (...args) => { errors.push(args); return originalError(...args); });
  t.mock.method(console, 'warn', (...args) => {
    if (String(args[0]).includes('[plugins]')) warnings.push(args);
    return originalWarn(...args);
  });
  let ownedHouse;
  t.after(() => {
    const index = player.getIndex();
    try { PM.emitPlayerLogout({ player, username: player.getUsername() }); }
    finally {
      try { House.releaseHouse(player, true); } finally { core.World.getPlayers().remove(player); }
      core.TaskManager.cancelTasks(player); core.TaskManager.cancelTasks(index);
      player.closeInterruptibleInterfaces();
      assert.equal(House.houseFor(player), null);
      if (ownedHouse) assert.equal(ownedHouse.isDestroyed(), true);
      assert.equal(Instance.PlayerHouseInstance.isAllocationLocation(player.getLocation()), false);
      assert.equal(player.isRegistered(), false);
      assert.equal(core.TaskManager.hasActiveTask(player), false);
      assert.equal(core.TaskManager.hasActiveTask(index), false);
      assert.deepEqual(errors, []); assert.deepEqual(warnings, []);
    }
  });
  player.setUsername(`eadgargate${++nextName}`);
  player.setLongUsername(core.Misc.stringToLongBigInt(player.getUsername()));
  player.setAttribute('bot-skip-persistence', true);
  assert.equal(core.World.getPlayers().add(player), true);
  const skills = player.getSkillManager();
  skills.setMaxLevels(core.Skill.HERBLORE, level, false);
  skills.setExperiences(core.Skill.HERBLORE, core.SkillManager.getExperienceForLevel(level), false);
  skills.setCurrentLevel(core.Skill.HERBLORE, level, false);
  assert.equal(skills.getMaxLevel(core.Skill.HERBLORE), level);
  assert.equal(skills.getCurrentLevel(core.Skill.HERBLORE), level);
  assert.equal(skills.getExperience(core.Skill.HERBLORE), core.SkillManager.getExperienceForLevel(level));
  for (const key of ['druidic_ritual', 'troll_stronghold']) {
    const prerequisite = registered(key);
    if (key !== missing) prerequisite.setStage(player, prerequisite.completionValue);
    assert.equal(prerequisite.isComplete(player), key !== missing, key);
  }
  assert.equal(registered('one_small_favour').isStarted(player), false);
  assert.equal(registered('eadgars_ruse').getStage(player), 0);
  assert.equal(core.WorldDefinition.isMembersWorld(), true);
  assert.ok(player.getHitpoints() > 0); assert.notEqual(player.isDyingReturn(), true);
  const lines = [];
  const sender = player.getPacketSender(), sendString = sender.sendString.bind(sender);
  t.mock.method(sender, 'sendString', (...args) => {
    if (args[1] === ((231 << 16) | 6)) lines.push(String(args[0]));
    return sendString(...args);
  });
  const connection = new ClientConnection(channel); connection.player = player;
  return { player, connection, lines, setHouse: house => { ownedHouse = house; } };
}
function boostInHouse(f) {
  const { player } = f;
  assert.equal(registered('druidic_ritual').isComplete(player), true);
  assert.equal(registered('troll_stronghold').isComplete(player), true);
  assert.equal(registered('one_small_favour').isStarted(player), false);
  const save = Instance.createDefaultHouseSave();
  player.setAttribute('construction:house', save);
  assert.equal(House.enterHouse(player, false), true);
  const house = House.houseFor(player); f.setHouse(house);
  assert.ok(house instanceof Instance.PlayerHouseInstance);
  assert.equal(house.owner, player); assert.equal(player.getPrivateArea(), house);
  // The seeded house-only ale is consumed inside; acquisition is not exercised.
  const id = core.ItemIdentifiers.GREENMANS_ALE_3;
  assert.equal(id, 7746);
  assert.equal(core.CacheDefinitions.getItem(id).inventoryActions[0], 'Drink');
  player.getInventory().add(new core.Item(id, 1));
  const slot = player.getInventory().getSlotForItemId(id);
  assert.ok(slot >= 0);
  const item = player.getInventory().getItems()[slot];
  assert.equal(PM.emitItemAction({ player, item, itemId: id, slot,
    interfaceId: core.Inventory.INTERFACE_ID, clickType: 1, option: 'Drink', handled: false }), true);
  assert.equal(player.getInventory().getAmount(id), 0);
  assert.equal(player.getInventory().getAmount(core.ItemIdentifiers.BEER_GLASS_4), 1);
  assert.equal(player.getSkillManager().getMaxLevel(core.Skill.HERBLORE), 30);
  assert.equal(player.getSkillManager().getCurrentLevel(core.Skill.HERBLORE), 31);
  assert.equal(House.houseFor(player), house);
  House.releaseHouse(player, true);
  assert.equal(House.houseFor(player), null); assert.equal(house.isDestroyed(), true);
  assert.equal(player.getPrivateArea(), null);
  assert.equal(Instance.PlayerHouseInstance.isAllocationLocation(player.getLocation()), false);
  assert.equal(player.getInventory().getAmount(id), 0);
  assert.equal(player.getInventory().getAmount(core.ItemIdentifiers.BEER_GLASS_4), 0);
  assert.equal(player.getSkillManager().getCurrentLevel(core.Skill.HERBLORE), 31);
  assert.equal(player.getSkillManager().getExperience(core.Skill.HERBLORE), core.SkillManager.getExperienceForLevel(30));
}
async function start(f, yes) {
  const { player } = f; f.lines.length = 0;
  // Explicit synthetic relocation; no walking or real travel is asserted.
  player.moveTo(new core.Location(2905, 3437, 1)); player.setNeedsPlacement(false);
  const id = core.NpcIdentifiers.SANFEW;
  const npc = core.NPC.create(id, new core.Location(2905, 3437, 1));
  assert.equal(PM.emitNpcInteraction({ player, npc, npcId: id, action: 'Talk-to', clickType: 1, handled: false }), true);
  for (let step = 0; step < 100; step++) {
    const prompt = MultiChatboxPrompt.getPending(player);
    if (prompt) {
      let wanted;
      if (prompt.options.includes('Ask general questions.')) wanted = 'Ask general questions.';
      else if (prompt.options.includes('Have you any more work for me, to help reclaim the circle?')) wanted = 'Have you any more work for me, to help reclaim the circle?';
      else if (/Start the Eadgar.s Ruse quest\?/i.test(prompt.title ?? '')) wanted = yes ? 'Yes.' : 'No.';
      else throw Error(`unexpected Sanfew menu: ${JSON.stringify({ title: prompt.title, options: prompt.options })}`);
      const choice = prompt.options.indexOf(wanted); assert.ok(choice >= 0);
      await f.connection.dispatch([{ type: 'dialogue_continue', widgetId: MultiChatboxPrompt.OPTIONS_WIDGET_ID, childIndex: choice + 1 }]);
      f.connection.processTurnPackets();
      if (/Start the Eadgar.s Ruse quest\?/i.test(prompt.title ?? '')) return true;
      continue;
    }
    if (!player.getDialogueManager().getCurrent()) return false;
    const group = player.getPacketSender().getChatboxGroupId();
    const child = ({ 231: 5, 217: 5, 229: 4 })[group];
    assert.notEqual(child, undefined, `known dialogue group ${group}`);
    assert.equal(player.getPacketSender().isChatboxInterface(group), true);
    await f.connection.dispatch([{ type: 'dialogue_continue', widgetId: (group << 16) | child, childIndex: -1 }]);
    f.connection.processTurnPackets();
  }
  throw Error('Sanfew dialogue did not finish');
}
test('house ale boost allows base 30 Herblore to start', async t => {
  const f = actor(t, 30); boostInHouse(f); assert.equal(await start(f, true), true);
  assert.equal(registered('eadgars_ruse').getStage(f.player), 10);
});
test('unboosted Herblore 30 reaches the skill refusal', async t => {
  const f = actor(t, 30); assert.equal(await start(f, true), false);
  assert.match(f.lines.join(' ').replace(/<br\s*\/?\s*>/gi, ' ').replace(/\s+/g, ' '), /not yet learned enough/i);
  assert.equal(registered('eadgars_ruse').getStage(f.player), 0);
});
test('ordinary Herblore 31 can start', async t => {
  const f = actor(t, 31); assert.equal(await start(f, true), true);
  assert.equal(registered('eadgars_ruse').getStage(f.player), 10);
});
test('missing Troll Stronghold refuses the generated quest route', async t => {
  const f = actor(t, 31, 'troll_stronghold'); assert.equal(await start(f, true), false);
  assert.match(f.lines.join(' ').replace(/<br\s*\/?\s*>/gi, ' ').replace(/\s+/g, ' '), /Not just yet/i);
  assert.equal(registered('eadgars_ruse').getStage(f.player), 0);
});
test('incomplete Druidic Ritual retains its Sanfew dialogue ownership', async t => {
  const f = actor(t, 1, 'druidic_ritual');
  registered('druidic_ritual').setStage(f.player, 1);
  assert.equal(registered('druidic_ritual').isComplete(f.player), false);
  f.player.moveTo(new core.Location(2905, 3437, 1)); f.player.setNeedsPlacement(false);
  const id = core.NpcIdentifiers.SANFEW;
  const npc = core.NPC.create(id, new core.Location(2905, 3437, 1));
  assert.equal(PM.emitNpcInteraction({ player: f.player, npc, npcId: id, action: 'Talk-to', clickType: 1, handled: false }), true);
  for (let step = 0; step < 20; step++) {
    const prompt = MultiChatboxPrompt.getPending(f.player);
    if (prompt) {
      assert.ok(prompt.options.includes("I've been sent to help purify the Varrock stone circle."),
        `actual Druidic menu: ${JSON.stringify(prompt.options)}`);
      assert.equal(registered('eadgars_ruse').getStage(f.player), 0);
      return;
    }
    assert.ok(f.player.getDialogueManager().getCurrent(), 'Druidic dialogue reaches its menu');
    const group = f.player.getPacketSender().getChatboxGroupId();
    const child = ({ 231: 5, 217: 5, 229: 4 })[group];
    assert.notEqual(child, undefined, `known dialogue group ${group}`);
    await f.connection.dispatch([{ type: 'dialogue_continue', widgetId: (group << 16) | child, childIndex: -1 }]);
    f.connection.processTurnPackets();
  }
  throw Error('Druidic menu not reached');
});
test('No leaves the quest unstarted', async t => {
  const f = actor(t, 31); assert.equal(await start(f, false), true);
  assert.equal(registered('eadgars_ruse').getStage(f.player), 0);
});
test('injected hook retains base-eligible drained compatibility', t => {
  const f = actor(t, 31); f.player.getSkillManager().setCurrentLevel(core.Skill.HERBLORE, 30, false);
  assert.equal(f.player.getSkillManager().getMaxLevel(core.Skill.HERBLORE), 31);
  assert.equal(f.player.getSkillManager().getCurrentLevel(core.Skill.HERBLORE), 30);
  PM.emitCustomEvent('npc-dialogue:hook', { player: f.player, npcId: core.NpcIdentifiers.SANFEW,
    hook: 'quest:eadgar-s-ruse:start' });
  assert.equal(registered('eadgars_ruse').getStage(f.player), 10);
});
