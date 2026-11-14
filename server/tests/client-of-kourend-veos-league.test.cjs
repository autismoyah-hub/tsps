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
// ANSWERS TABLE (derived from source; each row is the answer the runtime's resolveCondition gives, then chooseCondition's pick).
// Sources: XMarksTheSpot answerCondition (XMarksTheSpot.Quest.js:322-340) answers [1] "members' world" only for VEOS 8484 and
// VEOS_POST_QUEST 10723; Routes.Ferries.js answerVeosMembership (:185-189) answers the F2P text [2] only for definition name
// "Veos" (true on F2P, false on members); no plugin answers [3] 7m-RoE or [4] 4_GYUT (grep of plugins/ and tests/).
// Unmatched text: combatLevelCondition and skillLevelCondition return null (NpcDialogues.plugin.js:455-458 and the skill helper),
// so resolveCondition is null for [1], [3], [4] when no plugin answers.
//
//   npc (world)                       [1] members/teleport   [2] F2P       [3] Trailblazer   [4] Reloaded   pick
//   8484 / 10723 plain members        true                   false         null              null          [1] sail
//   8484 / 10723 league (any)         true                   false         null              null          [1] sail (league not checked by [1])
//   8484 / 10723 F2P                  false                  true          null              null          [2] not eligible
//   10724 plain members               null                   false         null              null          [1] sail (first null)
//   10724 league (any)                null                   false         null              null          [1] sail (first null)
//   10724 F2P                         null                   true          null              null          [2] not eligible
//
// Consequence: the prohibited line (condition [3]) is reached only when no earlier condition is true and no earlier one is null,
// i.e. never in this table for a plain or league world. The table is the DERIVATION; the run decides (the prediction for 10724 on a
// plain world is 'sail, no prohibited line').
//
// WORLD STATE: league:type is read by the world check in plugins/modes/league/Common.LeagueMode.js (worldLeague(),
// api.getPluginConfig("league:type", 0)) and by the quest's api.getPluginConfig. The test stubs PluginManager.getPluginConfig
// for the key "league:type" only, and restores it in afterEach. Type values (plugins/modes/data/leagues.json): 2 = Leagues II
// Trailblazer, 4 = Leagues IV Trailblazer Reloaded. A plain world has no league:type (stub returns the fallback).
//
// DERIVATION FOR THE ARRIVAL SIDE (npc 10724 = VEOS_5, Piscarilius, 1825,3691). The condition run [1]..[4] above is the
// same run for every Veos npc id. For 10724 the answers are: [1] buVj4X: XMarks answers only for 8484/10723, so 10724 gets NO
// XMarks answer; the Ferries membership answer covers only the free-to-play text, so [1] is unanswered (null). resolveCondition
// (NpcDialogues.plugin.js:634-639) falls back to combatLevelCondition and skillLevelCondition (plugins/npcs/NpcDialogues.plugin.js:
// 455-458, and the skill helper's 'if (!match) return null'), which return null for this text, so [1] is null. [2] HaZALO:
// answerVeosMembership returns !isMembersWorld() on a plain members world: false. [3] 7m-RoE and [4] 4_GYUT: null.
// chooseCondition (NpcDialogues.plugin.js:479-486): no true answer, so the first null (index 1, the teleport/sail branch) wins.
// DERIVED PREDICTION (to be CONFIRMED by the run, not assumed): on a plain members world the prohibited line is NOT reached
// from 10724 by this run; the sail branch plays. On a league world [1] is still null, so the teleport branch still wins. If the
// run shows the prohibited line for 10724, the derivation is wrong and the run's lines decide.
// X MARKS STATE (seeded through the quest API only): 'starting-off' (X Marks complete, stage 6, quest key x_marks_the_spot,
// ClientOfKourend.Quest.js:61 and :423) versus X Marks NOT complete (stage 0). The reporter's state is taken as X Marks complete,
// quest not started (ClientOfKourend stage 0), because the Port Sarim->Piscarilius sail is offered after the first conversation.
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

let core, quest = null, xMarks = null, index = 9900;
const cleanups = [];
const restore = [];
const PROHIBITED = /access to Great Kourend is currently prohibited/;
const SAIL = /Certainly! I hope you don't get seasick/;
const TAKE_ME = "That's great, can you take me there please?";
const WHERE = "Where's that?";
const PORT_SARIM_TILE = [3054, 3245, 0];
const PISCARILIUS_TILE = [1825, 3691, 0];
const X_MARKS_KEY = "x_marks_the_spot";
const X_MARKS_COMPLETE_STAGE = 6;

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
  xMarks = runtime.getRegisteredQuests().find((q) => q.key === X_MARKS_KEY);
  assert.ok(xMarks, "XMarksTheSpot registers through the real plugin API");
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
function actor(npcId, tile, stage = 0, { xMarksComplete = false } = {}) {
  const channel = { kind: "test", remoteAddress: "127.0.0.1", isOpen: () => true, send: () => true,
    close() {}, onData() {}, onError() {}, onClose() {} };
  const p = new Player(new PlayerSession(channel), new Location(...tile));
  p.setIndex(index++); p.setRegistered(true); p.setUsername("kourend" + index);
  p.setAttribute("bot-skip-persistence", true);
  quest.setStage(p, stage); // seeded through the quest API only (disclosed)
  if (xMarksComplete) xMarks.setStage(p, X_MARKS_COMPLETE_STAGE); // X Marks completion through its own quest API (disclosed)
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

// ---- ARRIVAL SIDE: Piscarilius Veos (npc 10724 at 1825,3691) ----
const PISC = () => core.NpcIdentifiers.VEOS_5;

test("ARRIVAL, plain members world, X Marks NOT complete (selectVariant null -> the first-time variant): probe (report-only)", async () => {
  world({ league: null, members: true });
  const a = actor(PISC(), PISCARILIUS_TILE, 0, { xMarksComplete: false });
  talkTo(a);
  await a.choose(WHERE);
  console.log(`# A1 pisc-xmarks-not-complete opening: ${JSON.stringify(a.lines.slice(0, 4))}`);
});

test("ARRIVAL, plain members world, X Marks complete ('starting-off'): travel option, play to end (report-only rows)", async () => {
  world({ league: null, members: true });
  const a = actor(PISC(), PISCARILIUS_TILE, 0, { xMarksComplete: true });
  talkTo(a);
  await a.choose(WHERE);
  console.log(`# A2 pisc-xmarks-complete opening: ${JSON.stringify(a.lines.slice(0, 4))} options: ${JSON.stringify(MultiChatboxPrompt.getPending(a.p)?.options ?? null)}`);
});

test("ARRIVAL, REPORTER STATE, plain members world (hard): the dialogue runs, the travel option is offered, and no prohibited line follows", async () => {
  world({ league: null, members: true });
  const a = actor(PISC(), PISCARILIUS_TILE, 0, { xMarksComplete: true });
  talkTo(a);
  await a.choose(WHERE);
  assert.ok(a.lines.length > 0, `the dialogue ran: ${JSON.stringify(a.lines)}`);
  const offered = MultiChatboxPrompt.getPending(a.p)?.options ?? [];
  console.log(`# A3 options offered after "Where's that?": ${JSON.stringify(offered)}`);
  assert.ok(offered.includes(TAKE_ME), `the travel option must be offered for 10724 (otherwise the derivation's branch is not reached): offered ${JSON.stringify(offered)}`);
  await a.choose(TAKE_ME);
  await a.playToEnd();
  console.log(`# A3 reporter-state lines: ${JSON.stringify(a.lines)}`);
  console.log(`# A3 reporter-state messages: ${JSON.stringify(a.messages)}`);
  assert.equal(a.lines.some((l) => PROHIBITED.test(l)), false, `a plain members world must not print the prohibited line: ${JSON.stringify(a.lines)}`);
});
