// Run after `yarn build`: node --test tests/brimhaven-rope-swing-route.test.cjs
// A rope swing click through the real packet, routing and walk-to path, on cache clipping.
const assert = require("node:assert/strict");
const { before, after, test } = require("node:test");
const path = require("node:path");
const { Server } = require("../dist/Server");
Server.installProductionPathResolver();
const { PluginManager } = require("../dist/plugins/PluginManager");
const { Player } = require("../dist/game/entity/impl/player/Player");
const { ObjectActionPacketListener } = require("../dist/net/packet/impl/ObjectActionPacketListener");
const core = PluginManager.getCoreApi();
const priorBots = process.env.DISABLE_PLAYER_BOTS;

before(async () => {
  process.env.DISABLE_PLAYER_BOTS = "1";
  await require("../dist/game/cache/CachePipeline").CachePipeline.initialize(path.resolve(__dirname, ".."));
  core.RegionManager.init();
});
after(() => {
  if (priorBots === undefined) delete process.env.DISABLE_PLAYER_BOTS;
  else process.env.DISABLE_PLAYER_BOTS = priorBots;
});

/** A player clicking a loc through the real packet listener, run until `cycles` have passed. */
function click(at, objectId, loc, option, cycles = 30) {
  for (const key of ["objectInteractionHooks", "objectRouteHooks"]) PluginManager[key] = [];
  for (const key of ["objectHooksByName", "objectHooksById", "customEventHooks"]) PluginManager[key] = new Map();
  const api = PluginManager.createApi("brimhaven-rope-swing-route");
  require("../plugins/minigames/brimhavenagilityarena/Shared.BrimhavenAgilityArena").init(api);
  require("../plugins/minigames/brimhavenagilityarena/Obstacles.BrimhavenAgilityArena")(api);

  const player = new Player(null);
  player.setUsername("offline1447");
  player.setIndex(1447);
  player.isRegistered = () => true;
  let sender;
  sender = new Proxy({}, { get: (target, key) => target[key] ?? (() => sender) });
  player.getPacketSender = () => sender;
  player.sendMessage = () => {};
  player.getSkillManager().setCurrentLevel(core.Skill.AGILITY, 99, false);
  core.RegionManager.loadMapFiles(loc[0], loc[1], false);
  player.setLocation(new core.Location(at[0], at[1], 3));
  player.setNeedsPlacement(false);

  const seen = [];
  new ObjectActionPacketListener().executeAction(player, objectId, loc[0], loc[1], 1, option);
  for (let n = 0; n < cycles; n++) {
    core.World.processCycle = (core.World.processCycle + 1) & 0x7fffffff;
    core.TaskManager.process(); player.getMovementQueue().beginCycle();
    player.getMovementQueue().process(); core.TaskManager.processWalkTo(player.getIndex());
    const tile = player.getLocation();
    seen.push(`${tile.getX()},${tile.getY()}`);
  }
  const end = player.getLocation();
  return { seen, end: [end.getX(), end.getY(), end.getZ()] };
}

test("a rope swing clicked from the platform corner walks to the rope's axis and swings", () => {
  const { seen, end } = click([2804, 9587], 3566, [2806, 9585], "Swing-on");
  assert.ok(seen.includes("2806,9587"), `walks to two tiles before the rope: ${seen.join(" ")}`);
  assert.deepEqual(end, [2806, 9582, 3], "lands three tiles past the rope");
});

test("monkey bars are walked to from the platform beside their blocked first bar, then crossed (rsprox 1735)", () => {
  // Each set's end pieces (3564), a start on each side's platform, and where the swing lands.
  const BARS = [
    { ends: [[2771, 9570], [2771, 9577]], sides: [[[2772, 9569], [2772, 9578]], [[2773, 9579], [2772, 9569]]] },
    { ends: [[2774, 9545], [2781, 9545]], sides: [[[2772, 9547], [2782, 9546]], [[2783, 9547], [2773, 9546]]] },
    { ends: [[2793, 9559], [2793, 9566]], sides: [[[2795, 9557], [2794, 9567]], [[2795, 9568], [2794, 9558]]] },
  ];
  for (const { ends, sides } of BARS) {
    for (const loc of ends) {
      for (const [from, landing] of sides) {
        const { seen, end } = click(from, 3564, loc, "Swing-across", 40);
        assert.deepEqual(end, [...landing, 3], `bars at ${loc} from ${from}: ${[...new Set(seen)].join(" ")}`);
      }
    }
  }
});

test('native Brimhaven hazards use real clipping, movement and session updates', async t => {
  const Arena = require('../plugins/minigames/BrimhavenAgilityArena.plugin');
  const Shared = require('../plugins/minigames/brimhavenagilityarena/Shared.BrimhavenAgilityArena');
  const { PlayerSession } = require('../dist/net/PlayerSession');
  const { AreaManager } = require('../dist/game/model/areas/AreaManager');
  const { MapObjects, ObjectManager } = core;
  const resetAnimation = core.Animation.DEFAULT_RESET_ANIMATION.getId();
  Shared.sessions.clear(); Shared.cycle.tick = 0;
  t.after(() => { PluginManager.emitServerShutdown({}); Shared.sessions.clear(); });
  Arena.register(PluginManager.createApi('native-brimhaven-blades'));
  // Production also finishes pending obstacles from Agility's later logout hook.
  require('../plugins/skills/Agility.plugin').register(PluginManager.createApi('native-agility'));
  PluginManager.emitServerStartup({});
  const turn = () => {
    core.World.process();
    core.World.forEachNetworkPlayer(p => p.getSession().flushPackets());
  };
  const phase = n => { while (Shared.cycle.tick % 7 !== n) turn(); };
  const at = (x,y,z=3) => new core.Location(x,y,z);
  const position = p => [p.getLocation().getX(),p.getLocation().getY(),p.getLocation().getZ()];
  const blade = (x,y,id) => MapObjects.get(id,at(x,y),null);
  let index = 1800;
  function actor(test,x,y,z=3,hp=38,agility=99) {
    const frames=[], events=[];
    const channel={kind:'test',isOpen:()=>true,send:b=>{frames.push(Buffer.from(b));return true;},close(){},onData(){},onClose(){},onError(){}};
    const session=new PlayerSession(channel), p=new Player(session); session.setPlayer(p);
    p.setUsername('native-blade-'+index++); p.setLocation(at(x,y,z)); p.setNeedsPlacement(false); p.setRunning(false);
    p.getSkillManager().setMaxLevels(core.Skill.HITPOINTS,hp,false).setCurrentLevel(core.Skill.HITPOINTS,hp,false);
    p.getSkillManager().setMaxLevels(core.Skill.AGILITY,agility,false).setCurrentLevel(core.Skill.AGILITY,agility,false);
    // Preserve real World registration and area processing; skip only account-file I/O.
    test.mock.method(p,'onAdd',()=>{}); test.mock.method(p,'onRemove',()=>{});
    const perform=p.performAnimation.bind(p);
    test.mock.method(p,'performAnimation',animation=>{events.push([Shared.cycle.tick,'performAnimation',animation]);return perform(animation);});
    const sender=p.getPacketSender();
    for(const name of ['sendObject','sendObjectRemoval','sendObjectAnimation','sendMessage','sendTypedMessage','sendSoundEffect']) {
      const original=sender[name].bind(sender);
      test.mock.method(sender,name,(...args)=>{events.push([Shared.cycle.tick,name,...args]);return original(...args);});
    }
    // Cleanup is installed before World admission, including a failed setup.
    test.after(()=>{
      if (p.isRegistered()) PluginManager.emitPlayerLogout({player:p,username:p.getUsername()});
      core.TaskManager.cancelTasks(p); core.TaskManager.cancelTasks(p.getIndex());
      if (p.isRegistered()) core.World.getPlayers().remove(p);
      core.World.refreshActiveRegions();
    });
    assert.equal(core.World.getPlayers().add(p),true);
    assert.equal(core.World.getPlayers().get(p.getIndex()),p);
    core.World.refreshActiveRegions(); AreaManager.process(p);
    return {p,session,frames,events,xp:p.getSkillManager().getExperience(core.Skill.AGILITY)};
  }

  await t.test('walking onto a saw tile produces three static/four animated ticks, next-tick damage and zero XP',test=>{
    const f=actor(test,2787,9579);
    f.p.getMovementQueue().requestWalk(at(2788,9579)); turn(); turn();
    assert.deepEqual(position(f.p),[2788,9579,3]); assert.ok(blade(2788,9579,3568));
    assert.equal(core.RegionManager.getClipping(2788,9579,3)&0x100,0);
    turn();
    assert.ok(blade(2788,9579,3567),'animated native scenery replaces both ground decorations');
    assert.equal(blade(2788,9579,3568),null); assert.equal(blade(2789,9579,3569),null);
    for(const x of [2788,2789]) assert.ok(core.RegionManager.getClipping(x,9579,3)&0x100);
    assert.equal(f.p.getHitpoints(),38,'hit lands next tick');
    assert.ok(f.events.some(([tick,name,type,text])=>tick===3&&name==='sendTypedMessage'&&text==='You were hit by the saw blade!'));
    assert.ok(f.p.getForceMovement()); assert.ok(f.events.some(([tick,name,animation])=>tick===3&&name==='performAnimation'&&animation.getId()===846));
    turn();
    assert.deepEqual(position(f.p),[2787,9579,3]); assert.equal(f.p.getHitpoints(),35);
    assert.equal(f.events.filter(([tick,name,animation])=>tick===4&&name==='performAnimation'&&animation.getId()===resetAnimation).length,0,
      'the delayed846 pose survives successful movement release without a second animation or an extra stall');
    assert.equal(f.p.getSkillManager().getExperience(core.Skill.AGILITY),f.xp);
    turn();turn(); assert.ok(blade(2788,9579,3567),'four animated ticks'); turn();
    assert.ok(blade(2788,9579,3568)); assert.ok(blade(2789,9579,3569)); assert.equal(blade(2788,9579,3567),null);
    assert.equal(core.RegionManager.getClipping(2788,9579,3)&0x100,0); assert.ok(f.frames.length);
  });

  await t.test('all six halves push outward on both axes, with no Agility requirement',async test=>{
    const cases=[[2761,9584,2761,9583],[2761,9585,2761,9586],[2783,9551,2783,9550],[2783,9552,2783,9553],[2788,9579,2787,9579],[2789,9579,2790,9579]];
    for(const [x,y,dx,dy] of cases) await test.test(`${x},${y}`,sub=>{
      phase(0); const f=actor(sub,dx,dy,3,99); f.p.getSkillManager().setCurrentLevel(core.Skill.AGILITY,1,false);
      f.p.getMovementQueue().requestWalk(at(x,y)); turn();turn();turn();turn();
      assert.deepEqual(position(f.p),[dx,dy,3]); assert.equal(f.p.getHitpoints(),93);
      assert.equal(f.p.getSkillManager().getExperience(core.Skill.AGILITY),f.xp);
    });
  });

  await t.test('early occupancy is not latched; a different plane, teleport or logout cannot be hit remotely',test=>{
    phase(0); const f=actor(test,2787,9579), other=actor(test,2788,9579,0);
    const leaving=actor(test,2789,9579), loggingOut=actor(test,2761,9584);
    f.p.getMovementQueue().requestWalk(at(2788,9579)); turn();
    f.p.getMovementQueue().requestWalk(at(2787,9579)); leaving.p.moveTo(at(2808,3193,0));
    PluginManager.emitPlayerLogout({player:loggingOut.p,username:loggingOut.p.getUsername()}); turn();turn();turn();
    assert.deepEqual(position(f.p),[2787,9579,3]);
    for(const a of [f,other,leaving,loggingOut]) assert.equal(a.p.getHitpoints(),38);
  });

  await t.test('cancelling a pending saw knockback clears its pose/lock and cannot damage after leaving',test=>{
    phase(0); const f=actor(test,2788,9579); turn();turn();turn();
    assert.ok(f.p.getForceMovement());
    core.TaskManager.cancelTasks(f.p); f.p.moveTo(at(2808,3193,0));
    assert.equal(f.p.getForceMovement(),null);
    assert.ok(f.events.some(([tick,name,animation])=>tick===Shared.cycle.tick&&name==='performAnimation'&&animation.getId()===resetAnimation));
    turn(); assert.deepEqual(position(f.p),[2808,3193,0]); assert.equal(f.p.getHitpoints(),38);
    assert.equal(f.p.getAttribute('agility.obstacle'),null);
  });

  await t.test('simultaneous occupants have separate hits; cycle audio reaches five tiles but not six',test=>{
    phase(0);
    const first=actor(test,2788,9579), second=actor(test,2789,9579,3,99);
    const nearby=actor(test,2788,9584), distant=actor(test,2788,9585);
    turn();turn();turn();
    const onset=Shared.cycle.tick;
    for(const f of [first,second,nearby]) assert.equal(f.events.filter(([tick,name,id])=>tick===onset&&name==='sendSoundEffect'&&id===1386).length,1);
    assert.equal(distant.events.filter(([,name,id])=>name==='sendSoundEffect'&&id===1386).length,0);
    turn();
    assert.deepEqual(position(first.p),[2787,9579,3]); assert.deepEqual(position(second.p),[2790,9579,3]);
    assert.equal(first.p.getHitpoints(),35); assert.equal(second.p.getHitpoints(),93);
    assert.equal(nearby.p.getHitpoints(),38); assert.equal(distant.p.getHitpoints(),38);
  });

  await t.test('a removed player cannot finish a pending knockback or receive its queued damage',test=>{
    phase(0); const f=actor(test,2788,9579); turn();turn();turn();
    PluginManager.emitPlayerLogout({player:f.p,username:f.p.getUsername()});
    core.World.getPlayers().remove(f.p); core.World.refreshActiveRegions(); turn();
    assert.equal(f.p.getHitpoints(),38); assert.deepEqual(position(f.p),[Shared.HUT.x,Shared.HUT.y,Shared.HUT.z]);
    assert.equal(f.p.getAttribute('agility.obstacle'),null); assert.equal(f.p.getForceMovement(),null);
  });

  await t.test('a late entrant sees the active scenery and its clipping blocks ordinary walking until restoration',test=>{
    phase(3); const f=actor(test,2787,9579); ObjectManager.onRegionChange(f.p,2752,9536);
    assert.ok(f.events.some(([,name,obj])=>name==='sendObject'&&obj.getId()===3567));
    assert.ok(f.events.some(([,name,obj])=>name==='sendObjectRemoval'&&obj.getId()===3568&&obj.getType()===22));
    f.p.getMovementQueue().requestWalk(at(2788,9579)); turn();turn();turn();
    assert.deepEqual(position(f.p),[2787,9579,3]); turn();
    f.p.getMovementQueue().requestWalk(at(2788,9579)); turn();
    assert.deepEqual(position(f.p),[2788,9579,3]); assert.equal(f.p.getHitpoints(),38);
  });

  await t.test('spinning blades animate once for the jumper and an observer, one tick after the jump starts',test=>{
    phase(0); const f=actor(test,2783,9572), observer=actor(test,2785,9573); Shared.setRandom(()=>0);
    f.p.getMovementQueue().requestWalk(at(2783,9573)); turn();turn(); const started=Shared.cycle.tick;
    assert.ok(f.events.some(([tick,name,animation])=>tick===started&&name==='performAnimation'&&animation.getId()===1115)); assert.equal(f.events.filter(([,name])=>name==='sendObjectAnimation').length,0);
    turn();
    for(const a of [f,observer]) {
      const animations=a.events.filter(([,name])=>name==='sendObjectAnimation'); assert.equal(animations.length,1);
      const [tick,,object,animation]=animations[0]; assert.equal(tick,started+1); assert.equal(object.getId(),3580); assert.equal(animation.getId(),1107);
      assert.deepEqual([object.getLocation().getX(),object.getLocation().getY(),object.getFace()],[2782,9573,2]);
    }
    for(let i=0;i<8;i++) turn();
    assert.deepEqual(position(f.p),[2783,9576,3]); assert.equal(f.events.filter(([,name])=>name==='sendObjectAnimation').length,1);
    assert.ok(f.p.getSkillManager().getExperience(core.Skill.AGILITY)>f.xp);
    assert.equal(f.events.filter(([,name])=>name==='performAnimation').at(-1)[2].getId(),resetAnimation,
      'the existing default runner completion still resets other obstacle animations');
  });

  // Floor-spike minimum level is the observed contract. The exact retail
  // refusal movement is not captured; these assertions only require no crossing.
  await t.test('under-level ordinary routes cannot cross floor spikes; level 20 can', async test => {
    const definition = core.CacheDefinitions.getObject(3582);
    assert.equal(definition.name, 'Floor spikes', 'active cache identifies the obstacle');
    assert.equal(core.GameConstants.EXPERIENCE_MULTIPLIER, 1);
    const oldRandom = Math.random;
    let rolls = 0;
    Shared.setRandom(() => {
      // This draw alone belongs to the actual spike outcome, not other world logic.
      if ((new Error().stack ?? '').includes('rollTrap')) { rolls++; return 0; }
      return oldRandom();
    });
    test.after(() => Shared.setRandom(oldRandom));
    const spans = [
      { label:'south spikes', from:[2772,9550], to:[2772,9553], axis:1, low:9550, high:9553 },
      { label:'east spikes', from:[2798,9568], to:[2801,9568], axis:0, low:2798, high:2801 },
      { label:'west spikes', from:[2761,9572], to:[2761,9575], axis:1, low:9572, high:9575 },
    ];
    for (const span of spans) for (const reverse of [false,true]) for (const running of [false,true]) {
      await test.test(`${span.label}, ${reverse ? 'reverse' : 'forward'}, ${running ? 'run' : 'walk'}`, sub => {
        const from = reverse ? span.to : span.from, to = reverse ? span.from : span.to;
        core.RegionManager.loadMapFiles(from[0],from[1],false);
        const dx = Math.sign(to[0]-from[0]), dy = Math.sign(to[1]-from[1]);
        const spikeObjects = [1,2].map(step => core.MapObjects.get(3582,at(from[0]+dx*step,from[1]+dy*step),null)).filter(Boolean);
        assert.ok(spikeObjects.length > 0, 'the active map contains floor spikes in this span');
        assert.ok(spikeObjects.some(object => object.getDefinition().getName() === 'Floor spikes'));
        function level(f, value) {
          const sm = f.p.getSkillManager();
          sm.setExperience(core.Skill.AGILITY, core.SkillManager.getExperienceForLevel(value));
          sm.setMaxLevels(core.Skill.AGILITY, value, false).setCurrentLevel(core.Skill.AGILITY, value, false);
          assert.equal(sm.getCurrentLevel(core.Skill.AGILITY), value);
          f.p.setRunning(running);
          return sm.getExperience(core.Skill.AGILITY);
        }
        const allowed = actor(sub,from[0],from[1]), allowedXp = level(allowed,20);
        const beforeRolls = rolls;
        allowed.p.getMovementQueue().requestWalk(at(to[0],to[1]));
        let actuallyRan = false;
        for (let i=0;i<12;i++) { turn(); actuallyRan ||= allowed.p.getMovementQueue().didRunThisCycle(); }
        if (running) assert.equal(actuallyRan,true,'the control actually consumed a running movement frame');
        assert.deepEqual(position(allowed.p),[...to,3], 'a real level-20 route crosses this span');
        assert.equal(rolls,beforeRolls+1,'the spike outcome consumes the controlled draw');
        assert.equal(allowed.p.getSkillManager().getExperience(core.Skill.AGILITY)-allowedXp,24);

        const denied = actor(sub,from[0],from[1]), deniedXp = level(denied,19);
        let crossed = false;
        const attempted = () => {
          denied.p.getMovementQueue().requestWalk(at(to[0],to[1]));
          for(let i=0;i<8;i++) {
            turn();
            const value = position(denied.p)[span.axis];
            crossed ||= reverse ? value <= span.low : value >= span.high;
          }
        };
        attempted();
        assert.equal(denied.p.getSkillManager().getCurrentLevel(core.Skill.AGILITY),19);
        assert.ok(denied.events.some(([,name,message])=>name==='sendMessage' && message.includes('at least 20')),
          'the native area observed the attempted under-level crossing');
        assert.equal(crossed,false,'the queued route cannot reach the opposite side');
        assert.equal(denied.p.getSkillManager().getExperience(core.Skill.AGILITY),deniedXp);
        assert.equal(denied.p.getMovementQueue().getDestination(),null,'refusal clears the pending route');
        attempted();
        assert.equal(crossed,false,'a fresh click after normal placement also cannot cross');
        assert.equal(denied.p.getSkillManager().getExperience(core.Skill.AGILITY),deniedXp);
      });
    }
  });

  await t.test('run enabled with a one-step spike approach uses the actual walking direction', test => {
    core.RegionManager.loadMapFiles(2772,9550,false);
    const f=actor(test,2772,9550,3,38,19);
    f.p.setRunning(true);
    f.p.getMovementQueue().requestWalk(at(2772,9551));
    turn();
    assert.equal(f.p.getMovementQueue().didRunThisCycle(),true,'run energy is consumed');
    assert.deepEqual(position(f.p),[2772,9550,3],'the TSPS policy restores the ordinary approach origin');
    assert.ok(f.events.some(([,name,message])=>name==='sendMessage' && message.includes('at least 20')));
  });

  await t.test('a same-cycle walk-to relocation is not treated as an ordinary spike approach', test => {
    core.RegionManager.loadMapFiles(2772,9548,false);
    const f=actor(test,2772,9550,3,38,19), target=actor(test,2772,9548);
    const serial=f.p.getTeleportSerial();
    let callbacks=0;
    f.p.getMovementQueue().walkToEntity(target.p, () => {
      assert.equal(f.p.getMovementQueue().steppedThisWorldCycle(),true);
      f.p.setLocation(at(2772,9551)); // model a distinct registered interaction's relocation
      callbacks++;
    });
    for(let i=0;i<6 && callbacks===0;i++) turn();
    assert.equal(callbacks,1,'real walk-to reach callback ran after an ordinary step');
    assert.equal(f.p.getTeleportSerial(),serial,'the callback used setLocation, not teleport');
    assert.deepEqual(position(f.p),[2772,9551,3], 'the unrelated relocation is not rolled back');
  });

  await t.test('a player initially placed on spikes can leave without an inferred approach', async test => {
    core.RegionManager.loadMapFiles(2772,9550,false);
    for (const [y, destination] of [[9551,9550],[9552,9553]]) await test.test(`from ${y}`, sub => {
      const f=actor(sub,2772,y,3,38,19);
      const xp=f.p.getSkillManager().getExperience(core.Skill.AGILITY);
      assert.equal(f.p.getSkillManager().getCurrentLevel(core.Skill.AGILITY),19);
      f.p.getMovementQueue().requestWalk(at(2772,destination));
      for(let i=0;i<5;i++) turn();
      assert.deepEqual(position(f.p),[2772,destination,3]);
      assert.equal(f.p.getSkillManager().getExperience(core.Skill.AGILITY),xp);
    });
  });

  await t.test('a teleport onto a spike is not mistaken for an approach and the player can leave', test => {
    core.RegionManager.loadMapFiles(2772,9550,false);
    const f=actor(test,2772,9550);
    f.p.getSkillManager().setExperience(core.Skill.AGILITY,core.SkillManager.getExperienceForLevel(19));
    f.p.getSkillManager().setMaxLevels(core.Skill.AGILITY,19,false).setCurrentLevel(core.Skill.AGILITY,19,false);
    const xp=f.p.getSkillManager().getExperience(core.Skill.AGILITY), prior=f.p.getTeleportSerial();
    f.p.moveTo(at(2772,9552));
    assert.ok(f.p.getTeleportSerial()>prior);
    turn(); // let the real World consume placement before a new walk request
    assert.deepEqual(position(f.p),[2772,9552,3], 'stale pre-teleport position is not a rollback target');
    f.p.getMovementQueue().requestWalk(at(2772,9553));
    for(let i=0;i<5;i++) turn();
    assert.deepEqual(position(f.p),[2772,9553,3], 'an already placed player can leave');
    assert.equal(f.p.getSkillManager().getExperience(core.Skill.AGILITY),xp);
  });

  await t.test('a same-tile teleport refreshes approach provenance for a later normal walk', test => {
    core.RegionManager.loadMapFiles(2772,9550,false);
    const f=actor(test,2772,9550);
    f.p.getSkillManager().setExperience(core.Skill.AGILITY,core.SkillManager.getExperienceForLevel(19));
    f.p.getSkillManager().setMaxLevels(core.Skill.AGILITY,19,false).setCurrentLevel(core.Skill.AGILITY,19,false);
    const serial=f.p.getTeleportSerial();
    f.p.moveTo(at(2772,9550)); turn();
    assert.ok(f.p.getTeleportSerial()>serial);
    assert.deepEqual(position(f.p),[2772,9550,3]);
    f.p.getMovementQueue().requestWalk(at(2772,9553));
    for(let i=0;i<8;i++) turn();
    assert.ok(position(f.p)[1]<9553, 'the later genuine approach still cannot cross');
  });

  await t.test('shutdown restores all cache decorations/clipping and stops the shared clock',()=>{
    phase(3); PluginManager.emitServerShutdown({});
    for(const [x,y,dx,dy] of [[2761,9584,0,1],[2783,9551,0,1],[2788,9579,1,0]]) {
      assert.ok(blade(x,y,3568)); assert.ok(blade(x+dx,y+dy,3569)); assert.equal(blade(x,y,3567),null);
      assert.equal(core.RegionManager.getClipping(x,y,3)&0x100,0);
    }
    assert.equal(core.World.getRemovedObjects().filter(object=>[3568,3569].includes(object.getId())).length,0);
    assert.equal(core.World.getObjects().filter(object=>object.getId()===3567).length,0);
    const stopped=Shared.cycle.tick; turn(); assert.equal(Shared.cycle.tick,stopped);
  });
});
