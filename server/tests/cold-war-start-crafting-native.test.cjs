// Registered food and dialogue with a synthetic, unregistered Larry; not a natural quest walk.
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
const { MultiChatboxPrompt } = require('../dist/game/model/menu/MultiChatboxPrompt');
let core, nextIndex = 4800;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  core = PM.getCoreApi();
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins'));
  PM.emitServerStartup({});
});
after(() => PM.emitServerShutdown({}));
function quest() {
  const found = require('../plugins/quests/QuestRuntime').getRegisteredQuests().find(q => q.key === 'cold_war');
  assert.ok(found, 'Cold War is registered');
  return found;
}
function actor(t, crafting, lowSkill) {
  const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => true,
    send: () => true, close() {}, onData() {}, onError() {}, onClose() {} };
  const player = new Player(new PlayerSession(channel));
  player.setIndex(nextIndex++); player.setUsername(`coldgate${nextIndex}`);
  player.setAttribute('bot-skip-persistence', true); player.setRegistered(true);
  player.setLocation(new core.Location(2611, 3276, 0)); player.setNeedsPlacement(false);
  const skills = player.getSkillManager();
  for (const [skill, level] of [[core.Skill.HUNTER, 10], [core.Skill.AGILITY, 30],
    [core.Skill.CRAFTING, crafting], [core.Skill.CONSTRUCTION, 34], [core.Skill.THIEVING, 15]]) {
    const actual = skill === lowSkill ? level - 1 : level;
    skills.setMaxLevels(skill, actual, false);
    skills.setExperiences(skill, core.SkillManager.getExperienceForLevel(actual), false);
    skills.setCurrentLevel(skill, actual, false);
    assert.equal(skills.getMaxLevel(skill), actual);
    assert.equal(skills.getCurrentLevel(skill), actual);
  }
  assert.ok(player.getHitpoints() > 0); assert.notEqual(player.isDyingReturn(), true);
  assert.equal(core.WorldDefinition.isMembersWorld(), true);
  assert.equal(quest().getStage(player), 0);
  const lines = [];
  const sender = player.getPacketSender();
  const sendString = sender.sendString.bind(sender);
  t.mock.method(sender, 'sendString', (...args) => {
    if (args[1] === ((231 << 16) | 6)) lines.push(String(args[0]));
    return sendString(...args);
  });
  const trace = [];
  for (const group of ['beforeNpcInteractionHooks', 'npcInteractionHooks', 'npcAnyInteractionHooks']) {
    for (const hook of PM[group]) {
      const original = hook.handler;
      t.mock.method(hook, 'handler', function (event) {
        const matches = event?.player === player && [core.NpcIdentifiers.LARRY, core.NpcIdentifiers.LARRY_2].includes(event?.npcId);
        const beforeHandled = matches && event.handled === true;
        const handlerName = matches && typeof hook.handlerName === 'function' ? hook.handlerName(event) : undefined;
        const result = original.call(this, event);
        if (matches && (!beforeHandled && event.handled === true || /MakingFriendsWithMyArm|NpcDialogues/.test(`${hook.pluginName} ${hook.profileName}`))) {
          trace.push({ group, plugin: hook.pluginName, profile: hook.profileName, handlerName, action: event.action,
            npcId: event.npcId, name: event.definition?.getName?.(), beforeHandled,
            afterHandled: event.handled === true });
        }
        return result;
      });
    }
  }
  const connection = new ClientConnection(channel); connection.player = player;
  t.after(() => {
    PM.emitPlayerLogout({ player, username: player.getUsername() });
    player.closeInterruptibleInterfaces(); player.setRegistered(false);
    core.TaskManager.cancelTasks(player); core.TaskManager.cancelTasks(player.getIndex());
    assert.equal(player.isRegistered(), false);
    assert.equal(core.TaskManager.hasActiveTask(player), false);
    assert.equal(core.TaskManager.hasActiveTask(player.getIndex()), false);
  });
  return { player, connection, lines, trace };
}
function eatPie(f) {
  const id = core.ItemIdentifiers.MUSHROOM_PIE;
  f.player.getInventory().add(new core.Item(id, 1));
  const slot = f.player.getInventory().getSlotForItemId(id);
  assert.ok(slot >= 0, 'mushroom pie is held');
  const item = f.player.getInventory().getItems()[slot];
  assert.equal(PM.emitItemAction({ player: f.player, item, itemId: id, slot,
    interfaceId: core.Inventory.INTERFACE_ID, clickType: 1, option: 'Eat', handled: false }), true);
  assert.equal(f.player.getInventory().getAmount(id), 0);
  assert.equal(f.player.getInventory().getAmount(core.ItemIdentifiers.HALF_A_MUSHROOM_PIE), 1);
  assert.equal(f.player.getSkillManager().getMaxLevel(core.Skill.CRAFTING), 26);
  assert.equal(f.player.getSkillManager().getCurrentLevel(core.Skill.CRAFTING), 30);
  assert.equal(f.player.getSkillManager().getExperience(core.Skill.CRAFTING), core.SkillManager.getExperienceForLevel(26));
}
async function talk(f) {
  f.lines.length = 0;
  const id = core.NpcIdentifiers.LARRY;
  const npc = core.NPC.create(id, new core.Location(2611, 3276, 0));
  assert.equal(PM.emitNpcInteraction({ player: f.player, npc, npcId: id,
    action: 'Talk-to', clickType: 1, handled: false }), true);
  assert.ok(f.trace.some(row => /MakingFriendsWithMyArm/.test(row.profile ?? '') && row.handlerName === 'talkToLarry' && !row.afterHandled),
    `Making Friends declined Larry: ${JSON.stringify(f.trace)}`);
  const owner = f.trace.find(row => !row.beforeHandled && row.afterHandled);
  assert.match(owner?.plugin ?? '', /NpcDialogues/, `actual owner trace: ${JSON.stringify(f.trace)}`);
  for (let step = 0; step < 80; step++) {
    const prompt = MultiChatboxPrompt.getPending(f.player);
    if (prompt) {
      const choice = prompt.options.findIndex(text => text === 'Okay, why not!');
      assert.ok(choice >= 0, `unexpected menu: ${JSON.stringify(prompt.options)}`);
      await f.connection.dispatch([{ type: 'dialogue_continue', widgetId: MultiChatboxPrompt.OPTIONS_WIDGET_ID, childIndex: choice + 1 }]);
      f.connection.processTurnPackets();
      return true;
    }
    if (!f.player.getDialogueManager().getCurrent()) return false;
    const group = f.player.getPacketSender().getChatboxGroupId();
    const child = ({ 231: 5, 217: 5, 229: 4 })[group];
    assert.notEqual(child, undefined, `known dialogue group ${group}`);
    assert.equal(f.player.getPacketSender().isChatboxInterface(group), true);
    await f.connection.dispatch([{ type: 'dialogue_continue', widgetId: (group << 16) | child, childIndex: -1 }]);
    f.connection.processTurnPackets();
  }
  throw Error('Larry dialogue did not finish');
}
function refusal(f) {
  assert.match(f.lines.join(' ').replace(/<br\s*\/?\s*>/gi, ' ').replace(/\s+/g, ' '),
    /experienced adventurer and you're not it/i, `actual Larry lines: ${JSON.stringify(f.lines)}; hooks: ${JSON.stringify(f.trace)}`);
}
test('a real mushroom pie boost admits Crafting 26 to the Larry start', async t => {
  const f = actor(t, 26); eatPie(f);
  assert.equal(f.player.getSkillManager().getCurrentLevel(core.Skill.CRAFTING), 30);
  await talk(f);
  assert.equal(quest().getStage(f.player), 1, `Larry hooks: ${JSON.stringify(f.trace)}`);
});
test('unboosted Crafting 26 cannot start', async t => {
  const f = actor(t, 26); assert.equal(await talk(f), false);
  refusal(f);
  assert.equal(quest().getStage(f.player), 0);
});
test('base Crafting 30 remains eligible', async t => {
  const f = actor(t, 30); assert.equal(await talk(f), true, `Larry hooks: ${JSON.stringify(f.trace)}`);
  assert.equal(quest().getStage(f.player), 1);
});
test('base-eligible Crafting remains accepted when temporarily lowered', async t => {
  const f = actor(t, 30);
  f.player.getSkillManager().setCurrentLevel(core.Skill.CRAFTING, 29, false);
  assert.equal(f.player.getSkillManager().getMaxLevel(core.Skill.CRAFTING), 30);
  assert.equal(f.player.getSkillManager().getCurrentLevel(core.Skill.CRAFTING), 29);
  assert.equal(await talk(f), true);
  assert.equal(quest().getStage(f.player), 1);
});
for (const name of ['HUNTER', 'AGILITY', 'CONSTRUCTION', 'THIEVING']) {
  test(`${name} base requirement remains necessary with Crafting 30`, async t => {
    const f = actor(t, 30, core.Skill[name]);
    assert.equal(await talk(f), false); refusal(f);
    assert.equal(quest().getStage(f.player), 0);
  });
}
test('active Making Friends with My Arm retains Rellekka Larry ownership', t => {
  const f = actor(t, 30);
  const quests = require('../plugins/quests/QuestRuntime').getRegisteredQuests();
  // Saved-ready state, not a playthrough of the prerequisite quests.
  for (const key of ['my_arms_big_adventure', 'swan_song', 'cold_war', 'romeo_and_juliet']) {
    const prerequisite = quests.find(q => q.key === key);
    assert.ok(prerequisite, `registered prerequisite ${key}`);
    prerequisite.setStage(f.player, prerequisite.completionValue);
    assert.equal(prerequisite.isComplete(f.player), true, key);
  }
  const skills = f.player.getSkillManager();
  for (const [skill, level] of [[core.Skill.MINING, 72], [core.Skill.AGILITY, 68],
    [core.Skill.FIREMAKING, 66], [core.Skill.CONSTRUCTION, 35]]) {
    skills.setMaxLevels(skill, level, false);
    skills.setExperiences(skill, core.SkillManager.getExperienceForLevel(level), false);
    skills.setCurrentLevel(skill, level, false);
    assert.equal(skills.getMaxLevel(skill), level);
  }
  const makingFriends = quests.find(q => q.key === 'making_friends_with_my_arm');
  assert.ok(makingFriends, 'Making Friends with My Arm is registered');
  makingFriends.setStage(f.player, 10);
  assert.equal(makingFriends.getStage(f.player), 10);
  const id = core.NpcIdentifiers.LARRY_2;
  // Synthetic placement at the source-supported Rellekka pier, not a natural walk.
  const pier = new core.Location(2707, 3735, 0);
  f.player.setLocation(pier); f.player.setNeedsPlacement(false);
  const npc = core.NPC.create(id, pier);
  assert.equal(PM.emitNpcInteraction({ player: f.player, npc, npcId: id,
    action: 'Talk-to', clickType: 1, handled: false }), true);
  assert.ok(f.trace.some(row => /MakingFriendsWithMyArm/.test(row.profile ?? '') && row.handlerName === 'talkToLarry' && row.afterHandled),
    `actual owner trace: ${JSON.stringify(f.trace)}`);
  assert.match(f.trace.find(row => !row.beforeHandled && row.afterHandled)?.profile ?? '', /MakingFriendsWithMyArm/,
    `actual owner trace: ${JSON.stringify(f.trace)}`);
  assert.match(f.lines.join(' ').replace(/<br\s*\/?\s*>/gi, ' ').replace(/\s+/g, ' '),
    /I can't speak right now/i, `actual Larry lines: ${JSON.stringify(f.lines)}`);
  assert.equal(makingFriends.getStage(f.player), 10);
});
