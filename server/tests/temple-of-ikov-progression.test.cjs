// After yarn build: node --test --test-concurrency=1 tests/temple-of-ikov-progression.test.cjs
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
  return {p,c,messages,input,click,add,talk,resume,approach,itemEvents,objectEvents};
}
test('the active map contains the displaced stairs and six canonical ice chests',t=>{
  const f=actor(t,2653,9806,0);
  // Compact source diagnostics bind connected geometry to the cache actually loaded by this run.
  const rows=[];
  for(const list of core.MapObjects.mapObjects.values()) for(const o of list) {
    const [x,y,z]=where({getLocation:()=>o.getLocation()});
    if(z!==0)continue;
    if((x>=2634&&x<=2653&&y>=9814&&y<=9823)||(x>=2647&&x<=2656&&y>=9800&&y<=9810)
      ||(o.getId()>=86&&o.getId()<=104&&x>=2630&&x<=2750&&y>=9758&&y<=9915)||(o.getId()===102&&x>=3110&&x<=3130&&y>=3470&&y<=3495)) {
      const visible=core.ObjectDefinition.forPlayer(o.getId(),f.p);
      rows.push([o.getId(),x,y,o.getType(),o.getFace(),core.ObjectDefinition.forId(o.getId()).getName(),visible.id,visible.getName()]);
    }
  }
  // Emit bounded rows before any mapping assertion. Provider raw-line limits cannot clip a giant array.
  for(const row of rows.sort((a,b)=>a[1]-b[1]||a[2]-b[2]||a[0]-b[0]))console.log('IKOV_CACHE_LOC',JSON.stringify(row));
  const down=templeStairs(98,f.p),up=templeStairs(96,f.p);
  assert.ok(down.getLocation().getDistance(up.getLocation())>4,'the real endpoints are displaced beyond generic pairing distance');
  for (const [x,y] of [[2710,9850],[2719,9838],[2729,9850],[2747,9848],[2738,9835],[2745,9821]]) object(103,x,y);
});
test('boots stairs refuse darkness through the ordinary descend input',async t=>{
  const f=actor(t,2653,9806,0),stairs=templeStairs(98,f.p);f.approach(stairs);
  await f.click(98,stairs.getLocation().getX(),stairs.getLocation().getY(),'Climb-down');
  assert.ok(f.messages.some(s=>s.includes("It's a bit dark down here!")), 'the quest explains why descent is refused');
  assert.ok(f.p.getLocation().getY()>9800,'no descent without light');
});
test('boots stairs descend with a lit source and return to the dungeon',async t=>{
  // Same canonical startup-data producer used by the existing traversal fixture.
  const { GroundItemSpawnDefinitionLoader } = require('../dist/game/definition/loader/impl/GroundItemSpawnDefinitionLoader');
  assert.equal(new GroundItemSpawnDefinitionLoader().load(),true,'canonical ground-spawn loader');
  const f=actor(t,2653,9806,0);f.add(core.ItemIdentifiers.LIT_TORCH);
  // Supplied cutting tool, not a naturally earned item or injected quest outcome.
  // The existing Web producer gives this real tool chance1; no RNG stub/retry.
  const boots=core.ItemIdentifiers.BOOTS_OF_LIGHTNESS,tool=core.ItemIdentifiers.WILDERNESS_SWORD_1;
  const webId=733,slashedId=734; // exact pinned242 definitions and existing Web producer
  f.add(tool);assert.equal(f.p.getInventory().getAmount(boots),0);
  const toolDefinition=core.ItemDefinition.forId(tool);
  assert.equal(toolDefinition.getName(),'Wilderness sword 1');
  assert.equal(toolDefinition.getEquipmentType().getSlot(),core.Equipment.WEAPON_SLOT);
  const down=templeStairs(98,f.p);f.approach(down);
  await f.click(98,down.getLocation().getX(),down.getLocation().getY(),'Climb-down',1,8);
  assert.ok(f.p.getLocation().getY()>=9759&&f.p.getLocation().getY()<=9768,'the lit descend reaches the boots room');
  // Guide boots-room bounds select actual map data; no web coordinate is invented.
  const webs=[];
  for(const list of core.MapObjects.mapObjects.values())for(const web of list){
    const at=web.getLocation();
    if(web.getId()===webId&&at.getZ()===0&&at.getX()>=2637&&at.getX()<=2654&&at.getY()>=9759&&at.getY()<=9767)webs.push(web);
  }
  assert.equal(webs.length,1,'setup: one actual733 web in the guide boots-room bounds');
  const web=webs[0],at=web.getLocation().clone(),area=web.getPrivateArea(),shape=web.getType(),face=web.getFace();
  assert.equal(area,f.p.getPrivateArea());
  assert.equal(core.MapObjects.getPrivateArea(f.p,webId,at),web,'setup: ordinary object lookup resolves that actual standing web');
  assert.equal(core.ObjectDefinition.forPlayer(webId,f.p).getInteractions()[0],'Slash');
  const stock=core.World.getItems().filter(item=>item.isStaticSpawn()&&!item.isPendingRemoval()&&item.getItem().getId()===boots&&item.getPosition().equals(new core.Location(2654,9767,0)));
  assert.equal(stock.length,1,'setup: actual generated static boots stock, not an injected item');
  const ground=stock[0];assert.equal(ground.getPrivateArea(),f.p.getPrivateArea());
  assert.equal(core.ItemOnGroundManager.canOperate(f.p,ground,core.ItemOnGroundManager.takeOperation(boots)),true);
  const taken=[],emitCustom=PM.emitCustomEvent.bind(PM);
  t.mock.method(PM,'emitCustomEvent',(name,event)=>{
    if(name==='ground-items:taken'&&event.player===f.p)taken.push(event);
    return emitCustom(name,event);
  });
  // Exactly one ordinary Slash: the production path walks to the real object,
  // cuts with its real carried tool and runs its own delayed replacement.
  await f.click(webId,at.getX(),at.getY(),'Slash',1,32);
  assert.ok(f.messages.includes('You slash the web apart.'));
  const slashed=core.MapObjects.getPrivateArea(f.p,slashedId,at);
  assert.ok(slashed,'ordinary slash replaces the standing web');
  assert.equal(slashed.getType(),shape);assert.equal(slashed.getFace(),face);assert.equal(slashed.getPrivateArea(),area);
  assert.equal(core.MapObjects.getPrivateArea(f.p,webId,at),null);
  // One ordinary ground Take keeps the actual stack identity through arrival.
  // No moveTo/teleport, inventory add or quest flag supplies the boots.
  await f.input({type:'ground_item_action',itemId:boots,x:2654,y:9767,option:'Take',stackId:ground.getId()},32);
  assert.equal(f.p.getInventory().getAmount(boots),1);
  assert.ok(taken.some(event=>event.groundItem===ground&&event.itemId===boots&&event.amount===1&&event.source==='pickup'));
  assert.ok(ground.isPendingRemoval()||!core.World.getItems().includes(ground),'the physical stock was removed by pickup');
  const up=templeStairs(96,f.p);
  const stand=ClimbLinks.landingTile(up,{x:f.p.getLocation().getX(),y:f.p.getLocation().getY()},f.p.getPrivateArea());
  assert.ok(stand,'setup: existing geometry helper finds a climbable return stand');
  await f.input({type:'move',worldX:stand.getX(),worldY:stand.getY(),modifierFlags:0,run:false},32);
  assert.deepEqual(where(f.p),[stand.getX(),stand.getY(),stand.getZ()],'ordinary walk returns to the stairs without placement bypass');
  await f.click(96,up.getLocation().getX(),up.getLocation().getY(),'Climb-up',1,12);
  assert.ok(f.p.getLocation().getY()>=9803&&f.p.getLocation().getY()<=9810,'the climb returns to the main room');
  assert.equal(f.p.getInventory().getAmount(boots),1,'the acquired optional boots survive the ordinary return');
});
test('using the physical lever on its bracket consumes it once and creates the Pull interaction',async t=>{
  const f=actor(t,2671,9805); f.add(core.ItemIdentifiers.LEVER); object(86,2671,9804);
  await f.input({type:'item_on_object',widgetId:3214,objectId:86,itemId:core.ItemIdentifiers.LEVER,slot:f.p.getInventory().getSlotForItemId(core.ItemIdentifiers.LEVER),x:2671,y:9804},5);
  assert.ok(f.itemEvents.some(event=>event.objectId===86&&event.itemId===core.ItemIdentifiers.LEVER),'native item-on-object dispatch reached the physical bracket producer');
  assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.LEVER),0,'fitting consumes the physical lever');
  assert.ok(f.messages.some(s=>s.includes('fit the lever into the bracket')));
  const fitted=core.MapObjects.getPrivateArea(f.p,87,new core.Location(2671,9804,0));
  assert.ok(fitted,'the fitted lever is a real object visible to the player');
  assert.ok(core.ObjectDefinition.forId(fitted.getId()).getInteractions().some(s=>s==='Pull'),'the fitted lever has a native Pull menu');
});
test('searching the trapped lever at 42 Thieving disarms its actual trap',async t=>{
  const f=actor(t,2665,9854);const sm=f.p.getSkillManager();sm.setMaxLevel(core.Skill.THIEVING,42);sm.setCurrentLevel(core.Skill.THIEVING,42);
  assert.equal(sm.getCurrentLevel(core.Skill.THIEVING),42);
  await f.click(91,2665,9855,'Search for traps',2,5);
  assert.ok(f.messages.some(s=>s.includes('disable the trap')),'successful search has the canonical trap feedback');
});
test('the ice area can supply arrows from its six actual chest producers',async t=>{
  const {AreaManager}=require('../dist/game/model/areas/AreaManager');
  const replays=[],operated=[],sends=[],owners=[];
  const ownedReplays=()=>replays.filter(row=>owners.includes(row.args[0]));
  function observe(label,facts){assert.ok(Buffer.byteLength(JSON.stringify(facts))<=1024*1024);console.log(label,JSON.stringify(facts));}
  const replay=core.ObjectManager.onRegionChange,emit=PM.emitCustomEvent;
  // Observe initial real PlayerSession tuples before either owned actor is constructed.
  t.mock.method(core.ObjectManager,'onRegionChange',function(...args){
    const result=Reflect.apply(replay,this,args);replays.push({args:[...args],result});
    assert.ok(replays.length<=128,'bounded actual replay observations');return result;
  });
  t.mock.method(PM,'emitCustomEvent',function(name,event,...args){
    const result=Reflect.apply(emit,this,[name,event,...args]);
    if(name==='object:operated'&&owners.includes(event.player))operated.push(event);
    assert.ok(operated.length<=128,'bounded actual operated observations');return result;
  });
  const f=actor(t,2710,9849),arrows=core.ItemIdentifiers.ICE_ARROWS;owners.push(f.p);
  const area=f.p.getArea();assert.ok(area&&AreaManager.areas.includes(area),'setup: actual registered chest area');
  const boundaries=area.getBoundaries();assert.ok(boundaries.length>0);
  const bases=[...new Set([...core.MapObjects.mapObjects.values()].flat())].filter(o=>
    o.getId()===103&&o.getOwner()===null&&o.getLocation().getZ()===f.p.getLocation().getZ()
      &&boundaries.some(boundary=>boundary.inside(o.getLocation())));
  bases.sort((a,b)=>a.getLocation().getX()-b.getLocation().getX()||a.getLocation().getY()-b.getLocation().getY());
  assert.equal(bases.length,6,'setup: exactly six actual closed cache producers in this registered area');
  assert.equal(new Set(bases.map(o=>where({getLocation:()=>o.getLocation()}).join(','))).size,6);
  for(const base of bases){
    const definition=core.ObjectDefinition.forPlayer(base.getId(),f.p);
    assert.equal(definition.id,103);assert.ok(definition.getInteractions().some(option=>option?.toLowerCase()==='open'));
  }
  const first=bases[0],at=first.getLocation();
  const b=actor(t,at.getX(),at.getY());owners.push(b.p);assert.equal(b.p.getArea(),area);
  const opened=new Map(),bOpened=new Map();
  const inventory=p=>structuredClone(p.getInventory().getItems().map(item=>({id:item.getId(),amount:item.getAmount(),meta:item.getMeta()})));
  const inventories=()=>owners.map(inventory);
  const objectFact=o=>({id:o.getId(),at:where({getLocation:()=>o.getLocation()}),type:o.getType(),face:o.getFace(),
    owner:o.getOwner()?.getUsername()??null,privateArea:o.getPrivateArea()===f.p.getPrivateArea()?'fixture-area':'other-area'});
  const facts=map=>[...map.values()].map(objectFact);
  const lookup=(p,base)=>core.MapObjects.getPrivateArea(p,104,base.getLocation());
  const shared=()=>{for(const base of bases)assert.equal(core.MapObjects.get(103,base.getLocation(),f.p.getPrivateArea()),base,'original shared base survives');};
  const standing=(p,map)=>{
    for(const [base,object]of map){
      assert.equal(lookup(p,base),object);assert.equal(object.getOwner(),p);
      assert.equal(object.getType(),base.getType());assert.equal(object.getFace(),base.getFace());assert.equal(object.getPrivateArea(),base.getPrivateArea());
      assert.equal(core.ObjectManager.objectsAt(base.getLocation()).filter(o=>o.getOwner()===p&&o.getId()===104).length,1);
      assert.ok(core.ObjectManager.objectsAt(base.getLocation()).includes(object));
      const tuple=ownedReplays().find(row=>row.args[0]===p);assert.ok(tuple,'setup: actual initial PlayerSession replay tuple');
      assert.ok(core.MapObjects.ownerObjectsInScene(p,tuple.args[1],tuple.args[2],104).includes(object));
    }
    shared();
  };
  for(const owner of owners){
    const sender=owner.getPacketSender(),send=sender.sendObject;
    t.mock.method(sender,'sendObject',function(...args){
      const result=Reflect.apply(send,this,args);sends.push({player:owner,object:args[0],result});
      assert.ok(sends.length<=512,'bounded actual object sender observations');return result;
    });
  }
  assert.equal(f.p.getInventory().getAmount(arrows),0);assert.equal(b.p.getInventory().getAmount(arrows),0);
  let motion=null;
  const firstSearchRoutes=[],firstSearchDispatches=[];
  const routeFact=row=>({object:objectFact(row.object),postDefinitionId:row.postDefinitionId,sourceLocation:row.sourceLocation,
    destination:row.destination,extraFlagsPresent:row.extraFlagsPresent,extraFlagsWasUndefined:row.extraFlagsWasUndefined,
    extraFlags:row.extraFlagsWasUndefined?null:row.extraFlags,width:row.width,length:row.length,sideMask:row.sideMask,
    effectiveMask:row.effectiveMask,face:row.face,shape:row.shape,error:row.error??null});
  function firstSearchComplete(){
    for(const row of [...firstSearchRoutes,...firstSearchDispatches])if(row.failure)throw row.failure;
    assert.ok(firstSearchRoutes.length<=1&&firstSearchDispatches.length<=1,'no duplicate first Search route/dispatch');
    const actual=f.objectEvents.slice(motion.searchEventsA),deeds=operated.slice(motion.searchOps);
    assert.ok(actual.length<=1&&deeds.length<=1,'no duplicate or wrong-owner first Search completion');
    for(const event of [...actual,...deeds]){assert.equal(event.player,f.p);assert.equal(event.object,motion.target);assert.equal(event.option,'search');}
    assert.equal(b.objectEvents.length,motion.searchEventsB,'first Search never dispatches for B');
    return actual.length===1&&actual[0].handled===true&&deeds.length===1&&firstSearchDispatches.length===1&&firstSearchDispatches[0].handled===true;
  }
  function advanceMotion(phase){
    assert.ok(motion.turns<16,'ordinary move and first Search share at most sixteen World turns');
    turns();motion.turns++;
    assert.equal(f.p.isRegistered(),true);assert.equal(f.p.isNeedsPlacement(),false);
    assert.equal(f.p.getLocation().getZ(),motion.start.getZ());assert.equal(f.p.getArea(),area);assert.equal(f.p.getPrivateArea(),motion.privateArea);
    assert.equal(f.p.hasFlag('movement:ignore-clipping'),false);assert.equal(f.p.getHitpoints(),motion.hp);
    standing(f.p,opened);standing(b.p,bOpened);
    if(firstSearchDispatches.length===0)assert.deepEqual(inventories(),motion.before,'no movement/approach reward before actual Search dispatch');
    else{assert.equal(phase,'first Search approach');assert.equal(firstSearchComplete(),true,'actual Search dispatch completes through handled+operated');}
    assert.deepEqual(inventory(b.p),motion.before[1],'motion/Search cannot change B inventory');
    if(phase==='outgoing move'){
      assert.equal(f.objectEvents.length,motion.eventsA);assert.equal(b.objectEvents.length,motion.eventsB);assert.equal(operated.length,motion.ops);
    }
    const queue=f.p.getMovementQueue(),queued=queue.getDestination();
    if(phase==='outgoing move'&&!f.p.getLocation().equals(motion.destination)&&queued)assert.ok(queued.equals(motion.destination),'ordinary queued endpoint remains the requested adjacent cell');
    if(phase==='outgoing move')motion.stepped=motion.stepped||queue.didMoveThisCycle();
    motion.trace.push(where(f.p));
    observe('IKOV_CHEST_MOVE',{phase,turn:motion.turns,at:where(f.p),requestedOutgoing:[motion.destination.getX(),motion.destination.getY(),motion.destination.getZ()],
      queuedDestination:queued?[queued.getX(),queued.getY(),queued.getZ()]:null,routeEvaluated:queue.wasRouteEvaluated(),alternativeRoute:queue.usedAlternativeRoute(),
      movedThisCycle:queue.didMoveThisCycle(),hp:f.p.getHitpoints(),ignore:false,inventories:inventories(),firstSearchDispatched:firstSearchDispatches.length,
      firstSearchRoutes:firstSearchRoutes.map(routeFact),firstSearchDispatches:firstSearchDispatches.map(row=>row.facts),A:facts(opened),B:facts(bOpened)});
  }
  async function action(who,base,id,option,clickType=1){
    const target=id===103?base:lookup(who.p,base);assert.ok(target);
    const before=inventories(),events=who.objectEvents.length,ops=operated.length;
    const firstSearch=motion!==null&&!motion.firstSearchDone&&who===f&&base===first&&id===104&&option==='Search'&&clickType===1;
    if(firstSearch){
      assert.equal(target,motion.target);assert.ok(f.p.getLocation().equals(motion.destination),'outgoing ordinary move already arrived');
      motion.searchEventsA=events;motion.searchEventsB=b.objectEvents.length;motion.searchOps=ops;motion.searchInputIssued=true;
      try{
        await who.input({type:'object_option',id,x:base.getLocation().getX(),y:base.getLocation().getY(),action:option,clickType},0);
        while(!firstSearchComplete())advanceMotion('first Search approach');
        assert.equal(firstSearchRoutes.length,1);assert.equal(firstSearchDispatches.length,1);motion.firstSearchDone=true;
      }catch(error){
        const queue=f.p.getMovementQueue(),queued=queue.getDestination();
        observe('IKOV_CHEST_FIRST_SEARCH_FAILURE',{at:where(f.p),sharedTurns:motion.turns,trace:motion.trace,hp:f.p.getHitpoints(),
          registered:f.p.isRegistered(),needsPlacement:f.p.isNeedsPlacement(),sameArea:f.p.getArea()===area,samePrivateArea:f.p.getPrivateArea()===motion.privateArea,
          queuedDestination:queued?[queued.getX(),queued.getY(),queued.getZ()]:null,routeEvaluated:queue.wasRouteEvaluated(),alternativeRoute:queue.usedAlternativeRoute(),inventories:inventories(),
          routes:firstSearchRoutes.map(routeFact),dispatches:firstSearchDispatches.map(row=>row.facts),
          error:{name:error.name,message:error.message,stack:error.stack},A:facts(opened),B:facts(bOpened)});
        throw error;
      }
    }else await who.click(id,base.getLocation().getX(),base.getLocation().getY(),option,clickType);
    const actual=who.objectEvents.slice(events).filter(e=>e.player===who.p);
    const deeds=operated.slice(ops).filter(e=>e.player===who.p);
    assert.equal(actual.length,1);assert.equal(actual[0].object,target);assert.equal(actual[0].definition?.id,id);assert.equal(actual[0].clickType,clickType);
    assert.equal(actual[0].option,option.toLowerCase());assert.equal(actual[0].handled,true);
    assert.equal(deeds.length,1);assert.equal(deeds[0].object,target);assert.equal(deeds[0].option,option.toLowerCase());
    shared();return {target,before,after:inventories(),event:actual[0],deed:deeds[0],
      firstSearch:firstSearch?{sharedMotionTurns:motion.turns,route:routeFact(firstSearchRoutes[0]),dispatch:firstSearchDispatches[0].facts}:undefined};
  }
  function record(label,result,movementSetup){
    const observation={label,actor:result.event.player.getUsername(),object:objectFact(result.target),option:result.event.option,handled:result.event.handled,
      operatedSameObject:result.deed.object===result.target,before:result.before,after:result.after,A:facts(opened),B:facts(bOpened)};
    if(movementSetup!==undefined)observation.movementSetup=movementSetup;
    if(result.firstSearch!==undefined)observation.firstSearch=result.firstSearch;
    observe('IKOV_CHEST_STEP',observation);
  }
  f.approach(first);
  let step=await action(f,first,103,'Open');assert.deepEqual(step.after,step.before);
  opened.set(first,lookup(f.p,first));assert.ok(opened.get(first));standing(f.p,opened);
  assert.equal(lookup(b.p,first),null,'foreign owner form cannot be resolved by B');record('A Open',step);
  b.approach(first);
  const beforeForeign=inventories(),foreignEvents=b.objectEvents.length,foreignOps=operated.length;
  await b.input({type:'object_option',id:104,x:at.getX(),y:at.getY(),action:'Search',clickType:1},4);
  assert.equal(b.objectEvents.length,foreignEvents);assert.equal(operated.length,foreignOps);assert.deepEqual(inventories(),beforeForeign);
  assert.equal(lookup(b.p,first),null);standing(f.p,opened);
  observe('IKOV_CHEST_FOREIGN_REFUSAL',{actor:b.p.getUsername(),at:objectFact(first).at,before:beforeForeign,after:inventories(),objectEvents:0,operatedEvents:0,A:facts(opened)});
  step=await action(b,first,103,'Open');assert.deepEqual(step.after,step.before);
  bOpened.set(first,lookup(b.p,first));assert.ok(bOpened.get(first));assert.notEqual(bOpened.get(first),opened.get(first));standing(f.p,opened);standing(b.p,bOpened);
  // The move endpoint is one actual connected cell; operation reach is checked at the real first Search dispatcher.
  const start=f.p.getLocation().clone(),size=f.p.getSize(),privateArea=f.p.getPrivateArea(),hp=f.p.getHitpoints(),beforeMove=inventories();
  assert.ok(Number.isInteger(size)&&size>0);assert.equal(f.p.getArea(),area);assert.equal(f.p.isRegistered(),true);assert.equal(f.p.isNeedsPlacement(),false);
  assert.ok(hp>0);assert.equal(f.p.hasFlag('movement:ignore-clipping'),false);
  const startBlocked=core.RegionManager.blocked(start,privateArea);
  const candidates=[[0,1],[1,0],[0,-1],[-1,0]].map(([dx,dy])=>{
    const tile=new core.Location(start.getX()+dx,start.getY()+dy,start.getZ());
    const inside=boundaries.some(boundary=>boundary.inside(tile)),blocked=core.RegionManager.blocked(tile,privateArea);
    const forward=core.RegionManager.canMovestart(start,tile,size,size,privateArea),reverse=core.RegionManager.canMovestart(tile,start,size,size,privateArea);
    return {tile,inside,blocked,forward,reverse,eligible:!startBlocked&&inside&&!blocked&&forward&&reverse};
  });
  const chosen=candidates.find(candidate=>candidate.eligible),destination=chosen?.tile,target=opened.get(first);
  const base104=core.ObjectDefinition.forId(target.getId()),resolved104=core.ObjectDefinition.forPlayer(target.getId(),f.p);
  record('B independent Open',step,{actor:f.p.getUsername(),sampledAfter:'B Open invariants',start:where(f.p),size,plane:start.getZ(),startBlocked,
    target:objectFact(target),baseFootprint:[base104.getSizeX(),base104.getSizeY()],resolvedDefinitionId:resolved104.id,sideMask:resolved104.getBlockingMask(),
    candidates:candidates.map(candidate=>({at:[candidate.tile.getX(),candidate.tile.getY(),candidate.tile.getZ()],inside:candidate.inside,blocked:candidate.blocked,
      forward:candidate.forward,reverse:candidate.reverse,eligible:candidate.eligible})),chosen:destination?[destination.getX(),destination.getY(),destination.getZ()]:null});
  assert.ok(destination,'setup: an actual same-area adjacent bidirectional clipping-valid edge exists');
  assert.equal(Math.abs(destination.getX()-start.getX())+Math.abs(destination.getY()-start.getY()),1);
  motion={start,size,privateArea,hp,before:beforeMove,destination,target,turns:0,trace:[where(f.p)],stepped:false,
    eventsA:f.objectEvents.length,eventsB:b.objectEvents.length,ops:operated.length,searchInputIssued:false,firstSearchDone:false};
  const route=PM.emitObjectRoute,interact=PM.emitObjectInteraction;
  t.mock.method(PM,'emitObjectRoute',function(event,...args){
    const result=Reflect.apply(route,this,[event,...args]);
    if(motion.searchInputIssued&&!motion.firstSearchDone&&event.player===f.p){
      const extra=event.blockAccessFlags;
      const footprint=core.ObjectDefinition.forId(event.object.getId()),resolved=core.ObjectDefinition.forPlayer(event.object.getId(),f.p);
      const observed={object:event.object,postDefinitionId:event.definition?.id??null,sourceLocation:{...event.sourceLocation},
        destination:event.destination==null?null:{...event.destination},extraFlagsPresent:Object.hasOwn(event,'blockAccessFlags'),extraFlagsWasUndefined:extra===undefined,extraFlags:extra,
        width:footprint.getSizeX(),length:footprint.getSizeY(),sideMask:resolved.getBlockingMask(),effectiveMask:resolved.getBlockingMask()|(extra===undefined?0:extra),
        face:event.object.getFace(),shape:10};
      firstSearchRoutes.push(observed);
      try{
        assert.equal(firstSearchRoutes.length,1,'one original first Search route event');
        assert.equal(event.object,target);assert.equal(event.objectId,104);assert.equal(event.clickType,1);assert.equal(event.object.getOwner(),f.p);
        assert.equal(event.object.getType(),10);assert.equal(observed.postDefinitionId,104);assert.equal(resolved.id,104);
        assert.ok(extra===undefined||Number.isInteger(extra),'actual extra access flags or production undefined default');
        assert.equal(observed.destination,null,'setup: generic owned chest Search has no custom route destination');
      }catch(error){observed.failure=error;observed.error={name:error.name,message:error.message,stack:error.stack};throw error;}
    }
    return result;
  });
  t.mock.method(PM,'emitObjectInteraction',function(event,...args){
    if(!(motion.searchInputIssued&&!motion.firstSearchDone&&event.player===f.p))return Reflect.apply(interact,this,[event,...args]);
    // Capture the actual dispatcher position BEFORE the original handler can change anything.
    const position=where(f.p),source=event.sourceLocation?{...event.sourceLocation}:null;
    const dispatch={facts:{actor:f.p.getUsername(),at:position,sourceLocation:source,cycle:core.World.getProcessCycle(),
      object:objectFact(event.object),definitionId:event.definition?.id??null,clickType:event.clickType,option:event.option,
      inventoryBefore:inventories()},handled:false};
    firstSearchDispatches.push(dispatch);
    try{
      assert.equal(firstSearchDispatches.length,1,'one original first Search dispatcher');assert.equal(firstSearchRoutes.length,1);
      const effective=firstSearchRoutes[0];if(effective.failure)throw effective.failure;
      assert.equal(event.object,target);assert.equal(event.objectId,104);assert.equal(event.definition?.id,104);assert.equal(event.clickType,1);assert.equal(event.option,'search');
      assert.equal(event.object.getOwner(),f.p);assert.equal(event.object.getType(),10);
      assert.ok(source);assert.deepEqual(position,[source.x,source.y,source.z],'event-time sourceLocation is the real actor position');
      assert.equal(f.p.getArea(),area);assert.equal(f.p.getPrivateArea(),privateArea);assert.equal(f.p.getLocation().getZ(),start.getZ());
      assert.equal(f.p.hasFlag('movement:ignore-clipping'),false);assert.equal(f.p.getHitpoints(),hp);assert.deepEqual(inventories(),beforeMove,'no reward before real Search handler');
      standing(f.p,opened);standing(b.p,bOpened);
      const currentBase=core.ObjectDefinition.forId(target.getId()),currentResolved=core.ObjectDefinition.forPlayer(target.getId(),f.p);
      assert.equal(currentBase.getSizeX(),effective.width);assert.equal(currentBase.getSizeY(),effective.length);assert.equal(currentResolved.getBlockingMask(),effective.sideMask);
      assert.equal(target.getFace(),effective.face);assert.equal(effective.object,target);assert.equal(effective.destination,null);
      const reached=core.PathFinder.reachedObject(f.p,target.getLocation().getX(),target.getLocation().getY(),effective.width,effective.length,effective.face,effective.shape,effective.effectiveMask);
      dispatch.facts.reach=reached;dispatch.facts.effectiveRoute=routeFact(effective);
      assert.equal(reached,true,'real first Search dispatcher reaches actual104 with observed effective access mask');
      const result=Reflect.apply(interact,this,[event,...args]);
      dispatch.handled=event.handled;dispatch.facts.handledAtReturn=event.handled;
      return result;
    }catch(error){dispatch.failure=error;dispatch.facts.error={name:error.name,message:error.message,stack:error.stack};throw error;}
  });
  await f.input({type:'move',worldX:destination.getX(),worldY:destination.getY(),modifierFlags:0,run:false},0);
  while(!f.p.getLocation().equals(destination))advanceMotion('outgoing move');
  assert.ok(motion.turns>0&&motion.stepped,'ordinary queue actually stepped before exact outgoing arrival');
  assert.ok(motion.trace.some(position=>position.join(',')!==motion.trace[0].join(',')),'ordinary movement is not a no-op');
  const tuple=ownedReplays().find(row=>row.args[0]===f.p);assert.ok(tuple,'setup: actual PlayerSession-produced replay arguments');
  const beforeReplay=inventories(),sendStart=sends.length,opsStart=operated.length,eventsA=f.objectEvents.length,eventsB=b.objectEvents.length;
  Reflect.apply(core.ObjectManager.onRegionChange,core.ObjectManager,tuple.args);
  const replaySends=sends.slice(sendStart);assert.ok(replaySends.some(row=>row.player===f.p&&row.object===opened.get(first)),'actual own104 replay send(s), duplicates allowed');
  assert.equal(replaySends.some(row=>row.player===f.p&&row.object===bOpened.get(first)),false,'no foreign B104 sent to A');
  assert.deepEqual(inventories(),beforeReplay);assert.equal(operated.length,opsStart);assert.equal(f.objectEvents.length,eventsA);assert.equal(b.objectEvents.length,eventsB);
  standing(f.p,opened);standing(b.p,bOpened);
  observe('IKOV_CHEST_REPLAY',{actor:f.p.getUsername(),base:[tuple.args[1],tuple.args[2]],
    sends:replaySends.map(row=>({actor:row.player.getUsername(),object:objectFact(row.object)})),before:beforeReplay,after:inventories(),A:facts(opened),B:facts(bOpened)});
  // Preserve the original finite six-producer unforced search and final 1..5 assertion.
  for(const base of bases){
    if(base!==first)f.approach(base);
    if(!opened.has(base)){
      step=await action(f,base,103,'Open');assert.deepEqual(step.after,step.before);opened.set(base,lookup(f.p,base));assert.ok(opened.get(base));standing(f.p,opened);record('A Open',step);
    }
    const beforeCount=f.p.getInventory().getAmount(arrows),messages=f.messages.length;
    step=await action(f,base,104,'Search');const delta=f.p.getInventory().getAmount(arrows)-beforeCount;
    assert.deepEqual(step.after[1],step.before[1],'A search cannot change B inventory');
    if(delta===0){assert.deepEqual(step.after,step.before);assert.ok(f.messages.slice(messages).some(text=>text==='You search the chest, but find nothing.'));}
    else{assert.ok(delta>=1&&delta<=5);assert.deepEqual(step.after[0].filter(item=>item.id>0&&item.id!==arrows),step.before[0].filter(item=>item.id>0&&item.id!==arrows));assert.ok(f.messages.slice(messages).some(text=>text==='You found some ice arrows!'));}
    standing(f.p,opened);standing(b.p,bOpened);record('A unforced Search',step);
    if(f.p.getInventory().getAmount(arrows)>0)break;
  }
  const count=f.p.getInventory().getAmount(arrows);assert.ok(count>=1&&count<=5,`one containing chest supplies1..5 arrows, actual=${count}`);
  const beforeLogout=inventories(),bObjects=[...bOpened.values()];PM.emitPlayerLogout({player:b.p,username:b.p.getUsername()});
  for(const [base,object]of bOpened){
    assert.equal(lookup(b.p,base),null);assert.equal(core.ObjectManager.objectsAt(base.getLocation()).includes(object),false);
    const bt=ownedReplays().find(row=>row.args[0]===b.p);assert.ok(bt);assert.equal(core.MapObjects.ownerObjectsInScene(b.p,bt.args[1],bt.args[2],104).includes(object),false);
  }
  assert.deepEqual(inventories(),beforeLogout);standing(f.p,opened);shared();
  observe('IKOV_CHEST_LOGOUT',{actor:b.p.getUsername(),removed:bObjects.map(objectFact),before:beforeLogout,after:inventories(),A:facts(opened),scope:'fixture production logout hook; not network account logout'});
  bOpened.clear();
  for(const [base,object]of opened){
    f.approach(base);step=await action(f,base,104,'Shut',2);assert.equal(step.target,object);assert.deepEqual(step.after,step.before);
    assert.equal(lookup(f.p,base),null);assert.equal(core.ObjectManager.objectsAt(base.getLocation()).includes(object),false);
    const current=ownedReplays().find(row=>row.args[0]===f.p);assert.ok(current);assert.equal(core.MapObjects.ownerObjectsInScene(f.p,current.args[1],current.args[2],104).includes(object),false);
    assert.equal(core.MapObjects.getPrivateArea(f.p,103,base.getLocation()),base);opened.delete(base);shared();record('A Shut',step);
  }
});
test('first Winelda conversation without roots plays the introduction',async t=>{
  const f=actor(t,2654,9876); assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.LIMPWURT_ROOT),0);
  await f.talk(core.NpcIdentifiers.WINELDA,2655,9876);
  const line=f.p.getDialogueManager().getCurrent();assert.ok(line,'ordinary talk opens a real dialogue');
  assert.match(line.getText(),/Hehe! We see you're in a pickle!/);
});
test('wearing Lucien\'s pendant selects the hostile Guardian greeting',async t=>{
  const f=actor(t,2644,9897);f.p.getEquipment().setItem(core.Equipment.AMULET_SLOT,new core.Item(core.ItemIdentifiers.PENDANT_OF_LUCIEN,1));
  assert.equal(f.p.getEquipment().get(core.Equipment.AMULET_SLOT).getId(),core.ItemIdentifiers.PENDANT_OF_LUCIEN);
  await f.talk(core.NpcIdentifiers.GUARDIAN_OF_ARMADYL,2645,9897);
  const line=f.p.getDialogueManager().getCurrent();assert.ok(line);assert.match(line.getText(),/foul agent of Lucien/);
});
function restoreEncounterDoor(t) {
  const closed=object(93,2646,9870);
  t.after(()=>{if(!core.MapObjects.exists(closed))core.ObjectManager.register(closed,true);});
}
// Criterion correction after NATIVE001/reviewer015: face1 at y9870 blocks
// 9870->9871. Original immutable16 receipts remain failed and unchanged.
function gateWitness(label, actor, turn) {
  const p=actor.p;
  const leaves=[9870,9871].flatMap(y=>(core.MapObjects.mapObjects.get(core.MapObjects.getHash(2646,y,0))??[])
    .map(o=>({id:o.getId(),type:o.getType(),face:o.getFace(),at:where(o)})));
  const row={label,turn,at:where(p),registered:p.isRegistered(),placement:p.isNeedsPlacement(),
    queue:p.getMovementQueue().size(),ignoreClipping:p.hasFlag('movement:ignore-clipping'),
    stage:quest.getStage(p),leaves,clips:[9869,9870,9871,9872].map(y=>[2646,y,0,core.RegionManager.getClipping(2646,y,0,p.getPrivateArea())])};
  console.log('IKOV_GATE_EDGE',JSON.stringify(row));
  return row;
}
async function traceGateMove(label, actor) {
  const rows=[gateWitness(label,actor,'before-input')];
  await actor.input({type:'move',worldX:2646,worldY:9872,modifierFlags:0,run:false},0);
  for(let i=1;i<=8;i++){turns();rows.push(gateWitness(label,actor,i));}
  return rows;
}
test('encountering the Warrior cannot knock the owner through the unearned door',async t=>{
  restoreEncounterDoor(t);const a=actor(t,2646,9869);
  // Observe the real owner's local-NPC update data during the existing World turns.
  // This wrapper calls and returns the original producer unchanged; no flags,
  // text, actor views, emitters or network scheduling are injected.
  const session=a.p.getSession(),createActorUpdates=session.createActorUpdates,updates=[];
  assert.equal(typeof createActorUpdates,'function','native session exposes its compiled actor-update producer');
  t.mock.method(session,'createActorUpdates',function(...args){
    const view=Reflect.apply(createActorUpdates,this,args),npc=args[0];
    if(npc!==a.p&&npc.getId?.()===core.NpcIdentifiers.FIRE_WARRIOR_OF_LESARKUS&&npc.getOwner?.()===a.p){
      updates.push({npc,view:{...view},tick:core.World.getProcessCycle(),
        registered:npc.isRegistered()&&core.World.getNpcs().get(npc.getIndex())===npc,
        local:a.p.getLocalNpcs().includes(npc),ownerOnly:npc.isOwnerOnly(),
        samePlane:npc.getLocation().getZ()===a.p.getLocation().getZ(),
        sameArea:npc.getPrivateArea()===a.p.getPrivateArea()});
    }
    return view;
  });
  // Observe the original fixture channel boundary, including its normal batched flush.
  // Preserve every original call, argument and return; no extra tick/flush or packet.
  assert.deepEqual(session.npcSyncState.indices,[],'setup: owner has no pre-existing encoded local NPC state');
  assert.equal(a.p.getLocalNpcs().length,0,'setup: owner has no pre-existing local NPCs; never clear or seed them');
  const channel=session.channel,send=channel.send,sent=[];let sentBytes=0;
  assert.equal(typeof send,'function','the owner has its original native channel');
  t.mock.method(channel,'send',function(...args){
    assert.ok(Buffer.isBuffer(args[0]),'normal channel output is binary');
    const bytes=Buffer.from(args[0]);
    assert.ok(sent.length<64&&sentBytes+bytes.length<=1024*1024,'bounded original channel observations');
    const result=Reflect.apply(send,this,args);
    sent.push({bytes:bytes.toString('base64'),level:a.p.getLocation().getZ(),result});sentBytes+=bytes.length;
    return result;
  });
  await a.click(93,2646,9870,'Open',1,8);
  const warriors=[...core.World.getNpcs()].filter(n=>n.getId()===core.NpcIdentifiers.FIRE_WARRIOR_OF_LESARKUS&&n.getOwner()===a.p);
  assert.equal(warriors.length,1,'the ordinary input spawns exactly one owned quest encounter');
  const warrior=warriors[0];
  // Exact overhead lines from the qualified Temple of Ikov opening-door transcript.
  // Returned pre-encoding NPC update evidence is not rendered-overhead/pixel proof.
  for(const text of ['You will not pass!','Amitus! Setitii!']){
    const spoken=updates.filter(row=>row.view.forcedChat===text);
    assert.ok(spoken.length>0,`ordinary owner NPC updates contain exact overhead: ${text}`);
    assert.ok(spoken.every(row=>row.npc===warrior&&row.registered&&row.local&&row.ownerOnly&&row.samePlane&&row.sameArea),
      'overhead updates belong to the registered local owner-only Warrior in the owner plane and area');
    assert.equal(a.messages.some(message=>message.includes(text)),false,'intended overhead does not fall back to chatbox text');
  }
  // Reuse the existing client batch/NPC consumers without registering a TS loader
  // in this World process (the existing Mimic test uses the same child boundary).
  // These are ONLY bytes actually passed to this owner's original channel.send.
  assert.ok(sent.length>0,'the normal flush called the original owner channel');
  assert.ok(sent.every(row=>row.result===true),'the original fixture channel accepted each observed send');
  const childSource=String.raw`
    const assert=require('node:assert/strict'),fs=require('node:fs');
    const {decodeBatchedServerPackets}=require('../client/network/packet/ServerBinaryDecoder');
    const {NpcUpdateDecoder}=require('../client/game/sync/NpcUpdateDecoder');
    const input=JSON.parse(fs.readFileSync(0,'utf8')),decoder=new NpcUpdateDecoder(),types=new Map(),rows=[],spawns=[];
    assert.ok(input.sent.length>0&&input.sent.length<=64);
    let frames=0,total=0;
    for(const item of input.sent){
      const bytes=Buffer.from(item.bytes,'base64');total+=bytes.length;assert.ok(total<=1024*1024);
      const batch=decodeBatchedServerPackets(bytes);assert.equal(batch.stalled,false,'the existing client batch consumer accepts the actual complete send');
      for(const message of batch.messages){
        if(message.type!=='npc_info')continue;
        const payload=message.payload;
        // P9 is a same-plane main-world fixture; real root tile and flags come from the wire.
        // clientCycle is an unused-for-say consumer fixture parameter, not retail timing.
        const frame=decoder.decode(payload.packet,{large:payload.large,crossFloor:payload.crossFloor,
          loopCycle:payload.loopCycle,clientCycle:0,localTileX:payload.rootTileX,localTileY:payload.rootTileY,
          rootTileX:payload.rootTileX,rootTileY:payload.rootTileY,level:item.level});frames++;
        for(const index of frame.removals)types.delete(index);
        for(const spawn of frame.spawns){types.set(spawn.npcId,spawn.typeId);spawns.push({index:spawn.npcId,typeId:spawn.typeId,tick:payload.loopCycle});}
        for(const [index,block]of frame.updateBlocks){
          if(block.say===undefined)continue;
          assert.ok(frame.localNpcIds.includes(index),'decoded overhead belongs to a retained local NPC');
          assert.ok(types.has(index),'decoded overhead has a real decoded spawn/type association');
          rows.push({index,typeId:types.get(index),text:block.say,tick:payload.loopCycle});
        }
      }
    }
    assert.ok(frames>0,'actual channel output contains decoded NPC_INFO');
    process.stdout.write(JSON.stringify({frames,rows,spawns}));
  `;
  const {spawnSync}=require('node:child_process');
  const decoded=spawnSync(process.execPath,[require.resolve('ts-node/dist/bin.js'),'--transpile-only',
    '--compiler-options','{"target":"es2020","module":"commonjs"}','-e',childSource],
    {cwd:path.resolve(__dirname,'..'),input:JSON.stringify({sent}),encoding:'utf8',timeout:10000,maxBuffer:1024*1024});
  assert.ifError(decoded.error);assert.equal(decoded.signal,null,'existing protocol consumer child completes');
  assert.equal(decoded.status,0,`existing protocol consumer failed: ${decoded.stderr}`);
  assert.equal(decoded.stderr,'','protocol consumer emits no hidden warnings/errors');
  const wire=JSON.parse(decoded.stdout);
  assert.ok(wire.spawns.some(row=>row.index===warrior.getIndex()&&row.typeId===warrior.getId()),
    'the unchanged client decoder received the actual owned Warrior index and type on this channel');
  for(const text of ['You will not pass!','Amitus! Setitii!']){
    const encoded=wire.rows.filter(row=>row.text===text);
    assert.ok(encoded.length>0,`actual owner channel decodes exact NPC overhead: ${text}`);
    assert.ok(encoded.every(row=>row.index===warrior.getIndex()&&row.typeId===warrior.getId()&&
      updates.some(update=>update.npc===warrior&&update.view.forcedChat===text&&update.tick===row.tick&&
        update.registered&&update.local&&update.ownerOnly&&update.samePlane&&update.sameArea)),
      'encoded say is bound to the same real registered local owned Warrior and producer tick');
  }
  console.log('IKOV_WARRIOR_WIRE',JSON.stringify({layer:'owner-fixture-channel-send/existing-client-decoders',
    npcIndex:warrior.getIndex(),npcId:warrior.getId(),npcInfoFrames:wire.frames,
    batches:sent.map(row=>{const bytes=Buffer.from(row.bytes,'base64');return {bytes:bytes.length,
      sha256:require('node:crypto').createHash('sha256').update(bytes).digest('hex'),originalSendResult:row.result};}),
    overhead:wire.rows.filter(row=>row.index===warrior.getIndex()&&row.typeId===warrior.getId()&&
      ['You will not pass!','Amitus! Setitii!'].includes(row.text))}));
  assert.ok(a.p.getLocation().getY()<9870,'the encounter cannot knock the owner through the locked door');
});
test('a character credited with the Warrior kill can cross the same door from its south approach',async t=>{
  restoreEncounterDoor(t);const a=actor(t,2646,9869,60);
  await a.click(93,2646,9870,'Open',1,8);
  await traceGateMove('authorized-positive',a);
  assert.equal(a.p.getLocation().getX(),2646);assert.equal(a.p.getLocation().getZ(),0);
  assert.ok(a.p.getLocation().getY()>=9871,'positive route control: the same doorway is traversable when earned');
});
test('an independent bystander cannot walk orthogonally through another player\'s encounter gate',async t=>{
  restoreEncounterDoor(t);const a=actor(t,2646,9869),b=actor(t,2646,9868);
  await a.click(93,2646,9870,'Open',1,8);
  b.p.moveTo(new core.Location(2646,9869,0));turns(3);
  assert.equal(b.p.isNeedsPlacement(),false);assert.deepEqual(where(b.p),[2646,9869,0]);
  const trace=await traceGateMove('independent-bystander-corrected-edge',b);
  assert.ok(trace.every(row=>row.registered&&!row.placement&&!row.ignoreClipping),'bystander remains a settled ordinary actor without passage authorization');
  assert.ok(trace.every(row=>row.at[0]===2646&&row.at[2]===0),'bounded ordinary route remains on the tested column and plane');
  assert.ok(trace.some(row=>row.at[1]===9870),'positive approach witness: the bystander reaches the near-side edge, rather than being held by an unrelated obstruction');
  assert.ok(trace.every(row=>row.at[1]<9871),'a separate actor cannot cross the actual north edge9870->9871');
});
test('the registered completion reward preserves an acquired pair of boots and pays XP once',t=>{
  const f=actor(t,3122,3483,70),boots=core.ItemIdentifiers.BOOTS_OF_LIGHTNESS;f.add(boots);
  const sm=f.p.getSkillManager(),ranged=sm.getExperience(core.Skill.RANGED),fletching=sm.getExperience(core.Skill.FLETCHING);
  assert.equal(quest.complete(f.p,{scroll:false,announce:false}),true);
  assert.equal(f.p.getInventory().getAmount(boots),1,'completion must not grant another dungeon pickup');
  assert.equal(sm.getExperience(core.Skill.RANGED)-ranged,10500);assert.equal(sm.getExperience(core.Skill.FLETCHING)-fletching,8000);
  assert.equal(quest.complete(f.p,{scroll:false,announce:false}),false);assert.equal(f.p.getInventory().getAmount(boots),1);
  assert.equal(sm.getExperience(core.Skill.RANGED)-ranged,10500);assert.equal(sm.getExperience(core.Skill.FLETCHING)-fletching,8000);
});

// The observed sequence list contains his entrance teleport as well as combat.
// This checks the loaded server role, not the rendered size of an animated model.
test('the Fire Warrior does not use his smoke-puff entrance as a defence animation', () => {
  const core = PM.getCoreApi();
  const id = core.CacheDefinitions.getNpcId('ikov_firewarrior');
  assert.equal(id, 3448, 'the active cache resolves the quest actor');
  const entrance = core.CacheDefinitions.getSeqId('human_smokepuff_tele_appear');
  assert.equal(entrance, 803, 'the candidate is an entrance teleport, not a block');
  const warrior = core.NPC.create(id, new core.Location(2646, 9867, 0));
  assert.equal(warrior.getAttackAnim(), core.CacheDefinitions.getSeqId('human_caststrike'),
    'positive control: the observed spell remains his attack');
  assert.equal(warrior.getCurrentDefinition().getDeathAnim(), 836,
    'positive control: the observed death retains its role');
  assert.notEqual(warrior.getBlockAnim(), entrance, 'taking damage must not replay the entrance teleport');
  assert.equal(warrior.getBlockAnim(), core.CacheDefinitions.getSeqId('human_unarmedblock'),
    'an unclassified entrance must leave the human rig default intact');
});

test('the animation loader preserves explicitly configured creature combat roles', () => {
  const core = PM.getCoreApi();
  const mole = core.NpcDefinition.forId(5779);
  assert.deepEqual([mole.getAttackAnim(), mole.getDefenceAnim(), mole.getDeathAnim()], [3312, 3311, 3310]);
});

test('captured combat roles retain spawn-named attacks and defensive teleports',()=>{
  const spawn=core.NpcDefinition.forId(10),abyssal=core.NpcDefinition.forId(415);
  assert.equal(core.CacheDefinitions.getSeqName(1540),'demon_spawn_attack');
  assert.deepEqual([spawn.getAttackAnim(),spawn.getDefenceAnim(),spawn.getDeathAnim()],[1540,1542,1541],
    'a spawn word in an actual captured attack/parry/death does not erase its combat role');
  assert.equal(core.CacheDefinitions.getSeqName(2309),'abyssal_demon_teleport');
  assert.equal(abyssal.getDefenceAnim(),2309,'the capture-supported defensive blink remains a block');
});
test('an unnamed observed candidate still fills a missing role in a mixed list',t=>{
  // Inject only the legacy untyped observed-list input. The cache, resolver, configured
  // roles and resulting NpcDefinition are real; this is not a new live-capture claim.
  const unnamed=7145;
  assert.equal(core.CacheDefinitions.getSeqName(unnamed),null,'the active cache leaves this observed legacy candidate unnamed');
  const loader=new NpcDefinitionLoader(),read=loader.read.bind(loader);
  t.mock.method(loader,'read',(directory,name)=>name==='npc-animations.json'
    ? {'3448':[unnamed,803,711,836]} : read(directory,name));
  t.after(()=>{t.mock.restoreAll();new NpcDefinitionLoader().load();});
  assert.equal(loader.load(),true);
  const warrior=core.NpcDefinition.forId(3448);
  assert.equal(warrior.getAttackAnim(),711);assert.equal(warrior.getDeathAnim(),836);
  assert.equal(warrior.getDefenceAnim(),unnamed,'an unknown name is still eligible; a named entrance does not become a block');
});
