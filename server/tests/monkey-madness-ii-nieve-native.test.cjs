// Candidate UNRUN; baseline is separate unchanged-source overlay, not a result.
const assert=require('node:assert/strict');
const {test}=require('node:test');
const h=require('./helpers/mmii-nieve-native.cjs');

test('Nieve real recruitment, travelling and Stronghold ordinary HP assistance positive',async t=>{
  await h.assistancePositive(t);
});
test('Nieve airship-only action cannot recruit or spawn helper; earned145 survives recovery',t=>{
  const f=h.actor(t,110);
  h.PM.emitCustomEvent('npc-dialogue:action',{player:f.p,npcId:7118,stepId:'FWgGzC'});
  assert.ok(h.quest.getStage(f.p)<140);assert.equal(h.owned(f.p,7109).length,0);
  h.quest.setStage(f.p,145);
  h.PM.emitPlayerLogin({player:f.p});h.tick(3);
  assert.equal(h.quest.getStage(f.p),145);
});
test('Nieve recall suppresses several automatic swings until observed break and a new real engagement',async t=>{
  const {f,helper,target}=await h.assistancePositive(t);
  const component=h.core.CacheDefinitions.getComponentId('wornitems:call_follower');
  const e={player:f.p,packet:{widgetId:component,buttonNum:1},handled:false};
  h.PM.emitCustomEvent('player:widget-action',e);assert.equal(e.handled,true);
  const before=h.started.length;
  h.tick(32);
  assert.equal(h.started.slice(before).some(x=>x.attacker===helper&&x.target===target),false,
    'several auto swings at the continuing owner target cannot restart helper');
  f.p.getCombat().reset();h.tick(2);
  const clickType=target.getCurrentDefinition(f.p).getActions().indexOf('Attack')+1;
  await h.input(f,{type:'npc_option',index:target.getIndex(),clickType});
  h.until(()=>h.started.slice(before).some(x=>x.attacker===helper&&x.target===target),240,
    'observed null then new ordinary owner attack can rearm');
});
test('Nieve forged widget op and foreign completion cannot change durable progress',async t=>{
  const f=h.actor(t),helper=await h.recruit(f);
  const before=f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills')??0;
  const e={player:f.p,packet:{widgetId:h.core.CacheDefinitions.getComponentId('wornitems:call_follower'),buttonNum:2},handled:false};
  h.PM.emitCustomEvent('player:widget-action',e);assert.equal(e.handled,true);
  const enemy=[...h.core.World.getNpcs()].find(n=>n?.getOwner?.()===f.p&&n.getId()===7097);assert.ok(enemy);
  h.PM.emitCustomEvent('npc:death-completed',{npc:enemy,occurrence:{},killer:null});
  assert.equal(f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills')??0,before);
});
test('Nieve owner logout invalidates generation and removes only that owner actors; earned stage remains',async t=>{
  const f=h.actor(t),g=h.actor(t),helper=await h.recruit(f);
  h.PM.emitPlayerLogout({player:f.p,username:f.p.getUsername()});h.tick(2);
  assert.equal(helper.isRegistered(),false);assert.equal(g.p.isRegistered(),true);
  assert.equal(h.quest.getStage(f.p),140);assert.equal(h.owned(f.p,7109).length,0);
});

test('Nieve real guarded Call follower recovers a missing helper as a fresh generation without attacking',async t=>{
  const f=h.actor(t),old=await h.recruit(f);
  h.PM.removeNpc(old);h.tick(2);
  assert.equal(old.isRegistered(),false);assert.equal(h.owned(f.p,7109).length,0);
  const id=h.core.CacheDefinitions.getComponentId('wornitems:call_follower');
  await h.input(f,{type:'widget_action',widgetId:id,groupId:id>>>16,childId:id&65535,slot:-1,buttonNum:1},2);
  const replacement=h.owned(f.p,7109);
  assert.equal(replacement.length,1,'actual Call recovers one helper');assert.notEqual(replacement[0],old);
  assert.equal(replacement[0].getCombat().getTarget(),null);h.tick(4);
  assert.equal(replacement[0].getCombat().getTarget(),null,'next ticks cannot start an attack');
});

test('generic NPC natural completion occurs once and unregister/re-register rejects an old death task',t=>{
  const f=h.actor(t), npc=h.PM.spawnNpc({id:7097,x:2439,y:3516,z:0,owner:f.p,ownerOnly:true,wanderRadius:0});
  assert.ok(npc.isRegistered());
  npc.setHitpoints(0);const occurrence=npc.getDeathOccurrence();assert.ok(occurrence);
  h.until(()=>h.completed.some(e=>e.npc===npc&&e.occurrence===occurrence),40,'ordinary natural death boundary');
  h.tick(10);
  assert.equal(h.completed.filter(e=>e.npc===npc&&e.occurrence===occurrence).length,1);
  // Direct setter is lifecycle setup ONLY, not helper combat/HP-loss evidence.
  const second=h.PM.spawnNpc({id:7097,x:2437,y:3512,z:0,owner:f.p,ownerOnly:true,wanderRadius:0});
  second.setHitpoints(0);const old=second.getDeathOccurrence();assert.ok(old);
  h.PM.removeNpc(second);second.setHitpoints(second.getMaxHitpoints());
  assert.equal(h.core.World.getNpcs().add(second),true);
  h.tick(40);assert.equal(second.isRegistered(),true,'old task cannot remove replacement life');
  assert.equal(second.getHitpoints(),second.getMaxHitpoints());
  assert.equal(h.completed.some(e=>e.npc===second&&e.occurrence===old),false);
});

test('generic beforeDeath heal/re-kill coalesces one deferred ordinary evaluation without borrowing a helper witness',t=>{
  const f=h.actor(t), npc=h.PM.spawnNpc({id:7097,x:2437,y:3512,z:0,owner:f.p,ownerOnly:true,wanderRadius:0});
  const api=h.PM.createApi('NieveReentryControl');let entered=0, depth=0,maxDepth=0;
  api.onNpcBeforeDeath(e=>{
    if(e.npc!==npc)return;
    depth++;maxDepth=Math.max(maxDepth,depth);
    if(++entered===1){npc.setHitpoints(npc.getMaxHitpoints());npc.setHitpoints(0);}
    depth--;
  });
  npc.setHitpoints(0);
  assert.equal(maxDepth,1,'beforeDeath callbacks cannot synchronously recurse');
  assert.equal(npc.getDeathOccurrence(),null,'deferred acceptance was not present in earlier synchronous event');
  h.until(()=>h.completed.some(e=>e.npc===npc),50,'new life reaches ordinary deferred natural death');
  assert.equal(entered,2,'one later ordinary entry evaluation');
  assert.equal(h.completed.filter(e=>e.npc===npc).length,1);
  assert.equal(f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills')??0,0,'foreign setup actor grants no ally progress');
});


test('generic positive HP recovery restores a living actor without external dying/unblock rescue, then accepts fresh death',t=>{
  const f=h.actor(t),npc=h.PM.spawnNpc({id:7097,x:2439,y:3516,z:0,owner:f.p,ownerOnly:true,wanderRadius:0});
  npc.setHitpoints(0);const old=npc.getDeathOccurrence();
  h.until(()=>npc.getMovementQueue().isMovementBlocked(),5,'death case1 owns movement hold');
  assert.equal(npc.isDyingFunction(),true);assert.equal(npc.getMovementQueue().isMovementBlocked(),true);
  npc.setHitpoints(npc.getMaxHitpoints());
  assert.equal(npc.isDyingFunction(),false);assert.equal(npc.getMovementQueue().isMovementBlocked(),false);
  assert.equal(npc.getDeathOccurrence(),null);h.tick(2);assert.equal(npc.isRegistered(),true);
  npc.setHitpoints(0);const fresh=npc.getDeathOccurrence();assert.ok(fresh);assert.notEqual(fresh,old);
  h.until(()=>h.completed.some(e=>e.npc===npc&&e.occurrence===fresh),40,'fresh life naturally dies');
  assert.equal(h.completed.some(e=>e.npc===npc&&e.occurrence===old),false);
});

test('Nieve actual helper-only lethal combat has empty Player credit, then four distinct slots progress once',async t=>{
  const f=h.actor(t);await h.recruit(f);
  // A genuine low-level owner may miss. Observe ordinary rolls; never force one.
  f.p.getSkillManager().setMaxLevels(h.core.Skill.ATTACK,1,false).setCurrentLevels(h.core.Skill.ATTACK,1,false);
  const api=h.PM.createApi('NieveMissObservation');let next=null;
  api.onPlayerDealtDamage(e=>{
    if(e.player!==f.p||next)return;
    next={target:e.target,damage:e.hit.getTotalDamage()};
    // Stop repeated owner attacks by an ordinary combat reset, after actual source engagement.
    f.p.getCombat().reset();
  });
  let target=null, helper=null, legacyForTarget=0, duplicateChecked=false;
  const duplicatePlayerDeaths=new Set();
  api.onCustomEvent('npc:death-completed',e=>{
    if(e.killer!==f.p||e.npc.getId()!==7097||e.npc.getOwner()!==f.p||duplicatePlayerDeaths.has(e.npc))return;
    duplicatePlayerDeaths.add(e.npc);
    assert.equal(e.npc.isRegistered(),true);assert.equal(e.npc.isDyingFunction(),true);
    assert.equal(e.npc.getHitpoints(),0);assert.equal(e.npc.getDeathOccurrence(),e.occurrence);
    const count=f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills');
    h.PM.emitCustomEvent('npc:death-completed',{...e});
    assert.equal(f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills'),count,
      'owner Player duplicate reaches the live slot-bit guard before removal');
  });
  api.onNpcDeath(e=>{if(e.npc===target)legacyForTarget++;});
  api.onCustomEvent('npc:death-completed',e=>{
    if(e.npc!==target||duplicateChecked)return;
    duplicateChecked=true;
    assert.equal(target.isRegistered(),true);assert.equal(target.isDyingFunction(),true);
    assert.equal(target.getHitpoints(),0);assert.equal(target.getDeathOccurrence(),e.occurrence);
    assert.equal(h.core.World.getRemoveNPCQueue().includes(target),false);
    const count=f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills');
    assert.equal(count,1,'original consumer committed before this later observer');
    const observed=h.completed.filter(x=>x.npc===target).length;
    const task=[...h.tasks].find(task=>task.npc===target&&task.constructor.name==='NPCDeathTask');
    assert.ok(task);task.execute();
    assert.equal(h.completed.filter(x=>x.npc===target).length,observed,'same-task reentry emits no second completion');
    h.PM.emitCustomEvent('npc:death-completed',{...e});
    assert.equal(f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills'),count,
      'duplicate consumer dispatch while occurrence is STILL current grants no second slot');
  });
  for(let attempt=0;attempt<12&&!target;attempt++){
    next=null;
    const enemy=[...h.core.World.getNpcs()].find(n=>n?.getOwner?.()===f.p&&n.getId()===7097);
    assert.ok(enemy);helper=h.owned(f.p,7109)[0];
    const clickType=enemy.getCurrentDefinition(f.p).getActions().indexOf('Attack')+1;
    await h.input(f,{type:'npc_option',index:enemy.getIndex(),clickType});
    h.until(()=>next!==null,80,'real owner attack reaches ordinary queued-hit boundary');
    if(next.damage===0){target=enemy;break;}
    // A damaging attempt is genuine but cannot prove empty Player credit. Fresh
    // encounter recovery replaces its actors; never erase a living damage map.
    h.PM.emitPlayerLogout({player:f.p,username:f.p.getUsername()});
    h.PM.emitPlayerLogin({player:f.p});h.tick(2);
  }
  assert.ok(target,'bounded genuine owner miss supplies an empty-credit assistance attempt');
  assert.deepEqual(target.getCombat().getRecentDamagers(),[]);
  const before=h.applied.length;
  const skills=[h.core.Skill.ATTACK,h.core.Skill.STRENGTH,h.core.Skill.DEFENCE,h.core.Skill.HITPOINTS,h.core.Skill.SLAYER];
  const xp=skills.map(skill=>f.p.getSkillManager().getExperience(skill));
  h.until(()=>h.completed.some(e=>e.npc===target),1600,'actual canonical helper-only HP0 reaches natural completion');
  const death=h.completed.find(e=>e.npc===target);
  assert.equal(death.killer,null,'no fabricated Player killer');
  assert.equal(legacyForTarget,0,'Player-only death/reward event not manufactured');
  assert.deepEqual(skills.map(skill=>f.p.getSkillManager().getExperience(skill)),xp,'helper-only kill grants no player combat/Slayer XP');
  assert.ok(h.applied.slice(before).some(e=>e.target===target&&e.source===helper&&e.hp===0&&e.lost>0));
  assert.equal(h.applied.slice(before).some(e=>e.target===target&&e.source===f.p&&e.lost>0),false);
  assert.equal(f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills'),1);
  h.PM.emitCustomEvent('npc:death-completed',death);
  assert.equal(f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills'),1,'duplicate genuine occurrence cannot increment');
  f.p.getSkillManager().setMaxLevels(h.core.Skill.ATTACK,99,false).setCurrentLevels(h.core.Skill.ATTACK,99,false);
  assert.equal(duplicateChecked,true,'duplicate was exercised during actual pre-removal completion');
  const known=['2436,3518,0','2439,3516,0','2437,3512,0','2432,3510,0'];
  const anchor=n=>{const p=n.getSpawnPosition();return `${p.getX()},${p.getY()},${p.getZ()}`;};
  assert.ok(known.includes(anchor(target)));
  const killed=new Set([target]),killedSlots=new Set([anchor(target)]);
  for(let slot=1;slot<4;slot++){
    const enemy=[...h.core.World.getNpcs()].find(n=>n?.getOwner?.()===f.p&&n.getId()===7097&&!killedSlots.has(anchor(n)));
    assert.ok(enemy,'different unearned source anchor remains present');assert.ok(known.includes(anchor(enemy)));
    killed.add(enemy);killedSlots.add(anchor(enemy));next={};
    const clickType=enemy.getCurrentDefinition(f.p).getActions().indexOf('Attack')+1;
    await h.input(f,{type:'npc_option',index:enemy.getIndex(),clickType});
    h.until(()=>h.completed.some(e=>e.npc===enemy),800,'ordinary player/ally combat completes another real slot');
    assert.equal(f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills'),slot+1);
  }
  assert.ok(duplicatePlayerDeaths.size>0,'actual Player death route exercised live-slot duplicate guard');
  assert.equal(killed.size,4);assert.equal(killedSlots.size,4);
  assert.deepEqual([...killedSlots].sort(),known.slice().sort());
  assert.equal(f.p.getAttribute('monkey-madness-ii:nieve-stronghold-mask'),15,'all four durable source-slot bits committed');
  assert.equal(h.quest.getStage(f.p),145);
  h.PM.emitPlayerLogout({player:f.p,username:f.p.getUsername()});h.PM.emitPlayerLogin({player:f.p});h.tick(3);
  assert.equal(h.quest.getStage(f.p),145);assert.equal(f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills'),4);
  assert.equal([...h.core.World.getNpcs()].filter(n=>n?.getOwner?.()===f.p&&n.getId()===7097).length,0);
});

test('Nieve legacy count is capped and migrated once across login; earned145 is not regressed',t=>{
  const f=h.actor(t,140);f.p.setAttribute('quest.monkey_madness_ii.stronghold-kills',9);
  h.PM.emitPlayerLogin({player:f.p});h.tick(2);
  assert.equal(f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills'),4);
  assert.equal(f.p.getAttribute('monkey-madness-ii:nieve-legacy-credit'),4);
  assert.equal(h.quest.getStage(f.p),145);
  h.PM.emitPlayerLogout({player:f.p,username:f.p.getUsername()});h.PM.emitPlayerLogin({player:f.p});h.tick(2);
  assert.equal(f.p.getAttribute('monkey-madness-ii:nieve-legacy-credit'),4);
  assert.equal(f.p.getAttribute('monkey-madness-ii:nieve-stronghold-mask')??0,0,'unknown historical slots not invented');
  assert.equal(h.quest.getStage(f.p),145);
});

test('Nieve real awarded pet excludes recruitment recovery and unhandled Call retains ordinary pet behavior',async t=>{
  const f=h.actor(t,140);
  h.PM.emitCustomEvent('npc-drops:roll',{player:f.p,drops:[{itemId:h.core.ItemIdentifiers.OLMLET,amount:1}]});
  const pet=f.p.getAttribute('pets:current');assert.ok(pet?.isRegistered(),'actual pet award producer');
  h.PM.emitPlayerLogin({player:f.p});h.tick(2);assert.equal(h.owned(f.p,7109).length,0);
  const id=h.core.CacheDefinitions.getComponentId('wornitems:call_follower');
  await h.input(f,{type:'widget_action',widgetId:id,groupId:id>>>16,childId:id&65535,slot:-1,buttonNum:1},2);
  assert.ok(f.p.getAttribute('pets:current')?.isPet());assert.equal(h.owned(f.p,7109).length,0);
});

test('Nieve common busy guard denies missing-helper recovery before quest custom request',async t=>{
  const f=h.actor(t,140);f.p.setStatus(h.core.PlayerStatus.TRADING);
  const id=h.core.CacheDefinitions.getComponentId('wornitems:call_follower');
  await h.input(f,{type:'widget_action',widgetId:id,groupId:id>>>16,childId:id&65535,slot:-1,buttonNum:1},1);
  assert.equal(h.owned(f.p,7109).length,0);f.p.setStatus(h.core.PlayerStatus.NONE);
});

for(const phase of ['before-case1','after-case1-with-prior-block'])test(`generic recovery ${phase} preserves prior movement restriction and accepts second death`,t=>{
  const f=h.actor(t),npc=h.PM.spawnNpc({id:7097,x:2439,y:3516,z:0,owner:f.p,ownerOnly:true,wanderRadius:0});
  const movement=npc.getMovementQueue(),prior=phase.includes('prior-block');
  movement.setBlockMovement(prior);const generation=movement.getBlockMovementGeneration();
  npc.setHitpoints(0);const old=npc.getDeathOccurrence();
  if(prior)h.until(()=>movement.getBlockMovementGeneration()!==generation,5,'case1 took its own hold');
  npc.setHitpoints(npc.getMaxHitpoints());
  assert.equal(npc.isDyingFunction(),false);assert.equal(movement.isMovementBlocked(),prior);
  npc.setHitpoints(0);const fresh=npc.getDeathOccurrence();assert.ok(fresh);assert.notEqual(old,fresh);
  h.until(()=>h.completed.some(e=>e.npc===npc&&e.occurrence===fresh),40,'second life natural completion');
  assert.equal(h.completed.some(e=>e.npc===npc&&e.occurrence===old),false);
});

async function playerKillWithRemains(t,revive){
  const f=h.actor(t),preexisting=new Set(h.core.World.getNpcs()),key=`nieve-native-remains:${f.p.getUsername()}`;
  const npc=h.PM.spawnNpc({id:7097,x:2436,y:3518,z:0,wanderRadius:0,unique:key});
  const spawn=npc.getSpawnPosition().clone(),derived=new Set([npc]),added=[];
  // Ordinary unowned spawns naturally respawn; owner-scoped quest spawns deliberately do not.
  const api=h.PM.createApi('NieveRemainsControl');let legacy=0;
  api.onCustomEvent('npc:added',e=>{
    const n=e.npc;
    if(!preexisting.has(n)&&n.getUniqueKey()===key&&n.getSpawnPosition().equals(spawn)){
      derived.add(n);added.push(n);
    }
  });
  t.after(()=>{
    for(const task of h.tasks)if(task.npc?.getUniqueKey?.()===key)task.stop();
    for(const n of derived)h.PM.removeNpc(n);
    for(const n of preexisting)if(n.getId()===7097&&n.getOwner()==null)
      assert.equal(n.isRegistered(),true,'unrelated preexisting same-ID spawns preserved');
  });
  api.onNpcDeath(e=>{if(e.npc===npc){legacy++;e.remains={ticks:4,respawnTicks:3};}});
  const clickType=npc.getCurrentDefinition(f.p).getActions().indexOf('Attack')+1;
  await h.input(f,{type:'npc_option',index:npc.getIndex(),clickType});
  h.until(()=>h.completed.some(e=>e.npc===npc),800,'real owner combat and legacy remains negotiation');
  assert.equal(legacy,1);assert.equal(npc.isRegistered(),true);assert.equal(npc.isDyingFunction(),false);
  assert.equal(npc.getHitpoints(),0);assert.equal(npc.getDeathOccurrence(),null);
  f.p.getCombat().reset();
  if(revive){
    npc.setHitpoints(npc.getMaxHitpoints());h.tick(12);
    assert.equal(npc.isRegistered(),true,'old remains cannot remove revived corpse');
    assert.equal(npc.isDyingFunction(),false);assert.equal(npc.getMovementQueue().isMovementBlocked(),false);
    assert.equal(h.completed.filter(e=>e.npc===npc).length,1);
  }else{
    h.tick(12);assert.equal(npc.isRegistered(),false,'ordinary retained corpse removed');
    const replacements=[...derived].filter(n=>n!==npc&&n.isRegistered()&&n.getUniqueKey()===key&&n.getSpawnPosition().equals(spawn));
    assert.equal(replacements.length,1,'one clone with THIS corpse unique key and spawn identity');
    assert.equal(added.length,1,'exact one new registration from this fixture lineage');
    assert.equal(added[0],replacements[0]);assert.equal(preexisting.has(replacements[0]),false);
    assert.equal(h.completed.filter(e=>e.npc===npc).length,1,'remains never emits a second completion');
  }
}
test('generic natural player death retains remains cleanup and one ordinary respawn',t=>playerKillWithRemains(t,false));
test('generic revived remains survives original delayed cleanup without another completion',t=>playerKillWithRemains(t,true));

test('generic area.leave revival revokes old cleanup before later mutations',t=>{
  const f=h.actor(t),npc=h.PM.spawnNpc({id:7097,x:2436,y:3518,z:0,owner:f.p,ownerOnly:true,wanderRadius:0});
  const {NPCDeathTask}=require('../dist/game/task/impl/NPCDeathTask');let leaves=0;
  class RecoveryArea extends h.core.Area{
    constructor(){super([]);}
    leave(actor){leaves++;actor.setHitpoints(actor.getMaxHitpoints());}
  }
  npc.setHitpoints(0);
  const area=new RecoveryArea();npc.setArea(area);
  // Direct remove is a lifecycle boundary control, NOT assistance proof. Its
  // actual callback revives the NPC; old task must revalidate before mutating.
  const task=new NPCDeathTask(npc);task.remove();
  assert.equal(leaves,1);assert.equal(npc.isRegistered(),true);assert.equal(npc.getArea(),area);
  assert.equal(npc.getHitpoints(),npc.getMaxHitpoints());assert.equal(npc.isDyingFunction(),false);
});

test('Nieve legitimate owner kill after ordinary chase outside assistance envelope retains one slot progress',async t=>{
  const f=h.actor(t);await h.recruit(f);
  const enemy=[...h.core.World.getNpcs()].find(n=>n?.getOwner?.()===f.p&&n.getId()===7097);assert.ok(enemy);
  const clickType=enemy.getCurrentDefinition(f.p).getActions().indexOf('Attack')+1;
  await h.input(f,{type:'npc_option',index:enemy.getIndex(),clickType},4);
  // Recall follows without starting more helper attacks, letting the existing
  // enemy pursuit follow the ordinary owner/helper movement out of the envelope.
  const id=h.core.CacheDefinitions.getComponentId('wornitems:call_follower');
  await h.input(f,{type:'widget_action',widgetId:id,groupId:id>>>16,childId:id&65535,slot:-1,buttonNum:1});
  const here=f.p.getLocation(),destinations=[[2444,3516],[2427,3513],[2436,3523]].map(([x,y])=>new h.core.Location(x,y,0));
  const dest=destinations.find(at=>!h.core.RegionManager.blocked(at,null)&&h.core.RegionManager.canMovestart(here,at,1,1,null));
  assert.ok(dest,'positive loaded ordinary route outside source envelope');
  await h.input(f,{type:'move',worldX:dest.getX(),worldY:dest.getY(),modifierFlags:0,run:false});
  h.until(()=>f.p.getLocation().equals(dest),80,'owner walked outside by ordinary packet');
  const outside=()=>{const at=enemy.getLocation();return at.getX()<2431||at.getX()>2440||at.getY()<3509||at.getY()>3519;};
  h.until(outside,160,'actual enemy pursuit crosses assistance-only envelope');
  assert.ok(enemy.getHitpoints()>0,'ordinary chase positive precedes kill');
  await h.input(f,{type:'npc_option',index:enemy.getIndex(),clickType});
  h.until(()=>h.completed.some(e=>e.npc===enemy),800,'real owner combat kills outside envelope');
  const event=h.completed.find(e=>e.npc===enemy);assert.equal(event.killer,f.p);
  assert.equal(f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills'),1);
  h.PM.emitCustomEvent('npc:death-completed',event);
  assert.equal(f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills'),1);
});

for(const reverse of [false,true])test(`Nieve guarded missing-helper recall is independent of existing widget-listener order reverse=${reverse}`,async t=>{
  const f=h.actor(t,140),hooks=h.PM.customEventHooks.get('player:widget-action'),saved=hooks.slice();
  if(reverse)hooks.reverse();
  try{
    const id=h.core.CacheDefinitions.getComponentId('wornitems:call_follower');
    await h.input(f,{type:'widget_action',widgetId:id,groupId:id>>>16,childId:id&65535,slot:-1,buttonNum:1},2);
    assert.equal(h.owned(f.p,7109).length,1);assert.equal(h.owned(f.p,7109)[0].getCombat().getTarget(),null);
  }finally{hooks.splice(0,hooks.length,...saved);}
});

test('Nieve real Call cannot recover into an actual private context',async t=>{
  const f=h.actor(t,140);
  class DeniedInstance extends h.core.PrivateArea{
    constructor(){super([new h.core.Boundary(2430,2445,3505,3525,0)]);}
  }
  const area=new DeniedInstance();area.enter(f.p);assert.equal(f.p.getPrivateArea(),area);
  const id=h.core.CacheDefinitions.getComponentId('wornitems:call_follower');
  await h.input(f,{type:'widget_action',widgetId:id,groupId:id>>>16,childId:id&65535,slot:-1,buttonNum:1},2);
  assert.equal(h.owned(f.p,7109).length,0);area.leave(f.p,false);
});

test('Nieve actual pet blocks the real recruitment dialogue/action without advancing stage',async t=>{
  const f=h.actor(t);
  h.PM.emitCustomEvent('npc-drops:roll',{player:f.p,drops:[{itemId:h.core.ItemIdentifiers.OLMLET,amount:1}]});
  assert.ok(f.p.getAttribute('pets:current')?.isRegistered());
  await h.recruit(f,false);assert.equal(h.quest.getStage(f.p),120);
});

test('Nieve regular Area follower denial blocks safe Call recovery without changing quest stage',async t=>{
  const f=h.actor(t,140);
  class BarredArea extends h.core.Area{
    constructor(){super([new h.core.Boundary(2430,2445,3505,3525,0)]);}
    barsFollowers(){return true;}
  }
  const area=new BarredArea();f.p.setArea(area);area.enter(f.p);
  const id=h.core.CacheDefinitions.getComponentId('wornitems:call_follower');
  await h.input(f,{type:'widget_action',widgetId:id,groupId:id>>>16,childId:id&65535,slot:-1,buttonNum:1},2);
  assert.equal(h.owned(f.p,7109).length,0);assert.equal(h.quest.getStage(f.p),140);
  area.leave(f.p,false);f.p.setArea(null);
});

test('Nieve actual owned enemy occurrence rejects foreign killer and stale replacement; unearned slot recovers',async t=>{
  const f=h.actor(t),foreign=h.actor(t);await h.recruit(f);
  const enemy=[...h.core.World.getNpcs()].find(n=>n?.getOwner?.()===f.p&&n.getId()===7097);
  const position=enemy.getSpawnPosition().clone();
  enemy.setHitpoints(0);const occurrence=enemy.getDeathOccurrence();assert.ok(occurrence);
  h.PM.emitCustomEvent('npc:death-completed',{npc:enemy,occurrence,killer:foreign.p});
  assert.equal(f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills')??0,0);
  h.tick(40);
  const replacement=[...h.core.World.getNpcs()].find(n=>n!==enemy&&n?.getOwner?.()===f.p&&n.getId()===7097&&n.getSpawnPosition().equals(position));
  assert.ok(replacement?.isRegistered(),'uncredited slot physically recovers with a new actor');
  h.PM.emitCustomEvent('npc:death-completed',{npc:enemy,occurrence,killer:f.p});
  assert.equal(f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills')??0,0,'old life cannot donate slot credit');
  h.PM.emitCustomEvent('npc:death-completed',{npc:replacement,occurrence:null,killer:f.p});
  assert.equal(f.p.getAttribute('quest.monkey_madness_ii.stronghold-kills')??0,0,'living slot/null occurrence cannot donate credit');
});

test('Nieve capped legacy migration survives actual JSON persistence hydration without inventing historical slots',t=>{
  const f=h.actor(t,140);f.p.setAttribute('quest.monkey_madness_ii.stronghold-kills',9);
  h.PM.emitPlayerLogin({player:f.p});h.tick(2);
  const {PlayerSave}=require('../dist/game/entity/impl/player/persistence/PlayerSave');
  const codec=new(require('../plugins/persistence/JsonPlayerPersistence.plugin').JsonPlayerPersistence)();
  const raw=JSON.stringify(PlayerSave.fromPlayer(f.p),codec.replacer.bind(codec));
  h.PM.emitPlayerLogout({player:f.p,username:f.p.getUsername()});
  const g=h.actor(t);
  codec.hydratePlayerSave(JSON.parse(raw,codec.reviver.bind(codec))).applyToPlayer(g.p);
  h.PM.emitPlayerLogin({player:g.p,username:g.p.getUsername(),isNewAccount:false});h.tick(3);
  assert.equal(g.p.getAttribute('monkey-madness-ii:nieve-legacy-migrated'),true);
  assert.equal(g.p.getAttribute('monkey-madness-ii:nieve-legacy-credit'),4);
  assert.equal(g.p.getAttribute('monkey-madness-ii:nieve-stronghold-mask')??0,0);
  assert.equal(g.p.getAttribute('quest.monkey_madness_ii.stronghold-kills'),4);
  assert.equal(h.quest.getStage(g.p),145);
  assert.equal([...h.core.World.getNpcs()].filter(n=>n?.getOwner?.()===g.p&&n.getId()===7097).length,0);
});

test('generic recovery does not release a newer external movement hold',t=>{
  const f=h.actor(t),npc=h.PM.spawnNpc({id:7097,x:2439,y:3516,z:0,owner:f.p,ownerOnly:true,wanderRadius:0});
  const movement=npc.getMovementQueue();npc.setHitpoints(0);
  h.until(()=>movement.isMovementBlocked(),5,'death acquired movement hold');
  movement.setBlockMovement(true);const external=movement.getBlockMovementGeneration();
  npc.setHitpoints(npc.getMaxHitpoints());
  assert.equal(npc.isDyingFunction(),false);assert.equal(movement.isMovementBlocked(),true);
  assert.equal(movement.getBlockMovementGeneration(),external,'recovery cannot release another owner hold');
});

test('Nieve genuine held Yes after NPC removal ends the option without successful branch continuation',async t=>{
  const f=h.actor(t),offer=await h.offerRecruitment(f);
  assert.equal(h.quest.getStage(f.p),120);h.PM.removeNpc(offer.npc);
  const before=h.transcriptEvents.length;
  await h.selectOfferedYes(f,offer);h.tick(4);
  const events=h.transcriptEvents.slice(before).filter(e=>e.player===f.p&&e.npc===offer.npc);
  assert.ok(events.some(e=>e.observationKind==='choice'&&e.option==='Yes.'&&e.end===true));
  assert.deepEqual(events.filter(e=>e.observationKind==='line'||e.observationKind==='action'),[],
    'refused Yes did not execute its real transcript success branch');
  assert.equal(h.quest.getStage(f.p),120);assert.equal(h.owned(f.p,7109).length,0);
});

test('Nieve held real Yes rechecks an actual pet award and ends successful narrative at the action boundary',async t=>{
  const f=h.actor(t),offer=await h.offerRecruitment(f);
  h.PM.emitCustomEvent('npc-drops:roll',{player:f.p,drops:[{itemId:h.core.ItemIdentifiers.OLMLET,amount:1}]});
  assert.ok(f.p.getAttribute('pets:current')?.isRegistered(),'real changed eligibility after Yes was offered');
  const before=h.transcriptEvents.length;
  await h.selectOfferedYes(f,offer);h.tick(4);
  const events=h.transcriptEvents.slice(before).filter(e=>e.player===f.p&&e.npc===offer.npc);
  assert.ok(events.some(e=>e.observationKind==='choice'&&e.option==='Yes.'&&e.end===true),
    'the producer actually emitted Yes; this is not selection of No or a filtered option');
  assert.deepEqual(events.filter(e=>e.observationKind==='line'||e.observationKind==='action'),[]);
  assert.equal(h.quest.getStage(f.p),120);assert.equal(h.owned(f.p,7109).length,0);
});

test('Nieve replacement scoped playback cancels its actual old callback without revoking the newer attempt',async t=>{
  const f=h.actor(t),old=await h.offerRecruitment(f),fresh=await h.offerRecruitment(f);
  assert.notEqual(old.npc,fresh.npc);
  const before=h.transcriptEvents.length;
  // In-process stale-callback lifecycle control using the REAL old menu callback;
  // not a fabricated choice event or a client-loader permutation claim.
  old.callback(f.p,old.yes,'Yes.');
  assert.equal(h.transcriptEvents.slice(before).some(e=>e.player===f.p&&e.option==='Yes.'),false,
    'actual scoped cancellation stops old producer before it emits a choice');
  assert.equal(h.quest.getStage(f.p),120);assert.equal(h.owned(f.p,7109).length,0);
  await h.selectOfferedYes(f,fresh);
  assert.equal(h.quest.getStage(f.p),140);assert.equal(h.owned(f.p,7109).length,1,
    'old cancellation did not erase the new conversation');
});

test('Nieve native prompt replacement routes a stale widget click to current UI without recruiting',async t=>{
  const f=h.actor(t),old=await h.offerRecruitment(f);let selected=0;
  const api=h.PM.createApi('NieveReplacementPrompt');
  assert.equal(api.sendMultiChatboxPrompt(f.p,'Replacement UI','Continue.',()=>{selected++;},'Cancel.',()=>{}),true);
  await h.input(f,{type:'dialogue_continue',widgetId:(219<<16)|1,childIndex:1},2);
  assert.equal(selected,1,'current native pending prompt owns the widget callback');
  assert.equal(h.quest.getStage(f.p),120);assert.equal(h.owned(f.p,7109).length,0);
  assert.ok(old.npc.isRegistered(),'no NPC removal was required for UI replacement safety');
});

test('Nieve real No cancels its scoped playback and stale actual Yes callback cannot recruit',async t=>{
  const f=h.actor(t),offer=await h.offerRecruitment(f);
  const pending=h.core.MultiChatboxPrompt.getPending(f.p),no=pending.options.indexOf('No.');assert.ok(no>=0);
  await h.input(f,{type:'dialogue_continue',widgetId:(219<<16)|1,childIndex:no+1},2);
  const before=h.transcriptEvents.length;
  offer.callback(f.p,offer.yes,'Yes.');
  assert.equal(h.transcriptEvents.slice(before).some(e=>e.player===f.p&&e.option==='Yes.'),false);
  assert.equal(h.quest.getStage(f.p),120);assert.equal(h.owned(f.p,7109).length,0);
});
