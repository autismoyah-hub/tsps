// Seeded completion snapshot and synthetic Herman; no natural journey or full quest playthrough.
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

// Registered awards total 108 QP. Recruitment Drive and Fremennik Trials are excluded.
const COMPLETED = `the_ides_of_milk learning_the_ropes below_ice_mountain x_marks_the_spot
misthalin_mystery rune_mysteries pirates_treasure goblin_diplomacy witchs_potion dorics_quest
prince_ali_rescue imp_catcher ernest_the_chicken the_restless_ghost sheep_shearer shield_of_arrav
demon_slayer romeo_and_juliet cooks_assistant druidic_ritual fishing_contest monks_friend clock_tower
hazeel_cult sheep_herder plague_city jungle_potion dwarf_cannon murder_mystery gertrudes_cat
death_plateau elemental_workshop_i priest_in_peril rag_and_bone_man_i eagles_peak biohazard
tower_of_life client_of_kourend a_porcine_of_interest children_of_the_sun
the_ribbiting_tale_of_a_lily_pad_labour_dispute ethically_acquired_antiquities pandemonium
current_affairs a_ruff_situation creature_of_fenkenstrain fight_arena garden_of_tranquillity
ghosts_ahoy grand_tree haunted_mine holy_grail horror_from_the_deep merlins_crystal nature_spirit
observatory_quest one_small_favour shilo_village a_souls_bane troll_stronghold`.split(/\s+/);

let core, nextName = 0;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  core = PM.getCoreApi();
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins'));
  PM.emitServerStartup({});
});
after(() => PM.emitServerShutdown({}));

function quest(key) {
  const found = require('../plugins/quests/QuestRuntime').getRegisteredQuests().find(q => q.key === key);
  assert.ok(found, `registered quest ${key}`);
  return found;
}

function actor(t, firemaking, missing = null) {
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
  t.after(() => {
    const index = player.getIndex();
    try { PM.emitPlayerLogout({ player, username: player.getUsername() }); }
    finally {
      player.closeInterruptibleInterfaces();
      core.World.getPlayers().remove(player);
      core.TaskManager.cancelTasks(player); core.TaskManager.cancelTasks(index);
      assert.equal(player.isRegistered(), false);
      assert.equal(core.TaskManager.hasActiveTask(player), false);
      assert.equal(core.TaskManager.hasActiveTask(index), false);
      assert.deepEqual(errors, []); assert.deepEqual(warnings, []);
    }
  });
  player.setUsername(`swangate${nextName++}`);
  player.setLongUsername(core.Misc.stringToLongBigInt(player.getUsername()));
  assert.notEqual(player.getLongUsername(), 0n);
  player.setAttribute('bot-skip-persistence', true);
  assert.equal(core.World.getPlayers().add(player), true);
  player.setLocation(new core.Location(2330, 3690, 0)); player.setNeedsPlacement(false);
  const skills = player.getSkillManager();
  // A post-reward, trained saved snapshot: all other skills are 99 with matching XP.
  for (const skill of core.Skill.values()) {
    const level = skill === core.Skill.FIREMAKING ? firemaking : 99;
    skills.setMaxLevels(skill, level, false);
    skills.setExperiences(skill, core.SkillManager.getExperienceForLevel(level), false);
    skills.setCurrentLevel(skill, level, false);
    assert.equal(skills.getMaxLevel(skill), level);
    assert.equal(skills.getCurrentLevel(skill), level);
    assert.equal(skills.getExperience(skill), core.SkillManager.getExperienceForLevel(level));
  }
  assert.equal(new Set(COMPLETED).size, COMPLETED.length);
  let points = 0;
  for (const key of COMPLETED) {
    if (key === missing) continue;
    const completed = quest(key);
    completed.setStage(player, completed.completionValue);
    assert.equal(completed.isComplete(player), true, key);
    points += completed.questPoints;
  }
  // Horror from the Deep's zero-QP miniquest prerequisite is part of the snapshot.
  const barcrawl = quest('alfred-grimhands-barcrawl');
  barcrawl.setStage(player, barcrawl.completionValue);
  assert.equal(barcrawl.isComplete(player), true);
  assert.equal(barcrawl.questPoints || 0, 0);
  assert.equal(points, missing ? 106 : 108);
  // Reconciled saved-state snapshot, not execution of quest completion or reward producers.
  player.setAttribute('quest.points', points);
  assert.equal(player.getAttribute('quest.points'), points);
  for (const key of ['one_small_favour', 'garden_of_tranquillity']) {
    const query = { player, key, complete: false };
    PM.emitCustomEvent('quest:is-complete', query);
    assert.equal(query.complete, key !== missing, `canonical completion for ${key}`);
  }
  assert.equal(quest('swan_song').getStage(player), 0);
  assert.equal(core.WorldDefinition.isMembersWorld(), true);
  assert.equal(player.isRegistered(), true);
  assert.ok(player.getHitpoints() > 0);
  assert.notEqual(player.isDyingReturn(), true);
  assert.equal(player.getLocation().getX(), 2330);
  assert.equal(player.getLocation().getY(), 3690);
  const lines = [];
  const sender = player.getPacketSender();
  const originalSendString = sender.sendString.bind(sender);
  t.mock.method(sender, 'sendString', (text, id) => {
    lines.push(String(text));
    return originalSendString(text, id);
  });
  const connection = new ClientConnection(channel); connection.player = player;
  return { player, connection, lines };
}

async function advance(f) {
  assert.ok(f.player.getDialogueManager().getCurrent(), 'active generated dialogue');
  const group = f.player.getPacketSender().getChatboxGroupId();
  const child = ({ 231: 5, 217: 5, 229: 4 })[group];
  assert.notEqual(child, undefined, `known dialogue group ${group}`);
  assert.equal(f.player.getPacketSender().isChatboxInterface(group), true);
  await f.connection.dispatch([{ type: 'dialogue_continue', widgetId: (group << 16) | child, childIndex: -1 }]);
  f.connection.processTurnPackets();
}

async function talkToHerman(f) {
  const id = core.NpcIdentifiers.HERMAN_CARANOS;
  const npc = core.NPC.create(id, new core.Location(2330, 3690, 0));
  assert.equal(PM.emitNpcInteraction({ player: f.player, npc, npcId: id,
    action: 'Talk-to', clickType: 1, handled: false }), true);
  const choices = ["What's the rush?", 'Do you need any help?', "I'm a brave adventurer! Can I try?"];
  const normalized = text => text.replace(/<br\s*\/?\s*>/gi, ' ').replace(/<[^>]+>/g, '').replace(/\s+/g, ' ').trim();
  const refusal = "No, I'm sorry, but you simply do not have enough experience. I've got a LOT at stake here, and I can't afford to have an amateur getting in the way.";
  const accepted = "Yes, that's right. And do watch out for the sea trolls!";
  let selected = 0;
  for (let step = 0; step < 150; step++) {
    if (quest('swan_song').getStage(f.player) === 5 && f.lines.some(line => normalized(line) === accepted)) {
      assert.equal(selected, choices.length);
      return { outcome: 'started', lines: f.lines };
    }
    const prompt = MultiChatboxPrompt.getPending(f.player);
    if (prompt) {
      assert.ok(selected < choices.length, 'no unexpected Herman menu');
      const index = prompt.options.indexOf(choices[selected]);
      assert.ok(index >= 0, `missing ${choices[selected]}`);
      selected++;
      await f.connection.dispatch([{ type: 'dialogue_continue', widgetId: MultiChatboxPrompt.OPTIONS_WIDGET_ID, childIndex: index + 1 }]);
      f.connection.processTurnPackets();
    } else if (f.player.getDialogueManager().getCurrent()) {
      const current = f.player.getDialogueManager().getCurrent();
      if (current instanceof core.ActionDialogue && current.isSelfAdvancing()) {
        core.TaskManager.process();
      } else {
        assert.ok(current instanceof core.NpcDialogue || current instanceof core.PlayerDialogue,
          `unexpected dialogue entry ${current.constructor.name}`);
        await advance(f);
      }
    } else {
      assert.equal(selected, choices.length);
      assert.ok(f.lines.some(line => normalized(line) === refusal), `actual terminal refusal; seen ${JSON.stringify(f.lines)}`);
      return { outcome: 'refused', lines: f.lines };
    }
  }
  throw Error('Herman dialogue exceeded the step bound without reaching its outcome');
}

test('Firemaking 42 remains a positive control', async t => {
  const f = actor(t, 42); const result = await talkToHerman(f);
  assert.equal(quest('swan_song').getStage(f.player), 5, JSON.stringify(result));
  assert.equal(result.outcome, 'started');
});
test('Firemaking 40 can start with the reconciled quest-point snapshot', async t => {
  const f = actor(t, 40); const result = await talkToHerman(f);
  assert.equal(quest('swan_song').getStage(f.player), 5, JSON.stringify(result));
  assert.equal(result.outcome, 'started');
});
test('missing One Small Favour still prevents starting', async t => {
  const f = actor(t, 42, 'one_small_favour'); const result = await talkToHerman(f);
  assert.equal(quest('swan_song').getStage(f.player), 0);
  assert.equal(result.outcome, 'refused');
});
