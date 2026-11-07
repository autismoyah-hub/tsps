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
const carriedItems = player => player.getInventory().getItems().filter(item => item.getId() >= 0).map(item => [item.getId(), item.getAmount(), item.getMeta()]);
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
async function assertRejectedStart(f, key, items) {
  const player = f.player;
  assert.equal(registeredQuest(key).getStage(player), 0);
  assert.equal(Number(player.getAttribute('quest.points')) || 0, 0);
  assert.deepEqual(carriedItems(player), items);
  assert.equal(player.getDialogueManager().getCurrent(), undefined, 'the accepted source tail is not played');
  assert.equal(player.getPacketSender().getChatboxGroupId(), -1, 'the existing refusal messages are uncovered');
  assert.equal(MultiChatboxPrompt.getPending(player), null);
  for (let n = 0; n < 3; n++) { await choose(f, 1); await nextLine(f); }
  assert.equal(registeredQuest(key).getStage(player), 0, 'stale native replies cannot reach later quest actions');
  assert.deepEqual(carriedItems(player), items);
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

test('Haunted Mine generated start refuses missing Priest in Peril', async t => {
  captureErrors(t);
  const missing = fixture(t); seed(t, missing, 35, false);
  const items = carriedItems(missing.player);
  await followNativeStartRoute(missing, route);
  await assertRejectedStart(missing, 'haunted_mine', items);
  missing.player.getSession().flushPackets();
  assert.match(Buffer.concat(missing.packets).toString('utf8'), /You must complete Priest in Peril/);

});

test('Haunted Mine mapped shard requires current Crafting 35', t => {
  captureErrors(t);
  const low = fixture(t); seed(t, low, 34);
  const pointsBefore = Number(low.player.getAttribute('quest.points')) || 0;
  const strengthBefore = low.player.getSkillManager().getExperience(core.Skill.STRENGTH);
  registeredQuest('haunted_mine').setStage(low.player, 10);
  low.player.getInventory().adds(core.ItemIdentifiers.CHISEL, 1);
  assert.equal(low.player.getInventory().contains(core.ItemIdentifiers.CHISEL), true);
  assert.ok(low.player.getInventory().getFreeSlots() > 0);
  assert.equal(low.player.getSkillManager().getCurrentLevel(core.Skill.CRAFTING), 34);
  // The shortcut destination is a source anchor for the crystal room; the
  // actual outcrop is discovered from the loaded map, not a guessed tile.
  core.RegionManager.loadMapFiles(2800, 4429);
  const ids = new Set([core.ObjectIdentifiers.CRYSTAL_OUTCROP,
    core.ObjectIdentifiers.CRYSTAL_OUTCROP_2, core.ObjectIdentifiers.CRYSTAL_OUTCROP_3]);
  const object = [...core.MapObjects.mapObjects.values()].flat().find(candidate => {
    const at = candidate.getLocation();
    return ids.has(candidate.getId()) && at.getZ() === 0 &&
      Math.abs(at.getX() - 2800) < 64 && Math.abs(at.getY() - 4429) < 64;
  });
  assert.ok(object, 'mapped crystal room outcrop exists');
  const location = object.getLocation();
  const definition = object.getDefinition();
  const interactions = definition.getInteractions();
  const optionIndex = interactions.findIndex(option => String(option).toLowerCase() === 'cut');
  assert.ok(optionIndex >= 0, `mapped outcrop ${object.getId()} at ${location.getX()},${location.getY()},${location.getZ()} has Cut action; actual ${JSON.stringify(interactions)}`);
  const queue = low.player.getMovementQueue();
  const spec = queue.buildObjectRouteSpec(object, 0);
  const adjacent = [[0, -1], [0, 1], [-1, 0], [1, 0]].find(([dx, dy]) => {
    low.player.setLocation(new core.Location(location.getX() + dx, location.getY() + dy, location.getZ()));
    return !core.RegionManager.blocked(low.player.getLocation(), low.player.getPrivateArea()) && queue.reachesObject(object, spec);
  });
  assert.ok(adjacent, 'an adjacent tile reaches the mapped outcrop');
  low.player.needsPlacement = false;
  const click = () => PM.emitObjectInteraction({ player: low.player, object, objectId: object.getId(),
    definition, location: { x: location.getX(), y: location.getY(), z: location.getZ() }, clickType: optionIndex + 1, handled: false });
  const packetStart = low.packets.length;
  assert.equal(click(), true);
  low.player.getSession().flushPackets();
  assert.match(Buffer.concat(low.packets.slice(packetStart)).toString('utf8'), /Crafting level of 35/);
  assert.equal(registeredQuest('haunted_mine').getStage(low.player), 10);
  assert.equal(low.player.getInventory().contains(core.ItemIdentifiers.SALVE_SHARD), false);
  assert.equal(Number(low.player.getAttribute('quest.points')) || 0, pointsBefore);
  assert.equal(low.player.getSkillManager().getExperience(core.Skill.STRENGTH), strengthBefore);
  assert.equal(low.player.getDialogueManager().getCurrent(), undefined);
  const gate = PM.emitCanGainExperience(low.player, core.Skill.STRENGTH, 22000);
  assert.equal(core.GameConstants.EXPERIENCE_MULTIPLIER, 1);
  assert.notEqual(gate.allow, false);
  assert.equal(gate.multiplier, 1);
  assert.notEqual(low.player.getAttribute(SkillManager.XP_LOCKED_ATTRIBUTE), true);
  low.player.getSkillManager().setCurrentLevel(core.Skill.CRAFTING, 35, false);
  assert.equal(click(), true);
  assert.equal(low.player.getSkillManager().getMaxLevel(core.Skill.CRAFTING), 34);
  assert.equal(low.player.getSkillManager().getExperience(core.Skill.CRAFTING), SkillManager.getExperienceForLevel(34));
  assert.equal(registeredQuest('haunted_mine').getStage(low.player), 11);
  assert.equal(low.player.getInventory().getAmount(core.ItemIdentifiers.SALVE_SHARD), 1);
  assert.equal(Number(low.player.getAttribute('quest.points')) || 0, pointsBefore + 2);
  assert.equal(low.player.getSkillManager().getExperience(core.Skill.STRENGTH), strengthBefore + 22000);
});
