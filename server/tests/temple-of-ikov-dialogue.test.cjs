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
    const request=JSON.stringify({root:ownedRoot,username:f.p.getUsername(),profile:f.p.getProfile(),serialized});
    assert.ok(Buffer.byteLength(request,'utf8')<=1024*1024);
    const result=childResult=spawnSync(process.execPath,['-e',childSource],{cwd:path.resolve(__dirname,'..'),
      input:request,timeout:10000,maxBuffer:1024*1024});
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
    observation={layer:'isolated-json-flush/new-instance-filesystem-load',profile:proof.profile,
      normalizedUsername:proof.normalizedUsername,filePath:proof.filePath,privateRoot:ownedRoot,regularFile:stat.isFile()&&!stat.isSymbolicLink(),
      fileBytes:proof.fileBytes,fileSHA256:proof.fileSHA256,fileBase64:file.toString('base64'),loadedJSON:proof.loadedJSON,
      loadedPosition:JSON.parse(JSON.stringify(reopened.position)),writerPending:proof.writerPending,writerFailed:proof.writerFailed,
      readerPending:proof.readerPending,readerFailed:proof.readerFailed,readerWrites:proof.readerWrites};
    // Existing root stdout transport frames this bounded logical JSON into <=3072-byte payloads.
    assert.ok(Buffer.byteLength(JSON.stringify(observation),'utf8')<=1024*1024);
  }catch(error){failure=error;}
  finally{
    for(const cleanup of [
      ()=>{assert.equal(core.GameConstants.PLAYER_PERSISTENCE,backend);assert.equal(Json.JsonPlayerPersistence.SAVE_DIRECTORY,parentDirectory);},
      ()=>{const stat=fs.lstatSync(root);assert.ok(stat.isDirectory()&&!stat.isSymbolicLink());
        const actual=fs.realpathSync(root);if(ownedRoot!==undefined)assert.equal(actual,ownedRoot);
        // Before a child starts, mkdtemp alone owns this still-private root too.
        assert.ok(ownedRoot!==undefined||childResult===undefined);
        fs.rmSync(root,{recursive:true});assert.equal(fs.existsSync(root),false,'only owned private store root cleaned after child exit');}
    ])try{cleanup();}catch(error){failure=failure?new AggregateError([failure,error],'store proof and owned cleanup failed'):error;}
  }
  if(failure){
    if(childResult)console.log('IKOV_JSON_STORE_CHILD_FAILURE',JSON.stringify({
      error:childResult.error?{name:childResult.error.name,code:childResult.error.code,message:childResult.error.message}:null,
      status:childResult.status,signal:childResult.signal,
      stdoutBase64:Buffer.isBuffer(childResult.stdout)?childResult.stdout.toString('base64'):null,
      stderrBase64:Buffer.isBuffer(childResult.stderr)?childResult.stderr.toString('base64'):null}));
    throw failure;
  }
  disposeActor(f.p);
  console.log('IKOV_JSON_STORE',JSON.stringify({...observation,ownedRootCleaned:true}));
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
