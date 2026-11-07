// Seeded completion snapshot and registered firebox; no natural journey or full quest playthrough.
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after, test } = require('node:test');
process.env.DISABLE_PLAYER_BOTS = '1';
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager: PM } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');
const { MapObjects } = require('../dist/game/entity/impl/object/MapObjects');
const { ObjectDefinition } = require('../dist/game/definition/ObjectDefinition');
const { Inventory } = require('../dist/game/model/container/impl/Inventory');

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
  core.RegionManager.init();
  core.RegionManager.loadMapFiles(2335, 3674);
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
  return { player };
}


function firebox(f) {
  const loc = new core.Location(2335, 3674, 0);
  const object = MapObjects.getObjectsAt(loc, f.player.getPrivateArea()).find(o => o.getId() === core.ObjectIdentifiers.FIREBOX);
  assert.ok(object, 'registered Swan Song firebox at configured location');
  const definition = ObjectDefinition.forPlayer(object.getId(), f.player);
  assert.ok(definition);
  assert.equal(MapObjects.getPrivateArea(f.player, definition.id, loc), object);
  for (let dx = -2; dx <= 4; dx++) for (let dy = -2; dy <= 4; dy++) {
    const tile = new core.Location(loc.getX() + dx, loc.getY() + dy, loc.getZ());
    if (core.RegionManager.blocked(tile, f.player.getPrivateArea())) continue;
    f.player.moveTo(tile); f.player.setNeedsPlacement(false);
    if (core.PathFinder.reachedObject(f.player, loc.getX(), loc.getY(), definition.getSizeX(),
      definition.getSizeY(), object.getFace(), 10, definition.getBlockingMask())) return { object, definition, loc };
  }
  assert.fail('no reachable passable firebox tile');
}
function use(f, box, id) {
  const slots = f.player.getInventory().getItems();
  const slot = slots.findIndex(item => item && item.getId() === id);
  assert.ok(slot >= 0, `carried item ${id}`);
  const event = { player: f.player, object: box.object, objectId: box.object.getId(),
    definition: box.definition, item: slots[slot], itemId: id, itemSlot: slot,
    interfaceType: Inventory.INTERFACE_ID,
    location: { x: box.loc.getX(), y: box.loc.getY(), z: box.loc.getZ() }, handled: false };
  assert.equal(PM.emitItemOnObject(event), true);
  assert.equal(event.handled, true);
}
function setup(t, base, current = base) {
  const f = actor(t, base), s = f.player.getSkillManager();
  s.setCurrentLevel(core.Skill.FIREMAKING, current, false);
  assert.equal(s.getMaxLevel(core.Skill.FIREMAKING), base);
  assert.equal(s.getCurrentLevel(core.Skill.FIREMAKING), current);
  assert.equal(s.getExperience(core.Skill.FIREMAKING), core.SkillManager.getExperienceForLevel(base));
  quest('swan_song').setStage(f.player, 65);
  assert.equal(quest('swan_song').getStage(f.player), 65);
  const inv = f.player.getInventory();
  inv.add(new core.Item(core.ItemIdentifiers.LOGS, 1));
  inv.add(new core.Item(core.ItemIdentifiers.TINDERBOX, 1));
  const box = firebox(f);
  assert.equal(Number(f.player.getAttribute('quest.swan_song.firebox-loaded')) || 0, 0);
  assert.equal(Number(f.player.getAttribute('quest.swan_song.firebox-lit')) || 0, 0);
  use(f, box, core.ItemIdentifiers.LOGS);
  assert.equal(Number(f.player.getAttribute('quest.swan_song.firebox-loaded')), 1);
  assert.equal(inv.getAmount(core.ItemIdentifiers.LOGS), 0);
  assert.equal(inv.getAmount(core.ItemIdentifiers.TINDERBOX), 1);
  f.player.closeInterruptibleInterfaces();
  return { ...f, box };
}
function ignite(f, current) {
  const skill = core.Skill.FIREMAKING, manager = f.player.getSkillManager();
  const xp = manager.getExperience(skill);
  assert.equal(quest('swan_song').getStage(f.player), 65);
  assert.equal(Number(f.player.getAttribute('quest.swan_song.firebox-loaded')), 1);
  assert.equal(manager.getCurrentLevel(skill), current);
  assert.equal(xp, core.SkillManager.getExperienceForLevel(manager.getMaxLevel(skill)));
  use(f, f.box, core.ItemIdentifiers.TINDERBOX);
  assert.equal(quest('swan_song').getStage(f.player), 65);
  assert.equal(manager.getExperience(skill), xp);
}
test('Firemaking 42 lights the loaded firebox', t => {
  const f = setup(t, 42); ignite(f, 42);
  assert.equal(Number(f.player.getAttribute('quest.swan_song.firebox-lit')), 1);
  assert.equal(f.player.getInventory().getAmount(core.ItemIdentifiers.TINDERBOX), 1);
});
test('Firemaking 41 cannot light the loaded firebox', t => {
  const f = setup(t, 41); ignite(f, 41);
  assert.equal(Number(f.player.getAttribute('quest.swan_song.firebox-lit')) || 0, 0);
  assert.equal(Number(f.player.getAttribute('quest.swan_song.firebox-loaded')), 1);
  assert.equal(f.player.getInventory().getAmount(core.ItemIdentifiers.TINDERBOX), 1);
});
test('effective Firemaking 42 admits a boosted level 40', t => {
  const f = setup(t, 40, 42); ignite(f, 42);
  assert.equal(Number(f.player.getAttribute('quest.swan_song.firebox-lit')), 1);
});
test('an already-lit firebox remains lit after the level falls', t => {
  const f = setup(t, 42); ignite(f, 42);
  assert.equal(Number(f.player.getAttribute('quest.swan_song.firebox-lit')), 1);
  f.player.closeInterruptibleInterfaces();
  f.player.getSkillManager().setCurrentLevel(core.Skill.FIREMAKING, 41, false);
  ignite(f, 41);
  assert.equal(Number(f.player.getAttribute('quest.swan_song.firebox-lit')), 1);
});
