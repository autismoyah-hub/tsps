// Air altar uses an owned synthetic object at the standard fixture tile; no natural scene or live client claim.
const assert=require('node:assert/strict'), path=require('node:path');
const {before,after,test}=require('node:test');
process.env.DISABLE_PLAYER_BOTS='1';
require('../dist/Server').Server.installProductionPathResolver();
const {CachePipeline}=require('../dist/game/cache/CachePipeline');
const {PluginManager:PM}=require('../dist/plugins/PluginManager');
const {Player}=require('../dist/game/entity/impl/player/Player');
const {PlayerSession}=require('../dist/net/PlayerSession');
const {ClientConnection}=require('../dist/net/NetworkBuilder');
const {SkillManager}=require('../dist/game/content/skill/SkillManager');
const {MAIN_PROFILE}=require('../dist/game/entity/impl/player/persistence/PlayerProfile');
const {isMembersWorld}=require('../dist/game/definition/WorldDefinition');
const {MapObjects}=require('../dist/game/entity/impl/object/MapObjects');
const {ObjectDefinition}=require('../dist/game/definition/ObjectDefinition');
const {PathFinder}=require('../dist/game/model/movement/path/PathFinder');
const {GameConstants}=require('../dist/game/GameConstants');
const {MultiChatboxPrompt}=require('../dist/game/model/menu/MultiChatboxPrompt');
let core, next=0, altarId; const errors=[], originalError=console.error, providerAnswers=new WeakMap();
console.error=(...args)=>{errors.push(args);originalError(...args);};
before(async()=>{await CachePipeline.initialize(path.resolve(__dirname,'..')); PM.loadFromDirectory(path.resolve(__dirname,'../plugins')); core=PM.getCoreApi();core.RegionManager.init();core.RegionManager.loadMapFiles(3222,3218);PM.emitServerStartup({}); PM.createApi('AirDaeyaltProviderObserver').onCustomEvent('runecrafting:essence',event=>{const prior=providerAnswers.get(event.player)??[]; prior.push([...event.pure]); providerAnswers.set(event.player,prior);}); altarId=core.CacheDefinitions.getLocId('air_altar');});
after(()=>{try{PM.emitServerShutdown({});}finally{console.error=originalError;assert.deepEqual(errors,[],'no caught startup, action or cleanup errors');}});
function actor(t) {
 const channel={kind:'test',remoteAddress:'127.0.0.1',isOpen:()=>true,send:()=>true,close(){},onData(){},onError(){},onClose(){}};
 const p=new Player(new PlayerSession(channel)), c=new ClientConnection(channel), name=`airdaeyalt${++next}`; let altar=null, registered=false, tiles=[]; const api=PM.createApi('AirDaeyaltOwnedCleanup');
 t.after(()=>{try{try{p.closeInterruptibleInterfaces();}finally{try{core.TaskManager.cancelTasks(p);core.TaskManager.cancelTasks(p.getIndex());for(const n of new Set([...core.World.getNpcs(),...core.World.getAddNPCQueue()]))if(n?.getOwner?.()===p)api.removeNpc(n);}finally{if(registered)core.ObjectManager.deregister(altar,true);}}}finally{try{if(p.isRegistered())core.World.getPlayers().remove(p);}finally{assert.ok(!p.isRegistered());assert.equal(core.TaskManager.hasActiveTask(p),false);assert.equal(core.TaskManager.hasActiveTask(p.getIndex()),false);assert.equal(MultiChatboxPrompt.getPending(p),null);assert.equal([...core.World.getNpcs()].some(n=>n?.getOwner?.()===p),false);assert.equal(core.World.getAddNPCQueue().some(n=>n?.getOwner?.()===p),false);for(const tile of tiles){assert.equal(core.RegionManager.getClipping(tile.x,tile.y,0,null),tile.clip);assert.deepEqual(MapObjects.getObjectsAt(tile.loc,null),tile.map);assert.deepEqual([...core.ObjectManager.objectsAt(tile.loc)],tile.runtime);assert.deepEqual(core.World.getRemovedObjects().filter(o=>o?.getLocation()?.equals(tile.loc)),tile.removed);}assert.deepEqual(errors,[]);}}});
 p.setUsername(name);p.setLongUsername(core.Misc.stringToLongBigInt(name));assert.notEqual(p.getLongUsername(),0n);p.setAttribute('bot-skip-persistence',true);c.player=p;p.getSession().setPlayer(p);p.getSession().setTurnPacketProcessor(()=>c.processTurnPackets());assert.equal(core.World.getPlayers().add(p),true);p.moveTo(new core.Location(3222,3218,0));p.setNeedsPlacement(false);assert.equal(p.isNeedsPlacement(),false);assert.ok(p.getHitpoints()>0);assert.equal(p.busy(),false);
 assert.equal(isMembersWorld(),true);assert.equal(GameConstants.EXPERIENCE_MULTIPLIER,1);assert.equal(p.getProfile(),MAIN_PROFILE);const league=require('../plugins/modes/league/Common.LeagueMode');assert.equal(league.worldLeague(),null);const sentinel=Symbol('unanswered'), query={player:p,league:sentinel};PM.emitCustomEvent('league:profile',query);assert.equal(query.league,null,'actual league provider answered main profile');t.diagnostic(`league:type raw=${String(league.api.getPluginConfig('league:type',0))}`);
 const skill=core.Skill.RUNECRAFTING, skills=p.getSkillManager();skills.setMaxLevels(skill,1,false).setExperience(skill,SkillManager.getExperienceForLevel(1)).setCurrentLevel(skill,1,false);assert.equal(skills.getMaxLevel(skill),1);assert.equal(skills.getCurrentLevel(skill),1);
 assert.ok(altarId>0);assert.match(core.ItemDefinition.forId(core.ItemIdentifiers.PURE_ESSENCE).getName(),/^Pure essence$/i);assert.match(core.ItemDefinition.forId(core.ItemIdentifiers.DAEYALT_ESSENCE).getName(),/^Daeyalt essence$/i);const def=ObjectDefinition.forPlayer(altarId,p), footprint=ObjectDefinition.forId(altarId)??def;assert.ok(def&&footprint);
 // All candidates and their footprint borders are in this same loaded region.
 assert.ok(core.RegionManager.getRegion(3222,3218)?.isLoaded(),'synthetic candidates are in a loaded region');
 // Select an entirely clear synthetic footprint + border before registering anything.
 let loc=null;
 for(let x=3210;x<=3230&&!loc;x++)for(let y=3210;y<=3230&&!loc;y++){
  const trial=[];let clear=true;
  for(let tx=x-1;tx<=x+footprint.getSizeX();tx++)for(let ty=y-1;ty<=y+footprint.getSizeY();ty++){const tileLoc=new core.Location(tx,ty,0),map=[...MapObjects.getObjectsAt(tileLoc,null)],runtime=[...core.ObjectManager.objectsAt(tileLoc)],clip=core.RegionManager.getClipping(tx,ty,0,null),removed=core.World.getRemovedObjects().filter(o=>o?.getLocation()?.equals(tileLoc));if(map.length||runtime.length||removed.length||clip!==0)clear=false;trial.push({x:tx,y:ty,loc:tileLoc,map,runtime,clip,removed});}
  if(clear){loc=new core.Location(x,y,0);tiles=trial;}
 }
 assert.ok(loc,'a clean synthetic footprint and border are available');p.moveTo(new core.Location(loc.getX()-1,loc.getY(),0));p.setNeedsPlacement(false);
 altar=new core.GameObject(altarId,loc,10,0,null);registered=true;core.ObjectManager.register(altar,true);assert.ok(core.ObjectManager.objectsAt(loc).includes(altar));assert.equal(core.World.findCacheObject(p,altarId,loc),altar);
 assert.equal(core.RegionManager.blocked(p.getLocation(),p.getPrivateArea()),false);assert.equal(PathFinder.reachedObject(p,loc.getX(),loc.getY(),footprint.getSizeX(),footprint.getSizeY(),altar.getFace(),10,def.getBlockingMask()),true,'synthetic altar reachable from actual standing tile');const actions=def.getInteractions(), index=actions.findIndex(x=>x?.toLowerCase()==='craft-rune');assert.ok(index>=0,'active altar exposes Craft-rune');return {p,c,skill,skills,index,altar,loc};
}
async function craft(f){const before=(providerAnswers.get(f.p)??[]).length;await f.c.dispatch([{type:'object_option',id:altarId,x:f.loc.getX(),y:f.loc.getY(),clickType:f.index+1}]);for(let i=0;i<15;i++)core.World.process();const answers=(providerAnswers.get(f.p)??[]).slice(before);assert.ok(answers.length>0,'registered essence query reached');assert.ok(answers.every(ids=>ids.every(Number.isInteger)));return answers;}
const amount=(f,id)=>f.p.getInventory().getAmount(id);
test('ordinary pure essence crafts at the real Air consumer',async t=>{const f=actor(t), i=core.ItemIdentifiers;f.p.getInventory().add(new core.Item(i.PURE_ESSENCE,1),true);const xp=f.skills.getExperience(f.skill);const answers=await craft(f);t.diagnostic('actual essence provider IDs '+JSON.stringify(answers));assert.equal(amount(f,i.PURE_ESSENCE),0);assert.equal(amount(f,i.AIR_RUNE),1);assert.equal(f.skills.getExperience(f.skill)-xp,5);});
test('preowned Daeyalt crafts Air with its XP',async t=>{const f=actor(t), i=core.ItemIdentifiers;f.p.getInventory().add(new core.Item(i.DAEYALT_ESSENCE,1),true);const xp=f.skills.getExperience(f.skill);const answers=await craft(f);t.diagnostic('actual essence provider IDs '+JSON.stringify(answers));assert.equal(amount(f,i.DAEYALT_ESSENCE),0);assert.equal(amount(f,i.AIR_RUNE),1);assert.equal(f.skills.getExperience(f.skill)-xp,7.5);});
test('mixed Daeyalt is used first and pure requires another action',async t=>{const f=actor(t), i=core.ItemIdentifiers;f.p.getInventory().add(new core.Item(i.PURE_ESSENCE,1),true);f.p.getInventory().add(new core.Item(i.DAEYALT_ESSENCE,1),true);const xp=f.skills.getExperience(f.skill);const firstAnswers=await craft(f);t.diagnostic('first action provider IDs '+JSON.stringify(firstAnswers));assert.equal(amount(f,i.DAEYALT_ESSENCE),0);assert.equal(amount(f,i.PURE_ESSENCE),1);assert.equal(amount(f,i.AIR_RUNE),1);assert.equal(f.skills.getExperience(f.skill)-xp,7.5);const secondAnswers=await craft(f);t.diagnostic('second action provider IDs '+JSON.stringify(secondAnswers));assert.equal(amount(f,i.PURE_ESSENCE),0);assert.equal(amount(f,i.AIR_RUNE),2);assert.equal(f.skills.getExperience(f.skill)-xp,12.5);});
