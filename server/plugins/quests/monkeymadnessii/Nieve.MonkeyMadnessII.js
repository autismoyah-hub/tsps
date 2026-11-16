/**
 * Chapter IV's owned travelling companion and Stronghold assistance.
 * Wiki MMII15374569/Nieve15359405; cache242 follower7109. Canonical NPC combat,
 * with no retail-DPS/animation/immunity override. Cavern assistance is unfinished:
 * its starting-room and owned-copy inputs are not inferred from the outer Area.
 */
const FOLLOWER_ID = 7109;
const RECRUITED = 140, STRONGHOLD_DONE = 145, CAVERN_ENTERED = 155;
const MASK = "monkey-madness-ii:nieve-stronghold-mask";
const LEGACY = "monkey-madness-ii:nieve-legacy-credit";
const MIGRATED = "monkey-madness-ii:nieve-legacy-migrated";

module.exports = function createNieve(api, parent) {
  const { Task, TaskManager, World, RegionManager } = api.core;
  const states = new WeakMap(), actors = new WeakMap(), witnesses = new WeakMap();
  let generation = 0;
  // This is the source-spawn envelope plus one adjacent walking tile, not a
  // guessed retail/cavern room. Assistance is bounded to this Stronghold slice.
  const xs = parent.slotTiles.map(p => p.getX()), ys = parent.slotTiles.map(p => p.getY());
  const boundary = new api.core.Boundary(Math.min(...xs) - 1, Math.max(...xs) + 1,
    Math.min(...ys) - 1, Math.max(...ys) + 1, 0);
  function inStronghold(npc) { return boundary.inside(npc.getLocation()); }
  class StrongholdAssistanceArea extends api.core.Area {
    constructor() { super([boundary]); }
    canAttack(attacker, target) {
      const event = { attacker, target, allow: null };
      canAttack(event);
      return event.allow;
    }
  }
  // Only the cache follower's combat method receives the travelling actor guard;
  // no quest predicate is registered on the global per-world attack hook.
  function NieveCombat(npc) {
    const base = api.core.CombatFactory.MELEE_COMBAT;
    const method = Object.create(base);
    method.canAttack = function(attacker, target) {
      const identity = actors.get(npc);
      if (identity?.slot === null) {
        const event = { attacker, target, allow: null };
        canAttack(event);
        if (event.allow === false || !inStronghold(npc) || !inStronghold(target)) return false;
      }
      return base.canAttack.call(this, attacker, target);
    };
    return method;
  }


  function hasPet(player) {
    const request = { player, pet: null };
    api.emitCustomEvent("pets:current", request);
    return request.pet != null;
  }
  function current(state) {
    return !!state && state.active && states.get(state.player) === state;
  }
  function live(actor) {
    return !!actor && actor.isRegistered() && actor.getHitpoints() > 0
      && !actor.isDyingFunction?.() && !World.getRemoveNPCQueue().includes(actor);
  }
  function eligible(player) {
    const stage = parent.stageOf(player);
    return player.isRegistered() && player.getHitpoints() > 0
      && stage >= RECRUITED && stage < CAVERN_ENTERED && !hasPet(player);
  }
  function maskOf(player) { return (Number(player.getAttribute(MASK) ?? 0) | 0) & 15; }
  function bits(mask) { let n = 0; for (; mask; mask &= mask - 1) n++; return n; }
  function migrate(player) {
    if (player.getAttribute(MIGRATED) === true) return;
    const value = Number(parent.legacyKills(player));
    player.setAttribute(LEGACY, Math.max(0, Math.min(4, Number.isFinite(value) ? Math.floor(value) : 0)));
    player.setAttribute(MASK, 0);
    player.setAttribute(MIGRATED, true);
  }
  function earned(player) {
    return Math.min(4, Number(player.getAttribute(LEGACY) ?? 0) + bits(maskOf(player)));
  }
  function teardown(state) {
    if (!current(state)) return;
    // Invalidate before removeNpc can deliver any further lifecycle hook.
    state.active = false;
    states.delete(state.player);
    state.task?.stop();
    for (const npc of [state.helper, ...state.slots]) {
      if (!npc) continue;
      witnesses.delete(npc); actors.delete(npc);
      npc.getCombat().reset(); npc.setFollowing(null);
      api.removeNpc(npc);
    }
  }
  function context(state, npc) {
    return npc.getPrivateArea() === state.player.getPrivateArea()
      && npc.getLocation().getZ() === state.player.getLocation().getZ();
  }
  function targetOf(state, npc) {
    const identity = actors.get(npc);
    return current(state) && identity?.state === state && identity.slot != null
      && identity.generation === state.generation && state.slots[identity.slot] === npc
      && npc.getOwner() === state.player && npc.isRegistered()
      && !World.getRemoveNPCQueue().includes(npc) && context(state, npc);
  }
  function spawnSlots(state) {
    if (parent.stageOf(state.player) !== RECRUITED) return;
    const mask = maskOf(state.player);
    for (let slot = 0; slot < 4; slot++) {
      if (mask & (1 << slot)) continue;
      const old = state.slots[slot];
      if (old?.isRegistered() && !World.getRemoveNPCQueue().includes(old)) continue;
      const npc = parent.spawnSlot(state.player, slot);
      state.slots[slot] = npc;
      if (npc) actors.set(npc, { state, slot, generation: state.generation });
    }
  }
  function follow(state) {
    state.helper.getCombat().reset();
    state.helper.setFollowing(state.player);
  }
  function process(state) {
    if (!current(state)) return;
    if (!eligible(state.player) || !live(state.helper) || !context(state, state.helper)) {
      teardown(state); return;
    }
    const ownerTarget = state.player.getCombat().getTarget();
    // Automatic swings/Combat generation changes do not establish a new engagement.
    if (state.suppressed && ownerTarget !== state.suppressedTarget) {
      state.suppressed = false;
      state.suppressedTarget = null;
    }
    if (state.target && (!live(state.target) || !targetOf(state, state.target)
      || !inStronghold(state.helper) || !inStronghold(state.target)
      || state.suppressed || (ownerTarget && ownerTarget !== state.target))) state.target = null;
    if (state.target) {
      if (state.helper.getCombat().getTarget() !== state.target) state.helper.getCombat().attack(state.target);
    } else if (state.helper.getCombat().getTarget() || state.helper.getFollowing() !== state.player) follow(state);
    if (parent.stageOf(state.player) >= STRONGHOLD_DONE) {
      for (const npc of state.slots) if (npc) { actors.delete(npc); witnesses.delete(npc); api.removeNpc(npc); }
      state.slots.fill(null);
    } else spawnSlots(state);
  }
  function recover(player) {
    if (!eligible(player)) return null;
    if (player.getPrivateArea() !== null || player.getArea()?.barsFollowers?.(player) === true) return null;
    const existing = states.get(player);
    const nearby = current(existing) && live(existing.helper) && context(existing, existing.helper)
      && player.getLocation().isWithinDistance(existing.helper.getLocation(), api.core.GameConstants.PET_FOLLOW_AUTO_TELEPORT_DISTANCE);
    if (nearby) { spawnSlots(existing); return existing; }
    teardown(existing);
    migrate(player);
    const total = earned(player);
    parent.setKills(player, total);
    if (total >= 4 && parent.stageOf(player) === RECRUITED) parent.advance(player, STRONGHOLD_DONE);
    const here = player.getLocation();
    const at = player.outterTiles().find(tile => !RegionManager.blocked(tile, null)
      && RegionManager.canMovestart(tile, here, 1, 1, null));
    if (!at) return null;
    const helper = api.spawnNpc({ id: FOLLOWER_ID, x: at.getX(), y: at.getY(), z: at.getZ(),
      owner: player, ownerOnly: true, wanderRadius: 0 });
    if (!helper) return null;
    // Existing minion permission: an owned ally may join its owner's opponent.
    // The scoped method/Area still restricts it to this generation's quest slots.
    helper.setMultiCombat(true);
    // A travelling companion belongs to the same scope; private cavern recovery is
    // deliberately unsupported until its owned encounter is qualified.
    if (player.getPrivateArea() !== null) { api.removeNpc(helper); return null; }
    const state = { player, helper, generation: ++generation, active: true,
      slots: Array(4).fill(null), target: null, suppressed: false, suppressedTarget: null, task: null };
    states.set(player, state); actors.set(helper, { state, slot: null, generation: state.generation });
    class CompanionTask extends Task {
      constructor() { super(1, player); }
      execute() { process(state); }
    }
    state.task = new CompanionTask();
    helper.setFollowing(player);
    TaskManager.submit(state.task);
    spawnSlots(state);
    return state;
  }
  function recruit(player, npcId) {
    if (!parent.isRecruitNpc(npcId) || parent.stageOf(player) < 120 || parent.stageOf(player) >= RECRUITED
      || !player.isRegistered() || player.getHitpoints() <= 0) return false;
    if (hasPet(player)) { player.sendMessage("Please pick up your pet before asking Nieve to follow you."); return false; }
    if (player.getPrivateArea() !== null) return false;
    parent.advance(player, RECRUITED);
    recover(player);
    return true;
  }
  function queuedHit({ player, target }) {
    const state = states.get(player);
    if (!current(state) || !eligible(player) || !live(state.helper) || !live(target)
      || !targetOf(state, target) || parent.stageOf(player) !== RECRUITED
      || player.getCombat().getTarget() !== target || state.suppressed
      || !inStronghold(state.helper) || !inStronghold(target)) return;
    state.target = target;
  }
  function appliedHit({ target, hit }) {
    const identity = actors.get(target), state = identity?.state;
    if (!state || !targetOf(state, target) || !eligible(state.player)
      || !live(state.helper) || !context(state, state.helper) || !inStronghold(state.helper) || !inStronghold(target)
      || hit.getSource() !== state.helper || hit.getHealthLost() <= 0 || target.getHitpoints() !== 0
      || hit.isReflected() || hit.getOrigin() !== null) return;
    const occurrence = target.getDeathOccurrence();
    if (!occurrence) return; // A deferred entry cannot borrow this synchronous hit later.
    witnesses.set(target, { occurrence, hit, source: state.helper, generation: state.generation });
  }
  function completedDeath({ npc, occurrence, killer }) {
    const identity = actors.get(npc), state = identity?.state;
    if (!state || !targetOf(state, npc) || !eligible(state.player)
      || parent.stageOf(state.player) !== RECRUITED || !occurrence || npc.getDeathOccurrence() !== occurrence
      || npc.getHitpoints() > 0 || !npc.isDyingFunction()) return;
    const witness = witnesses.get(npc);
    const helperKill = witness?.occurrence === occurrence && witness.source === state.helper
      && witness.generation === state.generation && live(state.helper);
    if (killer !== state.player && !helperKill) return;
    const bit = 1 << identity.slot, oldMask = maskOf(state.player);
    if (oldMask & bit) return;
    // Commit before parent advancement/callbacks, shared by player and helper deaths.
    witnesses.delete(npc);
    state.player.setAttribute(MASK, oldMask | bit);
    const total = earned(state.player);
    parent.setKills(state.player, total);
    state.target = null;
    if (total >= 4) parent.advance(state.player, STRONGHOLD_DONE);
  }
  function canAttack(event) {
    const helperIdentity = actors.get(event.attacker), targetIdentity = actors.get(event.target);
    if (helperIdentity?.slot === null) {
      const state = helperIdentity.state;
      if (!current(state) || state.target !== event.target || state.suppressed
        || !targetOf(state, event.target) || !eligible(state.player)) event.allow = false;
    }
    if (targetIdentity?.slot === null) {
      const state = targetIdentity.state;
      if (!targetOf(state, event.attacker) || !current(state)) event.allow = false;
    }
  }
  function recall(request) {
    if (request.handled || !eligible(request.player)) return;
    const player = request.player;
    const state = recover(player);
    if (!current(state) || !live(state.helper) || !context(state, state.helper)) return;
    const here = player.getLocation(), there = state.helper.getLocation();
    // Walking recall only: no teleport across a denied region/area boundary.
    if (!RegionManager.canMovestart(there, here, 1, 1, player.getPrivateArea())) return;
    state.suppressed = true;
    state.suppressedTarget = player.getCombat().getTarget();
    state.target = null;
    state.helper.getMovementQueue().reset();
    follow(state);
    request.handled = true;
  }
  function login({ player }) { recover(player); }
  function clear({ player }) { teardown(states.get(player)); }
  function retire(player) { teardown(states.get(player)); }
  function following(player) { return current(states.get(player)) && live(states.get(player).helper); }

  api.persistAttribute(MASK, 0);
  api.persistAttribute(LEGACY, 0);
  api.persistAttribute(MIGRATED, false);
  api.onPlayerDealtDamage(queuedHit);
  api.onCustomEvent("combat:damage-applied", appliedHit);
  api.onCustomEvent("npc:death-completed", completedDeath);
  api.onCustomEvent("followers:recall-request", recall);
  api.registerArea(new StrongholdAssistanceArea());
  api.registerNpcCombatMethodProvider([FOLLOWER_ID], NieveCombat, { singleton: false });
  api.onPlayerLogin(login);
  api.onPlayerLogout(clear);
  api.onPlayerDeath(clear);
  return { recruit, recover, retire, following, hasPet };
};
