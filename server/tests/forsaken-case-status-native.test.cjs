// Unrun draft: production object dispatch, dialogue continuation, and cache interface state.
// Stage and machine states are seeded; this does not prove the four puzzle producers.
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
const Runtime = require('../plugins/quests/QuestRuntime');
const { PathFinder } = require('../dist/game/model/movement/path/PathFinder');
const { PlayerSave } = require('../dist/game/entity/impl/player/persistence/PlayerSave');
const dispatches = []; let originalDispatch;
const replaceOnClose = new WeakSet();
const promptOnClose = new WeakSet();
const unlockOnClose = new WeakSet();
function openReplacementOnClose({player, interfaceId}) {
  if (interfaceId === 229 && replaceOnClose.delete(player)) player.getPacketSender().sendInterface(12);
  if (interfaceId === 229 && promptOnClose.delete(player)) { player.setEnteredAmountAction({}); player.getPacketSender().sendEnterAmountPrompt("Amount?"); }
  if (interfaceId === 229 && unlockOnClose.delete(player)) quest.setStage(player,8);
}
let core, quest, next = 0;
const PARENT = 34588, SEALED = 33487, X = 1381, Y = 3819, Z = 1;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins'));
  originalDispatch=PM.emitObjectInteraction.bind(PM);
  PM.emitObjectInteraction=(event)=>{const handled=originalDispatch(event);dispatches.push({player:event.player,object:event.object,id:event.objectId,option:event.option,handled});return handled;};
  core = PM.getCoreApi(); PM.createApi('ForsakenStatusNativeFixture').onCustomEvent('interface:closed',openReplacementOnClose);
  core.RegionManager.init(); core.RegionManager.loadMapFiles(X, Y);
  PM.emitServerStartup({});
  quest = Runtime.getRegisteredQuests().findLast(q => q.key === 'the_forsaken_tower');
  assert.ok(quest);
});
after(() => {try {PM.emitServerShutdown({});} finally {PM.emitObjectInteraction=originalDispatch;}});
function makePlayer(t, requestedName) {
  const channel = {kind:'test',remoteAddress:'127.0.0.1',isOpen:()=>true,send:()=>true,close(){},onData(){},onError(){},onClose(){}};
  const p = new Player(new PlayerSession(channel)); p.setUsername(requestedName ?? `case${next++}`); p.setLongUsername(core.Misc.stringToLongBigInt(p.getUsername()));
  p.setAttribute('bot-skip-persistence', true); assert.equal(core.World.getPlayers().add(p), true);
  const c = new ClientConnection(channel); c.player = p; p.getSession().setPlayer(p); p.getSession().setTurnPacketProcessor(() => c.processTurnPackets());
  const text=[]; const sender=p.getPacketSender(), sendString=sender.sendString.bind(sender);
  sender.sendString=(value,id,...rest)=>{ text.push({value:String(value),id}); return sendString(value,id,...rest); };
  let retired=false; function retire(){ if(retired)return; retired=true; PM.emitPlayerLogout({player:p,username:p.getUsername()}); p.closeInterruptibleInterfaces(); core.TaskManager.cancelTasks(p); core.TaskManager.cancelTasks(p.getIndex()); if(p.isRegistered())core.World.getPlayers().remove(p); }
  t.after(retire); return {p,c,text,retire};
}
async function tick(f, packet) { if(packet) await f.c.dispatch([packet]); for(let i=0;i<8;i++){core.World.process();f.c.processTurnPackets();} }
function seed(f) {
  quest.setStage(f.p,3); PM.emitPlayerLogin({player:f.p,username:f.p.getUsername(),isNewAccount:false});
}
async function inspect(f, packetId=SEALED) {
  const loc=new core.Location(X,Y,Z); const object=core.World.findCacheObject(f.p,PARENT,loc); assert.ok(object,'canonical display-case parent exists');
  assert.equal(core.ObjectDefinition.forPlayer(object.getId(),f.p)?.id,SEALED);
  f.p.moveTo(new core.Location(X-1,Y,Z)); f.p.setNeedsPlacement(false); f.text.length=0;
  const def=core.ObjectDefinition.forPlayer(object.getId(),f.p);
  const footprint=core.ObjectDefinition.forId(object.getId());
  assert.equal(PathFinder.reachedObject(f.p,X,Y,footprint.getSizeX(),footprint.getSizeY(),object.getFace(),object.getType(),def.getBlockingMask()),true);
  dispatches.length=0; await tick(f,{type:'object_option',id:packetId,x:X,y:Y,clickType:1});
  assert.ok(dispatches.some(e=>e.player===f.p && e.object===object && e.handled),'packet reached registered case object');
}
async function resume(f) { await tick(f,{type:'dialogue_continue',widgetId:(229<<16)|4,childIndex:-1}); }
test('Inspect shows text before the status interface; close and reopen are safe',async t=>{
  const f=makePlayer(t);seed(f);await inspect(f);
  assert.ok(f.text.some(x=>x.value.includes('The display case is sealed.')));
  assert.equal(f.p.getPacketSender().getChatboxGroupId(),229); assert.notEqual(f.p.getInterfaceId(),623);
  await resume(f); assert.equal(f.p.getPacketSender().getChatboxGroupId(),-1); assert.equal(f.p.getInterfaceId(),623);
  assert.equal(f.p.getDialogueManager().isActive(),false);
  f.p.closeInterruptibleInterfaces(); await resume(f); assert.notEqual(f.p.getInterfaceId(),623);
  await inspect(f,PARENT); await resume(f); assert.equal(f.p.getInterfaceId(),623);
});
test('replacement or movement cancels the pending status interface',async t=>{
  const f=makePlayer(t);seed(f);await inspect(f);f.p.closeInterruptibleInterfaces();await resume(f);assert.notEqual(f.p.getInterfaceId(),623);
  await inspect(f); await tick(f,{type:'move',worldX:X-1,worldY:Y,modifierFlags:0}); await resume(f); assert.notEqual(f.p.getInterfaceId(),623);
  await inspect(f);quest.setStage(f.p,8);await resume(f);assert.notEqual(f.p.getInterfaceId(),623);
});
test('login restores the four machine varbits for interface 623',async t=>{
  const f=makePlayer(t); quest.setStage(f.p,3);
  for(const [key,value] of [['electricity',4],['furnace',4],['refinery',4],['altar',0]]) f.p.setAttribute(`quest.the_forsaken_tower.${key}`,value);
  PM.emitPlayerLogin({player:f.p,username:f.p.getUsername(),isNewAccount:false});
  for(const [id,value] of [[7797,4],[7798,4],[7799,4],[7800,0]]) assert.equal(f.p.getPacketSender().getVarbit(id),value);
  await inspect(f);await resume(f);assert.equal(f.p.getInterfaceId(),623);
});
test('unlocked case Search still gives the hammer',async t=>{
  const f=makePlayer(t); quest.setStage(f.p,8); PM.emitPlayerLogin({player:f.p,username:f.p.getUsername(),isNewAccount:false});
  const loc=new core.Location(X,Y,Z), object=core.World.findCacheObject(f.p,PARENT,loc); assert.ok(object);
  assert.equal(core.ObjectDefinition.forPlayer(object.getId(),f.p)?.id,33488);
  f.p.moveTo(new core.Location(X-1,Y,Z));f.p.setNeedsPlacement(false);
  dispatches.length=0;await tick(f,{type:'object_option',id:33488,x:X,y:Y,clickType:1});
  assert.ok(dispatches.some(e=>e.player===f.p && e.object===object && e.option.toLowerCase()==='search' && e.handled));
  assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.DINHS_HAMMER),1);
  assert.equal(quest.getStage(f.p),9);assert.notEqual(f.p.getInterfaceId(),623);
});
test('a feature opened by the chatbox close hook keeps ownership',async t=>{
  const f=makePlayer(t);seed(f);await inspect(f);replaceOnClose.add(f.p);await resume(f);
  assert.equal(f.p.getInterfaceId(),12);assert.notEqual(f.p.getInterfaceId(),623);
});
test('a prompt armed by the chatbox close hook keeps ownership',async t=>{
  const f=makePlayer(t);seed(f);await inspect(f);promptOnClose.add(f.p);await resume(f);
  assert.equal(f.p.getPacketSender().hasInputPrompt(),true);assert.notEqual(f.p.getInterfaceId(),623);
});
test('machine state survives JSON save and player hydration',async t=>{
  const f=makePlayer(t);quest.setStage(f.p,3);
  f.p.setAttribute('quest.the_forsaken_tower.electricity',4);f.p.setAttribute('quest.the_forsaken_tower.furnace',4);
  PM.emitPlayerLogin({player:f.p,username:f.p.getUsername(),isNewAccount:false});
  const persistence=new (require('../plugins/persistence/JsonPlayerPersistence.plugin').JsonPlayerPersistence)();
  const raw=JSON.stringify(PlayerSave.fromPlayer(f.p),persistence.replacer.bind(persistence));
  const saved=persistence.hydratePlayerSave(JSON.parse(raw,persistence.reviver.bind(persistence))),name=f.p.getUsername();f.retire();
  const g=makePlayer(t,name);saved.applyToPlayer(g.p);PM.emitPlayerLogin({player:g.p,username:g.p.getUsername(),isNewAccount:false});
  assert.equal(g.p.getPacketSender().getVarbit(7797),4);assert.equal(g.p.getPacketSender().getVarbit(7798),4);
  await inspect(g);await resume(g);assert.equal(g.p.getInterfaceId(),623);
});
test('removing the inspected case before Continue does not open its status',async t=>{
  const f=makePlayer(t);seed(f);await inspect(f);
  const loc=new core.Location(X,Y,Z), object=core.World.findCacheObject(f.p,PARENT,loc);assert.ok(object);
  core.ObjectManager.deregister(object,true);
  try { await resume(f); assert.notEqual(f.p.getInterfaceId(),623); }
  finally { core.ObjectManager.register(object,true); }
});
test('case unlocking in a close hook prevents a stale status opening',async t=>{
  const f=makePlayer(t);seed(f);await inspect(f);unlockOnClose.add(f.p);await resume(f);
  assert.equal(quest.getStage(f.p),8);assert.notEqual(f.p.getInterfaceId(),623);
});
test('machine producer transitions update the status and unlock the case',async t=>{
  // Synthetic producer dispatches exercise the real registered hooks, not native map reach.
  const f=makePlayer(t);quest.setStage(f.p,3);
  for(const [key,value] of [['electricity',3],['furnace',3],['refinery',3],['altar',0],['flags',64],['pylons',14<<5],['held-disk',1]]) f.p.setAttribute(`quest.the_forsaken_tower.${key}`,value);
  f.p.getInventory().adds(core.ItemIdentifiers.TINDERBOX,1);
  PM.emitPlayerLogin({player:f.p,username:f.p.getUsername(),isNewAccount:false});
  for(const [bit,value] of [[7797,3],[7798,3],[7799,3],[7800,0],[7847,0],[7848,14],[7849,0]]) assert.equal(f.p.getPacketSender().getVarbit(bit),value);
  assert.equal(f.p.getAttribute('quest.the_forsaken_tower.held-disk'),1);
  const trigger=(id,child,option)=>{
    const location=new core.Location(X,Y,Z),object=new core.GameObject(id,location,10,0,null),definition=core.ObjectDefinition.forId(child);
    const clickType=definition.getInteractions().indexOf(option)+1; assert.ok(clickType>0, `${option} available`);
    assert.equal(PM.emitObjectInteraction({player:f.p,object,objectId:id,definition,location:{x:X,y:Y,z:Z},option,clickType,handled:false}),true);
  };
  for(const [id,child,option,bit,value] of [[33500,33500,'Align',7797,4],[33503,33503,'Light',7798,4],[34595,33518,'Activate',7799,4]]) {
    trigger(id,child,option);assert.equal(f.p.getPacketSender().getVarbit(bit),value);assert.equal(quest.getStage(f.p),3);
    await inspect(f);await resume(f);assert.equal(f.p.getInterfaceId(),623);f.p.closeInterruptibleInterfaces();
  }
  trigger(34599,33525,'Rebalance');assert.equal(f.p.getPacketSender().getVarbit(7800),2);assert.equal(f.p.getPacketSender().getVarbit(7848),15);
  assert.equal(f.p.getAttribute('quest.the_forsaken_tower.held-disk'),0);assert.equal(quest.getStage(f.p),8);
  const loc=new core.Location(X,Y,Z),object=core.World.findCacheObject(f.p,PARENT,loc);assert.ok(object);
  assert.equal(core.ObjectDefinition.forPlayer(object.getId(),f.p)?.id,33488);
  f.p.moveTo(new core.Location(X-1,Y,Z));f.p.setNeedsPlacement(false);dispatches.length=0;
  await tick(f,{type:'object_option',id:33488,x:X,y:Y,clickType:1});
  assert.ok(dispatches.some(e=>e.player===f.p && e.object===object && e.option.toLowerCase()==='search' && e.handled));
  assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.DINHS_HAMMER),1);assert.equal(quest.getStage(f.p),9);
});
test('logout while inspection text is pending cannot open the status',async t=>{
  const f=makePlayer(t);seed(f);await inspect(f);f.retire();
  assert.equal(f.p.isRegistered(),false);assert.equal(f.p.getDialogueManager().isActive(),false);
  await resume(f);assert.notEqual(f.p.getInterfaceId(),623);
});
