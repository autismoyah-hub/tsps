# Tekton in the Chambers of Xeric

[Play Tekton’s 30-second encounter](../../render/fight-real-tekton/encounter.gif) · [Tekton’s 12-frame sheet](../../render/fight-real-tekton/encounter.png)  
[Play the Warden-General’s 30-second encounter](../../render/fight-arena-arena1/encounter.gif) · [Warden-General’s 12-frame sheet](../../render/fight-arena-arena1/encounter.png)

The clip opens as Tekton plants his feet on the west room’s combat tile. It follows all nine normal swings, the walk back to the anvil, three two-spark volleys during a twelve-tick repair, and his first enraged attack. The cut at 30.0 seconds catches the swing. Its hit is due at the cut, after the last sample at 29.9 seconds. This is a deterministic reconstruction of one legal sequence of server decisions. The GIFs carry no sound, health bars, damage numbers, or live raid session data.

## Room, assets and camera

The source is the revision-239 disk cache resolved by `fx.SRC`. Every NPC, animation, projectile, tile graphic and object is drawn by cache ID. The stage is the unrotated plane-1 template for the west instance, with seed (3282,5296). Tekton’s 4 × 4 fighting footprint runs from (3280,5296) through (3283,5299). The raider stands at (3281,5295), adjacent to Tekton’s south side and clear of the corner. The actor returns to the plugin’s home base (3279,5298) beside Giant Anvil 29867. The player stays on the same tile. [Fight data](fight-tekton.json) · [Arena catalogue](../research-arenas/arenas.py) · [Room survey](../research-arenas/ARENAS.md)

| Camera or output | Warden-General | Tekton |
|---|---:|---:|
| Aim in world tiles | (1068.725, 1065.425) | (3281.725, 5295.925) |
| Yaw in 1/16384 turns | 700 | 700 |
| Horizontal setback / elevation above aim | 1800 / 1900 | 1800 / 1900 |
| Target lift / focal length | 340 units / 670 px | 340 units / 670 px |
| Image, sampling and playback | 608 × 432, 5 cycles, 100 ms | 608 × 432, 5 cycles, 100 ms |
| Sheet | 4 × 3 half-size views | 4 × 3 half-size views |

The room survey’s generic played camera uses yaw 0, setback 2300, elevation 1650 and lift 90. The reference instead matches the completed Warden film. Both use the same camera angle and view width at the aim point (about 18.6 tiles). The orange-lit back wall and anvil identify Tekton’s room. The camera crops part of the perimeter, including the distant entrance fire. Each sheet selects the nearest sample to each of twelve evenly spaced times, including the first and last. The timestamp, attack and action use the Warden caption font, position, bar height and colours. [Warden film code](../tools-fight-arena/play_arena.py) · [Tekton film code](play_real.py)

### Plane support and renderer check

The stock `asset_arena` builds the world mesh for plane 0. The arena survey’s `arena_view` adds a plane selection for the mesh and camera, but constructs the player, projectile endpoints, world-spot positions and dynamic object keys on plane 0. In a plane-1 view the client filters the player and effects by plane. The local build routes those inputs to the selected plane and feeds form changes and queued walk steps to the same client NPC. It compiles against the existing client library. Geometry, animation, depth, textures, lighting, projection, draw order and effect clocks use the existing renderer. Nex fallback was unnecessary. [Local build script](build_view.py) · [Survey renderer patch](../research-arenas/rust/patch_arena_view.py)

The stock Warden executable and the local build produced identical full 608 × 432 RGB frames on a plane-0 stock-cache control at cycles 0, 40 and 125, covering actors, a swing, a projectile and a landing. On plane 1, removing one element at a time from the repair scene changed 7,598–8,761 visible pixels for Tekton and 345–356 for the raider across thirteen samples. Both sparks combined changed 184 visible pixels at cycle 105 (2.1 s). The two landing graphics changed 1,366 at cycle 170 (3.4 s). These counts exclude the caption area. [Test](verify.py) · [Pixel counts](visibility.json)

## Choreography and timing

One server tick is 30 client cycles or 0.6 seconds. The encounter uses the plugin’s first-visit swing limit, legal repair duration, melee style choices and spark rolls. The chosen attack order, two-step walk and rolls are fixed for repeatable review. A player flinch on melee contact follows the Warden film convention. The entrance fire is object 30021 and the temporary footprint blocker is 30023. The protected safe tile is by the doorway.

| Action | What the plugin schedules | What the film shows |
|---|---|---|
| Plant and normal melee | Fighting form 7542, stance 7478 delayed 30 cycles. First swing 3 ticks after plant, then every 3 ticks. Stab 7482 and crush 7484 hit one tick later. Slash 7483 hits two ticks later. | Plant at tick 0. Swings on ticks 3, 6, 9, 12, 15, 18, 21, 24 and 27. The slash at 27 changes to walking form on the same tick. The player is on the attacked side. |
| Return and repair | After the ninth swing, walking form 7541. First step two ticks later. At home, form 7545, animation 7475 with a 15-cycle delay. Repair lasts 12, 15, 18 or 21 ticks. Healing is every third tick except the last. | Steps at 29 and 30, repair starts at 31. The twelve-tick option ends at 43. Healing has no separate effect graphic in the film. |
| Repair sparks | Start on the tick after arrival, repeat every 4 ticks, two per eligible raider. Fixed room sources rotate within each volley. Launch delay 1–27 cycles, angle 20–25, heights and progress zero. End cycle is 60 + 5 × Chebyshev tile distance. Landing graphic on the ceiling of end / 30 ticks, plus 0–5 cycles. Damage 10–20 within one tile. | Pairs launch from (3271,5291) and (3284,5285) on ticks 32, 36 and 40. Gfx 660 flies to (3280,5295) and (3282,5294). Both distances are nine tiles. Delays are 9 and 21 cycles, angles 21 and 24, both end at cycle 105. Gfx 659 starts at relative cycles 122 and 125. The solo repair clip includes the fading landings. |
| Enraged melee | Rise 7487, walking form 7543 two ticks later. Plant as 7544 with 7488 delayed 30 cycles. First swing two ticks later, then every three. Stab 7493, slash 7494 and crush 7492 retain the normal hit delays. Six enraged swings before calming, sixteen total before the next retreat. | Rise at 43, approach at 45, plant at 47, stab at 49. The individual films cover each enraged style. |
| Walking stomp | Tekton checks each moving tick for a player inside the footprint and deals 1–3 damage. It has no dedicated attack graphic. | A separate walking-form clip stages the player in the path and shows the underfoot contact. The encounter raider stays outside Tekton’s return path. |

The plugin’s executable swing counters and repair values take precedence over the historical wiki summary in its opening comment. The cut starts at first plant, after the dormant 7540 rise, and ends before a kill. [Tekton plugin, forms, timings, room variant and combat callbacks](<{TSPS_CHECKOUT}/server/plugins/minigames/cox/Tekton.ChambersOfXeric.js>) · [Server landing-tick calculation](<{TSPS_CHECKOUT}/server/src/main/typescript/elvarg/game/model/Projectile.ts>) · [Tile distance](<{TSPS_CHECKOUT}/server/src/main/typescript/elvarg/game/model/Location.ts>) · [Reference fights table](../fx-warden-general/REPORT.md)

## Films and sheet inspection

| Film | Duration | What to look for on the 12-frame sheet |
|---|---:|---|
| [Encounter](../../render/fight-real-tekton/encounter.gif) | 30.0 s | The planted swings occupy the first two rows. The return opens space between the actors. The paired landings are visible at 24.5 s, and the last two panels show the change from amber to red. The small raider has low contrast against the dark floor. [Sheet](../../render/fight-real-tekton/encounter.png) |
| [Stab](../../render/fight-real-tekton/stab.gif) | 1.8 s | The blade points down along the near edge of the footprint at 0.6 s. The raider remains visible below the tip. [Sheet](../../render/fight-real-tekton/stab.png) |
| [Slash](../../render/fight-real-tekton/slash.gif) | 1.8 s | The blade sweeps across Tekton’s front and extends to screen right. The raider stays below the swing. [Sheet](../../render/fight-real-tekton/slash.png) |
| [Hammer / crush](../../render/fight-real-tekton/crush.gif) | 1.8 s | The raised hammer at 0.3–0.5 s and lowered head at 0.6 s read most clearly. The anvil shares the upper silhouette at the top of the lift. [Sheet](../../render/fight-real-tekton/crush.png) |
| [Enraged stab](../../render/fight-real-tekton/enraged_stab.gif) | 1.8 s | The red thrust descends in front of Tekton’s bright footprint. The raider is visible just beyond it. [Sheet](../../render/fight-real-tekton/enraged_stab.png) |
| [Enraged slash](../../render/fight-real-tekton/enraged_slash.gif) | 1.8 s | The red blade crosses his body, then reaches screen right. The near edge of the red footprint helps locate the raider. [Sheet](../../render/fight-real-tekton/enraged_slash.png) |
| [Enraged hammer / crush](../../render/fight-real-tekton/enraged_crush.gif) | 1.8 s | The hammer rises beside the anvil, then drops near the raider at 0.6 s. The anvil and dark hammer overlap visually at the apex. [Sheet](../../render/fight-real-tekton/enraged_crush.png) |
| [Repair sparks](../../render/fight-real-tekton/repair_sparks.gif) | 7.5 s | The thin amber projectiles approach from screen left and the lower edge. Their distant launch points approach or cross the frame edge. The two upright landing bursts flank the raider at 3.4 s. Late embers blend into the warm, mottled floor. [Sheet](../../render/fight-real-tekton/repair_sparks.png) |
| [Walking stomp](../../render/fight-real-tekton/stomp.gif) | 1.8 s | In this clip the raider stands at (3279,5297) on the western side. Tekton’s northwest step catches that tile at 0.6 s and his northward step clears it at 1.2 s. Both bodies are visible. The hit is a footprint check with no separate burst or literal foot-to-helmet contact. [Sheet](../../render/fight-real-tekton/stomp.png) |

## Reproduction

From Git Bash, run the local player with the specified isolated Python. With no arguments it writes the encounter and all eight individual films. An attack name selects a single film. The included local renderer can be rebuilt with `build_view.py`. The verification script compares captured RGB frames.

```bash
{PYTHON} -I {ROOT}/studies/tools-real-fight/play_real.py
{PYTHON} -I {ROOT}/studies/tools-real-fight/verify.py
```

## Command failures and recovery

- The initial PowerShell discovery probe `Get-Command bash` missed Bash on PATH. Five read commands passed to the shell tool with Git Bash as the shell were interpreted by cmd, which rejected `cat`. These targeted the Warden arena files, Warden fight files, fight compiler, arena survey and Tekton plugin. Reissuing the reads through the installed Git Bash executable with `-lc` worked.
- The initial `ls -d` probes for the new build, render folder and delivery-root `.git` found none. The build and output use fresh dedicated directories. The client checkout also had no `.git` at the probed location.
- Toolchain discovery found no `target/release/.fingerprint`, `{WORK_DIR}/*rust*`, `/c/msys64/mingw64/bin/gcc.exe`, client `.cargo`, `.cargo/config.toml` or `build.rs`. `which gcc` and the `where.exe gcc` fallback failed. An abbreviated `6f996e*` build-artifact glob missed its target. The full path worked. The installed GNU Rust toolchain, its bundled linker and the existing release libraries built the local executable. Rust emitted two existing unused-camera warnings.
- An `rg` probe for `src/client_state/actor*.rs` missed the source. Movement ticking lives in `src/client_state.rs`. One `ls` of a transient render directory ran after cleanup, and one `rg` of the encounter log ran before encoding finished. Neither affected rendering.
- A final inline Python query of the arena survey JSON lost its string quotes through the shell wrapper and raised `SyntaxError`. Reading the JSON directly through Git Bash worked.
- A shell pipeline checking report links lost quoting around its loop and returned an unexpected end-of-file error. [The delivery checker](check_delivery.py) checks the links and decodes every final GIF and sheet directly.

