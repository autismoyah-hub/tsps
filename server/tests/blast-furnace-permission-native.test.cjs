// Unexecuted review draft. Requires an isolated one-shot process: plugin module
// globals are initialized here and are not restored for same-process reuse.
// Selective registrations, directly placed actors and configured NPC; no server
// startup or natural entrance journey is represented by this fixture.
// Run only after independent source and cleanup review.
// The no-permission expectations are inferred from the Wiki foreman wording;
// no exact grant tick or persistence is asserted.
const assert = require('node:assert/strict');

// Real PluginManager cleanup pattern, adapted from qh728-q039.test.cjs.
// Only use in a serial, isolated test process with no concurrent registrations.
function snapshotHooks(pm) {
  const arrays = ['serverStartupHooks', 'serverShutdownHooks', 'logoutHooks',
    'itemOnObjectHooks', 'npcInteractionHooks', 'npcAnyInteractionHooks',
    'npcDialogueConditionHooks', 'canShopHooks'];
  const maps = ['objectHooksByName', 'objectHooksById', 'customEventHooks'];
  const lengths = new Map(arrays.map(key => [key, pm[key].length]));
  const entries = new Map(maps.map(key => [key,
    new Map([...pm[key]].map(([name, hooks]) => [name, hooks.length]))]));
  const order = pm.nextObjectHookOrder;
  return () => {
    for (const [key, length] of lengths) pm[key].length = length;
    for (const [key, old] of entries) {
      for (const [name, hooks] of [...pm[key]]) {
        if (!old.has(name)) pm[key].delete(name);
        else hooks.length = old.get(name);
      }
    }
    pm.nextObjectHookOrder = order;
  };
}

// No simulated permission flag, direct dialogue callback or line-event spy.
async function setupNativeWorld() {
  const path = require('node:path');
  const { createRequire } = require('node:module');
  const server = path.resolve(process.cwd());
  const req = createRequire(path.join(server, 'tests', 'blast-furnace.test.cjs'));
  req('../dist/Server').Server.installProductionPathResolver();
  const { CachePipeline } = req('../dist/game/cache/CachePipeline');
  const { PluginManager: pm } = req('../dist/plugins/PluginManager');
  const { NpcDefinitionLoader } = req('../dist/game/definition/loader/impl/NpcDefinitionLoader');
  const { PlayerSave } = req('../dist/game/entity/impl/player/persistence/PlayerSave');
  const { AreaManager } = req('../dist/game/model/areas/AreaManager');
  await CachePipeline.initialize(server);
  const core = pm.getCoreApi();
  core.RegionManager.init();
  core.RegionManager.loadMapFiles(1945, 4965);
  new NpcDefinitionLoader().load();
  core.ObjectDefinition.init();
  const restoreHooks = snapshotHooks(pm);
  const areaLength = AreaManager.areas.length;
  const hadPersistentKey = PlayerSave.persistentAttributeKeys.has('blast-furnace');
  const api = pm.createApi('BlastFurnacePermissionDraft');
  const furnace = req('../plugins/minigames/BlastFurnace.plugin');
  const dialogues = req('../plugins/npcs/NpcDialogues.plugin');
  const bankCommon = req('../plugins/interface/bankpin/Common.BankPin');
  const bankGuard = req('../plugins/interface/bankpin/Guard.BankPin');
  const spawned = [];
  const originalSpawn = api.spawnNpc;
  api.spawnNpc = options => { const npc = originalSpawn(options); spawned.push(npc); return npc; };
  const cleanup = players => {
    const errors = [];
    const attempt = fn => { try { fn(); } catch (error) { errors.push(error); } };
    for (const p of players) {
      attempt(() => p.getPacketSender().sendInterfaceRemoval());
      attempt(() => p.getDialogueManager().reset());
      attempt(() => furnace._test.logout({ player: p }));
      attempt(() => core.TaskManager.cancelTasks(p));
      attempt(() => core.TaskManager.cancelTasks(p.getIndex()));
      attempt(() => { if (p.isRegistered()) core.World.getPlayers().remove(p); });
    }
    for (const npc of spawned) attempt(() => { if (npc?.isRegistered()) pm.removeNpc(npc); });
    attempt(() => { AreaManager.areas.length = areaLength; AreaManager.areaSearchIndex = null; });
    attempt(() => { if (!hadPersistentKey) PlayerSave.persistentAttributeKeys.delete('blast-furnace'); });
    attempt(restoreHooks);
    if (errors.length) throw new AggregateError(errors, 'fixture cleanup failed');
  };
  try {
    furnace.register(api);
    dialogues.register(api);
    bankCommon.init(api);
    bankGuard.attach(api);
    assert.ok((pm.customEventHooks.get('bank-pin:guard') ?? [])
      .some(hook => hook.pluginName === 'BlastFurnacePermissionDraft'), 'real PIN guard registered');
    assert.equal(core.WorldDefinition.hasGlobalWorldTag('blast-furnace'), true, 'automated world');
    const belt = [...core.MapObjects.mapObjects.values()].flat().find(object =>
      object.getDefinition()?.getName() === 'Conveyor belt' &&
      object.getDefinition().getInteractions()?.includes('Put-ore-on') &&
      object.getLocation().isWithinDistance(new core.Location(1942, 4966, 0), 20));
    assert.ok(belt, 'loaded scene belt');
    const players = [];
    return { req, core, pm, api, furnace, belt, spawned, bankCommon, players, serial: 0,
      close() {
        cleanup(players);
      } };
  } catch (error) {
    try { cleanup([]); } catch (cleanupError) {
      throw new AggregateError([error, cleanupError], 'setup and cleanup failed');
    }
    throw error;
  }
}
function makeActor(world, level) {
  const { req, core, furnace, bankCommon } = world;
  const { Player } = req('../dist/game/entity/impl/player/Player');
  const { PlayerSession } = req('../dist/net/PlayerSession');
  const { ClientConnection } = req('../dist/net/NetworkBuilder');
  const { SkillManager } = req('../dist/game/content/skill/SkillManager');
  const channel = { kind: 'test', isOpen: () => true, send: () => true,
    close() {}, onData() {}, onClose() {}, onError() {} };
  const session = new PlayerSession(channel), p = new Player(session);
  session.setPlayer(p);
  p.setUsername(`bfpermit${++world.serial}`);
  p.setLocation(new core.Location(1942, 4965, 0));
  assert.equal(p.getLocation().getZ(), 0);
  assert.equal(world.belt.getLocation().getZ(), 0);
  for (const loc of [p.getLocation(), world.belt.getLocation()]) {
    assert.ok(loc.getX() >= 1934 && loc.getX() <= 1959 && loc.getY() >= 4955 && loc.getY() <= 4976, 'furnace room');
  }
  assert.ok(p.getLocation().isWithinDistance(world.belt.getLocation(), 20));
  p.onAdd = () => {}; p.onRemove = () => {};
  assert.equal(core.World.getPlayers().add(p), true);
  world.players.push(p);
  assert.equal(core.World.getPlayers().get(p.getIndex()), p);
  const skills = p.getSkillManager();
  skills.setMaxLevels(core.Skill.SMITHING, level, false)
    .setCurrentLevel(core.Skill.SMITHING, level, false)
    .setExperience(core.Skill.SMITHING, SkillManager.getExperienceForLevel(level));
  assert.equal(skills.getMaxLevel(core.Skill.SMITHING), level);
  assert.equal(skills.getCurrentLevel(core.Skill.SMITHING), level);
  p.getInventory().adds(core.ItemIdentifiers.IRON_ORE, 1);
  if (level < 60) p.getInventory().adds(core.ItemIdentifiers.COINS, 2500);
  const state = furnace._test.stateOf(p);
  state.coffer = 1000;
  assert.equal(bankCommon.hasPin(p), false);
  const probe = { player: p, feature: 'blast-furnace-belt', resume() {}, locked: false };
  world.api.emitCustomEvent('bank-pin:guard', probe);
  assert.equal(probe.locked, false);
  const connection = new ClientConnection(channel); connection.player = p;
  return { world, p, connection };
}
async function resume(actor, choice) {
  const { p, connection, world: { core } } = actor;
  const group = p.getPacketSender().getChatboxGroupId();
  await connection.dispatch([{ type: 'dialogue_continue',
    widgetId: choice == null ? (group << 16) | ([231, 217].includes(group) ? 5 : 0)
      : core.MultiChatboxPrompt.OPTIONS_WIDGET_ID, childIndex: choice ?? -1 }]);
  connection.processTurnPackets();
}
async function nextMenu(actor, observed) {
  const { p, world: { core } } = actor;
  for (let i = 0; i < 100; i++) {
    const pending = core.MultiChatboxPrompt.getPending(p);
    if (pending) return pending;
    const current = p.getDialogueManager().getCurrent();
    if (!current) return null;
    const text = current.getText?.();
    if (text) observed.push({ text: String(text).replace(/\s+/g, ' ').trim(),
      group: p.getPacketSender().getChatboxGroupId() });
    await resume(actor);
  }
  throw Error('dialogue did not reach menu or closure');
}
async function talkAndChoose(actor, route) {
  const { p, world } = actor, { core, pm } = world;
  const spawn = world.req('../data/definitions/npc-spawns.json')
    .find(s => s.id === 2923 && s.name === 'Blast Furnace Foreman');
  assert.ok(spawn);
  const npc = core.NPC.create(spawn.id, new core.Location(spawn.x, spawn.y, spawn.level));
  assert.equal(core.World.getNpcs().add(npc), true); world.spawned.push(npc);
  assert.equal(core.World.getNpcs().get(npc.getIndex()), npc);
  const definition = npc.getCurrentDefinition(p);
  assert.equal(definition.getName(), 'Blast Furnace Foreman');
  const clickType = definition.getActions().indexOf('Talk-to') + 1;
  assert.ok(clickType > 0);
  assert.equal(pm.emitNpcInteraction({ player: p, npc, npcId: npc.getContentId(p),
    definition, clickType, handled: false }), true);
  const labels = route === 'permission'
    ? ['What?', "What's the blast furnace?", 'Can I use the furnace to smelt ore?', 'I have level 60!']
    : route === 'paid'
      ? ['What?', "What's the blast furnace?", 'Can I use the furnace to smelt ore?', "I don't have level 60.", 'Okay.']
      : ['Okay.'];
  const observed = [];
  for (const label of labels) {
    const pending = await nextMenu(actor, observed);
    assert.ok(pending, `missing menu for ${label}`);
    const index = pending.options.indexOf(label);
    assert.ok(index >= 0, `missing ${label} in ${JSON.stringify(pending.options)}`);
    await resume(actor, index + 1);
  }
  assert.equal(await nextMenu(actor, observed), null, 'dialogue closed');
  assert.equal(core.MultiChatboxPrompt.getPending(p), null);
  if (route === 'permission') assert.ok(observed.some(line => line.group === 231 && line.text === 'Feel free to use the furnace. Remember, you only need half as much coal as with a regular furnace.'));
  if (route === 'paid') assert.ok(observed.some(line => line.group === 231 && line.text.startsWith('Okay, you can use the furnace for ten minutes.')));
}
function loadIron(actor, permit = 'none') {
  const { world, p } = actor;
  const clickType = world.belt.getDefinition().getInteractions().indexOf('Put-ore-on') + 1;
  return observeBelt(world.pm, world.core, world.furnace, p, world.belt, clickType, { permit });
}
async function payForeman(actor) { await talkAndChoose(actor, 'paid'); }

function unchanged(before, after) {
  assert.equal(after.inventoryIron, before.inventoryIron);
  assert.equal(after.conveyorIron, before.conveyorIron);
}
function admitted(before, after) {
  assert.equal(after.inventoryIron, before.inventoryIron - 1);
  assert.equal(after.conveyorIron, before.conveyorIron + 1);
}

// Sample immediately around dispatch: advancing tasks can remove conveyor loads.
function observeBelt(pm, core, furnace, player, belt, clickType, { permit = 'none' } = {}) {
  const state = furnace._test.stateOf(player);
  const iron = core.ItemIdentifiers.IRON_ORE;
  const snapshot = () => ({
    inventoryIron: player.getInventory().getAmount(iron),
    conveyorIron: state.conveyor.filter(load => load.item === iron)
      .reduce((sum, load) => sum + load.count, 0),
  });
  assert.equal(state.coffer > 0, true);
  if (permit === 'paid') assert.ok(state.permitUntil > Date.now());
  if (permit === 'none') assert.equal(state.permitUntil, 0);
  assert.deepEqual(state.ores, {});
  assert.deepEqual(state.bars, {});
  assert.deepEqual(state.conveyor, []);
  assert.equal(player.getInventory().getAmount(iron), 1);
  const before = snapshot();
  const handled = pm.emitObjectInteraction({ player, object: belt,
    objectId: belt.getId(), clickType, location: belt.getLocation(), handled: false });
  const after = snapshot();
  assert.equal(handled, true);
  return { before, after };
}

// Intended sequencing: each observation is synchronous, before task ticks.
// Implement as independently reported cases after the setup gate succeeds.
async function draft(t) {
  const world = await setupNativeWorld();
  let failure;
  try {
    const positive = await makeActor(world, 60);
    await talkAndChoose(positive, 'permission');
    let result = loadIron(positive, 'free');
    admitted(result.before, result.after); // setup gate precedes target
    await t.test('inferred: no conversation leaves ore at belt', async () => {
      const fresh = makeActor(world, 60);
      const observation = loadIron(fresh);
      unchanged(observation.before, observation.after);
    });
    await t.test('inferred: unrelated conversation does not grant permission', async () => {
      const unrelated = makeActor(world, 60);
      await talkAndChoose(unrelated, 'unrelated-exit');
      const observation = loadIron(unrelated);
      unchanged(observation.before, observation.after);
    });
    await t.test('below 60 paid dialogue preserves admission', async () => {
      const paid = makeActor(world, 50);
      const before = loadIron(paid);
      unchanged(before.before, before.after);
      await payForeman(paid);
      const after = loadIron(paid, 'paid');
      admitted(after.before, after.after);
    });
  } catch (error) { failure = error; }
  try { await world.close(); } catch (error) {
    if (failure) throw new AggregateError([failure, error], 'fixture and cleanup failed');
    throw error;
  }
  if (failure) throw failure;
}

module.exports = { draft, snapshotHooks, observeBelt };
require('node:test').test('Blast Furnace inferred free permission admission', draft);
