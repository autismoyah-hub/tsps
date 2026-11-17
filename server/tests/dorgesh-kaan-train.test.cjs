// After yarn build: node --test tests/dorgesh-kaan-train.test.cjs
//
// The Dorgesh-Kaan - Keldagrim train against the Wiki (Another Slice of H.A.M. needed, 50 ticks
// a trip) and rsprox_transcripts/232/03199.txt (the lines, the ride's ticks, the arrival tiles).
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, test } = require('node:test');

require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');

let core;
let Train;
let ModeCommon;
let Regions;
const faults = new WeakMap();
const loginObservations = new WeakMap();
const completed = new Set();
const diaryTasks = [];

before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  core = PluginManager.getCoreApi();
  const api = PluginManager.createApi('DorgeshKaanTrainTest');
  const emit = api.emitCustomEvent.bind(api);
  api.emitCustomEvent = (name, query) => {
    const fault = faults.get(query?.player);
    const key = name === 'league:resolve-travel' ? 'resolve' : name === 'league:profile' ? 'profile' : null;
    if (key && fault?.[key] === 'missing') return;
    if (key && fault?.[key] === 'unanswered') { emit(name, { ...query }); return; }
    return emit(name, query);
  };
  api.persistAttribute('league:areas');
  ModeCommon = require('../plugins/modes/league/Common.LeagueMode');
  ModeCommon.init(api);
  api.onPlayerLogin(event => {
    const observed = loginObservations.get(event.player) ?? [];
    observed.push({ event, player: event.player, username: event.username, profile: event.profile,
      registered: event.player.isRegistered(), position: xyz(event.player),
      marker: event.player.getAttribute(Train._test.RECOVERY_ATTRIBUTE) });
    loginObservations.set(event.player, observed);
  }); // Real registered hook, before Account/Train: records save-applied state at dispatch entry.
  require('../plugins/modes/league/Account.LeagueMode')(api);
  require('../plugins/modes/leagueareas/Common.LeagueAreaUnlocks').init(api);
  Regions = require('../plugins/modes/leagueareas/Regions.LeagueAreaUnlocks');
  require('../plugins/modes/leagueareas/Enforcement.LeagueAreaUnlocks')(api);
  assert.ok(core.CacheMaps.getRegionIds().includes((38 << 8) | 86), 'actual cache must contain named station square');

  api.onCustomEvent('diary:task', (event) => diaryTasks.push([event.player, event.diary, event.task]));
  api.onCustomEvent('quest:is-complete', (request) => {
    if (request.key === 'another_slice_of_ham') request.complete = completed.has(request.player);
  });
  Train = require('../plugins/world/DorgeshKaanTrain.plugin');
  Train.register(api);
});

let index = 1500;
function rider({ done = true, at = [2483, 5538] } = {}) {
  const player = new Player(null, new core.Location(at[0], at[1], 0));
  player.setIndex(index++);
  player.setUsername('train-test');
  player.setRegistered(true);
  player.setNeedsPlacement(false);
  player.getSession = () => ({ sendClientPacket: () => true, write() {} });
  player.messages = [];
  player.sendMessage = (text) => player.messages.push(text);
  player.sounds = [];
  const sender = player.getPacketSender();
  const sendSound = sender.sendSound.bind(sender);
  sender.sendSound = (id, ...rest) => { player.sounds.push(id); return sendSound(id, ...rest); };
  if (done) completed.add(player);
  PluginManager.updatePlayerZones(player);
  return player;
}

function step(player, x, y) {
  player.moveTo(new core.Location(x, y, 0));
  PluginManager.updatePlayerZones(player);
}

/** One game tick: the tasks, then each rider's zones (as the player process does). */
function tick(players, n = 1) {
  for (let i = 0; i < n; i++) {
    core.TaskManager.process();
    for (const player of players) PluginManager.updatePlayerZones(player);
  }
}

const xyz = (player) => [player.getLocation().getX(), player.getLocation().getY(), player.getLocation().getZ()];

test('stepping into the Dorgesh-Kaan train: the line, then 36 ticks on the ride to Keldagrim on the recorded ticks, and 50 ticks a trip aboard', () => {
  const player = rider();
  let now = 0;
  const departures = [];
  const run = (n = 1) => {
    for (let i = 0; i < n; i++) {
      now++;
      const before = player.messages.length;
      tick([player]);
      for (const text of player.messages.slice(before)) if (text.startsWith('The train takes you')) departures.push(now);
    }
  };
  step(player, 2480, 5538);
  assert.deepEqual(player.messages, ['The train will leave for Keldagrim shortly.']);
  run(36);
  assert.deepEqual(departures, [36], '36 ticks after the line');
  assert.equal(player.messages.at(-1), 'The train takes you to Keldagrim.');
  assert.equal(player.getMovementQueue().isMovementBlocked(), true);
  run(3);
  assert.deepEqual(xyz(player), [2542, 5506, 0], 'the cutscene room 3 ticks on');
  run();
  assert.deepEqual(player.sounds.slice(-1), [3584]);
  run(9);
  assert.deepEqual(xyz(player), [2443, 5533, 0], 'the teleport at +13: the other train, tile turned half round (2480,5538 -> 2443,5533)');
  assert.equal(player.getMovementQueue().isMovementBlocked(), true, 'still held until +14 (rsprox 2868 / 2869)');
  assert.equal(player.messages.at(-1), 'The train takes you to Keldagrim.', 'no line yet at the teleport');
  run();
  assert.equal(player.getMovementQueue().isMovementBlocked(), false);
  assert.equal(player.messages.at(-1), 'The train will leave for Dorgesh-Kaan shortly.', '+14: still aboard, the next one');
  assert.deepEqual(diaryTasks.filter(([who]) => who === player).map(([, diary, task]) => [diary, task]), [['lumbridge', 'take-the-train-from-dorgesh-kaan-to-keldagrim']]);

  run(36);
  assert.deepEqual(departures, [36, 86], 'the next departure 50 ticks after the first (rsprox 2855 -> 2905)');
  run(14);
  assert.deepEqual(xyz(player), [2480, 5538, 0]);
  assert.equal(diaryTasks.filter(([who]) => who === player).length, 1, 'Keldagrim to Dorgesh-Kaan is not the task');
  core.TaskManager.cancelTasks(player);
});

test('stepping out before it leaves: no ride; and without Another Slice of H.A.M. nothing happens', () => {
  const player = rider({ at: [2441, 5533] });
  step(player, 2443, 5530);
  assert.deepEqual(player.messages, ['The train will leave for Dorgesh-Kaan shortly.']);
  tick([player], 10);
  step(player, 2441, 5533);
  tick([player], 40);
  assert.equal(player.messages.length, 1, 'no departure');
  assert.deepEqual(xyz(player), [2441, 5533, 0]);

  const stranger = rider({ done: false });
  step(stranger, 2480, 5541);
  tick([stranger], 40);
  assert.deepEqual(stranger.messages, []);
  assert.deepEqual(xyz(stranger), [2480, 5541, 0]);
});

test('the arrival tile mirrors the departure one, as all five recorded trips', () => {
  const { arrivalTile } = Train._test;
  const to = (x, y) => { const at = arrivalTile(new core.Location(x, y, 0)); return [at.getX(), at.getY()]; };
  assert.deepEqual(to(2479, 5538), [2444, 5533]);
  assert.deepEqual(to(2444, 5533), [2479, 5538]);
  assert.deepEqual(to(2480, 5538), [2443, 5533]);
  assert.deepEqual(to(2443, 5533), [2480, 5538]);
  assert.deepEqual(to(2443, 5530), [2480, 5541]);
});

test('a logout mid-ride is saved on the other train', () => {
  const player = rider();
  step(player, 2479, 5538);
  tick([player], 36 + 5);
  assert.deepEqual(xyz(player), [2542, 5506, 0]);
  Train._test.logout({ player });
  assert.deepEqual(xyz(player), [2444, 5533, 0]);
  assert.equal(Train._test.riding.has(player), false);
});

// R1 contract/lifecycle regressions. Native real-core/cache tests; NOT executed during source preparation.
const { PlayerSession } = require('../dist/net/PlayerSession');
const { PlayerSave } = require('../dist/game/entity/impl/player/persistence/PlayerSave');
let faultActorId = 0;
function worldRider(t, { profile = 'league', league = 4, unlocked = [1, 2, 8], at = [2483, 5538], save = null, fault = null } = {}) {
  const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => false, send: () => true,
    close() {}, onData() {}, onError() {}, onClose() {} };
  const session = new PlayerSession(channel);
  const p = new Player(session, new core.Location(...at, 0));
  t.after(() => {
    faults.delete(p);
    if (p.isRegistered()) core.World.getPlayers().remove(p);
    const queue = core.World.getRemovePlayerQueue();
    for (let i = queue.indexOf(p); i !== -1; i = queue.indexOf(p)) queue.splice(i, 1);
    completed.delete(p);
  });
  session.setPlayer(p);
  p.setUsername(save ? save.getDisplayName() : 'trainfault' + (++faultActorId));
  p.setLongUsername(core.Misc.stringToLongBigInt(p.getUsername()));
  p.setProfile(profile);
  const username = p.getUsername();
  assert.equal(Boolean(p.isRegistered()), false, 'new actor is semantically unregistered before profile/save application');
  assert.equal(Array.from(core.World.getPlayers()).some(actor => actor === p), false,
    'exact actor is absent from actual World collection before save application');
  if (save) {
    save.applyToPlayer(p);
    assert.equal(p.getLocation().equals(save.getPosition()), true, 'actual save position applied before World.add');
    assert.deepEqual(p.getAttribute(Train._test.RECOVERY_ATTRIBUTE), save.attributes[Train._test.RECOVERY_ATTRIBUTE] ?? null,
      'actual persisted recovery projection applied before World.add');
  } else { p.setAttribute('league:type', league); p.setAttribute('league:areas', unlocked); }
  assert.equal(p.getUsername(), username, 'save projection does not replace selected login identity');
  assert.equal(p.getProfile(), profile, 'runtime profile established before save apply remains authoritative');
  if (fault) faults.set(p, fault);
  completed.add(p);
  assert.equal(Boolean(p.isRegistered()), false, 'save application has not registered the actor before add');
  assert.equal(Array.from(core.World.getPlayers()).some(actor => actor === p), false,
    'exact actor is absent from actual World collection immediately before add');
  assert.equal(core.World.getPlayers().add(p), true);
  assert.equal(p.isRegistered(), true, 'actual successful World.add explicitly registers the actor');
  assert.equal(Array.from(core.World.getPlayers()).filter(actor => actor === p).length, 1,
    'exact actor appears once in actual World collection before dispatch');
  // Same complete event shape NetworkBuilder uses, after profile/save apply and World.add.
  dispatchLogin(p); // Real hooks must actually run before fault/recovery assertions; no bootstrap proof.

  p.setNeedsPlacement(false);
  PluginManager.updatePlayerZones(p);
  return p;
}
function dispatchLogin(player) {
  const event = { player, username: player.getUsername(), profile: player.getProfile(), isNewAccount: false };
  assert.ok(event.username, 'real onPlayerLogin wrapper requires username');
  assert.equal(player.isRegistered(), true, 'World.add precedes dispatcher');
  const before = loginObservations.get(player)?.length ?? 0;
  const position = xyz(player), marker = player.getAttribute(Train._test.RECOVERY_ATTRIBUTE);
  PluginManager.emitPlayerLogin(event);
  const observed = loginObservations.get(player);
  assert.equal(observed?.length, before + 1, 'real registered observer was dispatched exactly once');
  const current = observed.at(-1);
  assert.equal(current.event, event); assert.equal(current.player, player);
  assert.equal(current.username, player.getUsername()); assert.equal(current.profile, event.profile);
  assert.equal(current.registered, true); assert.deepEqual(current.position, position);
  assert.deepEqual(current.marker, marker, 'observer sees pre-recovery save-applied marker on the same actor');
}
function captureLogoutSaves(t) {
  const saves = [];
  t.mock.method(core.GameConstants.PLAYER_PERSISTENCE, 'cancelRequestedSave', () => {});
  t.mock.method(core.GameConstants.PLAYER_PERSISTENCE, 'save', (p, reason) => {
    saves.push({ p, reason, registered: p.isRegistered(), xyz: xyz(p), marker: p.getAttribute(Train._test.RECOVERY_ATTRIBUTE), snapshot: PlayerSave.fromPlayer(p) });
  });
  return saves; // Explicit fake persistence capture; not SQLite/committed account evidence.
}
function removeThroughWorld(p) {
  core.World.getPlayers().remove(p); // Actual MobileList.remove -> registeredfalse -> Player.onLogout -> save.
  const queue = core.World.getRemovePlayerQueue();
  const i = queue.indexOf(p); if (i !== -1) queue.splice(i, 1);
}
function enterRide(p, ticksAfterDeparture = 5) {
  assert.equal(p.isRegistered(), true); assert.ok(p.getHitpoints() > 0);
  const type = p.getAttribute('league:type'), origin = new core.Location(2479, 5538, 0);
  assert.equal(Regions.isOpenAt(type, p, origin), true, 'positive authorized departure prerequisite');
  assert.equal(Regions.isOpenAt(type, p, Train._test.arrivalTile(origin)), true, 'positive authorized destination prerequisite');
  step(p, 2479, 5538); tick([p], 36 + ticksAfterDeparture);
  assert.equal(Train._test.riding.has(p), true, 'actual ordinary departure allocated the intended ride before fault injection');
}
function injectFault(t, p, kind) {
  if (kind === 'resolve-missing' || kind === 'resolve-unanswered') faults.set(p, { resolve: kind.slice(8) });
  else if (kind === 'profile-missing' || kind === 'profile-unanswered') faults.set(p, { profile: kind.slice(8) });
  else if (kind === 'resolver-throws-before-rewrite') {
    const original = Regions.isOpenAt;
    t.mock.method(Regions, 'isOpenAt', (type, actor, location) => { if (actor === p) throw new Error('R1 injected resolver failure'); return original(type, actor, location); });
  } else if (kind === 'profile-throws') {
    const original = ModeCommon.leagueOf;
    t.mock.method(ModeCommon, 'leagueOf', actor => { if (actor === p) throw new Error('R1 injected profile failure'); return original(actor); });
  } else if (kind === 'unknown-type') p.setAttribute('league:type', 999);
  else if (kind === 'getter-throws') return t.mock.method(p, 'getProfile', () => { throw new Error('R1 transient getter failure'); });
  else if (kind === 'getter-unknown') return t.mock.method(p, 'getProfile', () => undefined);
}
for (const kind of ['resolve-missing', 'resolve-unanswered', 'resolver-throws-before-rewrite',
  'profile-missing', 'profile-unanswered', 'profile-throws', 'unknown-type', 'getter-throws', 'getter-unknown']) {
  test('R1 live arrival stops and World logout retains recovery on ' + kind, t => {
    const saves = captureLogoutSaves(t); const p = worldRider(t);
    enterRide(p); const getterFault = injectFault(t, p, kind); tick([p], 8); // +13 arrival deadline.
    assert.equal(Train._test.riding.has(p), false);
    assert.equal(core.World.getRemovePlayerQueue().includes(p), true, 'normal removal requested');
    assert.equal(p.getMovementQueue().isMovementBlocked(), true, 'stopped until normal removal');
    assert.ok(p.getAttribute(Train._test.RECOVERY_ATTRIBUTE));
    getterFault?.mock.restore(); // Real persistence also needs an authoritative profile namespace.
    removeThroughWorld(p);
    const saved = saves.findLast(v => v.p === p);
    assert.equal(saved.registered, false);
    assert.deepEqual(saved.xyz, [2479, 5538, 0], 'LASTVALIDATED physical origin, not current policy grant');
    assert.equal(saved.marker.state, 'fault');
    assert.equal(p.getArea(), null);
    assert.equal(p.getMovementQueue().isMovementBlocked(), false);
    const at = xyz(p); tick([p], 80); assert.deepEqual(xyz(p), at, 'no stale token callback moves');
    assert.equal(diaryTasks.some(([who]) => who === p), false);
  });
  if (!kind.startsWith('getter-')) test('R1 normal registeredfalse logout saves anchor and marker on ' + kind, t => {
    const saves = captureLogoutSaves(t); const p = worldRider(t);
    enterRide(p); injectFault(t, p, kind); removeThroughWorld(p);
    const saved = saves.findLast(v => v.p === p);
    assert.equal(saved.registered, false);
    assert.deepEqual(saved.xyz, [2479, 5538, 0]);
    assert.equal(saved.marker.state, 'fault');
    assert.equal(Train._test.riding.has(p), false); assert.equal(p.getArea(), null);
    const at = xyz(p); tick([p], 80); assert.deepEqual(xyz(p), at);
  });
}
test('R1 genuine main and hardcore ignore stale league attributes; known nonarea leagues remain allowed', () => {
  for (const profile of ['main', 'hardcore']) {
    const p = rider(); p.setProfile(profile); p.setAttribute('league:type', 999);
    step(p, 2479, 5538); tick([p], 49); assert.deepEqual(xyz(p), [2444, 5533, 0]);
    tick([p]); assert.equal(p.getMovementQueue().isMovementBlocked(), false);
    Train._test.logout({ player: p }); p.setRegistered(false);
  }
  for (const type of [1, 2, 3]) {
    const p = rider(); p.setProfile('league'); p.setAttribute('league:type', type);
    step(p, 2479, 5538); tick([p], 49); assert.deepEqual(xyz(p), [2444, 5533, 0]);
    Train._test.logout({ player: p }); p.setRegistered(false);
  }
});
test('R1 late policy denial chooses fresh origin or existing home without diary/reboard', t => {
  for (const keepOrigin of [true, false]) {
    const p = worldRider(t); enterRide(p);
    const token = Train._test.riding.get(p);
    if (keepOrigin) p.setAttribute('league:areas', [1, 2]);
    else {
      // SYNTHETIC policy/type transition AFTER a valid type4 departure, not ordinary same-type train travel.
      const areaCommon = require('../plugins/modes/leagueareas/Common.LeagueAreaUnlocks');
      assert.deepEqual(areaCommon.league(6).defaults, [21]);
      assert.equal(areaCommon.league(6).areas.includes(1), false, 'Misthalin can never unlock in type6');
      assert.equal(areaCommon.league(6).areas.includes(21), true);
      p.setAttribute('league:type', 6); p.setAttribute('league:areas', [21]);
      t.diagnostic('SYNTHETIC_TYPE4_TO6_POLICY_TRANSITION_AFTER_VALID_DEPARTURE_NOT_ORDINARY_SAME_TYPE_TRAIN');
    }
    const type = p.getAttribute('league:type');
    assert.deepEqual(p.getAttribute('league:areas'), keepOrigin ? [1, 2] : [21]);
    assert.equal(Regions.isOpenAt(type, p, token.to), false, 'fresh destination genuinely locked');
    assert.equal(Regions.isOpenAt(type, p, token.origin), keepOrigin, 'fresh origin status qualified');
    const expected = keepOrigin ? token.origin.clone()
      : require('../plugins/modes/leagueareas/Enforcement.LeagueAreaUnlocks').home(type, p);
    assert.equal(Regions.isOpenAt(type, p, expected), true, 'actual existing authority permits selected origin/home before asserting success');
    tick([p], 8);
    assert.deepEqual(xyz(p), [expected.getX(), expected.getY(), expected.getZ()]);
    assert.equal(Regions.isOpenAt(type, p, p.getLocation()), true);
    assert.equal(Train._test.riding.has(p), false); assert.equal(Train._test.waiting.has(p), false);
    assert.equal(p.getAttribute(Train._test.RECOVERY_ATTRIBUTE), null);
    assert.equal(p.getMovementQueue().isMovementBlocked(), false);
    assert.equal(diaryTasks.some(([who]) => who === p), false);
  }
});
test('R1 pending departure rechecks policy and source zone before allocating scene/token', t => {
  const p = worldRider(t); step(p, 2479, 5538); p.setAttribute('league:areas', [1, 2]); tick([p], 36);
  assert.deepEqual(xyz(p), [2479, 5538, 0]); assert.equal(Train._test.riding.has(p), false);
  const q = worldRider(t); step(q, 2479, 5538); step(q, 2483, 5538); tick([q], 36);
  assert.equal(Train._test.riding.has(q), false); assert.deepEqual(xyz(q), [2483, 5538, 0]);
});
test('R1 synchronous redirect away and away-back serial both stop success and preserve recovery', t => {
  for (const back of [false, true]) {
    const p = worldRider(t); enterRide(p);
    const original = p.moveTo.bind(p); let once = false;
    const mock = t.mock.method(p, 'moveTo', destination => {
      const result = original(destination);
      if (!once && destination.getX() === 2444) {
        once = true; original(new core.Location(2483, 5538, 0)); if (back) original(destination);
      }
      return result;
    });
    tick([p], 8); mock.mock.restore();
    assert.equal(Train._test.riding.has(p), false); assert.ok(p.getAttribute(Train._test.RECOVERY_ATTRIBUTE));
    assert.equal(core.World.getRemovePlayerQueue().includes(p), true);
    removeThroughWorld(p); assert.deepEqual(xyz(p), [2479, 5538, 0]);
    assert.equal(diaryTasks.some(([who]) => who === p), false);
  }
});
test('R1 save apply precedes World.add/login; recovery stays stopped until a later fresh authority/placement', t => {
  const saves = captureLogoutSaves(t); const p = worldRider(t);
  enterRide(p); faults.set(p, { resolve: 'missing' }); removeThroughWorld(p);
  const first = saves.findLast(v => v.p === p); assert.ok(first.marker);
  const next = worldRider(t, { save: first.snapshot, fault: { profile: 'missing' } });
  assert.notEqual(next, p); assert.equal(next.getUsername(), p.getUsername());
  assert.equal(next.getProfile(), p.getProfile());
  assert.ok(next.getAttribute(Train._test.RECOVERY_ATTRIBUTE));
  assert.equal(next.getMovementQueue().isMovementBlocked(), true);
  assert.equal(core.World.getRemovePlayerQueue().includes(next), true, 'recovery requests removal; later login listeners/bootstrap are not aborted');
  removeThroughWorld(next); const second = saves.findLast(v => v.p === next); assert.ok(second.marker);
  const recovered = worldRider(t, { save: second.snapshot });
  assert.deepEqual(xyz(recovered), [2479, 5538, 0]);
  assert.equal(recovered.getAttribute(Train._test.RECOVERY_ATTRIBUTE), null);
  assert.equal(recovered.getMovementQueue().isMovementBlocked(), false);
});
test('R1 logout phase matrix uses real World removal before save, including before scene and before finish', t => {
  const saves = captureLogoutSaves(t);
  for (const after of [1, 5, 13]) {
    const p = worldRider(t); enterRide(p, after); faults.set(p, { resolve: 'missing' }); removeThroughWorld(p);
    const saved = saves.findLast(v => v.p === p); assert.equal(saved.registered, false);
    assert.deepEqual(saved.xyz, [2479, 5538, 0]); assert.ok(saved.marker);
    const at = xyz(p); tick([p], 80); assert.deepEqual(xyz(p), at);
  }
});

test('R1 inferred room policy covers doors, full envelopes, perimeter fallback and all-plane x/y contract only', () => {
  for (const type of [4, 5, 6]) {
    for (const [x, y, owner] of [[2434,5513,8],[2445,5558,8],[2438,5535,8],[2444,5533,8],
      [2478,5513,1],[2489,5558,1],[2488,5536,1],[2479,5538,1]]) assert.equal(Regions.areaAt(type,x,y),owner);
    for (const [x,y] of [[2433,5535],[2446,5535],[2438,5512],[2438,5559],[2477,5536],[2490,5536],
      [2455,5535],[2542,5506],[2542,5500]]) assert.equal(Regions.areaAt(type,x,y),null);
    const actor = rider(); actor.setAttribute('league:areas',[8]);
    for (let plane=0;plane<4;plane++) {
      assert.equal(Regions.isOpenAt(type,actor,new core.Location(2438,5535,plane)),true);
      assert.equal(Regions.isOpenAt(type,actor,new core.Location(2488,5536,plane)),false);
    }
    actor.setRegistered(false);
  }
  // Political x/y assignment test; NOT proof of plane1..3 floor/cache/collision or true outer-wall margins.
});
test('R1 all source-mirrored compartments keep opposite station ownership without assigning shared scene', () => {
  const data = Train._test.data;
  for (const [key, station] of Object.entries(data.stations)) {
    const z=station.zone, expected=key==='dorgesh'?8:1;
    for (let x=z.minX;x<=z.maxX;x++) for (let y=z.minY;y<=z.maxY;y++) {
      const to=Train._test.arrivalTile(new core.Location(x,y,0));
      assert.equal(Regions.areaAt(4,to.getX(),to.getY()),expected);
    }
  }
  assert.equal(Regions.areaAt(4,2542,5506),null);
  assert.equal(Regions.areaAt(4,2542,5500),null);
});

// R2-R5 controls: actual existing core APIs, materialized UNRUN; not durable DB or full bootstrap proof.
test('R2 HP-zero cancels pending journey; real death respawn survives logout/save apply and permits later boarding', t => {
  const { PlayerDeathTask } = require('../dist/game/task/impl/PlayerDeath');
  const saves = captureLogoutSaves(t);
  t.mock.method(core.GameConstants.PLAYER_PERSISTENCE, 'requestSave', () => {}); // no real backend/disk evidence
  const p = worldRider(t); enterRide(p);
  p.setHitpoints(0); tick([p], 1); // actual scheduled ride callback's HP-zero branch
  assert.equal(Train._test.riding.has(p), false);
  assert.equal(p.getAttribute(Train._test.RECOVERY_ATTRIBUTE), null);
  const death = new PlayerDeathTask(p);
  for (let i = 0; i < 5; i++) death.execute(); // ordinary real animation/respawn ordering
  assert.ok(p.getHitpoints() > 0);
  const respawn = xyz(p); assert.notDeepEqual(respawn, [2479, 5538, 0]);
  tick([p], 80); assert.deepEqual(xyz(p), respawn, 'invalidated token cannot undo respawn');
  removeThroughWorld(p);
  const saved = saves.findLast(v => v.p === p);
  assert.deepEqual(saved.xyz, respawn); assert.equal(saved.marker, null);
  const next = worldRider(t, { save: saved.snapshot });
  assert.deepEqual(xyz(next), respawn); assert.equal(next.getAttribute(Train._test.RECOVERY_ATTRIBUTE), null);
  step(next, 2483, 5538); step(next, 2479, 5538);
  assert.equal(Train._test.waiting.has(next), true, 'normal respawn/relogin does not leave boarding poisoned');
});
test('R2 real death observer cancels ride even when HP-zero callback has not run yet', t => {
  const { PlayerDeathTask } = require('../dist/game/task/impl/PlayerDeath');
  captureLogoutSaves(t); t.mock.method(core.GameConstants.PLAYER_PERSISTENCE, 'requestSave', () => {});
  const p = worldRider(t); enterRide(p); p.setHitpoints(0);
  const death = new PlayerDeathTask(p); for (let i=0;i<5;i++) death.execute();
  const respawn=xyz(p);
  assert.equal(Train._test.riding.has(p),false);assert.equal(p.getAttribute(Train._test.RECOVERY_ATTRIBUTE),null);
  tick([p],80);assert.deepEqual(xyz(p),respawn);
});
test('R3 actual removal queue honors combat hold while disconnected queued world interactions hit the handled gate', async t => {
  const { ClientConnection } = require('../dist/net/NetworkBuilder');
  const { HeadlessBinaryChannel } = require('../dist/net/BinaryChannel');
  captureLogoutSaves(t); const p = worldRider(t); enterRide(p);
  const channel = new HeadlessBinaryChannel(); const connection = new ClientConnection(channel); connection.player=p;
  let downstream=0;
  connection.loginExtension={ packet:()=>{downstream++;return true;} }; // downstream sentinel, no fake input guard
  const packets=[{type:'move',x:2483,y:5538,plane:0,run:false},
    {type:'teleport',x:2483,y:5538,plane:0},{type:'npc_option',index:1,clickType:1},
    {type:'object_option',x:2483,y:5538,id:23285,clickType:1},
    {type:'inventory_action',slot:0,itemId:995,option:1}];
  await connection.dispatch(packets);connection.processTurnPackets();
  assert.equal(downstream,packets.length,'positive pending-ride dispatch reaches the downstream sentinel');
  downstream=0;
  await connection.dispatch(packets); // genuine turnPackets queue BEFORE disconnect
  assert.equal(connection.turnPackets.length,packets.length,'actual pending packet count before disconnect');
  p.getCombat().setUnderAttack(new Player(null));
  faults.set(p,{resolve:'missing'});tick([p],8);
  channel.close();
  assert.equal(p.readyToLeaveWorld(),false);assert.equal(p.isRegistered(),true);
  core.World.process(); // real remove queue must retain recent-combat actor
  assert.equal(p.isRegistered(),true);assert.equal(core.World.getRemovePlayerQueue().includes(p),true);
  connection.processTurnPackets(); // production path explicitly processes queued input after disconnect
  assert.equal(connection.turnPackets.length,0,'the queued packets were actually consumed');
  assert.equal(downstream,0,'all tested world interaction packets handled before downstream dispatch');
  const held=xyz(p);tick([p],20);assert.deepEqual(xyz(p),held);assert.equal(Train._test.riding.has(p),false);
  assert.equal(diaryTasks.some(([who])=>who===p),false);
  p.getCombat().setUnderAttack(null); // release actual combat hold; no readyToLeaveWorld override
  core.World.process();assert.equal(p.isRegistered(),false);
});
test('R3 ordinary pending journey does not claim unrelated input; fault login does not abort later login hooks', t => {
  captureLogoutSaves(t);const p=worldRider(t);enterRide(p);
  const ordinary={player:p,packet:{type:'npc_option',index:1,clickType:1},handled:false};
  PluginManager.emitCustomEvent('player:world-input',ordinary);assert.equal(ordinary.handled,false);
  faults.set(p,{resolve:'missing'});tick([p],8);
  const fault={...ordinary,handled:false};PluginManager.emitCustomEvent('player:world-input',fault);assert.equal(fault.handled,true);
  const api=PluginManager.createApi('TrainR3LaterLoginTest');let continued=0;
  api.onPlayerLogin(({player})=>{if(player===p)continued++;});
  dispatchLogin(p);assert.equal(continued,1,'complete real login dispatch continues after removal request');
  // This deliberately does not claim all packets, direct APIs, later handlers or rendered bootstrap are aborted.
});
for(const marker of [
  {v:2,state:'fault',profile:'league',league:4,anchor:[2479,5538,0]},
  {v:1,state:'bogus',profile:'league',league:4,anchor:[2479,5538,0]},
  {v:1,state:'fault',profile:'league',league:999,anchor:[2479,5538,0]},
  {v:1,state:'fault',profile:'main',league:4,anchor:[2479,5538,0]},
  {v:1,state:'fault',profile:'league',league:4,anchor:[2479.5,5538,0]},
  {v:1,state:'fault',profile:'main',league:null,anchor:[2479,5538,0]},
  {v:1,state:'fault',profile:'unknown',league:null,anchor:[2479,5538,0]},
]) test('R5 malformed/unknown/cross-profile recovery uses physical fallback and still requires fresh policy '+JSON.stringify(marker),t=>{
  captureLogoutSaves(t);const p=worldRider(t);p.setAttribute(Train._test.RECOVERY_ATTRIBUTE,marker);
  faults.set(p,{resolve:'missing'});Train._test.recover({player:p});
  assert.ok(p.getAttribute(Train._test.RECOVERY_ATTRIBUTE));
  const input={player:p,packet:{type:'move'},handled:false};PluginManager.emitCustomEvent('player:world-input',input);assert.equal(input.handled,true);
  removeThroughWorld(p);
  const spawn=core.WorldDefinition.WORLD_SPAWN;assert.deepEqual(xyz(p),[spawn.getX(),spawn.getY(),spawn.getZ()]);
});
test('R5 stale same-profile anchor/old league is provenance only; fresh current policy can correct it before clearing',t=>{
  // Standalone SYNTHETIC stale-save recovery in currenttype6, never a type6 Misthalin train departure.
  captureLogoutSaves(t);
  const areaCommon = require('../plugins/modes/leagueareas/Common.LeagueAreaUnlocks');
  assert.deepEqual(areaCommon.league(6).defaults, [21]);
  assert.equal(areaCommon.league(6).areas.includes(1), false);
  assert.equal(areaCommon.league(6).areas.includes(21), true);
  const hub = areaCommon.area(21).teleport; assert.equal(hub.z, 0, 'actual cache home plane prerequisite');
  const p=worldRider(t,{league:6,unlocked:[21],at:[hub.x,hub.y]});
  const anchor=new core.Location(2479,5538,0);
  p.setAttribute(Train._test.RECOVERY_ATTRIBUTE,{v:1,state:'pending',profile:'league',league:5,anchor:[2479,5538,0]});
  assert.equal(p.getAttribute('league:type'),6);assert.deepEqual(p.getAttribute('league:areas'),[21]);
  assert.equal(Regions.isOpenAt(6,p,anchor),false);
  const home=require('../plugins/modes/leagueareas/Enforcement.LeagueAreaUnlocks').home(6,p);
  assert.equal(Regions.isOpenAt(6,p,home),true,'fresh current policy permits actual home before recovery');
  const serial=p.getTeleportSerial();
  Train._test.recover({player:p});
  assert.deepEqual(xyz(p),[home.getX(),home.getY(),home.getZ()]);
  assert.equal(p.getTeleportSerial(),serial+1);
  assert.equal(p.getAttribute(Train._test.RECOVERY_ATTRIBUTE),null);
  assert.equal(p.getMovementQueue().isMovementBlocked(),false);
  t.diagnostic('SYNTHETIC_STANDALONE_STALE_SAVE_RECOVERY_CURRENT_TYPE6_NO_TRAIN_TRAVEL_CLAIM');
});

test('R4 persistent getter failure is train-local physical cleanup only, not correct-namespace backend saving',t=>{
  const p=worldRider(t);enterRide(p);
  const getter=t.mock.method(p,'getProfile',()=>{throw new Error('persistent profile outage');});
  p.setRegistered(false); // explicitly train-local unregistered branch; NOT World.remove/backend evidence
  Train._test.logout({player:p});
  assert.equal(Train._test.riding.has(p),false);assert.ok(p.getAttribute(Train._test.RECOVERY_ATTRIBUTE));
  assert.deepEqual(xyz(p),[2479,5538,0]);
  getter.mock.restore();p.setRegistered(true); // restore namespace before normal harness cleanup/backend capture
});

test('002 causal login control: missing username skips real wrapper; complete same-player event dispatches',t=>{
  captureLogoutSaves(t);const p=worldRider(t);
  const before=loginObservations.get(p).length;
  PluginManager.emitPlayerLogin({player:p,profile:p.getProfile(),isNewAccount:false});
  assert.equal(loginObservations.get(p).length,before,'the old incomplete fixture event is ignored');
  dispatchLogin(p);assert.equal(loginObservations.get(p).length,before+1);
});
test('002 invalid default-unlock state refuses unavailable home and retains fault through actual logout/login',t=>{
  const saves=captureLogoutSaves(t);const p=worldRider(t);enterRide(p);
  p.setAttribute('league:areas',[2]); // Explicit malformed type4 state, NOT a valid-home positive.
  const home=require('../plugins/modes/leagueareas/Enforcement.LeagueAreaUnlocks').home(4,p);
  assert.equal(Regions.isOpenAt(4,p,home),false,'invalid state really denies computed home');
  tick([p],8);
  assert.equal(Train._test.riding.has(p),false);
  assert.equal(p.getAttribute(Train._test.RECOVERY_ATTRIBUTE).state,'fault');
  assert.equal(core.World.getRemovePlayerQueue().includes(p),true);
  assert.equal(p.getMovementQueue().isMovementBlocked(),true);
  const input={player:p,packet:{type:'move'},handled:false};
  PluginManager.emitCustomEvent('player:world-input',input);assert.equal(input.handled,true);
  removeThroughWorld(p);const saved=saves.findLast(v=>v.p===p);
  assert.deepEqual(saved.xyz,[2479,5538,0],'physical LASTVALIDATED rollback, not fresh policy clearance');
  assert.equal(saved.marker.state,'fault');
  const next=worldRider(t,{save:saved.snapshot});
  assert.equal(next.getUsername(),p.getUsername());assert.equal(next.getProfile(),'league');
  assert.equal(next.getAttribute(Train._test.RECOVERY_ATTRIBUTE).state,'fault');
  assert.equal(core.World.getRemovePlayerQueue().includes(next),true);
  assert.equal(next.getMovementQueue().isMovementBlocked(),true);
  const queued={player:next,packet:{type:'npc_option'},handled:false};
  PluginManager.emitCustomEvent('player:world-input',queued);assert.equal(queued.handled,true);
});
