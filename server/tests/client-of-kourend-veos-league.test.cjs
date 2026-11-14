// RSPSApp/tsps#5157 part 1 LEAGUE WORLD (league:type 2, stubbed before load) (test-only; unrun; for review before any job). Run after `yarn build`, from server/:
//   node --test tests/client-of-kourend-veos-league.test.cjs
//
// PROCESS: DISABLE_PLAYER_BOTS=1 is set before the server modules load (as the #5289 test does); the full production plugin tree
// loads through PluginManager.loadFromDirectory after RegionManager.init (akd-lamps precedent: tests/akd-lamps.test.cjs:30-40),
// then emitServerStartup; emitServerShutdown runs in after().
// LEAGUE WORLD: league:type 2 (Leagues II Trailblazer, plugins/modes/data/leagues.json) is stubbed BEFORE PM.loadFromDirectory, so LeagueMode loads as a league world (type 4 is not run here: a separate file would be needed).
//
// REAL EVENT: the condition hooks are called with the event resolveCondition builds (NpcDialogues.plugin.js:633-637): player,
// npc (core.NPC.create at the tile), npcId, definition (npc.getCurrentDefinition(player), as emitNpcInteraction fills it,
// PluginManager.ts:1251), pages (the dialogue-index pages for the npc id with the Transcript: prefix stripped, from
// data/definitions/npc-dialogue-index.json; 1063 has no entry and is printed as such), text, stepId.
// Hooks that THROW are listed separately and never counted as answers.
//
// CONDITION RUN (the Veos Port Sarim first-time variant, option "That's great, can you take me there please?"):
//   [1] buVj4X members/teleport   [2] HaZALO free-to-play   [3] 7m-RoE Trailblazer league world (prohibited)   [4] 4_GYUT Reloaded
// chooseCondition (NpcDialogues.plugin.js:479-486): first true wins, else first null, else none.
// The derivation table in the earlier draft is NOT used: the probe rows decide (# PROBE lines).
//
// STUBS (restored in after() or afterEach): membership via WorldDefinition.isMembersWorld (WorldDefinition.ts:307; read by
// XMarksTheSpot.Quest.js:337-338 and Routes.Ferries.js:187) per test; league:type via PluginManager.getPluginConfig (read by
// api.getPluginConfig; LeagueMode loads its profile/hooks at plugin load, Common.LeagueMode.js:~39).
// Quests: runtime.getRegisteredQuests() by key client_of_kourend and x_marks_the_spot; x_marks completion is seeded with
// xMarks.setStage(6) (XMarksTheSpot.Quest.js:33); the Kourend stage with kourend.setStage (0 unless a test says otherwise).
//
// NPC ids: 1063 (Port Sarim spawn definition, npc-spawns.json), 8484 = NpcIdentifiers.VEOS, 10723 = VEOS_4, 10724 = VEOS_5
// (Piscarilius, 1825,3691).
//
// BASELINE ROW (C4): the OTHER-NPC probe (npcId 1) records the hook answers for the same league texts on the BEFORE run. It is
// report-only here; the AFTER run must assert equality with the recorded BEFORE values (not asserted in this BEFORE file).

const savedBots = process.env.DISABLE_PLAYER_BOTS;
process.env.DISABLE_PLAYER_BOTS = "1";
const assert = require("node:assert/strict");
const path = require("node:path");
const { before, after, afterEach, test } = require("node:test");
const { Server } = require("../dist/Server");
Server.installProductionPathResolver();
const { CachePipeline } = require("../dist/game/cache/CachePipeline");
const { PluginManager: PM } = require("../dist/plugins/PluginManager");
const { Player } = require("../dist/game/entity/impl/player/Player");
const { PlayerSession } = require("../dist/net/PlayerSession");
const { ClientConnection } = require("../dist/net/NetworkBuilder");
const { Location } = require("../dist/game/model/Location");
const { MultiChatboxPrompt } = require("../dist/game/model/menu/MultiChatboxPrompt");
const { ObjectDefinition } = require("../dist/game/definition/ObjectDefinition");
const WorldDefinition = require("../dist/game/definition/WorldDefinition");
const runtime = require("../plugins/quests/QuestRuntime");
const DIALOGUE_INDEX = require("../data/definitions/npc-dialogue-index.json");

runtime.startTranscript = () => true;

let core, kourend = null, xMarks = null, index = 9900;
const cleanups = [], restore = [];
const PROHIBITED = /access to Great Kourend is currently prohibited/;
const SAIL = /Certainly! I hope you don't get seasick/;
const TAKE_ME = "That's great, can you take me there please?";
const WHERE = "Where's that?";
const PORT_SARIM_TILE = [3054, 3245, 0];
const PISCARILIUS_TILE = [1825, 3691, 0];
const CONDITIONS = [
  { order: 1, id: "buVj4X", text: "If there are no restrictions to travelling and the player is on a members' world:" },
  { order: 2, id: "HaZALO", text: "If the player is on a free-to-play world:" },
  { order: 3, id: "7m-RoE", text: "If the game world is Trailblazer League world:" },
  { order: 4, id: "4_GYUT", text: "If the game world is Trailblazer League Reloaded world:" },
];

before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, ".."));
  // league:type stubbed BEFORE the plugin load (LeagueMode reads it at load: Common.LeagueMode.js:~39): a Trailblazer world (type 2).
  const realConfig = PM.getPluginConfig.bind(PM);
  PM.getPluginConfig = (key, fallback) => (key === "league:type" ? 2 : realConfig(key, fallback));
  PM.loadFromDirectory(path.resolve(__dirname, "../plugins"));
  core = PM.getCoreApi();
  core.RegionManager.init();
  core.RegionManager.loadMapFiles(3054, 3245);
  ObjectDefinition.init();
  PM.emitServerStartup({});
  kourend = runtime.getRegisteredQuests().find((q) => q.key === "client_of_kourend");
  xMarks = runtime.getRegisteredQuests().find((q) => q.key === "x_marks_the_spot");
  assert.ok(kourend && xMarks, "both Kourend quests are registered by the production plugin set");
});

after(() => {
  PM.emitServerShutdown({});
  if (savedBots === undefined) delete process.env.DISABLE_PLAYER_BOTS; else process.env.DISABLE_PLAYER_BOTS = savedBots;
});

afterEach(() => {
  while (restore.length) restore.pop()();
  while (cleanups.length) cleanups.pop()();
});

/** Stub membership for this test only (restored in afterEach). */
function membership(members) {
  const real = WorldDefinition.isMembersWorld;
  WorldDefinition.isMembersWorld = () => members;
  restore.push(() => { WorldDefinition.isMembersWorld = real; });
  const coreWorld = core.WorldDefinition;
  if (coreWorld && typeof coreWorld.isMembersWorld === "function") {
    const realCore = coreWorld.isMembersWorld;
    coreWorld.isMembersWorld = () => members;
    restore.push(() => { coreWorld.isMembersWorld = realCore; });
  }
}

/** A real player at a tile; quest stages seeded through the quest objects; chatbox lines and game messages recorded apart. */
function actor(npcId, tile, { kourendStage = 0, xMarksComplete = false } = {}) {
  const channel = { kind: "test", remoteAddress: "127.0.0.1", isOpen: () => true, send: () => true,
    close() {}, onData() {}, onError() {}, onClose() {} };
  const p = new Player(new PlayerSession(channel), new Location(...tile));
  p.setIndex(index++); p.setRegistered(true); p.setUsername("kourend" + index);
  p.setAttribute("bot-skip-persistence", true);
  kourend.setStage(p, kourendStage);
  if (xMarksComplete) xMarks.setStage(p, 6);
  const connection = new ClientConnection(channel);
  connection.player = p;
  const npc = core.NPC.create(npcId, new Location(...tile));
  const lines = [], messages = [];
  const sender = p.getPacketSender();
  const wrap = (name, record) => {
    const original = sender[name].bind(sender);
    sender[name] = (...args) => { record(...args); return original(...args); };
  };
  wrap("sendString", (text) => lines.push(String(text)));
  wrap("sendMessage", (text) => messages.push(String(text)));
  wrap("sendTypedMessage", (type, text) => messages.push(String(text)));
  cleanups.push(() => { p.closeInterruptibleInterfaces(); p.setRegistered(false); core.TaskManager.cancelTasks(p); });
  const send = async (packet) => { await connection.dispatch([packet]); connection.processTurnPackets(); };
  const next = () => send({ type: "dialogue_continue", widgetId: (p.getPacketSender().getChatboxGroupId() << 16) | 5, childIndex: -1 });
  async function choose(text) {
    for (let n = 0; !MultiChatboxPrompt.getPending(p) && n < 60; n++) {
      assert.ok(p.getDialogueManager().getCurrent(), `dialogue ended before "${text}"; lines: ${JSON.stringify(lines)}`);
      await next();
    }
    const prompt = MultiChatboxPrompt.getPending(p);
    assert.ok(prompt?.options.includes(text), `"${text}" offered: ${JSON.stringify(prompt?.options)}`);
    await send({ type: "dialogue_continue", widgetId: MultiChatboxPrompt.OPTIONS_WIDGET_ID, childIndex: prompt.options.indexOf(text) + 1 });
  }
  async function playToEnd(limit = 160) {
    for (let n = 0; n < limit && p.getDialogueManager().getCurrent(); n++) {
      if (MultiChatboxPrompt.getPending(p)) break;
      await next();
    }
  }
  return { p, npc, npcId, lines, messages, choose, playToEnd };
}

const talkTo = (a) => PM.emitNpcInteraction({ player: a.p, npc: a.npc, npcId: a.npcId, action: "Talk-to", clickType: 1, handled: false });

/** The event resolveCondition builds (NpcDialogues.plugin.js:633-637), with the definition and pages the runtime fills. */
function realEvent(player, npcId, tile, condition) {
  const npc = core.NPC.create(npcId, new Location(...tile));
  const definition = npc.getCurrentDefinition(player);
  const pages = (DIALOGUE_INDEX[String(npcId)] ?? []).map((entry) => ({ page: String(entry.page).replace(/^Transcript:/, ""), variants: entry.variants }));
  return { player, npc, npcId, definition, pages, text: condition.text, stepId: condition.id };
}

/**
 * Hook attribution: every registered condition hook is called with the real event; each non-null, non-undefined answer is
 * recorded with its plugin name, and each hook that throws is listed separately (never counted as an answer). The runtime's
 * emitNpcDialogueCondition answer and chooseCondition's pick are printed for the run's four conditions in order.
 */
function probe(player, npcId, tile, label) {
  const hooks = PM.npcDialogueConditionHooks ?? [];
  const hasIndex = DIALOGUE_INDEX[String(npcId)] !== undefined;
  const rows = CONDITIONS.map((c) => {
    const event = realEvent(player, npcId, tile, c);
    const answers = [], thrown = [];
    for (const hook of hooks) {
      try {
        const value = hook.handler(event);
        if (value !== undefined && value !== null) answers.push({ plugin: hook.pluginName, value });
      } catch (error) {
        thrown.push({ plugin: hook.pluginName, error: String(error?.message ?? error) });
      }
    }
    return { order: c.order, id: c.id, emitted: PM.emitNpcDialogueCondition(event), answers, thrown };
  });
  const emitted = rows.map((r) => r.emitted ?? null);
  let pick = emitted.findIndex((a) => a === true);
  if (pick === -1) pick = emitted.findIndex((a) => a === null);
  const pickedId = pick === -1 ? null : CONDITIONS[pick].id;
  console.log(`# PROBE ${label} npc=${npcId} indexEntry=${hasIndex} hooks=${hooks.length} ${JSON.stringify(rows.map((r) => ({ id: r.id, emitted: r.emitted, answersCount: r.answers.length, answers: r.answers, thrownCount: r.thrown.length, thrown: r.thrown.slice(0, 5) })))} pick=${pickedId}`);
  return { rows, pickedId };
}

test("HARNESS PROOF (GREEN on base): the production set loads and the Port Sarim Veos (8484) opens the dialogue and plays its first line", async () => {
  membership(true);
  const a = actor(core.NpcIdentifiers.VEOS, PORT_SARIM_TILE);
  talkTo(a);
  await a.choose(WHERE);
  assert.ok(a.lines.length > 0, `the dialogue ran: ${JSON.stringify(a.lines)}`);
  console.log(`# H0 port-sarim opening lines: ${JSON.stringify(a.lines.slice(0, 3))}`);
});

test("PROBE Trailblazer league world (type 2, stubbed BEFORE the plugin load): the hooks for 8484 and 10724", () => {
  membership(true);
  for (const [npcId, tile] of [[core.NpcIdentifiers.VEOS, PORT_SARIM_TILE], [core.NpcIdentifiers.VEOS_5, PISCARILIUS_TILE]]) {
    const a = actor(npcId, tile);
    probe(a.p, npcId, tile, "league-2");
  }
});

test("REAL DIALOGUE, league world (type 2), Port Sarim Veos: recorded (the tree's refusal survives only if the production hooks answer 7m-RoE; not asserted)", async () => {
  membership(true);
  const a = actor(core.NpcIdentifiers.VEOS, PORT_SARIM_TILE);
  talkTo(a);
  await a.choose(WHERE);
  await a.choose(TAKE_ME);
  await a.playToEnd();
  console.log(`# R5 league-2 port-sarim lines: ${JSON.stringify(a.lines)}`);
});

test("REAL DIALOGUE, league world (type 2), arrival Veos (10724), X Marks complete: recorded (report-only)", async () => {
  membership(true);
  const a = actor(core.NpcIdentifiers.VEOS_5, PISCARILIUS_TILE, { xMarksComplete: true });
  talkTo(a);
  await a.choose(WHERE);
  const offered = MultiChatboxPrompt.getPending(a.p)?.options ?? [];
  console.log(`# R6 league-2 arrival options: ${JSON.stringify(offered)}`);
  if (offered.includes(TAKE_ME)) { await a.choose(TAKE_ME); await a.playToEnd(); }
  console.log(`# R6 league-2 arrival lines: ${JSON.stringify(a.lines)}`);
});
