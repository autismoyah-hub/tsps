const assert = require('node:assert/strict');
const path = require('node:path');
const {before, after, test} = require('node:test');
process.env.DISABLE_PLAYER_BOTS = '1';
require('../dist/Server').Server.installProductionPathResolver();
const {CachePipeline} = require('../dist/game/cache/CachePipeline');
const {PluginManager: PM} = require('../dist/plugins/PluginManager');
const {Player} = require('../dist/game/entity/impl/player/Player');
const {PlayerSession} = require('../dist/net/PlayerSession');
const {ClientConnection} = require('../dist/net/NetworkBuilder');
const {MultiChatboxPrompt} = require('../dist/game/model/menu/MultiChatboxPrompt');
const {decodeClientPackets} = require('../dist/net/protocol/ClientProtocol');
const {ClientPacketId} = require('../dist/net/protocol/NativeClientPackets');
let core, index=4100;
before(async()=>{
  await CachePipeline.initialize(path.resolve(__dirname,'..'));
  core=PM.getCoreApi();
  PM.loadFromDirectory(path.resolve(__dirname,'../plugins'));
  PM.emitServerStartup({});
});
after(()=>PM.emitServerShutdown({}));
function actor(t) {
  const packets=[];
  const channel={kind:'test',remoteAddress:'127.0.0.1',isOpen:()=>true,send:p=>{packets.push(p);return true;},close(){},onData(){},onError(){},onClose(){}};
  const player=new Player(new PlayerSession(channel));
  player.setIndex(index++); player.setUsername('eyesgate'+index); player.setRegistered(true);
  const connection=new ClientConnection(channel); connection.player=player;
  t.after(()=>{player.closeInterruptibleInterfaces();player.setRegistered(false);core.TaskManager.cancelTasks(player.getIndex());core.TaskManager.cancelTasks(player);});
  return {player,connection,packets};
}
function quest(key) {
  const result=require('../plugins/quests/QuestRuntime').getRegisteredQuests().find(q=>q.key===key);
  assert.ok(result,`registered quest ${key}`); return result;
}
function level(player,skill,base,current=base) {
  const skills=player.getSkillManager(); skills.setMaxLevels(skill,base,false);skills.setCurrentLevel(skill,current,false);
  assert.equal(skills.getMaxLevel(skill),base);assert.equal(skills.getCurrentLevel(skill),current);
}
function choiceSource() {
  const record=require('../data/definitions/npc-dialogues.json')['The Eyes of Glouphrie'];
  const hook='quest:the-eyes-of-glouphrie:start';
  function find(steps) {
    for(const step of steps) {
      if(step.options?.some(option=>option.hook===hook))return step;
      for(const nested of [step.steps??[],...(step.options??[]).map(option=>option.steps??[])]) {
        const found=find(nested);if(found)return found;
      }
    }
  }
  for(const [variant,steps] of Object.entries(record.variants)) {
    const choice=find(steps);if(choice)return {variant,choice,hook};
  }
  throw Error('missing generated Brimstail start choice');
}
async function start(f) {
  const {variant,choice,hook}=choiceSource();
  assert.equal(variant,'getting-started');
  const request={player:f.player,npcId:core.NpcIdentifiers.BRIMSTAIL,variant:{page:'The Eyes of Glouphrie',variant},exact:true,select:()=>[choice]};
  PM.emitCustomEvent('npc-dialogue:start',request);assert.equal(request.handled,true);
  const pending=MultiChatboxPrompt.getPending(f.player);assert.ok(pending,'real source choice is active');
  const child=choice.options.findIndex(option=>option.hook===hook)+1;
  assert.ok(child>0);assert.match(pending.options[child-1],/^yes\.?$/i);
  const frame=Buffer.alloc(7);frame[0]=ClientPacketId.RESUME_PAUSEBUTTON;frame[1]=(child>>8)&255;frame[2]=(child+128)&255;frame.writeUInt32BE(MultiChatboxPrompt.OPTIONS_WIDGET_ID>>>0,3);
  const decoded=decodeClientPackets(frame);assert.deepEqual(decoded,[{type:'dialogue_continue',childIndex:child,widgetId:MultiChatboxPrompt.OPTIONS_WIDGET_ID}]);
  await f.connection.dispatch(decoded);f.connection.processTurnPackets();
}
function ready(f,magic=45,construction=5,boostedConstruction=construction,complete=true) {
  level(f.player,core.Skill.MAGIC,magic);level(f.player,core.Skill.CONSTRUCTION,construction,boostedConstruction);
  const prerequisite=quest('grand_tree');if(complete)prerequisite.setStage(f.player,prerequisite.completionValue);
  const request={player:f.player,key:'grand_tree',complete:false};PM.emitCustomEvent('quest:is-complete',request);assert.equal(request.complete,complete);
  assert.equal(quest('the_eyes_of_glouphrie').getStage(f.player),0);
}
test('Brimstail start: Magic 45 starts when Grand Tree and Construction qualify',async t=>{
  const f=actor(t);ready(f);await start(f);assert.equal(quest('the_eyes_of_glouphrie').getStage(f.player),1);
});
test('Brimstail start: Magic 46 remains an allowed positive control',async t=>{
  const f=actor(t);ready(f,46);await start(f);assert.equal(quest('the_eyes_of_glouphrie').getStage(f.player),1);
});
test('Brimstail start: boosted Construction does not replace base level 5',async t=>{
  const f=actor(t);ready(f,46,4,5);await start(f);assert.equal(quest('the_eyes_of_glouphrie').getStage(f.player),0);
  f.player.getSession().flushPackets();assert.ok(Buffer.concat(f.packets).toString('utf8').includes('Construction level of at least 5'));
});
test('Brimstail start: unfinished Grand Tree remains a refusal',async t=>{
  const f=actor(t);ready(f,46,5,5,false);await start(f);assert.equal(quest('the_eyes_of_glouphrie').getStage(f.player),0);
});
test('Hazelmere mind-link uses base Magic 46, not a boost',t=>{
  const f=actor(t),npcId=core.NpcIdentifiers.HAZELMERE;
  const condition=stepId=>PM.emitNpcDialogueCondition({player:f.player,npcId,stepId,text:''});
  level(f.player,core.Skill.MAGIC,45,46);assert.equal(condition('IhqB-U'),true);assert.equal(condition('qErm7A'),false);
  level(f.player,core.Skill.MAGIC,46,46);assert.equal(condition('IhqB-U'),false);assert.equal(condition('qErm7A'),true);
});
