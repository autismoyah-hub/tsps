// From server/: node --test tests/qh728-q094.test.cjs (after yarn build).
// Focused Recipe for Disaster contracts: fresh start through the banquet opening,
// all eight rescue hand-ins, the Culinaromancer finale, and the scoped fixes for
// the Amik token drop, Wise Old Man ownership, goblin slop recovery, the Awowogei
// snake pit, and Traiborn's lost-ingredient recovery.
const assert = require('node:assert/strict');
const { test, before } = require('node:test');
const path = require('node:path');
const { Server } = require('../dist/Server');
Server.installProductionPathResolver();
const { PluginManager } = require('../dist/plugins/PluginManager');
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { NpcDefinitionLoader } = require('../dist/game/definition/loader/impl/NpcDefinitionLoader');
const { RegionManager } = require('../dist/game/collision/RegionManager');
const runtime = require('../plugins/quests/QuestRuntime');
const core = PluginManager.getCoreApi();
const noop = () => {};

// Register only the quests this audit reads instead of the whole Quests.plugin
// list, so an unrelated in-flight quest source cannot make this scoped test fail.
const TEST_QUEST_MODULES = [
  'RecipeForDisaster',
  'CooksAssistant', 'FishingContest', 'GoblinDiplomacy', 'BigChompyBirdHunting',
  'Biohazard', 'DemonSlayer', 'MurderMystery', 'NatureSpirit', 'WitchsHouse',
  'GertrudesCat', 'ShadowOfTheStorm', 'MonkeyMadnessI', 'FamilyCrest', 'HeroesQuest',
  'LostCity', 'MerlinsCrystal', 'ShieldOfArrav', 'DragonSlayer', 'ShiloVillage',
  'JunglePotion', 'DruidicRitual', 'UndergroundPass', 'PlagueCity', 'WaterfallQuest',
  'DesertTreasureI', 'HorrorFromTheDeep', 'LegendsQuest', 'OneSmallFavour',
];

before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  new NpcDefinitionLoader().load();
  RegionManager.init();
});

/** What a real sender's getVarbit answers: the last value sent, else 0. */
const varbitReader = (log) => (id) =>
  log.findLast(([method, varbitId]) => method === 'sendVarbit' && varbitId === id)?.[2] ?? 0;

function makePlayer() {
  const attrs = new Map();
  const items = new Map();
  const equipment = new Map();
  const messages = [];
  const moves = [];
  const xp = new Map();
  const log = [];
  let full = false;
  let location = new core.Location(3209, 3215, 0);
  let area = null;
  const inventory = {
    getAmount: id => items.get(id) ?? 0,
    contains: id => (items.get(id) ?? 0) > 0,
    adds: (id, amount = 1) => {
      if (!items.has(id) && inventory.getFreeSlots() <= 0) {
        messages.push('Not enough space in your inventory.');
        return false;
      }
      items.set(id, (items.get(id) ?? 0) + amount);
      return true;
    },
    deleteNumber: (id, amount = 1) => {
      items.set(id, Math.max(0, (items.get(id) ?? 0) - amount));
      return true;
    },
    getFreeSlots: () => (full ? 0 : 28 - [...items.values()].filter(amount => amount > 0).length),
    isFull: () => full || inventory.getFreeSlots() <= 0,
    getItems: () => [...items.entries()].filter(([, amount]) => amount > 0)
      .map(([id, amount]) => ({ getId: () => id, getAmount: () => amount, getDefinition: () => ({ getWeight: () => 0 }) })),
  };
  const sender = new Proxy({ getVarbit: varbitReader(log) }, { get: (target, prop) => target[prop] ?? ((...args) => { log.push([String(prop), ...args]); return sender; }) });
  const skillManager = {
    getCurrentLevel: skill => xp.get(`level:${skill}`) ?? 99,
    getMaxLevel: skill => xp.get(`level:${skill}`) ?? 99,
    getExperience: skill => xp.get(`xp:${skill}`) ?? 0,
    addExperience: (skill, amount) => xp.set(`xp:${skill}`, (xp.get(`xp:${skill}`) ?? 0) + amount),
    addExperiences: (skill, amount) => xp.set(`xp:${skill}`, (xp.get(`xp:${skill}`) ?? 0) + amount),
    setCurrentLevelCombat: (skill, value) => xp.set(`level:${skill}`, value),
  };
  return {
    attrs, items, equipment, messages, moves, log, xp,
    setFull(value) { full = value; },
    setLocation(x, y, z = 0) { location = new core.Location(x, y, z); },
    getAttribute: key => attrs.get(key),
    setAttribute: (key, value) => attrs.set(key, value),
    getInventory: () => inventory,
    getEquipment: () => ({ get: slot => {
      const id = equipment.get(slot);
      return id === undefined ? null : { getId: () => id };
    } }),
    getSkillManager: () => skillManager,
    getUsername: () => 'rfd-test', getDisplayName: () => 'rfd-test',
    getPrivateArea: () => null,
    getPacketSender: () => sender,
    getDialogueManager: () => ({ isActive: () => false, getCurrent: () => null, startDialogues: noop, handleResume: noop, reset: noop }),
    getLocation: () => location,
    getArea: () => area,
    setArea: next => { area = next; },
    getPrayerActive: () => [],
    getAsPlayer() { return this; },
    moveTo: next => { location = next; moves.push([next.getX(), next.getY(), next.getZ()]); },
    performAnimation: noop,
    sendMessage: text => messages.push(text),
    setRunEnergy: noop,
    setSpecialPercentage: noop,
    isRegistered: () => true,
    isPlayer: () => true,
  };
}

function harness() {
  const events = {}, variants = [], conditions = [], npcInteraction = [];
  const itemOnItem = [], itemOnNpc = [], itemOnObject = [];
  const objectInteraction = [], npcDeath = [], custom = {};
  const spawned = [], ground = [], transcripts = [], transcriptCalls = [];
  const coreApi = Object.create(core);
  Object.defineProperty(coreApi, 'Boundary', { value: class Boundary {
    constructor(...values) { this.values = values; }
  } });
  Object.defineProperty(coreApi, 'PrivateArea', { value: class PrivateArea {
    constructor(boundaries) { this.boundaries = boundaries; this.members = new Set(); this.destroyed = false; }
    enter(mobile) { this.members.add(mobile); mobile.setArea?.(this); }
    leave(mobile) { this.members.delete(mobile); mobile.setArea?.(null); if (!this.members.size) this.destroyed = true; }
    hasMember(mobile) { return this.members.has(mobile); }
    isDestroyed() { return this.destroyed; }
    getTemplateObjects() { return []; }
  } });
  Object.defineProperty(coreApi, 'ItemOnGroundManager', { value: class extends core.ItemOnGroundManager {
    static registerLocation(player, item, position) {
      const floorItem = super.registerLocation(player, item, position);
      ground.push([player, item, position, floorItem]);
      return floorItem;
    }
  } });
  Object.defineProperty(coreApi, 'PrayerHandler', { value: { deactivatePrayers: noop } });
  const api = new Proxy({
    core: coreApi,
    persistAttribute: noop,
    onNpcDialogueVariant: handler => variants.push(handler),
    onNpcDialogueCondition: handler => conditions.push(handler),
    onItemOnItem: handler => itemOnItem.push(handler),
    onItemOnNpc: handler => itemOnNpc.push(handler),
    onItemOnObject: handler => itemOnObject.push(handler),
    onObjectInteraction: (...args) => objectInteraction.push(args),
    onNpcInteraction: (...args) => npcInteraction.push(args),
    onNpcDeath: handler => npcDeath.push(handler),
    onPlayerLogin: noop,
    onPlayerLogout: noop,
    onCustomEvent: (key, handler) => (custom[key] ??= []).push(handler),
    emitCustomEvent: (key, event) => (custom[key] ??= []).forEach(handler => handler(event)),
    sendMultiChatboxPrompt: (player, title, ...pairs) => {
      player.prompt = { title, options: pairs.filter((_, index) => index % 2 === 0), pairs };
      return true;
    },
    spawnNpc: definition => {
      let area = null;
      let transform = definition.id;
      const npc = {
        definition,
        getId: () => transform,
        getLocation: () => new coreApi.Location(definition.x ?? 1900, definition.y ?? 5355, definition.z ?? 0),
        getArea: () => area,
        setArea: next => { area = next; },
        setNpcTransformationId: id => { transform = id; },
        getCombat: () => ({ getLastAttack: () => ({ reset: noop }) }),
        getSize: () => 1,
        isNpc: () => true,
        isRegistered: () => true,
      };
      spawned.push(npc);
      return npc;
    },
    getItemOnGroundManager: () => ({ register: (...args) => ground.push(args) }),
  }, { get: (target, key) => target[key] ?? noop });
  const originalStartTranscript = runtime.startTranscript;
  runtime.startTranscript = (...args) => {
    transcriptCalls.push(args);
    transcripts.push(args.slice(3, 5));
    return true;
  };
  for (const moduleName of TEST_QUEST_MODULES) {
    require(`../plugins/quests/quests/${moduleName}.Quest`)(api);
  }
  const quest = runtime.getRegisteredQuests().findLast(entry => entry.key === 'recipe_for_disaster');
  return { api, core: coreApi, quest, events, variants, conditions, npcInteraction,
    itemOnItem, itemOnNpc, itemOnObject, objectInteraction, npcDeath, custom, spawned, ground,
    transcripts, transcriptCalls, realStartTranscript: originalStartTranscript,
    restoreTranscripts: () => {
      runtime.startTranscript = originalStartTranscript;
      for (const [, , , floorItem] of ground) {
        const index = core.World.getItems().indexOf(floorItem);
        if (index !== -1) core.World.getItems().splice(index, 1);
      }
    } };
}

function completeQuest(h, player, key) {
  const entry = runtime.getRegisteredQuests().findLast(quest => quest.key === key);
  assert.ok(entry, `missing prerequisite quest ${key}`);
  entry.setStage(player, entry.completionValue);
}

function custom(h, name) {
  return h.custom[name] ?? [];
}

function findStepById(steps, id, found = []) {
  for (const step of steps ?? []) {
    if (!step || typeof step !== 'object') continue;
    if (step.id === id) found.push(step);
    findStepById(step.steps, id, found);
    for (const option of step.options ?? []) findStepById(option.steps, id, found);
  }
  return found;
}

function importedVariant(h, page, variant) {
  return runtime.loadTranscripts(h.api)[page].variants[variant];
}

function importedConditionText(h, page, variant, stepId) {
  const node = findStepById(importedVariant(h, page, variant), stepId)[0];
  assert.ok(node, `${page}/${variant} condition ${stepId}`);
  return node.text;
}

function assertLatestVarbit(player, id, expected, message = '') {
  const writes = player.log.filter(([method, varbitId]) => method === 'sendVarbit' && varbitId === id);
  assert.ok(writes.length, `varbit ${id} was sent`);
  assert.equal(writes.at(-1)[2], expected, message || `varbit ${id}`);
}

// Registers real quest sources into the real PluginManager NPC-interaction hooks,
// matching production registration order. Only NPC hooks are needed for the
// Rantz ownership regression.
function registerRealNpcHooks(names) {
  const beforeInteraction = PluginManager.npcInteractionHooks.length;
  const beforeVariant = PluginManager.npcDialogueVariantHooks.length;
  const api = new Proxy({ core }, { get(target, key) {
    if (key in target) return target[key];
    if (key === 'persistAttribute') return noop;
    if (key === 'onNpcDialogueVariant') return handler => PluginManager.npcDialogueVariantHooks.push({ pluginName: 'qh728-q094', handler });
    if (key === 'onNpcInteraction') return (...args) => {
      const [name, actions] = args;
      if (typeof name === 'function') {
        PluginManager.npcInteractionHooks.push({ pluginName: 'qh728-q094', handler: name });
        return;
      }
      for (const [action, handler] of Object.entries(actions ?? {})) {
        PluginManager.npcInteractionHooks.push({ pluginName: 'qh728-q094', handler: event => {
          if (event.handled || event.definition?.getName?.() !== name) return;
          if (event.definition.getActions?.()[event.clickType - 1] !== action) return;
          if (handler(event) !== false) event.handled = true;
        } });
      }
    };
    if (key === 'spawnNpc') return definition => ({ definition, getId: () => definition.id, isRegistered: () => true });
    return () => noop;
  } });
  for (const name of names) require(`../plugins/quests/quests/${name}.Quest`)(api);
  return { cleanup() {
    PluginManager.npcInteractionHooks.length = beforeInteraction;
    PluginManager.npcDialogueVariantHooks.length = beforeVariant;
  } };
}

function dialoguePlayer() {
  const { DialogueManager } = require('../dist/game/model/dialogues/DialogueManager');
  const p = makePlayer();
  p.manager = new DialogueManager(p);
  p.getDialogueManager = () => p.manager;
  const packet = new Proxy({
    getVarbit: varbitReader(p.log),
    sendInterfaceRemoval() { p.manager.reset(); p.prompt = null; return packet; },
  }, { get: (target, key) => target[key] ?? ((...args) => { p.log.push([String(key), ...args]); return packet; }) });
  p.getPacketSender = () => packet;
  return p;
}

function withHarnessConditions(h, fn) {
  const original = PluginManager.emitNpcDialogueCondition;
  PluginManager.emitNpcDialogueCondition = event => {
    for (const handler of h.conditions) {
      const answer = handler(event);
      if (typeof answer === 'boolean') return answer;
    }
    return null;
  };
  try {
    return fn();
  } finally {
    PluginManager.emitNpcDialogueCondition = original;
  }
}

function runRegisteredTranscript(h, player, npcId, handler, choice = 'Ask about helping the Lumbridge Guide.') {
  const choices = Array.isArray(choice) ? choice : [choice];
  const texts = [];
  runtime.startTranscript = (...args) => {
    h.transcriptCalls.push(args);
    return h.realStartTranscript(...args);
  };
  const event = { player, npcId, clickType: 1, definition: { getActions: () => ['Talk-to'] }, handled: false };
  assert.equal(handler(event), true);
  assert.equal(event.handled, true);
  withHarnessConditions(h, () => {
    for (let step = 0; step < 300 && (player.manager.getCurrent() || player.prompt); step++) {
      if (player.prompt) {
        const prompt = player.prompt;
        const wanted = choices.find(option => prompt.options.includes(option));
        assert.ok(wanted, `missing choice ${choices.join(' / ')} in ${JSON.stringify(prompt.options)} after ${JSON.stringify(texts)}`);
        const index = prompt.options.indexOf(wanted);
        player.prompt = null;
        prompt.pairs[index * 2 + 1]();
        continue;
      }
      const current = player.manager.getCurrent();
      const text = current?.getText?.();
      if (text) texts.push(text);
      player.manager.advance();
    }
  });
  return texts;
}

function runImportedVariant(h, player, npcId, page, variant, choices = []) {
  const handler = event => {
    assert.equal(runtime.startTranscript(h.api, event.player, npcId, page, variant), true,
      `imported transcript exists: ${page}/${variant}`);
    event.handled = true;
    return true;
  };
  return runRegisteredTranscript(h, player, npcId, handler, choices);
}


// ---- #5685 measurement (not for commit unless it becomes the regression test) ----
const MON = () => core.NpcIdentifiers;

function firstTalk(label, setup, choices) {
  const h = harness();
  try {
    const p = dialoguePlayer();
    h.quest.setStage(p, 4);
    p.equipment.set(core.Equipment.AMULET_SLOT, core.ItemIdentifiers.MSPEAK_AMULET);
    setup?.(p);
    const talk = h.npcInteraction.find(([name]) => name === 'Mizaru')?.[1]?.['Talk-to'];
    assert.ok(talk);
    const texts = runRegisteredTranscript(h, p, MON().MIZARU, talk, choices);
    const variant = h.transcriptCalls.at(-1)?.slice(3, 5);
    console.log(`MEASURE ${label}: variant=${JSON.stringify(variant)} first=${JSON.stringify(texts[0])} hasHello=${texts.includes("Who's there? Hello?")} n=${texts.length} varbit1914=${JSON.stringify(p.log.filter(([m, id]) => m === 'sendVarbit' && id === 1914).map(x => x[2]))} state=${p.attrs.get('recipe-for-disaster:awowogei-state')}`);
    return { texts, variant, p };
  } finally {
    h.restoreTranscripts();
  }
}

test('M1 first Talk-to with amulet after the king: which page plays', () => {
  const r = firstTalk('first-talk', p => p.attrs.set('recipe-for-disaster:awowogei-state', 8 | 64),
    ['What are you doing here?', "That's all for the moment."]);
  assert.equal(r.variant[1], 'ape-atoll-talking-to-the-three-wise-monkeys',
    'the first conversation page (What? eh? Did you say something?) plays on the first Talk-to');
  assert.equal(r.texts[0], 'What? eh? Did you say something?');
  assert.ok(r.texts.includes("Who's there? Hello?"));
});

test('M1b stage-10 varbit follows the displayed Who is there line', () => {
  const r = firstTalk('first-talk-varbit', p => p.attrs.set('recipe-for-disaster:awowogei-state', 8 | 64),
    ['What are you doing here?', "That's all for the moment."]);
  const writes = r.p.log.filter(([m, id]) => m === 'sendVarbit' && id === 1914).map(x => x[2]);
  assert.equal(writes.at(-1), 10);
});

test('M2 real cache objects around the Awowogei snake pit', () => {
  const { MapObjects } = require('../dist/game/entity/impl/object/MapObjects');
  const { ObjectDefinition } = require('../dist/game/definition/ObjectDefinition');
  const L = core.Location;
  const seen = new Map();
  for (const [x0, x1, y0, y1, tag] of [[3015, 3032, 5480, 5496, 'pit'], [2915, 2930, 2715, 2730, 'surface']]) {
    for (let x = x0; x <= x1; x++) for (let y = y0; y <= y1; y++) {
      for (const o of MapObjects.getObjectsAt(new L(x, y, 0), null)) {
        const def = ObjectDefinition.forId(o.getId());
        const name = def?.getName?.();
        const acts = def?.getActions?.();
        if (/rope|pit|snake|hole|cave|ladder/i.test(String(name)) || (acts ?? []).some(a => /climb|enter/i.test(String(a)))) {
          seen.set(`${tag}:${o.getId()}@${x},${y}`, `${name} type=${o.getType?.()} acts=${JSON.stringify(acts)}`);
        }
      }
    }
  }
  for (const [k, v] of seen) console.log('MEASURE obj', k, v);
  const rope = ObjectDefinition.forId(15571);
  console.log('MEASURE ROPE_21 def', rope?.getName?.(), JSON.stringify(rope?.getActions?.()));
  const pit = ObjectDefinition.forId(core.ObjectIdentifiers.PIT_3);
  console.log('MEASURE PIT_3', core.ObjectIdentifiers.PIT_3, pit?.getName?.(), JSON.stringify(pit?.getActions?.()));
});

test('M3 real plugin load: who claims Climb on the pit rope', () => {
  PluginManager.loadFromDirectory(path.resolve(__dirname, '../plugins'));
  const { MapObjects } = require('../dist/game/entity/impl/object/MapObjects');
  const { ObjectDefinition } = require('../dist/game/definition/ObjectDefinition');
  const L = core.Location;
  const object = MapObjects.get(15571, new L(3023, 5488, 0), null);
  console.log('MEASURE M3 object', !!object);
  const named = PluginManager.objectHooksByName.get('Rope') ?? [];
  console.log('MEASURE M3 Rope named hooks', JSON.stringify(named.map(h => ({ plugin: h.pluginName, order: h.order }))));
  const generic = PluginManager.objectInteractionHooks.filter(h => h.handler.name === 'handleAwowogeiSnakePit');
  console.log('MEASURE M3 RFD pit hook', JSON.stringify(generic.map(h => ({ plugin: h.pluginName, order: h.order }))));
  const p = makePlayer();
  p.attrs.set('recipe-for-disaster:awowogei-state', 8 | 64 | 1 | 2 | 4 | 16 | 32 | 128);
  runtime.getRegisteredQuests().findLast(e => e.key === 'recipe_for_disaster').setStage(p, 4);
  p.setLocation(3022, 5488, 0);
  p.getPrivateArea = () => null;
  const def = ObjectDefinition.forId(15571);
  const event = { player: p, object, definition: def, objectId: 15571, clickType: 1, option: 'climb',
    location: { x: 3023, y: 5488, z: 0 }, sourceLocation: { x: 3022, y: 5488, z: 0 }, handled: false };
  let err = null;
  try { PluginManager.emitObjectInteraction(event); } catch (e) { err = String(e).slice(0, 300); }
  console.log('MEASURE M3 result', JSON.stringify({ handled: event.handled, moves: p.moves, prompt: p.prompt?.title ?? null, messages: p.messages, err }));
});
