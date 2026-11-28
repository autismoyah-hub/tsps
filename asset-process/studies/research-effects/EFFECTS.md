# Effects: what a fight shows that is not the boss or the room (revision 239 cache)

Private to this machine. Everything here was read from the client's source (`client/rust-client/src/`, read only)
or measured on the cache by the scripts named. P = `{PYTHON} -I -B`, run from
`{ROOT}/studies/research-effects/`.

```
P census.py                    every spot animation -> data/spots.jsonl          P summary.py all|tokens|grep REGEX -> data/census.txt
P gallery.py sheets|gifs|measure   the study set by job -> render/research-effects-gallery/, data/gallery-measure.json
P objanim.py [REGEX]           scenery objects that animate -> data/objanim.txt
P fx.py sheet|gif|dump|faithful    the harness (section 3)                       P fxtools.py reencode|sheet|compare|gate|lineup|style
```

## 1. What the game's effects are made of

**A spot animation ("graphic", "spotanim")** is one small definition: archive 2, group 13, file = id (`spot_animation.rs`).

| opcode | field | notes |
|---|---|---|
| 1 (u16) / 3 (u32) | model | ONE model (archive 7). No list: an effect is a single mesh |
| 2 | sequence | optional; 169 of 4,008 have none (a static mesh: arrows, bolts, thrown things) |
| 4, 5 | scale xz, scale y | 128 = natural; applied AFTER the animation; 548 set it (64, 96, 100, 256, 300 commonest) |
| 6 | rotation | only 90, 180, 270 do anything (quarter turns); 129 set it |
| 7, 8 | ambient, contrast | light = `light(64 + ambient, 850 + contrast, (-30, -50, -30))`: the NPC light, from above. Contrast is used as stored (an NPC's is multiplied by 5, a spot's is not) |
| 10 | a flag (clears the pick tag) | |
| 40, 41 | recolours, retextures | 1,092 are recoloured: one mesh serves many effects |

There is nothing else: no height, no duration, no job. Whether a spot animation is a projectile, an impact or a floor
marker is decided by the PACKET that names it (section 2). The same id can be all three.

**How it is drawn** (`world_actors.rs` `place_spot` and the projectile branch of `triangles`): the model is recoloured,
lit ONCE in its rest pose and cached per id; each frame a copy is posed by the sequence (`Model::animate_classic`),
scaled, quarter-turned, then
- on an actor: lifted by the packet's height, placed at the actor's centre on the ground, turned by the actor's facing,
  given the actor's own transparency override, and lifted with the actor by the height byte (sequence opcode 16) of
  whatever the actor is doing (`shift_actor_heights`: found by the harness, 26,058 pixels differed until it was repeated);
- on a tile: lifted by the packet's height (one byte), placed at the tile centre at the ground's elevation, never turned;
- as a projectile: every vertex turned about the X axis by the flight pitch, placed at the flight position turned by
  the flight yaw. THE MODEL FLIES WITH ITS -z END FORWARD (the Bronze arrow's box is z -5 .. 72: the tip is at the origin).
Painting is the scenery path (`Mode::Auto`): per-pixel depth, faces of one model in model order, translucent faces in a
second pass sorted far to near; face priorities do nothing; per-face alpha blends `(256 - a) / 256` of the face over
what is behind.

**Lifetimes** (`actor_spot.rs`, `world_spot.rs`, `client_state.rs`):
- on an actor or a tile the effect is REMOVED AT THE FIRST WRAP OF ITS SEQUENCE, whatever the sequence's loop fields
  say. A spot animation on an actor or tile never loops. A persistent thing is (a) a sequence with a long last hold
  (Hydra's pool: 5 frames of 3 cycles, then one frame held 465), (b) the same id sent again, or separate
  spawn / idle / despawn ids (Doom, the Wyrd, smoke devil pools), or (c) not a spot animation at all but an OBJECT.
- a projectile's sequence does loop (its clock is never tested for the wrap); the projectile ends at its END cycle.
- a spot with NO sequence on an actor or tile is removed at once: static meshes are projectiles only.
- an actor holds several at once in numbered slots; a new one in the same slot replaces the old.

**Projectiles** (`projectile.rs` `Motion::advance`): from `start` to `end` (client cycles) the point moves in a straight
line over the ground from the source (pushed `offset` units toward the target first: 64 = half a tile, out of the
caster's body) to the target, which is re-read every cycle if it is an actor (it homes). Height starts at the source
ground minus `start height`, ends at the target ground minus `end height`, with an initial climb of
`tan(slope * 1.40625 deg)` times the ground speed and constant acceleration between: `slope` 0 is a straight line,
16 is the ordinary arc, 64 would be vertical. Yaw and pitch follow the velocity every cycle. Heights are units (128 a
tile; a player is about 190 tall). The owner's server stores them in quarters (`WeaponProfile.ts`: startHeight 40.75,
endHeight 36.5, delay 41 cycles, slope ("angle") 16 or 5, offset ("progress") 64 or 11; duration = 5 + 5 per tile of
distance): 40.75 means 163 units, chest height.

**Scenery objects that animate** (object definition opcode 24 names a sequence; `location_animation*.rs`; `LOC_ANIM`
replays one): 5,408 of 62,194 object definitions (8.7%), 1,232 distinct sequences (`data/objanim.txt`). This is where
persistent hazards and arena pieces live: `tob_xarpus_acidpool` 32744 and `hydraboss_pool` 34511 are OBJECTS (the spot
animations of the same names are the splash that lands), `nightmare_spores` 37739, `tob_sotetseg_lighttile` 33035,
Verzik's collapsing pillars 32688, Hydra's vents 34568 to 34570, and the Great Olm's head and hands (29881, 29884,
29887) are objects with sequences, not NPCs. An object can block movement, be clicked, and stays until removed; a spot
animation can do none of those. The project already builds objects; an object WITH a sequence is one more opcode and a
sequence, and is not built or drawn here (section 7).

**Also on screen, not models** (noted, not pursued): hit splats (85 definitions, archive 2 group 33) and health bars
(83, group 32) are sprites drawn over the actor; overhead prayer and skull icons are sprites; the hint arrow is a
model drawn by the client; camera shake (`CAM_SHAKE`) and screen fades are interface and camera packets.

## 2. Census (`data/census.txt`, `data/spots.jsonl`)

4,008 spot animations, every one named in the cache's own name table (archive 24, group 8), 1,845 distinct models and
1,972 distinct sequences: reuse is the norm (`wardens_lightning` plays the Hallowed Sepulchre's lightning; the
Whisperer's ranged attack is the Nightmare's; Nex's five attack impacts are the Ancient Magicks' impacts).

| | all | ids 0..999 | 1000..1999 | 2000..2999 | 3000 up |
|---|---|---|---|---|---|
| faces, 10 / 50 / 90 centile | 66 / 390 / 1,688 | 48 / 200 / 500 | 66 / 288 / 1,074 | 158 / 673 / 2,707 | 99 / 760 / 1,982 |
| largest dimension, units | 50 / 168 / 510 | 40 / 106 / 256 | 50 / 139 / 412 | 85 / 256 / 600 | 80 / 285 / 704 |
| colours (covering 90% of surface) | 3 (3) | 3 (2) | 4 (3) | 4 (3) | 4 (3) |
| any translucent face | 58% | 38% | 58% | 66% | 71% |
| wholly translucent | 12% | 16% | 15% | 9% | 10% |
| any flat-shaded face | 21% | 3% | 22% | 28% | 32% |
| textured | 7 effects | | | | |
| sequence: classic / skeletal / none | 3,701 / 138 / 169 | 936 / 0 / 64 | 941 / 0 / 59 | 875 / 98 / 27 | 949 / 40 / 19 |
| life, seconds (classic) 10 / 50 / 90 | 0.4 / 1.2 / 4.2 | 0.3 / 1.1 / 4.6 | 0.4 / 1.2 / 3.7 | 0.6 / 1.2 / 3.6 | 0.5 / 1.2 / 4.2 |
| frames 10 / 50 / 90 | 6 / 15 / 44 | | | | |
| frame hold, cycles 10 / 50 / 90 | 2 / 4 / 5 | | | | |

The maximum is 3,854 faces. Among translucent models the mean alpha is 46 / 111 / 175 at the 10th, 50th and 90th
centile (0 opaque, 255 gone): effects are mostly half see-through or less. 140 models carry bone weights. Most are
SMOOTH shaded and untextured: an effect is a few plain colours, vertex-lit, exactly like the rest of the game.

How motion is built (classic sequences, 3,701): scale in 2,941, alpha in 3,002, translate in 2,986, rotate in 2,805;
all four together in 1,850 (half). The largest dimension changes by 1.0 / 1.4 / 17 times (10 / 50 / 90 centile) within
a sequence; 1,723 grow by half or more. 84% play once; 16% carry loop fields (which only projectiles honour).
The mesh never changes: an effect is ONE mesh whose parts are labelled, scaled from nothing, moved, turned and faded.
"Particles" are small labelled solids moved by translate and scale groups; a cloud is 20 to 40 lumps on their own
labels (Doom's slam: 2,021 faces on 75 vertex labels and 243 groups). A frame base for an effect has 5 / 43 / 240
groups (10 / 50 / 90 centile): the old ones are tiny, the new ones use the whole table.

By job, from the names (`summary.py` JOBS; a name rule, so approximate):

| job | n | faces 10 / 50 / 90 | largest dimension 50 | translucent | life s 10 / 50 / 90 | grows 1.5x or more | notes |
|---|---|---|---|---|---|---|---|
| projectile | 863 | 50 / 232 / 1,001 | 112 | 54% | 0.2 / 0.6 / 1.9 (loop period) | 15% | 139 have no sequence; rotate is the commonest transform (587 of 712); 185 loop |
| impact | 524 | 178 / 408 / 2,440 | 232 | 66% | 0.4 / 0.6 / 2.1 | 64% | scale in 484 of 512, alpha in 477 |
| launch / cast | 322 | 44 / 224 / 1,081 | 96 | 46% | 0.8 / 1.2 / 2.5 | 56% | translate in 306 of 320 (it follows the hands) |
| ground hazard | 241 | 174 / 484 / 1,638 | 256 | 64% | 0.5 / 1.2 / 3.0 | 44% | |
| telegraph | 103 | 190 / 497 / 1,470 | 240 | 77%, 26% wholly | 1.1 / 2.4 / 6.0 | 62% (median 3.5x) | 27 use scale ALONE; 1 or 2 colours at the median |
| aura / shield | 339 | 96 / 448 / 3,032 | 225 | 61%, 19% wholly | 0.6 / 1.5 / 3.0 | 34% | |

## 3. The study set: 58 real effects drawn by id, by job

`render/research-effects-gallery/sheet-JOB.png` (six moments each, game size) and `motion-JOB.gif` (game timing).
Names are the cache's. On-screen numbers are from `gallery.py measure` (`data/gallery-measure.json`): the game's
512 x 334 view, a player about 62 pixels tall (about 1,100 pixels of area); "pixels" = pixels the effect visibly
changes at its largest; "distance" = mean change per channel of those pixels from the scene behind (0..255).

| job | effects (id name: pixels / distance) |
|---|---|
| projectile (16; all given the same 60-cycle flight) | 10 bronze_arrow_travel 36/31; Nex: 1997 smoke 223/29, 1999 shadow 73/30, 2004 ice 335/56, 2007 finale 215/45; Nightmare: 1764 magic 90/51, 1766 ranged 117/30; 1340 olm_generic_range_proj 210/31; Verzik: 1580 phase 1 365/119, 1583 phase 2 ranged 408/64, 1593 phase 3 range 238/24; 2206 toa_wardens_prayer_ranged_travel 741/74; 3445 vfx_doom_boss_blood_projectile 91/84; 3254 vfx_yama_flaming_rock_projectile_01 1,564/61; 1477 vorkath_ranged_travel 187/43; 1711 crystal_hunllef_range_travel 126/64 |
| impact (10, on a player at height 100) | Nex: 1998 smoke 81/21, 2008 finale 770/39, 2005 ice 3,873/76; 1765 nightmare_magic_impact 770/42; Verzik: 1582 impact pillar 2,067/45, 1584 ranged impact 1,844/27; 2205 toa_wardens_prayer_melee_impact 663/30; 3255 vfx_yama_flaming_rock_impact_01 1,706/78; 2712 vfx_doom_scorpion_attack_impact_01 84/69; 1478 vorkath_ranged_impact 120/45 |
| casting, charging, auras, shields (10, on a player) | Nex: 2016 turmoil 57/89, 2015 blood siphon 322/31, 2017 summon 350/71; 1760 nightmare_staff_volatile_cast 214/116; 2125 tumekens_shadow_casting 1,111/49; 3243 vfx_npc_yama_magic_shadow_spotanim01 4,803/35; 2141 kephri_shield_alive 8,283/34; 1558 tob_nylocas_shielded 1,089/39; 1597 verzik_powerblast_protection 1,665/39; 3280 vfx_yama_fire_immunity 1,651/49 |
| ground hazards (8, on a tile) | 1654 hydraboss_pools_landed_0 397/90 (9.6 s); 1767 nightmare_rift 1,142/42; 2013 nex_mushroom_cloud_spotanim 991/122; 3857 vfx_wyrd_blood_pool_idle_01 618/50; 3429 vfx_doom_boss_blood_splat_n 215/106; 1356 olm_shockwave 816/40; 2432 spotanim_duke_vent_01_idle_01 9,916/27; 2453 spotanim_whisperer_tentacle_01_idle_01 (almost nothing drawn alone) |
| telegraphs (8) | 1446 gargboss_debris_shadow_90 437/18; 2198 wardens_lightning_warn 429/72 and 2199 wardens_darkness_warn 429/48 (recoloured lightning, not floor marks); 3415 vfx_doom_boss_burrowed_movement_telegraph 495/30; 2667 npc_colossi_colossi_tripleattack_01_telegraph 952/23; 1595 verzik_powerblast_safezone 529/58; 2146 kephri_indicate_player_for_dung 645/31; 2440 vfx_duke_sucellus_attack_melee_spotanim_tile_01 1,097/52 (the ice that rises IS the warning and the hit) |
| the hit after a telegraph (5 drawn; 1445 is a static projectile and draws nothing on a tile) | 2197 wardens_lightning 429/48; 3370 vfx_doom_area_slam 2,901/13; 2216 spotanim_wardens01_explosion01_explode01 5,249/39 (skeletal); 3262 vfx_player_yama_falling_rock_impact01 1,432/62; 2148 kephri_kamikaze_scarab_explode 975/30 (skeletal) |

What makes them read, measured on that set (medians; ranges in the table above):

| | pixels at largest | distance from scene | life (cycles) |
|---|---|---|---|
| projectile | 212 (the arrow is 36; boss projectiles 73 to 741; one meteor 1,564) | 48 | set by the packet |
| impact | 770 | 44 | 35 |
| telegraph | 512 | 39; the classic shadow is 18 | 90 |
| the hit that follows | 1,432 | 39 | 84 |
| ground hazard | 717 | 46; pools 50 to 106 | 60 |

- A boss projectile is a fifth of a player's area and is told from an arrow by colour and translucency, not by size.
- A telegraph is smaller, flatter and quieter than the hit that follows it: about a third of the pixels, one or two
  colours, most of it translucent; it starts at almost nothing and GROWS over its life (the falling-rock shadow: one
  black disc, 264 faces, six alpha rings 130 to 230, scale only, 14 to 110 units across in 13 frames whose holds
  lengthen 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 9, 10, 13: fast then slow; ids exist for 60, 90, 120 ... 300 cycles, one per
  fall time). Its length IS the warning: 56 to 300 cycles, 2 to 10 game ticks.
- A hit is 30 to 84 cycles, starts at or near full size, and is three times the telegraph's area.
- A pool is opaque, one saturated hue in 4 tones, about one tile (130 units), 3 to 33 units high, and the most
  saturated thing on the floor (distance 90 to 106).
- An impact starts from nothing (first frame 0 to 6 pixels), reaches its largest in the last third, and ends by alpha.

## 4. Rules an author can build to

1. **One mesh, labelled parts, a small frame base of its own.** 50 to 500 faces for a projectile, 180 to 1,100 for an
   impact, 190 to 1,200 for a telegraph or hazard; a boss's signature effect may reach 2,000 to 3,000. Three or four
   colours cover 90% of the surface. Smooth shading is the default; flat facets are for crystal, metal and shards.
2. **Give every moving part a vertex label and every part that fades a face label.** A vertex transform acts on vertex
   labels; the alpha transform acts on FACE labels. A model with no vertex labels at all is not moved by any frame
   (only its alpha animates): our encoder must then write none (found on the first projectile ceiling: 15,802 pixels).
3. **Motion is scale, turn, move and fade of parts.** Grow from 10 to 30% to full size; scale about a pivot label at
   the effect's centre throws parts outward. End by fading (alpha +8 a step; 32 steps = gone), not by shrinking alone.
4. **Alpha values.** 0 opaque; 40 to 110 for the body of a glow; 130 to 230 for shadows, trails and outer shells.
   NEVER 254: the client draws alpha 254 by a special path (dark, shifted one pixel). 255 is not drawn. To bring a
   part IN, model it at 250 and key the alpha negative.
5. **Light.** Lit once at rest, from above, like an NPC. Glows use ambient 30 to 100 and contrast 60 to 100 (flatter
   and brighter); pools and shadows 0 / 0.
6. **Projectile.** Front is -z; centre the mesh on the origin (it is the flight point). 45 to 400 units long. Give it a
   looping sequence of 20 to 60 cycles (set loop frames = frame count): a spin, a pulse, a flickering trail. The server
   chooses heights (source 160 to 190 for a creature's hands, target 120 to 150 for a chest), slope 0 to 16, offset 64.
   Aim for 150 to 400 pixels at game size; an arrow is 36.
7. **Impact.** Centred on the origin; the packet lifts it (height 100 to 124 for a chest). 30 to 60 cycles, frame holds
   of 2 to 3. Starts from nothing.
8. **Telegraph.** On the floor at height 1 to 3 (a face at 0 loses to the ground), up-facing faces only. One or two
   colours, mostly translucent, lower contrast than the hit. It must show time passing: grow, fill or close in over
   its whole life, and its life is the warning in game ticks x 30 cycles.
9. **Hit.** Starts large and bright on the telegraph's tile at the cycle the telegraph ends. 30 to 84 cycles.
10. **Persistent hazard.** As a spot animation it must carry its whole life in one sequence (a long hold) or be sent
    again; anything that must block, be clicked, or stay indefinitely is an OBJECT with a sequence.
11. **Ids.** A spot id is two bytes in every packet; the model id may be four bytes in the definition (opcode 3) but
    must stay under 65,536 to be fetched from a server (LIVE.md).
12. **Reuse.** One mesh and sequence, recoloured (opcode 40) and rescaled (4, 5), is how the game makes a family.

## 5. The harness (`fx.py`, `client/rust-client/examples/asset_effect.rs`)

`asset_effect CACHE SCRIPT` loads a real map location with the client's map loader, puts NPCs and players into
`ClientState` by hand (as `live_world.rs`), starts effects as the three packets start them, runs the client's own
clocks cycle by cycle (`tick_actor_sequences`, `draw_projectiles_in_views`, `draw_root_graphics`) and writes any set
of cycles as pictures through `world_actors::Actors::triangles` and `Frame::draw_iter`. Effects are given BY ID (the
client's whole path) or AS FILES (`def:`: model file + definition text + loose sequence directory; the example
repeats the client's steps on the same structs, since `place` is private). No change to `src/`.

`fx.py sheet|gif OUT EFFECT.. --job spot|actor|nactor|proj [--close]`: contact sheets and moving pictures at the game's
timing, at game size (a fixed lens: a player 62 pixels tall) or 2.5 times closer, in Lumbridge's yard (a real floor).

Proof (`P fx.py faithful 1997 2008 2014 2016 3370 1446 1654 2007`; log `render/research-effects-faithful/faithful.log`):
eight real effects dumped to loose files with their own bytes and drawn through the file path against the same
effects by id: alone on a tile close, on a tile at game size, on a player, on an NPC, as a projectile; ten moments each:
**0 differing of 76,800,000 pixels over 400 frame pairs.** (First run: 26,058 pixels differed, all on a player: the
sequence height byte above.)

Limits: the file path poses classic sequences only (138 real effects are skeletal; they draw by id). Sound is not
played. The live frame's object traversal order is not repeated (as in `live_world`): several translucent effects
overlapping may sort differently from the live client; untested. Animated objects are not drawn by this harness.

## 6. The rebuild test (`fxtools.py`)

- `reencode ID DIR [degradations]`: a real effect through OUR encoders (model, sequence, frames, frame base,
  definition) as loose files; with nothing changed it is the ceiling.
- `sheet ID JOB DIR`: the reference sheet: six rows of pictures by id (in its job at game size, close, quarter turn,
  from behind; alone from above and from ground level), two moving pictures, and numbers (definition, counts, palette
  with shares, alpha values, rest box, per-frame hold, posed box and mean alpha, transform kinds, group types).
  No vertex and no face.
- `compare` / `gate ID OURS.def JOB`: static numbers from the files; motion from pictures at 10 moments, two sides,
  close and game size (mask overlap, coverage, tone); thresholds in the file's docstring, all provisional.
- `lineup OUT.gif JOB EFFECT..`: a shuffled moving line-up and its key file.

Proved (`proof/`, pictures `render/research-effects-proof/`):

| case | result |
|---|---|
| ceiling, 2008 nex_finale_attack_impact on a player | PASS; 0 differing of 2,160,000 pixels |
| ceiling, 1595 verzik_powerblast_safezone on a tile | PASS; 0 of 2,160,000 |
| ceiling, 2206 toa_wardens_prayer_ranged_travel as a projectile | PASS; 0 of 2,160,000 (FAILED first, 15,802 pixels: rule 2) |
| 2008 with every scale transform removed (same mesh, same rest pose) | FAIL: posed size 120 against 341, overlap 0.24, coverage 0.36, tone 23.6 |
| 2008 lightness +14 | FAIL on colour distance 3.5 (it PASSED before that check was added: tone alone, 7.0, does not catch it) |
| 2206 opaque | FAIL: translucent share 0 against 1.0, tone 95 |
| 2206 stretched 130% | FAIL: box, posed size, overlap 0.60, coverage 1.70 |
| 1595 at two thirds speed | FAIL: life 244 against 183, worst tone 41 |
| 1595 every vertex moved by up to 2 units | PASS (overlap 0.90, tone 5.3): below what the gate can see, and probably below what an eye can |
| 2206 hue +8 | PASS: its paint is saturation 0, where hue changes almost nothing |

## 7. Not proven, not built

- No effect has been rebuilt blind; the gate's thresholds have met only a ceiling and synthetic damage.
- Skeletal effects (138), textured effects (7), animated objects, and overlapping translucent effects in the live
  frame's order are outside the harness or the encoders.
- Nothing was seen in a logged-in client; no server sent a packet.
- The job table of section 2 rests on names; the study set of section 3 was checked by drawing.
