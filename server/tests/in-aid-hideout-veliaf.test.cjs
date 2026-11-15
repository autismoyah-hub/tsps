// Run after `yarn build`: node --test tests/in-aid-hideout-veliaf.test.cjs
// A player at the Myreque hideout sees ONE Veliaf Hurtz. The world spawn (cache row 9620, transform varbit 10367) already
// shows him as 5048; In Aid used to add an owner-only 5048 beside it for players whose In Aid stage was set before login
// (stage < 20, or 290 <= stage < 320). Real PluginManager, real login, real NPC world; counts the NPCs the player can
// see that resolve to a Veliaf Hurtz. The lines starting VELIAF-LINEAGE are the observation (id, owner, resolved id, tile).
const assert = require('node:assert/strict');
const path = require('node:path');
const { before, after, test } = require('node:test');
require('../dist/Server').Server.installProductionPathResolver();
const { CachePipeline } = require('../dist/game/cache/CachePipeline');
const { PluginManager: PM } = require('../dist/plugins/PluginManager');
const { Player } = require('../dist/game/entity/impl/player/Player');
const { PlayerSession } = require('../dist/net/PlayerSession');

const HIDEOUT = { x: 3506, y: 9838, z: 0 };
let core, next = 4100;
before(async () => {
  await CachePipeline.initialize(path.resolve(__dirname, '..'));
  PM.loadFromDirectory(path.resolve(__dirname, '../plugins')); core = PM.getCoreApi();
  PM.emitServerStartup({});
});
after(() => PM.emitServerShutdown({}));

function loggedIn(t, stage) {
  const channel = { kind: 'test', remoteAddress: '127.0.0.1', isOpen: () => true, send: () => true, close() {}, onData() {}, onError() {}, onClose() {}, flush() {} };
  const p = new Player(new PlayerSession(channel)); p.setIndex(next++); p.setUsername('aidveliaf' + next);
  p.setRegistered(true); p.setLocation(new core.Location(HIDEOUT.x, HIDEOUT.y, 0)); p.needsPlacement = false;
  if (stage !== undefined) p.setAttribute('quest.in_aid_of_the_myreque.stage', stage); // set BEFORE login, as a saved player has it
  PM.emitPlayerLogin({ player: p, username: p.getUsername() });
  t.after(() => { PM.emitPlayerLogout?.({ player: p, username: p.getUsername() }); p.setRegistered(false); core.TaskManager.cancelTasks(p.getIndex()); });
  return p;
}

function visibleVeliafs(p) {
  const seen = [];
  for (const npc of core.World.getNpcsNear(new core.Location(HIDEOUT.x, HIDEOUT.y, 0), 14)) {
    if (npc.isOwnerOnly?.() && npc.getOwner?.() !== p) continue; // another player's private spawn
    const resolvedId = npc.getContentId(p);
    const name = core.NpcDefinition.forId(resolvedId)?.getName?.();
    if (name === 'Veliaf Hurtz') seen.push({ id: npc.getId(), resolvedId, ownerOnly: !!npc.isOwnerOnly?.(), x: npc.getLocation().getX(), y: npc.getLocation().getY() });
  }
  return seen;
}

for (const stage of [undefined, 0, 10, 290, 300]) {
  test(`a player at the hideout with In Aid stage ${stage ?? 'unset'} (set before login) sees exactly one Veliaf Hurtz`, (t) => {
    const p = loggedIn(t, stage);
    const seen = visibleVeliafs(p);
    console.log(`VELIAF-LINEAGE stage=${stage ?? 'unset'} ${JSON.stringify(seen)}`);
    assert.equal(seen.length, 1, JSON.stringify(seen));
  });
}
