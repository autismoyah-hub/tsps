// Run after `yarn build`: node --test tests/crystal-bow-shots-per-bow.test.cjs
// A crystal bow drops a stage every 250 shots. The count was kept on the player, keyed by the worn bow's id, so firing
// one shot from a second bow reset the first bow's count: swapping two bows, neither ever degraded. The count is on
// each bow now. Real Player and equipment, the real CombatFactory.decrementAmmo.
const assert = require("node:assert/strict");
const path = require("node:path");
const { before, test } = require("node:test");
require("../dist/Server").Server.installProductionPathResolver();
const { CachePipeline } = require("../dist/game/cache/CachePipeline");
const { PluginManager } = require("../dist/plugins/PluginManager");
const { Player } = require("../dist/game/entity/impl/player/Player");
const { CombatFactory } = require("../dist/game/content/combat/CombatFactory");
const { RangedWeapon } = require("../dist/game/content/combat/ranged/RangedData");
const { ItemIdentifiers: I } = require("../dist/util/ItemIdentifiers");

let core;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, ".."));
  core = PluginManager.getCoreApi();
  const implementation = path.resolve(__dirname, "../plugins/items/CrystalBowShield.plugin.js");
  if (require("node:fs").existsSync(implementation)) require(implementation).register(PluginManager.createApi("CrystalBowShieldTest"));
});

function archer(t) {
  const p = new Player(null);
  p.getSession = () => ({ sendClientPacket: () => true, write: () => true });
  p.setUsername("crystal-archer");
  t.mock.method(p.getCombat(), "getRangedWeapon", () => RangedWeapon.CRYSTAL_BOW);
  return p;
}
const wield = (p, item) => p.getEquipment().setItem(core.Equipment.WEAPON_SLOT, item);
const fire = (p, shots) => CombatFactory.decrementAmmo(p, p.getLocation(), shots);

test("a kept shot with extra charges given back (combat:consume-charge extra) takes shots off the bow's stage count", (t) => {
  const p = archer(t);
  const bow = new core.Item(I.CRYSTAL_BOW_FULL, 1);
  wield(p, bow);
  fire(p, 10);
  const emit = PluginManager.emitCustomEvent;
  t.mock.method(PluginManager, "emitCustomEvent", (name, event) => {
    if (name === "combat:consume-charge" && event.player === p) { event.skip = true; event.extra = 3; return; }
    return emit.call(PluginManager, name, event);
  });
  fire(p, 1);
  assert.equal(bow.getMetaValue(CombatFactory.CRYSTAL_BOW_SHOTS_META), 7);
  fire(p, 5);
  assert.equal(bow.getMetaValue(CombatFactory.CRYSTAL_BOW_SHOTS_META), 0, "never below the stage's start");
});

test("each crystal bow keeps its own shot count: swapping bows doesn't reset it", (t) => {
  const p = archer(t);
  const first = new core.Item(I.CRYSTAL_BOW_FULL, 1);
  const second = new core.Item(I.CRYSTAL_BOW_FULL, 1);
  wield(p, first);
  fire(p, 249);
  assert.equal(first.getId(), I.CRYSTAL_BOW_FULL, "249 shots: still full");
  wield(p, second);
  fire(p, 1);
  wield(p, first);
  fire(p, 1);
  assert.equal(first.getId(), I.CRYSTAL_BOW_9_10, "the 250th shot from this bow drops it a stage");
  assert.equal(second.getId(), I.CRYSTAL_BOW_FULL, "the other bow fired once");
});

// The equipped item and count are fixture-set; the real ammo dispatcher and
// attack-finished event are exercised without claiming a full projectile run.
test('current crystal bow keeps its count on the item and becomes inactive after the last shot', (t) => {
  const p = archer(t), bow = new core.Item(I.CRYSTAL_BOW, 1);
  wield(p, bow);
  fire(p, 2499);
  assert.equal(bow.getId(), I.CRYSTAL_BOW);
  assert.equal(bow.getMetaValue('crystal-bow-shield:charges'), 1);
  fire(p, 1);
  assert.equal(bow.getId(), I.CRYSTAL_BOW, 'last shot retains its weapon until attack-finished');
  PluginManager.emitCustomEvent('combat:attack-finished', { attacker: p });
  assert.equal(bow.getId(), I.CRYSTAL_BOW_INACTIVE_);
});

// Exercise a combat hit's modifier and real hit queue with fixture-set damage;
// this does not claim coverage of every special attack or environmental source.
test('a crystal shield spends a charge only when the marked combat hit lands', (t) => {
  const { HitDamage } = require('../dist/game/content/combat/hit/HitDamage');
  const { HitMask } = require('../dist/game/content/combat/hit/HitMask');
  const { CombatType } = require('../dist/game/content/combat/CombatType');
  const attacker = archer(t), target = archer(t);
  target.setRegistered(true);
  t.after(() => target.setRegistered(false));
  const shield = new core.Item(I.CRYSTAL_SHIELD, 1);
  shield.setMetaValue('crystal-bow-shield:charges', 1);
  target.getEquipment().setItem(core.Equipment.SHIELD_SLOT, shield);
  const hit = new HitDamage(1, HitMask.RED);
  CombatFactory.applyIncomingHitDamage(attacker, target, CombatType.MELEE, hit);
  assert.equal(shield.getId(), I.CRYSTAL_SHIELD, 'roll alone does not spend a charge');
  target.getCombat().getHitQueue().addPendingDamage([hit]);
  target.getCombat().getHitQueue().process(1);
  assert.equal(shield.getId(), I.CRYSTAL_SHIELD_INACTIVE_);
});

test('crystal bow charge-saving hook and independent item counts are preserved', (t) => {
  const p = archer(t), api = PluginManager.createApi('CrystalBowSaveTest');
  let save = true;
  api.onCustomEvent('combat:consume-charge', event => { if (event.player === p && save) event.skip = true; });
  const first = new core.Item(I.CRYSTAL_BOW, 1), second = new core.Item(I.CRYSTAL_BOW, 1);
  wield(p, first); fire(p, 1);
  assert.equal(first.getMetaValue('crystal-bow-shield:charges'), 2500);
  save = false; fire(p, 1);
  wield(p, second); fire(p, 1);
  wield(p, first); fire(p, 1);
  assert.equal(first.getMetaValue('crystal-bow-shield:charges'), 2498);
  assert.equal(second.getMetaValue('crystal-bow-shield:charges'), 2499);
});

test('current bow charge metadata survives player save and restore', (t) => {
  const { PlayerSave } = require('../dist/game/entity/impl/player/persistence/PlayerSave');
  const p = archer(t), restored = archer(t);
  const bow = new core.Item(I.CRYSTAL_BOW, 1);
  bow.setMetaValue('crystal-bow-shield:charges', 17);
  wield(p, bow);
  PlayerSave.fromPlayer(p).applyToPlayer(restored);
  assert.equal(restored.getEquipment().get(core.Equipment.WEAPON_SLOT).getMetaValue('crystal-bow-shield:charges'), 17);
});

test('crystal shield applies charges to the shield worn when the combat hit lands', (t) => {
  const { HitDamage } = require('../dist/game/content/combat/hit/HitDamage');
  const { HitMask } = require('../dist/game/content/combat/hit/HitMask');
  const { CombatType } = require('../dist/game/content/combat/CombatType');
  const attacker = archer(t), target = archer(t);
  target.setRegistered(true);
  t.after(() => target.setRegistered(false));
  const first = new core.Item(I.CRYSTAL_SHIELD, 1), second = new core.Item(I.CRYSTAL_SHIELD, 1);
  first.setMetaValue('crystal-bow-shield:charges', 4);
  second.setMetaValue('crystal-bow-shield:charges', 4);
  target.getEquipment().setItem(core.Equipment.SHIELD_SLOT, first);
  const hit = new HitDamage(1, HitMask.RED);
  CombatFactory.applyIncomingHitDamage(attacker, target, CombatType.MELEE, hit);
  target.getEquipment().setItem(core.Equipment.SHIELD_SLOT, second);
  target.getCombat().getHitQueue().addPendingDamage([hit]);
  target.getCombat().getHitQueue().process(2);
  assert.equal(first.getMetaValue('crystal-bow-shield:charges'), 4);
  assert.equal(second.getMetaValue('crystal-bow-shield:charges'), 3);
  const zero = new HitDamage(0, HitMask.BLUE);
  CombatFactory.applyIncomingHitDamage(attacker, target, CombatType.MELEE, zero);
  target.getCombat().getHitQueue().addPendingDamage([zero]);
  target.getCombat().getHitQueue().process(3);
  assert.equal(second.getMetaValue('crystal-bow-shield:charges'), 3);
  const cancelled = new HitDamage(1, HitMask.RED);
  CombatFactory.applyIncomingHitDamage(attacker, target, CombatType.MELEE, cancelled);
  target.getCombat().getHitQueue().addPendingDamage([cancelled]);
  target.getCombat().getHitQueue().clear();
  target.getCombat().getHitQueue().process(4);
  assert.equal(second.getMetaValue('crystal-bow-shield:charges'), 3);
});

test('the shield captured at damage application is charged if a later hook moves it', (t) => {
  const { HitDamage } = require('../dist/game/content/combat/hit/HitDamage');
  const { HitMask } = require('../dist/game/content/combat/hit/HitMask');
  const { CombatType } = require('../dist/game/content/combat/CombatType');
  const attacker = archer(t), target = archer(t);
  target.setRegistered(true);
  t.after(() => target.setRegistered(false));
  const first = new core.Item(I.CRYSTAL_SHIELD, 1), second = new core.Item(I.CRYSTAL_SHIELD, 1);
  first.setMetaValue('crystal-bow-shield:charges', 4);
  second.setMetaValue('crystal-bow-shield:charges', 4);
  target.getEquipment().setItem(core.Equipment.SHIELD_SLOT, first);
  PluginManager.createApi('CrystalShieldSwapTest').onCustomEvent('combat:damage-apply', event => {
    if (event.target !== target) return;
    target.getInventory().setItem(0, first);
    target.getEquipment().setItem(core.Equipment.SHIELD_SLOT, second);
  });
  const hit = new HitDamage(1, HitMask.RED);
  CombatFactory.applyIncomingHitDamage(attacker, target, CombatType.MELEE, hit);
  target.getCombat().getHitQueue().addPendingDamage([hit]);
  target.getCombat().getHitQueue().process(5);
  assert.equal(first.getMetaValue('crystal-bow-shield:charges'), 3);
  assert.equal(second.getMetaValue('crystal-bow-shield:charges'), 4);
});
