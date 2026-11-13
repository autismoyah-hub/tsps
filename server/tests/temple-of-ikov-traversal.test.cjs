// Additional #4834 native traversal/stock/physical-producer controls, after yarn build.
// Separate from the immutable16-case BEFORE002 comparison; supplied setup is explicit.
// Real cache, production plugins, World and decoded client inputs. Positions, supplies and
// quest checkpoints are declared setup: these cases do not claim a natural playthrough or rendering.
process.env.DISABLE_PLAYER_BOTS = '1';
const assert = require('node:assert/strict'), path = require('node:path');
const { before, after, test } = require('node:test');
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { NpcDefinitionLoader } = require('../dist/game/definition/loader/impl/NpcDefinitionLoader');
const { PluginManager: PM } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');
const { ClientConnection } = require('../dist/net/NetworkBuilder');
const { MultiChatboxPrompt } = require('../dist/game/model/menu/MultiChatboxPrompt');
const { getRegisteredQuests } = require('../plugins/quests/QuestRuntime');
const ClimbLinks = require('../plugins/objects/ClimbLinks');
let core, quest, serial = 0, originalSubmit, originalWarn, originalError, originalLog;
const tasks = new Set(), errors = [];
before(async () => {
  originalWarn = console.warn; originalError = console.error; originalLog = console.log;
  console.log = (...args) => {
    if (args.some(value => value instanceof Error)) errors.push(args.map(String).join(' '));
    originalLog(...args);
  };
  console.warn = (...args) => {
    if (/\[plugins\].*hook.*(?:threw|failed)/.test(args.map(String).join(' '))) errors.push(args.map(String).join(' '));
    originalWarn(...args);
  };
  console.error = (...args) => { errors.push(args.map(String).join(' ')); originalError(...args); };
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  new NpcDefinitionLoader().load();
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins')); core = PM.getCoreApi();
  originalSubmit = core.TaskManager.submit;
  core.TaskManager.submit = function(task, ...args) { tasks.add(task); return originalSubmit.call(this, task, ...args); };
  core.RegionManager.init();
  for (const [x,y] of [[2651,9805],[2639,9764],[2665,9855],[2710,9850],[2646,9870],[3122,3483]]) core.RegionManager.loadMapFiles(x,y);
  PM.emitServerStartup({});
  const { GroundItemSpawnDefinitionLoader } = require('../dist/game/definition/loader/impl/GroundItemSpawnDefinitionLoader');
  assert.equal(new GroundItemSpawnDefinitionLoader().load(),true,'canonical ground-spawn loader');
  quest = getRegisteredQuests().findLast(q => q.key === 'temple_of_ikov'); assert.ok(quest);
});
after(() => {
  try {
    if (core) PM.emitServerShutdown({});
    for (const task of tasks) if (task.isRunning()) task.stop();
    assert.deepEqual(errors, [], 'no swallowed plugin/engine failures');
  } finally {
    console.warn = originalWarn; console.error = originalError; console.log = originalLog;
    if (core) core.TaskManager.submit = originalSubmit;
  }
});
function turns(n=1) { for (let i=0;i<n;i++) core.World.process(); }
const where = p => [p.getLocation().getX(),p.getLocation().getY(),p.getLocation().getZ()];
function object(id,x,y,player=null) {
  core.RegionManager.loadMapFiles(x,y);
  const found = player ? core.MapObjects.getPrivateArea(player,id,new core.Location(x,y,0))
    : (core.MapObjects.mapObjects.get(core.MapObjects.getHash(x,y,0)) ?? []).find(o => o.getId()===id);
  assert.ok(found, `actual map object ${id} at ${x},${y},0`); return found;
}
function templeStairs(id,player) {
  const matches=[];
  for(const list of core.MapObjects.mapObjects.values()) for(const candidate of list) {
    const at=candidate.getLocation();
    if(at.getZ()!==0||at.getX()<2630||at.getX()>2670||at.getY()<9758||at.getY()>9814)continue;
    if(core.ObjectDefinition.forPlayer(candidate.getId(),player)?.id===id)matches.push(candidate);
  }
  assert.equal(matches.length,1,`one actual visible stairs producer${id} in the loaded Temple of Ikov region`);
  return matches[0];
}
function actor(t,x,y,stage=10) {
  const channel = {kind:'test',remoteAddress:'127.0.0.1',isOpen:()=>true,send:()=>true,close(){},onData(){},onError(){},onClose(){}};
  const session = new PlayerSession(channel), p = new Player(session), c = new ClientConnection(channel);
  c.player=p; session.setPlayer(p); session.setTurnPacketProcessor(()=>c.processTurnPackets());
  p.setUsername(`ikovqa${++serial}`); p.setLongUsername(core.Misc.stringToLongBigInt(p.getUsername()));
  p.setAttribute('bot-skip-persistence',true); assert.equal(core.World.getPlayers().add(p),true);
  p.moveTo(new core.Location(x,y,0)); quest.setStage(p,stage);
  turns(4); assert.equal(p.isNeedsPlacement(),false); assert.equal(p.isRegistered(),true); assert.ok(p.getHitpoints()>0);
  const messages=[]; const send=p.sendMessage.bind(p); p.sendMessage=text=>{messages.push(text);return send(text);};
  const sender=p.getPacketSender(), sendString=sender.sendString.bind(sender);
  sender.sendString=(text,...args)=>{messages.push(String(text));return sendString(text,...args);};
  const spawned=[];
  const objectEvents=[],itemEvents=[],emitObject=PM.emitObjectInteraction.bind(PM),emitItem=PM.emitItemOnObject.bind(PM);
  t.mock.method(PM,'emitObjectInteraction',event=>{if(event.player===p)objectEvents.push(event);return emitObject(event);});
  t.mock.method(PM,'emitItemOnObject',event=>{if(event.player===p)itemEvents.push(event);return emitItem(event);});
  t.after(()=>{
    for (const npc of spawned) PM.removeNpc(npc);
    for (const item of [...core.World.getItems()]) if (item.getOwner()===p.getUsername()) {
      core.ItemOnGroundManager.deregister(item);
      const at=core.World.getItems().indexOf(item); if(at>=0)core.World.getItems().splice(at,1);
    }
    assert.equal(core.World.getItems().some(item=>item.getOwner()===p.getUsername()),false);
    const index=p.getIndex(); PM.emitPlayerDisconnect({player:p,username:p.getUsername()}); PM.emitPlayerLogout({player:p,username:p.getUsername()});
    p.closeInterruptibleInterfaces(); p.getMovementQueue().reset(); p.getMovementQueue().walkToReset();
    core.TaskManager.cancelTasks(p); core.TaskManager.cancelTasks(index); core.World.getPlayers().remove(p);
    assert.equal(p.isRegistered(),false); assert.equal(core.TaskManager.hasActiveTask(p),false); assert.equal(core.TaskManager.hasActiveTask(index),false);
  });
  async function input(packet,n=3) { await c.dispatch([packet]); turns(n); }
  async function click(id,ox,oy,action='Open',clickType=1,n=4) {
    const target=object(id,ox,oy,p),before=objectEvents.length;
    await input({type:'object_option',id,x:ox,y:oy,action,clickType},n);
    assert.ok(objectEvents.slice(before).some(event=>event.object===target&&event.definition?.id===id&&event.clickType===clickType),
      `ordinary input reached the actual registered object dispatcher for ${id}`);
  }
  function approach(target) {
    const at=target.getLocation(),def=core.ObjectDefinition.forPlayer(target.getId(),p);
    const tile=ClimbLinks.landingTile(target,{x:at.getX(),y:at.getY()-1},p.getPrivateArea());
    assert.ok(tile,'the existing cache reach helper finds a usable approach');
    p.moveTo(tile);turns(3);assert.equal(p.isNeedsPlacement(),false);assert.deepEqual(where(p),[tile.getX(),tile.getY(),tile.getZ()]);
    assert.equal(core.PathFinder.reachedObject(p,at.getX(),at.getY(),def.getSizeX(),def.getSizeY(),target.getFace(),target.getType(),def.getBlockingMask()),true,
      'the settled actor actually reaches this rotated cache object before input');
  }
  function add(id,n=1) {const was=p.getInventory().getAmount(id);p.getInventory().adds(id,n);assert.equal(p.getInventory().getAmount(id),was+n);}
  async function talk(id,nx,ny) {
    const npc=PM.spawnNpc({id,x:nx,y:ny,z:0,owner:p,ownerOnly:true,wanderRadius:0}); assert.ok(npc);spawned.push(npc); turns(3);
    assert.equal(npc.isRegistered(),true);assert.ok(npc.getIndex()>=0);assert.equal(core.World.getNpcs().get(npc.getIndex()),npc);
    assert.equal(p.isNeedsPlacement(),false);assert.ok(p.getLocation().isWithinDistance(npc.getLocation(),3));
    assert.ok([...p.getLocalNpcs()].includes(npc),'the actual visible actor is available before Talk-to');
    const actions=core.NpcDefinition.forId(id).getActions(); const clickType=actions.indexOf('Talk-to')+1;assert.ok(clickType>0);
    await input({type:'npc_option',index:npc.getIndex(),clickType},4);return npc;
  }
  async function resume() {
    const group=p.getPacketSender().getChatboxGroupId(); assert.ok(group>=0,'real mounted source chatbox');
    await input({type:'dialogue_continue',widgetId:(group<<16)|(group===229?4:5),childIndex:-1},1);
  }
  return {p,c,messages,input,click,add,talk,resume,approach,itemEvents};
}
function settle(f,x,y,z=0){
  const at=new core.Location(x,y,z);assert.equal(core.RegionManager.blocked(at,f.p.getPrivateArea()),false,'supplied approach is a real unblocked tile');
  f.p.moveTo(at);turns(3);assert.equal(f.p.isNeedsPlacement(),false);assert.deepEqual(where(f.p),[x,y,z]);
}
function health(f){f.p.getSkillManager().setMaxLevel(core.Skill.HITPOINTS,99);f.p.getSkillManager().setCurrentLevel(core.Skill.HITPOINTS,99);assert.equal(f.p.getHitpoints(),99);}
function useLever(f){return f.input({type:'item_on_object',widgetId:3214,objectId:86,itemId:core.ItemIdentifiers.LEVER,slot:f.p.getInventory().getSlotForItemId(core.ItemIdentifiers.LEVER),x:2671,y:9804},4);}
test('the lit stairs return after the source is removed; extinguished light still refuses descent',async t=>{
  const f=actor(t,2653,9806),torch=core.ItemIdentifiers.LIT_TORCH;f.add(torch);
  const down=templeStairs(98,f.p);f.approach(down);await f.click(98,2650,9804,'Climb-down',1,8);
  assert.ok(f.p.getLocation().getY()<9770);f.p.getInventory().deleteNumber(torch,1);
  const up=templeStairs(96,f.p);f.approach(up);await f.click(96,2638,9763,'Climb-up',1,10);
  assert.ok(f.p.getLocation().getY()>9800,'return does not trap a player whose light has gone');
  f.add(core.ItemIdentifiers.TORCH);f.approach(down);await f.click(98,2650,9804,'Climb-down');
  assert.ok(f.p.getLocation().getY()>9800);assert.ok(f.messages.some(x=>x.includes("It's a bit dark down here!")));
});
test('fitting alone does not unlock the ice gate; Pull consumes no second lever and authorizes both directions',async t=>{
  const f=actor(t,2671,9805);f.add(core.ItemIdentifiers.LEVER,2);await useLever(f);assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.LEVER),1);
  await useLever(f);assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.LEVER),1,'stale base-bracket use cannot debit another lever');
  settle(f,2662,9802);await f.click(89,2662,9803,'Open');assert.equal(f.p.getLocation().getY(),9802,'fit without Pull stays locked');
  const fitted=object(87,2671,9804,f.p);f.approach(fitted);await f.click(87,2671,9804,'Pull');
  assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.LEVER),1);assert.ok(object(88,2671,9804,f.p));
  settle(f,2662,9802);await f.click(89,2662,9803,'Open',1,6);assert.equal(f.p.getLocation().getY(),9803);
  await f.click(89,2662,9803,'Open',1,6);assert.equal(f.p.getLocation().getY(),9802);
});
test('leaving removes an unpulled private lever; a pulled mechanism survives re-entry',async t=>{
  const f=actor(t,2671,9805);f.add(core.ItemIdentifiers.LEVER,2);await useLever(f);assert.ok(object(87,2671,9804,f.p));
  settle(f,2677,3405);settle(f,2671,9805);
  assert.equal(core.MapObjects.getPrivateArea(f.p,87,new core.Location(2671,9804,0)),null);
  await useLever(f);const fitted=object(87,2671,9804,f.p);f.approach(fitted);await f.click(87,2671,9804,'Pull');
  settle(f,2677,3405);settle(f,2671,9805);assert.ok(object(88,2671,9804,f.p));assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.LEVER),0);
});
test('an unauthorized bystander cannot follow through the ice gate while it is visibly open',async t=>{
  const a=actor(t,2671,9805),b=actor(t,2662,9802);a.add(core.ItemIdentifiers.LEVER);await useLever(a);
  const fitted=object(87,2671,9804,a.p);a.approach(fitted);await a.click(87,2671,9804,'Pull');settle(a,2662,9802);
  await a.click(89,2662,9803,'Open',1,1);
  assert.ok(core.MapObjects.get(38848,new core.Location(2662,9803,0),null),'positive open-scene blocker witness');
  await b.input({type:'move',worldX:2662,worldY:9804,modifierFlags:0,run:false},5);
  assert.ok(b.p.getLocation().getY()<9803,'ordinary independent follow movement remains clipped');
  assert.ok(a.p.getLocation().getY()>=9803,'authorized actor actually crossed the same boundary');
});
test('Search42 and Pull unlock the actual trap door without a fall and permit return',async t=>{
  const f=actor(t,2665,9854);health(f);f.p.getSkillManager().setMaxLevel(core.Skill.THIEVING,41);f.p.getSkillManager().setCurrentLevel(core.Skill.THIEVING,42);
  await f.click(91,2665,9855,'Search for traps',2);await f.click(91,2665,9855,'Pull',1,6);
  assert.equal(f.p.getHitpoints(),99);assert.equal(f.p.getAttribute('temple-of-ikov:trap-pulled'),1);
  settle(f,2648,9857);await f.click(92,2648,9857,'Open',1,6);assert.equal(f.p.getLocation().getY(),9858);
  await f.click(92,2648,9857,'Open',1,6);assert.equal(f.p.getLocation().getY(),9857);
});
test('an unsearched Pull deals20 through native damage, does not unlock, and the real pit ladder returns',async t=>{
  const f=actor(t,2665,9854);health(f);f.p.getSkillManager().setMaxLevel(core.Skill.THIEVING,41);f.p.getSkillManager().setCurrentLevel(core.Skill.THIEVING,41);
  await f.click(91,2665,9855,'Search for traps',2);assert.ok(f.messages.some(x=>x==='You find nothing.'));
  await f.click(91,2665,9855,'Pull',1,7);assert.equal(f.p.getHitpoints(),79);assert.notEqual(f.p.getAttribute('temple-of-ikov:trap-pulled'),1);
  assert.ok(f.p.getLocation().isWithinDistance(new core.Location(2682,9849,0),3),'actual resolved pit landing');
  const ladder=object(101,2682,9849,f.p);f.approach(ladder);await f.click(101,2682,9849,'Climb-up',1,9);
  assert.ok(f.p.getLocation().isWithinDistance(new core.Location(2665,9849,0),2),'cache-topology return is reachable');
});
test('a refused chest reward retains quantity and one-chest stock; later successful loot can select the same chest',async t=>{
  const f=actor(t,2710,9849),arrows=core.ItemIdentifiers.ICE_ARROWS,limit=0x7fffffff;
  f.p.getInventory().setItem(0,new core.Item(arrows,limit-2));
  const random=core.Misc.random.bind(core.Misc);let amounts=0,selections=0;
  // Only this source's two relevant domains, during the bounded interaction.
  // Other RNG calls retain their real implementation; this is not capture RNG.
  t.mock.method(core.Misc,'random',(min,max)=>min===0&&max===5?(selections++,0):min===1&&max===5?(++amounts===1?5:3):random(min,max));
  f.approach(object(103,2710,9850,f.p));await f.click(103,2710,9850,'Open');await f.click(104,2710,9850,'Search');
  assert.equal(f.p.getInventory().getAmount(arrows),limit-2);assert.ok(f.messages.some(x=>x.includes('not have enough space')));assert.equal(amounts,1);assert.equal(selections,1);
  f.p.getInventory().deleteNumber(arrows,10);await f.click(104,2710,9850,'Search');
  assert.equal(f.p.getInventory().getAmount(arrows),limit-7,'same sampled5 survives refused credit');assert.equal(amounts,1);assert.equal(selections,2);
  await f.click(104,2710,9850,'Search');assert.equal(f.p.getInventory().getAmount(arrows),limit-4,'same open chest may immediately hold the next3');
  assert.equal(amounts,2);assert.equal(selections,3);
  await f.click(104,2710,9850,'Shut',2);assert.ok(object(103,2710,9850,f.p));
});
test('Lucien house canonical door permits entry and exit on either branch',async t=>{
  const closed=object(102,3122,3488);
  t.after(()=>{
    for(const o of [...core.ObjectManager.objectsAt(new core.Location(3122,3487,0))])if(o.getId()===11403)core.ObjectManager.deregister(o,true);
    core.ObjectManager.register(closed,true);
  });
  for(const side of [1,2]){
    const f=actor(t,3122,3489,70);f.p.setAttribute('temple-of-ikov:state-version',1);f.p.setAttribute('temple-of-ikov:side',side);
    if(side===1)await f.click(102,3122,3488,'Open');
    else assert.ok(core.MapObjects.get(11403,new core.Location(3122,3487,0),null),'second branch uses the actual standing open door');
    await f.input({type:'move',worldX:3122,worldY:3486,modifierFlags:0,run:false},5);assert.ok(f.p.getLocation().getY()<3488);
    await f.input({type:'move',worldX:3122,worldY:3489,modifierFlags:0,run:false},5);assert.ok(f.p.getLocation().getY()>3488);
  }
});
