// Run after `yarn build`: node --test tests/temple-of-ikov-lucien-spawns.test.cjs
// Wiki (Temple of Ikov, Flying Horse Inn): the quest starts with Lucien at the Flying Horse Inn in
// East Ardougne; his house, where he takes the staff or is fought, is between Edgeville and the GE.
const assert = require('node:assert/strict');
const { test, before } = require('node:test');
const path = require('node:path');
const { Server } = require('../dist/Server');
Server.installProductionPathResolver();
const { PluginManager } = require('../dist/plugins/PluginManager');
const { NpcDefinitionLoader } = require('../dist/game/definition/loader/impl/NpcDefinitionLoader');
const { getRegisteredQuests } = require('../plugins/quests/QuestRuntime');
const register = require('../plugins/quests/quests/TempleOfIkov.Quest');

const INN_LUCIEN = 13608;
const HOUSE_LUCIEN = 13607;
const FLYING_HORSE_INN = { x: 2573, y: 3321, z: 0 };
const NEAR_EDGEVILLE = { x: 3122, y: 3483, z: 0 };

let login;
let quest;
const spawns = [];

before(async () => {
  await require('../dist/game/cache/CachePipeline').CachePipeline.initialize(path.resolve(__dirname, '..'));
  new NpcDefinitionLoader().load();
  const api = new Proxy({
    core: PluginManager.getCoreApi(),
    onPlayerLogin: (fn) => { login = fn; },
    spawnNpc: (spec) => { spawns.push(spec); return { id: spec.id }; },
    removeNpc() {},
  }, { get: (t, p) => (p in t ? t[p] : () => {}) });
  register(api);
  quest = getRegisteredQuests().findLast((entry) => entry.key === 'temple_of_ikov');
});

function player() {
  const attributes = new Map(), varps = new Map(), varbits = new Map();
  const sender = new Proxy({
    sendConfig(id, value) { varps.set(id, value); return sender; },
    getVarp: (id) => varps.get(id) ?? 0,
    sendVarbit(id, value) { varbits.set(id, value); return sender; },
    getVarbit: (id) => varbits.get(id) ?? 0,
  }, { get: (t, p) => (p in t ? t[p] : () => sender) });
  return {
    getAttribute: (key) => attributes.get(key),
    setAttribute: (key, value) => attributes.set(key, value),
    getPacketSender: () => sender,
    getUsername: () => 'ikovtester',
    isRegistered: () => true,
    sendMessage() {},
  };
}

const at = (spec) => ({ x: spec.x, y: spec.y, z: spec.z });

test('before the quest, the Lucien who starts it is at the Flying Horse Inn', () => {
  spawns.length = 0;
  login({ player: player() });
  const inn = spawns.filter((spec) => spec.id === INN_LUCIEN);
  assert.equal(inn.length, 1);
  assert.deepEqual(at(inn[0]), FLYING_HORSE_INN);
  assert.equal(spawns.some((spec) => spec.id === HOUSE_LUCIEN), false, 'no house Lucien before the start');
});

test('once started, the attackable Lucien waits at his house near Edgeville', () => {
  spawns.length = 0;
  const p = player();
  quest.setStage(p, 10);
  login({ player: p });
  const house = spawns.filter((spec) => spec.id === HOUSE_LUCIEN);
  assert.equal(house.length, 1);
  assert.deepEqual(at(house[0]), NEAR_EDGEVILLE);
  assert.deepEqual(at(spawns.find((spec) => spec.id === INN_LUCIEN)), FLYING_HORSE_INN);
});

// The observed sequence list contains his entrance teleport as well as combat.
// This checks the loaded server role, not the rendered size of an animated model.
test('the Fire Warrior does not use his smoke-puff entrance as a defence animation', () => {
  const core = PluginManager.getCoreApi();
  const id = core.CacheDefinitions.getNpcId('ikov_firewarrior');
  assert.equal(id, 3448, 'the active cache resolves the quest actor');
  const entrance = core.CacheDefinitions.getSeqId('human_smokepuff_tele_appear');
  assert.equal(entrance, 803, 'the candidate is an entrance teleport, not a block');
  const warrior = core.NPC.create(id, new core.Location(2646, 9867, 0));
  assert.equal(warrior.getAttackAnim(), core.CacheDefinitions.getSeqId('human_caststrike'),
    'positive control: the observed spell remains his attack');
  assert.equal(warrior.getCurrentDefinition().getDeathAnim(), 836,
    'positive control: the observed death retains its role');
  assert.notEqual(warrior.getBlockAnim(), entrance, 'taking damage must not replay the entrance teleport');
  assert.equal(warrior.getBlockAnim(), core.CacheDefinitions.getSeqId('human_unarmedblock'),
    'an unclassified entrance must leave the human rig default intact');
});

test('the animation loader preserves explicitly configured creature combat roles', () => {
  const core = PluginManager.getCoreApi();
  const mole = core.NpcDefinition.forId(5779);
  assert.deepEqual([mole.getAttackAnim(), mole.getDefenceAnim(), mole.getDeathAnim()], [3312, 3311, 3310]);
});
