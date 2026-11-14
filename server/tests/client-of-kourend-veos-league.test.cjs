// RSPSApp/tsps#5157 part 1 (test-only; unrun; for review before any job). Run after `yarn build`, from server/:
//   node --test tests/client-of-kourend-veos-league.test.cjs
//
// PRODUCTION PLUGIN SET: the whole plugin tree is loaded with PluginManager.loadFromDirectory (precedent: tests/akd-lamps.test.cjs
// :33), then emitServerStartup, exactly as the production server does. The quest objects are taken from runtime.getRegisteredQuests()
// (keys: client_of_kourend, x_marks_the_spot). Nothing is required by hand.
//
// REAL RUNTIME: NpcDialogues answers the Talk-to condition run through PluginManager.emitNpcDialogueCondition (every registered
// hook), then the combat and skill fallbacks, then chooseCondition (plugins/npcs/NpcDialogues.plugin.js:479-486: first true wins,
// else first null, else none). The run is the Veos Port Sarim first-time variant, option "That's great, can you take me there
// please?", conditions in order: [1] buVj4X (members/teleport), [2] HaZALO (free-to-play), [3] 7m-RoE (Trailblazer league world:
// prohibited line), [4] 4_GYUT (Trailblazer Reloaded: "cannot pass").
//
// WORLD STATE (both stubbed per test, restored in afterEach):
//   - league: PluginManager.getPluginConfig("league:type"); the league world check reads it as api.getPluginConfig("league:type", 0)
//     (plugins/modes/league/Common.LeagueMode.js worldLeague()); types from plugins/modes/data/leagues.json: 2 = Leagues II Trailblazer,
//     4 = Leagues IV Trailblazer Reloaded.
//   - membership: WorldDefinition.isMembersWorld (src/main/typescript/elvarg/game/definition/WorldDefinition.ts:307), read by
//     XMarksTheSpot.Quest.js:337-338 and Routes.Ferries.js:187 (answerVeosMembership) and by core.WorldDefinition.
//
// HOOK ATTRIBUTION PROBE (the decisive datum): for each condition step of the run and each NPC id, every registered
// npc-dialogue condition hook is called directly (PluginManager.npcDialogueConditionHooks, PluginManager.ts:317-3520) and each
// non-null answer is printed with its pluginName. The chooseCondition pick is then computed from those answers.
//
// NPC ids: 1063 is the Port Sarim spawn definition id (npc-spawns.json), 8484 = NpcIdentifiers.VEOS, 10723 = VEOS_4,
// 10724 = VEOS_5 (Piscarilius, 1825,3691). The Talk-to rows use 8484 (Port Sarim) and 10724 (arrival).
//
// SEEDED STATE (through the quest objects only, disclosed): client_of_kourend stage (0 or as set by the test) and x_marks_the_spot
// stage 6 (complete, the X Marks completion stage: XMarksTheSpot.Quest.js:33) versus not complete.
const assert = require("node:assert/strict");
const path = require("node:path");
const { before, afterEach, test } = require("node:test");
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

runtime.startTranscript = () => true;

let core, kourend = null, xMarks = null, index = 9900;
const cleanups = [];
const restore = [];
const PROHIBITED = /access to Great Kourend is currently prohibited/;
const SAIL = /Certainly! I hope you don't get seasick/;
const TAKE_ME = "That's great, can you take me there please?";
const WHERE = "Where's that?";
const PORT_SARIM_TILE = [3054, 3245, 0];
const PISCARILIUS_TILE = [1825, 3691, 0];
const KOUREND_KEY = "client_of_kourend";
const X_MARKS_KEY = "x_marks_the_spot";
const X_MARKS_COMPLETE_STAGE = 6;
const CONDITIONS = [
  { order: 1, id: "buVj4X", text: "If there are no restrictions to travelling and the player is on a members' world:" },
  { order: 2, id: "HaZALO", text: "If the player is on a free-to-play world:" },
  { order: 3, id: "7m-RoE", text: "If the game world is Trailblazer League world:" },
  { order: 4, id: "4_GYUT", text: "If the game world is Trailblazer League Reloaded world:" },
];

before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, ".."));
  PM.loadFromDirectory(path.resolve(__dirname, "../plugins"));
  PM.emitServerStartup({});
  core = PM.getCoreApi();
  ObjectDefinition.init();
  kourend = runtime.getRegisteredQuests().find((q) => q.key === KOUREND_KEY);
  xMarks = runtime.getRegisteredQuests().find((q) => q.key === X_MARKS_KEY);
  assert.ok(kourend, "client_of_kourend is registered by the production plugin set");
  assert.ok(xMarks, "x_marks_the_spot is registered by the production plugin set");
});

afterEach(() => {
  while (restore.length) restore.pop()();
  while (cleanups.length) cleanups.pop()();
});

/** Stub the league type (league:type) and the membership flag for this test only. */
function world({ league = null, members = true } = {}) {
  const realConfig = PM.getPluginConfig.bind(PM);
  PM.getPluginConfig = (key, fallback) => (key === "league:type" && league !== null ? league : realConfig(key, fallback));
  restore.push(() => { PM.getPluginConfig = realConfig; });
  const realMembers = WorldDefinition.isMembersWorld;
  WorldDefinition.isMembersWorld = () => members;
  restore.push(() => { WorldDefinition.isMembersWorld = realMembers; });
  const coreWorld = core.WorldDefinition;
  if (coreWorld && typeof coreWorld.isMembersWorld === "function") {
    const realCore = coreWorld.isMembersWorld;
    coreWorld.isMembersWorld = () => members;
    restore.push(() => { coreWorld.isMembersWorld = realCore; });
  }
}

/** A real player at a tile; the quest stages seeded through the quest objects; lines and messages recorded apart. */
function actor(npcId, tile, { kourendStage = 0, xMarksComplete = false } = {}) {
  const channel = { kind: "test", remoteAddress: "127.0.0.1", isOpen: () => true, send: () => true,
    close() {}, onData() {}, onError() {}, onClose() {} };
  const p = new Player(new PlayerSession(channel), new Location(...tile));
  p.setIndex(index++); p.setRegistered(true); p.setUsername("kourend" + index);
  p.setAttribute("bot-skip-persistence", true);
  kourend.setStage(p, kourendStage);
  if (xMarksComplete) xMarks.setStage(p, X_MARKS_COMPLETE_STAGE);
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

/**
 * Hook attribution (the decisive datum): every registered condition hook is called directly for each condition step, and each
 * non-null answer is recorded with its plugin name; the runtime's pick is then computed as chooseCondition does (first true,
 * else first null). The emitNpcDialogueCondition answer and the combat/skill fallbacks are recorded as the runtime resolves them.
 */
function probeConditions(player, npcId, label) {
  const hooks = PM.npcDialogueConditionHooks ?? [];
  const rows = CONDITIONS.map((c) => {
    const event = { player, npc: null, npcId, definition: null, pages: [], text: c.text, stepId: c.id };
    const answers = hooks.map((hook) => {
      let value;
      try { value = hook.handler(event); } catch (error) { value = `threw ${error.message}`; }
      return { plugin: hook.pluginName, value };
    }).filter((a) => a.value !== undefined && a.value !== null);
    const emitted = PM.emitNpcDialogueCondition(event);
    return { order: c.order, id: c.id, emitted, answers };
  });
  const answersOnly = rows.map((r) => (r.emitted ?? null));
  let pick = answersOnly.findIndex((a) => a === true);
  if (pick === -1) pick = answersOnly.findIndex((a) => a === null);
  const pickedId = pick === -1 ? null : CONDITIONS[pick].id;
  console.log(`# PROBE ${label} npc=${npcId} ${JSON.stringify(rows.map((r) => ({ id: r.id, emitted: r.emitted, answers: r.answers })))} pick=${pickedId}`);
  return { rows, pickedId };
}

test("HARNESS PROOF (GREEN on base): the production set loads, the Port Sarim Veos (8484) opens the dialogue and plays its first line", async () => {
  world();
  const a = actor(core.NpcIdentifiers.VEOS, PORT_SARIM_TILE);
  talkTo(a);
  await a.choose(WHERE);
  assert.ok(a.lines.length > 0, `the dialogue ran: ${JSON.stringify(a.lines)}`);
  console.log(`# H0 port-sarim opening lines: ${JSON.stringify(a.lines.slice(0, 3))}`);
});

test("PROBE, plain members world: the condition hooks that answer each step for 1063, 8484, 10723 and 10724", () => {
  world({ league: null, members: true });
  for (const npcId of [1063, core.NpcIdentifiers.VEOS, core.NpcIdentifiers.VEOS_4, core.NpcIdentifiers.VEOS_5]) {
    const a = actor(npcId, PORT_SARIM_TILE);
    probeConditions(a.p, npcId, "plain-members");
  }
});

test("PROBE, Trailblazer (2) and Reloaded (4) worlds: the same probe with the league stubbed", () => {
  for (const league of [2, 4]) {
    world({ league, members: true });
    const a = actor(core.NpcIdentifiers.VEOS_5, PISCARILIUS_TILE);
    probeConditions(a.p, core.NpcIdentifiers.VEOS_5, `league-${league}`);
  }
});

test("PROBE, free-to-play world: the same probe with membership stubbed false", () => {
  world({ league: null, members: false });
  const a = actor(core.NpcIdentifiers.VEOS_5, PISCARILIUS_TILE);
  probeConditions(a.p, core.NpcIdentifiers.VEOS_5, "f2p");
});

test("REAL DIALOGUE, Port Sarim Veos (8484), plain members world (control): the sail branch plays", async () => {
  world({ league: null, members: true });
  const a = actor(core.NpcIdentifiers.VEOS, PORT_SARIM_TILE);
  talkTo(a);
  await a.choose(WHERE);
  await a.choose(TAKE_ME);
  await a.playToEnd();
  console.log(`# R1 port-sarim-plain lines: ${JSON.stringify(a.lines)}`);
  assert.equal(a.lines.some((l) => SAIL.test(l)), true, `the sail branch must play: ${JSON.stringify(a.lines)}`);
});

test("REAL DIALOGUE, arrival Veos (10724 at 1825,3691), plain members world, X Marks complete (the reporter's state): no prohibited line", async () => {
  world({ league: null, members: true });
  const a = actor(core.NpcIdentifiers.VEOS_5, PISCARILIUS_TILE, { xMarksComplete: true });
  talkTo(a);
  await a.choose(WHERE);
  assert.ok(a.lines.length > 0, `the dialogue ran: ${JSON.stringify(a.lines)}`);
  const offered = MultiChatboxPrompt.getPending(a.p)?.options ?? [];
  console.log(`# R2 arrival options: ${JSON.stringify(offered)}`);
  assert.ok(offered.includes(TAKE_ME), `the travel option must be offered for 10724: offered ${JSON.stringify(offered)}`);
  await a.choose(TAKE_ME);
  await a.playToEnd();
  console.log(`# R2 arrival lines: ${JSON.stringify(a.lines)}`);
  assert.equal(a.lines.some((l) => PROHIBITED.test(l)), false, `a plain members world must not print the prohibited line: ${JSON.stringify(a.lines)}`);
});

test("REAL DIALOGUE, arrival Veos (10724), plain members world, X Marks NOT complete: recorded (report-only)", async () => {
  world({ league: null, members: true });
  const a = actor(core.NpcIdentifiers.VEOS_5, PISCARILIUS_TILE, { xMarksComplete: false });
  talkTo(a);
  await a.choose(WHERE);
  const offered = MultiChatboxPrompt.getPending(a.p)?.options ?? [];
  console.log(`# R3 arrival (X Marks not complete) options: ${JSON.stringify(offered)}`);
  if (offered.includes(TAKE_ME)) { await a.choose(TAKE_ME); await a.playToEnd(); }
  console.log(`# R3 arrival (X Marks not complete) lines: ${JSON.stringify(a.lines)}`);
});

test("LEAGUE CONTROL (Trailblazer, type 2): the tree's refusal is correct content and must survive (recorded, Port Sarim)", async () => {
  world({ league: 2, members: true });
  const a = actor(core.NpcIdentifiers.VEOS, PORT_SARIM_TILE);
  talkTo(a);
  await a.choose(WHERE);
  await a.choose(TAKE_ME);
  await a.playToEnd();
  console.log(`# R4 trailblazer lines: ${JSON.stringify(a.lines)}`);
});

test("F2P CONTROL (membership stubbed false): the free-to-play branch answers; no sail line", async () => {
  world({ league: null, members: false });
  const a = actor(core.NpcIdentifiers.VEOS, PORT_SARIM_TILE);
  talkTo(a);
  await a.choose(WHERE);
  await a.choose(TAKE_ME);
  await a.playToEnd();
  console.log(`# R5 f2p lines: ${JSON.stringify(a.lines)}`);
  assert.equal(a.lines.some((l) => SAIL.test(l)), false, `F2P must not sail: ${JSON.stringify(a.lines)}`);
});

test("OTHER NPC UNCHANGED: a league condition of a different tree is answered by the same hooks as on the base (probe, recorded)", () => {
  world({ league: 2, members: true });
  const probe = { player: actor(core.NpcIdentifiers.VEOS, PORT_SARIM_TILE).p, npc: null, npcId: 1, stepId: "7m-RoE", text: CONDITIONS[2].text };
  const answer = PM.emitNpcDialogueCondition(probe);
  console.log(`# R6 other-npc 7m-RoE answer (npcId 1): ${JSON.stringify(answer)}`);
});
