/**
 * The Dorgesh-Kaan - Keldagrim train (https://oldschool.runescape.wiki/w/Dorgesh-Kaan-Keldagrim_Train_System),
 * from plugins/world/data/dorgesh-kaan-train.json: after Another Slice of H.A.M., a player who steps
 * into either station's train is told it will leave shortly, and is taken to the other city unless
 * they step out first. Staying aboard, they go back and forth.
 *
 * Recorded (rsprox_transcripts/232/03199.txt): the departure line and fade, the short cutscene of
 * the train driving by in a room of its own, and the arrival on the other train's matching tile;
 * the tick of every step is in the data file. A logout mid-ride is saved at the arrival tile; a
 * death ends the ride where the player is.
 */
const path = require("path");
const fs = require("fs");
const { createQuestScenes } = require("../quests/QuestScenes");

const QUEST_KEY = "another_slice_of_ham";
/** The Lumbridge & Draynor hard diary's "Take the train from Dorgesh-Kaan to Keldagrim." */
const DIARY_TASK = { diary: "lumbridge", task: "take-the-train-from-dorgesh-kaan-to-keldagrim" };
/** Toplevel 161's overlay_atmosphere, where the fade (and the drive-by) mount. */
const ATMOSPHERE_UID = (161 << 16) | 1;
const RECOVERY_ATTRIBUTE = "dorgesh-kaan-train:recovery";

let api;
let core;
let data;
let scenes;
let overlayId;

/** Players waiting in a train -> { station, token }; and those riding -> the arrival tile. */
const waiting = new Map();
const riding = new Map();

function questComplete(player) {
  const request = { player, key: QUEST_KEY, complete: null };
  api.emitCustomEvent("quest:is-complete", request);
  return request.complete === true;
}

function message(player, key, city) {
  player.sendMessage(data.messages[key].replace("{city}", city));
}

function setVar(player, name, value) {
  const id = core.CacheDefinitions.getVarbitId(name);
  if (id !== null && id !== undefined) player.getPacketSender().sendVarbit(id, value);
}

function later(token, ticks, run) {
  core.TaskManager.submit(new core.CountdownTask(token, ticks, run));
}

/** Where a rider from `at` steps off: the same place in the other train. */
function arrivalTile(at) {
  const [sumX, sumY] = data.mirror;
  return new core.Location(sumX - at.getX(), sumY - at.getY(), at.getZ());
}

/** A source-supported physical anchor is not current policy clearance. */
function usable(location) {
  if (!location || typeof location.clone !== "function") return false;
  const xyz = [location.getX(), location.getY(), location.getZ()];
  return xyz.every(Number.isInteger) && xyz[0] >= 0 && xyz[0] < 16384 && xyz[1] >= 0 && xyz[1] < 16384 && xyz[2] >= 0 && xyz[2] < 4;
}
/** Marker provenance is metadata, never current travel permission. No timestamp/expiry is implied. */
function recoveryMarker(player) {
  const saved = player.getAttribute(RECOVERY_ATTRIBUTE);
  if (!saved || saved.v !== 1 || !["pending", "fault"].includes(saved.state)
      || !["main", "hardcore", "league", "unknown"].includes(saved.profile)
      || !Array.isArray(saved.anchor) || saved.anchor.length !== 3 || !saved.anchor.every(Number.isInteger)) return null;
  if (saved.profile === "league" ? !Number.isInteger(saved.league) || saved.league < 1 || saved.league > 6 : saved.league !== null) return null;
  const anchor = new core.Location(...saved.anchor);
  return usable(anchor) ? { ...saved, anchor: [...saved.anchor] } : null;
}
function sameRecoveryProfile(player, saved) {
  try { return saved.profile !== "unknown" && player.getProfile() === saved.profile; }
  catch { return false; }
}
function physicalAnchor(player, token) {
  if (usable(token?.faultAnchor)) return token.faultAnchor.clone();
  const saved = recoveryMarker(player);
  if (saved && sameRecoveryProfile(player, saved)) return new core.Location(...saved.anchor);
  // Malformed/orphan/cross-profile marker: configured physical fallback, NEVER an area grant.
  return core.WorldDefinition.WORLD_SPAWN.clone();
}
function remember(player, anchor, state, reply = {}) {
  const previous = recoveryMarker(player);
  // Preserve the last validated provenance through faults; do not manufacture a current approval.
  player.setAttribute(RECOVERY_ATTRIBUTE, { v: 1, anchor: [anchor.getX(), anchor.getY(), anchor.getZ()],
    state, profile: reply.profile ?? previous?.profile ?? "unknown", league: reply.league ?? previous?.league ?? null });
}
/** Only the existing NetworkBuilder WORLD_INTERACTIONS gate is covered, not every server action. */
function holdFaultInput(event) {
  const saved = event.player.getAttribute(RECOVERY_ATTRIBUTE);
  if (saved != null && (!riding.has(event.player) || saved.state !== "pending")) event.handled = true;
}
/** Death observers run before ordinary respawn placement. Do not turn a normal death into rollback. */
function died({ player }) {
  if (riding.has(player)) finish(player, false);
}
function travelReply(player, destination, previous) {
  try {
    // Public runtime character namespace is authoritative for genuine nonleague characters.
    const profile = player.getProfile();
    if (["main", "hardcore"].includes(profile)) return { status: "inactive", profile, league: null,
      destination: destination.clone(), recovery: previous.clone(), corrected: false };
    if (profile !== "league") return null;
    const query = { player, destination, previous, status: "unanswered" };
    api.emitCustomEvent("league:resolve-travel", query);
    if (!["resolved", "inactive"].includes(query.status) || query.profile !== "league"
        || !Number.isInteger(query.league) || typeof query.corrected !== "boolean"
        || !usable(query.destination) || !usable(query.recovery)) return null;
    if (query.status === "inactive" && query.inactiveReason !== "known-nonarea-league") return null;
    return query;
  } catch { return null; }
}
function clearRecovery(player) { player.setAttribute(RECOVERY_ATTRIBUTE, null); }
function stopForPolicyFault(player, token) {
  const anchor = physicalAnchor(player, token);
  remember(player, anchor, "fault");
  waiting.delete(player);
  if (!player.isRegistered()) {
    // Normal removal has no World movement enforcement. Place BEFORE fallible UI cleanup/save.
    riding.delete(player);
    player.setLocation(anchor);
    scenes.controls(player, false);
    setVar(player, "fov_clamp", 0);
    return;
  }
  try { finish(player, false); }
  finally {
    try { scenes.controls(player, true, true); }
    finally { player.requestLogout(); }
  }
}
function resolvedArrival(player, token) {
  const reply = travelReply(player, token.to.clone(), token.origin.clone());
  if (!reply) { stopForPolicyFault(player, token); return null; }
  token.to = reply.destination;
  token.faultAnchor = reply.recovery.clone();
  token.corrected = token.corrected || reply.corrected;
  return token.to;
}
function recover({ player }) {
  if (player.getAttribute(RECOVERY_ATTRIBUTE) == null) return;
  const anchor = physicalAnchor(player);
  const reply = travelReply(player, anchor, anchor);
  if (!reply) { stopForPolicyFault(player); return; }
  const token = { faultAnchor: reply.recovery };
  const serial = player.getTeleportSerial();
  player.moveTo(reply.destination);
  if (!player.isRegistered() || !player.getLocation().equals(reply.destination) || player.getTeleportSerial() !== serial + 1) {
    stopForPolicyFault(player, token); return;
  }
  clearRecovery(player); // Only after fresh authority AND checked actual placement.
  scenes.controls(player, false);
  setVar(player, "fov_clamp", 0);
}

function board({ player, zone }) {
  const key = Object.keys(data.stations).find((name) => data.stations[name].zone === zone);
  if (!key || riding.has(player) || player.getAttribute(RECOVERY_ATTRIBUTE) != null || !questComplete(player)) return;
  const station = data.stations[key];
  if (!core.TeleportHandler.destinationAllowed(player, arrivalTile(player.getLocation()))) return;
  const policy = travelReply(player, arrivalTile(player.getLocation()), player.getLocation());
  if (!policy || policy.corrected) return;
  const token = {};
  waiting.set(player, { station: key, token });
  message(player, "boarded", data.stations[station.to].name);
  later(token, data.departTicks, () => {
    if (waiting.get(player)?.token !== token) return;
    waiting.delete(player);
    if (player.isRegistered?.() === false || player.getHitpoints?.() <= 0) return;
    depart(player, key);
  });
}

function alight({ player, zone }) {
  const entry = waiting.get(player);
  if (entry && data.stations[entry.station].zone === zone) waiting.delete(player);
}

/** The ride, on the recorded ticks (see the data file's `_ride`). */
function depart(player, key) {
  const station = data.stations[key];
  const destination = data.stations[station.to];
  const ride = data.ride;
  if (!inside(station.zone, player.getLocation())) return;
  const to = arrivalTile(player.getLocation());
  if (!core.TeleportHandler.destinationAllowed(player, to)) return;
  const policy = travelReply(player, to, player.getLocation());
  if (!policy || policy.corrected) return;
  const token = { to, from: key, origin: player.getLocation().clone(), corrected: false, faultAnchor: policy.recovery.clone() };
  remember(player, token.faultAnchor, "pending", policy);
  riding.set(player, token);
  message(player, "departs", destination.name);
  scenes.fade(player, true);
  scenes.controls(player, true, false);
  const step = (tick, run) => later(token, tick, () => {
    if (riding.get(player) !== token) return;
    if (player.isRegistered?.() === false) return;
    if (player.getHitpoints?.() <= 0) return finish(player, false);
    run();
  });
  step(ride.sceneTick, () => {
    const [minX, maxX, minY, maxY, z] = ride.bounds;
    scenes.open(player, "dorgesh-kaan train", [[minX, maxX, minY, maxY, z]], { allowSummonPet: false, barsFollowers: true });
    player.moveTo(new core.Location(ride.scene[0], ride.scene[1], 0));
    scenes.controls(player, true, true);
    setVar(player, "fov_clamp", 1);
    scenes.hideTabs(player);
  });
  step(ride.fadeInTick, () => {
    scenes.fade(player, false);
    const [mx, my, mh] = ride.cameraMove, [lx, ly, lh] = ride.cameraLook;
    const sender = player.getPacketSender();
    sender.sendCameraMoveTo(mx, my, mh, { rate: ride.cameraMoveRates[0], rate2: ride.cameraMoveRates[1] });
    sender.sendCameraLookAt(lx, ly, lh, { rate: ride.cameraLookRates[0], rate2: ride.cameraLookRates[1] });
    sender.sendSound(ride.sound, 1, 0);
  });
  step(ride.overlayTick, () => player.getPacketSender().sendSubInterface(ATMOSPHERE_UID, overlayId, 1));
  step(ride.fadeOutTick, () => scenes.fade(player, true));
  step(ride.arriveTick, () => arrive(player));
  step(ride.restoreTick, () => finish(player, true));
}

/** The teleport onto the other train, still held; the rest comes a tick later (see `finish`). */
function arrive(player) {
  const token = riding.get(player);
  const destination = resolvedArrival(player, token);
  if (!destination || riding.get(player) !== token) return;
  if (scenes.get(player)) scenes.end(player, false);
  const serial = player.getTeleportSerial();
  player.moveTo(destination);
  if (riding.get(player) !== token) return;
  if (!player.isRegistered() || !player.getLocation().equals(destination) || player.getTeleportSerial() !== serial + 1) {
    stopForPolicyFault(player, token); return;
  }
  clearRecovery(player);
  if (token.corrected) { finish(player, false); return; }
  scenes.controls(player, true, true);
}

/**
 * The ride over, a tick after the teleport (`arrived`), or where the player is (a death): the
 * controls, tabs and camera back, the fade in, and, still aboard, the other train's line, whose
 * 36 ticks start here (the zone's own enter came while riding, at the teleport).
 */
function finish(player, arrived) {
  const token = riding.get(player);
  if (!token) return;
  riding.delete(player);
  // Ordinary HP-zero/death/cancel ends the pending journey. Intentional faults were reclassified first.
  if (!arrived && player.getAttribute(RECOVERY_ATTRIBUTE)?.state === "pending") clearRecovery(player);
  if (scenes.get(player)) scenes.end(player, false);
  scenes.controls(player, false);
  player.getPacketSender().sendCameraReset();
  setVar(player, "fov_clamp", 0);
  scenes.restoreTabs(player);
  scenes.fade(player, false);
  if (!arrived || token.corrected) return;
  const destination = data.stations[token.from].to;
  const zone = data.stations[destination].zone;
  if (inside(zone, player.getLocation())) board({ player, zone });
  if (token.from === "dorgesh") api.emitCustomEvent("diary:task", { player, ...DIARY_TASK });
}

function inside(zone, at) {
  return at.getX() >= zone.minX && at.getX() <= zone.maxX && at.getY() >= zone.minY && at.getY() <= zone.maxY
    && (!zone.levels || zone.levels.includes(at.getZ()));
}

/** A scene ended by a logout: the save is on the other train. */
function sceneEnded(player, scene, logout) {
  const token = riding.get(player);
  if (!logout || !token) return;
  riding.delete(player);
  const destination = resolvedArrival(player, token);
  if (!destination) return;
  player.setLocation(destination);
  if (player.getLocation().equals(destination)) clearRecovery(player);
  else { stopForPolicyFault(player, token); return; }
  setVar(player, "fov_clamp", 0);
}

function logout({ player }) {
  waiting.delete(player);
  const token = riding.get(player);
  if (!token) {
    if (player.getAttribute(RECOVERY_ATTRIBUTE) != null) stopForPolicyFault(player);
    return;
  }
  if (scenes.get(player)) scenes.end(player, true);
  else {
    riding.delete(player);
    const destination = resolvedArrival(player, token);
    if (!destination) return;
    player.setLocation(destination);
    if (player.getLocation().equals(destination)) clearRecovery(player);
    else { stopForPolicyFault(player, token); return; }
    scenes.controls(player, false);
    setVar(player, "fov_clamp", 0);
  }
}

function initialize(pluginApi) {
  api = pluginApi;
  core = pluginApi.core;
  data = JSON.parse(fs.readFileSync(path.join(__dirname, "data", "dorgesh-kaan-train.json"), "utf8"));
  overlayId = core.CacheDefinitions.getInterfaceId(data.ride.overlay);
  scenes = createQuestScenes(api, { onEnd: sceneEnded });
}

module.exports = {
  name: "DorgeshKaanTrain",
  description: "The Dorgesh-Kaan - Keldagrim train: after Another Slice of H.A.M., stepping into either station's train takes the player to the other city.",
  members: true,
  register(pluginApi) {
    initialize(pluginApi);
    for (const station of Object.values(data.stations)) {
      api.onZoneEnter(station.zone, board);
      api.onZoneExit(station.zone, alight);
    }
    api.persistAttribute(RECOVERY_ATTRIBUTE, null);
    api.onPlayerLogin(recover);
    api.onPlayerLogout(logout);
    api.onCustomEvent("player:world-input", holdFaultInput);
    api.onCustomEvent("player:died", died);
  },
  _test: { waiting, riding, arrivalTile, board, alight, depart, finish, logout, recover, RECOVERY_ATTRIBUTE, get data() { return data; } },
};
