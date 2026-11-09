// Draft, unrun: after `yarn build`, node --test tests/defender-elias-dismissal-native.test.cjs.
// Stage/keys are seeded; NPC talk uses PM directly, not an NPC option packet.
// Save is JSON hydration, not a filesystem persistence round trip. Stale Yes is synthetic.
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after, test } = require('node:test');
process.env.DISABLE_PLAYER_BOTS = '1';
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager: PM } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');
const { ClientConnection } = require('../dist/net/NetworkBuilder');
const { MultiChatboxPrompt } = require('../dist/game/model/menu/MultiChatboxPrompt');
const Runtime = require('../plugins/quests/QuestRuntime');
const { PathFinder } = require('../dist/game/model/movement/path/PathFinder');
let core, quest, nextPlayer = 0;
const objectDispatches = []; let originalObjectDispatch;
const { PlayerSave } = require('../dist/game/entity/impl/player/persistence/PlayerSave');
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins'));
  core = PM.getCoreApi();
  originalObjectDispatch=PM.emitObjectInteraction.bind(PM);
  PM.emitObjectInteraction=(event)=>{ const handled=originalObjectDispatch(event); objectDispatches.push({player:event.player,id:event.objectId,option:event.option,handled}); return handled; };
  core.RegionManager.init(); core.RegionManager.loadMapFiles(3343,3515); core.RegionManager.loadMapFiles(3283,3501);
  PM.emitServerStartup({});
  quest = Runtime.getRegisteredQuests().findLast(q => q.key === 'defender_of_varrock');
  assert.ok(quest);
});
after(() => { try { PM.emitServerShutdown({}); } finally { PM.emitObjectInteraction=originalObjectDispatch; } });
function makePlayer(t, requestedName) {
  const channel = {kind:'test',remoteAddress:'127.0.0.1',isOpen:()=>true,send:()=>true,close(){},onData(){},onError(){},onClose(){}};
  const p = new Player(new PlayerSession(channel));
  const username = requestedName ?? `dovdismiss${nextPlayer++}`; p.setUsername(username); p.setLongUsername(core.Misc.stringToLongBigInt(username));
  p.setAttribute('bot-skip-persistence', true);
  assert.equal(core.World.getPlayers().add(p), true);
  const text=[]; const sender=p.getPacketSender(), original=sender.sendString.bind(sender);
  sender.sendString=(value,id,...rest)=>{ text.push(String(value)); return original(value,id,...rest); };
  const conn = new ClientConnection(channel); conn.player = p;
  p.getSession().setPlayer(p); p.getSession().setTurnPacketProcessor(() => conn.processTurnPackets());
  let retired=false;
  function retire() { if(retired) return; retired=true; PM.emitPlayerLogout({player:p,username:p.getUsername()}); p.closeInterruptibleInterfaces(); core.TaskManager.cancelTasks(p); core.TaskManager.cancelTasks(p.getIndex()); if (p.isRegistered()) core.World.getPlayers().remove(p); }
  t.after(retire);
  return {p,conn,text,retire};
}
async function advance(f) {
  const group = f.p.getPacketSender().getChatboxGroupId();
  const child = ({231:5,217:5,229:4})[group];
  assert.notEqual(child, undefined, `dialogue group ${group}`);
  await f.conn.dispatch([{type:'dialogue_continue',widgetId:(group<<16)|child,childIndex:-1}]); f.conn.processTurnPackets();
}
async function waitMenu(f, text) {
  for (let n=0;n<80;n++) {
    const pending=MultiChatboxPrompt.getPending(f.p);
    if (pending) { assert.ok(pending.options.includes(text), `${text}: ${pending.options}`); return pending; }
    assert.ok(f.p.getDialogueManager().isActive(), `dialogue ended before ${text}`); await advance(f);
  }
  assert.fail(`menu ${text} not reached`);
}
async function menu(f, text) {
  const pending=await waitMenu(f,text), index=pending.options.indexOf(text);
  await f.conn.dispatch([{type:'dialogue_continue',widgetId:MultiChatboxPrompt.OPTIONS_WIDGET_ID,childIndex:index+1}]); f.conn.processTurnPackets();
}
async function finish(f) {
  for(let n=0;n<80 && f.p.getDialogueManager().isActive();n++) { assert.equal(MultiChatboxPrompt.getPending(f.p),null); await advance(f); }
  assert.equal(f.p.getDialogueManager().isActive(),false);
}
function talk(f) {
  // Login creates the owner-only NPC; movement to the inn is seeded.
  const npc=[...core.World.getNpcs()].find(n=>n.getOwner?.()===f.p && n.getId()===core.NpcIdentifiers.ELIAS_WHITE);
  assert.ok(npc, 'quest login created owner Elias');
  f.p.moveTo(new core.Location(3283,3501,0));
  assert.equal(PM.emitNpcInteraction({player:f.p,npc,npcId:npc.getId(),action:'Talk-to',clickType:1,handled:false}),true);
}
async function trapdoor(f) {
  // CoA startup installs 50141 on the real parent tile; require the actual registered object
  // and native reach before sending the object option packet.
  const loc=new core.Location(3343,3515,0), id=50141;
  const object=core.World.findCacheObject(f.p,id,loc); assert.ok(object, 'installed trapdoor');
  const definition=core.ObjectDefinition.forPlayer(id,f.p); assert.equal(definition.getInteractions()[0],'Open');
  f.p.moveTo(new core.Location(3342,3515,0)); f.p.setNeedsPlacement(false);
  assert.equal(PathFinder.reachedObject(f.p,3343,3515,definition.getSizeX(),definition.getSizeY(),object.getFace(),10,definition.getBlockingMask()),true);
  objectDispatches.length=0; await f.conn.dispatch([{type:'object_option',id,x:3343,y:3515,clickType:1}]);
  for(let i=0;i<8;i++) { core.World.process(); f.conn.processTurnPackets(); }
  assert.ok(objectDispatches.some(event=>event.player===f.p && event.id===id && event.handled), 'installed loc reached registered handlers');
}
function where(f) { const p=f.p.getLocation(); return [p.getX(),p.getY(),p.getZ()]; }
for (const holder of ['player','elias']) test(`dismissed Elias: ${holder} unlocks; fetch required after logout`, async t => {
  const f=makePlayer(t), p=f.p;
  quest.setStage(p,10); p.setAttribute('defender-of-varrock:schema',3);
  for(const bit of [9659,9660,9661,9662,9663,9664]) p.setAttribute(`quest.defender_of_varrock.trail.${bit}`,1);
  p.setAttribute('quest.defender_of_varrock.key',holder);
  if(holder==='player') p.getInventory().adds(core.ItemIdentifiers.GRUBBY_KEY_4,1);
  PM.emitPlayerLogin({player:p,username:p.getUsername(),isNewAccount:false});
  await trapdoor(f); await menu(f,"I'm not ready yet."); await finish(f);
  assert.equal(quest.getStage(p),12); assert.ok((Number(p.getAttribute('quest.defender_of_varrock.bits')) & (1<<7))!==0);
  if(holder==='player') assert.equal(p.getInventory().getAmount(core.ItemIdentifiers.GRUBBY_KEY_4),0);
  // An unrelated later choice with the same text must not reuse the old quest context.
  talk(f); await menu(f,"Let's get going."); await finish(f);
  PM.emitCustomEvent('npc-dialogue:choice',{player:p,npcId:1,pages:[{page:'Unrelated',variants:['other']}],option:'I need to do something else.'});
  await trapdoor(f); await menu(f,"I'm not ready yet."); await finish(f);
  talk(f); await menu(f,'More options...'); await menu(f,'I need to do something else.'); await finish(f);
  f.text.length=0; await trapdoor(f); assert.deepEqual(where(f),[3342,3515,0]); assert.equal(quest.getStage(p),12);
  assert.ok(f.text.some(x=>x.includes('I should probably fetch Elias first.'))); assert.equal(MultiChatboxPrompt.getPending(f.p),null); await finish(f);
  const persistence=new (require('../plugins/persistence/JsonPlayerPersistence.plugin').JsonPlayerPersistence)();
  const raw=JSON.stringify(PlayerSave.fromPlayer(p),persistence.replacer.bind(persistence));
  const parsed=JSON.parse(raw,persistence.reviver.bind(persistence));
  assert.equal(parsed.attributes['defender-of-varrock:elias-dismissed'],1,'dismissal is registered for persistence');
  const saved=persistence.hydratePlayerSave(parsed), username=p.getUsername(); f.retire();
  const g=makePlayer(t,username); saved.applyToPlayer(g.p); PM.emitPlayerLogin({player:g.p,username:g.p.getUsername(),isNewAccount:false});
  g.text.length=0; await trapdoor(g); assert.deepEqual(where(g),[3342,3515,0]); assert.equal(quest.getStage(g.p),12);
  assert.ok(g.text.some(x=>x.includes('I should probably fetch Elias first.'))); assert.equal(MultiChatboxPrompt.getPending(g.p),null); await finish(g);
  // The rejoin acceptance itself leads to a nested menu; dismissing there must
  // restore absence even after the follow action has fired.
  talk(g); await menu(g,'Ready when you are.'); await menu(g,'More...'); await menu(g,'I need to do something else.'); await finish(g);
  g.text.length=0; await trapdoor(g); assert.deepEqual(where(g),[3342,3515,0]);
  assert.ok(g.text.some(x=>x.includes('I should probably fetch Elias first.'))); assert.equal(MultiChatboxPrompt.getPending(g.p),null); await finish(g);
  talk(g); await menu(g,'Ready when you are.'); await menu(g,"Let's get going"); await finish(g);
  await trapdoor(g); await menu(g,"I'm not ready yet."); await finish(g);
  assert.deepEqual(where(g),[3342,3515,0]); assert.equal(quest.getStage(g.p),12);
  // Synthetic interrupted prompts: a stale confirmation cannot use a remaining key
  // to bypass the six clues or a lost unlock flag.
  for (const field of ['trail','unlock','stage']) {
    await trapdoor(g); await waitMenu(g,"Let's do it.");
    if(field==='trail') g.p.setAttribute('quest.defender_of_varrock.trail.9659',0);
    else if(field==='unlock') g.p.setAttribute('quest.defender_of_varrock.bits',0);
    else quest.setStage(g.p,10);
    await menu(g,"Let's do it."); assert.deepEqual(where(g),[3342,3515,0]); assert.equal(quest.getStage(g.p),field==='stage'?10:12);
    await finish(g);
    if(field==='trail') g.p.setAttribute('quest.defender_of_varrock.trail.9659',1);
    else if(field==='unlock') g.p.setAttribute('quest.defender_of_varrock.bits',1<<7);
    else quest.setStage(g.p,12);
  }
  // Simulate an intervening state change after the prompt was drawn; choosing the
  // stale Yes must re-check the producer's current absence state.
  await trapdoor(g); await waitMenu(g,"Let's do it.");
  g.p.setAttribute('defender-of-varrock:elias-dismissed',1);
  await menu(g,"Let's do it."); assert.deepEqual(where(g),[3342,3515,0]); assert.equal(quest.getStage(g.p),12);
  await finish(g);
  talk(g); await menu(g,'Ready when you are.'); await menu(g,"Let's get going"); await finish(g);
  g.text.length=0; await trapdoor(g); await menu(g,"Let's do it."); assert.deepEqual(where(g),[3560,4551,0]); assert.equal(quest.getStage(g.p),14);
  await finish(g); for(let i=0;i<5;i++) core.TaskManager.process();
  assert.ok(g.text.some(x=>x.includes('Fascinating... What is this place?')), 'base dialogue follows entry');
});

test('legacy stage 12 remains recoverable and a prior entrant can return', async t => {
  const f=makePlayer(t), p=f.p;
  quest.setStage(p,12); p.setAttribute('defender-of-varrock:schema',3);
  p.setAttribute('quest.defender_of_varrock.bits',1<<7);
  for(const bit of [9659,9660,9661,9662,9663,9664]) p.setAttribute(`quest.defender_of_varrock.trail.${bit}`,1);
  PM.emitPlayerLogin({player:p,username:p.getUsername(),isNewAccount:false});
  assert.equal(p.getAttribute('defender-of-varrock:elias-dismissed') ?? 0,0);
  await trapdoor(f); await menu(f,"I'm not ready yet."); await finish(f);
  assert.deepEqual(where(f),[3342,3515,0]); assert.equal(quest.getStage(p),12);
  quest.setStage(p,14); p.setAttribute('quest.defender_of_varrock.bits',(1<<7)|1);
  await trapdoor(f); assert.deepEqual(where(f),[3560,4551,0]); assert.equal(quest.getStage(p),14);
});

test('interrupted first-unlock prompt ignores an unrelated transcript choice', async t => {
  const f=makePlayer(t), p=f.p;
  quest.setStage(p,10); p.setAttribute('defender-of-varrock:schema',3);
  for(const bit of [9659,9660,9661,9662,9663,9664]) p.setAttribute(`quest.defender_of_varrock.trail.${bit}`,1);
  p.setAttribute('quest.defender_of_varrock.key','player'); p.getInventory().adds(core.ItemIdentifiers.GRUBBY_KEY_4,1);
  PM.emitPlayerLogin({player:p,username:p.getUsername(),isNewAccount:false});
  await trapdoor(f); await waitMenu(f,"Let's do it.");
  assert.ok((Number(p.getAttribute('quest.defender_of_varrock.bits')) & (1<<7))!==0);
  // Start a real unrelated transcript, replacing the open quest prompt. This is
  // seeded dialogue playback, rather than an actual visit to Sedridor.
  assert.equal(Runtime.startTranscript(PM.createApi('DovCompetingDialogue'),p,core.NpcIdentifiers.ARCHMAGE_SEDRIDOR,
    'Temple of the Eye','enter-the-gate-talking-to-sedridor-or-persten'),true);
  await menu(f,"Let's do it."); assert.deepEqual(where(f),[3342,3515,0]); assert.equal(quest.getStage(p),12);
  p.getDialogueManager().reset();
  await trapdoor(f); await menu(f,"Let's do it."); assert.deepEqual(where(f),[3560,4551,0]); assert.equal(quest.getStage(p),14);
});
