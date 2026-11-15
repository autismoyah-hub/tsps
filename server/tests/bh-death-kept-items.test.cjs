// Run after `yarn build`: node --test tests/bh-death-kept-items.test.cjs
// #5644 ask (a): "in the Bounty Hunter world I drop ALL items on death although not skulled". MEASUREMENT on the real engine
// (every plugin loaded, real victim and killer, the real PlayerDeathTask): what a death in Daimon's Crater keeps for an
// unskulled player with ordinary items, with preset-flagged items (what the crater's loadouts hand out), skulled, and with
// Protect Item, next to the same death outside the crater. BH-KEPT lines print the ids kept at each case.
const assert = require("node:assert/strict");
const path = require("node:path");
const { before, after, test } = require("node:test");
require("../dist/Server").Server.installProductionPathResolver();
const { CachePipeline } = require("../dist/game/cache/CachePipeline");
const { PluginManager: PM } = require("../dist/plugins/PluginManager");
const { Player } = require("../dist/game/entity/impl/player/Player");
const { PlayerSession } = require("../dist/net/PlayerSession");
const { PlayerDeathTask } = require("../dist/game/task/impl/PlayerDeath");

let core, I, next = 9930;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, ".."));
  PM.loadFromDirectory(path.resolve(__dirname, "../plugins"));
  core = PM.getCoreApi(); I = core.ItemIdentifiers;
  PM.emitServerStartup({});
});
after(() => PM.emitServerShutdown({}));

const CRATER = [3380, 4000, 0], OUTSIDE = [3100, 3600, 0];
function actor(t, at) {
  const channel = { kind: "test", remoteAddress: "127.0.0.1", isOpen: () => true, send: () => true, close() {}, onData() {}, onError() {}, onClose() {} };
  const p = new Player(new PlayerSession(channel));
  p.setIndex(next++); p.setUsername("bhdeath" + p.getIndex()); p.setRegistered(true);
  p.setAttribute("bot-skip-persistence", true);
  p.setLocation(new core.Location(...at)); p.needsPlacement = false;
  t.after(() => { p.setRegistered(false); core.TaskManager.cancelTasks(p); });
  return p;
}
const GEAR = () => [I.ABYSSAL_WHIP, I.DRAGON_SCIMITAR, I.RUNE_PLATEBODY, I.RUNE_PLATELEGS, I.SHARK];
function fill(p, preset) {
  GEAR().forEach((id, slot) => {
    const item = new core.Item(id, 1);
    if (preset) { item.setMetaValue(core.Item.UNTRADEABLE_META, true); item.setMetaValue(core.Item.UNBANKABLE_META, true); item.setMetaValue(core.Item.PRESET_META, true); }
    p.getInventory().setItem(slot, item);
  });
}
const keptIds = (p) => PlayerDeathTask.getItemsToKeep(p).map((item) => item.getId()).sort((a, b) => a - b);
const report = (label, p) => console.log(`BH-KEPT ${label} amountToKeep=${PlayerDeathTask.getAmountToKeep(p)} skulled=${p.getSkullTimer() > 0} kept=${JSON.stringify(keptIds(p))}`);

for (const [where, at] of [["in Daimon's Crater", CRATER], ["outside the crater (level-11 Wilderness)", OUTSIDE]]) {
  test(`${where}: an unskulled player with ordinary items keeps the three most valuable`, (t) => {
    const p = actor(t, at); fill(p, false);
    report(`${where} ordinary unskulled`, p);
    assert.equal(p.getSkullTimer() > 0, false, "not skulled");
    assert.deepEqual(keptIds(p), [I.ABYSSAL_WHIP, I.DRAGON_SCIMITAR, I.RUNE_PLATEBODY].sort((a, b) => a - b));
  });
  test(`${where}: preset-flagged items (the loadouts' gear) are never kept`, (t) => {
    const p = actor(t, at); fill(p, true);
    report(`${where} preset items`, p);
    assert.deepEqual(keptIds(p), []);
  });
  test(`${where}: skulled keeps nothing`, (t) => {
    const p = actor(t, at); fill(p, false);
    core.CombatFactory.skull(p, core.SkullType.WHITE_SKULL, core.CombatFactory.PVP_SKULL_SECONDS);
    report(`${where} skulled`, p);
    assert.deepEqual(keptIds(p), []);
  });
}
