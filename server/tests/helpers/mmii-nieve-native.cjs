// Native candidate, UNRUN. Supplied stage120 and waiting NPC are setup, not a
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
  core.RegionManager.loadMapFiles(2435, 3517, 0);
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
function until(fn, n, message) { for (let i=0; i<n && !fn(); i++) tick(); assert.ok(fn(), message); }
function owned(p, id) { return [...core.World.getNpcs()].filter(n => n?.getOwner?.()===p && n.getId()===id); }
function actor(t, stage = 120) {
  const channel = { kind:'test', remoteAddress:'127.0.0.1', isOpen:()=>true, send:()=>true,
    close(){},onData(){},onError(){},onClose(){} };
  const session = new PlayerSession(channel), p = new Player(session), c = new ClientConnection(channel);
  c.player=p;session.setPlayer(p);session.setTurnPacketProcessor(()=>c.processTurnPackets());
  p.setUsername(`nieve${++serial}`);p.setLongUsername(core.Misc.stringToLongBigInt(p.getUsername()));
  p.setAttribute('bot-skip-persistence',true);p.setAutoRetaliate(false);
  assert.equal(core.World.getPlayers().add(p),true);
  for (const skill of [core.Skill.ATTACK,core.Skill.STRENGTH,core.Skill.DEFENCE,core.Skill.HITPOINTS])
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
  const here=f.p.getLocation();
  const candidates=[[4,0],[-4,0],[0,4],[0,-4]].map(([x,y])=>here.transform(x,y))
    .filter(at=>!core.RegionManager.blocked(at,null)&&core.RegionManager.canMovestart(here,at,1,1,null));
  assert.ok(candidates.length,'loaded Stronghold supplies a valid four-tile travel route');
  await input(f,{type:'move',worldX:candidates[0].getX(),worldY:candidates[0].getY(),modifierFlags:0,run:false});
  until(()=>f.p.getLocation().equals(candidates[0]),20,'ordinary player walk reaches route');
  until(()=>helper.getLocation().getDistance(from)>0,20,'helper follows by actual steps BEFORE combat');
  assert.equal(helper.getFollowing(),f.p,'ordinary follower pointer belongs to owner');
  assert.equal(started.some(e=>e.attacker===helper),false,'travel steps came before any helper attack');
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
  started,ownerHits,tasks,get core(){return core;},get quest(){return quest;},PM};
