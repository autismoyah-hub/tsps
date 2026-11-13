/**
 * Temple of Ikov (members).
 *
 * The words come from the "Temple of Ikov" transcript page; this plugin supplies
 * the variant selector for Lucien, Winelda and the Guardians of Armadyl (all
 * indexed), the start hook, the prose-condition answers, the limpwurt-root
 * teleport, the Fire Warrior kill and the staff hand-in that completes the
 * quest.
 *
 * Legacy saved stages keep their meaning; independent meeting/payment/kill and
 * branch facts determine chronological progress sent through varp 26.
 *
 * The world's Lucien spawns (3443/3444) are transform placeholders with no
 * options in this cache, so each quester gets a private working Lucien: 13608
 * (Talk-to only) at the Flying Horse Inn and 13607 (level 14, Talk-to/Attack)
 * at his house. Both are removed on completion/logout. The Fire Warrior (3448)
 * is spawned when his door is opened; Winelda's teleports cross the lava; the
 * push-wall and ice-arrow chest are wired. The Staff of Armadyl is a physical
 * ground spawn; its native pickup is guarded by the real guardian encounter.
 * The Armadyl pendant is awarded at its transcript's physical message action.
 *
 * The player's side (varp-less attribute) drives the Guardians: agreeing to help
 * them sets "guardians" and hands over the Armadyl pendant; taking Lucien's
 * side in the dialogue sets "lucien" and unlocks taking the staff.
 */
module.exports = function registerTempleOfIkovQuest(api) {
  const { Item, Skill, ItemIdentifiers, NpcIdentifiers } = api.core;
  const { registerQuest, giveReward, startTranscript } = require("../QuestRuntime");

  const LUCIEN_NPC_IDS = new Set([
    NpcIdentifiers.LUCIEN,
    NpcIdentifiers.LUCIEN_2,
    NpcIdentifiers.LUCIEN_3,
    NpcIdentifiers.LUCIEN_4,
    NpcIdentifiers.LUCIEN_5,
    NpcIdentifiers.LUCIEN_6,
    3443,
    3444,
  ]);
  /** 13608 has Talk-to only (the inn informant); 13607 is the level-14 fight. */
  const INN_LUCIEN_NPC_ID = NpcIdentifiers.LUCIEN_5;
  const HOUSE_LUCIEN_NPC_ID = NpcIdentifiers.LUCIEN_4;
  const WINELDA_NPC_ID = NpcIdentifiers.WINELDA;
  const WINELDA_NPC_IDS = new Set([WINELDA_NPC_ID]);
  const GUARDIAN_NPC_IDS = new Set([
    NpcIdentifiers.GUARDIAN_OF_ARMADYL,
    NpcIdentifiers.GUARDIAN_OF_ARMADYL_2,
  ]);
  const FIRE_WARRIOR_NPC_ID = NpcIdentifiers.FIRE_WARRIOR_OF_LESARKUS;

  const VARP_TEMPLE_OF_IKOV = 26;
  const STAGE_STARTED = 10;
  const STAGE_CROSSED_LAVA = 50;
  const STAGE_FIRE_WARRIOR_KILLED = 60;
  const STAGE_HAS_STAFF = 70;
  const STAGE_COMPLETE = 80;

  const PENDANT_OF_LUCIEN_ITEM_ID = ItemIdentifiers.PENDANT_OF_LUCIEN;
  const ARMADYL_PENDANT_ITEM_ID = ItemIdentifiers.ARMADYL_PENDANT;
  const STAFF_OF_ARMADYL_ITEM_ID = ItemIdentifiers.STAFF_OF_ARMADYL;
  const ICE_ARROWS_ITEM_ID = ItemIdentifiers.ICE_ARROWS;
  const LIMPWURT_ROOT_ITEM_ID = ItemIdentifiers.LIMPWURT_ROOT;
  const LIMPWURT_REQUIRED = 20;
  const LEVER_BRACKET_ID = 86;
  const FITTED_LEVER_ID = 87;
  const LOWERED_LEVER_ID = 88;
  const LEVER_BRACKET_TILE = { x: 2671, y: 9804, z: 0 };
  const TRAPPED_LEVER_ID = 91;
  const TRAPPED_LEVER_TILE = { x: 2665, y: 9855, z: 0 };
  const BOOTS_STAIRS_DOWN_ID = 98;
  const BOOTS_STAIRS_DOWN_TILE = { x: 2650, y: 9804, z: 0 };
  const ICE_GATE_IDS = new Set([89, 90]);
  const TRAP_DOOR_ID = 92;
  const PIT_LADDER_ID = 101;
  const PIT_LADDER_TILE = { x: 2682, y: 9849, z: 0 };
  const PIT_EXIT_ID = 100;
  const PIT_EXIT_TILE = { x: 2665, y: 9849, z: 0 };

  // World tiles (cache placements): Lucien placeholders, the fire-warrior door,
  // the wall north of the skeletons and the Temple of Ikov ice chest.
  // Wiki: the start is the Flying Horse Inn in East Ardougne (the disguised 3444's placement);
  // his house, where he is fought or given the staff, is between Edgeville and the GE (3443's).
  const INN_LUCIEN_TILE = { x: 2573, y: 3321, z: 0 };
  const HOUSE_LUCIEN_TILE = { x: 3122, y: 3483, z: 0 };
  const FIRE_WARRIOR_DOOR_ID = 93;
  const FIRE_WARRIOR_DOOR_TILE = { x: 2646, y: 9870, z: 0 };
  /** The cache placement: a straight wall piece (shape 0) facing rotation 1. */
  const FIRE_WARRIOR_DOOR_TYPE = 0;
  const FIRE_WARRIOR_DOOR_FACE = 1;
  const FIRE_WARRIOR_TILE = { x: 2646, y: 9867, z: 0 };
  // Inside the encounter room, south of the door. The former north-side tile
  // carried the player through the very gate the warrior was meant to protect.
  const FIRE_WARRIOR_KNOCKBACK_TILE = { x: 2646, y: 9868, z: 0 };
  const PUSH_WALL_ID = 1597;
  const PUSH_WALL_TILE = { x: 2643, y: 9892, z: 0 };
  const ICE_CHEST_CLOSED_ID = 103;
  const ICE_CHEST_OPEN_ID = 104;
  const ICE_CHEST_TILES = [
    [2710, 9850], [2719, 9838], [2729, 9850], [2747, 9848], [2738, 9835], [2745, 9821],
  ];
  /** West side of the lava, in the skeleton corridor, per Winelda's teleport. */
  const WINELDA_LANDING_TILE = { x: 2644, y: 9890, z: 0 };

  const START_HOOK = "quest:temple-of-ikov:start";
  /** Winelda's root hand-in (UB9hSI) and her repeat teleport (7xhwuW). */
  const WINELDA_ROOT_TELEPORT_STEP_ID = "UB9hSI";
  const WINELDA_REPEAT_TELEPORT_STEP_ID = "7xhwuW";
  /** The Fire Warrior's fire bolt knocks the player back from the door. */
  const FIRE_WARRIOR_KNOCKBACK_STEP_ID = "i4DijU";
  /** The guardian attacks when Lucien's follower takes the staff. */
  const TAKE_STAFF_ACTION_ID = "zJVI_k";
  /** Lucien's two quest-complete endings. */
  const LUCIEN_STAFF_COMPLETE_STEP_ID = "f8loKo";
  const LUCIEN_DEFEAT_COMPLETE_STEP_ID = "rgR_39";
  const COMPLETE_ACTION_IDS = new Set([LUCIEN_STAFF_COMPLETE_STEP_ID, LUCIEN_DEFEAT_COMPLETE_STEP_ID]);
  /** Guardian "attacks the player" stage directions: the player backed Lucien. */
  const LUCIEN_LOYALTY_STEP_IDS = new Set(["Z00rUz", "GUJaTp", "yrI1iq", "yPcDYb", "yW7QWI", "KcVBoC", "ir-ZYZ", TAKE_STAFF_ACTION_ID]);
  const GUARDIAN_PENDANT_STEP_ID = "UvrBfr";
  const LUCIEN_STAFF_HANDOVER_STEP_ID = "VBn1hb";
  const LOST_LUCIEN_PENDANT_STEP_ID = "BH3ct0";

  const LEGACY_SIDE_ATTRIBUTE = "quest.temple_of_ikov.side";
  const SIDE_ATTRIBUTE = "temple-of-ikov:side";
  const STATE_VERSION_ATTRIBUTE = "temple-of-ikov:state-version";
  const WINELDA_MET_ATTRIBUTE = "temple-of-ikov:met-winelda";
  const WINELDA_PAID_ATTRIBUTE = "temple-of-ikov:paid-winelda";
  const WARRIOR_KILLED_ATTRIBUTE = "temple-of-ikov:warrior-killed";
  const TRAP_DISARMED_ATTRIBUTE = "temple-of-ikov:trap-disarmed";
  const TRAP_PULLED_ATTRIBUTE = "temple-of-ikov:trap-pulled";
  const STAFF_DELIVERED_ATTRIBUTE = "temple-of-ikov:staff-delivered";
  const ICE_GATE_UNLOCKED_ATTRIBUTE = "temple-of-ikov:ice-gate-unlocked";
  const LUCIEN_DEFEATED_ATTRIBUTE = "temple-of-ikov:lucien-defeated";
  const STATE_VERSION = 1;
  const SIDE_NONE = 0;
  const SIDE_GUARDIANS = 1;
  const SIDE_LUCIEN = 2;

  let quest;
  /** Private owner-only NPCs, one set per player, never world-visible. */
  const lucienSpawnsByPlayer = new WeakMap();
  const fireWarriorByPlayer = new WeakMap();
  const fireOpeningByPlayer = new WeakMap();
  const defeatedLucienByPlayer = new WeakMap();
  const guardianClearedByPlayer = new WeakSet();
  const staffTakeByPlayer = new WeakMap();
  const bracketByPlayer = new WeakMap();
  const iceChestsByPlayer = new WeakMap();
  const trapTasksByPlayer = new WeakMap();
  const STAFF_TILE = { x: 2638, y: 9906, z: 0 };
  const LUCIEN_PRIVATE_NPC_IDS = new Set([INN_LUCIEN_NPC_ID, HOUSE_LUCIEN_NPC_ID]);
  const FIRE_WARRIOR_PRIVATE_NPC_IDS = new Set([FIRE_WARRIOR_NPC_ID]);

  const held = (player, itemId, amount = 1) =>
    player.getInventory().getAmount(itemId) >= amount;
  const limpwurtCount = (player) => player.getInventory().getAmount(LIMPWURT_ROOT_ITEM_ID);
  const thievingLevel = (player) => player.getSkillManager().getCurrentLevel(Skill.THIEVING);
  function sideOf(player) {
    const canonical = Number(player.getAttribute(SIDE_ATTRIBUTE));
    if (canonical === SIDE_GUARDIANS || canonical === SIDE_LUCIEN) return canonical;
    // Old saves acquire the new attribute's default0 before the quest login
    // migration. Only that unmigrated format may consult the old side; after
    // migration an explicit canonical0/invalid value must not resurrect it.
    if (player.getAttribute(STATE_VERSION_ATTRIBUTE) !== STATE_VERSION) {
      const legacy = Number(player.getAttribute(LEGACY_SIDE_ATTRIBUTE));
      if (legacy === SIDE_GUARDIANS || legacy === SIDE_LUCIEN) return legacy;
    }
    return SIDE_NONE;
  }
  const hasFlag = (player, key) => player.getAttribute(key) === 1;
  /** The old stage numbers record payment before combat; keep their saved meaning. */
  function progressAt(player, stage = quest.getStage(player)) {
    const legacy = !hasFlag(player, STATE_VERSION_ATTRIBUTE);
    return {
      paid: legacy ? stage >= STAGE_CROSSED_LAVA : hasFlag(player, WINELDA_PAID_ATTRIBUTE),
      killed: legacy ? stage >= STAGE_FIRE_WARRIOR_KILLED : hasFlag(player, WARRIOR_KILLED_ATTRIBUTE),
      met: legacy ? stage >= STAGE_CROSSED_LAVA : hasFlag(player, WINELDA_MET_ATTRIBUTE),
    };
  }
  function migrateProgress(player) {
    if (hasFlag(player, STATE_VERSION_ATTRIBUTE)) return;
    const old = progressAt(player);
    player.setAttribute(WINELDA_PAID_ATTRIBUTE, old.paid ? 1 : 0);
    player.setAttribute(WARRIOR_KILLED_ATTRIBUTE, old.killed ? 1 : 0);
    player.setAttribute(WINELDA_MET_ATTRIBUTE, old.met ? 1 : 0);
    player.setAttribute(SIDE_ATTRIBUTE, sideOf(player));
    player.setAttribute(LEGACY_SIDE_ATTRIBUTE, SIDE_NONE);
    player.setAttribute(STATE_VERSION_ATTRIBUTE, STATE_VERSION);
  }
  /** Publish chronological native progress without demoting the legacy saved stage. */
  function nativeStage(stage, player) {
    if (stage >= STAGE_COMPLETE || stage < STAGE_STARTED) return stage;
    const progress = progressAt(player, stage);
    if (sideOf(player) === SIDE_GUARDIANS) return 70;
    if (progress.killed) return progress.paid ? 60 : progress.met ? 50 : 40;
    if (hasFlag(player, TRAP_PULLED_ATTRIBUTE)) return 30;
    if (hasFlag(player, TRAP_DISARMED_ATTRIBUTE)) return 20;
    return 10;
  }
  const wearingPendant = (player, itemId) =>
    (player.getEquipment().getItems() || []).some(
      (item) => item && item.getId() === itemId && item.getAmount() > 0
    );
  const wearingArmadylPendant = (player) => wearingPendant(player, ARMADYL_PENDANT_ITEM_ID);
  function ownsItem(player, itemId, exceptGroundItem = null) {
    return held(player, itemId) || wearingPendant(player, itemId)
      || api.core.Bank.contains(player, new Item(itemId, 1))
      || api.core.World.getItems().some(item => item !== exceptGroundItem && !item.isPendingRemoval()
        && item.getOwner() === player.getUsername() && item.getItem().getId() === itemId
        && item.getItem().getAmount() > 0);
  }

  function actualSpeaker(event, ids, range = 3) {
    const { player, npc, npcId } = event;
    return Boolean(player && npc && ids.has(npcId) && npc.getContentId(player) === npcId
      && player.isRegistered() && npc.isRegistered() && npc.isVisible()
      && (!npc.getOwner() || npc.getOwner() === player)
      && npc.getPrivateArea() === player.getPrivateArea()
      && player.getLocation().isWithinDistance(npc.getLocation(), range)
      && player.getLocalNpcs().includes(npc));
  }

  function sameTile(location, tile) {
    return (
      Boolean(location) &&
      location.x === tile.x &&
      location.y === tile.y &&
      (location.z ?? 0) === tile.z
    );
  }

  function constructLocation(tile) {
    return new api.core.Location(tile.x, tile.y, tile.z);
  }

  function buildJournal(player, questHandle) {
    const stage = questHandle.getStage(player);
    if (stage >= STAGE_COMPLETE) {
      return [
        "<str>I recovered the Staff of Armadyl.</str>",
        "<str>I decided the fate of Lucien and the guardians.</str>",
        "",
        "<col=ff0000>QUEST COMPLETE!</col>",
      ];
    }
    if (sideOf(player) === SIDE_GUARDIANS) {
      return [
        "I refused to steal the Staff of Armadyl.",
        "I must defeat <col=800000>Lucien</col> while wearing the",
        "Armadyl pendant.",
      ];
    }
    if (hasFlag(player, STAFF_DELIVERED_ATTRIBUTE)) {
      return ["I gave Lucien the Staff of Armadyl.", "I should finish speaking to him at his house."];
    }
    if (held(player, STAFF_OF_ARMADYL_ITEM_ID)) {
      return [
        "I have taken the Staff of Armadyl.",
        "I should bring it to <col=800000>Lucien</col> at his house.",
      ];
    }
    const progress = progressAt(player, stage);
    if (progress.paid && progress.killed) {
      return ["Winelda has helped me cross the lava.", "I can follow the corridor and push the wall to reach the guardians."];
    }
    if (progress.killed) {
      return [
        "I defeated the Fire Warrior of Lesarkus.",
        "I should speak to Winelda and bring her 20 limpwurt roots.",
      ];
    }
    if (stage >= STAGE_STARTED) {
      return [
        "Lucien gave me his pendant.",
        "I should explore the temple, repair the lever and find ice arrows.",
      ];
    }
    return [
      "I can start this quest by speaking to <col=800000>Lucien</col>",
      "at the Flying Horse Inn in East Ardougne.",
      "It requires 42 Thieving.",
    ];
  }

  function grantReward(player, questHandle, options = {}) {
    removeSpawns(player);
    if (options.experience === false) return;
    player.getSkillManager().addExperiences(Skill.RANGED, 10500);
    player.getSkillManager().addExperiences(Skill.FLETCHING, 8000);
  }

  /** Which transcript variant each speaker plays, by quest stage and side. */
  function selectVariant({ npcId, player }) {
    if (!LUCIEN_NPC_IDS.has(npcId) && !WINELDA_NPC_IDS.has(npcId)
      && !GUARDIAN_NPC_IDS.has(npcId) && npcId !== FIRE_WARRIOR_NPC_ID) return null;
    migrateProgress(player);
    const stage = quest.getStage(player);
    if (LUCIEN_NPC_IDS.has(npcId)) {
      if (npcId === HOUSE_LUCIEN_NPC_ID || npcId === 3444) {
        return stage >= STAGE_HAS_STAFF
          ? "finishing-up-talking-to-lucien-near-varrock"
          : "starting-out-talking-to-lucien-again";
      }
      if (stage === 0) return "starting-out-talking-to-lucien";
      if (stage < STAGE_COMPLETE) return "starting-out-talking-to-lucien-again";
      return "finishing-up-talking-to-lucien-near-varrock";
    }
    if (npcId === WINELDA_NPC_ID) {
      const progress = progressAt(player);
      if (progress.paid) return "inside-the-dungeon-talking-to-winelda-after-giving-the-limpwurts";
      if (progress.met || limpwurtCount(player) >= LIMPWURT_REQUIRED) return "inside-the-dungeon-talking-to-winelda-again";
      return "inside-the-dungeon-talking-to-winelda";
    }
    if (GUARDIAN_NPC_IDS.has(npcId)) {
      if (held(player, STAFF_OF_ARMADYL_ITEM_ID)) {
        return "inside-the-dungeon-talking-to-a-guardian-of-armadyl-while-carrying-the-staff-of-armadyl";
      }
      const side = sideOf(player);
      if (side === SIDE_GUARDIANS) {
        return "inside-the-dungeon-talking-to-the-guardian-of-armadyl-after-agreeing-to-kill-lucien";
      }
      return wearingPendant(player, PENDANT_OF_LUCIEN_ITEM_ID)
        ? "inside-the-dungeon-talking-to-a-guardian-of-armadyl-while-wearing-the-pendant-of-lucien"
        : "inside-the-dungeon-talking-to-a-guardian-of-armadyl-while-not-wearing-the-pendant-of-lucien";
    }
    if (npcId === FIRE_WARRIOR_NPC_ID) {
      return "inside-the-dungeon-talking-to-the-fire-warrior-of-lesarkus";
    }
    return null;
  }

  /** Answer the page's prose conditions. */
  function answerCondition({ player, npcId, text }) {
    if (!LUCIEN_NPC_IDS.has(npcId) && !GUARDIAN_NPC_IDS.has(npcId) && npcId !== WINELDA_NPC_ID && npcId !== FIRE_WARRIOR_NPC_ID) return null;
    const value = String(text).toLowerCase();
    if (value.includes("inventory is full")) return player.getInventory().isFull();
    if (value.includes("lost the pendant")) return !ownsItem(player, PENDANT_OF_LUCIEN_ITEM_ID);
    if (value.includes("still has the pendant")) return ownsItem(player, PENDANT_OF_LUCIEN_ITEM_ID);
    if (value.includes("with 42 thieving")) return thievingLevel(player) >= 42;
    if (value.includes("without 42 thieving")) return thievingLevel(player) < 42;
    if (value.includes("no limpwurt roots")) return limpwurtCount(player) === 0;
    if (value.includes("1-19 limpwurt roots")) {
      const count = limpwurtCount(player);
      return count >= 1 && count <= 19;
    }
    if (value.includes("20 limpwurt roots")) return limpwurtCount(player) >= LIMPWURT_REQUIRED;
    if (value.includes("lost the armadyl pendant")) return !ownsItem(player, ARMADYL_PENDANT_ITEM_ID);
    if (value.includes("still has the armadyl pendant")) return ownsItem(player, ARMADYL_PENDANT_ITEM_ID);
    const readyToHandIn = held(player, STAFF_OF_ARMADYL_ITEM_ID) || hasFlag(player, STAFF_DELIVERED_ATTRIBUTE);
    if (value.includes("does not have the staff")) return !readyToHandIn;
    if (value.includes("has the staff")) return readyToHandIn;
    return null;
  }

  // ==========================================================================
  // Private Lucien / Fire Warrior spawns
  // ==========================================================================

  function spawnPrivateNpc(player, id, tile) {
    const npc = api.spawnNpc({
      id,
      x: tile.x,
      y: tile.y,
      z: tile.z,
      wanderRadius: 0,
      owner: player,
      ownerOnly: true,
    });
    // Without this the death task queues the definition's own respawn.
    if (npc) npc.__skipDefaultRespawn = true;
    return npc;
  }

  /**
   * A missed logout leaves a player's private NPCs in the world while the WeakMap
   * forgets them (it is keyed by the discarded player object). Clear any leftover
   * owner NPCs for this player before spawning a fresh set, so logins cannot pile
   * copies on one tile. Ownership is matched by object first, then by username for
   * spawns from an earlier session of the same account.
   */
  function removeStalePrivateSpawns(player, ids) {
    const stale = [];
    api.core.World.getNpcs().forEach((npc) => {
      if (!ids.has(npc.getId()) || !npc.isOwnerOnly?.()) return;
      const owner = npc.getOwner?.();
      if (owner === player || (owner != null && owner.getUsername?.() === player.getUsername())) {
        stale.push(npc);
      }
    });
    for (const npc of stale) api.removeNpc(npc);
  }

  /**
   * The inn Lucien exists from login while the quest is open; his house double
   * (the attackable one) exists once the quest has started.
   */
  function ensureLucien(player) {
    if (quest.isComplete(player)) {
      removeLucien(player);
      return;
    }
    let entry = lucienSpawnsByPlayer.get(player);
    if (!entry) {
      removeStalePrivateSpawns(player, LUCIEN_PRIVATE_NPC_IDS);
      entry = { inn: null, house: null };
      lucienSpawnsByPlayer.set(player, entry);
    }
    if (!entry.inn) entry.inn = spawnPrivateNpc(player, INN_LUCIEN_NPC_ID, INN_LUCIEN_TILE);
    if (quest.getStage(player) >= STAGE_STARTED && !entry.house) {
      entry.house = spawnPrivateNpc(player, HOUSE_LUCIEN_NPC_ID, HOUSE_LUCIEN_TILE);
    }
  }

  function removeLucien(player) {
    const entry = lucienSpawnsByPlayer.get(player);
    if (!entry) return;
    lucienSpawnsByPlayer.delete(player);
    if (entry.inn) api.removeNpc(entry.inn);
    if (entry.house) api.removeNpc(entry.house);
  }

  function ensureFireWarrior(player) {
    const existing = fireWarriorByPlayer.get(player);
    if (existing) return existing;
    removeStalePrivateSpawns(player, FIRE_WARRIOR_PRIVATE_NPC_IDS);
    const npc = spawnPrivateNpc(player, FIRE_WARRIOR_NPC_ID, FIRE_WARRIOR_TILE);
    if (npc) fireWarriorByPlayer.set(player, npc);
    return npc;
  }

  function removeFireWarrior(player) {
    fireOpeningByPlayer.get(player)?.stop();
    const npc = fireWarriorByPlayer.get(player);
    if (!npc) return;
    fireWarriorByPlayer.delete(player);
    api.removeNpc(npc);
  }

  function removeSpawns(player) {
    removeLucien(player);
    removeFireWarrior(player);
  }

  // ==========================================================================
  // Dialogue hooks
  // ==========================================================================

  /** Wiki: 42 Thieving and 40 Ranged. */
  const REQUIRED_SKILLS = [
    { skill: Skill.THIEVING, level: 42, label: "Thieving" },
    { skill: Skill.RANGED, level: 40, label: "Ranged" },
  ];

  function missingSkillRequirements(player) {
    const skills = player.getSkillManager();
    return REQUIRED_SKILLS.filter((requirement) => skills.getMaxLevel(requirement.skill) < requirement.level);
  }

  function notifySkillRequirements(player) {
    for (const requirement of missingSkillRequirements(player)) {
      player.sendMessage(
        `You need a ${requirement.label} level of at least ${requirement.level} to start this quest.`
      );
    }
  }

  function handleStartHook(event) {
    const { player, npcId, hook } = event;
    if (!LUCIEN_NPC_IDS.has(npcId) || hook !== START_HOOK) return;
    if (!actualSpeaker(event, LUCIEN_NPC_IDS) || lucienSpawnsByPlayer.get(player)?.inn !== event.npc) {
      refuseSourceAction(event, "You can no longer accept that Lucien's mission.");
      return;
    }
    migrateProgress(player);
    if (quest.getStage(player) < STAGE_STARTED) {
      if (missingSkillRequirements(player).length > 0) {
        notifySkillRequirements(player);
        event.end = true;
        return;
      }
      if (!ownsItem(player, PENDANT_OF_LUCIEN_ITEM_ID)
        && player.getInventory().getAddableAmount(new Item(PENDANT_OF_LUCIEN_ITEM_ID, 1)) < 1) {
        refuseSourceAction(event, "You need a free inventory slot to take Lucien's pendant.");
        return;
      }
      quest.setStage(player, STAGE_STARTED);
      if (!ownsItem(player, PENDANT_OF_LUCIEN_ITEM_ID)) {
        giveReward(player, PENDANT_OF_LUCIEN_ITEM_ID, 1);
      }
      ensureLucien(player);
    }
  }

  function takeLimpwurts(player) {
    let remaining = LIMPWURT_REQUIRED;
    while (remaining > 0 && player.getInventory().getAmount(LIMPWURT_ROOT_ITEM_ID) > 0) {
      player.getInventory().deleteNumber(LIMPWURT_ROOT_ITEM_ID, 1);
      remaining--;
    }
  }

  function refuseSourceAction(event, text) {
    event.handled = true;
    event.end = true;
    event.text = text;
    event.player.sendMessage(text);
  }

  function handleAction(event) {
    const { player, npcId, stepId } = event;
    if (npcId === WINELDA_NPC_ID) {
      if (!actualSpeaker(event, WINELDA_NPC_IDS)) return;
      migrateProgress(player);
      if (stepId === WINELDA_ROOT_TELEPORT_STEP_ID) {
        if (quest.getStage(player) < STAGE_STARTED || quest.isComplete(player)) return;
        const progress = progressAt(player);
        if (progress.paid || !progress.killed) return;
        if (limpwurtCount(player) < LIMPWURT_REQUIRED) return;
        takeLimpwurts(player);
        player.setAttribute(WINELDA_PAID_ATTRIBUTE, 1);
        player.setAttribute(WINELDA_MET_ATTRIBUTE, 1);
        quest.setStage(player, Math.max(quest.getStage(player), STAGE_CROSSED_LAVA));
        player.moveTo(constructLocation(WINELDA_LANDING_TILE));
        return;
      }
      if (stepId === WINELDA_REPEAT_TELEPORT_STEP_ID) {
        if (!progressAt(player).paid) return;
        player.moveTo(constructLocation(WINELDA_LANDING_TILE));
        return;
      }
    }
    if (LUCIEN_NPC_IDS.has(npcId) && stepId === LOST_LUCIEN_PENDANT_STEP_ID) {
      if (!actualSpeaker(event, LUCIEN_NPC_IDS) || lucienSpawnsByPlayer.get(player)?.inn !== event.npc
        || quest.getStage(player) < STAGE_STARTED || quest.isComplete(player)) {
        refuseSourceAction(event, "You can no longer ask that Lucien for a pendant.");
        return;
      }
      if (ownsItem(player, PENDANT_OF_LUCIEN_ITEM_ID)) {
        refuseSourceAction(event, "You already have Lucien's pendant.");
        return;
      }
      // This replacement message explicitly drops the pendant at the player's
      // feet when the bag has filled since the dialogue began.
      giveReward(player, PENDANT_OF_LUCIEN_ITEM_ID, 1);
      return;
    }
    if (GUARDIAN_NPC_IDS.has(npcId) && stepId === GUARDIAN_PENDANT_STEP_ID) {
      if (!actualSpeaker(event, GUARDIAN_NPC_IDS)) {
        refuseSourceAction(event, "You can no longer speak to that guardian.");
        return;
      }
      const progress = progressAt(player);
      if (quest.getStage(player) < STAGE_STARTED || quest.isComplete(player)
        || !progress.paid || !progress.killed || wearingPendant(player, PENDANT_OF_LUCIEN_ITEM_ID)) {
        refuseSourceAction(event, "The guardian cannot give you a pendant now.");
        return;
      }
      if (sideOf(player) === SIDE_GUARDIANS) {
        event.handled = true;
        player.sendMessage("The guardian has already given you a pendant.");
        return;
      }
      // The source explicitly drops this pendant at the player's feet on a full bag.
      giveReward(player, ARMADYL_PENDANT_ITEM_ID, 1);
      player.setAttribute(SIDE_ATTRIBUTE, SIDE_GUARDIANS);
      quest.setStage(player, Math.max(quest.getStage(player), STAGE_HAS_STAFF));
      return;
    }
    if (LUCIEN_NPC_IDS.has(npcId) && stepId === LUCIEN_STAFF_HANDOVER_STEP_ID) {
      if (!actualSpeaker(event, LUCIEN_NPC_IDS) || lucienSpawnsByPlayer.get(player)?.house !== event.npc) {
        refuseSourceAction(event, "You can no longer give the staff to that Lucien.");
        return;
      }
      if (quest.getStage(player) < STAGE_STARTED || quest.isComplete(player) || sideOf(player) === SIDE_GUARDIANS) {
        refuseSourceAction(event, "You cannot give Lucien the staff now.");
        return;
      }
      if (hasFlag(player, STAFF_DELIVERED_ATTRIBUTE)) {
        event.handled = true;
        player.sendMessage("You have already given Lucien the Staff of Armadyl.");
        return;
      }
      if (!held(player, STAFF_OF_ARMADYL_ITEM_ID)) {
        refuseSourceAction(event, "You no longer have the Staff of Armadyl.");
        return;
      }
      player.getInventory().deleteNumber(STAFF_OF_ARMADYL_ITEM_ID, 1);
      player.setAttribute(STAFF_DELIVERED_ATTRIBUTE, 1);
      return;
    }
    if (npcId === FIRE_WARRIOR_NPC_ID && stepId === FIRE_WARRIOR_KNOCKBACK_STEP_ID) {
      if (fireWarriorByPlayer.get(player) !== event.npc || !actualSpeaker(event, FIRE_WARRIOR_PRIVATE_NPC_IDS, 16)) return;
      const cast = api.core.CacheDefinitions.getSeqId("human_caststrike");
      if (cast != null) event.npc.performAnimation(new api.core.Animation(cast));
      player.moveTo(constructLocation(FIRE_WARRIOR_KNOCKBACK_TILE));
      return;
    }
    if (GUARDIAN_NPC_IDS.has(npcId) && LUCIEN_LOYALTY_STEP_IDS.has(stepId)) {
      if (!actualSpeaker(event, GUARDIAN_NPC_IDS, 16) || quest.getStage(player) < STAGE_STARTED || quest.isComplete(player)) return;
      player.setAttribute(SIDE_ATTRIBUTE, SIDE_LUCIEN);
      event.npc.getCombat().attack(player);
      return;
    }
    if (LUCIEN_NPC_IDS.has(npcId) && COMPLETE_ACTION_IDS.has(stepId)) {
      if (!quest.isComplete(player)) {
        if (stepId === LUCIEN_STAFF_COMPLETE_STEP_ID && (!hasFlag(player, STAFF_DELIVERED_ATTRIBUTE)
          || !actualSpeaker(event, LUCIEN_NPC_IDS) || lucienSpawnsByPlayer.get(player)?.house !== event.npc)) return;
        if (stepId === LUCIEN_DEFEAT_COMPLETE_STEP_ID && (!hasFlag(player, LUCIEN_DEFEATED_ATTRIBUTE)
          || (defeatedLucienByPlayer.get(player) !== event.npc
            && (!actualSpeaker(event, LUCIEN_NPC_IDS) || lucienSpawnsByPlayer.get(player)?.house !== event.npc)))) return;
        quest.complete(player);
      }
    }
  }

  function inGuardianRoom(location) {
    return location.getZ() === 0 && location.getX() >= 2633 && location.getX() <= 2651
      && location.getY() >= 9893 && location.getY() <= 9915;
  }

  function handleGuardianDeath({ killer, npc, npcId }) {
    if (!killer || !npc || !GUARDIAN_NPC_IDS.has(npcId) || !npc.isRegistered() || npc.getHitpoints() > 0
      || api.core.World.getNpcs().get(npc.getIndex()) !== npc
      || (npc.getOwner() && npc.getOwner() !== killer)
      || npc.getPrivateArea() !== killer.getPrivateArea() || !inGuardianRoom(npc.getLocation())) return;
    migrateProgress(killer);
    if (quest.getStage(killer) < STAGE_STARTED || quest.isComplete(killer)
      || !progressAt(killer).paid || sideOf(killer) === SIDE_GUARDIANS) return;
    // The actual killer of a room guardian can return to the table for the staff.
    // This encounter receipt is session-scoped; it is consumed only by a successful pickup.
    guardianClearedByPlayer.add(killer);
  }

  function nearestGuardian(player) {
    return player.getLocalNpcs().filter(npc => GUARDIAN_NPC_IDS.has(npc.getContentId(player))
      && npc.isRegistered() && npc.isVisible() && npc.getHitpoints() > 0 && inGuardianRoom(npc.getLocation())
      && (!npc.getOwner() || npc.getOwner() === player) && npc.getPrivateArea() === player.getPrivateArea())
      .sort((a, b) => player.getLocation().getDistance(a.getLocation()) - player.getLocation().getDistance(b.getLocation())
        || a.getIndex() - b.getIndex())[0];
  }

  function refuseStaffTake(event, text) {
    event.allow = false;
    event.player.sendMessage(text);
  }

  function handleStaffTake(event) {
    const { player, groundItem, itemId, source } = event;
    if (itemId !== STAFF_OF_ARMADYL_ITEM_ID || event.allow !== true) return;
    if (!player.isRegistered() || !api.core.World.getItems().includes(groundItem)) {
      event.allow = false;
      return;
    }
    migrateProgress(player);
    if (quest.isComplete(player)) {
      refuseStaffTake(event, "You have completed the quest of the Temple of Ikov.");
      return;
    }
    if (source === "telegrab") {
      refuseStaffTake(event, "You cannot use Telekinetic Grab on the Staff of Armadyl.");
      return;
    }
    const progress = progressAt(player);
    if (quest.getStage(player) < STAGE_STARTED || !progress.paid || !progress.killed) {
      refuseStaffTake(event, "You are not ready to take the Staff of Armadyl.");
      return;
    }
    if (ownsItem(player, STAFF_OF_ARMADYL_ITEM_ID, groundItem) || hasFlag(player, STAFF_DELIVERED_ATTRIBUTE)) {
      refuseStaffTake(event, "You already have the Staff of Armadyl.");
      return;
    }
    if (sideOf(player) === SIDE_GUARDIANS) {
      event.allow = false;
      startTranscript(api, player, NpcIdentifiers.GUARDIAN_OF_ARMADYL, PAGE,
        "inside-the-dungeon-taking-the-staff-of-armadyl-after-agreeing-to-kill-lucien");
      return;
    }
    if (groundItem.isStaticSpawn() && groundItem.getPosition().equals(constructLocation(STAFF_TILE))
      && !guardianClearedByPlayer.has(player)) {
      event.allow = false;
      const guardian = nearestGuardian(player);
      if (!guardian) {
        player.sendMessage("The guardians are still protecting the staff.");
        return;
      }
      startTranscript(api, player, guardian.getContentId(player), PAGE,
        "inside-the-dungeon-taking-the-staff-of-armadyl-while-helping-lucien", undefined, guardian);
      return;
    }
    staffTakeByPlayer.set(player, groundItem);
  }

  function handleStaffTaken({ player, groundItem, itemId, source, amount }) {
    if (itemId !== STAFF_OF_ARMADYL_ITEM_ID || source !== "pickup" || amount !== 1
      || staffTakeByPlayer.get(player) !== groundItem || !held(player, STAFF_OF_ARMADYL_ITEM_ID)) return;
    staffTakeByPlayer.delete(player);
    guardianClearedByPlayer.delete(player);
    player.setAttribute(SIDE_ATTRIBUTE, SIDE_LUCIEN);
    quest.setStage(player, Math.max(quest.getStage(player), STAGE_HAS_STAFF));
  }

  function canTelegrabStaff(event) {
    if (event.groundItem?.getItem().getId() !== STAFF_OF_ARMADYL_ITEM_ID) return;
    event.allow = false;
    event.player.sendMessage("You cannot use Telekinetic Grab on the Staff of Armadyl.");
  }

  /** Killing the Fire Warrior advances the quest once the lava is crossed. */
  function handleFireWarriorDeath({ killer, npc, npcId }) {
    if (!killer || npcId !== FIRE_WARRIOR_NPC_ID || fireWarriorByPlayer.get(killer) !== npc || npc.getOwner() !== killer) return;
    fireWarriorByPlayer.delete(killer);
    migrateProgress(killer);
    if (quest.getStage(killer) >= STAGE_STARTED && !quest.isComplete(killer)) {
      killer.setAttribute(WARRIOR_KILLED_ATTRIBUTE, 1);
      quest.setStage(killer, Math.max(quest.getStage(killer), STAGE_FIRE_WARRIOR_KILLED));
    }
  }

  /** Killing the house Lucien wearing the Armadyl pendant is the good ending. */
  function handleLucienDeath({ killer, npc, npcId }) {
    if (!killer || npcId !== HOUSE_LUCIEN_NPC_ID) return;
    const entry = lucienSpawnsByPlayer.get(killer);
    if (entry?.house !== npc || npc.getOwner() !== killer) return;
    if (entry?.house) {
      api.removeNpc(entry.house);
      entry.house = null;
    }
    if (quest.isComplete(killer) || sideOf(killer) !== SIDE_GUARDIANS) return;
    if (!wearingArmadylPendant(killer)) return;
    killer.setAttribute(LUCIEN_DEFEATED_ATTRIBUTE, 1);
    defeatedLucienByPlayer.set(killer, npc);
    startTranscript(api, killer, HOUSE_LUCIEN_NPC_ID, PAGE, "finishing-up-defeating-lucien", undefined, npc);
  }

  function canLaunchFireWarriorAttack(player, method) {
    const { Ammunition, CombatType, Equipment, RangedWeapon, WeaponInterfaces } = api.core;
    const arrows = player.getEquipment().get(Equipment.AMMUNITION_SLOT);
    if (method?.type?.() !== CombatType.RANGED || arrows?.getId?.() !== ICE_ARROWS_ITEM_ID || arrows.getAmount() < 1) return false;
    const weapon = player.getWeapon();
    if (weapon && [WeaponInterfaces.DART, WeaponInterfaces.KNIFE, WeaponInterfaces.THROWNAXE,
      WeaponInterfaces.OBBY_RINGS, WeaponInterfaces.BLOWPIPE].filter(Boolean).includes(weapon)) return true;
    return Ammunition.firesFromAmmoSlot(player) &&
      RangedWeapon.getFor(player)?.getAmmunitionData()?.some(ammo => ammo.getItemId() === ICE_ARROWS_ITEM_ID) === true;
  }

  function handleCanAttack(event) {
    const { attacker, target, method } = event;
    if (!attacker?.getEquipment || !target) return;
    const targetId = typeof target.getId === "function" ? target.getId() : undefined;
    if (targetId === FIRE_WARRIOR_NPC_ID && attacker.isPlayer?.() && method) {
      // Judge ordinary attacks before launch; previously queued and reflected damage is unchanged.
      if (!canLaunchFireWarriorAttack(attacker, method)) event.allow = false;
      return;
    }
    // Lucien instead requires the Armadyl pendant; his transcript explains a refusal.
    if (targetId !== HOUSE_LUCIEN_NPC_ID) return;
    if (wearingArmadylPendant(attacker)) return;
    event.allow = false;
    if (!attacker.getDialogueManager?.()?.isActive?.()) {
      startTranscript(
        api,
        attacker,
        HOUSE_LUCIEN_NPC_ID,
        PAGE,
        "finishing-up-attacking-lucien-without-wearing-the-armadyl-pendant"
      );
    }
  }

  // ==========================================================================
  // Interactions
  // ==========================================================================

  function replaceBracket(player, base, id) {
    const old = bracketByPlayer.get(player);
    if (old) api.core.ObjectManager.deregister(old, true);
    // An owner-only overlay keeps the cache's bracket and every other player's
    // state intact. Owner objects do not modify shared map clipping.
    const object = new api.core.GameObject(id, base.getLocation().clone(), base.getType(),
      base.getFace(), base.getPrivateArea()).setOwner(player);
    api.core.ObjectManager.register(object, true);
    bracketByPlayer.set(player, object);
  }

  function removeBracket(player) {
    const object = bracketByPlayer.get(player);
    if (!object) return;
    bracketByPlayer.delete(player);
    api.core.ObjectManager.deregister(object, true);
    const base = api.core.MapObjects.get(LEVER_BRACKET_ID, constructLocation(LEVER_BRACKET_TILE), player.getPrivateArea());
    if (base) player.getPacketSender().sendObject(base);
  }

  function guardBootsStairs(event) {
    if (event.objectId !== BOOTS_STAIRS_DOWN_ID
      || !sameTile(event.object?.getLocation(), BOOTS_STAIRS_DOWN_TILE)) return;
    if (!require("../../world/Darkness.plugin.js").hasLightSource(event.player)) {
      event.handled = true;
      event.player.getDialogueManager().startDialogues(new api.core.DialogueChainBuilder()
        .add(new api.core.StatementDialogue(0, "It's a bit dark down here! You decide not to venture far."))
        .add(new api.core.EndDialogue(1)));
    }
  }

  function climbOutOfPit(event) {
    if (event.objectId !== PIT_LADDER_ID || !sameTile(event.object?.getLocation(), PIT_LADDER_TILE)) return;
    event.handled = true;
    const exit = api.core.MapObjects.get(PIT_EXIT_ID, constructLocation(PIT_EXIT_TILE), event.player.getPrivateArea());
    const destination = exit && require("../../objects/ClimbLinks").landingTile(exit,
      { x: PIT_EXIT_TILE.x, y: PIT_EXIT_TILE.y - 1 }, event.player.getPrivateArea());
    if (destination) api.emitCustomEvent("ladders:climbUp", { player: event.player, object: event.object, destination });
  }

  function pullTrappedLever(player) {
    if (trapTasksByPlayer.has(player)) return;
    const movement = player.getMovementQueue();
    if (movement.isMovementBlocked()) return;
    const ladder = api.core.MapObjects.get(PIT_LADDER_ID, constructLocation(PIT_LADDER_TILE), player.getPrivateArea());
    // The cache provides the separate pit and its ladder. Resolve a walkable
    // landing beside it; this is not a claimed exact retail falling tile.
    const landing = ladder && require("../../objects/ClimbLinks").landingTile(ladder,
      { x: PIT_LADDER_TILE.x + 1, y: PIT_LADDER_TILE.y }, player.getPrivateArea());
    if (!hasFlag(player, TRAP_DISARMED_ATTRIBUTE) && !landing) return;
    const start = player.getLocation().clone(), area = player.getPrivateArea();
    const animation = api.core.CacheDefinitions.getSeqId("macro_lever_switch_down");
    if (animation != null) player.performAnimation(new api.core.Animation(animation));
    movement.setBlockMovement(true).reset();
    const task = new (class extends api.core.Task {
      constructor() { super(1, player); }
      execute() {
        if (player.isRegistered() && player.getHitpoints() > 0 && player.getLocation().equals(start)
          && player.getPrivateArea() === area) {
          if (hasFlag(player, TRAP_DISARMED_ATTRIBUTE)) {
            player.setAttribute(TRAP_PULLED_ATTRIBUTE, 1);
            quest.setStage(player, quest.getStage(player));
            player.sendMessage("You hear the clunking of some hidden machinery.");
          } else {
            player.moveTo(landing);
            player.getCombat().getHitQueue().addPendingDamage([new api.core.HitDamage(20, api.core.HitMask.RED)]);
            player.forceChat("Ouch!");
          }
        }
        this.stop();
      }
      stop() {
        if (trapTasksByPlayer.get(player) === this) {
          trapTasksByPlayer.delete(player);
          movement.setBlockMovement(false);
        }
        super.stop();
      }
    })();
    trapTasksByPlayer.set(player, task);
    api.getTaskManager().submit(task);
  }

  function guardDungeonDoors(event) {
    const { player, objectId, location } = event;
    if (!location || (location.z ?? 0) !== 0) return;
    if (ICE_GATE_IDS.has(objectId) && (location.x === 2661 || location.x === 2662)
      && (location.y === 9803 || location.y === 9802)) {
      if (quest.isComplete(player) || hasFlag(player, ICE_GATE_UNLOCKED_ATTRIBUTE)) return;
      event.handled = true;
      player.sendMessage("The gate is locked. Perhaps a lever opens it.");
      return;
    }
    if (objectId === TRAP_DOOR_ID && location.x === 2648 && (location.y === 9857 || location.y === 9858)) {
      if (quest.isComplete(player) || hasFlag(player, TRAP_PULLED_ATTRIBUTE) || progressAt(player).killed) return;
      event.handled = true;
      player.sendMessage("The door is locked.");
      return;
    }
    if (objectId === FIRE_WARRIOR_DOOR_ID && location.x === FIRE_WARRIOR_DOOR_TILE.x
      && (location.y === FIRE_WARRIOR_DOOR_TILE.y || location.y === FIRE_WARRIOR_DOOR_TILE.y + 1)) {
      if (quest.isComplete(player) || progressAt(player).killed) return;
      event.handled = true;
      if (quest.getStage(player) >= STAGE_STARTED) openFireWarriorDoor(player);
      else player.sendMessage("The door is locked.");
    }
  }

  function handleLeverOnBracket(event) {
    const { player, object, objectId, itemId, location } = event;
    if (objectId !== LEVER_BRACKET_ID || !sameTile(location, LEVER_BRACKET_TILE)
      || itemId !== ItemIdentifiers.LEVER) return;
    event.handled = true;
    if (!player.isRegistered() || !object || bracketByPlayer.has(player)
      || hasFlag(player, ICE_GATE_UNLOCKED_ATTRIBUTE) || !held(player, itemId)) return;
    if (api.core.MapObjects.getPrivateArea(player, objectId, object.getLocation()) !== object) return;
    player.getInventory().deleteNumber(itemId, 1);
    replaceBracket(player, object, FITTED_LEVER_ID);
    player.sendMessage("You fit the lever into the bracket.");
  }

  function npcAction(event) {
    const actions = event.definition?.getActions?.() ?? [];
    return String(actions[event.clickType - 1] ?? "").toLowerCase();
  }

  /** The private Luciens are not indexed, so their Talk-to is replayed here. */
  function handleNpcInteraction(event) {
    const { player, npcId } = event;
    const action = npcAction(event);
    if (npcId === WINELDA_NPC_ID && action === "talk-to") {
      if (!actualSpeaker(event, WINELDA_NPC_IDS)) return;
      const variant = selectVariant(event);
      event.handled = true;
      if (startTranscript(api, player, npcId, PAGE, variant, undefined, event.npc)) {
        player.setAttribute(WINELDA_MET_ATTRIBUTE, 1);
        quest.setStage(player, quest.getStage(player));
      }
      return;
    }
    if (npcId === INN_LUCIEN_NPC_ID) {
      if (action !== "talk-to") return;
      event.handled = true;
      const stage = quest.getStage(player);
      const variant =
        stage < STAGE_STARTED
          ? "starting-out-talking-to-lucien"
          : "starting-out-talking-to-lucien-again";
      startTranscript(api, player, npcId, PAGE, variant, undefined, event.npc);
      return;
    }
    if (npcId === HOUSE_LUCIEN_NPC_ID && action === "talk-to") {
      event.handled = true;
      const variant = hasFlag(player, LUCIEN_DEFEATED_ATTRIBUTE)
        ? "finishing-up-defeating-lucien" : "finishing-up-talking-to-lucien-near-varrock";
      startTranscript(api, player, npcId, PAGE, variant, undefined, event.npc);
    }
  }

  function iceChestIndex(location) {
    if ((location?.z ?? 0) !== 0) return -1;
    return ICE_CHEST_TILES.findIndex(([x, y]) => location.x === x && location.y === y);
  }

  function removeIceChests(player) {
    const state = iceChestsByPlayer.get(player);
    if (!state) return;
    iceChestsByPlayer.delete(player);
    for (const object of state.open.values()) {
      api.core.ObjectManager.deregister(object, true);
      const base = api.core.MapObjects.get(ICE_CHEST_CLOSED_ID, object.getLocation(), player.getPrivateArea());
      if (base) player.getPacketSender().sendObject(base);
    }
  }

  function handleIceChest(event, index) {
    const { player, object, objectId } = event;
    event.handled = true;
    let state = iceChestsByPlayer.get(player);
    if (!state) {
      state = { selected: api.core.Misc.random(0, ICE_CHEST_TILES.length - 1), amount: null, open: new Map() };
      iceChestsByPlayer.set(player, state);
    }
    const action = String(event.option ?? event.definition?.getInteractions?.()[event.clickType - 1]).toLowerCase();
    if (objectId === ICE_CHEST_CLOSED_ID && action === "open") {
      if (state.open.has(index)) return;
      const opened = new api.core.GameObject(ICE_CHEST_OPEN_ID, object.getLocation().clone(), object.getType(),
        object.getFace(), object.getPrivateArea()).setOwner(player);
      api.core.ObjectManager.register(opened, true);
      state.open.set(index, opened);
      return;
    }
    if (objectId !== ICE_CHEST_OPEN_ID || state.open.get(index) !== object) return;
    if (action === "shut") {
      state.open.delete(index);
      api.core.ObjectManager.deregister(object, true);
      const base = api.core.MapObjects.get(ICE_CHEST_CLOSED_ID, object.getLocation(), player.getPrivateArea());
      if (base) player.getPacketSender().sendObject(base);
      return;
    }
    if (action !== "search") return;
    if (index !== state.selected) {
      player.sendMessage("You search the chest, but find nothing.");
      return;
    }
    const amount = state.amount ?? (state.amount = api.core.Misc.random(1, 5));
    if (player.getInventory().getAddableAmount(new Item(ICE_ARROWS_ITEM_ID, amount)) < amount) {
      player.sendMessage("You do not have enough space for the arrows.");
      return;
    }
    player.getInventory().adds(ICE_ARROWS_ITEM_ID, amount);
    player.sendMessage("You found some ice arrows!");
    // The quest guide explicitly permits searching this same open chest again:
    // it can be selected on consecutive successful searches. Opening is visual
    // state; it does not consume or reroll the one containing stock.
    state.selected = api.core.Misc.random(0, ICE_CHEST_TILES.length - 1);
    state.amount = null;
  }

  function openFireWarriorDoor(player) {
    if (quest.getStage(player) < STAGE_STARTED || quest.isComplete(player)) return;
    migrateProgress(player);
    if (progressAt(player).killed || fireOpeningByPlayer.has(player)) return;
    const npc = ensureFireWarrior(player);
    if (!npc) return;
    const start = player.getLocation().clone(), area = player.getPrivateArea();
    const task = new (class extends api.core.Task {
      constructor() { super(1, player); this.waited = 0; }
      execute() {
        if (!player.isRegistered() || player.getHitpoints() <= 0 || !player.getLocation().equals(start)
          || player.getPrivateArea() !== area || fireWarriorByPlayer.get(player) !== npc || progressAt(player).killed) {
          this.stop();
          return;
        }
        // The NPC is queued during the object op. Wait for the ordinary World
        // registration/local-list update so overhead cannot fall back to chat.
        if (!actualSpeaker({ player, npc, npcId: FIRE_WARRIOR_NPC_ID }, FIRE_WARRIOR_PRIVATE_NPC_IDS, 16)) {
          if (++this.waited >= 8) removeFireWarrior(player);
          return;
        }
        this.stop();
        startTranscript(api, player, FIRE_WARRIOR_NPC_ID, PAGE,
          "inside-the-dungeon-opening-door-in-the-fire-warrior-room", undefined, npc);
      }
      stop() {
        if (fireOpeningByPlayer.get(player) === this) fireOpeningByPlayer.delete(player);
        super.stop();
      }
    })();
    fireOpeningByPlayer.set(player, task);
    api.getTaskManager().submit(task);
  }

  function handleObjectInteraction(event) {
    const { player, objectId, location } = event;
    if (objectId === FITTED_LEVER_ID && sameTile(location, LEVER_BRACKET_TILE)) {
      event.handled = true;
      if (bracketByPlayer.get(player) !== event.object) return;
      player.setAttribute(ICE_GATE_UNLOCKED_ATTRIBUTE, 1);
      replaceBracket(player, event.object, LOWERED_LEVER_ID);
      player.sendMessage("You hear the clunking of some hidden machinery.");
      return;
    }
    if (objectId === TRAPPED_LEVER_ID && sameTile(location, TRAPPED_LEVER_TILE)
      && String(event.option ?? event.definition?.getInteractions?.()[event.clickType - 1]).toLowerCase() === "search for traps") {
      event.handled = true;
      if (thievingLevel(player) < 42) {
        player.sendMessage("You find nothing.");
        return;
      }
      player.setAttribute(TRAP_DISARMED_ATTRIBUTE, 1);
      quest.setStage(player, quest.getStage(player));
      player.sendMessage("You find a trap on the lever! You disable the trap.");
      return;
    }
    if (objectId === TRAPPED_LEVER_ID && sameTile(location, TRAPPED_LEVER_TILE)
      && String(event.option ?? event.definition?.getInteractions?.()[event.clickType - 1]).toLowerCase() === "pull") {
      event.handled = true;
      pullTrappedLever(player);
      return;
    }
    if (objectId === FIRE_WARRIOR_DOOR_ID && sameTile(location, FIRE_WARRIOR_DOOR_TILE)) {
      if (quest.getStage(player) < STAGE_STARTED || quest.isComplete(player)) return;
      migrateProgress(player);
      if (progressAt(player).killed) return;
      event.handled = true;
      openFireWarriorDoor(player);
      return;
    }
    if (objectId === PUSH_WALL_ID && sameTile(location, PUSH_WALL_TILE)) {
      event.handled = true;
      const here = player.getLocation();
      const north = here.getY() <= PUSH_WALL_TILE.y;
      const y = north ? PUSH_WALL_TILE.y + 1 : PUSH_WALL_TILE.y - 1;
      player.moveTo(new api.core.Location(PUSH_WALL_TILE.x, y, here.getZ()));
      return;
    }
    if (objectId === ICE_CHEST_CLOSED_ID || objectId === ICE_CHEST_OPEN_ID) {
      const index = iceChestIndex(location);
      if (index >= 0) handleIceChest(event, index);
    }
  }

  function handleLogin({ player }) {
    migrateProgress(player);
    quest.setStage(player, quest.getStage(player));
    ensureLucien(player);
  }

  function handleLogout({ player }) {
    if (player) {
      removeSpawns(player);
      removeBracket(player);
      removeIceChests(player);
      trapTasksByPlayer.get(player)?.stop();
      guardianClearedByPlayer.delete(player);
      staffTakeByPlayer.delete(player);
      defeatedLucienByPlayer.delete(player);
    }
  }

  class TempleOfIkovDungeon extends api.core.Area {
    postEnter(mobile) {
      if (!mobile.isPlayer()) return;
      const player = mobile.getAsPlayer();
      if (hasFlag(player, ICE_GATE_UNLOCKED_ATTRIBUTE)) {
        const base = api.core.MapObjects.get(LEVER_BRACKET_ID, constructLocation(LEVER_BRACKET_TILE), player.getPrivateArea());
        if (base) replaceBracket(player, base, LOWERED_LEVER_ID);
      }
    }
    postLeave(mobile) {
      if (!mobile.isPlayer()) return;
      const player = mobile.getAsPlayer();
      // A fitted but unpulled lever is lost on leaving the dungeon; the pulled
      // mechanism is durable. Cache chests remain closed for the next visit.
      removeBracket(player);
      removeIceChests(player);
      trapTasksByPlayer.get(player)?.stop();
      removeFireWarrior(player);
      guardianClearedByPlayer.delete(player);
      staffTakeByPlayer.delete(player);
    }
  }

  const PAGE = "Temple of Ikov";

  quest = registerQuest(api, {
    key: "temple_of_ikov",
    name: "Temple of Ikov",
    varpId: VARP_TEMPLE_OF_IKOV,
    varpValue: nativeStage,
    startedValue: STAGE_STARTED,
    completionValue: STAGE_COMPLETE,
    questPoints: 1,
    xpRewards: [
      { skillId: Skill.RANGED.getIndex(), amount: 10500, label: "Ranged" },
      { skillId: Skill.FLETCHING.getIndex(), amount: 8000, label: "Fletching" },
    ],
    scrollItemId: ItemIdentifiers.BOOTS_OF_LIGHTNESS,
    otherRewards: ["Boots of lightness (optional dungeon pickup)", "Shiny key shortcut (optional)"],
    buildJournal,
    onReward: grantReward,
  });

  api.persistAttribute(LEGACY_SIDE_ATTRIBUTE, SIDE_NONE);
  api.persistAttribute(SIDE_ATTRIBUTE, SIDE_NONE);
  api.persistAttribute(STATE_VERSION_ATTRIBUTE, 0);
  api.persistAttribute(WINELDA_MET_ATTRIBUTE, 0);
  api.persistAttribute(WINELDA_PAID_ATTRIBUTE, 0);
  api.persistAttribute(WARRIOR_KILLED_ATTRIBUTE, 0);
  api.persistAttribute(TRAP_DISARMED_ATTRIBUTE, 0);
  api.persistAttribute(TRAP_PULLED_ATTRIBUTE, 0);
  api.persistAttribute(STAFF_DELIVERED_ATTRIBUTE, 0);
  api.persistAttribute(ICE_GATE_UNLOCKED_ATTRIBUTE, 0);
  api.persistAttribute(LUCIEN_DEFEATED_ATTRIBUTE, 0);
  api.registerArea(new TempleOfIkovDungeon([new api.core.Boundary(2624, 2753, 9758, 9916, 0)]));
  api.onNpcDialogueVariant(selectVariant);
  api.onNpcDialogueCondition(answerCondition);
  api.onCustomEvent("npc-dialogue:hook", handleStartHook);
  api.onCustomEvent("npc-dialogue:action", handleAction);
  api.onNpcInteraction(handleNpcInteraction);
  api.onObjectInteraction(handleObjectInteraction);
  api.onItemOnObject(handleLeverOnBracket, { noted: false });
  api.onCustomEvent("ladders:climb", guardBootsStairs);
  api.onCustomEvent("ladders:climb", climbOutOfPit);
  api.onCustomEvent("door:toggle", guardDungeonDoors);
  api.onCanAttack(handleCanAttack);
  api.onNpcDeath(handleFireWarriorDeath);
  api.onNpcDeath(handleLucienDeath);
  api.onNpcDeath(handleGuardianDeath);
  api.onCustomEvent("ground-items:take", handleStaffTake);
  api.onCustomEvent("ground-items:taken", handleStaffTaken);
  api.onCustomEvent("magic:can-telekinetic-grab", canTelegrabStaff);
  api.onPlayerLogin(handleLogin);
  api.onPlayerLogout(handleLogout);
};
