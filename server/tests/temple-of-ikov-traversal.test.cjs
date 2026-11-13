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
const { CombatRange } = require('../dist/game/content/combat/CombatRange');
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
  const npcEvents=[],emitNpc=PM.emitNpcInteraction.bind(PM);
  t.mock.method(PM,'emitNpcInteraction',event=>{if(event.player===p)npcEvents.push(event);return emitNpc(event);});
  function speakerState(npc){return npc?{index:npc.getIndex(),id:npc.getContentId(p),at:where(npc),registered:npc.isRegistered(),visible:npc.isVisible(),local:p.getLocalNpcs().includes(npc),
    ownerMatches:!npc.getOwner()||npc.getOwner()===p,sameArea:npc.getPrivateArea()===p.getPrivateArea(),distance:p.getLocation().getDistance(npc.getLocation()),hp:npc.getHitpoints(),fightBack:npc.getDefinition().doesFightBack(),
    targetIndex:npc.getCombat().getTarget()?.getIndex()??null,targetIsPlayer:npc.getCombat().getTarget()===p}:null;}
  const watchedCombat=new Set(),combatEvents=[],actionNpcs=[];
  function watchCombat(npc){
    if(!npc||watchedCombat.has(npc))return;watchedCombat.add(npc);
    const combat=npc.getCombat();
    for(const method of['attack','reset','setTarget','cancelIfCurrent']){
      const original=combat[method];assert.equal(typeof original,'function','actual maintained combat method '+method);
      t.mock.method(combat,method,function(...args){
        const sample=phase=>{const row={phase,method,cycle:core.World.getProcessCycle(),player:where(p),npc:speakerState(npc),argumentIndex:args[0]?.getIndex?.()??null,
          argumentIsPlayer:args[0]===p,validTarget:core.CombatFactory.validTarget(npc,p),stack:new Error().stack.split('\n').slice(2,6)};
          combatEvents.push(row);console.log('IKOV_GUARDIAN_COMBAT',JSON.stringify(row));};
        sample('before');try{return Reflect.apply(original,this,args);}finally{sample('after');}
      });
    }
  }
  const permission=core.CombatFactory.canAttackPermission;
  t.mock.method(core.CombatFactory,'canAttackPermission',function(...args){
    const result=Reflect.apply(permission,this,args);
    if(watchedCombat.has(args[0])&&args[1]===p){const row={method:'canAttackPermission',cycle:core.World.getProcessCycle(),npc:speakerState(args[0]),response:result};combatEvents.push(row);console.log('IKOV_GUARDIAN_COMBAT',JSON.stringify(row));}
    return result;
  });
  const reach=CombatRange.canReach;
  t.mock.method(CombatRange,'canReach',function(...args){
    const result=Reflect.apply(reach,this,args);
    if(watchedCombat.has(args[0])&&args[2]===p){const row={method:'canReach',cycle:core.World.getProcessCycle(),npc:speakerState(args[0]),reached:result};combatEvents.push(row);console.log('IKOV_GUARDIAN_COMBAT',JSON.stringify(row));}
    return result;
  });
  const events=[],emitCustom=PM.emitCustomEvent.bind(PM);
  t.mock.method(PM,'emitCustomEvent',(name,event)=>{
    if(event?.player===p&&name==='npc-dialogue:action'){watchCombat(event.npc);actionNpcs.push({stepId:event.stepId,npc:event.npc});}
    const before=event?.player===p?speakerState(event.npc):null;
    const result=emitCustom(name,event);
    if(event?.player===p&&['ground-items:take','ground-items:taken','magic:can-telekinetic-grab','npc-dialogue:action','npc-dialogue:message'].includes(name)){
      const row={name,itemId:event.itemId??event.groundItem?.getItem()?.getId(),source:event.source,stepId:event.stepId,npcId:event.npcId,allow:event.allow,handled:event.handled,end:event.end,before,after:speakerState(event.npc)};
      events.push(row);console.log('IKOV_PRODUCER_EVENT',JSON.stringify(row));
    }
    return result;
  });
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
    const before=npcEvents.length;await input({type:'npc_option',index:npc.getIndex(),clickType},4);
    console.log('IKOV_GUARDIAN_TALK',JSON.stringify({player:where(p),npc:speakerState(npc),dispatched:npcEvents.slice(before).some(event=>event.npc===npc),mounted:!!p.getDialogueManager().getCurrent(),queue:p.getMovementQueue().size(),messages}));
    assert.ok(npcEvents.slice(before).some(event=>event.npc===npc),'setup: real supplied Guardian Talk-to reaches its native NPC dispatcher');return npc;
  }
  async function resume() {
    const group=p.getPacketSender().getChatboxGroupId(); assert.ok(group>=0,'real mounted source chatbox');
    await input({type:'dialogue_continue',widgetId:(group<<16)|(group===229?4:5),childIndex:-1},1);
  }
  return {p,c,messages,input,click,add,talk,resume,approach,itemEvents,events,combatEvents,actionNpcs,speakerState};
}
function settle(f,x,y,z=0){
  const at=new core.Location(x,y,z);assert.equal(core.RegionManager.blocked(at,f.p.getPrivateArea()),false,'supplied approach is a real unblocked tile');
  f.p.moveTo(at);turns(3);assert.equal(f.p.isNeedsPlacement(),false);assert.deepEqual(where(f.p),[x,y,z]);
}
function health(f){f.p.getSkillManager().setMaxLevel(core.Skill.HITPOINTS,99);f.p.getSkillManager().setCurrentLevel(core.Skill.HITPOINTS,99);assert.equal(f.p.getHitpoints(),99);}
function surfaceLanding(f){
  core.RegionManager.loadMapFiles(2677,3405);
  const objects=core.MapObjects.mapObjects.get(core.MapObjects.getHash(2677,3405,0))??[];
  console.log('IKOV_SURFACE_SETUP',JSON.stringify(objects.map(o=>({id:o.getId(),type:o.getType(),face:o.getFace(),at:where(o),name:core.ObjectDefinition.forPlayer(o.getId(),f.p)?.getName()}))));
  const ladders=objects.filter(o=>core.ObjectDefinition.forPlayer(o.getId(),f.p)?.getName()==='Ladder');
  assert.equal(ladders.length,1,'actual surface entrance ladder supplies an existing landing helper target');
  const tile=ClimbLinks.landingTile(ladders[0],{x:2678,y:3405},null);assert.ok(tile,'existing native helper finds an unblocked surface stand tile');
  assert.equal(tile.getZ(),0);return tile;
}
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
  const outside=surfaceLanding(f);settle(f,outside.getX(),outside.getY());settle(f,2671,9805);
  assert.equal(core.MapObjects.getPrivateArea(f.p,87,new core.Location(2671,9804,0)),null);
  await useLever(f);const fitted=object(87,2671,9804,f.p);f.approach(fitted);await f.click(87,2671,9804,'Pull');
  settle(f,outside.getX(),outside.getY());settle(f,2671,9805);assert.ok(object(88,2671,9804,f.p));assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.LEVER),0);
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

const STAFF=84;
function staffSpawn(){
  const matches=core.World.getItems().filter(item=>item.isStaticSpawn()&&item.getItem().getId()===STAFF&&!item.isPendingRemoval()&&item.getPosition().equals(new core.Location(2638,9906,0)));
  assert.equal(matches.length,1,'one actual canonical staff spawn');assert.equal(matches[0].getRespawnTimer(),100,'canonical importer timer is in native ticks');return matches[0];
}
function staffState(f,ground){
  const at=ground.getPosition();
  return{player:where(f.p),registered:f.p.isRegistered(),placement:f.p.isNeedsPlacement(),busy:f.p.busy(),queue:f.p.getMovementQueue().size(),
    ground:[at.getX(),at.getY(),at.getZ()],stackId:ground.getId(),stock:core.World.getItems().includes(ground),
    sameArea:ground.getPrivateArea()===f.p.getPrivateArea(),canSee:core.ItemOnGroundManager.canSee(f.p,ground),
    canOperate:core.ItemOnGroundManager.canOperate(f.p,ground,core.ItemOnGroundManager.takeOperation(STAFF)),
    reached:core.PathFinder.reachedObj(f.p,at),telegrabRange:CombatRange.withinApproachDistance(f.p,at,10),
    book:f.p.getSpellbook().getInterfaceId(),pickupReady:f.p.getLastItemPickup().elapsedTime(300),clickReady:f.p.getClickDelay().elapsedTime(500),messages:f.messages.slice(-8)};
}
function staffApproach(f){
  const ground=staffSpawn(),at=ground.getPosition();core.RegionManager.loadMapFiles(at.getX(),at.getY());
  const objects=core.MapObjects.mapObjects.get(core.MapObjects.getHash(at.getX(),at.getY(),at.getZ()))??[];
  console.log('IKOV_STAFF_SETUP',JSON.stringify({initial:staffState(f,ground),objects:objects.map(o=>({id:o.getId(),type:o.getType(),face:o.getFace(),at:where(o),solid:o.getDefinition().isSolid()}))}));
  const obstacle=objects.find(o=>o.getDefinition().isSolid());
  const tile=obstacle?ClimbLinks.landingTile(obstacle,{x:at.getX(),y:at.getY()-1},f.p.getPrivateArea()):at.clone();
  assert.ok(tile,'existing landing helper supplies a ground-item approach');settle(f,tile.getX(),tile.getY(),tile.getZ());
  const actual=staffState(f,ground);console.log('IKOV_STAFF_SETUP',JSON.stringify({settled:actual}));
  assert.ok(actual.sameArea&&actual.canSee&&actual.canOperate&&actual.reached&&!actual.busy,'setup: real visible operable ground stack is reached before Take');
  return ground;
}
function staffGuardianTile(f){
  const from=f.p.getLocation(),candidates=[];
  for(const [dx,dy]of[[0,1],[1,0],[-1,0],[0,-1]]){
    const at=from.clone().add(dx,dy),blocked=core.RegionManager.blocked(at,f.p.getPrivateArea()),connected=core.RegionManager.canMovestart(from,at,1,1,f.p.getPrivateArea());
    console.log('IKOV_STAFF_GUARDIAN_SETUP',JSON.stringify({at:[at.getX(),at.getY(),at.getZ()],blocked,connected,clip:core.RegionManager.getClipping(at.getX(),at.getY(),at.getZ(),f.p.getPrivateArea())}));
    if(!blocked&&connected)candidates.push(at);
  }
  assert.ok(candidates.length,'setup: actual unblocked same-bank adjacent Guardian tile is connected to the true pickup approach');return candidates[0];
}
async function takeStaff(f){
  const ground=staffSpawn();console.log('IKOV_STAFF_TAKE',JSON.stringify({when:'before',state:staffState(f,ground)}));
  assert.equal(f.p.getLastItemPickup().elapsedTime(300),true,'normal pickup delay has elapsed');
  await f.input({type:'ground_item_action',itemId:STAFF,x:2638,y:9906,option:'Take',stackId:ground.getId()},0);
  for(let turn=1;turn<=6;turn++){turns();console.log('IKOV_STAFF_TAKE',JSON.stringify({turn,state:staffState(f,ground)}));}
}
test('the real staff pickup requires its actual Guardian death, credits once and respawns through the native timer',async t=>{
  const f=actor(t,2638,9905,60);health(f);const ground=staffApproach(f);
  const guardianTile=staffGuardianTile(f),suppliedNpc=await f.talk(core.NpcIdentifiers.GUARDIAN_OF_ARMADYL,guardianTile.getX(),guardianTile.getY());f.p.closeInterruptibleInterfaces();staffApproach(f);
  assert.equal(core.RegionManager.canMovestart(f.p.getLocation(),suppliedNpc.getLocation(),1,1,f.p.getPrivateArea()),true,'setup: Guardian and true staff approach share an unblocked melee edge');
  assert.equal(core.TaskManager.hasActiveTask(f.p.getIndex()),false,'setup: no unfinished Talk-to route survives into the staff input');
  const candidates=f.p.getLocalNpcs().filter(n=>[core.NpcIdentifiers.GUARDIAN_OF_ARMADYL,core.NpcIdentifiers.GUARDIAN_OF_ARMADYL_2].includes(n.getContentId(f.p))).map(f.speakerState);
  console.log('IKOV_GUARDIAN_CANDIDATES',JSON.stringify({player:where(f.p),suppliedIndex:suppliedNpc.getIndex(),candidates}));
  await takeStaff(f);assert.equal(f.p.getInventory().getAmount(STAFF),0);assert.ok(core.World.getItems().includes(ground));
  for(let i=0;i<8&&!f.actionNpcs.some(e=>e.stepId==='zJVI_k')&&f.p.getDialogueManager().getCurrent();i++)await f.resume();
  const selected=f.actionNpcs.filter(e=>e.stepId==='zJVI_k');assert.equal(selected.length,1,'one actual source staff intervention chooses its speaker');const npc=selected[0].npc;
  assert.ok(npc&&npc.isRegistered()&&f.p.getLocalNpcs().includes(npc),'actual selected Guardian is registered and local');
  for(let i=0;i<8&&npc.getCombat().getTarget()!==f.p&&f.p.getDialogueManager().getCurrent();i++)await f.resume();
  console.log('IKOV_GUARDIAN_INTERVENTION',JSON.stringify({player:where(f.p),npc:where(npc),targetIndex:npc.getCombat().getTarget()?.getIndex()??null,events:f.events,combatEvents:f.combatEvents,messages:f.messages}));
  assert.ok(f.events.some(e=>e.name==='ground-items:take'),'ordinary Take reached the production physical permission hook');
  assert.ok(f.events.some(e=>e.name==='npc-dialogue:action'&&e.stepId==='zJVI_k'&&e.after?.index===npc.getIndex()),'actual source attack step binds the actual selected Guardian');
  assert.equal(npc.getCombat().getTarget()===f.p,true,'actual source Guardian intervention starts combat');
  const hp=npc.getHitpoints();npc.getCombat().addDamage(f.p,hp);npc.getCombat().getHitQueue().addPendingDamage([new core.HitDamage(hp,core.HitMask.RED)]);
  for(let i=0;i<24&&npc.isRegistered();i++)turns();assert.equal(npc.isRegistered(),false,'native NPCDeathTask runs the real Guardian death producer');
  // Capacity failure does not consume the earned death receipt or despawn stock.
  for(let i=0;i<f.p.getInventory().capacity();i++)f.p.getInventory().setItem(i,new core.Item(core.ItemIdentifiers.BRONZE_DAGGER,1));
  await takeStaff(f);assert.equal(f.p.getInventory().getAmount(STAFF),0);assert.ok(core.World.getItems().includes(ground));
  f.p.getInventory().deleteNumber(core.ItemIdentifiers.BRONZE_DAGGER,1);await takeStaff(f);
  assert.equal(f.p.getInventory().getAmount(STAFF),1);assert.equal(quest.getStage(f.p),70);assert.equal(core.World.getItems().includes(ground),false);
  turns(101);const respawn=staffSpawn();assert.notEqual(respawn,ground);
  // Preserve the prior full-bag/refused-first-credit control, but let this
  // separate duplicate attempt reach the quest permission hook before credit.
  assert.equal(f.p.getInventory().getFreeSlots(),0);f.p.getInventory().deleteNumber(core.ItemIdentifiers.BRONZE_DAGGER,1);
  assert.equal(f.p.getInventory().getAmount(STAFF),1);assert.equal(f.p.getInventory().getFreeSlots(),1);
  const capacity=core.ItemOnGroundManager.inventoryCapacity(f.p,respawn.getItem()),duplicateSetup=staffState(f,respawn);
  console.log('IKOV_STAFF_DUPLICATE_SETUP',JSON.stringify({capacity,staffHeld:f.p.getInventory().getAmount(STAFF),freeSlots:f.p.getInventory().getFreeSlots(),state:duplicateSetup}));
  assert.equal(capacity,1,'setup: second staff could physically fit before quest permission');
  assert.ok(duplicateSetup.stock&&duplicateSetup.sameArea&&duplicateSetup.canSee&&duplicateSetup.canOperate&&duplicateSetup.reached&&!duplicateSetup.busy,'setup: actual respawn is visible operable and reached');
  const eventStart=f.events.length,messageStart=f.messages.length;await takeStaff(f);
  const freshEvents=f.events.slice(eventStart),freshMessages=f.messages.slice(messageStart);
  console.log('IKOV_STAFF_DUPLICATE_RESULT',JSON.stringify({events:freshEvents,messages:freshMessages,staffHeld:f.p.getInventory().getAmount(STAFF),stock:core.World.getItems().includes(respawn)}));
  assert.ok(freshEvents.some(e=>e.name==='ground-items:take'&&e.itemId===STAFF&&e.source==='pickup'&&e.allow===false),'held-staff guard actually refuses the new physical permission attempt');
  assert.ok(freshMessages.includes('You already have the Staff of Armadyl.'),'fresh duplicate guard feedback');
  assert.equal(freshEvents.some(e=>e.name==='ground-items:taken'&&e.itemId===STAFF),false,'duplicate refusal never reaches a second successful transfer');
  assert.equal(f.p.getInventory().getAmount(STAFF),1,'second spawn cannot duplicate the held quest staff');assert.ok(core.World.getItems().includes(respawn));
});
test('the real spell-on-ground producer refuses staff Telekinetic Grab before spending runes or granting credit',async t=>{
  const f=actor(t,2638,9905,60);const ground=staffApproach(f);f.p.getSkillManager().setMaxLevel(core.Skill.MAGIC,99);f.p.getSkillManager().setCurrentLevel(core.Skill.MAGIC,99);
  f.add(core.ItemIdentifiers.LAW_RUNE);f.add(core.ItemIdentifiers.AIR_RUNE);
  const spell=core.CacheDefinitions.getSpellByName('Telekinetic Grab');assert.ok(spell);assert.equal(core.CacheDefinitions.getSpellName(spell.widgetId,spell.itemId),'Telekinetic Grab');
  const events=[],emit=PM.emitCustomEvent.bind(PM);t.mock.method(PM,'emitCustomEvent',(name,event)=>{const result=emit(name,event);if(name==='magic:can-telekinetic-grab'&&event.player===f.p)events.push(event);return result;});
  assert.equal(f.p.getClickDelay().elapsedTime(500),true);
  console.log('IKOV_TELEGRAB_SETUP',JSON.stringify({state:staffState(f,ground),spell}));
  assert.equal(f.p.getSpellbook().getInterfaceId(),core.MagicSpellbook.NORMAL.getInterfaceId());
  assert.equal(CombatRange.withinApproachDistance(f.p,ground.getPosition(),10),true,'setup: actual projectile approach distance and line before spell input');
  await f.input({type:'spell_on_ground',groundItemId:STAFF,x:2638,y:9906,spellWidget:spell.widgetId,spellChild:spell.widgetId&0xffff,spellItemId:spell.itemId},6);
  assert.equal(events.length,1,'the normal spell reached the actual staff permission producer');assert.equal(events[0].allow,false);
  assert.equal(f.p.getInventory().getAmount(STAFF),0);assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.LAW_RUNE),1);assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.AIR_RUNE),1);
  assert.ok(f.messages.some(s=>s.includes('cannot use Telekinetic Grab')));staffSpawn();
});
test('Guardian allegiance and completed quest each refuse the real static staff transfer',async t=>{
  const helper=actor(t,2638,9905,60);staffApproach(helper);helper.p.setAttribute('temple-of-ikov:side',1);
  await takeStaff(helper);assert.equal(helper.p.getInventory().getAmount(STAFF),0);assert.ok(helper.messages.some(s=>s.includes('decide not to steal the staff')));helper.p.closeInterruptibleInterfaces();
  const done=actor(t,2638,9905,70);staffApproach(done);assert.equal(quest.complete(done.p,{experience:false,scroll:false,announce:false}),true);await takeStaff(done);
  assert.equal(done.p.getInventory().getAmount(STAFF),0);assert.ok(done.messages.some(s=>s.includes('completed the quest of the Temple of Ikov')));staffSpawn();
});

// Positive traversal/source binding only. Heavy crossing's lower-route landing
// remains unresolved; this does not endorse the withdrawn bank-recovery draft.
function actualBridgeRow(){
  const candidates=[];
  for(const regionId of [(41<<8)|152,(41<<8)|153]){
    const region=core.getTemplateRegion(regionId);assert.ok(region,'already loaded Temple source region');
    for(const loc of region.chunks.flat()){
      const x=(regionId>>8)*64+loc.localX,y=(regionId&255)*64+loc.localY;
      if(x<2630||x>2660||y<9744||y>9835)continue;
      const gameval=core.CacheDefinitions.getLocNames().get(loc.id),name=core.ObjectDefinition.forId(loc.id)?.getName();
      if((x>=2643&&x<=2655&&y>=9827&&y<=9831)||/ladder|stair|bridge|lava/i.test(`${gameval} ${name}`))
        console.log('IKOV_BRIDGE_SOURCE_LOC',JSON.stringify({id:loc.id,x,y,sourcePlane:loc.plane,type:loc.type,face:loc.face,gameval,name}));
    }
  }
  for(let y=9827;y<=9831;y++){
    const xs=[];
    for(let x=2643;x<=2655;x++){
      const region=core.getTemplateRegion(((x>>6)<<8)|(y>>6));assert.ok(region,'actual cache region for primary-map crossing vicinity');
      const flag=region.settings[(1<<12)|((x&63)<<6)|(y&63)];
      console.log('IKOV_BRIDGE_TILE',JSON.stringify({x,y,plane:0,upperSettings:flag,clip:core.RegionManager.getClipping(x,y,0,null)}));
      if(flag&2)xs.push(x);
    }
    if(xs.length<2)continue;
    const min=Math.min(...xs),max=Math.max(...xs);
    if(max-min+1!==xs.length)continue;
    const west=new core.Location(min-1,y,0),east=new core.Location(max+1,y,0);
    if(core.RegionManager.blocked(west,null)||core.RegionManager.blocked(east,null))continue;
    candidates.push({min,max,y,west,east});
  }
  candidates.sort((a,b)=>Math.abs(a.y-9829)-Math.abs(b.y-9829));
  assert.ok(candidates.length>0,'actual contiguous bridge settings and usable banks in the revision-qualified primary-map vicinity');
  return candidates[0];
}
test('negative worn weight walks the actual cache bridge row both ways without a detour or damage',async t=>{
  const row=actualBridgeRow(),f=actor(t,row.east.getX(),row.y);health(f);
  f.p.getEquipment().setItem(core.Equipment.FEET_SLOT,new core.Item(core.ItemIdentifiers.BOOTS_OF_LIGHTNESS,1));
  assert.ok(f.p.getCarriedWeightGrams()<0,'actual worn-weight producer supplies negative carried grams');
  for(const [label,to]of [['west',row.west],['east',row.east]]){
    const trace=[where(f.p)],permissions=[f.p.hasFlag('movement:ignore-clipping')],hitpoints=[f.p.getHitpoints()];
    await f.input({type:'move',worldX:to.getX(),worldY:to.getY(),modifierFlags:0,run:false},0);
    for(let i=1;i<=16;i++){turns();trace.push(where(f.p));permissions.push(f.p.hasFlag('movement:ignore-clipping'));hitpoints.push(f.p.getHitpoints());console.log('IKOV_BRIDGE_ROUTE',JSON.stringify({label,turn:i,at:where(f.p),hp:f.p.getHitpoints(),grams:f.p.getCarriedWeightGrams(),ignoreClipping:f.p.hasFlag('movement:ignore-clipping')}));}
    assert.ok(trace.every(at=>at[1]===row.y&&at[2]===0),'ordinary input stays on this exact bridge row/plane');
    assert.ok(trace.some(at=>at[0]>=row.min&&at[0]<=row.max),'observed movement actually enters the cache bridge span');
    assert.deepEqual(where(f.p),[to.getX(),row.y,0],'ordinary input reaches the opposite real bank');
    assert.ok(hitpoints.every(hp=>hp===99),'99 HP at every observed movement turn');
    assert.ok(permissions.every(flag=>flag===false),'no clipping privilege at any observed movement turn');
  }
});
test('negative bridge passage keeps shared walls and does not authorize a concurrent nonnegative bystander',async t=>{
  const row=actualBridgeRow(),light=actor(t,row.east.getX(),row.y),heavy=actor(t,row.east.getX(),row.y);
  light.p.getEquipment().setItem(core.Equipment.FEET_SLOT,new core.Item(core.ItemIdentifiers.BOOTS_OF_LIGHTNESS,1));heavy.add(core.ItemIdentifiers.BOOTS_OF_LIGHTNESS);
  assert.ok(light.p.getCarriedWeightGrams()<0);assert.ok(heavy.p.getCarriedWeightGrams()>=0,'carried boots do not give their worn effect');
  const clips=()=>[9828,9829].flatMap(y=>[2648,2649,2650].map(x=>[x,y,core.RegionManager.getClipping(x,y,0,null)]));
  const before=clips(),trace=[];
  for(const f of[light,heavy])await f.input({type:'move',worldX:row.west.getX(),worldY:row.y,modifierFlags:0,run:false},0);
  for(let turn=1;turn<=16;turn++){turns();const state={turn,light:where(light.p),heavy:where(heavy.p),heavyPermission:heavy.p.hasFlag('movement:ignore-clipping'),heavyGrams:heavy.p.getCarriedWeightGrams()};trace.push(state);console.log('IKOV_BRIDGE_BYSTANDER',JSON.stringify(state));}
  assert.deepEqual(where(light.p),[row.west.getX(),row.y,0],'negative actor crosses through the real content producer');
  assert.ok(trace.every(r=>r.heavy[0]>=2650&&r.heavy[1]===row.y&&r.heavy[2]===0&&!r.heavyPermission&&r.heavyGrams>=0),'nonnegative actor remains on the east side without borrowed permission');
  assert.deepEqual(clips(),before,'all shared bridge-wall clipping remains intact');
  assert.equal(light.p.hasFlag('movement:ignore-clipping'),false);
});
test('an owned bridge passage releases its movement lock on real logout and area leave',async t=>{
  const row=actualBridgeRow();
  for(const ending of['logout','area leave']){
    const f=actor(t,row.east.getX(),row.y);f.p.getEquipment().setItem(core.Equipment.FEET_SLOT,new core.Item(core.ItemIdentifiers.BOOTS_OF_LIGHTNESS,1));
    await f.input({type:'move',worldX:row.west.getX(),worldY:row.y,modifierFlags:0,run:false},0);
    for(let turn=1;turn<=16&&!f.p.getAttribute('agility.obstacle');turn++){turns();console.log('IKOV_BRIDGE_CANCEL_APPROACH',JSON.stringify({ending,turn,at:where(f.p),taskRunning:f.p.getAttribute('agility.obstacle')?.isRunning()??false}));}
    const task=f.p.getAttribute('agility.obstacle');console.log('IKOV_BRIDGE_CANCEL',JSON.stringify({ending,when:'before',at:where(f.p),taskRunning:task?.isRunning(),blocked:f.p.getMovementQueue().isMovementBlocked()}));
    assert.ok(task?.isRunning(),'actual maintained scripted-walk task owns the reached bridge passage');assert.equal(task.player,f.p);
    assert.equal(f.p.getMovementQueue().isMovementBlocked(),true);
    if(ending==='logout')PM.emitPlayerLogout({player:f.p,username:f.p.getUsername()});
    else{const outside=surfaceLanding(f);settle(f,outside.getX(),outside.getY());}
    console.log('IKOV_BRIDGE_CANCEL',JSON.stringify({ending,when:'after',at:where(f.p),taskRunning:task.isRunning(),blocked:f.p.getMovementQueue().isMovementBlocked()}));
    assert.equal(task.isRunning(),false);assert.equal(f.p.getAttribute('agility.obstacle'),null);
    assert.equal(f.p.getMovementQueue().isMovementBlocked(),false);assert.equal(f.p.hasFlag('movement:ignore-clipping'),false);
  }
});

// The distant approach is selected from actual contiguous unblocked cache tiles,
// outside source030's old initial x<=2655 bank rectangle. No geometry is changed.
function fartherBridgeStart(row){
  let far=row.east.clone();
  for(let x=row.east.getX()+1;x<=2663;x++){
    const next=new core.Location(x,row.y,0);core.RegionManager.loadMapFiles(x,row.y);
    const blocked=core.RegionManager.blocked(next,null),connected=core.RegionManager.canMovestart(far,next,1,1,null);
    console.log('IKOV_BRIDGE_FAR_SETUP',JSON.stringify({at:[x,row.y,0],blocked,connected,clip:core.RegionManager.getClipping(x,row.y,0,null)}));
    if(blocked||!connected)break;
    far=next;
  }
  assert.ok(far.getX()>=2656,'setup: actual contiguous east approach starts outside the old bank rectangle');return far;
}
test('one ordinary cross-bridge click from a farther cache-verified approach reaches the opposite bank',async t=>{
  const row=actualBridgeRow(),far=fartherBridgeStart(row),f=actor(t,far.getX(),far.getY());health(f);
  f.p.getEquipment().setItem(core.Equipment.FEET_SLOT,new core.Item(core.ItemIdentifiers.BOOTS_OF_LIGHTNESS,1));
  assert.ok(f.p.getCarriedWeightGrams()<0);const trace=[];
  await f.input({type:'move',worldX:row.west.getX(),worldY:row.y,modifierFlags:0,run:false},0);
  for(let turn=1;turn<=24;turn++){turns();const sample={turn,at:where(f.p),hp:f.p.getHitpoints(),ignore:f.p.hasFlag('movement:ignore-clipping'),task:!!f.p.getAttribute('agility.obstacle')};trace.push(sample);console.log('IKOV_BRIDGE_FAR_ROUTE',JSON.stringify(sample));}
  assert.deepEqual(where(f.p),[row.west.getX(),row.y,0],'the first click crosses without a second click at the wall');
  assert.ok(trace.some(r=>r.task),'actual maintained bridge passage was reached');
  assert.ok(trace.every(r=>r.at[1]===row.y&&r.at[2]===0&&r.hp===99&&!r.ignore),'the distant ordinary approach retains row, plane, HP and scoped movement');
});
test('superseding ordinary interaction or walk cancels the pending bridge approach',async t=>{
  const row=actualBridgeRow(),far=fartherBridgeStart(row);
  for(const replacement of['ground Take','walk cancellation']){
    const f=actor(t,far.getX(),far.getY());health(f);
    f.p.getEquipment().setItem(core.Equipment.FEET_SLOT,new core.Item(core.ItemIdentifiers.BOOTS_OF_LIGHTNESS,1));
    const coin=core.ItemIdentifiers.COINS,edge=new core.Location(2650,row.y,0);
    const ground=new core.ItemOnGround(core.GroundItemState.SEEN_BY_PLAYER,f.p.getUsername(),edge,new core.Item(coin,1),false,-1,f.p.getPrivateArea());
    core.ItemOnGroundManager.register(ground);
    assert.equal(core.ItemOnGroundManager.canOperate(f.p,ground,core.ItemOnGroundManager.takeOperation(coin)),true,'actual visible operable fixture coin before ordinary interaction');
    await f.input({type:'move',worldX:row.west.getX(),worldY:row.y,modifierFlags:0,run:false},1);
    console.log('IKOV_BRIDGE_REPLACE',JSON.stringify({replacement,when:'before',at:where(f.p),pending:core.TaskManager.hasActiveTask(f.p.getIndex()),passage:!!f.p.getAttribute('agility.obstacle')}));
    assert.equal(f.p.getAttribute('agility.obstacle'),undefined,'setup: first walk is still approaching the edge');
    assert.equal(core.TaskManager.hasActiveTask(f.p.getIndex()),true,'actual maintained approach task is pending');
    if(replacement==='ground Take')await f.input({type:'ground_item_action',itemId:coin,x:edge.getX(),y:edge.getY(),option:'Take',stackId:ground.getId()},0);
    else await f.input({type:'move',worldX:far.getX(),worldY:far.getY(),modifierFlags:0,run:false},0);
    const trace=[];
    for(let turn=1;turn<=24;turn++){turns();const sample={replacement,turn,at:where(f.p),passage:!!f.p.getAttribute('agility.obstacle'),blocked:f.p.getMovementQueue().isMovementBlocked(),ignore:f.p.hasFlag('movement:ignore-clipping')};trace.push(sample);console.log('IKOV_BRIDGE_REPLACE',JSON.stringify(sample));}
    assert.ok(trace.every(r=>r.at[0]>=2650&&r.at[1]===row.y&&r.at[2]===0&&!r.passage&&!r.blocked&&!r.ignore),'obsolete crossing never borrows the superseding route');
    if(replacement==='ground Take'){
      assert.equal(f.p.getInventory().getAmount(coin),1,'ordinary superseding Take actually completes at the real stack');
      assert.equal(core.World.getItems().includes(ground),false,'successful normal transfer removes fixture stock');
      assert.ok(f.events.some(e=>e.name==='ground-items:taken'&&e.itemId===coin),'normal taken producer witnessed');
    }else assert.deepEqual(where(f.p),[far.getX(),row.y,0],'ordinary replacement walk wins');
  }
});

async function ordinaryWalk(f,to,bound,label){
  await f.input({type:'move',worldX:to.getX(),worldY:to.getY(),modifierFlags:0,run:false},0);
  for(let turn=1;turn<=bound;turn++){
    turns();console.log('IKOV_CONNECTED_ROUTE',JSON.stringify({label,turn,at:where(f.p),destination:[to.getX(),to.getY(),to.getZ()],ignore:f.p.hasFlag('movement:ignore-clipping'),queue:f.p.getMovementQueue().size()}));
    assert.equal(f.p.hasFlag('movement:ignore-clipping'),false);
    if(f.p.getLocation().equals(to))return;
  }
  assert.deepEqual(where(f.p),[to.getX(),to.getY(),to.getZ()],label+' arrives through ordinary movement');
}
test('the actual southern lower-route ladder exits south of Hemenster and ordinary east travel returns to the temple entrance',async t=>{
  // Declared lower-route ladder setup, NOT a bridge-fall landing or trigger.
  const f=actor(t,2637,9809),ladder=object(17385,2637,9808,f.p);
  const end=ClimbLinks.otherEnd(ladder,ClimbLinks.UP);console.log('IKOV_LOWER_ROUTE',JSON.stringify({lower:{id:ladder.getId(),at:where(ladder),face:ladder.getFace(),type:ladder.getType()},surface:end?{id:end.getId(),at:where(end),name:core.ObjectDefinition.forId(end.getId()).getName()}:null}));
  assert.ok(end,'setup: actual lower ladder has a cache-derived surface counterpart');
  assert.equal(core.ObjectDefinition.forId(end.getId()).getName(),'Ladder');assert.equal(end.getLocation().getZ(),0);
  assert.ok(end.getLocation().getX()>=2624&&end.getLocation().getX()<=2650&&end.getLocation().getY()>=3390&&end.getLocation().getY()<3420,'surface counterpart lies in the supported south-Hemenster vicinity');
  f.approach(ladder);const expected=ClimbLinks.destination(ladder,ClimbLinks.UP,f.p.getLocation(),f.p.getPrivateArea());assert.ok(expected);
  await f.click(ladder.getId(),2637,9808,'Climb-up',1,8);assert.deepEqual(where(f.p),[expected.getX(),expected.getY(),expected.getZ()]);
  const entrance=surfaceLanding(f);await ordinaryWalk(f,entrance,96,'east from supported southern exit to temple entrance');
  const raw=core.MapObjects.mapObjects.get(core.MapObjects.getHash(2677,3405,0))??[],down=raw.find(o=>core.ObjectDefinition.forPlayer(o.getId(),f.p)?.getName()==='Ladder');assert.ok(down);
  const destination=ClimbLinks.destination(down,ClimbLinks.DOWN,f.p.getLocation(),f.p.getPrivateArea());assert.ok(destination,'actual entrance resolves its dungeon counterpart');
  await f.click(down.getId(),2677,3405,'Climb-down',1,8);assert.deepEqual(where(f.p),[destination.getX(),destination.getY(),destination.getZ()]);
  assert.ok(f.p.getLocation().getY()>9700&&f.p.getLocation().getY()<9916,'real climb re-enters Temple dungeon');
});
test('Winelda paid teleport reaches the optional-key corridor and the Guardian wall opens without owning the shiny key',async t=>{
  const f=actor(t,2654,9876,40);health(f);
  for(const[key,value]of[['temple-of-ikov:state-version',1],['temple-of-ikov:warrior-killed',1],['temple-of-ikov:paid-winelda',0],['temple-of-ikov:met-winelda',0]])f.p.setAttribute(key,value);
  f.add(core.ItemIdentifiers.LIMPWURT_ROOT,20);const key=core.ItemIdentifiers.SHINY_KEY;
  assert.equal(f.p.getInventory().getAmount(key),0);
  await f.talk(core.NpcIdentifiers.WINELDA,2655,9876);
  for(let i=0;i<20&&!f.p.getAttribute('temple-of-ikov:paid-winelda')&&f.p.getDialogueManager().getCurrent();i++)await f.resume();
  assert.equal(f.p.getAttribute('temple-of-ikov:paid-winelda'),1,'actual source teleport accepts the physical roots');
  assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.LIMPWURT_ROOT),0);
  assert.ok(f.events.some(e=>e.name==='npc-dialogue:action'&&e.stepId==='UB9hSI'),'actual native dialogue reaches Winelda payment action');
  assert.deepEqual(where(f.p),[2644,9890,0],'observed authored teleport destination, not a claimed retail exact-tile capture');
  f.p.closeInterruptibleInterfaces();
  const wall=object(1597,2643,9892,f.p),near=new core.Location(2643,9891,0);
  assert.equal(core.RegionManager.blocked(near,f.p.getPrivateArea()),false,'actual near side of Guardian wall is unblocked');
  await ordinaryWalk(f,near,16,'paid landing to Guardian wall with no shiny key');
  assert.equal(f.p.getInventory().getAmount(key),0);await f.click(wall.getId(),2643,9892,'Push',1,8);
  assert.deepEqual(where(f.p),[2643,9893,0],'Guardian chamber entry is not gated by the optional shortcut key');
  await f.click(wall.getId(),2643,9892,'Push',1,8);assert.deepEqual(where(f.p),[2643,9891,0]);
  const stock=core.World.getItems().filter(item=>item.isStaticSpawn()&&!item.isPendingRemoval()&&item.getItem().getId()===key&&item.getPosition().equals(new core.Location(2628,9859,0)));
  assert.equal(stock.length,1,'actual canonical shiny key supply in the guide corridor');
  assert.equal(core.RegionManager.blocked(stock[0].getPosition(),f.p.getPrivateArea()),false,'setup: canonical optional key has an unblocked ground approach');
  await ordinaryWalk(f,stock[0].getPosition(),96,'optional corridor detour to shiny key');
  assert.ok(core.ItemOnGroundManager.canOperate(f.p,stock[0],core.ItemOnGroundManager.takeOperation(key))&&core.PathFinder.reachedObj(f.p,stock[0].getPosition()),'actual visible reachable key before Take');
  await f.input({type:'ground_item_action',itemId:key,x:2628,y:9859,option:'Take',stackId:stock[0].getId()},6);
  assert.equal(f.p.getInventory().getAmount(key),1,'optional key is obtained through real pickup, not injected');
  assert.ok(f.events.some(e=>e.name==='ground-items:taken'&&e.itemId===key));
  await ordinaryWalk(f,near,96,'key corridor back to Guardian wall');await f.click(wall.getId(),2643,9892,'Push',1,8);
  assert.deepEqual(where(f.p),[2643,9893,0]);assert.equal(f.p.getInventory().getAmount(key),1,'optional shortcut key is retained');
});
