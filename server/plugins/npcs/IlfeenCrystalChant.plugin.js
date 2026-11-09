/**
 * Ilfeen's post-Roving Elves crystal weapon seed service.
 * Prices and shared discounts: https://oldschool.runescape.wiki/w/Crystal_weapon_seed
 * The inserted menu and refusal wording are server choices; the retained transcript
 * does not describe the full transaction or the current dialogue sequence.
 */
const { startTranscript } = require("../quests/QuestRuntime");

const PAID_CHANTS_ATTRIBUTE = "ilfeen:paid-crystal-chants";
const LATER_VARIANT = "after-roving-elves-after-the-first-time";
const FIRST_REFERENCE = "TUezAf";
const GREETING = "I am, but you'll need to supply the seed.";
const PRICES = {
  bow: [900000, 720000, 540000, 360000, 180000],
  shield: [750000, 600000, 450000, 300000, 150000],
};

let api;
let core;
const offers = new WeakMap();

function isIlfeen(npcId) {
  return npcId === core.NpcIdentifiers.ILFEEN || npcId === core.NpcIdentifiers.ILFEEN_2;
}

function complete(player) {
  const request = { player, key: "roving_elves", complete: null };
  api.emitCustomEvent("quest:is-complete", request);
  return request.complete === true;
}

function count(player) {
  const value = Number(player.getAttribute(PAID_CHANTS_ATTRIBUTE));
  if (player.getAttribute(PAID_CHANTS_ATTRIBUTE) == null) return 0;
  return Number.isSafeInteger(value) && value >= 0 ? Math.min(value, 4) : 4;
}

function continuation(player, npcId, npc) {
  // Preserve the existing Eyes of Glouphrie and book branches after the new menu.
  return startTranscript(api, player, npcId, "Ilfeen", LATER_VARIANT, (steps) => steps.slice(4), npc);
}

function openMenu(player, npcId, npc) {
  if (!complete(player) || !isIlfeen(npcId)) return;
  const tier = count(player);
  const token = {};
  offers.set(player, token);
  const pick = (kind) => () => {
    if (offers.get(player) !== token) return;
    offers.delete(player);
    buy(player, npcId, kind, tier);
  };
  api.sendMultiChatboxPrompt(player, "What would you like Ilfeen to make?",
    `Crystal bow (${PRICES.bow[tier].toLocaleString("en-US")} coins).`, pick("bow"),
    `Crystal shield (${PRICES.shield[tier].toLocaleString("en-US")} coins).`, pick("shield"),
    "Continue talking.", () => {
      if (offers.get(player) !== token) return;
      offers.delete(player);
      continuation(player, npcId, npc);
    });
}

/** Recheck the quote and commit a preflighted inventory without replacing unrelated item identities. */
function buy(player, npcId, kind, quotedTier) {
  if (!isIlfeen(npcId) || !Object.hasOwn(PRICES, kind)) return;
  if (!complete(player) || count(player) !== quotedTier) {
    player.sendMessage("That offer has changed. Please speak to Ilfeen again.");
    return;
  }
  const inventory = player.getInventory();
  const seed = core.ItemIdentifiers.CRYSTAL_WEAPON_SEED;
  const coins = core.ItemIdentifiers.COINS;
  const fee = PRICES[kind][quotedTier];
  if (inventory.getAmount(seed) < 1 || inventory.getAmount(coins) < fee) {
    player.sendMessage("You need a crystal weapon seed and enough coins for that.");
    return;
  }
  const product = kind === "bow" ? core.ItemIdentifiers.CRYSTAL_BOW : core.ItemIdentifiers.CRYSTAL_SHIELD;
  const before = inventory.getItems();
  const plan = new core.Inventory(player).setItems(inventory.getCopiedItems());
  plan.deleted(seed, 1, false);
  plan.deleted(coins, fee, false);
  const output = new core.Item(product, 1);
  output.setMetaValue("crystal-bow-shield:charges", 2500);
  if (plan.getAddableAmount(output) < 1) {
    player.sendMessage("You do not have enough inventory space.");
    return;
  }
  plan.add(output, false);
  const after = plan.getItems();
  for (let slot = 0; slot < before.length; slot++) {
    const old = before[slot];
    const next = after[slot];
    if (old.equals(next)) continue;
    // Preserve a surviving coin stack's identity; removed and new items get new identities.
    if (old.getId() === coins && next.getId() === coins && old.getAmount() > next.getAmount()) {
      old.setAmount(next.getAmount());
    } else {
      inventory.setItem(slot, next);
    }
  }
  player.setAttribute(PAID_CHANTS_ATTRIBUTE, Math.min(4, quotedTier + 1));
  inventory.refreshItems();
  player.sendMessage(`Ilfeen chants your seed into a crystal ${kind}.`);
}

// The service menu already offers both choices with the current price. The
// dump's static-price branch has no transaction and must not claim a handover.
function condition(event) {
  if (!isIlfeen(event.npcId) || !event.pages?.some(({ page }) => page === "Ilfeen")) return null;
  if (event.text === "With a crystal seed:") return false;
  return null;
}

function continueFirstConversation(event) {
  if (!event.player || !isIlfeen(event.npcId) || event.stepId !== FIRST_REFERENCE || !complete(event.player)) return;
  if (startTranscript(api, event.player, event.npcId, "Ilfeen", LATER_VARIANT, undefined, event.npc)) {
    event.handled = true;
    event.end = true;
  }
}

function offerFromConversation(event) {
  if (!event.player || !isIlfeen(event.npcId) || event.text !== GREETING || !complete(event.player)) return;
  if (!event.pages?.some(({ page }) => page === "Ilfeen")) return;
  if (event.player.getInventory().getAmount(core.ItemIdentifiers.CRYSTAL_WEAPON_SEED) < 1) return;
  event.after = () => openMenu(event.player, event.npcId, event.npc);
}

function attach(pluginApi) {
  api = pluginApi;
  core = api.core;
}

module.exports = {
  name: "IlfeenCrystalChant",
  description: "Ilfeen chants crystal weapon seeds into bows and shields for a shared progressive coin fee.",
  members: true,
  register(pluginApi) {
    attach(pluginApi);
    api.persistAttribute(PAID_CHANTS_ATTRIBUTE, 0);
    api.onNpcDialogueCondition(condition);
    api.onCustomEvent("npc-dialogue:action", continueFirstConversation);
    api.onCustomEvent("npc-dialogue:line", offerFromConversation);
  },
};
