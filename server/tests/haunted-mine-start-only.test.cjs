const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after, test } = require('node:test');
process.env.DISABLE_PLAYER_BOTS = '1';
const { Server } = require('../dist/Server');
Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager: PM } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');
const { ClientConnection } = require('../dist/net/NetworkBuilder');
const { MultiChatboxPrompt } = require('../dist/game/model/menu/MultiChatboxPrompt');
let core, nextIndex = 2500;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  core = PM.getCoreApi();
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins'));
  PM.emitServerStartup({});
  core.RegionManager.init();
});
after(() => PM.emitServerShutdown({}));
function fixture(t) {
  const packets = [];
  const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => true,
    send: packet => { packets.push(packet); return true; }, close() {}, onData() {}, onError() {}, onClose() {} };
  const player = new Player(new PlayerSession(channel));
  player.setIndex(nextIndex++);
  player.setUsername('holidaydialogueqa');
  player.setAttribute('bot-skip-persistence', true);
  player.setRegistered(true);
  const connection = new ClientConnection(channel);
  connection.player = player;
  t.after(() => {
    let cleanupError;
    const clean = fn => { try { fn(); } catch (error) { cleanupError ??= error; } };
    clean(() => player.closeInterruptibleInterfaces());
    clean(() => core.World.unregisterPlayerPosition(player));
    clean(() => core.GameConstants.PLAYER_PERSISTENCE.cancelRequestedSave(player));
    clean(() => core.TaskManager.cancelTasks(player));
    clean(() => core.TaskManager.cancelTasks(player.getIndex()));
    clean(() => player.setRegistered(false));
    clean(() => {
      for (const bucket of core.World.playerTileOccupants.values()) {
        assert.ok(!bucket.includes(player), 'owned player is absent from world tile occupants');
      }
    });
    if (cleanupError) throw cleanupError;
  });
  return { player, packets, connection };
}
async function choose(f, childIndex) {
  await f.connection.dispatch([{ type: 'dialogue_continue', widgetId: MultiChatboxPrompt.OPTIONS_WIDGET_ID, childIndex }]);
  f.connection.processTurnPackets();
}
async function nextLine(f) {
  const group = f.player.getPacketSender().getChatboxGroupId();
  await f.connection.dispatch([{ type: 'dialogue_continue', widgetId: (group << 16) | 5, childIndex: -1 }]);
  f.connection.processTurnPackets();
}

const registeredQuest = key => require('../plugins/quests/QuestRuntime').getRegisteredQuests().find(quest => quest.key === key);
function completePrerequisites(player, keys) {
  for (const key of keys) { const quest = registeredQuest(key); assert.ok(quest, key); quest.setStage(player, quest.completionValue); }
}
async function followNativeStartRoute(f, definition) {
  const npcId = core.NpcIdentifiers[definition.npc], npc = core.NPC.create(npcId, new core.Location(3444, 3258, 0));
  assert.equal(PM.emitNpcInteraction({ player: f.player, npc, npcId, action: 'Talk-to', clickType: 1, handled: false }), true);
  for (const option of definition.route) {
    for (let n = 0; n < 80 && !MultiChatboxPrompt.getPending(f.player); n++) {
      assert.ok(f.player.getDialogueManager().getCurrent(), `source dialogue reaches ${option}`);
      await nextLine(f);
    }
    const menu = MultiChatboxPrompt.getPending(f.player), index = menu?.options.indexOf(option) ?? -1;
    assert.ok(index >= 0, `${option} in ${JSON.stringify(menu)}`);
    await choose(f, index + 1);
  }
}
const { SkillManager } = require('../dist/game/content/skill/SkillManager');
const route = { npc: 'ZEALOT', route: ['I follow the path of Saradomin.', 'I come seeking challenges and quests.', 'Yes.'] };
function captureErrors(t) {
  const errors = [], original = console.error;
  t.mock.method(console, 'error', (...args) => { errors.push(args); original(...args); });
  t.after(() => assert.deepEqual(errors, [], 'plugin and dialogue dispatch reported no errors'));
}
function seed(t, f, level, priest = true) {
    const skills = f.player.getSkillManager();
    for (const skill of core.Skill.values()) {
      skills.setExperiences(skill, SkillManager.getExperienceForLevel(60), false)
        .setMaxLevels(skill, 60, false).setCurrentLevel(skill, 60, false);
      assert.equal(skills.getExperience(skill), SkillManager.getExperienceForLevel(60));
    }
    skills.setExperiences(core.Skill.CRAFTING, SkillManager.getExperienceForLevel(level), false)
      .setMaxLevels(core.Skill.CRAFTING, level, false).setCurrentLevel(core.Skill.CRAFTING, level, false);
    assert.equal(skills.getExperience(core.Skill.CRAFTING), SkillManager.getExperienceForLevel(level));
    assert.equal(skills.getMaxLevel(core.Skill.CRAFTING), level);
    assert.equal(skills.getCurrentLevel(core.Skill.CRAFTING), level);
    if (priest) {
      completePrerequisites(f.player, ['priest_in_peril', 'the_restless_ghost']);
      registeredQuest('nature_spirit').setStage(f.player, 10);
    }
    f.player.setLocation(new core.Location(3444, 3257, 0));
    if (priest) {
      assert.equal(registeredQuest('priest_in_peril').isComplete(f.player), true);
      assert.equal(registeredQuest('the_restless_ghost').isComplete(f.player), true);
      assert.equal(registeredQuest('nature_spirit').getStage(f.player), 10);
      assert.equal(registeredQuest('nature_spirit').isComplete(f.player), false);
    }
}

test('Haunted Mine generated start accepts base Crafting 34 after the 35 control', async t => {
  captureErrors(t);
  const normal = fixture(t); seed(t, normal, 35);
  await followNativeStartRoute(normal, route);
  assert.equal(registeredQuest('haunted_mine').getStage(normal.player), 1);
  assert.ok(normal.player.getDialogueManager().getCurrent());

  const low = fixture(t); seed(t, low, 34);
  const pointsBefore = Number(low.player.getAttribute('quest.points')) || 0;
  const strengthBefore = low.player.getSkillManager().getExperience(core.Skill.STRENGTH);
  const varbits = [];
  const sender = low.player.getPacketSender();
  const sendVarbit = sender.sendVarbit.bind(sender);
  t.mock.method(sender, 'sendVarbit', (id, value) => { varbits.push([id, value]); return sendVarbit(id, value); });
  await followNativeStartRoute(low, route);
  assert.equal(registeredQuest('haunted_mine').getStage(low.player), 1);
  assert.ok(low.player.getDialogueManager().getCurrent());
  assert.equal(low.player.getAttribute('haunted-mine:schema'), 1);
  assert.ok(varbits.some(([id, value]) => id === 2397 && value === 1));
  assert.equal(Number(low.player.getAttribute('quest.points')) || 0, pointsBefore);
  assert.equal(low.player.getSkillManager().getExperience(core.Skill.STRENGTH), strengthBefore);

});

