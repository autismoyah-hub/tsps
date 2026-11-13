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
  return {p,c,messages,input,click,add,talk,resume,approach,itemEvents};
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
  const f=actor(t,2653,9806,0);f.add(core.ItemIdentifiers.LIT_TORCH);
  const down=templeStairs(98,f.p);f.approach(down);
  await f.click(98,down.getLocation().getX(),down.getLocation().getY(),'Climb-down',1,8);
  assert.ok(f.p.getLocation().getY()>=9759&&f.p.getLocation().getY()<=9768,'the lit descend reaches the boots room');
  const up=templeStairs(96,f.p);f.approach(up);
  await f.click(96,up.getLocation().getX(),up.getLocation().getY(),'Climb-up',1,12);
  assert.ok(f.p.getLocation().getY()>=9803&&f.p.getLocation().getY()<=9810,'the climb returns to the main room');
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
  const f=actor(t,2710,9849); const arrows=core.ItemIdentifiers.ICE_ARROWS;assert.equal(f.p.getInventory().getAmount(arrows),0);
  for(const [x,y] of [[2710,9850],[2719,9838],[2729,9850],[2747,9848],[2738,9835],[2745,9821]]) {
    f.approach(object(103,x,y,f.p));
    await f.click(103,x,y,'Open');
    const location=new core.Location(x,y,0);
    const found=core.MapObjects.getPrivateArea(f.p,104,location)??core.MapObjects.getPrivateArea(f.p,103,location);assert.ok(found);
    if(found.getId()===104)await f.click(104,x,y,'Search');
    if(f.p.getInventory().getAmount(arrows)>0)break;
  }
  const count=f.p.getInventory().getAmount(arrows);assert.ok(count>=1&&count<=5,`one containing chest supplies1..5 arrows, actual=${count}`);
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
  await a.click(93,2646,9870,'Open',1,8);
  assert.ok([...core.World.getNpcs()].some(n=>n.getId()===core.NpcIdentifiers.FIRE_WARRIOR_OF_LESARKUS&&n.getOwner()===a.p),'the ordinary input spawns the quest encounter');
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
