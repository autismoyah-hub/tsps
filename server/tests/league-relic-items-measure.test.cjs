// Run after `yarn build`: node --test tests/league-relic-items-measure.test.cjs
// #5586 'Raging Echoes relic audit' (league 5): MEASUREMENT of the item-granting relics on the real engine (every plugin
// loaded, a real league-5 character with points, Unlock.unlock = the plugin's own pick path): what each pick puts in the
// inventory/bank, and what the Leagues Tutor's Collect-items owes afterwards. RELIC-ITEMS lines print per relic.
const assert = require("node:assert/strict");
const path = require("node:path");
const { before, after, test } = require("node:test");
require("../dist/Server").Server.installProductionPathResolver();
const { CachePipeline } = require("../dist/game/cache/CachePipeline");
const { PluginManager: PM } = require("../dist/plugins/PluginManager");
const { Player } = require("../dist/game/entity/impl/player/Player");
const { PlayerSession } = require("../dist/net/PlayerSession");

let core, next = 9960;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, ".."));
  PM.loadFromDirectory(path.resolve(__dirname, "../plugins"));
  core = PM.getCoreApi();
  PM.emitServerStartup({});
});
after(() => PM.emitServerShutdown({}));

const data = require("../plugins/modes/data/league-relics.json");
const Unlock = require("../plugins/modes/leaguerelics/Unlock.LeagueRelics");
const Collect = require("../plugins/modes/leaguerelics/Collect.LeagueRelics");
const Common = require("../plugins/modes/leaguerelics/Common.LeagueRelics");

function leaguer(t) {
  const channel = { kind: "test", remoteAddress: "127.0.0.1", isOpen: () => true, send: () => true, close() {}, onData() {}, onError() {}, onClose() {} };
  const p = new Player(new PlayerSession(channel));
  p.setIndex(next++); p.setUsername("relic" + p.getIndex()); p.setRegistered(true);
  p.setAttribute("bot-skip-persistence", true);
  p.setAttribute("league:type", 5);
  p.setLocation(new core.Location(3222, 3218, 0)); p.needsPlacement = false;
  const messages = [];
  p.sendMessage = (m) => { messages.push(m); };
  t.after(() => { p.setRegistered(false); core.TaskManager.cancelTasks(p); });
  PM.emitCustomEvent("league:award-task", { player: p, id: "measure", points: 1_000_000, name: "measure" });
  return { p, messages };
}
const ids = (p) => p.getInventory().getItems().filter((i) => i && i.getId() > 0).map((i) => i.getId());
const banked = (p) => { const out = []; for (let tab = 0; tab < 10; tab++) { const b = p.getBank(tab); if (b) for (const i of b.getItems?.() ?? []) if (i && i.getId() > 0 && i.getAmount() > 0) out.push(i.getId()); } return out; };

test("league 5 is a league character with points in this harness", (t) => {
  const { p } = leaguer(t);
  assert.equal(Common.leagueOf(p), 5);
  console.log(`RELIC-ITEMS points=${Common.points(p)} cacheLeague(5)=${JSON.stringify(Common.cacheLeague(5) ? Object.keys(Common.cacheLeague(5)) : null)} tutor=${JSON.stringify(Common.cacheLeague(5)?.tutor ?? null)}`);
  assert.ok(Common.points(p) >= 1_000_000);
});

const L5 = data.leagues["5"];
L5.tiers.forEach((tier, tierIndex) => tier.relics.forEach((relic, slotIndex) => {
  const granted = Unlock.grantedItems(relic);
  if (granted.length === 0) return;
  test(`league 5 ${relic.key} (tier ${tierIndex}, slot ${slotIndex + 1}) grants ${relic.grants.join(", ")}`, (t) => {
    const { p, messages } = leaguer(t);
    const ok = Unlock.unlock(p, tierIndex, slotIndex + 1, { bankOpen: true });
    const inv = ids(p), bank = banked(p);
    const owedAfter = Collect.owed(p, 5);
    console.log(`RELIC-ITEMS ${relic.key} unlock=${ok} wanted=${JSON.stringify(granted)} inventory=${JSON.stringify(inv)} bank=${JSON.stringify(bank)} owedByTutor=${JSON.stringify(owedAfter)} msgs=${JSON.stringify(messages.slice(-2))}`);
    assert.equal(ok, true, `unlock: ${messages.join(" | ")}`);
    for (const id of granted) assert.ok(inv.includes(id) || bank.includes(id), `item ${id} delivered`);
  });
}));
