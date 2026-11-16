// Successor fixture candidate, UNRUN; original d0ca ERROR70/six failures are retained. Supplied stage120 and waiting NPC are setup, not a
// full quest playthrough. No combat-roll/HP/damage/cache/route override.
process.env.DISABLE_PLAYER_BOTS = '1';
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after } = require('node:test');
require('../../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../../dist/game/cache/CachePipeline');
const { PluginManager: PM } = require('../../dist/plugins/PluginManager');
const { Player } = require('../../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../../dist/net/PlayerSession');
const { ClientConnection } = require('../../dist/net/NetworkBuilder');
const Runtime = require('../../plugins/quests/QuestRuntime');
let core, quest, serial = 0, oldError, oldSubmit;
const tasks=new Set();
const fixtures = new Set(), errors = [], applied = [], completed = [], started = [], ownerHits = [], transcriptEvents = [];
before(async () => {
  oldError = console.error;
  console.error = (...args) => { errors.push(args.map(String).join(' ')); oldError(...args); };
  await CachePipeline.initialize(path.resolve(__dirname, '../..'));
  PM.loadFromDirectory(path.resolve(__dirname, '../../plugins'));
  core = PM.getCoreApi(); core.RegionManager.init();
  const { NpcDefinitionLoader } = require('../../dist/game/definition/loader/impl/NpcDefinitionLoader');
  assert.equal(new NpcDefinitionLoader().load(), true);
  // Third argument is loadSurroundings, not the floor. Keep real nearby routes loaded.
  core.RegionManager.loadMapFiles(2435, 3517);
  PM.emitServerStartup({});
  oldSubmit=core.TaskManager.submit;
  core.TaskManager.submit=function(task,...args){tasks.add(task);return oldSubmit.call(this,task,...args);};
  quest = Runtime.getRegisteredQuests().find(q => q.key === 'monkey_madness_ii');
  assert.ok(quest, 'existing parent quest registered');
  const api = PM.createApi('NieveNativeObservation');
  api.onCustomEvent('combat:damage-applied', ({ target, hit }) => {
    applied.push({ target, source: hit.getSource(), lost: hit.getHealthLost(), hp: target.getHitpoints(),
      occurrence: target.getDeathOccurrence?.(), cycle: core.World.getProcessCycle() });
  });
  api.onCustomEvent('npc:death-completed', event => completed.push({ ...event }));
  api.onCustomEvent('combat:attack-started', event => started.push({ ...event, cycle:core.World.getProcessCycle() }));
  api.onPlayerDealtDamage(event => ownerHits.push({ ...event, damage:event.hit.getTotalDamage(), cycle:core.World.getProcessCycle() }));
  api.onCustomEvent('npc-dialogue:line',e=>transcriptEvents.push({...e,observationKind:'line'}));
  api.onCustomEvent('npc-dialogue:action',e=>transcriptEvents.push({...e,observationKind:'action'}));
  api.onCustomEvent('npc-dialogue:choice',e=>transcriptEvents.push({...e,observationKind:'choice'}));
});
after(() => {
  // Isolated native test-process shutdown ONLY. Observation does not prove all
  // tasks are fixture-owned, or establish whole-sandbox/process containment.
  for(const task of tasks)task.stop();
  if(core)core.TaskManager.submit=oldSubmit;
  console.error = oldError;
  assert.equal(fixtures.size, 0, 'fixture owners cleaned up');
  assert.deepEqual(errors, [], 'no swallowed engine/plugin exception');
});
function tick(n = 1) { for (let i = 0; i < n; i++) core.World.process(); }
function until(fn, n, message, state) {
  for (let i=0; i<n && !fn(); i++) tick();
  const result=fn();
  if(!result&&state)console.log(JSON.stringify({event:'nieve-fixture-prerequisite',message,state:state()}));
  assert.ok(result, message);
}
function describe(actor){
  if(!actor)return null;
  const at=actor.getLocation(), target=actor.getCombat().getTarget();
  return {id:actor.getId?.()??null,index:actor.getIndex(),registered:actor.isRegistered(),hp:actor.getHitpoints(),
    dying:actor.isDyingFunction(),x:at.getX(),y:at.getY(),z:at.getZ(),target:target?.getIndex()??null,
    privateArea:actor.getPrivateArea()?.constructor?.name??null};
}
function snapshot(f,...npcs){return {owner:describe(f.p),stage:quest.getStage(f.p),
  legacy:f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills'),
  migrated:f.p.getAttribute('monkey-madness-ii:nieve-legacy-migrated'),
  legacyCredit:f.p.getAttribute('monkey-madness-ii:nieve-legacy-credit'),
  mask:f.p.getAttribute('monkey-madness-ii:nieve-stronghold-mask'),
  pet:!!f.p.getAttribute('pets:current'),actors:npcs.map(describe),
  companionTasks:[...tasks].filter(t=>t.constructor.name==='CompanionTask'&&t.key===f.p).map(t=>({running:t.isRunning()}))};}
async function protectMelee(f){
  const group=core.CacheDefinitions.getInterfaceId('prayerbook');assert.ok(Number.isInteger(group),'actual cache prayer interface');
  const child=Array.from({length:64},(_,i)=>i).find(i=>core.PrayerHandler.prayerIdForChild(i)===core.PrayerHandler.PROTECT_FROM_MELEE);
  assert.ok(Number.isInteger(child),'existing canonical prayer producer maps protection');
  if(!core.PrayerHandler.isActivated(f.p,core.PrayerHandler.PROTECT_FROM_MELEE))
    await input(f,{type:'widget_action',groupId:group,childId:child,widgetId:(group<<16)|child,buttonNum:1});
  assert.equal(core.PrayerHandler.isActivated(f.p,core.PrayerHandler.PROTECT_FROM_MELEE),true,'real prayer widget activates protection');
  if(!f.prayerSupplied){
    const id=core.ItemIdentifiers.PRAYER_POTION_4_;assert.ok(Number.isInteger(id),'canonical prayer potion identifier');
    f.p.getInventory().adds(id,8);f.prayerSupplied=true;
  }
}
async function combatWait(f,target,fn,n,message){
  const start=core.World.getProcessCycle(),deadline=start+n;
  // ALL actual World cycles count, including inventory and re-engagement inputs.
  for(let i=0;i<n&&core.World.getProcessCycle()<deadline&&!fn();i++){
    assert.ok(f.p.isRegistered()&&f.p.getHitpoints()>0,JSON.stringify({message:'living owner prerequisite',state:snapshot(f,target)}));
    if(f.p.getSkillManager().getCurrentLevel(core.Skill.PRAYER)<20){
      const id=[4,3,2,1].map(d=>core.ItemIdentifiers[`PRAYER_POTION_${d}_`]).find(id=>Number.isInteger(id)&&f.p.getInventory().getAmount(id)>0);
      assert.ok(id,'supplied ordinary potion remains available');
      const old=f.p.getCombat().getTarget(),before=f.p.getSkillManager().getCurrentLevel(core.Skill.PRAYER);
      await input(f,{type:'inventory_action',widgetId:3214,itemId:id,slot:f.p.getInventory().getSlotForItemId(id),option:'Drink',optionIndex:1});
      assert.ok(f.p.getSkillManager().getCurrentLevel(core.Skill.PRAYER)>before,'real drink restores prayer');
      if(core.World.getProcessCycle()<deadline&&old===target&&target.getHitpoints()>0&&f.p.getCombat().getTarget()!==target){
        const clickType=target.getCurrentDefinition(f.p).getActions().indexOf('Attack')+1;
        await input(f,{type:'npc_option',index:target.getIndex(),clickType});
      }
    }
    if(core.World.getProcessCycle()<deadline&&!fn())tick();
  }
  const actualTicks=core.World.getProcessCycle()-start;
  assert.ok(actualTicks<=n,JSON.stringify({message:'actual observation tick cap',allowed:n,actualTicks}));
  const result=fn();if(!result)console.log(JSON.stringify({event:'nieve-fixture-prerequisite',message,state:snapshot(f,target)}));
  assert.ok(result,message);
}
function outsideRoute(f,enemy){
  const here=f.p.getLocation(),finder=new core.RsmodRouteFinding();
  for(let radius=1;radius<=12;radius++)for(const [dx,dy]of [[1,0],[0,1],[-1,0],[0,-1],[1,1],[-1,1],[-1,-1],[1,-1]]){
    const x=here.getX()+dx*radius,y=here.getY()+dy*radius;
    if(x>=2431&&x<=2440&&y>=3509&&y<=3519)continue;
    const at=new core.Location(x,y,here.getZ());if(core.RegionManager.blocked(at,f.p.getPrivateArea()))continue;
    const route=finder.findRoute({level:here.getZ(),srcX:here.getX(),srcY:here.getY(),srcSize:1,destX:x,destY:y,
      destWidth:1,destLength:1,locShape:-1,moveNear:false,privateArea:f.p.getPrivateArea()});
    if(!route.success||route.alternative||route.endX!==x||route.endY!==y)continue;
    // The positive also needs a real two-tile gorilla approach, not merely a
    // one-tile player path through an unusable gap. Query; never inject a route.
    if(enemy){
      const e=enemy.getLocation(),approach=finder.findRoute({level:e.getZ(),srcX:e.getX(),srcY:e.getY(),srcSize:enemy.getSize(),
        destX:x,destY:y,destWidth:1,destLength:1,locShape:-2,moveNear:false,privateArea:enemy.getPrivateArea()});
      if(!approach.success||approach.alternative)continue;
      if(approach.endX>=2431&&approach.endX<=2440&&approach.endY>=3509&&approach.endY<=3519)continue;
    }
    return at;
  }
  return null;
}
function owned(p, id) { return [...core.World.getNpcs()].filter(n => n?.getOwner?.()===p && n.getId()===id); }
function actor(t, stage = 120, loadedAttributes = {}) {
  const channel = { kind:'test', remoteAddress:'127.0.0.1', isOpen:()=>true, send:()=>true,
    close(){},onData(){},onError(){},onClose(){} };
  const session = new PlayerSession(channel), p = new Player(session), c = new ClientConnection(channel);
  c.player=p;session.setPlayer(p);session.setTurnPacketProcessor(()=>c.processTurnPackets());
  p.setUsername(`nieve${++serial}`);p.setLongUsername(core.Misc.stringToLongBigInt(p.getUsername()));
  p.setAttribute('bot-skip-persistence',true);p.setAutoRetaliate(false);
  // Supplied old-save state precedes first eligible placement/recovery; no disk-roundtrip claim.
  for(const [key,value]of Object.entries(loadedAttributes))p.setAttribute(key,value);
  assert.equal(core.World.getPlayers().add(p),true);
  for (const skill of [core.Skill.ATTACK,core.Skill.STRENGTH,core.Skill.DEFENCE,core.Skill.HITPOINTS,core.Skill.PRAYER])
    p.getSkillManager().setMaxLevels(skill,99,false).setCurrentLevels(skill,99,false);
  quest.setStage(p,stage);p.moveTo(new core.Location(2435,3517,0));
  until(()=>!p.isNeedsPlacement(),5,'native owner placement');
  const f={p,c};fixtures.add(f);
  t.after(()=>{
    PM.emitPlayerLogout({player:p,username:p.getUsername()});
    for(const task of tasks)if(task.npc?.getOwner?.()===p)task.stop();
    for(const n of [...core.World.getNpcs()])if(n?.getOwner?.()===p)PM.removeNpc(n);
    core.TaskManager.cancelTasks(p);core.TaskManager.cancelTasks(p.getIndex());
    core.World.getPlayers().remove(p);fixtures.delete(f);
    assert.equal(p.isRegistered(),false);
  });
  return f;
}
async function input(f,packet,turns=1){await f.c.dispatch([packet]);tick(turns);}
async function recruit(f,expectRecruit=true){
  const waiting=PM.spawnNpc({id:7108,x:2436,y:3517,z:0,owner:f.p,ownerOnly:true,wanderRadius:0});
  assert.ok(waiting?.isRegistered(),'supplied genuine waiting Nieve');
  const clickType=waiting.getCurrentDefinition(f.p).getActions().indexOf('Talk-to')+1;
  assert.ok(clickType>0,'actual cache Talk-to action');
  await input(f,{type:'npc_option',index:waiting.getIndex(),clickType},4);
  let accepted=false;
  for(let i=0;i<160&&(f.p.getDialogueManager().isActive()||core.MultiChatboxPrompt.getPending(f.p));i++){
    const current=f.p.getDialogueManager().getCurrent();
    const options=core.MultiChatboxPrompt.getPending(f.p)?.options??current?.getOptions?.()??[];
    if(options.length){
      const labels=options.map(o=>typeof o==='string'?o:o.getText?.()??o.text);
      let index=labels.indexOf('Yes.');
      if(index<0)index=labels.indexOf('Talk about Monkey Madness II.');
      assert.ok(index>=0,'known source recruitment introduction or Yes menu');
      if(labels[index]==='Yes.')accepted=true;
      await input(f,{type:'dialogue_continue',widgetId:(219<<16)|1,childIndex:index+1},4);
    }else{
      const group=f.p.getPacketSender().getChatboxGroupId(), child={193:0,229:4,11:4,217:5,231:5}[group];
      assert.ok(Number.isInteger(child),`native transcript continuation ${group}`);
      await input(f,{type:'dialogue_continue',widgetId:(group<<16)|child,childIndex:group===193?0:-1},4);
    }
  }
  assert.equal(accepted,true,'real Yes was selected, including pet-refusal controls');
  assert.equal(f.p.getDialogueManager().isActive(),false,'actual transcript finished');
  assert.equal(core.MultiChatboxPrompt.getPending(f.p),null,'no option prompt left behind');
  if(!expectRecruit){assert.equal(quest.getStage(f.p),120,'pet blocks real recruitment action');assert.equal(owned(f.p,7109).length,0);return null;}
  assert.equal(quest.getStage(f.p),140,'real recruitment advances parent');
  until(()=>owned(f.p,7109).length===1,8,'recruitment creates one registered owned helper');
  return owned(f.p,7109)[0];
}
async function offerRecruitment(f){
  const npc=PM.spawnNpc({id:7108,x:2436,y:3517,z:0,owner:f.p,ownerOnly:true,wanderRadius:0});
  assert.ok(npc?.isRegistered());
  const clickType=npc.getCurrentDefinition(f.p).getActions().indexOf('Talk-to')+1;
  assert.ok(clickType>0);
  await input(f,{type:'npc_option',index:npc.getIndex(),clickType},4);
  for(let i=0;i<160;i++){
    const pending=core.MultiChatboxPrompt.getPending(f.p);
    if(pending){
      const yes=pending.options.indexOf('Yes.');
      if(yes>=0){
        const actual=core.MultiChatboxPrompt.pendingPrompts.get(f.p);
        assert.ok(actual?.options[yes]?.callback);
        return {npc,yes,callback:actual.options[yes].callback};
      }
      const intro=pending.options.indexOf('Talk about Monkey Madness II.');
      assert.ok(intro>=0,'source introduction precedes recruitment menu');
      await input(f,{type:'dialogue_continue',widgetId:(219<<16)|1,childIndex:intro+1},4);
    }else{
      assert.equal(f.p.getDialogueManager().isActive(),true,'real recruitment chain remains active');
      const group=f.p.getPacketSender().getChatboxGroupId(),child={193:0,229:4,11:4,217:5,231:5}[group];
      assert.ok(Number.isInteger(child));
      await input(f,{type:'dialogue_continue',widgetId:(group<<16)|child,childIndex:group===193?0:-1},4);
    }
  }
  assert.fail('real offered Yes not reached within bound');
}
async function selectOfferedYes(f,offer){
  await input(f,{type:'dialogue_continue',widgetId:(219<<16)|1,childIndex:offer.yes+1},3);
}
async function assistancePositive(t){
  const f=actor(t), helper=await recruit(f);
  const from=helper.getLocation().clone();
  // Use ordinary queued player walking to an adjacent collision-valid route.
  const here=f.p.getLocation().clone();
  const candidates=[[4,0],[-4,0],[0,4],[0,-4]].map(([x,y])=>here.transform(x,y))
    .filter(at=>!core.RegionManager.blocked(at,null)&&core.RegionManager.canMovestart(here,at,1,1,null));
  assert.ok(candidates.length,'loaded Stronghold supplies a valid four-tile travel route');
  await input(f,{type:'move',worldX:candidates[0].getX(),worldY:candidates[0].getY(),modifierFlags:0,run:false});
  until(()=>f.p.getLocation().equals(candidates[0]),20,'ordinary player walk reaches route');
  until(()=>helper.getLocation().getDistance(from)>0,20,'helper follows by actual steps BEFORE combat');
  assert.equal(helper.getFollowing(),f.p,'ordinary follower pointer belongs to owner');
  assert.equal(started.some(e=>e.attacker===helper),false,'travel steps came before any helper attack');
  // Preserve the real precombat travel positive, then return through the real walk producer.
  await input(f,{type:'move',worldX:here.getX(),worldY:here.getY(),modifierFlags:0,run:false});
  until(()=>f.p.getLocation().equals(here),20,'ordinary return to interior before sustained combat');
  await protectMelee(f);
  assert.ok(helper.isRegistered()&&helper.getHitpoints()>0,'live companion before combat');
  // Source-backed owned Stronghold id, discovered by spawned location/owner rather
  // than a new-module import. No forced attack roll or direct HP damage.
  const gorillas=[...core.World.getNpcs()].filter(n=>n?.getOwner?.()===f.p&&n.getId()===7097);
  assert.equal(gorillas.length,4,'four real quest actors');
  const anchors=gorillas.map(n=>{const p=n.getSpawnPosition();return `${p.getX()},${p.getY()},${p.getZ()}`;}).sort();
  assert.deepEqual(anchors,['2436,3518,0','2439,3516,0','2437,3512,0','2432,3510,0'].sort(),
    'four distinct source spawn anchors, not merely four objects');
  const target=gorillas.sort((a,b)=>a.getLocation().getDistance(f.p.getLocation())-b.getLocation().getDistance(f.p.getLocation()))[0];
  const actions=target.getCurrentDefinition(f.p).getActions(),clickType=actions.indexOf('Attack')+1;
  assert.ok(clickType>0,'actual target cache Attack');
  await input(f,{type:'npc_option',index:target.getIndex(),clickType},1);
  until(()=>applied.some(e=>e.target===target&&e.source===helper&&e.lost>0),240,
    'unchanged canonical helper combat causes actual positive target HP loss');
  assert.ok(helper.getLocation().getDistance(from)>0,'helper actually moved, not only a following flag');
  return {f,helper,target};
}
module.exports={actor,recruit,offerRecruitment,selectOfferedYes,transcriptEvents,assistancePositive,tick,until,input,owned,applied,completed,
  started,ownerHits,tasks,snapshot,protectMelee,combatWait,outsideRoute,get core(){return core;},get quest(){return quest;},PM};
