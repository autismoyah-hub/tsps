/**
 * Ilfeen-made crystal bows and shields carry 2,500 charges, then become inactive.
 * Untracked preexisting items retain their existing behavior.
 * https://oldschool.runescape.wiki/w/Crystal_bow
 * https://oldschool.runescape.wiki/w/Crystal_shield
 * A bow shot spends one charge (inferred from the documented shot count); a
 * shield spends one when a non-zero combat hit lands. Revert is separate.
 */
const CHARGES_META = "crystal-bow-shield:charges";
const FULL_CHARGES = 2500;
let api;
let core;
const spent = new WeakMap();
const observedHits = new WeakSet();
const impactShields = new WeakMap();

function isBow(id) {
  return id === core.ItemIdentifiers.CRYSTAL_BOW;
}
function isShield(id) {
  return id === core.ItemIdentifiers.CRYSTAL_SHIELD;
}
function isManaged(item) { return item?.getMetaValue?.(CHARGES_META) !== undefined; }
function remaining(item) {
  const value = item.getMetaValue(CHARGES_META);
  if (value === undefined) return 0;
  return Number.isSafeInteger(value) && value >= 0 ? Math.min(FULL_CHARGES, value) : 0;
}
function refresh(player) {
  player.getInventory().refreshItems();
  player.getEquipment().refreshItems();
  core.BonusManager.update(player);
  player.getUpdateFlag().flag(core.Flag.APPEARANCE);
  core.WeaponInterfaceManager.assign(player);
}
function checkAmmo(player, _amountRequired, silent) {
  const item = player.getEquipment().get(core.Equipment.WEAPON_SLOT);
  if (!item || !isBow(item.getId()) || !isManaged(item) || remaining(item) > 0) return null;
  if (!silent) player.sendMessage("Your crystal bow has no charges left. Revert it to a seed.");
  return false;
}

function decrementAmmo(player, _pos, amount) {
  const item = player.getEquipment().get(core.Equipment.WEAPON_SLOT);
  if (!item || !isBow(item.getId()) || !isManaged(item)) return false;
  let left = remaining(item);
  for (let shot = 0; shot < amount && left > 0; shot++) {
    const event = { player, itemId: item.getId(), skip: false };
    api.emitCustomEvent("combat:consume-charge", event);
    if (!event.skip) left--;
  }
  item.setMetaValue(CHARGES_META, left);
  if (left === 0) spent.set(player, item);
  return true;
}

function attackFinished({ attacker }) {
  const player = attacker?.isPlayer?.() ? attacker.getAsPlayer() : null;
  const item = player && spent.get(player);
  if (!item) return;
  spent.delete(player);
  if (!isBow(item.getId()) || remaining(item) !== 0) return;
  const equipment = player.getEquipment().getItems();
  const inventory = player.getInventory().getItems();
  if (!equipment.includes(item) && !inventory.includes(item)) return;
  item.setId(core.ItemIdentifiers.CRYSTAL_BOW_INACTIVE_);
  item.setMetaValue(CHARGES_META, 0);
  refresh(player);
  player.sendMessage("Your crystal bow has run out of charges.");
}

/** Attach to this combat hit; the callback runs only if the queued hit is applied. */
function incomingCombatHit(entity, hit) {
  if (!entity?.isPlayer?.() || !hit?.onApplied || observedHits.has(hit)) return;
  observedHits.add(hit);
  hit.onApplied((damage) => {
    const item = impactShields.get(hit);
    impactShields.delete(hit);
    if (!(damage > 0)) return;
    const player = entity.getAsPlayer();
    if (!item || !isShield(item.getId()) || !isManaged(item)) return;
    if (!player.getEquipment().getItems().includes(item) && !player.getInventory().getItems().includes(item)) return;
    const left = Math.max(0, remaining(item) - 1);
    if (left === 0) {
      item.setId(core.ItemIdentifiers.CRYSTAL_SHIELD_INACTIVE_);
      item.setMetaValue(CHARGES_META, 0);
      refresh(player);
      player.sendMessage("Your crystal shield has run out of charges.");
    } else {
      item.setMetaValue(CHARGES_META, left);
      player.getEquipment().refreshItems();
    }
  });
}

/** Capture the worn shield at application before health and reactive effects change state. */
function damageApplying({ target, hit }) {
  if (!hit || !observedHits.has(hit) || !target?.isPlayer?.()) return;
  const item = target.getAsPlayer().getEquipment().get(core.Equipment.SHIELD_SLOT);
  if (item && isShield(item.getId()) && isManaged(item)) impactShields.set(hit, item);
}

function check({ player, item }) {
  if (!item || !isManaged(item) || (!isBow(item.getId()) && !isShield(item.getId()))) return false;
  player.sendMessage(`Your ${isBow(item.getId()) ? "crystal bow" : "crystal shield"} has ${remaining(item).toLocaleString("en-US")} charges remaining.`);
  return true;
}

function revert({ player, item, slot }) {
  if (!item || !isManaged(item)) return false;
  const id = item.getId();
  if (!isBow(id) && !isShield(id) && id !== core.ItemIdentifiers.CRYSTAL_BOW_INACTIVE_
    && id !== core.ItemIdentifiers.CRYSTAL_SHIELD_INACTIVE_) return false;
  const charges = item.getMetaValue(CHARGES_META);
  api.sendMultiChatboxPrompt(player, "Revert this crystal equipment to a seed?",
    "Yes, revert it.", () => {
      if (player.getInventory().getItems()[slot] !== item || item.getId() !== id
        || item.getMetaValue(CHARGES_META) !== charges) {
        player.sendMessage("That item has changed. Please try again.");
        return;
      }
      item.setId(core.ItemIdentifiers.CRYSTAL_WEAPON_SEED);
      item.setMetaValue(CHARGES_META, undefined);
      player.getInventory().refreshItems();
      player.sendMessage("You revert the crystal equipment to a crystal weapon seed.");
    },
    "No, keep it.", () => {});
  return true;
}

function attach(pluginApi) { api = pluginApi; core = api.core; }
module.exports = {
  name: "CrystalBowShield",
  description: "Charges and reversion for explicitly tracked crystal bows and shields.",
  members: true,
  register(pluginApi) {
    attach(pluginApi);
    api.registerRangedAmmoHandler({ checkAmmo, decrementAmmo });
    api.registerIncomingDamageModifier(incomingCombatHit);
    api.onCustomEvent("combat:attack-finished", attackFinished);
    api.onCustomEvent("combat:damage-apply", damageApplying);
    api.onItemAction("Crystal bow", { Check: check, Revert: revert });
    api.onItemAction("Crystal bow (inactive)", { Revert: revert });
    api.onItemAction("Crystal shield", { Check: check, Revert: revert });
    api.onItemAction("Crystal shield (inactive)", { Revert: revert });
  },
};
