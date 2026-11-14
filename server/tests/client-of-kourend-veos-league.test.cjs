// RSPSApp/tsps#5157 part 1 (test-only, unrun; for review before any job). Real runtime: NpcDialogues + the real
// ClientOfKourend quest (+ XMarksTheSpot, which answers the Veos membership/teleport condition), real Player, PlayerSession
// and ClientConnection. Run after `yarn build`, from server/: node --test tests/client-of-kourend-veos-league.test.cjs
//
// THE TREE (npc-dialogues.json, page "Veos"), read in order, variant standard-dialogue-at-port-sarim-first-time, option
// "That's great, can you take me there please?" (the condition run, in order):
//   [1] buVj4X  "If there are no restrictions to travelling and the player is on a members' world:"  -> sail line + action
//               teleport WXJllC (Kourend). Answered by XMarksTheSpot answerCondition ("members' world") when the speaker's
//               npc id is VEOS 8484 or 10723 (XMarks: VEOS_NPC_ID, VEOS_POST_QUEST_NPC_ID); otherwise unanswered (null).
//   [2] HaZALO  "If the player is on a free-to-play world:"  -> "not quite ready" + end. Answered by Ferries
//               answerVeosMembership (Routes.Ferries.js) for the Veos definition.
//   [3] 7m-RoE  "If the game world is Trailblazer League world:"  -> "access to Great Kourend is currently prohibited" + end.
//               No plugin answers it (grep of plugins/ and tests/).
//   [4] 4_GYUT  "If the game world is Trailblazer League Reloaded world:" -> "You cannot pass ... unlocked access". No answer.
//
// THE RUNTIME RULE (NpcDialogues.plugin.js:479-486, chooseCondition): the first condition answered TRUE wins; otherwise
// the first UNANSWERED (null) condition wins; a run answered all-false plays nothing. resolveCondition (NpcDialogues.plugin.js
// :634-639) asks the plugins first (emitNpcDialogueCondition), then the combat/skill fallbacks.
// CONSEQUENCE (derived, to be confirmed by the BEFORE run): a null answer on [1] makes [1] the chosen branch just as an answer
// true does, so [3] (prohibited) is reached only if [1] is answered FALSE and [2] is not true. On a plain members world [1]
// is true (XMarks, Veos ids) or null (other ids): either way the sail branch plays and the prohibited line should NOT.
// The league worlds are the interesting case: XMarks answers [1] true on any members world, so the restriction the tree
// names in [1] ("no restrictions to travelling") is not checked by any answer. This test records what the runtime does.
//
// WORLD STATE: league:type is read by the world check in plugins/modes/league/Common.LeagueMode.js (worldLeague(),
// api.getPluginConfig("league:type", 0)) and by the quest's api.getPluginConfig. The test stubs PluginManager.getPluginConfig
// for the key "league:type" only, and restores it in afterEach. Type values (plugins/modes/data/leagues.json): 2 = Leagues II
// Trailblazer, 4 = Leagues IV Trailblazer Reloaded. A plain world has no league:type (stub returns the fallback).
//
// ARRIVAL SIDE (the reporter's state): Veos at Port Sarim (npc 8484 at the dock; spawn 1063) -> the ferry leg to Port Piscarilius
// (1824,3695, the kourend-piscarilius route) -> Talk-to the Piscarilius Veos (npc 10724 at 1825,3691). This file seeds the
// quest stage through the quest API only (quest.setStage); X Marks completion is not seeded (its state is read as-is).
// Scope of the claim: the runtime answers and lines for each probe; the reporter's exact arrival chain is NOT replayed here.
//
// CONTROLS: harness proof (GREEN on base): Talk-to the Port Sarim Veos opens a dialogue and plays its first line.
// Plain members world, Port Sarim Veos: no prohibited line; the sail line plays. F2P world: the F2P branch answers ([2]).
// Trailblazer world (type 2) and Trailblazer Reloaded (type 4): the probe records the lines (the tree's own refusal text is
// the correct content on a league world; the fix must not silence it). Other NPC: an unrelated league condition's answer is
// unchanged (asserted via the hook directly with a non-Veos npc id).
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
const runtime = require("../plugins/quests/QuestRuntime");

runtime.startTranscript = () => true;

let core, quest = null, index = 9900;
const cleanups = [];
const restore = [];
const PROHIBITED = /access to Great Kourend is currently prohibited/;
const SAIL = /Certainly! I hope you don't get seasick/;
const TAKE_ME = "That's great, can you take me there please?";
const WHERE = "Where's that?";
const PORT_SARIM_TILE = [3054, 3245, 0];
const PISCARILIUS_TILE = [1825, 3691, 0];

before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, ".."));
  ObjectDefinition.init();
  core = PM.getCoreApi();
  require("../plugins/npcs/NpcDialogues.plugin").register(PM.createApi("KourendRuntimeDialogue"));
  require("../plugins/quests/XMarksTheSpot.Quest")(PM.createApi("KourendRuntimeXMarks"));
  const count = runtime.getRegisteredQuests().length;
  require("../plugins/quests/quests/ClientOfKourend.Quest")(PM.createApi("KourendRuntimeQuest"));
  quest = runtime.getRegisteredQuests().slice(count).at(-1);
  assert.ok(quest, "ClientOfKourend registers through the real plugin API");
});

afterEach(() => {
  while (restore.length) restore.pop()();
  while (cleanups.length) cleanups.pop()();
});

/** Stub world.json league:type only; restored in afterEach. The quest reads it through PluginManager.getPluginConfig. */
function world({ league = null, members = true } = {}) {
  const real = PM.getPluginConfig.bind(PM);
  const stub = (key, fallback) => (key === "league:type" && league !== null ? league : real(key, fallback));
  PM.getPluginConfig = stub;
  restore.push(() => { PM.getPluginConfig = real; });
  return { members };
}

/** A real player at a tile, a real Veos NPC of the given id, lines and messages recorded apart. */
function actor(npcId, tile, stage = 0) {
  const channel = { kind: "test", remoteAddress: "127.0.0.1", isOpen: () => true, send: () => true,
    close() {}, onData() {}, onError() {}, onClose() {} };
  const p = new Player(new PlayerSession(channel), new Location(...tile));
  p.setIndex(index++); p.setRegistered(true); p.setUsername("kourend" + index);
  p.setAttribute("bot-skip-persistence", true);
  quest.setStage(p, stage); // seeded through the quest API only (disclosed)
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

test("HARNESS PROOF (must be GREEN on base): Talk-to the Port Sarim Veos opens the dialogue and plays its first line", async () => {
  world();
  const a = actor(core.NpcIdentifiers.VEOS, PORT_SARIM_TILE);
  talkTo(a);
  await a.choose(WHERE);
  assert.ok(a.lines.length > 0, `the dialogue played lines: ${JSON.stringify(a.lines)}`);
  console.log(`# R0 port-sarim opening lines: ${JSON.stringify(a.lines.slice(0, 3))}`);
});

test("PLAIN MEMBERS WORLD, Port Sarim Veos: the sail branch plays and no prohibited line is shown (BEFORE: expected red if the prohibited line appears)", async () => {
  world({ league: null, members: true });
  const a = actor(core.NpcIdentifiers.VEOS, PORT_SARIM_TILE);
  talkTo(a);
  await a.choose(WHERE);
  await a.choose(TAKE_ME);
  await a.playToEnd();
  console.log(`# R1 plain-members lines: ${JSON.stringify(a.lines)}`);
  console.log(`# R1 plain-members messages: ${JSON.stringify(a.messages)}`);
  assert.equal(a.lines.some((l) => PROHIBITED.test(l)), false, `a plain members world must not print the prohibited line: ${JSON.stringify(a.lines)}`);
  assert.equal(a.lines.some((l) => SAIL.test(l)), true, `the sail branch must still play: ${JSON.stringify(a.lines)}`);
});

test("TRAILBLAZER LEAGUE WORLD (type 2), Port Sarim Veos: probe the branch (the tree's refusal text is correct content here; recorded, not blanket-asserted)", async () => {
  world({ league: 2, members: true });
  const a = actor(core.NpcIdentifiers.VEOS, PORT_SARIM_TILE);
  talkTo(a);
  await a.choose(WHERE);
  await a.choose(TAKE_ME);
  await a.playToEnd();
  console.log(`# R2 trailblazer-lines: ${JSON.stringify(a.lines)}`);
  console.log(`# R2 trailblazer-messages: ${JSON.stringify(a.messages)}`);
});

test("TRAILBLAZER RELOADED (type 4), Port Sarim Veos: probe the branch (recorded)", async () => {
  world({ league: 4, members: true });
  const a = actor(core.NpcIdentifiers.VEOS, PORT_SARIM_TILE);
  talkTo(a);
  await a.choose(WHERE);
  await a.choose(TAKE_ME);
  await a.playToEnd();
  console.log(`# R3 reloaded-lines: ${JSON.stringify(a.lines)}`);
});

test("F2P CONTROL: the free-to-play answer ([2] HaZALO) still answers as the base does (recorded; no sail line on F2P)", async () => {
  world({ league: null, members: false });
  const a = actor(core.NpcIdentifiers.VEOS, PORT_SARIM_TILE);
  talkTo(a);
  await a.choose(WHERE);
  await a.choose(TAKE_ME);
  await a.playToEnd();
  console.log(`# R4 f2p-lines: ${JSON.stringify(a.lines)}`);
  assert.equal(a.lines.some((l) => SAIL.test(l)), false, `F2P must not sail: ${JSON.stringify(a.lines)}`);
});

test("OTHER NPC UNCHANGED: a non-Veos league condition is answered exactly as on the base (asserted through the real hook)", () => {
  world({ league: 2, members: true });
  const probe = { player: actor(core.NpcIdentifiers.VEOS, PORT_SARIM_TILE).p, npc: null, npcId: 1, stepId: "7m-RoE", text: "If the game world is Trailblazer League world:" };
  const answer = PM.emitNpcDialogueCondition(probe);
  console.log(`# R5 other-npc 7m-RoE answer (npcId 1): ${JSON.stringify(answer)}`);
  assert.equal(answer, null, "a non-Veos npc gets no answer from the Veos-scoped quest answer");
});
