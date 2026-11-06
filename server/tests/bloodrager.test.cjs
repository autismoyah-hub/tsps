// Real Player/equipment, registered weapon handler, attack and NPC hit queue. No live World claim.
const previousBotsSetting = process.env.DISABLE_PLAYER_BOTS;
process.env.DISABLE_PLAYER_BOTS = '1';
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after, test } = require('node:test');
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');
let core, observed = null, nextIndex = 6200;

before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  require('../plugins/items/ItemDefinitionLoader.plugin').register({ log() {}, onPlayerLogin() {}, registerContentEndpoint() {} });
  new (require('../dist/game/definition/loader/impl/NpcDefinitionLoader').NpcDefinitionLoader)().load();
  core = PluginManager.getCoreApi();
  core.RegionManager.init();
  core.RegionManager.loadMapFiles(3222, 3218);
  const api = PluginManager.createApi('BloodragerNative');
  require('../plugins/combat/specials/DualMacuahuitl.SpecialAttack')(api);
  require('../plugins/items/Food.plugin').register(api);
  api.onCombatHitDamage(event => {
    if (!observed || event.attacker !== observed.player) return;
    if (observed.forceZero && event.accurate) event.damage.setDamage(0);
    observed.splats.push(event);
    if (observed.rejectBeforeQueue && observed.splats.length === 2) event.target.setUntargetable(true);
  });
  api.onPlayerDealtDamage(({ player, target }) => {
    if (observed?.player === player && observed.changeAfterAdmissionCheck) target.setUntargetable(true);
  });
  api.onCustomEvent('combat:hit-rolled', event => { if (observed && event.attacker === observed.player) observed.ranges.push({ minimum: event.minimum, maximum: event.maximum }); });
});
after(() => {
  if (previousBotsSetting === undefined) delete process.env.DISABLE_PLAYER_BOTS;
  else process.env.DISABLE_PLAYER_BOTS = previousBotsSetting;
});

function actors(t, full = true, broken = false) {
  const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => true, send: () => true,
    close() {}, onData() {}, onError() {}, onClose() {} };
  const player = new Player(new PlayerSession(channel));
  let npc = null, registered = false;
  const errors = [], originalError = console.error;
  const submitted = [], originalSubmit = core.TaskManager.submit;
  console.error = function (...args) { errors.push(args.map(String).join(' ')); originalError.apply(console, args); };
  core.TaskManager.submit = function (task) {
    submitted.push(task);
    return originalSubmit.call(this, task);
  };
  t.after(() => {
    try {
      observed = null;
      for (const task of submitted) task.stop();
      if (registered) PluginManager.emitPlayerLogout({ player, username: player.getUsername() });
      player.setRegistered(false);
      core.TaskManager.cancelTasks(player.getIndex());
      if (npc) { npc.getCombat().getHitQueue().clear(); PluginManager.removeNpc(npc); }
    } finally {
      try { for (const task of submitted) task.stop(); }
      finally { core.TaskManager.submit = originalSubmit; console.error = originalError; }
    }
    assert.deepEqual(errors, [], 'no swallowed hook, task or teardown errors');
  });
  player.setIndex(nextIndex++); player.setUsername('Bloodrager' + player.getIndex());
  player.setRights(core.PlayerRights.NONE);
  player.setRegistered(true); registered = true; player.setAttribute('bot-skip-persistence', true);
  player.setLocation(new core.Location(3222, 3218, 0)); player.setNeedsPlacement(false);
  for (const skill of [core.Skill.HITPOINTS, core.Skill.ATTACK, core.Skill.STRENGTH, core.Skill.DEFENCE]) {
    player.getSkillManager().setMaxLevel(skill, 99); player.getSkillManager().setCurrentLevel(skill, 99);
  }
  player.setHitpoints(99);
  const I = core.ItemIdentifiers, E = core.Equipment;
  player.getEquipment().setItem(E.WEAPON_SLOT, new core.Item(I.DUAL_MACUAHUITL, 1));
  if (full) {
    player.getEquipment().setItem(E.HEAD_SLOT, new core.Item(broken ? I.BLOOD_MOON_HELM_BROKEN_ : I.BLOOD_MOON_HELM, 1));
    player.getEquipment().setItem(E.BODY_SLOT, new core.Item(broken ? I.BLOOD_MOON_CHESTPLATE_BROKEN_ : I.BLOOD_MOON_CHESTPLATE, 1));
    player.getEquipment().setItem(E.LEG_SLOT, new core.Item(broken ? I.BLOOD_MOON_TASSETS_BROKEN_ : I.BLOOD_MOON_TASSETS, 1));
  }
  const style = core.WeaponInterfaces.MACE;
  player.setWeapon(style); player.setFightType(style.getFightType()[0]);
  core.BonusManager.update(player);
  npc = PluginManager.spawnNpc({ id: core.NpcIdentifiers.GOBLIN, x: 3223, y: 3218, z: 0, wanderRadius: 0 });
  assert.ok(npc && npc.isRegistered()); npc.setNeedsPlacement(false);
  return { player, npc };
}

// Deterministic only inside the actual attack; unrelated random calls keep their real source.
function attack(player, npc, accuracy, procRolls, options = {}) {
  const accuracyOwner = core.AccuracyFormulasDpsCalc, originalAccuracy = accuracyOwner.rollAccuracy;
  const originalRandom = Math.random, rolls = [...accuracy], proc = [...procRolls];
  let unexpectedAccuracy = 0, unexpectedProc = 0;
  observed = { player, splats: [], ranges: [], accuracyRolls: [], ...options };
  accuracyOwner.rollAccuracy = function (attacker, target, ...rest) {
    if (attacker === player && target === npc) {
      observed.accuracyRolls.push(accuracyOwner.attackMeleeRoll(attacker));
      if (!rolls.length) { unexpectedAccuracy++; return false; }
      return rolls.shift();
    }
    return originalAccuracy.call(this, attacker, target, ...rest);
  };
  Math.random = function () {
    if (new Error().stack?.includes('attackFinished')) {
      if (!proc.length) { unexpectedProc++; return 0.99; }
      return proc.shift() / 100;
    }
    return originalRandom();
  };
  try {
    const method = core.CombatFactory.getMethod(player);
    assert.equal(core.CombatFactory.canAttackPermission(player, npc, false, method), core.CanAttackResponse.CAN_ATTACK,
      'real adjacent actors pass the ordinary attack policy');
    const launched = player.getCombat().executeAttack(method, npc, false, 0, false);
    assert.equal(unexpectedAccuracy, 0, 'no unplanned accuracy roll');
    assert.equal(unexpectedProc, 0, 'no unplanned set-effect roll');
    return { launched, method, splats: observed.splats, ranges: observed.ranges,
      accuracyRolls: observed.accuracyRolls, remainingAccuracy: rolls, remainingProc: proc };
  } finally {
    accuracyOwner.rollAccuracy = originalAccuracy;
    Math.random = originalRandom;
    observed = null;
  }
}

test('dual attack rolls two dependent splats and retains the odd maximum remainder', t => {
  assert.equal(core.CacheDefinitions.getObjId('dual_macuahuitl'), core.ItemIdentifiers.DUAL_MACUAHUITL);
  const { player, npc } = actors(t, false);
  let max = 0, chosen = 0;
  for (let level = 75; level <= 99; level++) {
    player.getSkillManager().setCurrentLevel(core.Skill.STRENGTH, level);
    max = core.DamageFormulas.calculateMaxMeleeHit(player);
    if (max > 1 && max % 2 === 1) { chosen = level; break; }
  }
  assert.ok(chosen, 'real equipped player has an attainable odd maximum');
  assert.equal(player.getSkillManager().getCurrentLevel(core.Skill.STRENGTH), chosen);
  const result = attack(player, npc, [true, true], []);
  assert.equal(result.launched, true, 'actual combat accepted the attack');
  assert.equal(result.splats.length, 2, 'ordinary dual attack rolls both splats');
  assert.equal(result.splats[0].hit, result.splats[1].hit);
  assert.deepEqual(result.splats[0].hit.getHitDelays(), [0, 1]);
  assert.deepEqual(result.ranges.map(range => range.maximum), [Math.floor(max / 2), Math.ceil(max / 2)]);
  assert.deepEqual(result.remainingAccuracy, []);
});

test('first normal miss suppresses the second accuracy roll', t => {
  const { player, npc } = actors(t, false);
  const result = attack(player, npc, [false], []);
  assert.equal(result.launched, true);
  assert.equal(result.splats.length, 2);
  assert.deepEqual(result.splats.map(splat => splat.accurate), [false, false]);
  assert.deepEqual(result.remainingAccuracy, []);
});

test('a different melee weapon still produces one ordinary splat', t => {
  const { player, npc } = actors(t, false);
  player.getEquipment().setItem(core.Equipment.WEAPON_SLOT, new core.Item(core.ItemIdentifiers.BRONZE_DAGGER, 1));
  core.BonusManager.update(player);
  const result = attack(player, npc, [true], []);
  assert.equal(result.launched, true);
  assert.equal(result.splats.length, 1);
});

test('each accurate splat can shorten the next attack once, including broken armour', t => {
  assert.equal(core.CacheDefinitions.getObjId('blood_moon_helm_broken'), core.ItemIdentifiers.BLOOD_MOON_HELM_BROKEN_);
  assert.equal(core.CacheDefinitions.getObjId('blood_moon_chestplate_broken'), core.ItemIdentifiers.BLOOD_MOON_CHESTPLATE_BROKEN_);
  assert.equal(core.CacheDefinitions.getObjId('blood_moon_tassets_broken'), core.ItemIdentifiers.BLOOD_MOON_TASSETS_BROKEN_);
  const { player, npc } = actors(t, true, true);
  const result = attack(player, npc, [true, true], [33, 32]);
  assert.equal(result.launched, true);
  assert.equal(result.splats.length, 2);
  assert.deepEqual(result.remainingProc, [], 'both accurate splats consume their independent roll');
  assert.equal(player.getCombat().getAttackDelay(), 3, 'the second successful roll makes the next attack one tick sooner');
});

test('a missing set leaves the regular attack delay unchanged', t => {
  const { player, npc } = actors(t, false);
  const result = attack(player, npc, [true, true], [0, 0]);
  assert.equal(result.launched, true);
  assert.equal(result.splats.length, 2);
  assert.deepEqual(result.remainingProc, [0, 0]);
  assert.equal(player.getCombat().getAttackDelay(), 4);
});

test('an accurate zero may proc but a missed splat does not roll the effect', t => {
  const { player, npc } = actors(t, true);
  // The native damage event supplies the zero; accuracy comes from the real hit path.
  const result = attack(player, npc, [true, false], [0], { forceZero: true });
  assert.equal(result.launched, true);
  assert.deepEqual(result.splats.map(splat => splat.accurate), [true, false]);
  assert.equal(result.splats[0].damage.getDamage(), 0);
  assert.deepEqual(result.remainingProc, [], 'only the accurate splat consumed a proc roll');
  assert.equal(player.getCombat().getAttackDelay(), 3);
});

test('Blood Infusion permits a second accurate splat after the first misses', t => {
  const { player, npc } = actors(t, true);
  player.setCombatSpecial(core.CombatSpecial.getById('dual_macuahuitl'));
  player.setSpecialPercentage(100); player.setSpecialActivated(true);
  const beforeHp = player.getHitpoints();
  const result = attack(player, npc, [false, true], []);
  assert.equal(result.launched, true);
  assert.deepEqual(result.splats.map(splat => splat.accurate), [false, true]);
  assert.equal(player.getHitpoints(), beforeHp - Math.floor(beforeHp / 4));
  assert.equal(player.getSpecialPercentage(), 75);
  assert.equal(player.getCombat().getAttackDelay(), 3, 'a successful special guarantees one earlier tick');
});

test('a missed Blood Infusion does not trigger its guaranteed effect', t => {
  const { player, npc } = actors(t, true);
  player.setCombatSpecial(core.CombatSpecial.getById('dual_macuahuitl'));
  player.setSpecialPercentage(100); player.setSpecialActivated(true);
  const result = attack(player, npc, [false, false], []);
  assert.equal(result.launched, true);
  assert.deepEqual(result.splats.map(splat => splat.accurate), [false, false]);
  assert.equal(player.getCombat().getAttackDelay(), 4);
});

test('Blood Infusion requires the full armour set', t => {
  const { player, npc } = actors(t, false);
  player.setCombatSpecial(core.CombatSpecial.getById('dual_macuahuitl'));
  player.setSpecialPercentage(100); player.setSpecialActivated(true);
  const method = core.CombatFactory.getMethod(player);
  assert.equal(core.CombatFactory.canAttack(player, method, npc, false, false), core.CanAttackResponse.COMBAT_METHOD_NOT_ALLOWED);
  assert.equal(player.getSpecialPercentage(), 100);
});

test('special accuracy is isolated from normal rolls in the same cycle', t => {
  const { player, npc } = actors(t, true);
  player.setCombatSpecial(core.CombatSpecial.getById('dual_macuahuitl'));
  player.setSpecialPercentage(100);
  const cycle = core.World.getProcessCycle();
  // A direct formula call establishes the normal cached-roll precondition; it is
  // not a claim that this fixture ran a cannon task or the full World loop.
  const normal = core.AccuracyFormulasDpsCalc.attackMeleeRoll(player);
  assert.ok(normal > 0);
  player.setSpecialActivated(true);
  const result = attack(player, npc, [false, true], []);
  assert.equal(result.launched, true);
  assert.equal(core.World.getProcessCycle(), cycle);
  assert.deepEqual(result.accuracyRolls, [Math.floor(normal * 1.25), Math.floor(normal * 1.25)]);
  assert.equal(player.isSpecialActivated(), false);
  assert.equal(core.AccuracyFormulasDpsCalc.attackMeleeRoll(player), normal);
});

test('special increases the real maximum before dividing it between splats', t => {
  const { player, npc } = actors(t, true);
  player.setCombatSpecial(core.CombatSpecial.getById('dual_macuahuitl'));
  player.setSpecialPercentage(100);
  const normalMaximum = core.DamageFormulas.calculateMaxMeleeHit(player);
  assert.ok(normalMaximum > 0);
  player.setSpecialActivated(true);
  const result = attack(player, npc, [true, true], []);
  const specialMaximum = Math.floor(normalMaximum * 1.25);
  assert.equal(result.launched, true);
  assert.deepEqual(result.ranges.map(range => range.maximum),
    [Math.floor(specialMaximum / 2), Math.ceil(specialMaximum / 2)]);
});

test('an injected post-modifier maximum of 47 divides into 23 and 24', t => {
  const { player, npc } = actors(t, true);
  const original = core.DamageFormulas.sourceMaxHit;
  core.DamageFormulas.sourceMaxHit = function (entity, type, ...rest) {
    return entity === player && type === core.CombatType.MELEE ? 47 : original.call(this, entity, type, ...rest);
  };
  try {
    const result = attack(player, npc, [true, true], [33, 33]);
    assert.equal(result.launched, true);
    assert.deepEqual(result.ranges.map(range => range.maximum), [23, 24]);
  } finally {
    core.DamageFormulas.sourceMaxHit = original;
  }
});

test('two successful procs still shorten only one tick and eating adds its own delay', t => {
  const { player, npc } = actors(t, true);
  const result = attack(player, npc, [true, true], [0, 0]);
  assert.equal(result.launched, true);
  assert.deepEqual(result.remainingProc, []);
  assert.equal(player.getCombat().getAttackDelay(), 3);
  const shark = new core.Item(core.ItemIdentifiers.SHARK, 1);
  player.getInventory().setItem(0, shark);
  player.setHitpoints(50);
  const event = { player, item: shark, itemId: shark.getId(), slot: 0,
    interfaceId: core.Inventory.INTERFACE_ID, clickType: 1, handled: false };
  PluginManager.emitItemAction(event);
  assert.equal(event.handled, true, 'the registered real food action handled the shark');
  assert.equal(player.getInventory().getAmount(shark.getId()), 0);
  assert.equal(player.getCombat().getAttackDelay(), 6, 'the shark adds three ticks to the shortened timer');
});

test('an older queued hit cannot make a rejected new swing proc', t => {
  const { player, npc } = actors(t, true);
  const originalCycle = core.World.processCycle;
  t.after(() => { core.World.processCycle = originalCycle; });
  const old = attack(player, npc, [true, true], [0, 0]);
  assert.equal(old.launched, true);
  assert.equal(npc.getCombat().getHitQueue().hasPendingHitFrom(player), true);
  core.World.processCycle += 4;
  // The injected second-roll callback makes only the new hit fail queue admission.
  const rejected = attack(player, npc, [true, true], [0, 0], { rejectBeforeQueue: true });
  assert.equal(rejected.launched, true);
  assert.equal(npc.isUntargetable(), true, 'the second-roll callback fired');
  assert.equal(npc.getCombat().getHitQueue().hasPendingHitFrom(player), true, 'older hit remains queued');
  assert.equal(npc.getCombat().getHitQueue().hasPendingHit(old.splats[0].hit), true);
  assert.equal(npc.getCombat().getHitQueue().hasPendingHit(rejected.splats[0].hit), false);
  assert.deepEqual(rejected.remainingProc, [0, 0]);
  assert.equal(player.getCombat().getAttackDelay(), 4);
});

test('a queued hit may proc when an admission callback changes the target state', t => {
  const { player, npc } = actors(t, true);
  // This observer runs after addPendingHit's target check, before actual queue insertion.
  const queued = attack(player, npc, [true, true], [0, 0], { changeAfterAdmissionCheck: true });
  assert.equal(queued.launched, true);
  assert.equal(npc.isUntargetable(), true);
  assert.equal(npc.getCombat().getHitQueue().hasPendingHitFrom(player), true);
  assert.equal(npc.getCombat().getHitQueue().hasPendingHit(queued.splats[0].hit), true);
  assert.deepEqual(queued.remainingProc, []);
  assert.equal(player.getCombat().getAttackDelay(), 3);
});

test('a later different swing is not shortened when the earlier hit reaches its queue', t => {
  const { player, npc } = actors(t, true);
  const originalCycle = core.World.processCycle;
  t.after(() => { core.World.processCycle = originalCycle; });
  const first = attack(player, npc, [true, true], [0, 0]);
  assert.equal(first.launched, true);
  assert.equal(player.getCombat().getAttackDelay(), 3);
  core.World.processCycle += 3;
  player.getEquipment().setItem(core.Equipment.WEAPON_SLOT, new core.Item(core.ItemIdentifiers.BRONZE_DAGGER, 1));
  core.BonusManager.update(player);
  const second = attack(player, npc, [true], []);
  assert.equal(second.launched, true);
  const current = player.getCombat().getAttackDelay();
  assert.ok(current > 0);
  npc.getCombat().getHitQueue().process(core.World.processCycle);
  assert.equal(player.getCombat().getAttackDelay(), current, 'a prior pending hit cannot shorten a newer swing');
});
