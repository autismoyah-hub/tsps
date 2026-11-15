// Run after `yarn build`: node --test tests/dt2-vault-statues-smith.test.cjs
// #5607 "Desert treasure ii: Cannot proceed after inspecting statues and talking to smith". The real route through the real
// plugin: the vault door's "Start the quest?" prompt and Yes, the door's message and the walk into the Ancient Vault, the
// four statues and the plaque inspected where the plugin expects them, then Talk-to Asgarnia Smith. DT2-OBSERVED lines print
// the stage, the statue bits and what Smith said at each step.
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after, test } = require('node:test');
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager: PM } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');
const { MultiChatboxPrompt } = require('../dist/game/model/menu/MultiChatboxPrompt');
const runtime = require('../plugins/quests/QuestRuntime');


let core, next = 7100;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins')); core = PM.getCoreApi();
  PM.emitServerStartup({});
});
after(() => PM.emitServerShutdown({}));

const complete = (p, key) => { const q = runtime.getRegisteredQuests().find((x) => x.key === key); assert.ok(q, key); q.setStage(p, q.completionValue ?? 2); };
const dt2 = () => runtime.getRegisteredQuests().find((q) => /desert_treasure_ii/.test(q.key));

function actor(t) {
  const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => true, send: () => true, close() {}, onData() {}, onError() {}, onClose() {}, flush() {} };
  const p = new Player(new PlayerSession(channel)); p.setIndex(next++); p.setUsername('dt2vault' + next);
  p.setRegistered(true); p.setLocation(new core.Location(3511, 2970, 0)); p.needsPlacement = false;
  t.after(() => { p.setRegistered(false); core.TaskManager.cancelTasks(p.getIndex()); });
  p.messages = [];
  p.sendMessage = (text) => { p.messages.push(text); };
  p.setSpellbook(core.MagicSpellbook.ANCIENT);
  for (const [skill, level] of [[core.Skill.FIREMAKING, 75], [core.Skill.MAGIC, 75], [core.Skill.THIEVING, 70], [core.Skill.HERBLORE, 62], [core.Skill.RUNECRAFTING, 60], [core.Skill.CONSTRUCTION, 60]]) p.getSkillManager().setMaxLevels(skill, level, false);
  for (const key of ['desert_treasure_i', 'secrets_of_the_north', 'enakhras_lament', 'temple_of_the_eye', 'the_garden_of_death', 'below_ice_mountain', 'his_faithful_servants']) complete(p, key);
  return p;
}
const stageOf = (p) => dt2().getStage(p);
const statuesOf = (p) => Number(p.getAttribute('desert-treasure-ii:statues')) || 0;
const clickObject = (p, id, x, y) => {
  const area = p.getAttribute('desert-treasure-ii:ancient-vault');
  const object = new core.GameObject(id, new core.Location(area.getBaseX() + (x - 3904), area.getBaseY() + (y - 9600), 1), 10, 0, null);
  PM.emitObjectInteraction({ player: p, object, objectId: id, location: object.getLocation(), clickType: 1 });
};
const talkSmith = (p) => {
  const smith = p.getAttribute('desert-treasure-ii:vault-smith');
  const before = p.messages.length;
  const handled = PM.emitNpcInteraction({ player: p, npc: smith, npcId: smith.getContentId(p), definition: smith.getCurrentDefinition(p), action: 'Talk-to', clickType: 1, handled: false });
  return { handled, said: p.messages.slice(before), dialogue: !!p.getDialogueManager().getCurrent() };
};

test('start at the vault door, walk in, inspect the four statues and the plaque, talk to Smith: the quest proceeds', (t) => {
  const p = actor(t);
  const object = new core.GameObject(core.ObjectIdentifiers.VAULT_DOOR_2, new core.Location(3511, 2970, 0), 10, 0, null);
  PM.emitObjectInteraction({ player: p, object, objectId: object.getId(), location: object.getLocation(), clickType: 1 });
  MultiChatboxPrompt.handleInterfaceActionClick({ player: p, buttonId: MultiChatboxPrompt.OPTIONS_WIDGET_ID, groupId: 219, action: 1 });
  for (let i = 0; i < 4 && p.getDialogueManager().getCurrent(); i++) p.getDialogueManager().advance();
  console.log(`DT2-OBSERVED after start: stage=${stageOf(p)} vaultArea=${!!p.getAttribute('desert-treasure-ii:ancient-vault')} smith=${!!p.getAttribute('desert-treasure-ii:vault-smith')} messages=${JSON.stringify(p.messages.slice(-2))}`);
  assert.ok(stageOf(p) > 0, 'the quest started');
  assert.ok(p.getAttribute('desert-treasure-ii:ancient-vault') && p.getAttribute('desert-treasure-ii:vault-smith'), 'the vault and Smith exist');
  const started = stageOf(p);
  for (const [id, x, y] of [[49499, 3941, 9635], [49501, 3931, 9635], [49503, 3931, 9625], [49505, 3941, 9625]]) {
    clickObject(p, id, x, y); if (p.getDialogueManager().getCurrent()) p.getDialogueManager().advance();
    console.log(`DT2-OBSERVED statue ${id}: bits=${statuesOf(p)} stage=${stageOf(p)}`);
  }
  clickObject(p, core.ObjectIdentifiers.PLAQUE_35, 3944, 9631);
  console.log(`DT2-OBSERVED plaque: bits=${statuesOf(p)} stage=${stageOf(p)}`);
  assert.equal(statuesOf(p) & 31, 31, 'four statues and the plaque are all counted');
  const first = talkSmith(p);
  console.log(`DT2-OBSERVED smith talk 1: ${JSON.stringify(first)} stage=${stageOf(p)}`);
  while (p.getDialogueManager().getCurrent()) p.getDialogueManager().advance();
  const second = talkSmith(p);
  console.log(`DT2-OBSERVED smith talk 2: ${JSON.stringify(second)} stage=${stageOf(p)}`);
  while (p.getDialogueManager().getCurrent()) p.getDialogueManager().advance();
  console.log(`DT2-OBSERVED final: stage=${stageOf(p)} (started=${started})`);
  assert.ok(stageOf(p) > started, `Smith moved the quest on after the statues were inspected (stage ${started} -> ${stageOf(p)})`);
});
