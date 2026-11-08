// Prospective real-engine regression; run after build, not executed during source-only discovery.
// https://oldschool.runescape.wiki/w/Dodgy_Squire?oldid=15304858
// https://oldschool.runescape.wiki/w/Transcript:Dodgy_Squire?oldid=14394381
// Below-cap Talk-to: select the bottom-right eligible item and offer its listed refund.
// Later refund assertions are only reached if the expected offer is first produced.
const assert = require("node:assert/strict");
const path = require("node:path");
const { before, test } = require("node:test");
require("../dist/Server").Server.installProductionPathResolver();
const { CachePipeline } = require("../dist/game/cache/CachePipeline");
const { NpcDefinitionLoader } = require("../dist/game/definition/loader/impl/NpcDefinitionLoader");
const { isMembersWorld } = require("../dist/game/definition/WorldDefinition");
const { PluginManager } = require("../dist/plugins/PluginManager");
const { Player } = require("../dist/game/entity/impl/player/Player");
const { PlayerSession } = require("../dist/net/PlayerSession");
const Rewards = require("../plugins/minigames/pestcontrol/Rewards.PestControl");

let core;
let nextIndex = 5700;

before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, ".."));
  new NpcDefinitionLoader().load();
  assert.equal(isMembersWorld(), true, "reward service requires a members world");
  PluginManager.loadFromDirectory(path.resolve(__dirname, "../plugins"));
  core = PluginManager.getCoreApi();
  core.ObjectDefinition.init();
});

function fixture(t, itemId) {
  const channel = { kind: "test", remoteAddress: "127.0.0.1", isOpen: () => true,
    send: () => true, close() {}, onData() {}, onError() {}, onClose() {} };
  const player = new Player(new PlayerSession(channel));
  player.setIndex(nextIndex++);
  player.setUsername(`squirerefund${player.getIndex()}`);
  player.setRegistered(true);
  player.setLocation(new core.Location(2667, 2656, 0));
  player.needsPlacement = false;
  t.after(() => {
    player.closeInterruptibleInterfaces();
    player.setRegistered(false);
    core.TaskManager.cancelTasks(player.getIndex());
  });
  assert.ok(player.getHitpoints() > 0, "the actor is alive");
  const id = core.NpcIdentifiers.DODGY_SQUIRE;
  assert.equal(id, 2060, "source identifier matches the expected NPC");
  const npc = core.NPC.create(id, new core.Location(2667, 2657, 0));
  assert.equal(npc.getDefinition().getName(), "Dodgy Squire", "the admitted cache supplies the NPC");
  assert.ok(core.ItemDefinition.forId(itemId).getName(), "the admitted cache supplies the item");
  Rewards.setPoints(player, 100);
  assert.equal(Rewards.getPoints(player), 100, "wallet starts below the cap");
  player.getInventory().add(new core.Item(itemId, 1), false);
  const slot = player.getInventory().getItems().findIndex((item) => item?.getId() === itemId);
  assert.ok(slot >= 0, "the exact item is in the inventory");
  const item = player.getInventory().getItems()[slot];
  assert.equal(item.getAmount(), 1);
  return { player, npc, item, slot, itemId };
}

const pending = (player) => core.MultiChatboxPrompt.getPending(player);
function advanceUntilPrompt(player) {
  const manager = player.getDialogueManager();
  for (let i = 0; i < 40 && !pending(player) && manager.isActive(); i++) manager.advance();
  return pending(player);
}
function answer(player, label) {
  const menu = pending(player);
  const index = menu?.options.indexOf(label) ?? -1;
  assert.ok(index >= 0, `choice ${label} is offered: ${JSON.stringify(menu)}`);
  const number = index + 1;
  assert.equal(core.MultiChatboxPrompt.handleInterfaceActionClick({
    player, buttonId: core.MultiChatboxPrompt.OPTIONS_WIDGET_ID, groupId: 219, slot: number, action: number,
  }), true, `the ${label} choice was handled`);
}
function beginTalk(f) {
  const { player, npc } = f;
  assert.equal(PluginManager.emitNpcInteraction({ player, npc, npcId: npc.getId(),
    definition: npc.getDefinition(), action: "Talk-to", clickType: 1, handled: false }), true);
  const greeting = player.getDialogueManager().getCurrent();
  assert.match(greeting?.getText?.() ?? "", /Psst! Got any Void kit you don't want/);
  const menu = advanceUntilPrompt(player);
  assert.ok(menu?.options.includes("What if I do?"), `the actual greeting leads to the exchange choice: ${JSON.stringify(menu)}`);
  answer(player, "What if I do?");
}
function assertUnchanged(f) {
  assert.equal(Rewards.getPoints(f.player), 100);
  assert.equal(f.player.getInventory().getItems()[f.slot], f.item);
  assert.equal(f.item.getId(), f.itemId);
  assert.equal(f.item.getAmount(), 1);
}
function assertTopOffer(f) {
  const menu = advanceUntilPrompt(f.player);
  assert.ok(menu, "an eligible Void knight top produces an exchange offer");
  assert.equal(menu.title, "Void knight top: Exchange for 200 points?");
  assert.deepEqual(menu.options, ["Accept", "Decline"]);
  assertUnchanged(f);
}

test("Talk-to offers the eligible top and accepting exchanges exactly it for 200 points", (t) => {
  const f = fixture(t, core.ItemIdentifiers.VOID_KNIGHT_TOP);
  assert.equal(core.ItemDefinition.forId(f.itemId).getName().toLowerCase(), "void knight top");
  beginTalk(f);
  assertTopOffer(f); // Predicted first missing-behavior assertion; later assertions are unreached if it fails.
  answer(f.player, "Accept");
  advanceUntilPrompt(f.player);
  assert.equal(f.player.getInventory().getAmount(f.itemId), 0, "the offered item was removed");
  assert.equal(Rewards.getPoints(f.player), 300, "the listed refund was credited once");
});

test("declining an eligible offer preserves the same item and points", (t) => {
  const f = fixture(t, core.ItemIdentifiers.VOID_KNIGHT_TOP);
  beginTalk(f);
  assertTopOffer(f);
  answer(f.player, "Decline");
  advanceUntilPrompt(f.player);
  assertUnchanged(f);
});

test("a Void seal alone is not eligible for exchange", (t) => {
  const f = fixture(t, core.ItemIdentifiers.VOID_SEAL_8_);
  assert.match(core.ItemDefinition.forId(f.itemId).getName(), /void seal/i);
  beginTalk(f);
  const menu = advanceUntilPrompt(f.player);
  assert.equal(menu, null, `no refund offer for a seal: ${JSON.stringify(menu)}`);
  assertUnchanged(f);
});
