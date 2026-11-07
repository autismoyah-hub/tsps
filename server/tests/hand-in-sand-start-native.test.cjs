// Registered brew and generated dialogue with a synthetic Bert; not natural walking or a playthrough.
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
const { NpcDialogue } = require('../dist/game/model/dialogues/entries/impl/NpcDialogue');
const { PlayerDialogue } = require('../dist/game/model/dialogues/entries/impl/PlayerDialogue');
const { MultiChatboxPrompt } = require('../dist/game/model/menu/MultiChatboxPrompt');
let core, nextIndex = 4900;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  core = PM.getCoreApi();
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins'));
  PM.emitServerStartup({});
});
after(() => PM.emitServerShutdown({}));
function quest() {
  const found = require('../plugins/quests/QuestRuntime').getRegisteredQuests().find(q => q.key === 'hand_in_the_sand');
  assert.ok(found); return found;
}
function actor(t, thieving, crafting) {
  const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => true,
    send: () => true, close() {}, onData() {}, onError() {}, onClose() {} };
  const player = new Player(new PlayerSession(channel));
  t.after(() => {
    PM.emitPlayerLogout({ player, username: player.getUsername() });
    player.closeInterruptibleInterfaces(); player.setRegistered(false);
    core.TaskManager.cancelTasks(player); core.TaskManager.cancelTasks(player.getIndex());
    assert.equal(MultiChatboxPrompt.getPending(player), null);
    assert.ok(!player.getDialogueManager().getCurrent());
    assert.equal(player.isRegistered(), false);
    assert.equal(core.TaskManager.hasActiveTask(player), false);
    assert.equal(core.TaskManager.hasActiveTask(player.getIndex()), false);
  });
  player.setIndex(nextIndex++); player.setUsername(`sandgate${nextIndex}`);
  player.setAttribute('bot-skip-persistence', true); player.setRegistered(true);
  player.setLocation(new core.Location(2551, 3099, 0)); player.setNeedsPlacement(false);
  const skills = player.getSkillManager();
  for (const [skill, base] of [[core.Skill.THIEVING, thieving], [core.Skill.CRAFTING, crafting]]) {
    skills.setMaxLevels(skill, base, false);
    skills.setExperiences(skill, core.SkillManager.getExperienceForLevel(base), false);
    skills.setCurrentLevel(skill, base, false);
    assert.equal(skills.getMaxLevel(skill), base);
    assert.equal(skills.getCurrentLevel(skill), base);
    assert.equal(skills.getExperience(skill), core.SkillManager.getExperienceForLevel(base));
  }
  assert.ok(player.getHitpoints() > 0); assert.notEqual(player.isDyingReturn(), true);
  assert.equal(core.WorldDefinition.isMembersWorld(), true);
  assert.equal(quest().getStage(player), 0);
  for (const key of [core.TimerKey.STUN, core.TimerKey.POTION, core.TimerKey.FOOD])
    assert.equal(player.getTimers().has(key), false); // No world timer ticks are advanced.
  const messages = [], actionSteps = [];
  const emit = PM.emitCustomEvent.bind(PM);
  t.mock.method(PM, 'emitCustomEvent', (name, event) => {
    if (name === 'npc-dialogue:action' && event.player === player && event.kind === 'message') actionSteps.push({ npcId: event.npcId, stepId: event.stepId, kind: event.kind });
    return emit(name, event);
  });
  const sendMessage = player.sendMessage.bind(player);
  t.mock.method(player, 'sendMessage', (...args) => { messages.push(String(args[0])); return sendMessage(...args); });
  const connection = new ClientConnection(channel); connection.player = player;
  return { player, connection, messages, actionSteps, choices: [], expected: [[thieving, thieving], [crafting, crafting]] };
}
function consume(f, id, option) {
  const def = core.CacheDefinitions.getItem(id);
  const names = { [core.ItemIdentifiers.BANDITS_BREW]: "Bandit's brew", [core.ItemIdentifiers.MUSHROOM_PIE]: 'Mushroom pie' };
  assert.equal(def.name, names[id]);
  assert.ok(def.noteTemplate < 0 && def.placeholderTemplate < 0);
  assert.equal(def.inventoryActions[0], option);
  f.player.getInventory().add(new core.Item(id, 1));
  const slot = f.player.getInventory().getSlotForItemId(id);
  assert.ok(slot >= 0);
  const item = f.player.getInventory().getItems()[slot];
  assert.equal(item.getId(), id); assert.equal(item.getAmount(), 1); assert.equal(item.getMeta(), null);
  assert.equal(f.player.getInventory().getSlotForItemId(id), slot);
  assert.equal(PM.emitItemAction({ player: f.player, item, itemId: id, slot,
    interfaceId: core.Inventory.INTERFACE_ID, clickType: 1, option, handled: false }), true);
  assert.equal(f.player.getInventory().getAmount(id), 0);
}
async function bert(f, answer = 'Yes.') {
  const skills = f.player.getSkillManager();
  for (const [skill, [base, current]] of [[core.Skill.THIEVING, f.expected[0]], [core.Skill.CRAFTING, f.expected[1]]]) {
    assert.equal(skills.getMaxLevel(skill), base);
    assert.equal(skills.getCurrentLevel(skill), current);
    assert.equal(skills.getExperience(skill), core.SkillManager.getExperienceForLevel(base));
  }
  const normal = text => String(text).replace(/\s+/g, ' ').trim();
  const speech = [
    [NpcDialogue, "Eeee, wha' shall I do! I'll mos' certainly lose tha job..."],
    [PlayerDialogue, "Lose your job? What's wrong, why?"],
    [NpcDialogue, "I w-w-work... over yon sand pit... and weeell... I found... this... hand! T'were buried in't Sand!"],
    [PlayerDialogue, "Eww a hand, in the sand! Why haven't you told the authorities?"],
    [NpcDialogue, "They's no' wha' they once was. Tha cap'ain o'the Guard spends near all o'the time drunk in yon pub."],
    [PlayerDialogue, "Oh? The Guard Captain is drunk in the pub you say? That's not good, what will you do?"],
    [NpcDialogue, "Weeellll... do yer think yer could 'elp me?"],
    ...(answer === 'Yes.' ? [[PlayerDialogue, "Sure, I'll give you a hand."],
      [NpcDialogue, "..... Nae, ye can 'ave the 'and as h'evidence."],
      [NpcDialogue, "P'raps tha smell will get t'Guard Cap'ain's nose out o'his beer fer 2 seconds!"]]
      : [[PlayerDialogue, "I want no part in this!"]]),
  ];
  let speechIndex = 0;
  const id = core.NpcIdentifiers.BERT;
  const npc = core.NPC.create(id, new core.Location(2551, 3099, 0));
  const definition = npc.getCurrentDefinition(f.player);
  assert.equal(npc.getId(), id); assert.equal(definition.getName(), 'Bert');
  assert.ok(definition.getActions().includes('Talk-to'));
  assert.equal(npc.getLocation().getZ(), f.player.getLocation().getZ());
  assert.equal(npc.getPrivateArea(), f.player.getPrivateArea());
  assert.equal(PM.emitNpcInteraction({ player: f.player, npc, npcId: id,
    action: 'Talk-to', clickType: 1, handled: false }), true);
  for (let step = 0; step < 100; step++) {
    const prompt = MultiChatboxPrompt.getPending(f.player);
    if (prompt) {
      assert.equal(prompt.title, 'Select an option');
      assert.equal(f.player.getPacketSender().getChatboxGroupId(), 219);
      assert.equal(f.player.getDialogueManager().getOwner(), undefined);
      assert.equal(f.player.getPacketSender().getChatboxOwner(), undefined);
      const first = ["Oh, you found a hand in the sand - that's nice for you.", "Eww a hand, in the sand! Why haven't you told the authorities?"];
      assert.deepEqual(prompt.options, f.choices.length === 0 ? first : ['Yes.', 'No.']);
      if (prompt.options.includes('Yes.')) f.sawYes = true;
      const wanted = prompt.options.includes("Eww a hand, in the sand! Why haven't you told the authorities?")
        ? "Eww a hand, in the sand! Why haven't you told the authorities?" : answer;
      f.choices.push(wanted);
      const choice = prompt.options.indexOf(wanted);
      assert.ok(choice >= 0, `unexpected menu ${JSON.stringify(prompt.options)}`);
      await f.connection.dispatch([{ type: 'dialogue_continue', widgetId: MultiChatboxPrompt.OPTIONS_WIDGET_ID, childIndex: choice + 1 }]);
      f.connection.processTurnPackets();
      continue;
    }
    if (!f.player.getDialogueManager().getCurrent()) {
      assert.equal(MultiChatboxPrompt.getPending(f.player), null);
      assert.equal(f.player.getPacketSender().getChatboxGroupId(), -1);
      const refused = f.messages.includes('You do not meet all of the requirements to start The Hand in the Sand quest.');
      assert.equal(speechIndex, refused ? 7 : speech.length);
      return;
    }
    assert.ok(f.player.getDialogueManager().getCurrent(), 'Bert dialogue remains active');
    const current = f.player.getDialogueManager().getCurrent();
    const expected = speech[speechIndex++];
    assert.ok(expected && current instanceof expected[0], 'expected native speech type');
    assert.equal(normal(current.getText()), normal(expected[1]));
    assert.equal(f.player.getDialogueManager().getOwner(), undefined);
    assert.equal(f.player.getPacketSender().getChatboxOwner(), undefined);
    const group = f.player.getPacketSender().getChatboxGroupId();
    assert.equal(group, current instanceof NpcDialogue ? 231 : 217);
    const child = ({ 231: 5, 217: 5, 229: 4 })[group];
    assert.notEqual(child, undefined, `known dialogue group ${group}`);
    assert.equal(f.player.getPacketSender().isChatboxInterface(group), true);
    await f.connection.dispatch([{ type: 'dialogue_continue', widgetId: (group << 16) | child, childIndex: -1 }]);
    f.connection.processTurnPackets();
  }
  throw Error('Bert start did not reach a terminal');
}
test('ordinary base 17 Thieving and 49 Crafting reaches Bert hand message', async t => {
  const f = actor(t, 17, 49); await bert(f);
  assert.equal(quest().getStage(f.player), 10);
  assert.equal(f.player.getInventory().getAmount(core.ItemIdentifiers.SANDY_HAND), 1);
  assert.deepEqual(f.choices.slice(1), ['Yes.']);
  assert.ok(f.actionSteps.some(e => e.npcId === core.NpcIdentifiers.BERT && e.stepId === 'qRJX_b' && e.kind === 'message'), 'actual hand message fired');
});
for (const [label, thieving, crafting, id, option, skill, expected, remaining] of [
  ['Thieving', 16, 49, 'BANDITS_BREW', 'Drink', 'THIEVING', 17, 'BEER_GLASS'],
  ['Crafting', 17, 45, 'MUSHROOM_PIE', 'Eat', 'CRAFTING', 49, 'HALF_A_MUSHROOM_PIE'],
]) test(`boosted ${label} below base requirement is refused`, async t => {
  const f = actor(t, thieving, crafting), skills = f.player.getSkillManager();
  assert.equal(core.ItemIdentifiers.BANDITS_BREW, 4627);
  assert.equal(core.ItemIdentifiers.MUSHROOM_PIE, 21690);
  assert.equal(core.ItemIdentifiers.HALF_A_MUSHROOM_PIE, 21687);
  consume(f, core.ItemIdentifiers[id], option);
  f.expected[label === 'Thieving' ? 0 : 1][1] = expected;
  const base = label === 'Thieving' ? thieving : crafting;
  assert.equal(skills.getMaxLevel(core.Skill[skill]), base);
  assert.equal(skills.getCurrentLevel(core.Skill[skill]), expected);
  assert.equal(skills.getExperience(core.Skill[skill]), core.SkillManager.getExperienceForLevel(base));
  assert.equal(f.player.getInventory().getAmount(core.ItemIdentifiers[remaining]), 1);
  await bert(f);
  assert.equal(quest().getStage(f.player), 0);
  assert.equal(f.player.getInventory().getAmount(core.ItemIdentifiers.SANDY_HAND), 0);
  assert.ok(f.messages.includes('You do not meet all of the requirements to start The Hand in the Sand quest.'));
  assert.notEqual(f.sawYes, true, 'refusal leaves no Yes option');
  assert.ok(!f.actionSteps.some(e => e.npcId === core.NpcIdentifiers.BERT && e.stepId === 'qRJX_b' && e.kind === 'message'));
});

for (const [thieving, crafting] of [[16, 49], [17, 48]]) test(`unboosted ${thieving}/${crafting} is refused`, async t => {
  const f = actor(t, thieving, crafting); await bert(f);
  assert.equal(quest().getStage(f.player), 0);
  assert.equal(f.player.getInventory().getAmount(core.ItemIdentifiers.SANDY_HAND), 0);
  assert.ok(f.messages.includes('You do not meet all of the requirements to start The Hand in the Sand quest.'));
  assert.notEqual(f.sawYes, true, 'refusal leaves no Yes option');
  assert.ok(!f.actionSteps.some(e => e.npcId === core.NpcIdentifiers.BERT && e.stepId === 'qRJX_b' && e.kind === 'message'));
  assert.ok(!f.actionSteps.some(e => e.npcId === core.NpcIdentifiers.BERT && e.stepId === 'qRJX_b' && e.kind === 'message'));
});
test('eligible player choosing No does not start', async t => {
  const f = actor(t, 17, 49); await bert(f, 'No.');
  assert.equal(quest().getStage(f.player), 0);
  assert.equal(f.player.getInventory().getAmount(core.ItemIdentifiers.SANDY_HAND), 0);
  assert.ok(!f.actionSteps.some(e => e.npcId === core.NpcIdentifiers.BERT && e.stepId === 'qRJX_b' && e.kind === 'message'));
  assert.deepEqual(f.choices.slice(1), ['No.']);
  assert.ok(!f.messages.includes('You do not meet all of the requirements to start The Hand in the Sand quest.'));
});
test('eligible base with drained current Thieving remains refused', async t => {
  const f = actor(t, 17, 49), skills = f.player.getSkillManager();
  skills.setCurrentLevel(core.Skill.THIEVING, 16, false);
  f.expected[0][1] = 16;
  assert.equal(skills.getMaxLevel(core.Skill.THIEVING), 17);
  assert.equal(skills.getCurrentLevel(core.Skill.THIEVING), 16);
  await bert(f);
  assert.equal(quest().getStage(f.player), 0);
  assert.equal(f.player.getInventory().getAmount(core.ItemIdentifiers.SANDY_HAND), 0);
  assert.ok(f.messages.includes('You do not meet all of the requirements to start The Hand in the Sand quest.'));
  assert.notEqual(f.sawYes, true, 'refusal leaves no Yes option');
  assert.ok(!f.actionSteps.some(e => e.npcId === core.NpcIdentifiers.BERT && e.stepId === 'qRJX_b' && e.kind === 'message'));
});
