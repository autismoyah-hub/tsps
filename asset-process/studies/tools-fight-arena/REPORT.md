# Warden-General in the tower court

[Play the 30-second encounter](../../render/fight-arena-arena1/encounter.gif) · [Inspect the 12-frame sheet](../../render/fight-arena-arena1/encounter.png)

This film uses the Producer's `night/arena1/boards/final` court and the fight films' `asset_arena.exe` renderer. The General stands on the star ring as NPC 16386. All ten timeline entries play, including Orbit during other attacks and the recurring sight after half health. The cache retains the boss's sequences 20271–20273 byte for byte. The timeline, attacks and phase data match the source fight.

## Map and stage

[arena_pack.py](arena_pack.py) reads the board's scene-played registry, definitions, models, BWR floor and placement table. It checks registry CRCs, the declared floor hash and the tile, shape and rotation of each placement. [The manifest](cache-verified-inputs/manifest.json) builds [the fight cache](cache-verified/) from a copy of `fx-warden-general/cache-own`. All package IDs were free. The packer read back 562 added files byte for byte.

Archive 5 group **4112**, region **(16,16)**, covers world tiles **1024–1087** on each axis. It was the first square in the packer's search with all nine squares of its 3 × 3 neighbourhood absent from the source map index. The court fits inside that square. Add 1024 to either court coordinate to obtain its world coordinate.

The map has 497 scenery placements, including 320 shape-22 floor decorations. It uses the room's flat heights, walk flags and terrain material 5808 (underlay definition 5807, RGB `282d2d`). All 30 models and 498 object definitions are packed. The static boss definition is available for inspection. The map omits its placement so the actor path draws him once. The eight fire piers and the Pass barrier use sequences 20320 and 20322, two bases and 27 classic frames extracted directly from the arena effect cache. The renderer reported nine animated map objects pinned at frame zero.

| Occupant or effect | Court tiles | World tiles |
|---|---|---|
| Warden-General | x 43–46, z 43–46 | x 1067–1070, z 1067–1070 |
| Player | (44,38) | (1068,1062) |
| Meridian warning and strike | x 41–47, z 38 | x 1065–1071, z 1062 |
| Orbit | (45,38) | (1069,1062) |

The original six-tile separation put the player at (44,37). All seven tiles had ground, but the entrance piers and parapet obscured part of the warning in the played view. Moving only the player one tile inward matches the composition's player tile. All seven target tiles and Orbit have floor and clear the scenery footprints. The attack offsets and boss position are preserved.

[play_arena.py](play_arena.py) reads the place, camera and cache from [fight-arena.json](fight-arena.json). The 608 × 432 fight camera keeps the room's played aim point, turns slightly from the door axis (yaw 700), and looks down from 1900 units with 1800 units of setback, target lift 340 and a 670-pixel lens. This keeps the combatants, the row ends and the head effect in frame.

## Static pixel comparison

**0 of 1,152,000 RGB pixels differed.** [proof.py](proof.py) re-rendered the empty court at 1280 × 900 with the room snapshot executable and its effect cache, then compared that capture with the board's saved top-empty image. It rendered the same 497 placements from the new map with the fight executable, one client cycle from the pinned start. Both cameras used local aim (45,45), yaw 0, distance 250, elevation 4055.04, target lift 0 and focal length 1000. The difference mask is empty across the ground, objects and background. This comparison covers the arena at that pose and cycle. Actor and fight-effect visibility was checked in the films. [Room capture](../../render/fight-arena-arena1/room-proof.png) · [Map capture](../../render/fight-arena-arena1/map-proof.png) · [Difference mask](../../render/fight-arena-arena1/difference.png) · [Counts and poses](../../render/fight-arena-arena1/pixel-proof.json)

## Film review

Each GIF samples every five client cycles. The player checked the encoded duration of every film. Each sheet contains 12 evenly spaced samples.

| Film | Duration | Sheet inspection |
|---|---:|---|
| [Encounter](../../render/fight-arena-arena1/encounter.gif) | 30.0 s | Boss and player stay in view. At 5.4 s the active dial and marked row share the floor. Some outer roof and north-wall details fall outside the combat framing. [Sheet](../../render/fight-arena-arena1/encounter.png) |
| [Star Lance](../../render/fight-arena-arena1/star_lance.gif) | 3.1 s | The bolt crosses the ring toward the player. The impact briefly covers the player's torso. The small blue-grey impact loses contrast against the slate as it fades. [Sheet](../../render/fight-arena-arena1/star_lance.png) |
| [Meridian Line](../../render/fight-arena-arena1/meridian_line.gif) | 3.1 s | Both ends of the warning and strike stay on screen. The dark early marks are faint. The player overlaps the centre segment and the bright strike passes through his tile. [Sheet](../../render/fight-arena-arena1/meridian_line.png) |
| [Orbit](../../render/fight-arena-arena1/orbit.gif) | 7.0 s | The dial appears beside the player, turns and disappears on release. The near fire pier leaves its tile visible. [Sheet](../../render/fight-arena-arena1/orbit.png) |
| [Half-health phase change](../../render/fight-arena-arena1/phase_change.gif) | 3.8 s | The ignition and sustained sight fit above the helmet. The brass sight is small against the ornamented far wall. [Sheet](../../render/fight-arena-arena1/phase_change.png) |

## Rebuild for final2

From Git Bash, run this one command after the revised board has been composed. The timestamp supplies the new cache path required by fxpack. The command refreshes the fight JSON, pixel proof and films in the same review directory.

```bash
{PYTHON} -I {ROOT}/studies/tools-fight-arena/arena_pack.py {ROOT}/prompts/arena1 "{ROOT}/studies/tools-fight-arena/cache-final2-$(date +%Y%m%d-%H%M%S)" --board final2 --films
```

The full pack, proof and five-film command ran successfully on `final`. The revised board was unavailable during this run. The importer checks that the composed court fits one square and that its source package IDs are free in the fight cache.

## Failed commands and recovery

- The initial `Get-Command bash,git,cuttle` lookup missed Bash and Git on PATH. Passing the Git Bash executable as the shell for the initial `pwd; ls ...` probe sent it to cmd, which rejected `pwd`. Invoking the installed `{GIT_BASH} -lc` directly worked.
- Initial `ls` probes for `night/arena1/scene-played` and its package looked one level too high. The composed package lives under `boards/final/scene-played`. The first `ls studies/tools-fight-arena` found no existing tool directory. `git rev-parse --show-toplevel` confirmed the delivery root has no Git repository. Work used dedicated build and render directories. The existence probes for the new render directory and pending `boards/final2` also found no directory at the time.
- The first `proof.py` invocation passed yaw as `0.0`. The room executable requires an integer and returned `ParseIntError`. The proof now passes integer yaw.
- The first `play_arena.py probe` loaded both actors and nine animated objects, then failed with `Invalid([quoted words withheld])`. The room loader mounts the authored floor palette in memory. Packing its matching underlay definition along with the terrain fixed the map load. The failed probe cache and temporary script were cleaned up.
- A final inline Python readback command lost shell quoting and returned `unexpected EOF`. The boss sequence and fight-data comparisons now run inside `proof.py` and passed on the delivered cache.

