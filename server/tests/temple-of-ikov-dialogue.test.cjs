// Native integration, after yarn build. Additional producer controls for #4834;
// separate from the immutable16-case BEFORE002 comparison. Supplied checkpoints
// and selected original transcript branches do not establish natural traversal.
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
const { PlayerSave } = require('../dist/game/entity/impl/player/persistence/PlayerSave');
const { MultiChatboxPrompt } = require('../dist/game/model/menu/MultiChatboxPrompt');
const { getRegisteredQuests } = require('../plugins/quests/QuestRuntime');
let core, quest, serial=0, submit, log, warn, error;
const tasks=new Set(), errors=[];
before(async()=>{
  log=console.log;warn=console.warn;error=console.error;
  console.log=(...args)=>{if(args.some(x=>x instanceof Error))errors.push(args.map(String).join(' '));log(...args);};
  console.warn=(...args)=>{if(/\[plugins\].*hook.*(?:threw|failed)/.test(args.map(String).join(' ')))errors.push(args.map(String).join(' '));warn(...args);};
  console.error=(...args)=>{errors.push(args.map(String).join(' '));error(...args);};
  await CachePipeline.initialize(path.resolve(__dirname,'..'));new NpcDefinitionLoader().load();
  PM.loadFromDirectory(path.resolve(__dirname,'../plugins'));core=PM.getCoreApi();
  submit=core.TaskManager.submit;
  core.TaskManager.submit=function(task,...args){tasks.add(task);return submit.call(this,task,...args);};
  core.RegionManager.init();
  for(const [x,y] of [[2573,3321],[3122,3483],[2645,9897]])core.RegionManager.loadMapFiles(x,y);
  PM.emitServerStartup({});quest=getRegisteredQuests().findLast(q=>q.key==='temple_of_ikov');assert.ok(quest);
});
after(()=>{
  try{if(core)PM.emitServerShutdown({});for(const task of tasks)if(task.isRunning())task.stop();assert.deepEqual(errors,[],'no swallowed engine or hook errors');}
  finally{console.log=log;console.warn=warn;console.error=error;if(core)core.TaskManager.submit=submit;}
});
function turns(n=1){for(let i=0;i<n;i++)core.World.process();}
function ownedNpcs(p){return [...new Set([...core.World.getNpcs(),...core.World.getAddNPCQueue()])].filter(n=>n.getOwner()===p);}
function disposeActor(p,spawned=[]){
  try{
    p.closeInterruptibleInterfaces();
    PM.emitPlayerDisconnect({player:p,username:p.getUsername()});PM.emitPlayerLogout({player:p,username:p.getUsername()});
    for(const npc of spawned)PM.removeNpc(npc);
    const remaining=ownedNpcs(p),ikovIds=new Set([INN,HOUSE,core.NpcIdentifiers.FIRE_WARRIOR_OF_LESARKUS]);
    console.log('IKOV_LOGOUT_BOUNDARY',JSON.stringify({username:p.getUsername(),registered:p.isRegistered(),
      remaining:remaining.map(n=>({id:n.getId(),index:n.getIndex(),registered:n.isRegistered(),lineage:n.getSpawnLineage?.()?.owner??null}))}));
    // Preserve the production quest cleanup oracle BEFORE removing unrelated
    // fixture-owned actors created by the full set of real login handlers.
    assert.equal(remaining.some(n=>ikovIds.has(n.getId())),false,'Ikov quest actors cleaned by production logout');
  }finally{
    // A failing oracle must still isolate the fixture for subsequent cases.
    // These are only this exact test player's actors, including queued adds.
    for(const npc of ownedNpcs(p))PM.removeNpc(npc);
    for(const item of [...core.World.getItems()])if(item.getOwner()===p.getUsername()){
      core.ItemOnGroundManager.deregister(item);const i=core.World.getItems().indexOf(item);if(i>=0)core.World.getItems().splice(i,1);
    }
    p.closeInterruptibleInterfaces();p.getMovementQueue().reset();p.getMovementQueue().walkToReset();
    core.TaskManager.cancelTasks(p);core.TaskManager.cancelTasks(p.getIndex());
    if(p.isRegistered())core.World.getPlayers().remove(p);
    assert.equal(p.isRegistered(),false);assert.equal(ownedNpcs(p).length,0,'all exact fixture-owned actors isolated');
    assert.equal(core.World.getItems().some(item=>item.getOwner()===p.getUsername()),false,'owned floor items isolated');
  }
}
function loginPlayer(p){
  assert.equal(p.isRegistered(),true);assert.equal(p.isPlayerBot(),false);
  const describe=()=>({username:p.getUsername(),registered:p.isRegistered(),placement:p.isNeedsPlacement(),
    version:p.getAttribute('temple-of-ikov:state-version'),side:p.getAttribute('temple-of-ikov:side'),
    legacySide:p.getAttribute('quest.temple_of_ikov.side'),
    owned:[...core.World.getNpcs()].filter(n=>n.getOwner()===p).map(n=>({id:n.getId(),index:n.getIndex(),registered:n.isRegistered()}))});
  console.log('IKOV_LOGIN_BOUNDARY',JSON.stringify({when:'before',...describe()}));
  // PluginManager's real registered wrapper requires BOTH player and username.
  // The old fixture omitted username, so it never entered quest login at all.
  PM.emitPlayerLogin({player:p,username:p.getUsername()});turns(4);
  console.log('IKOV_LOGIN_BOUNDARY',JSON.stringify({when:'after',...describe()}));
}
function actor(t,x=2573,y=3322,stage=10,{login=true,save=null,username=null}={}){
  const channel={kind:'test',remoteAddress:'127.0.0.1',isOpen:()=>true,send:()=>true,close(){},onData(){},onError(){},onClose(){}};
  const session=new PlayerSession(channel),p=new Player(session),c=new ClientConnection(channel);
  c.player=p;session.setPlayer(p);session.setTurnPacketProcessor(()=>c.processTurnPackets());
  p.setUsername(username??`ikovdialogue${++serial}`);p.setLongUsername(core.Misc.stringToLongBigInt(p.getUsername()));p.setAttribute('bot-skip-persistence',true);
  if(save)save.applyToPlayer(p);
  assert.equal(core.World.getPlayers().add(p),true);p.moveTo(new core.Location(x,y,0));if(!save)quest.setStage(p,stage);
  if(login)loginPlayer(p);else turns(4);
  assert.equal(p.isRegistered(),true);assert.equal(p.isNeedsPlacement(),false);assert.ok(p.getHitpoints()>0);
  const text=[],send=p.sendMessage.bind(p),sender=p.getPacketSender(),sendString=sender.sendString.bind(sender);
  p.sendMessage=value=>{text.push(String(value));return send(value);};
  sender.sendString=(value,...args)=>{text.push(String(value));return sendString(value,...args);};
  const spawned=[];
  t.after(()=>disposeActor(p,spawned));
  const input=async(packet,n=1)=>{await c.dispatch([packet]);turns(n);};
  function ready(npc){
    assert.ok(npc);turns(3);assert.equal(npc.isRegistered(),true);assert.equal(core.World.getNpcs().get(npc.getIndex()),npc);
    assert.ok(p.getLocalNpcs().includes(npc));assert.ok(p.getLocation().isWithinDistance(npc.getLocation(),3));return npc;
  }
  const own=id=>ready([...core.World.getNpcs()].find(n=>n.getOwner()===p&&n.getId()===id));
  const guardian=(x=2645,y=9897)=>{const n=PM.spawnNpc({id:core.NpcIdentifiers.GUARDIAN_OF_ARMADYL,x,y,z:0,owner:p,ownerOnly:true,wanderRadius:0});spawned.push(n);return ready(n);};
  const talk=async npc=>{ready(npc);const clickType=core.NpcDefinition.forId(npc.getId()).getActions().indexOf('Talk-to')+1;assert.ok(clickType>0);await input({type:'npc_option',index:npc.getIndex(),clickType},3);assert.ok(p.getDialogueManager().getCurrent());};
  const next=async()=>{const group=sender.getChatboxGroupId();assert.ok(group>=0);await input({type:'dialogue_continue',widgetId:(group<<16)|(group===229?4:5),childIndex:-1});};
  const drain=async()=>{for(let i=0;i<30&&p.getDialogueManager().getCurrent();i++){assert.equal(MultiChatboxPrompt.getPending(p),null,'selected branch has no unresolved menu');await next();}assert.equal(p.getDialogueManager().getCurrent(),undefined,'selected branch terminates');};
  function add(id,n=1){const before=p.getInventory().getAmount(id);p.getInventory().adds(id,n);assert.equal(p.getInventory().getAmount(id),before+n);}
  const choose=async text=>{
    for(let i=0;i<30&&!MultiChatboxPrompt.getPending(p)&&p.getDialogueManager().getCurrent();i++)await next();
    const menu=MultiChatboxPrompt.getPending(p);console.log('IKOV_MENU_BOUNDARY',JSON.stringify({wanted:text,menu}));
    assert.ok(menu,`source menu for ${text}`);const index=menu.options.findIndex(option=>option.includes(text));assert.ok(index>=0,`source option ${text}`);
    // Native interface action dispatch consumes a one-based option number.
    await input({type:'dialogue_continue',widgetId:MultiChatboxPrompt.OPTIONS_WIDGET_ID,childIndex:index+1});
  };
  return{p,text,input,own,guardian,talk,next,drain,add,choose};
}
function messageBranch(steps,id){
  if(!Array.isArray(steps))return null;
  const index=steps.findIndex(s=>s.id===id);
  if(index>=0){assert.ok(index>0,'actual source speech precedes the physical message');return steps.slice(index-1);}
  for(const s of steps){const found=messageBranch(s.steps,id);if(found)return found;for(const o of s.options??[]){const found=messageBranch(o.steps,id);if(found)return found;}}
  return null;
}
function stageMessage(f,npc,id,variant){
  // Real production playback of the unmodified imported source branch. It stops
  // on its preceding source line; native Continue commits the message afterward.
  const request={player:f.p,npc,npcId:npc.getId(),variant:{page:'Temple of Ikov',variant},exact:true,
    select:steps=>{const branch=messageBranch(steps,id);assert.ok(branch,`source message${id} exists`);return branch;}};
  PM.emitCustomEvent('npc-dialogue:start',request);assert.equal(request.handled,true);assert.ok(f.p.getDialogueManager().getCurrent());
}
const INN=13608,HOUSE=13607;
const HANDOVER='finishing-up-talking-to-lucien-near-varrock';
const GUARDIAN='inside-the-dungeon-talking-to-a-guardian-of-armadyl-while-not-wearing-the-pendant-of-lucien';
test('an unrelated NPC selector leaves every Ikov attribute byte unchanged',t=>{
  const f=actor(t,2573,3322,50,{login:false});
  const before=[...(f.p.attributes??new Map())].filter(([k])=>String(k).includes('ikov'));
  PM.emitNpcDialogueVariant({player:f.p,npcId:core.CacheDefinitions.getNpcId('fai_varrock_bluemoon_chef'),definition:null,npc:null});
  assert.deepEqual([...(f.p.attributes??new Map())].filter(([k])=>String(k).includes('ikov')),before);
});
for(const [name,tab,amount,replaced] of [['real positive bank',0,1,false],['zero placeholder',0,0,true],['stale search mirror',-1,1,true]])test(`lost-pendant Talk-to respects ${name}`,async t=>{
  const f=actor(t),id=core.ItemIdentifiers.PENDANT_OF_LUCIEN;
  f.p.getBank(tab<0?core.Bank.BANK_SEARCH_TAB_INDEX:tab).setItem(0,new core.Item(id,amount));
  await f.talk(f.own(INN));
  // The owned-positive path may finish in Lucien's ordinary choice menu; stop
  // there without choosing a different action. Lost paths end after the award.
  for(let i=0;i<16&&f.p.getDialogueManager().getCurrent()&&!MultiChatboxPrompt.getPending(f.p);i++)await f.next();
  assert.equal(f.p.getInventory().getAmount(id),replaced?1:0);
  assert.equal(f.text.some(s=>s.includes('Lucien has given you another pendant!')),replaced,'native message agrees with the physical replacement');
});
test('lost-pendant replacement preserves its source full-bag floor handout',async t=>{
  const f=actor(t),npc=f.own(INN),id=core.ItemIdentifiers.PENDANT_OF_LUCIEN;
  stageMessage(f,npc,'BH3ct0','starting-out-talking-to-lucien-again');
  for(let i=0;i<f.p.getInventory().capacity();i++)f.p.getInventory().setItem(i,new core.Item(core.ItemIdentifiers.BRONZE_DAGGER,1));
  await f.next();assert.equal(f.p.getInventory().getAmount(id),0);
  const floor=core.World.getItems().filter(item=>item.getOwner()===f.p.getUsername()&&item.getItem().getId()===id&&!item.isPendingRemoval());
  assert.equal(floor.length,1);assert.equal(floor[0].getItem().getAmount(),1);
  assert.ok(floor[0].getPosition().equals(f.p.getLocation()));
  assert.equal(f.text.some(s=>s.includes('Lucien has given you another pendant!')),true);assert.equal(f.p.getDialogueManager().getCurrent(),undefined);
});
for(const stale of ['staff','actor'])test(`staff message suppresses success after stale ${stale}`,async t=>{
  const f=actor(t,3122,3482,70),npc=f.own(HOUSE),id=core.ItemIdentifiers.STAFF_OF_ARMADYL;f.add(id);
  stageMessage(f,npc,'VBn1hb',HANDOVER);
  if(stale==='staff')f.p.getInventory().deleteNumber(id,1);else PM.removeNpc(npc);
  await f.next();assert.equal(quest.isComplete(f.p),false);assert.notEqual(f.p.getAttribute('temple-of-ikov:staff-delivered'),1);
  assert.equal(f.text.some(s=>/You give Lucien|Muhahahhahahaha|power of the staff/.test(s)),false);
  assert.ok(f.text.some(s=>/no longer|can no longer/.test(s)));assert.equal(f.p.getDialogueManager().getCurrent(),undefined);
});
test('Guardian message rechecks the worn pendant before its award and continuation',async t=>{
  const f=actor(t,2644,9897,60),npc=f.guardian();stageMessage(f,npc,'UvrBfr',GUARDIAN);
  f.p.getEquipment().setItem(core.Equipment.AMULET_SLOT,new core.Item(core.ItemIdentifiers.PENDANT_OF_LUCIEN,1));
  await f.next();assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.ARMADYL_PENDANT),0);
  assert.equal(f.text.some(s=>s.includes('The guardian has given you a pendant.')),false);assert.ok(f.text.some(s=>s.includes('cannot give you a pendant now')));
  assert.equal(f.p.getDialogueManager().getCurrent(),undefined);
});
test('physical staff delivery recovers idempotently and the Lucien ending pays once',async t=>{
  const f=actor(t,3122,3482,70),npc=f.own(HOUSE),staff=core.ItemIdentifiers.STAFF_OF_ARMADYL,boots=core.ItemIdentifiers.BOOTS_OF_LIGHTNESS;
  f.add(staff);f.add(boots);const skills=f.p.getSkillManager(),ranged=skills.getExperience(core.Skill.RANGED),fletching=skills.getExperience(core.Skill.FLETCHING);
  stageMessage(f,npc,'VBn1hb',HANDOVER);await f.next();assert.equal(f.p.getInventory().getAmount(staff),0);assert.equal(f.p.getAttribute('temple-of-ikov:staff-delivered'),1);
  assert.match(f.p.getDialogueManager().getCurrent().getText(),/Muhahahhahahaha/);
  f.p.closeInterruptibleInterfaces();stageMessage(f,npc,'VBn1hb',HANDOVER);await f.next();
  assert.equal(f.p.getInventory().getAmount(staff),0);assert.ok(f.text.some(s=>s.includes('already given Lucien')));await f.drain();
  assert.equal(quest.isComplete(f.p),true);assert.equal(f.p.getInventory().getAmount(boots),1);
  assert.equal(skills.getExperience(core.Skill.RANGED)-ranged,10500);assert.equal(skills.getExperience(core.Skill.FLETCHING)-fletching,8000);
  assert.equal(quest.complete(f.p,{scroll:false,announce:false}),false);assert.equal(skills.getExperience(core.Skill.RANGED)-ranged,10500);
});

function startMenu(steps){
  for(const step of steps){
    if(step.type==='choice'&&step.options?.some(option=>option.hook==='quest:temple-of-ikov:start'))return step;
    const nested=startMenu(step.steps??[]);if(nested)return nested;
    for(const option of step.options??[]){const found=startMenu(option.steps??[]);if(found)return found;}
  }
  return null;
}
for(const stale of ['removed actor','out-of-range actor'])test(`quest-start menu refuses a ${stale}`,async t=>{
  const f=actor(t,2573,3322,0),npc=f.own(INN);
  f.p.getSkillManager().setMaxLevel(core.Skill.THIEVING,42);f.p.getSkillManager().setCurrentLevel(core.Skill.THIEVING,42);
  f.p.getSkillManager().setMaxLevel(core.Skill.RANGED,40);f.p.getSkillManager().setCurrentLevel(core.Skill.RANGED,40);
  const playback={player:f.p,npc,npcId:npc.getId(),variant:{page:'Temple of Ikov',variant:'starting-out-talking-to-lucien'},exact:true,
    select:steps=>{const menu=startMenu(steps);assert.ok(menu);return [menu];}};
  PM.emitCustomEvent('npc-dialogue:start',playback);assert.equal(playback.handled,true);
  const menu=MultiChatboxPrompt.getPending(f.p);console.log('IKOV_MENU_BOUNDARY',JSON.stringify({wanted:'Yes.',stale,menu}));
  assert.ok(menu);const yes=menu.options.indexOf('Yes.');assert.ok(yes>=0);
  if(stale==='removed actor')PM.removeNpc(npc);else{npc.moveTo(new core.Location(2583,3321,0));turns(3);}
  await f.input({type:'dialogue_continue',widgetId:MultiChatboxPrompt.OPTIONS_WIDGET_ID,childIndex:yes+1});
  assert.equal(quest.getStage(f.p),0);assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.PENDANT_OF_LUCIEN),0);
  assert.ok(f.text.some(s=>s.includes('can no longer accept')));assert.equal(f.p.getDialogueManager().getCurrent(),undefined);
  assert.equal(f.text.some(s=>s.includes('Here is my pendant')),false,'no stale source success speech');
});

function lethalNativeHit(f,npc){
  // Supply the lethal damage amount at the native hit-queue boundary; the real
  // World/NPC death task must choose the recorded killer and emit its event.
  // This proves death/ending dispatch, not an ordinary combat accuracy roll.
  const hp=npc.getHitpoints();assert.ok(hp>0&&npc.isRegistered());npc.getCombat().addDamage(f.p,hp);
  npc.getCombat().getHitQueue().addPendingDamage([new core.HitDamage(hp,core.HitMask.RED)]);
  for(let i=0;i<24&&npc.isRegistered();i++)turns();
  assert.equal(npc.isRegistered(),false,'native death/removal completed');
}
function snapshotAndLogout(f,expected){
  if(expected===undefined){
    const save=PlayerSave.fromPlayer(f.p);save.attributes=JSON.parse(JSON.stringify(save.attributes));
    disposeActor(f.p);
    return save;
  }
  const {side,delivered,defeated}=expected;
  // Same production codec seam as run-energy-save-round-trip.test.cjs. Lazy
  // registration occurs only in D14/D15, after the existing suite setup.
  const Json=require('../plugins/persistence/JsonPlayerPersistence.plugin');
  const backend=core.GameConstants.PLAYER_PERSISTENCE;
  let codec,registrations=0;
  Json.register({getSkillManager:()=>core.SkillManager,setPlayerPersistence(value){codec=value;registrations++;},log(){}});
  assert.equal(registrations,1,'setup: production JSON codec registered without a host override');
  assert.ok(codec instanceof Json.JsonPlayerPersistence,'setup: actual exported production codec');
  assert.equal(core.GameConstants.PLAYER_PERSISTENCE,backend,'captured API never installs a backend');
  const inventory=JSON.parse(JSON.stringify(f.p.getInventory().getItems(),codec.replacer.bind(codec)));
  const equipment=JSON.parse(JSON.stringify(f.p.getEquipment().getItems(),codec.replacer.bind(codec)));
  const save=PlayerSave.fromPlayer(f.p);
  const serialized=JSON.stringify(save,codec.replacer.bind(codec));
  const parsed=JSON.parse(serialized,codec.reviver.bind(codec));
  const restored=codec.hydratePlayerSave(parsed);
  assert.notEqual(restored,save,'hydrate a new save from serialized bytes');
  assert.ok(restored instanceof PlayerSave,'compiled PlayerSave identity matches the suite');
  assert.notEqual(restored.attributes,save.attributes);
  assert.notEqual(restored.inventory,save.inventory);
  const canonical={'quest.temple_of_ikov.stage':70,'temple-of-ikov:state-version':1,
    'temple-of-ikov:side':side,'temple-of-ikov:staff-delivered':delivered,'temple-of-ikov:lucien-defeated':defeated};
  for(const attributes of [parsed.attributes,restored.attributes]){
    assert.ok(attributes&&typeof attributes==='object'&&!Array.isArray(attributes));
    for(const [key,value] of Object.entries(canonical)){
      if(value===0)assert.equal(Object.hasOwn(attributes,key),false,`${key}: production omits registered default zero`);
      else assert.equal(attributes[key],value,`${key}: canonical serialized/hydrated checkpoint`);
    }
  }
  assert.deepEqual(restored.attributes,parsed.attributes);
  assert.equal(parsed.inventory.length,28);assert.equal(parsed.equipment.length,14);
  assert.deepEqual(parsed.inventory,inventory);assert.deepEqual(parsed.equipment,equipment);
  assert.ok(restored.inventory.every(item=>item instanceof core.Item));
  assert.ok(restored.equipment.every(item=>item instanceof core.Item));
  assert.deepEqual(JSON.parse(JSON.stringify(restored.inventory,codec.replacer.bind(codec))),inventory);
  assert.deepEqual(JSON.parse(JSON.stringify(restored.equipment,codec.replacer.bind(codec))),equipment);
  assert.equal(core.GameConstants.PLAYER_PERSISTENCE,backend,'codec leaves the shared backend unchanged');
  // Keep the shared parent backend/path/host untouched: only this bounded child owns a file store.
  const fs=require('node:fs'),os=require('node:os'),crypto=require('node:crypto');
  const {spawnSync}=require('node:child_process');
  const parentDirectory=Json.JsonPlayerPersistence.SAVE_DIRECTORY;
  assert.ok(f.p.getEquipment().get(core.Equipment.WEAPON_SLOT).getId()<=0,'actual original checkpoint is unarmed');
  assert.equal(f.p.getProfile(),Json.JsonPlayerPersistence.PROFILES.MAIN_PROFILE);
  const root=fs.mkdtempSync(path.join(os.tmpdir(),'ikov-json-store-'));
  let ownedRoot,reopened,observation,failure,childResult;
  let producerObservation,producerFailureEvidence,producerChildrenTerminal=true,producerStagingSafe=true;
  const producerRaw=[];let proofDeadline,proofBudgetUsed=0;
  const proofLimit=1024*1024,proofReserve=8192;
  function chargeProof(value,label){const n=Buffer.isBuffer(value)?value.length:Buffer.byteLength(String(value),'utf8');assert.ok(proofBudgetUsed+n<=proofLimit-proofReserve,'aggregate FILE helper 1MiB: '+label);proofBudgetUsed+=n;}
  function remainingChildTime(){assert.ok(proofDeadline!==undefined);const ms=Number((proofDeadline-process.hrtime.bigint())/1000000n);assert.ok(ms>0,'original shared10000ms helper deadline');return ms;}
  function remainingChildBuffer(){const n=Math.floor((proofLimit-proofReserve-proofBudgetUsed)/4);assert.ok(n>1024,'aggregate child raw/encoded capacity');return n;}
  function retainChildResult(role,result){
    const bytes=b=>Buffer.isBuffer(b)?b:Buffer.alloc(0),stdout=bytes(result.stdout),stderr=bytes(result.stderr);
    const record={role,pid:result.pid??null,status:result.status,signal:result.signal,
      error:result.error?{name:result.error.name,code:result.error.code,message:result.error.message}:null,
      stdoutBytes:stdout.length,stderrBytes:stderr.length,stdoutSHA256:crypto.createHash('sha256').update(stdout).digest('hex'),
      stderrSHA256:crypto.createHash('sha256').update(stderr).digest('hex'),stdoutBase64:stdout.toString('base64'),stderrBase64:stderr.toString('base64'),rawComplete:true};
    record.rawComplete=Buffer.isBuffer(result.stdout)&&Buffer.isBuffer(result.stderr)&&!result.error&&result.signal===null;
    record.retentionScope=record.rawComplete?'complete returned child stream':'returned buffers only; child stream may be incomplete';
    if(!Buffer.isBuffer(result.stdout)){record.stdoutBytes=null;record.stdoutSHA256=null;record.stdoutBase64=null;}
    if(!Buffer.isBuffer(result.stderr)){record.stderrBytes=null;record.stderrSHA256=null;record.stderrBase64=null;}
    const encoded=JSON.stringify(record);
    if(Buffer.byteLength(encoded)>proofLimit-proofReserve-proofBudgetUsed){
      record.stdoutBase64=null;record.stderrBase64=null;record.rawComplete=false;record.unretainedRawBytes=stdout.length+stderr.length;
      producerFailureEvidence={reason:'encoded child artifact cap',partial:record};
      throw new Error('aggregate encoded child output exceeds unchanged helper cap');
    }
    chargeProof(encoded,'retained raw child result');return record;
  }
  function reportProducerFailure(error){
    const facts=producerFailureEvidence??{rawChildren:producerRaw,childrenTerminal:producerChildrenTerminal,stagingSafe:producerStagingSafe,ownedRoot,usedBytes:proofBudgetUsed};
    facts.error={name:error.name,message:error.message};
    let raw=JSON.stringify(facts);if(Buffer.byteLength(raw)>proofLimit-proofReserve)raw=JSON.stringify({reason:'failure evidence cap',rawComplete:false,unretainedJSONBytes:Buffer.byteLength(raw),partial:facts.partial??null,ownedRoot,childrenTerminal:producerChildrenTerminal,stagingSafe:producerStagingSafe});
    console.log('IKOV_JSON_PRODUCER_FAILURE',raw);
  }
  try{
    ownedRoot=fs.realpathSync(root);
    const childSource=String.raw`
    const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
    require('./dist/Server').Server.installProductionPathResolver();
    const Json=require('./plugins/persistence/JsonPlayerPersistence.plugin');
    const {SkillManager}=require('./dist/game/content/skill/SkillManager');
    const {PlayerSave}=require('./dist/game/entity/impl/player/persistence/PlayerSave');
    const {Item}=require('./dist/game/model/Item');
    const {Equipment}=require('./dist/game/model/container/impl/Equipment');
    const {GameConstants}=require('./dist/game/GameConstants');
    (async()=>{
      const request=fs.readFileSync(0,'utf8');assert.ok(Buffer.byteLength(request,'utf8')<=1024*1024);
      const input=JSON.parse(request);
      assert.equal(typeof input.username,'string');assert.ok(input.username.length>0);
      assert.equal(input.profile,Json.JsonPlayerPersistence.PROFILES.MAIN_PROFILE,'original main-profile fixture');
      assert.ok(path.isAbsolute(input.root));
      const root=fs.realpathSync(input.root),rootStat=fs.lstatSync(input.root);
      assert.equal(root,input.root);assert.ok(rootStat.isDirectory()&&!rootStat.isSymbolicLink());
      // Process-local configuration before any owned backend exists, as in player-saves.test.cjs.
      Json.JsonPlayerPersistence.SAVE_DIRECTORY=root;
      const backend=GameConstants.PLAYER_PERSISTENCE;
      let writer,reader,registrations=0,proof,failure;
      try{
        Json.register({getSkillManager:()=>SkillManager,setPlayerPersistence(value){writer=value;registrations++;},log(){}});
        assert.equal(registrations,1,'actual JSON registration without host override');
        assert.ok(writer instanceof Json.JsonPlayerPersistence);
        assert.equal(GameConstants.PLAYER_PERSISTENCE,backend,'captured child registration never installs a backend');
        const original=JSON.parse(input.serialized,writer.reviver.bind(writer));
        assert.ok(original.equipment[Equipment.WEAPON_SLOT].id<=0,'actual original checkpoint is unarmed; no second cache');
        const snapshot=writer.hydratePlayerSave(original);
        assert.ok(snapshot instanceof PlayerSave);
        assert.ok(snapshot.inventory.every(item=>item instanceof Item));
        assert.ok(snapshot.equipment.every(item=>item instanceof Item));
        assert.equal(writer.pendingSerializedSaves.size,0);assert.equal(writer.failedWrites.size,0);
        const filePath=writer.resolveFilePath(input.username,input.profile);
        assert.equal(path.dirname(filePath),root,'production main-profile path remains in owned root');
        assert.equal(fs.existsSync(filePath),false,'fresh private store has no previous account file');
        writer.saveOffline(input.username,snapshot,'ikov-fixture',input.profile);
        await writer.flush();
        assert.equal(writer.pendingSerializedSaves.size,0,'flush committed rather than retaining pending JSON');
        assert.equal(writer.failedWrites.size,0);
        const stat=fs.lstatSync(filePath);
        assert.ok(stat.isFile()&&!stat.isSymbolicLink());assert.ok(stat.size>0&&stat.size<=1024*1024);
        const file=fs.readFileSync(filePath);assert.equal(file.length,stat.size);
        const disk=JSON.parse(file.toString('utf8'),writer.reviver.bind(writer));
        for(const key of ['attributes','position','inventory','equipment','skills'])assert.deepEqual(disk[key],original[key],key+' committed from original snapshot');
        reader=new Json.JsonPlayerPersistence();
        assert.notEqual(reader,writer);assert.ok(reader instanceof Json.JsonPlayerPersistence);
        assert.equal(reader.pendingSerializedSaves.size,0);assert.equal(reader.failedWrites.size,0);assert.equal(reader.nextWriteSequence,0);
        assert.equal(reader.load,Json.JsonPlayerPersistence.prototype.load,'original production filesystem loader');
        assert.equal(reader.resolveFilePath(input.username,input.profile),filePath);
        const loaded=reader.load(input.username,input.profile);
        assert.ok(loaded instanceof PlayerSave);assert.notEqual(loaded,snapshot);
        assert.equal(reader.pendingSerializedSaves.size,0);assert.equal(reader.nextWriteSequence,0,'fresh loader never queued a write');
        const loadedJSON=JSON.stringify(loaded,reader.replacer.bind(reader));
        const loadedData=JSON.parse(loadedJSON,reader.reviver.bind(reader));
        for(const key of ['attributes','position','inventory','equipment','skills'])assert.deepEqual(loadedData[key],disk[key],key+' cold-loaded from committed file');
        assert.equal(GameConstants.PLAYER_PERSISTENCE,backend);
        proof={profile:input.profile,normalizedUsername:writer.normalizeUsername(input.username),filePath,
          fileBytes:file.length,fileSHA256:crypto.createHash('sha256').update(file).digest('hex'),fileBase64:file.toString('base64'),
          loadedJSON,writerPending:writer.pendingSerializedSaves.size,writerFailed:writer.failedWrites.size,
          readerPending:reader.pendingSerializedSaves.size,readerFailed:reader.failedWrites.size,readerWrites:reader.nextWriteSequence};
      }catch(error){failure=error;}
      finally{
        for(const store of [writer,reader])if(store){
          try{await store.flush();}catch(error){failure=failure?new AggregateError([failure,error],'store proof and final flush failed'):error;}
        }
      }
      if(failure)throw failure;
      const output=JSON.stringify(proof);assert.ok(Buffer.byteLength(output,'utf8')<=1024*1024);
      process.stdout.write(output);
    })().catch(error=>{console.error(error);process.exitCode=1;});
`;
    proofDeadline=process.hrtime.bigint()+10000000000n;
    const request=JSON.stringify({root:ownedRoot,username:f.p.getUsername(),profile:f.p.getProfile(),serialized});
    assert.ok(Buffer.byteLength(request,'utf8')<=1024*1024);
    chargeProof(childSource,'original static child argv');chargeProof(request,'original input');
    const result=childResult=spawnSync(process.execPath,['-e',childSource],{cwd:path.resolve(__dirname,'..'),
      input:request,timeout:remainingChildTime(),maxBuffer:remainingChildBuffer()});
    const originalRaw=retainChildResult('original-json70',result);producerRaw.push(originalRaw);chargeProof(JSON.stringify(originalRaw),'original failure-record reservation');
    assert.ifError(result.error);assert.equal(result.signal,null,'store child exits before owned-root cleanup');
    assert.equal(result.status,0,`production store child failed: ${result.stderr}`);
    assert.ok(Buffer.isBuffer(result.stdout)&&Buffer.isBuffer(result.stderr));
    assert.equal(result.stderr.length,0,'store child has no hidden import/write/flush errors');
    const text=result.stdout.toString('utf8');assert.deepEqual(Buffer.from(text,'utf8'),result.stdout,'exact UTF-8 child output');
    const proof=JSON.parse(text);
    assert.equal(proof.profile,f.p.getProfile());assert.equal(proof.normalizedUsername,codec.normalizeUsername(f.p.getUsername()));
    assert.equal(path.dirname(proof.filePath),ownedRoot);
    for(const key of ['writerPending','writerFailed','readerPending','readerFailed','readerWrites'])assert.equal(proof[key],0,key);
    const stat=fs.lstatSync(proof.filePath);assert.ok(stat.isFile()&&!stat.isSymbolicLink());
    assert.ok(stat.size>0&&stat.size<=1024*1024);
    const file=fs.readFileSync(proof.filePath),returned=Buffer.from(proof.fileBase64,'base64');
    assert.equal(returned.toString('base64'),proof.fileBase64);assert.deepEqual(file,returned,'actual parent file read equals child bytes');
    assert.equal(file.length,proof.fileBytes);assert.equal(file.length,stat.size);
    assert.equal(crypto.createHash('sha256').update(file).digest('hex'),proof.fileSHA256);
    const stored=JSON.parse(file.toString('utf8'),codec.reviver.bind(codec));
    for(const key of ['attributes','position','inventory','equipment','skills'])assert.deepEqual(stored[key],parsed[key],key+' stored original checkpoint');
    const loaded=JSON.parse(proof.loadedJSON,codec.reviver.bind(codec));
    for(const key of ['attributes','position','inventory','equipment','skills'])assert.deepEqual(loaded[key],stored[key],key+' returned by fresh filesystem load');
    reopened=codec.hydratePlayerSave(loaded);
    assert.ok(reopened instanceof PlayerSave);assert.notEqual(reopened,save);assert.notEqual(reopened,restored);
    assert.deepEqual(reopened.attributes,restored.attributes);
    assert.ok(reopened.inventory.every(item=>item instanceof core.Item));assert.ok(reopened.equipment.every(item=>item instanceof core.Item));
    assert.deepEqual(JSON.parse(JSON.stringify(reopened.inventory,codec.replacer.bind(codec))),inventory);
    assert.deepEqual(JSON.parse(JSON.stringify(reopened.equipment,codec.replacer.bind(codec))),equipment);
    assert.deepEqual(JSON.parse(JSON.stringify(reopened.position)),parsed.position,'verify loaded position BEFORE fixture movement');
    // FILE80 is a direct producer fixture; the original FILE70 proof/return stays separate.
    const sourceBase='0d84f10f124550b511a2c622bbfcf52232a4a719';
    const candidatePins=[{"rel":"areas/TutorialIsland.plugin.js","bytes":2159,"sha256":"2636b702695e322ce6f618f0707d9d8774ca012488ce972f84815f279adbe394","gitBlob":"a6098c6980dbefe007e694bb613c7c23f448278d"},{"rel":"bosses/Zulrah.plugin.js","bytes":619,"sha256":"813e22d060357d8cea4b2c1fc58fd37bd2dc41cd8ad40c86d6c866a5a3a003f9","gitBlob":"1c9c83eca4c023a3102c5a26c76f63daf57286c4"},{"rel":"interface/Settings.plugin.js","bytes":64343,"sha256":"352a171b562c142cd8fd3cf791a43e54e4ef71eb89a240913ba4f22577458543","gitBlob":"b568b24a2136c1f6d6b889aa606696b715791305"},{"rel":"items/DiaryRewards.plugin.js","bytes":1352,"sha256":"717dc33208a3f68891f25e3c6f680e8dbc346d1d4f60c23d73cc6afacd8cfd40","gitBlob":"c75043c758941b88efa8306ae86778a96b0950ad"},{"rel":"items/Potions.plugin.js","bytes":115878,"sha256":"4334d85381dea3271687a2e5faeed5ba4d0540b4f0af4d77628fb331773cd3c4","gitBlob":"7b6c54ccd4dca35e42a168591c88c74d95116889"},{"rel":"minigames/GolemCrafting.plugin.js","bytes":244,"sha256":"f0001f6d0ed5eb760caf831dd7ad119b272b90750dbfabebfb13fbbd6e0a9c4a","gitBlob":"fdf8438b217301d49628abe478a8667c161e34e6"},{"rel":"minigames/MasteringMixology.plugin.js","bytes":1145,"sha256":"7130ca50625e8a53c3aa9d3a3d445fdccf5b75dd31ce7de78eb207f73240196b","gitBlob":"07023c59bd5e252ddaf992975448d1d7da6681d0"},{"rel":"minigames/ValeTotems.plugin.js","bytes":1155,"sha256":"badc59885b38b31151c25bcdc84e787bd7e294ea8815d587b755d7236df8ca48","gitBlob":"322afdfb630f6b923d5c93ba2b5b520bf7da1193"},{"rel":"npcs/BarbarianTraining.plugin.js","bytes":19098,"sha256":"84f312bf54260606867bc642509e1c25df3fe267eff81a42fe34e5626f38e90b","gitBlob":"7d9463b40ec8ec3349c8336a6ffca5f714c8bfd6"},{"rel":"npcs/NpcDialogues.plugin.js","bytes":73841,"sha256":"3c6fc47d9422d0243d92f6a840183ecf01ca3c7db60f94f187e71e9222717d84","gitBlob":"d453ab4ea391ef09ea971e91bb375404ef869fd2"},{"rel":"npcs/RandomEvents.plugin.js","bytes":2870,"sha256":"bc1b87a927adfdc1cfbcf146574598f3b8dc6f64535ab3c68868441eecacc0d5","gitBlob":"26ae30aacc5bc86fb4347ae158245cd7542da6d0"},{"rel":"objects/Windmill.plugin.js","bytes":9490,"sha256":"0ff8dc0c7a70192bc4542ad2bff4995f08619e33aa529a15bf5635e45c2a718b","gitBlob":"91a5fd758e5ff55321bcb2671be0903773e29c24"},{"rel":"persistence/JsonPlayerPersistence.plugin.js","bytes":20618,"sha256":"dc738d0ef6e4e2c35e0bc3bd73f9b1070bb71421209ec8088e950fa995948d63","gitBlob":"d1ccbe3bbc743e14e38fe6a9bf610cc531e1085c"},{"rel":"persistence/SaveEvents.plugin.js","bytes":2034,"sha256":"5659056d0f57b63e6f6e4ddb5c53a5950a6368b931cff3ca23f27d6de398e3a1","gitBlob":"f16a657e05186ff9c2f73d3cec6cf70333629a90"},{"rel":"quests/Quests.plugin.js","bytes":8549,"sha256":"05f211a639939d834377f877879307418f6e3cbe58a4f14f3f4d79afefc7f0ce","gitBlob":"3190c41ed59bcfa259f420662bebdca0d9b0b8f0"},{"rel":"skills/Farming.plugin.js","bytes":481,"sha256":"8d4095b8879e33fd5d95d2f7e5d1149946ebd9b632321326a373a150789202a4","gitBlob":"efbebaff34ce6950cf61fa1d006fd88006b883e9"},{"rel":"treasuretrails/TreasureTrails.plugin.js","bytes":774,"sha256":"b20ebe1d3e9897eebf4ac142d047d0ade0c7f97d4d932ccc5471994b7106685a","gitBlob":"92b542b34ae6c9c545231b5ee85d62f98c03416c"},{"rel":"world/Music.plugin.js","bytes":33538,"sha256":"5a58b20de51821cd2ef7f1542007119e251bfc9facf235b7328e47e641745ccd","gitBlob":"4e959cf4aaaf19acb113af0cbb0d503d047cfdcc"}];
    const writerSource=String.raw`'use strict';
const ROLE='writer';
'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const inputBytes=fs.readFileSync(0);assert.ok(inputBytes.length>0&&inputBytes.length<=1024*1024);
const input=JSON.parse(inputBytes.toString('utf8'));
assert.equal(input.role,ROLE);assert.match(input.nonce,/^[0-9a-f]{32}$/);
function identity(){
  const raw=fs.readFileSync('/proc/self/stat','utf8'),v=raw.slice(raw.lastIndexOf(')')+2).trim().split(/\s+/);
  const children=new Set();for(const tid of fs.readdirSync('/proc/self/task')){
    try{for(const pid of fs.readFileSync('/proc/self/task/'+tid+'/children','utf8').trim().split(/\s+/))if(pid)children.add(Number(pid));}
    catch(e){if(e.code!=='ENOENT')throw e;}
  }
  return{pid:process.pid,ppid:Number(v[1]),pgrp:Number(v[2]),session:Number(v[3]),startTicks:v[19],children:[...children].sort((a,b)=>a-b)};
}
const initialOwner=identity();assert.deepEqual(initialOwner.children,[]);
fs.writeSync(1,'IKOV_FILE80_OWNER '+JSON.stringify({role:ROLE,nonce:input.nonce,source:input.source,owner:initialOwner})+'\n');
function directory(p){assert.ok(path.isAbsolute(p));const s=fs.lstatSync(p);assert.ok(s.isDirectory()&&!s.isSymbolicLink());assert.equal(fs.realpathSync(p),p);}
function regular(p){const s=fs.lstatSync(p);assert.ok(s.isFile()&&!s.isSymbolicLink());return s;}
directory(input.root);directory(input.store);directory(input.stage);
assert.equal(path.dirname(input.store),input.root);assert.equal(path.dirname(input.stage),input.root);
assert.equal(fs.realpathSync(process.cwd()),input.serverRoot);
assert.deepEqual(JSON.parse(fs.readFileSync(path.join(input.serverRoot,'data/definitions/world.json'),'utf8')).plugins,{TutorialIsland:false});
assert.equal(fs.existsSync(path.join(input.serverRoot,'data/definitions/world.local.json')),false);
assert.notEqual(process.env.BROWSER_HOST,'1');
const options=process.execArgv.filter((v,i,a)=>i===0||a[i-1]!=='-e').join(' ')+' '+(process.env.NODE_OPTIONS||'');
assert.ok(!/(?:^|[\s"'])--(?:preserve-symlinks(?:-main)?|require|import)(?:[=\s"']|$)/.test(options),'unqualified module-resolution/preload option');
const e=process.execArgv.indexOf('-e');assert.ok(e>=0);assert.equal(sha(Buffer.from(process.execArgv[e+1])),input.programSHA256);
const testFile=path.join(input.serverRoot,'tests/temple-of-ikov-dialogue.test.cjs');regular(testFile);
assert.equal(sha(fs.readFileSync(testFile)),input.source.testSHA256);
assert.equal(input.source.reviewedBase,'0d84f10f124550b511a2c622bbfcf52232a4a719');
assert.ok(Array.isArray(input.pins)&&input.pins.length===(ROLE==='writer'?18:2));
const stagedFiles=[],stagedDirs=[];function walkStage(dir,rel){
  for(const name of fs.readdirSync(dir).sort()){
    const child=path.join(dir,name),key=rel?rel+'/'+name:name,st=fs.lstatSync(child);
    if(st.isDirectory()&&!st.isSymbolicLink()){assert.equal(fs.realpathSync(child),child);stagedDirs.push(key);walkStage(child,key);}
    else{assert.ok(st.isSymbolicLink(),'only exact production FILE links');stagedFiles.push(key);}
  }
}
walkStage(input.stage,'');assert.deepEqual(stagedFiles.sort(),input.pins.map(p=>p.rel).sort());
const expectedDirs=new Set();for(const pin of input.pins){let dir=path.posix.dirname(pin.rel);while(dir!=='.'){expectedDirs.add(dir);dir=path.posix.dirname(dir);}}
assert.deepEqual(stagedDirs.sort(),[...expectedDirs].sort());
const resolutions=[];for(const pin of input.pins){
  assert.ok(!path.isAbsolute(pin.rel)&&!pin.rel.split('/').includes('..'));
  const link=path.join(input.stage,pin.rel),target=path.join(input.serverRoot,'plugins',pin.rel);
  assert.ok(fs.lstatSync(link).isSymbolicLink());assert.equal(fs.readlinkSync(link),target);
  assert.equal(fs.realpathSync(link),target);regular(target);
  const bytes=fs.readFileSync(target);assert.equal(bytes.length,pin.bytes);assert.equal(sha(bytes),pin.sha256);
  assert.equal(require.resolve(link),target);
  resolutions.push({rel:pin.rel,target:target,bytes:bytes.length,sha256:sha(bytes),resolved:require.resolve(link)});
}
require('./dist/Server').Server.installProductionPathResolver();
const Json=require('./plugins/persistence/JsonPlayerPersistence.plugin');
Json.JsonPlayerPersistence.SAVE_DIRECTORY=input.store;
const {PluginManager:PM}=require('./dist/plugins/PluginManager');
const {GameConstants}=require('./dist/game/GameConstants');
const {PlayerSave}=require('./dist/game/entity/impl/player/persistence/PlayerSave');
function data(save){return JSON.parse(JSON.stringify(save,backend.replacer.bind(backend)),backend.reviver.bind(backend));}
function selected(save){const o=data(save);return{attributes:o.attributes,inventory:o.inventory,equipment:o.equipment,skills:o.skills,position:o.position};}
function idle(){assert.equal(backend.pendingSerializedSaves.size,0);assert.equal(backend.failedWrites.size,0);assert.equal(GameConstants.PLAYER_PERSISTENCE,backend);assert.equal(Json.JsonPlayerPersistence.SAVE_DIRECTORY,input.store);}
function fileProof(){
  const name=backend.resolveFilePath(input.username,input.profile);assert.equal(path.dirname(name),input.store);
  const st=regular(name),bytes=fs.readFileSync(name);assert.equal(bytes.length,st.size);assert.ok(bytes.length>0&&bytes.length<=1024*1024);
  assert.deepEqual(fs.readdirSync(input.store).sort(),[path.basename(name)]);
  return{path:name,mode:st.mode&0o777,bytes:bytes.length,sha256:sha(bytes),base64:bytes.toString('base64'),json:bytes.toString('utf8')};
}
async function emit(proof){
  const owner=identity();assert.deepEqual(owner.children,[]);assert.equal(owner.startTicks,initialOwner.startTicks);
  const record={role:ROLE,nonce:input.nonce,source:input.source,owner:owner,resolutions:resolutions,...proof};
  const output='IKOV_FILE80_PROOF '+JSON.stringify(record,backend.replacer.bind(backend))+'\n';assert.ok(Buffer.byteLength(output)<=1024*1024);
  await new Promise((resolve,reject)=>process.stdout.write(output,e=>e?reject(e):resolve()));
}
let backend;
(async()=>{
  let actor,core,manager,submit,cache,primary,proof,startup=false;const tasks=new Set();
  try{
    assert.equal(input.profile,'main');assert.deepEqual(fs.readdirSync(input.store),[]);
    assert.equal(fs.readFileSync('target.txt','utf8').trim(),'osrs-242_2026-11-03');
    const cacheDir=path.resolve('caches/osrs-242_2026-11-03');directory(cacheDir);
    for(const name of ['main_file_cache.dat2','main_file_cache.idx255','info.json','keys.json'])regular(path.join(cacheDir,name));
    assert.ok(fs.statSync(path.join(cacheDir,'main_file_cache.idx255')).size>24*6,'prepared gameval cache, no download fallback');
    assert.equal(sha(fs.readFileSync(path.join(cacheDir,'info.json'))),'40dfaae45cea053345ee44375ce6d8e778df47a58bc666f4e97ad3db37bae36e');
    assert.equal(sha(fs.readFileSync(path.join(cacheDir,'keys.json'))),'44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a');
    const {CachePipeline}=require('./dist/game/cache/CachePipeline');cache=CachePipeline;
    await CachePipeline.initialize(input.serverRoot);
    new (require('./dist/game/definition/loader/impl/NpcDefinitionLoader').NpcDefinitionLoader)().load();
    manager=require('./dist/game/task/TaskManager').TaskManager;submit=manager.submit;
    manager.submit=function(task,...args){tasks.add(task);return submit.call(this,task,...args);};
    PM.loadFromDirectory(input.stage);core=PM.getCoreApi();assert.equal(core.TaskManager,manager);backend=core.GameConstants.PLAYER_PERSISTENCE;
    assert.ok(backend instanceof Json.JsonPlayerPersistence);assert.equal(backend.save,Json.JsonPlayerPersistence.prototype.save);
    assert.equal(backend.load,Json.JsonPlayerPersistence.prototype.load);idle();assert.equal(backend.nextWriteSequence,0);
    core.RegionManager.init();startup=true;PM.emitServerStartup({});
    const quests=require('./plugins/quests/QuestRuntime').getRegisteredQuests(),quest=quests.findLast(q=>q.key==='temple_of_ikov');assert.ok(quest);
    const {Player}=require('./dist/game/entity/impl/player/Player');
    const {PlayerSession}=require('./dist/net/PlayerSession');
    const channel={kind:'test',remoteAddress:'127.0.0.1',isOpen:()=>true,send:()=>true,close(){},onData(){},onError(){},onClose(){}};
    const session=new PlayerSession(channel);actor=new Player(session);session.setPlayer(actor);
    actor.setUsername(input.username);actor.setLongUsername(core.Misc.stringToLongBigInt(input.username));actor.setProfile(input.profile);
    actor.setAttribute('bot-skip-persistence',true);assert.equal(core.World.getPlayers().add(actor),true);
    assert.equal(actor.isPlayerBot(),false);assert.equal(actor.isRegistered(),true);
    const baseline=selected(PlayerSave.fromPlayer(actor));
    const parsed=JSON.parse(input.serialized,backend.reviver.bind(backend));
    assert.equal(parsed.inventory.length,28);assert.equal(parsed.equipment.length,14);
    assert.ok(parsed.equipment[core.Equipment.WEAPON_SLOT].id<=0,'unchanged unarmed supplied checkpoint');
    const hydrated=backend.hydratePlayerSave(parsed);assert.ok(hydrated instanceof PlayerSave);
    assert.ok(hydrated.inventory.every(i=>i instanceof core.Item));assert.ok(hydrated.equipment.every(i=>i instanceof core.Item));hydrated.applyToPlayer(actor);assert.ok(actor.getLocation() instanceof core.Location);
    const before=selected(PlayerSave.fromPlayer(actor));
    for(const key of ['attributes','inventory','equipment','skills','position'])assert.deepEqual(before[key],parsed[key],key+' genuine complete input preserved');
    assert.equal(quest.getStage(actor),70);assert.equal(actor.getAttribute('bot-skip-persistence'),true);
    const boots=core.ItemIdentifiers.BOOTS_OF_LIGHTNESS;assert.ok(Number.isInteger(boots)&&boots>0);
    const countBoots=()=>actor.getInventory().getAmount(boots)+actor.getEquipment().getAmount(boots);
    const bootCount=countBoots(),qp=Number(actor.getAttribute('quest.points'))||0;
    const ranged=actor.getSkillManager().getExperience(core.Skill.RANGED),fletching=actor.getSkillManager().getExperience(core.Skill.FLETCHING);
    assert.equal(quest.complete(actor,{scroll:false,announce:false}),true);
    assert.equal(quest.getStage(actor),80);assert.equal(actor.getAttribute('quest.points'),qp+1);
    const rangedDelta=actor.getSkillManager().getExperience(core.Skill.RANGED)-ranged,fletchingDelta=actor.getSkillManager().getExperience(core.Skill.FLETCHING)-fletching;
    assert.equal(rangedDelta,10500);assert.equal(fletchingDelta,8000);assert.equal(countBoots(),bootCount);
    const completed=selected(PlayerSave.fromPlayer(actor));
    const repeated=quest.complete(actor,{scroll:false,announce:false});assert.equal(repeated,false);
    assert.deepEqual(selected(PlayerSave.fromPlayer(actor)),completed,'same actor duplicate completion changes no saved state');
    assert.equal(actor.getAttribute('bot-skip-persistence'),true);idle();assert.equal(backend.nextWriteSequence,0,'skip flag leaves direct producer only');
    assert.equal(actor.isRegistered(),true);backend.save(actor,'ikov-fixture-completed80');await backend.flush();idle();
    assert.equal(backend.nextWriteSequence,1,'one actual installed save(liveactor)');
    const file=fileProof(),disk=JSON.parse(file.json,backend.reviver.bind(backend));
    for(const key of ['attributes','inventory','equipment','skills','position'])assert.deepEqual(disk[key],completed[key],key+' actual completed snapshot committed');
    assert.equal(disk.attributes['quest.temple_of_ikov.stage'],80);assert.equal(disk.attributes['quest.points'],qp+1);
    proof={scope:'hydrated70/registered-direct-complete80/direct-JSON-save-only',username:input.username,profile:input.profile,
      normalizedUsername:backend.normalizeUsername(input.username),privateDirectory:Json.JsonPlayerPersistence.SAVE_DIRECTORY,
      backendClass:backend.constructor.name,baseline:baseline,before:before,completed:completed,file:file,
      reward:{rangedDelta,fletchingDelta,questPointsDelta:actor.getAttribute('quest.points')-qp,bootsId:boots,bootsBefore:bootCount,bootsAfter:countBoots(),duplicateRefused:repeated===false},
      writes:backend.nextWriteSequence,pending:backend.pendingSerializedSaves.size,failed:backend.failedWrites.size};
  }catch(e){primary=e;}
  finally{
    for(const cleanup of [
      ()=>{if(actor&&core){actor.closeInterruptibleInterfaces();core.TaskManager.cancelTasks(actor);core.TaskManager.cancelTasks(actor.getIndex());
        if(actor.isRegistered())core.World.getPlayers().remove(actor);assert.equal(actor.isRegistered(),false);
        assert.equal([...core.World.getNpcs(),...core.World.getAddNPCQueue()].some(n=>n&&n.getOwner()===actor),false,'no child actor-owned NPC');
        assert.equal(core.World.getItems().some(i=>i.getOwner()===input.username),false,'no owned floor item');}},
      ()=>{if(startup)PM.emitServerShutdown({});},
      ()=>{for(const task of tasks){if(task.isRunning())task.stop();assert.equal(task.isRunning(),false);}if(manager&&submit)manager.submit=submit;},
      async()=>{if(backend){await backend.flush();idle();assert.equal(backend.nextWriteSequence,1);}},
      ()=>{if(cache&&cache.isInitialized())cache.getStore().close();}
    ])try{await cleanup();}catch(e){primary=primary?new AggregateError([primary,e],'producer and owned cleanup failed'):e;}
  }
  if(primary)throw primary;
  assert.ok(proof);proof.cleanup={actorAbsent:!actor.isRegistered(),tasksStopped:[...tasks].every(t=>!t.isRunning()),shutdownDispatched:startup,cacheCloseReturned:true,scene:{registeredNpcs:[...core.World.getNpcs()].filter(Boolean).length,queuedNpcs:core.World.getAddNPCQueue().length,floorItems:core.World.getItems().length}};
  assert.deepEqual(fileProof(),proof.file,'cleanup does not silently replace direct snapshot');
  await emit(proof);
})().catch(e=>{console.error(e);process.exitCode=1;});
`;
    const readerSource=String.raw`'use strict';
const ROLE='reader';
'use strict';
const assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const sha=b=>crypto.createHash('sha256').update(b).digest('hex');
const inputBytes=fs.readFileSync(0);assert.ok(inputBytes.length>0&&inputBytes.length<=1024*1024);
const input=JSON.parse(inputBytes.toString('utf8'));
assert.equal(input.role,ROLE);assert.match(input.nonce,/^[0-9a-f]{32}$/);
function identity(){
  const raw=fs.readFileSync('/proc/self/stat','utf8'),v=raw.slice(raw.lastIndexOf(')')+2).trim().split(/\s+/);
  const children=new Set();for(const tid of fs.readdirSync('/proc/self/task')){
    try{for(const pid of fs.readFileSync('/proc/self/task/'+tid+'/children','utf8').trim().split(/\s+/))if(pid)children.add(Number(pid));}
    catch(e){if(e.code!=='ENOENT')throw e;}
  }
  return{pid:process.pid,ppid:Number(v[1]),pgrp:Number(v[2]),session:Number(v[3]),startTicks:v[19],children:[...children].sort((a,b)=>a-b)};
}
const initialOwner=identity();assert.deepEqual(initialOwner.children,[]);
fs.writeSync(1,'IKOV_FILE80_OWNER '+JSON.stringify({role:ROLE,nonce:input.nonce,source:input.source,owner:initialOwner})+'\n');
function directory(p){assert.ok(path.isAbsolute(p));const s=fs.lstatSync(p);assert.ok(s.isDirectory()&&!s.isSymbolicLink());assert.equal(fs.realpathSync(p),p);}
function regular(p){const s=fs.lstatSync(p);assert.ok(s.isFile()&&!s.isSymbolicLink());return s;}
directory(input.root);directory(input.store);directory(input.stage);
assert.equal(path.dirname(input.store),input.root);assert.equal(path.dirname(input.stage),input.root);
assert.equal(fs.realpathSync(process.cwd()),input.serverRoot);
assert.deepEqual(JSON.parse(fs.readFileSync(path.join(input.serverRoot,'data/definitions/world.json'),'utf8')).plugins,{TutorialIsland:false});
assert.equal(fs.existsSync(path.join(input.serverRoot,'data/definitions/world.local.json')),false);
assert.notEqual(process.env.BROWSER_HOST,'1');
const options=process.execArgv.filter((v,i,a)=>i===0||a[i-1]!=='-e').join(' ')+' '+(process.env.NODE_OPTIONS||'');
assert.ok(!/(?:^|[\s"'])--(?:preserve-symlinks(?:-main)?|require|import)(?:[=\s"']|$)/.test(options),'unqualified module-resolution/preload option');
const e=process.execArgv.indexOf('-e');assert.ok(e>=0);assert.equal(sha(Buffer.from(process.execArgv[e+1])),input.programSHA256);
const testFile=path.join(input.serverRoot,'tests/temple-of-ikov-dialogue.test.cjs');regular(testFile);
assert.equal(sha(fs.readFileSync(testFile)),input.source.testSHA256);
assert.equal(input.source.reviewedBase,'0d84f10f124550b511a2c622bbfcf52232a4a719');
assert.ok(Array.isArray(input.pins)&&input.pins.length===(ROLE==='writer'?18:2));
const stagedFiles=[],stagedDirs=[];function walkStage(dir,rel){
  for(const name of fs.readdirSync(dir).sort()){
    const child=path.join(dir,name),key=rel?rel+'/'+name:name,st=fs.lstatSync(child);
    if(st.isDirectory()&&!st.isSymbolicLink()){assert.equal(fs.realpathSync(child),child);stagedDirs.push(key);walkStage(child,key);}
    else{assert.ok(st.isSymbolicLink(),'only exact production FILE links');stagedFiles.push(key);}
  }
}
walkStage(input.stage,'');assert.deepEqual(stagedFiles.sort(),input.pins.map(p=>p.rel).sort());
const expectedDirs=new Set();for(const pin of input.pins){let dir=path.posix.dirname(pin.rel);while(dir!=='.'){expectedDirs.add(dir);dir=path.posix.dirname(dir);}}
assert.deepEqual(stagedDirs.sort(),[...expectedDirs].sort());
const resolutions=[];for(const pin of input.pins){
  assert.ok(!path.isAbsolute(pin.rel)&&!pin.rel.split('/').includes('..'));
  const link=path.join(input.stage,pin.rel),target=path.join(input.serverRoot,'plugins',pin.rel);
  assert.ok(fs.lstatSync(link).isSymbolicLink());assert.equal(fs.readlinkSync(link),target);
  assert.equal(fs.realpathSync(link),target);regular(target);
  const bytes=fs.readFileSync(target);assert.equal(bytes.length,pin.bytes);assert.equal(sha(bytes),pin.sha256);
  assert.equal(require.resolve(link),target);
  resolutions.push({rel:pin.rel,target:target,bytes:bytes.length,sha256:sha(bytes),resolved:require.resolve(link)});
}
require('./dist/Server').Server.installProductionPathResolver();
const Json=require('./plugins/persistence/JsonPlayerPersistence.plugin');
Json.JsonPlayerPersistence.SAVE_DIRECTORY=input.store;
const {PluginManager:PM}=require('./dist/plugins/PluginManager');
const {GameConstants}=require('./dist/game/GameConstants');
const {PlayerSave}=require('./dist/game/entity/impl/player/persistence/PlayerSave');
function data(save){return JSON.parse(JSON.stringify(save,backend.replacer.bind(backend)),backend.reviver.bind(backend));}
function selected(save){const o=data(save);return{attributes:o.attributes,inventory:o.inventory,equipment:o.equipment,skills:o.skills,position:o.position};}
function idle(){assert.equal(backend.pendingSerializedSaves.size,0);assert.equal(backend.failedWrites.size,0);assert.equal(GameConstants.PLAYER_PERSISTENCE,backend);assert.equal(Json.JsonPlayerPersistence.SAVE_DIRECTORY,input.store);}
function fileProof(){
  const name=backend.resolveFilePath(input.username,input.profile);assert.equal(path.dirname(name),input.store);
  const st=regular(name),bytes=fs.readFileSync(name);assert.equal(bytes.length,st.size);assert.ok(bytes.length>0&&bytes.length<=1024*1024);
  assert.deepEqual(fs.readdirSync(input.store).sort(),[path.basename(name)]);
  return{path:name,mode:st.mode&0o777,bytes:bytes.length,sha256:sha(bytes),base64:bytes.toString('base64'),json:bytes.toString('utf8')};
}
async function emit(proof){
  const owner=identity();assert.deepEqual(owner.children,[]);assert.equal(owner.startTicks,initialOwner.startTicks);
  const record={role:ROLE,nonce:input.nonce,source:input.source,owner:owner,resolutions:resolutions,...proof};
  const output='IKOV_FILE80_PROOF '+JSON.stringify(record,backend.replacer.bind(backend))+'\n';assert.ok(Buffer.byteLength(output)<=1024*1024);
  await new Promise((resolve,reject)=>process.stdout.write(output,e=>e?reject(e):resolve()));
}
let backend;
(async()=>{
  let primary,proof;
  try{
    assert.equal(input.profile,'main');PM.loadFromDirectory(input.stage);backend=GameConstants.PLAYER_PERSISTENCE;
    assert.ok(backend instanceof Json.JsonPlayerPersistence);assert.equal(backend.load,Json.JsonPlayerPersistence.prototype.load);
    idle();assert.equal(backend.nextWriteSequence,0);
    const initial=fileProof();assert.equal(initial.sha256,input.expectedFileSHA256);assert.equal(initial.bytes,input.expectedFileBytes);
    const loaded=backend.load(input.username,input.profile);assert.ok(loaded instanceof PlayerSave);
    const {Item}=require('./dist/game/model/Item'),{Location}=require('./dist/game/model/Location');
    assert.ok(loaded.inventory.every(i=>i instanceof Item));assert.ok(loaded.equipment.every(i=>i instanceof Item));assert.ok(loaded.getPosition() instanceof Location);
    const loadedJSON=JSON.stringify(loaded,backend.replacer.bind(backend));
    const saved=JSON.parse(initial.json,backend.reviver.bind(backend)),decoded=JSON.parse(loadedJSON,backend.reviver.bind(backend));
    for(const key of ['attributes','inventory','equipment','skills','position'])assert.deepEqual(decoded[key],saved[key],key+' genuine fresh-process FILE load');
    assert.equal(decoded.inventory.length,28);assert.equal(decoded.equipment.length,14);
    assert.equal(decoded.attributes['quest.temple_of_ikov.stage'],80);idle();assert.equal(backend.nextWriteSequence,0);
    assert.deepEqual(fileProof(),initial,'reader left actual file bytes and metadata unchanged');
    proof={scope:'fresh-process-original-FILE-load80/no-actor-no-auth',username:input.username,profile:input.profile,
      normalizedUsername:backend.normalizeUsername(input.username),privateDirectory:Json.JsonPlayerPersistence.SAVE_DIRECTORY,
      backendClass:backend.constructor.name,file:initial,loadedJSON:loadedJSON,writes:backend.nextWriteSequence,pending:backend.pendingSerializedSaves.size,failed:backend.failedWrites.size};
  }catch(e){primary=e;}
  finally{try{if(backend){await backend.flush();idle();assert.equal(backend.nextWriteSequence,0);}}catch(e){primary=primary?new AggregateError([primary,e],'reader and flush failed'):e;}}
  if(primary)throw primary;await emit(proof);
})().catch(e=>{console.error(e);process.exitCode=1;});
`;
    const serverRoot=fs.realpathSync(path.resolve(__dirname,'..'));
    const source={reviewedBase:sourceBase,testSHA256:crypto.createHash('sha256').update(fs.readFileSync(__filename)).digest('hex')};
    const nonce=crypto.randomBytes(16).toString('hex');
    const store=path.join(ownedRoot,'producer-file80');fs.mkdirSync(store,{mode:0o700});
    const sourceWorld=path.join(serverRoot,'data/definitions/world.json');
    const parentWorldSHA=crypto.createHash('sha256').update(fs.readFileSync(sourceWorld)).digest('hex');
    assert.equal(parentWorldSHA,'0a9e7c31b5fd56a9d6fadb1fdcdfa5e13a0e1abcf1d6b98d7a7d931e3c230012','unchanged bound shipped world source');
    const parentEnv=JSON.stringify(process.env),parentBackendStatic=backend.constructor.DATABASE_PATH,linkRecords=[];
    const baselineGroup=ownedGroup();
    function digest(bytes){return crypto.createHash('sha256').update(bytes).digest('hex');}
    function procIdentity(pid){
      const raw=fs.readFileSync('/proc/'+pid+'/stat','utf8'),v=raw.slice(raw.lastIndexOf(')')+2).trim().split(/\s+/);
      return{pid:Number(pid),ppid:Number(v[1]),pgrp:Number(v[2]),session:Number(v[3]),startTicks:v[19]};
    }
    function ownedGroup(){
      const own=procIdentity(process.pid),out=[];
      for(const name of fs.readdirSync('/proc'))if(/^\d+$/.test(name)){
        try{const v=procIdentity(Number(name));if(v.pgrp===own.pgrp&&v.session===own.session)out.push(v);}
        catch(e){if(e.code!=='ENOENT'&&e.code!=='ESRCH')throw e;}
      }
      return out.sort((a,b)=>a.pid-b.pid);
    }
    function stage(role,pins){
      remainingChildTime();producerStagingSafe=false;const dir=path.join(ownedRoot,'producer-plugins-'+role);fs.mkdirSync(dir,{mode:0o700});
      for(const pin of pins){
        const target=path.join(serverRoot,'plugins',pin.rel),link=path.join(dir,pin.rel);
        assert.ok(!path.isAbsolute(pin.rel)&&!pin.rel.split('/').includes('..'));
        const st=fs.lstatSync(target);assert.ok(st.isFile()&&!st.isSymbolicLink());assert.equal(fs.realpathSync(target),target);
        const bytes=fs.readFileSync(target);assert.equal(bytes.length,pin.bytes);assert.equal(digest(bytes),pin.sha256);
        fs.mkdirSync(path.dirname(link),{recursive:true,mode:0o700});fs.symlinkSync(target,link,'file');
        const row={role,relativePath:pin.rel,link,target,bytes:bytes.length,sha256:pin.sha256,unlinked:false};linkRecords.push(row);
        assert.ok(fs.lstatSync(link).isSymbolicLink());assert.equal(fs.readlinkSync(link),target);assert.equal(fs.realpathSync(link),target);
      }
      return dir;
    }
    function unstage(){
      assert.equal(producerChildrenTerminal,true,'no unlink before known terminal owner group');
      for(const row of linkRecords)if(!row.unlinked){
        assert.ok(fs.lstatSync(row.link).isSymbolicLink());assert.equal(fs.readlinkSync(row.link),row.target);
        assert.equal(fs.realpathSync(row.link),row.target);const bytes=fs.readFileSync(row.target);
        assert.equal(bytes.length,row.bytes);assert.equal(digest(bytes),row.sha256);
        fs.unlinkSync(row.link);row.unlinked=true;assert.equal(fs.existsSync(row.link),false);
      }
      for(const role of ['writer','reader']){
        const dir=path.join(ownedRoot,'producer-plugins-'+role);if(!fs.existsSync(dir))continue;
        function empty(real){for(const name of fs.readdirSync(real)){const child=path.join(real,name),st=fs.lstatSync(child);assert.ok(st.isDirectory()&&!st.isSymbolicLink());empty(child);fs.rmdirSync(child);}}
        const st=fs.lstatSync(dir);assert.ok(st.isDirectory()&&!st.isSymbolicLink());assert.equal(fs.realpathSync(dir),dir);empty(dir);fs.rmdirSync(dir);
      }
      producerStagingSafe=true;
    }
    function fileBytes(proof){
      assert.equal(path.dirname(proof.path),store);const st=fs.lstatSync(proof.path);assert.ok(st.isFile()&&!st.isSymbolicLink());
      const bytes=fs.readFileSync(proof.path);assert.equal(bytes.length,st.size);assert.equal(bytes.length,proof.bytes);
      assert.equal(st.mode&0o777,proof.mode);assert.equal(digest(bytes),proof.sha256);
      const returned=Buffer.from(proof.base64,'base64');assert.equal(returned.toString('base64'),proof.base64);assert.deepEqual(bytes,returned);
      assert.equal(bytes.toString('utf8'),proof.json);assert.deepEqual(Buffer.from(proof.json),bytes);
      assert.deepEqual(fs.readdirSync(store).sort(),[path.basename(proof.path)]);return bytes;
    }
    function parsePhase(result,request){
      const text=result.stdout.toString('utf8');assert.deepEqual(Buffer.from(text),result.stdout);
      assert.ok(text.endsWith('\n'));const lines=text.slice(0,-1).split('\n'),owners=[],proofs=[],logs=[];
      const fixed=new Set(["[plugin:JsonPlayerPersistence] registered { saveDirectory: 'data/saves/characters' }",
        '[plugins] player persistence set by JsonPlayerPersistence: NullPlayerPersistence -> JsonPlayerPersistence',
        '[plugins] active='+(request.role==='writer'?17:1)+' disabled=1']);
      const seen=new Map();const addLog=line=>{seen.set(line,(seen.get(line)||0)+1);assert.equal(seen.get(line),1,'duplicate source log');};
      for(let i=0;i<lines.length;i++){
        const line=lines[i];
        if(line.startsWith('IKOV_FILE80_OWNER ')){owners.push(JSON.parse(line.slice(18)));continue;}
        if(line.startsWith('IKOV_FILE80_PROOF ')){proofs.push(JSON.parse(line.slice(18),codec.reviver.bind(codec)));continue;}
        logs.push(line);
        if(fixed.has(line)){addLog(line);continue;}
        if(line==='[plugin:JsonPlayerPersistence] registered {'){
          const detail=lines[++i],end=lines[++i];assert.equal(detail,"  saveDirectory: 'data/saves/characters'");assert.equal(end,'}');logs.push(detail,end);
          addLog("[plugin:JsonPlayerPersistence] registered { saveDirectory: 'data/saves/characters' }");continue;
        }
        assert.equal(request.role,'writer','reader has no other source log');
        if(line==='[plugin:Settings] registered'){addLog(line);continue;}
        if(/^\[plugin:Potions\] registered \{ potionEntries: \d+, potionGroups: \d+ \}$/.test(line)){addLog('Potions.registered');continue;}
        if(line==='[plugin:Potions] registered {'){
          const entries=lines[++i],groups=lines[++i],end=lines[++i];assert.match(entries,/^  potionEntries: \d+,$/);assert.match(groups,/^  potionGroups: \d+$/);assert.equal(end,'}');logs.push(entries,groups,end);addLog('Potions.registered');continue;
        }
        if(/^\[cache\] definitions npc=\d+ item=\d+ object=\d+$/.test(line)){addLog('CacheDefinitions.loaded');continue;}
        if(/^\[npc-definitions\] cache-backed; configured stats applied=\d+, mismatched=\d+, service animations captured=\d+, inferred=\d+ \(by name=\d+\), guessed=\d+, by family=\d+$/.test(line)){addLog('NpcDefinitionLoader.loaded');continue;}
        if(/^\[collision\] indexed \d+ cache regions$/.test(line)){addLog('RegionManager.indexed');continue;}
        if(line==='[World] Registering player - [username, host] : ['+inputName()+', undefined]'||line==='[World] Deregistering player - [username, host] : ['+inputName()+', undefined]'){addLog(line);continue;}
        throw new Error('unexpected FILE80 child stdout: '+line.slice(0,256));
      }
      for(const line of fixed)assert.equal(seen.get(line),1,'actual production registration log');
      assert.equal(owners.length,1);assert.equal(proofs.length,1);
      for(const r of [owners[0],proofs[0]]){assert.equal(r.role,request.role);assert.equal(r.nonce,nonce);assert.deepEqual(r.source,source);assert.equal(r.owner.pid,result.pid);assert.deepEqual(r.owner.children,[]);}
      assert.equal(owners[0].owner.startTicks,proofs[0].owner.startTicks);
      assert.deepEqual(proofs[0].resolutions,request.pins.map(pin=>({rel:pin.rel,target:path.join(serverRoot,'plugins',pin.rel),bytes:pin.bytes,sha256:pin.sha256,resolved:path.join(serverRoot,'plugins',pin.rel)})));
      return{proof:proofs[0],owner:owners[0],logs};
    }
    function inputName(){return f.p.getUsername();}
    function runPhase(role,program,pins,extra){
      remainingChildTime();const stageDir=stage(role,pins);
      const request={role,nonce,source,root:ownedRoot,store,stage:stageDir,serverRoot,pins,username:inputName(),profile:f.p.getProfile(),
        programSHA256:digest(Buffer.from(program)),...extra};
      const requestText=JSON.stringify(request);chargeProof(program,'static child argv');chargeProof(requestText,'child input');
      producerChildrenTerminal=false;const result=spawnSync(process.execPath,['-e',program],{cwd:serverRoot,input:requestText,timeout:remainingChildTime(),maxBuffer:remainingChildBuffer()});
      let ownerError;try{assert.deepEqual(ownedGroup(),baselineGroup,'original owned group after child terminal');producerChildrenTerminal=true;}catch(e){ownerError=e;}
      const record=retainChildResult(role,result);producerRaw.push(record);if(ownerError)throw ownerError;
      assert.ifError(result.error);assert.equal(result.status,0,'FILE80 '+role+' status');assert.equal(result.signal,null,'natural FILE80 '+role+' terminal');
      assert.ok(Buffer.isBuffer(result.stdout)&&Buffer.isBuffer(result.stderr));assert.equal(result.stderr.length,0,'no child stderr/warnings');remainingChildTime();
      const parsedPhase=parsePhase(result,request);assert.equal(parsedPhase.proof.profile,f.p.getProfile());
      assert.equal(parsedPhase.proof.username,inputName());assert.equal(parsedPhase.proof.normalizedUsername,codec.normalizeUsername(inputName()));
      assert.equal(parsedPhase.proof.privateDirectory,store);assert.equal(parsedPhase.proof.backendClass,'JsonPlayerPersistence');
      assert.equal(parsedPhase.proof.pending,0);assert.equal(parsedPhase.proof.failed,0);
      return parsedPhase;
    }
    let supplementFailure;
    try{
      const writer=runPhase('writer',writerSource,candidatePins,{serialized});assert.equal(writer.proof.writes,1);
      const actual=fileBytes(writer.proof.file),decoded=JSON.parse(actual.toString('utf8'),codec.reviver.bind(codec));
      for(const key of ['attributes','inventory','equipment','skills','position']){
        assert.deepEqual(writer.proof.before[key],parsed[key],'whole original input '+key);
        assert.deepEqual(decoded[key],writer.proof.completed[key],'whole actual completed '+key);
      }
      assert.equal(decoded.attributes['quest.temple_of_ikov.stage'],80);assert.equal(writer.proof.reward.duplicateRefused,true);
      assert.equal(writer.proof.reward.rangedDelta,10500);assert.equal(writer.proof.reward.fletchingDelta,8000);assert.equal(writer.proof.reward.questPointsDelta,1);
      assert.equal(writer.proof.reward.bootsBefore,writer.proof.reward.bootsAfter);
      const readerPins=candidatePins.filter(p=>['persistence/JsonPlayerPersistence.plugin.js','areas/TutorialIsland.plugin.js'].includes(p.rel));
      const reader=runPhase('reader',readerSource,readerPins,{expectedFileSHA256:writer.proof.file.sha256,expectedFileBytes:writer.proof.file.bytes});
      assert.equal(reader.proof.writes,0);assert.deepEqual(fileBytes(reader.proof.file),actual);
      const loaded=JSON.parse(reader.proof.loadedJSON,codec.reviver.bind(codec));
      for(const key of ['attributes','inventory','equipment','skills','position'])assert.deepEqual(loaded[key],decoded[key],'fresh process original loader '+key);
      producerObservation={layer:'hydrated-fixture/direct-registered-complete80/installed-JSON-save/fresh-process-FILE-load',source,nonce,
        writer:writer.proof,reader:reader.proof,rawChildren:producerRaw,sourceLogs:{writer:writer.logs,reader:reader.logs},
        ownedGroupBefore:baselineGroup,ownedGroupAfter:ownedGroup(),stagingLinks:linkRecords,
        deadlineMs:10000,budgetUsedBytes:proofBudgetUsed,scope:'no ordinary account/auth/stock save/world restart/crashdurability/SQLite/full15 proof'};
      chargeProof(JSON.stringify(producerObservation,codec.replacer.bind(codec)),'complete encoded producer proof');producerObservation.budgetUsedBytes=proofBudgetUsed;remainingChildTime();
    }catch(e){
      supplementFailure=e;const artifacts=[];
      if(producerChildrenTerminal)for(const name of fs.readdirSync(store)){
        const file=path.join(store,name),st=fs.lstatSync(file);const row={name,mode:st.mode&0o777,bytes:st.size,regular:st.isFile()&&!st.isSymbolicLink()};
        if(row.regular&&st.size<=Math.floor((proofLimit-proofReserve-proofBudgetUsed)/3)){
          const bytes=fs.readFileSync(file);row.sha256=digest(bytes);row.base64=bytes.toString('base64');row.complete=true;
          chargeProof(JSON.stringify(row),'failure store bytes');
        }else{row.complete=false;row.unretainedBytes=st.size;}
        artifacts.push(row);
      }
      producerFailureEvidence={...producerFailureEvidence,reason:e.message,rawChildren:producerRaw,artifacts,ownedRoot,childrenTerminal:producerChildrenTerminal,stagingLinks:linkRecords};
    }
    finally{
      for(const cleanup of [
        ()=>{assert.deepEqual(ownedGroup(),baselineGroup);producerChildrenTerminal=true;},
        ()=>{unstage();},
        ()=>{assert.equal(core.GameConstants.PLAYER_PERSISTENCE,backend);assert.equal(Json.JsonPlayerPersistence.SAVE_DIRECTORY,parentDirectory);
          assert.equal(backend.constructor.DATABASE_PATH,parentBackendStatic);assert.equal(JSON.stringify(process.env),parentEnv);assert.equal(digest(fs.readFileSync(sourceWorld)),parentWorldSHA);}
      ])try{cleanup();}catch(e){supplementFailure=supplementFailure?new AggregateError([supplementFailure,e],'producer proof and source/owned cleanup failed'):e;}
    }
    if(supplementFailure)throw supplementFailure;

    observation={layer:'isolated-json-flush/new-instance-filesystem-load',profile:proof.profile,
      normalizedUsername:proof.normalizedUsername,filePath:proof.filePath,privateRoot:ownedRoot,regularFile:stat.isFile()&&!stat.isSymbolicLink(),
      fileBytes:proof.fileBytes,fileSHA256:proof.fileSHA256,fileBase64:file.toString('base64'),loadedJSON:proof.loadedJSON,
      loadedPosition:JSON.parse(JSON.stringify(reopened.position)),writerPending:proof.writerPending,writerFailed:proof.writerFailed,
      readerPending:proof.readerPending,readerFailed:proof.readerFailed,readerWrites:proof.readerWrites};
    // Existing root stdout transport frames this bounded logical JSON into <=3072-byte payloads.
    assert.ok(Buffer.byteLength(JSON.stringify(observation),'utf8')<=1024*1024);
    chargeProof(JSON.stringify(observation),'original JSON70 public proof');remainingChildTime();
  }catch(error){failure=error;}
  finally{
    for(const cleanup of [
      ()=>{assert.equal(core.GameConstants.PLAYER_PERSISTENCE,backend);assert.equal(Json.JsonPlayerPersistence.SAVE_DIRECTORY,parentDirectory);},
      ()=>{const stat=fs.lstatSync(root);assert.ok(stat.isDirectory()&&!stat.isSymbolicLink());
        const actual=fs.realpathSync(root);if(ownedRoot!==undefined)assert.equal(actual,ownedRoot);
        // Before a child starts, mkdtemp alone owns this still-private root too.
        assert.ok(ownedRoot!==undefined||childResult===undefined);
        assert.equal(producerChildrenTerminal,true,'all owned children terminal before root removal');assert.equal(producerStagingSafe,true,'only validated staging links unlinked');
        fs.rmSync(root,{recursive:true});assert.equal(fs.existsSync(root),false,'only owned private store root cleaned after child exit');
        if(proofDeadline!==undefined)remainingChildTime();}
    ])try{cleanup();}catch(error){failure=failure?new AggregateError([failure,error],'store proof and owned cleanup failed'):error;}
  }
  if(failure){
    reportProducerFailure(failure);
    if(childResult)console.log('IKOV_JSON_STORE_CHILD_FAILURE',JSON.stringify({
      error:childResult.error?{name:childResult.error.name,code:childResult.error.code,message:childResult.error.message}:null,
      status:childResult.status,signal:childResult.signal,
      stdoutBase64:Buffer.isBuffer(childResult.stdout)?childResult.stdout.toString('base64'):null,
      stderrBase64:Buffer.isBuffer(childResult.stderr)?childResult.stderr.toString('base64'):null}));
    throw failure;
  }
  try{disposeActor(f.p);}catch(error){reportProducerFailure(error);throw error;}
  console.log('IKOV_JSON_STORE',JSON.stringify({...observation,ownedRootCleaned:true}));
  try{assert.ok(producerObservation);remainingChildTime();
    console.log('IKOV_JSON_PRODUCER_STORE',JSON.stringify({...producerObservation,ownedRootCleaned:true,stagingLinksUnlinked:producerStagingSafe},codec.replacer.bind(codec)));
  }catch(error){reportProducerFailure(error);throw error;}
  return reopened;
}
test('the real Guardian menu reaches the yearly-bath branch and physical pendant award',async t=>{
  const f=actor(t,2644,9897,60),npc=f.guardian();await f.talk(npc);
  await f.choose('I seek the Staff of Armadyl');await f.choose('Lucien will give me');await f.choose("yearly bath");
  // This records the imported narrative producer only. Exact retail animation,
  // gfx and timing remain a separate unresolved visual evidence requirement.
  await f.choose("Ok! I'll help!");await f.drain();
  assert.ok(f.text.some(s=>s.includes('The guardian splashes holy water over you.')));
  assert.ok(f.text.some(s=>s.includes('You have been cleansed!')));
  assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.ARMADYL_PENDANT),1);
  assert.equal(f.p.getAttribute('temple-of-ikov:side'),1);
});
test('only the actual house Lucien death earns the Guardian ending, which completes through native Continue',async t=>{
  let foreign;t.after(()=>PM.removeNpc(foreign));
  const f=actor(t,3122,3482,70),npc=f.own(HOUSE),boots=core.ItemIdentifiers.BOOTS_OF_LIGHTNESS;f.add(boots);
  f.p.setAttribute('temple-of-ikov:side',1);f.p.getEquipment().setItem(core.Equipment.AMULET_SLOT,new core.Item(core.ItemIdentifiers.ARMADYL_PENDANT,1));
  foreign=PM.spawnNpc({id:HOUSE,x:3123,y:3483,z:0,owner:f.p,ownerOnly:true,wanderRadius:0});turns(3);assert.ok(foreign.isRegistered());
  lethalNativeHit(f,foreign);assert.notEqual(f.p.getAttribute('temple-of-ikov:lucien-defeated'),1,'same-id foreign actor cannot credit the ending');
  const skills=f.p.getSkillManager(),ranged=skills.getExperience(core.Skill.RANGED),fletching=skills.getExperience(core.Skill.FLETCHING);
  lethalNativeHit(f,npc);assert.equal(f.p.getAttribute('temple-of-ikov:lucien-defeated'),1);assert.equal(quest.isComplete(f.p),false);
  assert.match(f.p.getDialogueManager().getCurrent().getText(),/You have defeated me for now/);await f.next();
  assert.equal(quest.isComplete(f.p),true);assert.equal(f.p.getInventory().getAmount(boots),1);
  assert.equal(skills.getExperience(core.Skill.RANGED)-ranged,10500);assert.equal(skills.getExperience(core.Skill.FLETCHING)-fletching,8000);
  assert.equal(quest.complete(f.p,{scroll:false,announce:false}),false);assert.equal(skills.getExperience(core.Skill.RANGED)-ranged,10500);
});
test('an earned Guardian ending survives PlayerSave and a fresh owned house actor',async t=>{
  const f=actor(t,3122,3482,70),npc=f.own(HOUSE);f.p.setAttribute('temple-of-ikov:side',1);
  f.p.getEquipment().setItem(core.Equipment.AMULET_SLOT,new core.Item(core.ItemIdentifiers.ARMADYL_PENDANT,1));lethalNativeHit(f,npc);
  const username=f.p.getUsername(),save=snapshotAndLogout(f,{side:1,delivered:0,defeated:1});assert.equal(save.attributes['temple-of-ikov:lucien-defeated'],1);
  const fresh=actor(t,3122,3482,0,{save,username});const replacement=fresh.own(HOUSE);assert.notEqual(replacement,npc);
  assert.equal(quest.getStage(fresh.p),70);assert.equal(fresh.p.getAttribute('temple-of-ikov:state-version'),1);
  assert.equal(fresh.p.getAttribute('temple-of-ikov:side'),1);assert.equal(fresh.p.getAttribute('temple-of-ikov:lucien-defeated'),1);
  assert.equal(fresh.p.getAttribute('temple-of-ikov:staff-delivered'),0);assert.equal(fresh.p.getAttribute('bot-skip-persistence'),true);
  assert.deepEqual(fresh.p.getInventory().getItems(),save.inventory);assert.deepEqual(fresh.p.getEquipment().getItems(),save.equipment);
  await fresh.talk(replacement);assert.match(fresh.p.getDialogueManager().getCurrent().getText(),/You have defeated me for now/);await fresh.next();assert.equal(quest.isComplete(fresh.p),true);
});
test('delivered staff recovery survives PlayerSave without a second physical debit',async t=>{
  const f=actor(t,3122,3482,70),npc=f.own(HOUSE),staff=core.ItemIdentifiers.STAFF_OF_ARMADYL;f.add(staff);
  stageMessage(f,npc,'VBn1hb',HANDOVER);await f.next();assert.equal(f.p.getInventory().getAmount(staff),0);
  const username=f.p.getUsername(),save=snapshotAndLogout(f,{side:0,delivered:1,defeated:0});assert.equal(save.attributes['temple-of-ikov:staff-delivered'],1);
  const fresh=actor(t,3122,3482,0,{save,username}),replacement=fresh.own(HOUSE);
  assert.equal(quest.getStage(fresh.p),70);assert.equal(fresh.p.getAttribute('temple-of-ikov:state-version'),1);
  assert.equal(fresh.p.getAttribute('temple-of-ikov:side'),0);assert.equal(fresh.p.getAttribute('temple-of-ikov:staff-delivered'),1);
  assert.equal(fresh.p.getAttribute('temple-of-ikov:lucien-defeated'),0);assert.equal(fresh.p.getAttribute('bot-skip-persistence'),true);
  assert.deepEqual(fresh.p.getInventory().getItems(),save.inventory);assert.deepEqual(fresh.p.getEquipment().getItems(),save.equipment);
  stageMessage(fresh,replacement,'VBn1hb',HANDOVER);await fresh.next();await fresh.drain();
  assert.equal(fresh.p.getInventory().getAmount(staff),0);assert.equal(quest.isComplete(fresh.p),true);assert.ok(fresh.text.some(s=>s.includes('already given Lucien')));
});
test('preset completion with experience disabled grants neither quest XP nor extra boots',t=>{
  const f=actor(t,3122,3482,70);f.add(core.ItemIdentifiers.BOOTS_OF_LIGHTNESS);
  const skills=f.p.getSkillManager(),r=skills.getExperience(core.Skill.RANGED),fl=skills.getExperience(core.Skill.FLETCHING);
  assert.equal(quest.complete(f.p,{experience:false,scroll:false,announce:false}),true);
  assert.equal(skills.getExperience(core.Skill.RANGED),r);assert.equal(skills.getExperience(core.Skill.FLETCHING),fl);assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.BOOTS_OF_LIGHTNESS),1);
});
for(const [legacy,paid,killed] of [[50,1,0],[60,1,1]])test(`legacy${legacy} preserves payment and honest kill credit through PlayerSave`,t=>{
  const f=actor(t,3122,3482,legacy);assert.equal(f.p.getAttribute('temple-of-ikov:paid-winelda'),paid);
  assert.equal(f.p.getAttribute('temple-of-ikov:warrior-killed'),killed);assert.equal(quest.getStage(f.p),legacy);
  const save=PlayerSave.fromPlayer(f.p),fresh=actor(t,3122,3482,0,{save});
  assert.equal(fresh.p.getAttribute('temple-of-ikov:paid-winelda'),paid);assert.equal(fresh.p.getAttribute('temple-of-ikov:warrior-killed'),killed);assert.equal(quest.getStage(fresh.p),legacy);
});

for(const canonical of [0,'invalid'])test(`migrated canonical side${canonical} does not resurrect an obsolete Guardian side at handover`,async t=>{
  const f=actor(t,3122,3482,70),npc=f.own(HOUSE);f.add(core.ItemIdentifiers.STAFF_OF_ARMADYL);
  f.p.setAttribute('temple-of-ikov:state-version',1);f.p.setAttribute('temple-of-ikov:side',canonical);f.p.setAttribute('quest.temple_of_ikov.side',1);
  stageMessage(f,npc,'VBn1hb',HANDOVER);await f.next();assert.equal(f.p.getInventory().getAmount(core.ItemIdentifiers.STAFF_OF_ARMADYL),0);
  assert.equal(f.p.getAttribute('temple-of-ikov:staff-delivered'),1);assert.match(f.p.getDialogueManager().getCurrent().getText(),/Muhahahhahahaha/);
});

for(const side of [1,2])test(`a genuine legacy-only side${side} save receives defaults before preserving its branch on login`,t=>{
  const seed=actor(t,3122,3482,70,{login:false});seed.p.setAttribute('quest.temple_of_ikov.side',side);
  const old=snapshotAndLogout(seed);
  for(const key of Object.keys(old.attributes))if(key.startsWith('temple-of-ikov:'))delete old.attributes[key];
  assert.equal(Object.hasOwn(old.attributes,'temple-of-ikov:side'),false);assert.equal(Object.hasOwn(old.attributes,'temple-of-ikov:state-version'),false);
  const loaded=actor(t,3122,3482,0,{save:old,login:false});
  assert.equal(loaded.p.getAttribute('temple-of-ikov:side'),0,'native applyToPlayer installs the new default before migration');
  assert.equal(loaded.p.getAttribute('temple-of-ikov:state-version'),0);assert.equal(loaded.p.getAttribute('quest.temple_of_ikov.side'),side);
  loginPlayer(loaded.p);
  assert.equal(loaded.p.getAttribute('temple-of-ikov:side'),side);assert.equal(loaded.p.getAttribute('quest.temple_of_ikov.side'),0);assert.equal(loaded.p.getAttribute('temple-of-ikov:state-version'),1);
  const saved=PlayerSave.fromPlayer(loaded.p);saved.attributes=JSON.parse(JSON.stringify(saved.attributes));
  const again=actor(t,3122,3482,0,{save:saved});assert.equal(again.p.getAttribute('temple-of-ikov:side'),side);assert.equal(again.p.getAttribute('quest.temple_of_ikov.side'),0);assert.equal(again.p.getAttribute('temple-of-ikov:state-version'),1);
});
